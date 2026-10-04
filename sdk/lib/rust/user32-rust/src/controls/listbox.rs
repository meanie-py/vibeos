use win32k_sys::win32k::*;
use core::ffi::c_void;

// Cross-control shared surface (HEADCOMBO, CBF_* flags, the undocumented
// LB_*_LOWER/UPPER + LB_CARETON/OFF messages). See controls/shared.rs.
use crate::controls::shared::*;
// Explicit single-name imports win over the two `*` glob imports above, resolving
// the LB_CARETON/LB_CARETOFF ambiguity (both reach this module via glob).
use win32k_sys::win32k::{LB_CARETOFF, LB_CARETON};
use crate::ntwrapper::GetControlColor;
// NtUser* syscall forwarders (ntwrapper.h EXTINLINE wrappers), not linkable exports.
use crate::ntwrapper::{
    BeginPaint, EndPaint, GetDCEx, InvalidateRect, SetCapture, SetFocus, SetSystemTimer,
    SetWindowPos, ShowScrollBar,
};
// SetWindowLongPtrW is a winuser.h macro for SetWindowLongW on x86; class.rs only
// defines a real (isize-typed) SetWindowLongPtrW for non-x86. cfg-split.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongPtrW(hWnd, nIndex, dwNewLong as isize) as LONG_PTR
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongW(hWnd, nIndex, dwNewLong as core::ffi::c_long) as LONG_PTR
}

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ─── Local consts the bindgen glob doesn't surface ────────────────────────────
const CP_ACP: UINT = 0;
const CSTR_LESS_THAN: INT = 1;
const CSTR_EQUAL: INT = 2;
const CSTR_GREATER_THAN: INT = 3;
const DRIVE_NO_ROOT_DIR: UINT = 1;
const NORM_IGNORECASE: DWORD = 0x0000_0001;
const LCID_INSTALLED: DWORD = 0x0000_0001;
const INVALID_HANDLE_VALUE: HANDLE = (-1isize) as HANDLE;
// IDC_ARROW = MAKEINTRESOURCEW(32512) (winuser.h)
const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;

// WIN32_FIND_DATAW (minwinbase.h) — bindgen doesn't surface it here.
#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [WCHAR; 260usize],
    pub cAlternateFileName: [WCHAR; 14usize],
}

// ─── Win32 / gdi32 / kernel32 / user32 public exports + a couple of unresolved
//     win32k internals. stdcall → extern "system" on i386 (`_Name@N`). ─────────
extern "system" {
    fn SetScrollInfo(hwnd: HWND, fnBar: INT, lpsi: *mut SCROLLINFO, redraw: INT) -> INT;

    fn SetLastError(dwErrCode: DWORD);
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: core::ffi::c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: core::ffi::c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: core::ffi::c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> core::ffi::c_int;
    fn MulDiv(nNumber: core::ffi::c_int, nNumerator: core::ffi::c_int, nDenominator: core::ffi::c_int) -> core::ffi::c_int;
    fn CharLowerW(lpsz: LPWSTR) -> LPWSTR;
    fn CompareStringW(
        Locale: LCID,
        dwCmpFlags: DWORD,
        lpString1: LPCWSTR,
        cchCount1: core::ffi::c_int,
        lpString2: LPCWSTR,
        cchCount2: core::ffi::c_int,
    ) -> core::ffi::c_int;

    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;

    fn GetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int) -> LONG_PTR;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn ClientToScreen(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    fn ReleaseCapture() -> BOOL;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> core::ffi::c_int;
    fn ScrollWindowEx(
        hWnd: HWND,
        dx: core::ffi::c_int,
        dy: core::ffi::c_int,
        prcScroll: *const RECT,
        prcClip: *const RECT,
        hrgnUpdate: HRGN,
        prcUpdate: *mut RECT,
        flags: UINT,
    ) -> core::ffi::c_int;
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    fn SetRect(lprc: *mut RECT, xLeft: core::ffi::c_int, yTop: core::ffi::c_int, xRight: core::ffi::c_int, yBottom: core::ffi::c_int) -> BOOL;
    fn KillSystemTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL;

    fn GetLastError() -> DWORD;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: usize) -> LPVOID;
    fn HeapSize(hHeap: HANDLE, dwFlags: DWORD, lpMem: *const c_void) -> usize;
    fn GetParent(hWnd: HWND) -> HWND;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn GetCapture() -> HWND;
    fn GetKeyState(nVirtKey: core::ffi::c_int) -> SHORT;
    fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn GetSysColor(nIndex: core::ffi::c_int) -> COLORREF;
    fn GetSysColorBrush(nIndex: core::ffi::c_int) -> HBRUSH;
    fn DragDetect(hwnd: HWND, pt: POINT) -> BOOL;
    fn GetScrollInfo(hwnd: HWND, fnBar: core::ffi::c_int, lpsi: *mut SCROLLINFO) -> BOOL;
    fn GetUserDefaultLCID() -> LCID;
    fn IsValidLocale(Locale: LCID, dwFlags: DWORD) -> BOOL;
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> UINT;
    fn FindFirstFileW(lpFileName: LPCWSTR, lpFindFileData: *mut WIN32_FIND_DATAW) -> HANDLE;
    fn FindNextFileW(hFindFile: HANDLE, lpFindFileData: *mut WIN32_FIND_DATAW) -> BOOL;
    fn FindClose(hFindFile: HANDLE) -> BOOL;

    // gdi32
    fn SelectObject(hdc: HDC, h: *mut c_void) -> *mut c_void;
    fn DeleteObject(ho: *mut c_void) -> BOOL;
    fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> core::ffi::c_int;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetWindowOrgEx(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int, lppt: *mut POINT) -> BOOL;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> core::ffi::c_int;
    fn ExtTextOutW(
        hdc: HDC,
        x: core::ffi::c_int,
        y: core::ffi::c_int,
        options: UINT,
        lprect: *const RECT,
        lpString: LPCWSTR,
        c: UINT,
        lpDx: *const core::ffi::c_int,
    ) -> BOOL;
    fn GetTextExtentPointA(hdc: HDC, lpString: LPCSTR, c: core::ffi::c_int, psizl: *mut SIZE) -> BOOL;
    fn GetTextExtentPoint32W(hdc: HDC, lpString: LPCWSTR, c: core::ffi::c_int, psizl: *mut SIZE) -> BOOL;
    fn GetCurrentObject(hdc: HDC, type_: UINT) -> *mut c_void;

    // user32 own-exports surviving as C (controls/edit.c) / siblings.
    fn TabbedTextOutW(
        hdc: HDC,
        x: core::ffi::c_int,
        y: core::ffi::c_int,
        lpString: LPCWSTR,
        chCount: core::ffi::c_int,
        nTabPositions: core::ffi::c_int,
        lpnTabStopPositions: *const core::ffi::c_int,
        nTabOrigin: core::ffi::c_int,
    ) -> LONG;
    fn DrawFocusRect(hDC: HDC, lprc: *const RECT) -> BOOL;

    // combo.c cross-TU (combo.rs exports it #[no_mangle] extern "system").
    fn COMBO_FlipListbox(lphc: LPHEADCOMBO, ok: BOOL, bRedrawButton: BOOL) -> BOOL;
}

// `set_control_clipping` — surviving C symbol in user32 (controls/uitools.c), cdecl.
extern "C" {
    fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(dst: *mut c_void, c: core::ffi::c_int, n: usize) -> *mut c_void;
}

// ─── No-op debug macros (ERR/WARN/TRACE/FIXME) ────────────────────────────────
// Consume all args (so any side-effect-bearing expression still type-checks)
// then discard — the C macros compile to nothing under the debug-channel guard.

#[inline]
unsafe fn debugstr_w(_s: LPCWSTR) -> *const i8 { b"\0".as_ptr() as *const i8 }
#[inline]
unsafe fn wine_dbgstr_rect(_r: *const RECT) -> *const i8 { b"\0".as_ptr() as *const i8 }

// ─── min / max / abs (the wine integer helpers) ───────────────────────────────
#[inline]
fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
#[inline]
fn abs(x: INT) -> INT { if x < 0 { -x } else { x } }

// ─── Wine wide-string helpers ─────────────────────────────────────────────────
#[inline]
unsafe fn strlenW(s: LPCWSTR) -> INT {
    let mut n: INT = 0;
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
#[inline]
unsafe fn strcmpW(s1: LPCWSTR, s2: LPCWSTR) -> INT {
    let mut i: isize = 0;
    loop {
        let a = *s1.offset(i);
        let b = *s2.offset(i);
        if a != b { return a as INT - b as INT; }
        if a == 0 { return 0; }
        i += 1;
    }
}
#[inline]
unsafe fn strncmpiW(s1: LPCWSTR, s2: LPCWSTR, mut n: INT) -> INT {
    let mut i: isize = 0;
    while n > 0 {
        let mut a = *s1.offset(i);
        let mut b = *s2.offset(i);
        if (b'A' as u16..=b'Z' as u16).contains(&a) { a += 32; }
        if (b'A' as u16..=b'Z' as u16).contains(&b) { b += 32; }
        if a != b { return a as INT - b as INT; }
        if a == 0 { return 0; }
        i += 1;
        n -= 1;
    }
    0
}
#[inline]
unsafe fn strlwrW(s: LPWSTR) -> LPWSTR { CharLowerW(s) }
#[inline]
unsafe fn struprW(s: LPWSTR) -> LPWSTR {
    let mut i: isize = 0;
    loop {
        let c = *s.offset(i);
        if c == 0 { break; }
        if (b'a' as u16..=b'z' as u16).contains(&c) { *s.offset(i) = c - 32; }
        i += 1;
    }
    s
}

// ─── listbox.c macros as inline fns (return the C int value; call sites use
//     both `!= 0`/`== 0` and bool context, so return UINT and let callers cast).
#[inline]
unsafe fn HAS_STRINGS(descr: *const LB_DESCR) -> BOOL {
    (IS_OWNERDRAW(descr) == 0 || (*descr).style & LBS_HASSTRINGS != 0) as BOOL
}
#[inline]
unsafe fn IS_MULTISELECT(descr: *const LB_DESCR) -> UINT {
    ((*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL) != 0
        && (*descr).style & LBS_NOSEL == 0) as UINT
}
#[inline]
unsafe fn ISWIN31() -> bool {
    crate::internal::LOWORD(GetVersion()) == 0x0a03
}
#[inline]
unsafe fn SEND_NOTIFICATION(descr: *const LB_DESCR, code: INT) -> LRESULT {
    SendMessageW(
        (*descr).owner,
        WM_COMMAND,
        crate::internal::MAKEWPARAM(
            GetWindowLongPtrW((*descr).self_, GWLP_ID) as u16,
            code as u16,
        ) as WPARAM,
        (*descr).self_ as LPARAM,
    )
}

extern "system" {
    fn GetVersion() -> DWORD;
}

// ---- file-scope preamble (translated) ----
pub const LB_ARRAY_GRANULARITY: INT = 16;

pub const LB_SCROLL_TIMEOUT: INT = 50;

pub const LB_TIMER_ID: INT = 2;

pub const LBS_DISPLAYCHANGED: u32 = 0x80000000;

#[repr(C)]
pub struct LB_ITEMDATA {
    pub str_: LPWSTR,
    pub selected: BOOL,
    pub height: UINT,
    pub data: ULONG_PTR,
}

#[repr(C)]
pub struct LB_DESCR {
    pub self_: HWND,
    pub owner: HWND,
    pub style: UINT,
    pub width: INT,
    pub height: INT,
    pub items: *mut LB_ITEMDATA,
    pub nb_items: INT,
    pub top_item: INT,
    pub selected_item: INT,
    pub focus_item: INT,
    pub anchor_item: INT,
    pub item_height: INT,
    pub page_size: INT,
    pub column_width: INT,
    pub horz_extent: INT,
    pub horz_pos: INT,
    pub nb_tabs: INT,
    pub tabs: *mut INT,
    pub avg_char_width: INT,
    pub wheel_remain: INT,
    pub caret_on: BOOL,
    pub captured: BOOL,
    pub in_focus: BOOL,
    pub font: HFONT,
    pub locale: LCID,
    pub lphc: LPHEADCOMBO,
    pub UIState: LONG,
}

pub const LB_TIMER_NONE: TIMER_DIRECTION = 0;
pub const LB_TIMER_UP: TIMER_DIRECTION = 1;
pub const LB_TIMER_LEFT: TIMER_DIRECTION = 2;
pub const LB_TIMER_DOWN: TIMER_DIRECTION = 3;
pub const LB_TIMER_RIGHT: TIMER_DIRECTION = 4;
pub type TIMER_DIRECTION = u32;

pub static mut LISTBOX_Timer: TIMER_DIRECTION = LB_TIMER_NONE;

pub static listboxW: [WCHAR; 8] = [
    'L' as WCHAR, 'i' as WCHAR, 's' as WCHAR, 't' as WCHAR,
    'B' as WCHAR, 'o' as WCHAR, 'x' as WCHAR, 0,
];

#[repr(C)]
pub struct builtin_class_descr {
    pub name: *const WCHAR,
    pub style: UINT,
    pub procA: WNDPROC,
    pub procW: WNDPROC,
    pub extra: INT,
    pub cursor: LPCWSTR,
    pub brush: HBRUSH,
}

// The builtin_class_descr table holds raw pointers (name/cursor/brush) and the
// two wndproc fn pointers — it is read-only init data, safe to share.
unsafe impl Sync for builtin_class_descr {}

// ListBoxWndProcA/W are #[no_mangle] extern "system" (the required export ABI).
// `WNDPROC` is spelled extern "stdcall" on i386 / extern "C" on amd64+arm64 by
// bindgen; the calling convention is identical on each arch, so reinterpret the
// fn pointer into the bindgen WNDPROC type.
type LbWndProcFn = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
const LB_WNDPROC_A: WNDPROC =
    unsafe { core::mem::transmute::<LbWndProcFn, WNDPROC>(ListBoxWndProcA) };
const LB_WNDPROC_W: WNDPROC =
    unsafe { core::mem::transmute::<LbWndProcFn, WNDPROC>(ListBoxWndProcW) };

#[no_mangle]
pub static LISTBOX_builtin_class: builtin_class_descr = builtin_class_descr {
    name: listboxW.as_ptr(),
    style: CS_DBLCLKS,
    procA: LB_WNDPROC_A,
    procW: LB_WNDPROC_W,
    extra: core::mem::size_of::<*mut LB_DESCR>() as INT,
    cursor: IDC_ARROW,
    brush: 0 as HBRUSH,
};

pub static combolboxW: [WCHAR; 10] = [
    'C' as WCHAR, 'o' as WCHAR, 'm' as WCHAR, 'b' as WCHAR,
    'o' as WCHAR, 'L' as WCHAR, 'B' as WCHAR, 'o' as WCHAR,
    'x' as WCHAR,
 0,];

#[no_mangle]
pub static COMBOLBOX_builtin_class: builtin_class_descr = builtin_class_descr {
    name: combolboxW.as_ptr(),
    style: CS_DBLCLKS | CS_SAVEBITS,
    procA: LB_WNDPROC_A,
    procW: LB_WNDPROC_W,
    extra: core::mem::size_of::<*mut LB_DESCR>() as INT,
    cursor: IDC_ARROW,
    brush: 0 as HBRUSH,
};

unsafe fn LISTBOX_GetCurrentPageSize(descr: *const LB_DESCR) -> INT {
    let mut i: INT;
    let mut height: INT;
    if ((*descr).style & LBS_OWNERDRAWVARIABLE) == 0 {
        return (*descr).page_size;
    }
    i = (*descr).top_item;
    height = 0;
    while i < (*descr).nb_items {
        height += (*(*descr).items.offset(i as isize)).height as INT;
        if height > (*descr).height {
            break;
        }
        i += 1;
    }
    if i == (*descr).top_item {
        1
    } else {
        i - (*descr).top_item
    }
}

unsafe fn LISTBOX_GetMaxTopIndex(descr: *const LB_DESCR) -> INT {
    let mut max: INT;
    let mut page: INT;

    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        page = (*descr).height;
        max = (*descr).nb_items - 1;
        while max >= 0 {
            page -= (*(*descr).items.offset(max as isize)).height as INT;
            if page < 0 {
                break;
            }
            max -= 1;
        }
        if max < (*descr).nb_items - 1 {
            max += 1;
        }
    } else if (*descr).style & LBS_MULTICOLUMN != 0 {
        page = (*descr).width / (*descr).column_width;
        if page < 1 {
            page = 1;
        }
        max = ((*descr).nb_items + (*descr).page_size - 1) / (*descr).page_size;
        max = (max - page) * (*descr).page_size;
    } else {
        max = (*descr).nb_items - (*descr).page_size;
    }
    if max < 0 {
        max = 0;
    }
    max
}

unsafe fn LISTBOX_UpdateScroll(descr: *mut LB_DESCR) {
    let mut info: SCROLLINFO = core::mem::zeroed();

    if (*descr).style & LBS_NOREDRAW != 0 {
        return;
    }
    info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;

    if (*descr).style & LBS_MULTICOLUMN != 0 {
        info.nMin = 0;
        info.nMax = ((*descr).nb_items - 1) / (*descr).page_size;
        info.nPos = (*descr).top_item / (*descr).page_size;
        info.nPage = ((*descr).width / (*descr).column_width) as u32;
        if info.nPage < 1 {
            info.nPage = 1;
        }
        info.fMask = SIF_RANGE | SIF_POS | SIF_PAGE;
        if (*descr).style & LBS_DISABLENOSCROLL != 0 {
            info.fMask |= SIF_DISABLENOSCROLL;
        }
        if (*descr).style & WS_HSCROLL != 0 {
            SetScrollInfo((*descr).self_, SB_HORZ as i32, &mut info, 1u8 as i32);
        }
        info.nMax = 0;
        info.fMask = SIF_RANGE;
        if (*descr).style & WS_VSCROLL != 0 {
            SetScrollInfo((*descr).self_, SB_VERT as i32, &mut info, 1u8 as i32);
        }
    } else {
        info.nMin = 0;
        info.nMax = (*descr).nb_items - 1;
        info.nPos = (*descr).top_item;
        info.nPage = LISTBOX_GetCurrentPageSize(descr) as u32;
        info.fMask = SIF_RANGE | SIF_POS | SIF_PAGE;
        if (*descr).style & LBS_DISABLENOSCROLL != 0 {
            info.fMask |= SIF_DISABLENOSCROLL;
        }
        if (*descr).style & WS_VSCROLL != 0 {
            SetScrollInfo((*descr).self_, SB_VERT as i32, &mut info, 1u8 as i32);
        }

        if (*descr).style & WS_HSCROLL != 0 && (*descr).horz_extent != 0 {
            info.nPos = (*descr).horz_pos;
            info.nPage = (*descr).width as u32;
            info.fMask = SIF_POS | SIF_PAGE;
            if (*descr).style & LBS_DISABLENOSCROLL != 0 {
                info.fMask |= SIF_DISABLENOSCROLL;
            }
            SetScrollInfo((*descr).self_, SB_HORZ as i32, &mut info, 1u8 as i32);
        } else {
            if (*descr).style & LBS_DISABLENOSCROLL != 0 {
                info.nMin = 0;
                info.nMax = 0;
                info.fMask = SIF_RANGE | SIF_DISABLENOSCROLL;
                SetScrollInfo((*descr).self_, SB_HORZ as i32, &mut info, 1u8 as i32);
            } else {
                ShowScrollBar((*descr).self_, SB_HORZ as i32, 0u8 as i32);
            }
        }
    }
}

unsafe fn LISTBOX_SetTopItem(descr: *mut LB_DESCR, mut index: INT, scroll: BOOL) -> LRESULT {
    let max = LISTBOX_GetMaxTopIndex(descr);

    TRACE!(b"setting top item %d, scroll %d\n\0".as_ptr() as *const i8, index, scroll);

    if index > max { index = max; }
    if index < 0 { index = 0; }
    if (*descr).style & LBS_MULTICOLUMN != 0 { index -= index % (*descr).page_size; }
    if (*descr).top_item == index { return LB_OKAY as LRESULT; }
    if scroll != 0 {
        let diff: INT;
        if (*descr).style & LBS_MULTICOLUMN != 0 {
            diff = ((*descr).top_item - index) / (*descr).page_size * (*descr).column_width;
        } else if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
            let mut d: INT = 0;
            if index > (*descr).top_item {
                let mut i = index - 1;
                while i >= (*descr).top_item {
                    d -= (*(*descr).items.offset(i as isize)).height as INT;
                    i -= 1;
                }
            } else {
                let mut i = index;
                while i < (*descr).top_item {
                    d += (*(*descr).items.offset(i as isize)).height as INT;
                    i += 1;
                }
            }
            diff = d;
        } else {
            diff = ((*descr).top_item - index) * (*descr).item_height;
        }

        if (*descr).style & LBS_MULTICOLUMN != 0 {
            ScrollWindowEx((*descr).self_, diff, 0, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                           SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN);
        } else {
            ScrollWindowEx((*descr).self_, 0, diff, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                           SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN);
        }
    } else {
        InvalidateRect((*descr).self_, core::ptr::null_mut(), 1u8 as i32);
    }
    (*descr).top_item = index;
    LISTBOX_UpdateScroll(descr);
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_UpdatePage(descr: *mut LB_DESCR) {
    let mut page_size: INT;

    if (*descr).item_height == 0 {
        page_size = 1;
    } else {
        page_size = (*descr).height / (*descr).item_height;
        if page_size < 1 {
            page_size = 1;
        }
    }
    if page_size == (*descr).page_size {
        return;
    }
    (*descr).page_size = page_size;
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
    }
    LISTBOX_SetTopItem(descr, (*descr).top_item, 0);
}

unsafe fn LISTBOX_UpdateSize(descr: *mut LB_DESCR) {
    let mut rect: RECT = core::mem::zeroed();
    let style: LONG = GetWindowLongPtrW((*descr).self_, GWL_STYLE as i32) as LONG;

    GetClientRect((*descr).self_, &mut rect);
    if style & (WS_HSCROLL as LONG) != 0 {
        rect.bottom += GetSystemMetrics(SM_CYHSCROLL as i32);
    }
    (*descr).width = rect.right - rect.left;
    (*descr).height = rect.bottom - rect.top;
    if ((*descr).style & LBS_NOINTEGRALHEIGHT) == 0 && ((*descr).style & LBS_OWNERDRAWVARIABLE) == 0 {
        let remaining: INT;
        let mut rect2: RECT = core::mem::zeroed();

        GetWindowRect((*descr).self_, &mut rect2);
        if (*descr).item_height != 0 {
            remaining = (*descr).height % (*descr).item_height;
        } else {
            remaining = 0;
        }
        if ((*descr).height > (*descr).item_height) && remaining != 0 {
            SetWindowPos(
                (*descr).self_,
                core::ptr::null_mut(),
                0,
                0,
                rect2.right - rect2.left,
                rect2.bottom - rect2.top - remaining,
                (SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE) as u32,
            );
            return;
        }
    }
    LISTBOX_UpdatePage(descr);
    LISTBOX_UpdateScroll(descr);

    if LISTBOX_GetItemRect(descr, (*descr).focus_item, &mut rect) == 1 {
        InvalidateRect((*descr).self_, &rect, 0u8 as i32);
    }
}

unsafe fn LISTBOX_GetItemRect(descr: *const LB_DESCR, index: INT, rect: *mut RECT) -> LRESULT {
    if index != 0 && (index >= (*descr).nb_items) {
        memset(rect as *mut core::ffi::c_void, 0, core::mem::size_of::<RECT>());
        SetLastError(ERROR_INVALID_INDEX);
        return LB_ERR as LRESULT;
    }
    SetRect(rect, 0, 0, (*descr).width, (*descr).height);
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        let col = (index / (*descr).page_size) - ((*descr).top_item / (*descr).page_size);
        (*rect).left += col * (*descr).column_width;
        (*rect).right = (*rect).left + (*descr).column_width;
        (*rect).top += (index % (*descr).page_size) * (*descr).item_height;
        (*rect).bottom = (*rect).top + (*descr).item_height;
    } else if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        let mut i: INT;
        (*rect).right += (*descr).horz_pos;
        if index >= 0 && index < (*descr).nb_items {
            if index < (*descr).top_item {
                i = (*descr).top_item - 1;
                while i >= index {
                    (*rect).top -= (*(*descr).items.offset(i as isize)).height as INT;
                    i -= 1;
                }
            } else {
                i = (*descr).top_item;
                while i < index {
                    (*rect).top += (*(*descr).items.offset(i as isize)).height as INT;
                    i += 1;
                }
            }
            (*rect).bottom = (*rect).top + (*(*descr).items.offset(index as isize)).height as INT;
        }
    } else {
        (*rect).top += (index - (*descr).top_item) * (*descr).item_height;
        (*rect).bottom = (*rect).top + (*descr).item_height;
        (*rect).right += (*descr).horz_pos;
    }

    TRACE!(b"item %d, rect %s\n\0".as_ptr() as *const i8, index, wine_dbgstr_rect(rect));

    (((*rect).left < (*descr).width) && ((*rect).right > 0) &&
        ((*rect).top < (*descr).height) && ((*rect).bottom > 0)) as LRESULT
}

unsafe fn LISTBOX_GetItemFromPoint( descr: *const LB_DESCR, x: INT, y: INT ) -> INT
{
    let mut index: INT = (*descr).top_item;

    if (*descr).nb_items == 0 { return -1; }
    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0
    {
        let mut pos: INT = 0;
        if y >= 0
        {
            while index < (*descr).nb_items
            {
                pos += (*(*descr).items.offset(index as isize)).height as INT;
                if pos > y { break; }
                index += 1;
            }
        }
        else
        {
            while index > 0
            {
                index -= 1;
                pos -= (*(*descr).items.offset(index as isize)).height as INT;
                if pos <= y { break; }
            }
        }
    }
    else if (*descr).style & LBS_MULTICOLUMN != 0
    {
        if y >= (*descr).item_height * (*descr).page_size { return -1; }
        if y >= 0 { index += y / (*descr).item_height; }
        if x >= 0 { index += (x / (*descr).column_width) * (*descr).page_size; }
        else { index -= (((x + 1) / (*descr).column_width) - 1) * (*descr).page_size; }
    }
    else
    {
        index += y / (*descr).item_height;
    }
    if index < 0 { return 0; }
    if index >= (*descr).nb_items { return -1; }
    index
}

unsafe fn LISTBOX_PaintItem(
    descr: *mut LB_DESCR,
    hdc: HDC,
    rect: *const RECT,
    index: INT,
    action: UINT,
    ignoreFocus: BOOL,
) {
    let mut item: *mut LB_ITEMDATA = core::ptr::null_mut();
    if index < (*descr).nb_items {
        item = &mut *(*descr).items.offset(index as isize);
    }

    if ((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) != 0 {
        let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
        let mut r: RECT = core::mem::zeroed();
        let hrgn: HRGN;

        if item.is_null() {
            if action == ODA_FOCUS {
                if ((*descr).UIState & UISF_HIDEFOCUS as LONG) == 0 {
                    DrawFocusRect(hdc, rect);
                }
            } else {
                ERR!(
                    "called with an out of bounds index %d(%d) in owner draw, Not good.\n",
                    index,
                    (*descr).nb_items,
                );
            }
            return;
        }

        GetClientRect((*descr).self_, &mut r);
        hrgn = set_control_clipping(hdc, &mut r);

        dis.CtlType = ODT_LISTBOX as UINT;
        dis.CtlID = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;
        dis.hwndItem = (*descr).self_;
        dis.itemAction = action;
        dis.hDC = hdc;
        dis.itemID = index as UINT;
        dis.itemState = 0;
        if (*item).selected != 0 {
            dis.itemState |= ODS_SELECTED as UINT;
        }
        if ignoreFocus == 0
            && (*descr).focus_item == index
            && (*descr).caret_on != 0
            && (*descr).in_focus != 0
        {
            dis.itemState |= ODS_FOCUS as UINT;
        }
        if IsWindowEnabled((*descr).self_) == 0 {
            dis.itemState |= ODS_DISABLED as UINT;
        }
        dis.itemData = (*item).data;
        // DRAWITEMSTRUCT.rcItem is RECTL; `rect` is the layout-identical tagRECT.
        dis.rcItem = *(rect as *const RECTL);
        TRACE!(
            b"[%p]: drawitem %d (%s) action=%02x state=%02x rect=%s\n\0".as_ptr() as PCHAR,
            (*descr).self_,
            index,
            debugstr_w((*item).str_),
            action,
            dis.itemState,
            wine_dbgstr_rect(rect),
        );
        SendMessageW(
            (*descr).owner,
            WM_DRAWITEM,
            dis.CtlID as WPARAM,
            &mut dis as *mut DRAWITEMSTRUCT as LPARAM,
        );
        SelectClipRgn(hdc, hrgn);
        if !hrgn.is_null() {
            DeleteObject(hrgn as *mut core::ffi::c_void);
        }
    } else {
        let mut oldText: COLORREF = 0;
        let mut oldBk: COLORREF = 0;

        if action == ODA_FOCUS {
            if ((*descr).UIState & UISF_HIDEFOCUS as LONG) == 0 {
                DrawFocusRect(hdc, rect);
            }
            return;
        }
        if !item.is_null() && (*item).selected != 0 {
            oldBk = SetBkColor(hdc, GetSysColor(COLOR_HIGHLIGHT as INT));
            oldText = SetTextColor(hdc, GetSysColor(COLOR_HIGHLIGHTTEXT as INT));
        } else {
            /* Upstream 16a65a0df29 (CORE-20433): paint the background with the brush
             * WM_CTLCOLORLISTBOX selected into the DC, not the ETO_OPAQUE bk colour. */
            let br = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
            FillRect(hdc, rect, br);
        }

        TRACE!(
            b"[%p]: painting %d (%s) action=%02x rect=%s\n\0".as_ptr() as PCHAR,
            (*descr).self_,
            index,
            if !item.is_null() {
                debugstr_w((*item).str_)
            } else {
                b"\0".as_ptr() as PCHAR
            },
            action,
            wine_dbgstr_rect(rect),
        );
        if item.is_null() {
            ExtTextOutW(
                hdc,
                (*rect).left + 1,
                (*rect).top,
                ETO_OPAQUE | ETO_CLIPPED,
                rect,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
        } else if ((*descr).style & LBS_USETABSTOPS) == 0 {
            /* Upstream 16a65a0df29: for an unselected item, limit the opaque box to the
             * text so the brush fill above shows through the rest of the row. The C
             * clamps `rc.right` to the bare extent `sz.cx`, a width used as a
             * coordinate: in any column but the first (rect->left > 0) that makes
             * rc.right < rc.left, and ETO_CLIPPED then clips the whole string away.
             * Clamp to where the text actually ends instead. */
            let mut rc: RECT = *rect;
            if (*item).selected == 0 {
                let mut sz: SIZE = core::mem::zeroed();
                GetTextExtentPoint32W(hdc, (*item).str_, strlenW((*item).str_) as INT, &mut sz);
                rc.right = core::cmp::min((*rect).left + 1 + sz.cx, rc.right);
            }
            ExtTextOutW(
                hdc,
                (*rect).left + 1,
                (*rect).top,
                ETO_OPAQUE | ETO_CLIPPED,
                &rc,
                (*item).str_,
                strlenW((*item).str_) as UINT,
                core::ptr::null_mut(),
            );
        } else {
            ExtTextOutW(
                hdc,
                (*rect).left + 1,
                (*rect).top,
                ETO_OPAQUE | ETO_CLIPPED,
                rect,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            TabbedTextOutW(
                hdc,
                (*rect).left + 1,
                (*rect).top,
                (*item).str_,
                strlenW((*item).str_),
                (*descr).nb_tabs,
                (*descr).tabs,
                0,
            );
        }
        if !item.is_null() && (*item).selected != 0 {
            SetBkColor(hdc, oldBk);
            SetTextColor(hdc, oldText);
        }
        if ignoreFocus == 0
            && (*descr).focus_item == index
            && (*descr).caret_on != 0
            && (*descr).in_focus != 0
            && ((*descr).UIState & UISF_HIDEFOCUS as LONG) == 0
        {
            DrawFocusRect(hdc, rect);
        }
    }
}

unsafe fn LISTBOX_SetRedraw(descr: *mut LB_DESCR, on: BOOL) {
    if on != 0 {
        if ((*descr).style & LBS_NOREDRAW) == 0 {
            return;
        }
        (*descr).style &= !LBS_NOREDRAW;
        if ((*descr).style & LBS_DISPLAYCHANGED) != 0 {
            InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
            if ((*descr).top_item + (*descr).page_size) > (*descr).nb_items {
                (*descr).top_item = (*descr).nb_items - (*descr).page_size;
                if (*descr).top_item < 0 {
                    (*descr).top_item = 0;
                }
            }
            (*descr).style &= !LBS_DISPLAYCHANGED;
        }
        LISTBOX_UpdateScroll(descr);
    } else {
        (*descr).style |= LBS_NOREDRAW;
    }
}

unsafe fn LISTBOX_RepaintItem(descr: *mut LB_DESCR, index: INT, action: UINT) {
    let hdc: HDC;
    let mut rect: RECT = core::mem::zeroed();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let hbrush: HBRUSH;
    let mut oldBrush: HBRUSH = core::ptr::null_mut();

    if IsWindowVisible((*descr).self_) == 0 {
        return;
    }
    if (*descr).style & LBS_NOREDRAW != 0 {
        (*descr).style |= LBS_DISPLAYCHANGED;
        return;
    }
    if LISTBOX_GetItemRect(descr, index, &mut rect) != 1 {
        return;
    }
    hdc = GetDCEx((*descr).self_, core::ptr::null_mut(), DCX_CACHE);
    if hdc.is_null() {
        return;
    }
    if !(*descr).font.is_null() {
        oldFont = SelectObject(hdc, (*descr).font as *mut core::ffi::c_void) as HFONT;
    }
    hbrush = GetControlColor((*descr).owner, (*descr).self_, hdc, WM_CTLCOLORLISTBOX);
    if !hbrush.is_null() {
        oldBrush = SelectObject(hdc, hbrush as *mut core::ffi::c_void) as HBRUSH;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(hdc, GetSysColor(COLOR_GRAYTEXT as i32));
    }
    SetWindowOrgEx(hdc, (*descr).horz_pos, 0, core::ptr::null_mut());
    LISTBOX_PaintItem(descr, hdc, &rect, index, action, 1);
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as *mut core::ffi::c_void);
    }
    if !oldBrush.is_null() {
        SelectObject(hdc, oldBrush as *mut core::ffi::c_void);
    }
    ReleaseDC((*descr).self_, hdc);
}

unsafe fn LISTBOX_DrawFocusRect(descr: *mut LB_DESCR, on: BOOL) {
    let hdc: HDC;
    let mut rect: RECT = core::mem::zeroed();
    let mut oldFont: HFONT = 0 as HFONT;

    if IsWindowVisible((*descr).self_) == 0 {
        return;
    }

    if (*descr).focus_item == -1 {
        return;
    }
    if (*descr).caret_on == 0 || (*descr).in_focus == 0 {
        return;
    }

    if LISTBOX_GetItemRect(descr, (*descr).focus_item, &mut rect) != 1 {
        return;
    }
    hdc = GetDCEx((*descr).self_, 0 as HRGN, DCX_CACHE);
    if hdc.is_null() {
        return;
    }
    if !(*descr).font.is_null() {
        oldFont = SelectObject(hdc, (*descr).font as *mut core::ffi::c_void) as HFONT;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(hdc, GetSysColor(COLOR_GRAYTEXT as i32));
    }
    SetWindowOrgEx(hdc, (*descr).horz_pos, 0, core::ptr::null_mut());
    LISTBOX_PaintItem(descr, hdc, &rect, (*descr).focus_item, ODA_FOCUS, if on != 0 { 0 } else { 1 });
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as *mut core::ffi::c_void);
    }
    ReleaseDC((*descr).self_, hdc);
}

unsafe fn LISTBOX_InitStorage(descr: *mut LB_DESCR, mut nb_items: INT) -> LRESULT {
    let item: *mut LB_ITEMDATA;

    nb_items += LB_ARRAY_GRANULARITY - 1;
    nb_items -= nb_items % LB_ARRAY_GRANULARITY;
    if !(*descr).items.is_null() {
        nb_items += (HeapSize(GetProcessHeap(), 0, (*descr).items as *mut _)
            / core::mem::size_of::<LB_ITEMDATA>()) as INT;
        item = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*descr).items as *mut _,
            nb_items as usize * core::mem::size_of::<LB_ITEMDATA>(),
        ) as *mut LB_ITEMDATA;
    } else {
        item = HeapAlloc(
            GetProcessHeap(),
            0,
            nb_items as usize * core::mem::size_of::<LB_ITEMDATA>(),
        ) as *mut LB_ITEMDATA;
    }

    if item.is_null() {
        SendMessageW(
            (*descr).owner,
            WM_COMMAND,
            crate::internal::MAKEWPARAM(
                GetWindowLongPtrW((*descr).self_, GWLP_ID) as u16,
                LBN_ERRSPACE as u16,
            ) as WPARAM,
            (*descr).self_ as LPARAM,
        );
        return LB_ERRSPACE as LRESULT;
    }
    (*descr).items = item;
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SetTabStops(descr: *mut LB_DESCR, count: INT, tabs: LPINT) -> BOOL {
    let mut i: INT;

    if ((*descr).style & LBS_USETABSTOPS) == 0 {
        SetLastError(ERROR_LB_WITHOUT_TABSTOPS);
        return 0;
    }

    HeapFree(GetProcessHeap(), 0, (*descr).tabs as *mut core::ffi::c_void);
    (*descr).nb_tabs = count;
    if (*descr).nb_tabs == 0 {
        (*descr).tabs = core::ptr::null_mut();
        return 1;
    }

    (*descr).tabs = HeapAlloc(
        GetProcessHeap(),
        0,
        ((*descr).nb_tabs as usize * core::mem::size_of::<INT>()) as _,
    ) as LPINT;
    if (*descr).tabs.is_null() {
        return 0;
    }

    memcpy(
        (*descr).tabs as *mut core::ffi::c_void,
        tabs as *const core::ffi::c_void,
        ((*descr).nb_tabs as usize * core::mem::size_of::<INT>()) as _,
    );

    i = 0;
    while i < (*descr).nb_tabs {
        *(*descr).tabs.offset(i as isize) =
            MulDiv(*(*descr).tabs.offset(i as isize), (*descr).avg_char_width, 4);
        i += 1;
    }

    1
}

unsafe fn LISTBOX_GetText( descr: *mut LB_DESCR, index: INT, buffer: LPWSTR, unicode: BOOL ) -> LRESULT {
    let len: DWORD;

    if (index < 0) || (index >= (*descr).nb_items) {
        SetLastError(ERROR_INVALID_INDEX);
        return LB_ERR as LRESULT;
    }
    if IS_OWNERDRAW(descr) == 0 || ((*descr).style & LBS_HASSTRINGS != 0) {
        if buffer.is_null() {
            let l = strlenW((*(*descr).items.offset(index as isize)).str_);
            if unicode != 0 {
                return l as LRESULT;
            }
            return WideCharToMultiByte( CP_ACP, 0, (*(*descr).items.offset(index as isize)).str_, l as i32,
                                        core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut() ) as LRESULT;
        }

        if unicode != 0 {
            strcpyW( buffer, (*(*descr).items.offset(index as isize)).str_ );
            len = strlenW(buffer) as DWORD;
        } else {
            len = (WideCharToMultiByte(CP_ACP, 0, (*(*descr).items.offset(index as isize)).str_, -1,
                                      buffer as LPSTR, 0x7FFFFFFF, core::ptr::null_mut(), core::ptr::null_mut()) - 1) as DWORD;
        }
    } else {
        if !buffer.is_null() {
            *(buffer as LPDWORD) = *(core::ptr::addr_of!((*(*descr).items.offset(index as isize)).data) as *const DWORD);
        }
        len = core::mem::size_of::<DWORD>() as DWORD;
    }
    len as LRESULT
}

#[inline]
unsafe fn IS_OWNERDRAW(descr: *const LB_DESCR) -> UINT {
    (*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)
}

unsafe fn LISTBOX_lstrcmpiW(lcid: LCID, str1: LPCWSTR, str2: LPCWSTR) -> INT {
    let ret = CompareStringW(lcid, NORM_IGNORECASE, str1, -1, str2, -1);
    if ret == CSTR_LESS_THAN {
        return -1;
    }
    if ret == CSTR_EQUAL {
        return 0;
    }
    if ret == CSTR_GREATER_THAN {
        return 1;
    }
    -1
}

unsafe fn LISTBOX_FindStringPos(descr: *mut LB_DESCR, str: LPCWSTR, exact: BOOL) -> INT {
    let mut index: INT;
    let mut min: INT;
    let mut max: INT;
    let mut res: INT;

    if ((*descr).style & LBS_SORT) == 0 {
        return -1;
    }
    min = 0;
    max = (*descr).nb_items;
    while min != max {
        index = (min + max) / 2;
        if IS_OWNERDRAW(descr) == 0 || ((*descr).style & LBS_HASSTRINGS) != 0 {
            res = LISTBOX_lstrcmpiW((*descr).locale, str, (*(*descr).items.offset(index as isize)).str_);
        } else {
            let mut cis: COMPAREITEMSTRUCT = core::mem::zeroed();
            let id = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;

            cis.CtlType = ODT_LISTBOX;
            cis.CtlID = id;
            cis.hwndItem = (*descr).self_;
            cis.itemID1 = -1i32 as UINT;
            cis.itemData1 = str as ULONG_PTR;
            cis.itemID2 = index as UINT;
            cis.itemData2 = (*(*descr).items.offset(index as isize)).data;
            cis.dwLocaleId = (*descr).locale;
            res = SendMessageW((*descr).owner, WM_COMPAREITEM, id as WPARAM, core::ptr::addr_of_mut!(cis) as LPARAM) as INT;
        }
        if res == 0 {
            return index;
        }
        if res < 0 {
            max = index;
        } else {
            min = index + 1;
        }
    }
    if exact != 0 { -1 } else { max }
}

unsafe fn LISTBOX_FindFileStrPos(descr: *mut LB_DESCR, str: LPCWSTR) -> INT {
    let mut min: INT;
    let mut max: INT;
    let mut res: INT;

    if !(!(((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) != 0)
        || (((*descr).style & LBS_HASSTRINGS) != 0))
    {
        return LISTBOX_FindStringPos(descr, str, 0);
    }
    min = 0;
    max = (*descr).nb_items;
    while min != max {
        let index = (min + max) / 2;
        let p: LPCWSTR = (*(*descr).items.offset(index as isize)).str_;
        if *p == '[' as u16 {
            if *str != '[' as u16 {
                res = -1;
            } else if *p.offset(1) == '-' as u16 {
                if *str.offset(1) == '-' as u16 {
                    res = *str.offset(2) as INT - *p.offset(2) as INT;
                } else {
                    res = -1;
                }
            } else {
                if *str.offset(1) == '-' as u16 {
                    res = 1;
                } else {
                    res = LISTBOX_lstrcmpiW((*descr).locale, str, p);
                }
            }
        } else {
            if *str == '[' as u16 {
                res = 1;
            } else {
                res = LISTBOX_lstrcmpiW((*descr).locale, str, p);
            }
        }
        if res == 0 {
            return index;
        }
        if res < 0 {
            max = index;
        } else {
            min = index + 1;
        }
    }
    max
}

unsafe fn LISTBOX_FindString(descr: *mut LB_DESCR, mut start: INT, str: LPCWSTR, exact: BOOL) -> INT {
    let mut i: INT;
    let mut item: *mut LB_ITEMDATA;

    if start >= (*descr).nb_items {
        start = -1;
    }
    item = (*descr).items.offset((start + 1) as isize);

    if !(IS_OWNERDRAW(descr) == 0) == false || ((*descr).style & LBS_HASSTRINGS) != 0 {
    }

    let has_strings = (((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) == 0)
        || ((*descr).style & LBS_HASSTRINGS) != 0;

    if has_strings {
        if str.is_null() || *str == 0 {
            return LB_ERR as INT;
        }
        if exact != 0 {
            i = start + 1;
            while i < (*descr).nb_items {
                if LISTBOX_lstrcmpiW((*descr).locale, str, (*item).str_) == 0 {
                    return i;
                }
                i += 1;
                item = item.offset(1);
            }
            i = 0;
            item = (*descr).items;
            while i <= start {
                if LISTBOX_lstrcmpiW((*descr).locale, str, (*item).str_) == 0 {
                    return i;
                }
                i += 1;
                item = item.offset(1);
            }
        } else {
            let len = strlenW(str);
            i = start + 1;
            while i < (*descr).nb_items {
                if strncmpiW(str, (*item).str_, len) == 0 {
                    return i;
                }
                if *(*item).str_ == '[' as u16 {
                    if strncmpiW(str, (*item).str_.offset(1), len) == 0 {
                        return i;
                    }
                    if *(*item).str_.offset(1) == '-' as u16
                        && strncmpiW(str, (*item).str_.offset(2), len) == 0
                    {
                        return i;
                    }
                }
                i += 1;
                item = item.offset(1);
            }
            i = 0;
            item = (*descr).items;
            while i <= start {
                if strncmpiW(str, (*item).str_, len) == 0 {
                    return i;
                }
                if *(*item).str_ == '[' as u16 {
                    if strncmpiW(str, (*item).str_.offset(1), len) == 0 {
                        return i;
                    }
                    if *(*item).str_.offset(1) == '-' as u16
                        && strncmpiW(str, (*item).str_.offset(2), len) == 0
                    {
                        return i;
                    }
                }
                i += 1;
                item = item.offset(1);
            }
        }
    } else {
        if exact != 0 && ((*descr).style & LBS_SORT) != 0 {
            return LISTBOX_FindStringPos(descr, str, 1);
        }

        i = start + 1;
        while i < (*descr).nb_items {
            if (*item).data == str as ULONG_PTR {
                return i;
            }
            i += 1;
            item = item.offset(1);
        }
        i = 0;
        item = (*descr).items;
        while i <= start {
            if (*item).data == str as ULONG_PTR {
                return i;
            }
            i += 1;
            item = item.offset(1);
        }
    }
    LB_ERR as INT
}

unsafe fn LISTBOX_GetSelCount(descr: *const LB_DESCR) -> LRESULT {
    let mut count: INT = 0;
    let mut item = (*descr).items;

    if ((*descr).style & LBS_MULTIPLESEL) == 0 ||
        ((*descr).style & LBS_NOSEL) != 0 {
        return LB_ERR as LRESULT;
    }
    let mut i: INT = 0;
    while i < (*descr).nb_items {
        if (*item).selected != 0 {
            count += 1;
        }
        i += 1;
        item = item.add(1);
    }
    count as LRESULT
}

unsafe fn LISTBOX_GetSelItems(descr: *const LB_DESCR, max: INT, array: LPINT) -> LRESULT {
    let mut item = (*descr).items as *const LB_ITEMDATA;

    if ((*descr).style & LBS_MULTIPLESEL) == 0 {
        return LB_ERR as LRESULT;
    }
    let mut count: INT = 0;
    let mut i: INT = 0;
    while i < (*descr).nb_items && count < max {
        if (*item).selected != 0 {
            *array.offset(count as isize) = i;
            count += 1;
        }
        i += 1;
        item = item.offset(1);
    }
    count as LRESULT
}

unsafe fn LISTBOX_Paint(descr: *mut LB_DESCR, hdc: HDC) -> LRESULT {
    let mut i: INT;
    let mut col_pos: INT = (*descr).page_size - 1;
    let mut rect: RECT = core::mem::zeroed();
    let mut focusRect: RECT = RECT { left: -1, top: -1, right: -1, bottom: -1 };
    let mut oldFont: HFONT = 0 as HFONT;
    let hbrush: HBRUSH;
    let mut oldBrush: HBRUSH = 0 as HBRUSH;

    if (*descr).style & LBS_NOREDRAW != 0 {
        return 0;
    }

    SetRect(&mut rect, 0, 0, (*descr).width, (*descr).height);
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        rect.right = rect.left + (*descr).column_width;
    } else if (*descr).horz_pos != 0 {
        SetWindowOrgEx(hdc, (*descr).horz_pos, 0, core::ptr::null_mut());
        rect.right += (*descr).horz_pos;
    }

    if (*descr).font != 0 as HFONT {
        oldFont = SelectObject(hdc, (*descr).font as _) as HFONT;
    }

    hbrush = GetControlColor((*descr).owner, (*descr).self_, hdc, WM_CTLCOLORLISTBOX);

    if hbrush != 0 as HBRUSH {
        oldBrush = SelectObject(hdc, hbrush as _) as HBRUSH;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(hdc, GetSysColor(COLOR_GRAYTEXT as _));
    }

    if (*descr).nb_items == 0
        && (*descr).focus_item != -1
        && (*descr).caret_on != 0
        && (*descr).in_focus != 0
    {
        rect.bottom = rect.top + (*descr).item_height;
        ExtTextOutW(
            hdc,
            0,
            0,
            ETO_OPAQUE | ETO_CLIPPED,
            &rect,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        LISTBOX_PaintItem(descr, hdc, &rect, (*descr).focus_item, ODA_FOCUS, 0);
        rect.top = rect.bottom;
    }

    i = (*descr).top_item;
    while i < (*descr).nb_items {
        if (*descr).style & LBS_OWNERDRAWVARIABLE == 0 {
            rect.bottom = rect.top + (*descr).item_height;
        } else {
            rect.bottom = rect.top + (*(*descr).items.offset(i as isize)).height as INT;
        }

        if i == (*descr).focus_item {
            focusRect = rect;
        }

        rect.bottom = min(rect.bottom, (*descr).height);

        LISTBOX_PaintItem(descr, hdc, &rect, i, ODA_DRAWENTIRE, 1);
        rect.top = rect.bottom;

        if (*descr).style & LBS_MULTICOLUMN != 0 && col_pos == 0 {
            if IS_OWNERDRAW(descr) == 0 {
                if rect.top < (*descr).height {
                    rect.bottom = (*descr).height;
                    ExtTextOutW(
                        hdc,
                        0,
                        0,
                        ETO_OPAQUE | ETO_CLIPPED,
                        &rect,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                    );
                }
            }

            rect.left += (*descr).column_width;
            rect.right += (*descr).column_width;
            rect.top = 0;
            col_pos = (*descr).page_size - 1;
        } else {
            col_pos -= 1;
            if rect.top >= (*descr).height {
                break;
            }
        }

        i += 1;
    }

    if focusRect.top != focusRect.bottom && (*descr).caret_on != 0 && (*descr).in_focus != 0 {
        LISTBOX_PaintItem(descr, hdc, &focusRect, (*descr).focus_item, ODA_FOCUS, 0);
    }

    if IS_OWNERDRAW(descr) == 0 {
        if rect.top < (*descr).height {
            rect.bottom = (*descr).height;
            /* Upstream 16a65a0df29: fill with the WM_CTLCOLORLISTBOX brush. */
            let br = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
            FillRect(hdc, &rect, br);
        }
        if rect.right < (*descr).width {
            rect.left = rect.right;
            rect.right = (*descr).width;
            rect.top = 0;
            rect.bottom = (*descr).height;
            /* Upstream 16a65a0df29: fill with the WM_CTLCOLORLISTBOX brush. */
            let br = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
            FillRect(hdc, &rect, br);
        }
    }
    if oldFont != 0 as HFONT {
        SelectObject(hdc, oldFont as _);
    }
    if oldBrush != 0 as HBRUSH {
        SelectObject(hdc, oldBrush as _);
    }
    0
}

unsafe fn LISTBOX_InvalidateItems(descr: *mut LB_DESCR, index: INT) {
    let mut rect: RECT = core::mem::zeroed();

    if LISTBOX_GetItemRect(descr, index, &mut rect) == 1 {
        if (*descr).style & LBS_NOREDRAW != 0 {
            (*descr).style |= LBS_DISPLAYCHANGED;
            return;
        }
        rect.bottom = (*descr).height;
        InvalidateRect((*descr).self_, &mut rect, 1);
        if (*descr).style & LBS_MULTICOLUMN != 0 {
            rect.left = rect.right;
            rect.right = (*descr).width;
            rect.top = 0;
            InvalidateRect((*descr).self_, &mut rect, 1);
        }
    }
}

unsafe fn LISTBOX_InvalidateItemRect(descr: *mut LB_DESCR, index: INT) {
    let mut rect: RECT = core::mem::zeroed();

    if LISTBOX_GetItemRect(descr, index, &mut rect) == 1 {
        InvalidateRect((*descr).self_, &mut rect, 1u8 as _);
    }
}

unsafe fn LISTBOX_GetItemHeight(descr: *const LB_DESCR, index: INT) -> LRESULT {
    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 && (*descr).nb_items > 0 {
        if index < 0 || index >= (*descr).nb_items {
            SetLastError(ERROR_INVALID_INDEX);
            return LB_ERR as LRESULT;
        }
        (*(*descr).items.offset(index as isize)).height as LRESULT
    } else {
        (*descr).item_height as LRESULT
    }
}

unsafe fn LISTBOX_SetItemHeight(descr: *mut LB_DESCR, index: INT, mut height: INT, repaint: BOOL) -> LRESULT {
    if height > 255 {
        return -1;
    }

    if height == 0 {
        height = 1;
    }

    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        if index < 0 || index >= (*descr).nb_items {
            SetLastError(ERROR_INVALID_INDEX);
            return LB_ERR as LRESULT;
        }
        (*(*descr).items.offset(index as isize)).height = height as UINT;
        LISTBOX_UpdateScroll(descr);
        if repaint != 0 {
            LISTBOX_InvalidateItems(descr, index);
        }
    } else if height != (*descr).item_height {
        (*descr).item_height = height;
        LISTBOX_UpdatePage(descr);
        LISTBOX_UpdateScroll(descr);
        if repaint != 0 {
            InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
        }
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SetHorizontalPos(descr: *mut LB_DESCR, mut pos: INT) {
    let diff: INT;

    if pos > (*descr).horz_extent - (*descr).width {
        pos = (*descr).horz_extent - (*descr).width;
    }
    if pos < 0 {
        pos = 0;
    }
    diff = (*descr).horz_pos - pos;
    if diff == 0 {
        return;
    }
    (*descr).horz_pos = pos;
    LISTBOX_UpdateScroll(descr);
    if abs(diff) < (*descr).width {
        let mut rect: RECT = core::mem::zeroed();
        if LISTBOX_GetItemRect(descr, (*descr).focus_item, &mut rect) == 1 {
            InvalidateRect((*descr).self_, &mut rect, 1u8 as _);
        }
        ScrollWindowEx(
            (*descr).self_,
            diff,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN) as _,
        );
    } else {
        InvalidateRect((*descr).self_, core::ptr::null_mut(), 1u8 as _);
    }
}

unsafe fn LISTBOX_SetHorizontalExtent(descr: *mut LB_DESCR, extent: INT) -> LRESULT {
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        return LB_OKAY as LRESULT;
    }
    if extent == (*descr).horz_extent {
        return LB_OKAY as LRESULT;
    }
    TRACE!(b"[%p]: new horz extent = %d\n\0".as_ptr() as *const i8, (*descr).self_, extent);
    (*descr).horz_extent = extent;
    if (*descr).style & WS_HSCROLL != 0 {
        let mut info: SCROLLINFO = core::mem::zeroed();
        info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
        info.nMin = 0;
        info.nMax = if (*descr).horz_extent != 0 { (*descr).horz_extent - 1 } else { 0 };
        info.fMask = SIF_RANGE;
        if (*descr).style & LBS_DISABLENOSCROLL != 0 {
            info.fMask |= SIF_DISABLENOSCROLL;
        }
        SetScrollInfo((*descr).self_, SB_HORZ as i32, &mut info, 1u8 as i32);
    }
    if (*descr).horz_pos > extent - (*descr).width {
        LISTBOX_SetHorizontalPos(descr, extent - (*descr).width);
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SetColumnWidth(descr: *mut LB_DESCR, width: INT) -> LRESULT {
    if width == (*descr).column_width {
        return LB_OKAY as LRESULT;
    }
    (*descr).column_width = width;
    LISTBOX_UpdatePage(descr);
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SetFont(descr: *mut LB_DESCR, font: HFONT) -> INT {
    let hdc: HDC;
    let mut oldFont: HFONT = 0 as HFONT;
    let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\0".as_ptr() as *const i8;
    let mut sz: SIZE = core::mem::zeroed();

    (*descr).font = font;

    hdc = GetDCEx((*descr).self_, 0 as HRGN, DCX_CACHE);
    if hdc.is_null() {
        ERR!("unable to get DC.\n");
        return 16;
    }
    if !font.is_null() {
        oldFont = SelectObject(hdc, font as *mut core::ffi::c_void) as HFONT;
    }
    GetTextExtentPointA(hdc, alphabet, 52, &mut sz);
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as *mut core::ffi::c_void);
    }
    ReleaseDC((*descr).self_, hdc);

    (*descr).avg_char_width = (sz.cx / 26 + 1) / 2;
    if ((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) == 0 {
        LISTBOX_SetItemHeight(descr, 0, sz.cy, 0);
    }
    sz.cy
}

unsafe fn LISTBOX_MakeItemVisible(descr: *mut LB_DESCR, index: INT, fully: BOOL) {
    let top: INT;

    TRACE!(b"current top item %d, index %d, fully %d\n\0".as_ptr() as *const i8,
          (*descr).top_item, index, fully);

    if index <= (*descr).top_item {
        top = index;
    } else if (*descr).style & LBS_MULTICOLUMN != 0 {
        let mut cols: INT = (*descr).width;
        if fully == 0 {
            cols += (*descr).column_width - 1;
        }
        if cols >= (*descr).column_width {
            cols /= (*descr).column_width;
        } else {
            cols = 1;
        }
        if index < (*descr).top_item + ((*descr).page_size * cols) {
            return;
        }
        top = index - (*descr).page_size * (cols - 1);
    } else if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        let mut height: INT = if fully != 0 {
            (*(*descr).items.offset(index as isize)).height as INT
        } else {
            1
        };
        let mut t = index;
        while t > (*descr).top_item {
            height += (*(*descr).items.offset((t - 1) as isize)).height as INT;
            if height > (*descr).height {
                break;
            }
            t -= 1;
        }
        top = t;
    } else {
        if index < (*descr).top_item + (*descr).page_size {
            return;
        }
        if fully == 0 && (index == (*descr).top_item + (*descr).page_size) &&
            ((*descr).height > ((*descr).page_size * (*descr).item_height)) {
            return;
        }
        top = index - (*descr).page_size + 1;
    }
    LISTBOX_SetTopItem(descr, top, 1);
}

unsafe fn LISTBOX_SetCaretIndex(descr: *mut LB_DESCR, index: INT, fully_visible: BOOL) -> LRESULT {
    let oldfocus = (*descr).focus_item;

    if (*descr).style & LBS_NOSEL != 0 {
        return LB_ERR as LRESULT;
    }
    if index < 0 || index >= (*descr).nb_items {
        return LB_ERR as LRESULT;
    }
    if index == oldfocus {
        return LB_OKAY as LRESULT;
    }

    LISTBOX_DrawFocusRect(descr, 0);
    (*descr).focus_item = index;

    LISTBOX_MakeItemVisible(descr, index, fully_visible);
    LISTBOX_DrawFocusRect(descr, 1);

    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SelectItemRange(descr: *mut LB_DESCR, mut first: INT, mut last: INT, on: BOOL) -> LRESULT {
    let mut i: INT;

    if (*descr).style & LBS_NOSEL != 0 {
        return LB_ERR as LRESULT;
    }
    if (*descr).style & LBS_MULTIPLESEL == 0 {
        return LB_ERR as LRESULT;
    }

    if (*descr).nb_items == 0 {
        return LB_OKAY as LRESULT;
    }

    if last == -1 || last >= (*descr).nb_items {
        last = (*descr).nb_items - 1;
    }
    if first < 0 {
        first = 0;
    }
    if last < first {
        return LB_OKAY as LRESULT;
    }

    if on != 0 {
        i = first;
        while i <= last {
            if (*(*descr).items.offset(i as isize)).selected == 0 {
                (*(*descr).items.offset(i as isize)).selected = 1;
                LISTBOX_InvalidateItemRect(descr, i);
            }
            i += 1;
        }
    } else {
        i = first;
        while i <= last {
            if (*(*descr).items.offset(i as isize)).selected != 0 {
                (*(*descr).items.offset(i as isize)).selected = 0;
                LISTBOX_InvalidateItemRect(descr, i);
            }
            i += 1;
        }
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_SetSelection(descr: *mut LB_DESCR, index: INT, on: BOOL, send_notify: BOOL) -> LRESULT {
    if (*descr).style & LBS_NOSEL != 0 {
        (*descr).selected_item = index;
        return LB_ERR as LRESULT;
    }
    if index < -1 || index >= (*descr).nb_items {
        return LB_ERR as LRESULT;
    }
    if (*descr).style & LBS_MULTIPLESEL != 0 {
        if index == -1 {
            return LISTBOX_SelectItemRange(descr, 0, (*descr).nb_items, on);
        } else {
            return LISTBOX_SelectItemRange(descr, index, index, on);
        }
    } else {
        let oldsel = (*descr).selected_item;
        if index == oldsel {
            return LB_OKAY as LRESULT;
        }
        if oldsel != -1 {
            (*(*descr).items.offset(oldsel as isize)).selected = 0;
        }
        if index != -1 {
            (*(*descr).items.offset(index as isize)).selected = 1;
        }
        if oldsel != -1 {
            LISTBOX_RepaintItem(descr, oldsel, ODA_SELECT);
        }
        (*descr).selected_item = index;
        if index != -1 {
            LISTBOX_RepaintItem(descr, index, ODA_SELECT);
        }
        if send_notify != 0 && (*descr).nb_items != 0 {
            SEND_NOTIFICATION(descr, if index != -1 { LBN_SELCHANGE as INT } else { LBN_SELCANCEL as INT });
        } else {
            if !(*descr).lphc.is_null() {
                (*(*descr).lphc).wState |= CBF_SELCHANGE;
            }
        }
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_MoveCaret(descr: *mut LB_DESCR, index: INT, fully_visible: BOOL) {
    if (index < 0) || (index >= (*descr).nb_items) {
        return;
    }

    LISTBOX_DrawFocusRect(descr, 0);

    if (*descr).style & LBS_EXTENDEDSEL != 0 {
        if (*descr).anchor_item != -1 {
            let first: INT = core::cmp::min(index, (*descr).anchor_item);
            let last: INT = core::cmp::max(index, (*descr).anchor_item);
            if first > 0 {
                LISTBOX_SelectItemRange(descr, 0, first - 1, 0);
            }
            LISTBOX_SelectItemRange(descr, last + 1, -1, 0);
            LISTBOX_SelectItemRange(descr, first, last, 1);
        }
    } else if (*descr).style & LBS_MULTIPLESEL == 0 {
        LISTBOX_SetSelection(descr, index, 1, 0);
    }

    (*descr).focus_item = index;
    LISTBOX_MakeItemVisible(descr, index, fully_visible);
    LISTBOX_DrawFocusRect(descr, 1);
}

unsafe fn LISTBOX_InsertItem(descr: *mut LB_DESCR, mut index: INT, str: LPWSTR, data: ULONG_PTR) -> LRESULT {
    let mut item: *mut LB_ITEMDATA;
    let mut max_items: INT;
    let oldfocus: INT = (*descr).focus_item;

    if index == -1 {
        index = (*descr).nb_items;
    } else if index < 0 || index > (*descr).nb_items {
        return LB_ERR as LRESULT;
    }
    if (*descr).items.is_null() {
        max_items = 0;
    } else {
        max_items = (HeapSize(GetProcessHeap(), 0, (*descr).items as *mut _)
            / core::mem::size_of::<LB_ITEMDATA>()) as INT;
    }
    if (*descr).nb_items == max_items {
        max_items += LB_ARRAY_GRANULARITY as INT;
        if !(*descr).items.is_null() {
            item = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*descr).items as *mut _,
                max_items as usize * core::mem::size_of::<LB_ITEMDATA>(),
            ) as *mut LB_ITEMDATA;
        } else {
            item = HeapAlloc(
                GetProcessHeap(),
                0,
                max_items as usize * core::mem::size_of::<LB_ITEMDATA>(),
            ) as *mut LB_ITEMDATA;
        }
        if item.is_null() {
            SEND_NOTIFICATION(descr, LBN_ERRSPACE);
            return LB_ERRSPACE as LRESULT;
        }
        (*descr).items = item;
    }

    item = (*descr).items.offset(index as isize);
    if index < (*descr).nb_items {
        core::ptr::copy(
            item,
            item.offset(1),
            ((*descr).nb_items - index) as usize,
        );
    }
    (*item).str_ = str;
    (*item).data = data;
    (*item).height = 0;
    (*item).selected = 0u8 as BOOL;
    (*descr).nb_items += 1;

    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
        let id = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;

        mis.CtlType = ODT_LISTBOX;
        mis.CtlID = id;
        mis.itemID = index as UINT;
        mis.itemData = (*(*descr).items.offset(index as isize)).data;
        mis.itemHeight = (*descr).item_height as UINT;
        SendMessageW((*descr).owner, WM_MEASUREITEM, id as WPARAM, &mut mis as *mut _ as LPARAM);
        (*item).height = if mis.itemHeight != 0 { mis.itemHeight } else { 1 };
        TRACE!(
            b"[%p]: measure item %d (%s) = %d\n\0".as_ptr(),
            (*descr).self_,
            index,
            if !str.is_null() { debugstr_w(str) } else { b"\0".as_ptr() as *const _ },
            (*item).height,
        );
    }

    LISTBOX_UpdateScroll(descr);
    LISTBOX_InvalidateItems(descr, index);

    if (*descr).nb_items == 1 {
        LISTBOX_SetCaretIndex(descr, 0, 0u8 as BOOL);
    } else if ISWIN31() && IS_MULTISELECT(descr) == 0 {
        (*descr).selected_item += 1;
        LISTBOX_SetSelection(descr, (*descr).selected_item - 1, 1u8 as BOOL, 0u8 as BOOL);
    } else {
        if index <= (*descr).selected_item {
            (*descr).selected_item += 1;
            (*descr).focus_item = oldfocus;
        }
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_InsertString(descr: *mut LB_DESCR, mut index: INT, mut str: LPCWSTR) -> LRESULT {
    let mut new_str: LPWSTR = core::ptr::null_mut();
    let mut data: ULONG_PTR = 0;
    let ret: LRESULT;

    if HAS_STRINGS(descr) != 0 {
        static empty_stringW: [u16; 1] = [0];
        if str.is_null() {
            str = empty_stringW.as_ptr();
        }
        new_str = HeapAlloc(GetProcessHeap(), 0, (strlenW(str) as usize + 1) * core::mem::size_of::<u16>()) as LPWSTR;
        if new_str.is_null() {
            SEND_NOTIFICATION(descr, LBN_ERRSPACE);
            return LB_ERRSPACE as LRESULT;
        }
        strcpyW(new_str, str);
    } else {
        data = str as ULONG_PTR;
    }

    if index == -1 {
        index = (*descr).nb_items;
    }
    ret = LISTBOX_InsertItem(descr, index, new_str, data);
    if ret != 0 {
        HeapFree(GetProcessHeap(), 0, new_str as *mut core::ffi::c_void);
        return ret;
    }

    index as LRESULT
}

unsafe fn LISTBOX_DeleteItem(descr: *mut LB_DESCR, index: INT) {
    let item_data: ULONG_PTR = (*(*descr).items.offset(index as isize)).data;
    let item_str: LPWSTR = (*(*descr).items.offset(index as isize)).str_;

    if (*descr).nb_items == 0 {
        SendMessageW((*descr).self_, LB_RESETCONTENT as UINT, 0, 0);
    }

    if ((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) != 0 || item_data != 0 {
        let mut dis: DELETEITEMSTRUCT = core::mem::zeroed();
        let id = GetWindowLongPtrW((*descr).self_, GWLP_ID as INT) as UINT;

        dis.CtlType = ODT_LISTBOX;
        dis.CtlID = id;
        dis.itemID = index as UINT;
        dis.hwndItem = (*descr).self_;
        dis.itemData = item_data;
        SendMessageW((*descr).owner, WM_DELETEITEM, id as WPARAM, core::ptr::addr_of_mut!(dis) as LPARAM);
    }

    if (((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) == 0)
        || ((*descr).style & LBS_HASSTRINGS) != 0
    {
        HeapFree(GetProcessHeap(), 0, item_str as *mut core::ffi::c_void);
    }
}

unsafe fn LISTBOX_RemoveItem(descr: *mut LB_DESCR, index: INT) -> LRESULT {
    let mut item: *mut LB_ITEMDATA;
    let mut max_items: INT;

    if (index < 0) || (index >= (*descr).nb_items) {
        return LB_ERR as LRESULT;
    }

    LISTBOX_InvalidateItems(descr, index);

    (*descr).nb_items -= 1;
    LISTBOX_DeleteItem(descr, index);

    if (*descr).nb_items == 0 {
        return LB_OKAY as LRESULT;
    }

    item = &mut *(*descr).items.offset(index as isize);
    if index < (*descr).nb_items {
        core::ptr::copy(
            item.offset(1) as *const LB_ITEMDATA,
            item,
            ((*descr).nb_items - index) as usize,
        );
    }
    if (*descr).anchor_item == (*descr).nb_items {
        (*descr).anchor_item -= 1;
    }

    max_items = (HeapSize(GetProcessHeap(), 0, (*descr).items as *mut _)
        / core::mem::size_of::<LB_ITEMDATA>()) as INT;
    if (*descr).nb_items < max_items - 2 * LB_ARRAY_GRANULARITY {
        max_items -= LB_ARRAY_GRANULARITY;
        item = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*descr).items as *mut _,
            (max_items as usize) * core::mem::size_of::<LB_ITEMDATA>(),
        ) as *mut LB_ITEMDATA;
        if !item.is_null() {
            (*descr).items = item;
        }
    }

    LISTBOX_UpdateScroll(descr);

    if (*descr).nb_items == (*descr).page_size {
        LISTBOX_SetTopItem(descr, 0, 1);
    }

    if IS_MULTISELECT(descr) == 0 {
        if index == (*descr).selected_item {
            (*descr).selected_item = -1;
        } else if index < (*descr).selected_item {
            (*descr).selected_item -= 1;
            if ISWIN31() {
                LISTBOX_SetSelection(descr, (*descr).selected_item + 1, 1, 0);
            }
        }
    }

    if (*descr).focus_item >= (*descr).nb_items {
        (*descr).focus_item = (*descr).nb_items - 1;
        if (*descr).focus_item < 0 {
            (*descr).focus_item = 0;
        }
    }
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_ResetContent(descr: *mut LB_DESCR) {
    let mut i: INT = (*descr).nb_items - 1;
    while i >= 0 {
        LISTBOX_DeleteItem(descr, i);
        i -= 1;
    }
    HeapFree(GetProcessHeap(), 0, (*descr).items as _);
    (*descr).nb_items = 0;
    (*descr).top_item = 0;
    (*descr).selected_item = -1;
    (*descr).focus_item = 0;
    (*descr).anchor_item = -1;
    (*descr).items = core::ptr::null_mut();
}

unsafe fn LISTBOX_SetCount(descr: *mut LB_DESCR, count: INT) -> LRESULT {
    let ret: LRESULT;

    if IS_OWNERDRAW(descr) == 0 || ((*descr).style & LBS_HASSTRINGS as UINT) != 0 {
        SetLastError(ERROR_SETCOUNT_ON_BAD_LB);
        return LB_ERR as LRESULT;
    }

    if count > (*descr).nb_items {
        while count > (*descr).nb_items {
            let ret = LISTBOX_InsertString(descr, -1, core::ptr::null_mut());
            if ret < 0 {
                return ret;
            }
        }
    } else if count < (*descr).nb_items {
        while count < (*descr).nb_items {
            let ret = LISTBOX_RemoveItem(descr, (*descr).nb_items - 1);
            if ret < 0 {
                return ret;
            }
        }
    }

    InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
    LB_OKAY as LRESULT
}

unsafe fn LISTBOX_Directory(descr: *mut LB_DESCR, attrib: UINT, filespec: LPCWSTR, long_names: BOOL) -> LRESULT {
    let handle: HANDLE;
    let mut ret: LRESULT = LB_OKAY as LRESULT;
    let mut entry: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut pos: i32;
    let mut maxinsert: LRESULT = LB_ERR as LRESULT;

    if attrib != (DDL_DRIVES | DDL_EXCLUSIVE) {
        handle = FindFirstFileW(filespec, &mut entry);
        if handle == INVALID_HANDLE_VALUE {
            let le = GetLastError();
            if le != ERROR_NO_MORE_FILES && le != ERROR_FILE_NOT_FOUND {
                return LB_ERR as LRESULT;
            }
        } else {
            loop {
                'cont: {
                    let mut buffer: [WCHAR; 270] = [0; 270];
                    if (entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                        let bracketW: [WCHAR; 2] = [']' as WCHAR, 0];
                        let dotW: [WCHAR; 2] = ['.' as WCHAR, 0];
                        if (attrib & DDL_DIRECTORY) == 0
                            || strcmpW(entry.cFileName.as_ptr(), dotW.as_ptr()) == 0
                        {
                            break 'cont;
                        }
                        buffer[0] = '[' as WCHAR;
                        if long_names == 0 && entry.cAlternateFileName[0] != 0 {
                            strcpyW(buffer.as_mut_ptr().add(1), entry.cAlternateFileName.as_ptr());
                        } else {
                            strcpyW(buffer.as_mut_ptr().add(1), entry.cFileName.as_ptr());
                        }
                        strcatW(buffer.as_mut_ptr(), bracketW.as_ptr());
                    } else {
                        const ATTRIBS: u32 = FILE_ATTRIBUTE_READONLY
                            | FILE_ATTRIBUTE_HIDDEN
                            | FILE_ATTRIBUTE_SYSTEM
                            | FILE_ATTRIBUTE_ARCHIVE;
                        if (attrib & DDL_EXCLUSIVE) != 0
                            && (attrib & ATTRIBS) != (entry.dwFileAttributes & ATTRIBS)
                        {
                            break 'cont;
                        }
                        if long_names == 0 && entry.cAlternateFileName[0] != 0 {
                            strcpyW(buffer.as_mut_ptr(), entry.cAlternateFileName.as_ptr());
                        } else {
                            strcpyW(buffer.as_mut_ptr(), entry.cFileName.as_ptr());
                        }
                    }
                    if long_names == 0 {
                        CharLowerW(buffer.as_mut_ptr());
                    }
                    pos = LISTBOX_FindFileStrPos(descr, buffer.as_ptr());
                    ret = LISTBOX_InsertString(descr, pos, buffer.as_ptr());
                    if ret < 0 {
                        // C `break;` exits the loop; leave the 'cont block and the
                        // `if ret < 0 { break; }` below breaks the loop.
                        break 'cont;
                    }
                    if ret <= maxinsert {
                        maxinsert += 1;
                    } else {
                        maxinsert = ret;
                    }
                }
                if ret < 0 {
                    break;
                }
                if FindNextFileW(handle, &mut entry) == 0 {
                    break;
                }
            }
            FindClose(handle);
        }
    }
    if ret >= 0 {
        ret = maxinsert;

        if (attrib & DDL_DRIVES) != 0 {
            let mut buffer: [WCHAR; 6] = [
                '[' as WCHAR,
                '-' as WCHAR,
                'a' as WCHAR,
                '-' as WCHAR,
                ']' as WCHAR,
                0,
            ];
            let mut root: [WCHAR; 4] = ['A' as WCHAR, ':' as WCHAR, '\\' as WCHAR, 0];
            let mut drive = 0;
            while drive < 26 {
                if GetDriveTypeW(root.as_ptr()) <= DRIVE_NO_ROOT_DIR {
                    drive += 1;
                    buffer[2] += 1;
                    root[0] += 1;
                    continue;
                }
                ret = LISTBOX_InsertString(descr, -1, buffer.as_ptr());
                if ret < 0 {
                    break;
                }
                drive += 1;
                buffer[2] += 1;
                root[0] += 1;
            }
        }
    }
    ret
}

unsafe fn LISTBOX_HandleVScroll(descr: *mut LB_DESCR, scrollReq: WORD, pos: WORD) -> LRESULT {
    let mut info: SCROLLINFO = core::mem::zeroed();

    if (*descr).style & LBS_MULTICOLUMN != 0 {
        return 0;
    }
    match scrollReq as u32 {
        SB_LINEUP => {
            LISTBOX_SetTopItem(descr, (*descr).top_item - 1, TRUE as BOOL);
        }
        SB_LINEDOWN => {
            LISTBOX_SetTopItem(descr, (*descr).top_item + 1, TRUE as BOOL);
        }
        SB_PAGEUP => {
            LISTBOX_SetTopItem(descr, (*descr).top_item - LISTBOX_GetCurrentPageSize(descr), TRUE as BOOL);
        }
        SB_PAGEDOWN => {
            LISTBOX_SetTopItem(descr, (*descr).top_item + LISTBOX_GetCurrentPageSize(descr), TRUE as BOOL);
        }
        SB_THUMBPOSITION => {
            LISTBOX_SetTopItem(descr, pos as INT, TRUE as BOOL);
        }
        SB_THUMBTRACK => {
            info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
            info.fMask = SIF_TRACKPOS;
            GetScrollInfo((*descr).self_, SB_VERT as i32, &mut info);
            LISTBOX_SetTopItem(descr, info.nTrackPos, TRUE as BOOL);
        }
        SB_TOP => {
            LISTBOX_SetTopItem(descr, 0, TRUE as BOOL);
        }
        SB_BOTTOM => {
            LISTBOX_SetTopItem(descr, (*descr).nb_items, TRUE as BOOL);
        }
        _ => {}
    }
    0
}

unsafe fn LISTBOX_HandleHScroll(descr: *mut LB_DESCR, scrollReq: WORD, pos: WORD) -> LRESULT {
    let mut info: SCROLLINFO = core::mem::zeroed();
    let mut page: INT;

    if (*descr).style & LBS_MULTICOLUMN != 0 {
        match scrollReq as u32 {
            SB_LINELEFT => {
                LISTBOX_SetTopItem(descr, (*descr).top_item - (*descr).page_size, 1u8.into());
            }
            SB_LINERIGHT => {
                LISTBOX_SetTopItem(descr, (*descr).top_item + (*descr).page_size, 1u8.into());
            }
            SB_PAGELEFT => {
                page = (*descr).width / (*descr).column_width;
                if page < 1 { page = 1; }
                LISTBOX_SetTopItem(descr, (*descr).top_item - page * (*descr).page_size, 1u8.into());
            }
            SB_PAGERIGHT => {
                page = (*descr).width / (*descr).column_width;
                if page < 1 { page = 1; }
                LISTBOX_SetTopItem(descr, (*descr).top_item + page * (*descr).page_size, 1u8.into());
            }
            SB_THUMBPOSITION => {
                LISTBOX_SetTopItem(descr, pos as INT * (*descr).page_size, 1u8.into());
            }
            SB_THUMBTRACK => {
                info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
                info.fMask = SIF_TRACKPOS;
                GetScrollInfo((*descr).self_, SB_HORZ as INT, &mut info);
                LISTBOX_SetTopItem(descr, info.nTrackPos * (*descr).page_size, 1u8.into());
            }
            SB_LEFT => {
                LISTBOX_SetTopItem(descr, 0, 1u8.into());
            }
            SB_RIGHT => {
                LISTBOX_SetTopItem(descr, (*descr).nb_items, 1u8.into());
            }
            _ => {}
        }
    } else if (*descr).horz_extent != 0 {
        match scrollReq as u32 {
            SB_LINELEFT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos - 1);
            }
            SB_LINERIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos + 1);
            }
            SB_PAGELEFT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos - (*descr).width);
            }
            SB_PAGERIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos + (*descr).width);
            }
            SB_THUMBPOSITION => {
                LISTBOX_SetHorizontalPos(descr, pos as INT);
            }
            SB_THUMBTRACK => {
                info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
                info.fMask = SIF_TRACKPOS;
                GetScrollInfo((*descr).self_, SB_HORZ as INT, &mut info);
                LISTBOX_SetHorizontalPos(descr, info.nTrackPos);
            }
            SB_LEFT => {
                LISTBOX_SetHorizontalPos(descr, 0);
            }
            SB_RIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_extent - (*descr).width);
            }
            _ => {}
        }
    }
    0
}

unsafe fn LISTBOX_HandleMouseWheel(descr: *mut LB_DESCR, delta: SHORT) -> LRESULT {
    let mut pulScrollLines: UINT = 3;

    SystemParametersInfoW(SPI_GETWHEELSCROLLLINES, 0, &mut pulScrollLines as *mut UINT as *mut _, 0);

    if (delta < 0 && (*descr).wheel_remain < 0) ||
       (delta > 0 && (*descr).wheel_remain > 0) {
        (*descr).wheel_remain += delta as INT;
    } else {
        (*descr).wheel_remain = delta as INT;
    }

    if (*descr).wheel_remain != 0 && pulScrollLines != 0 {
        pulScrollLines = min((*descr).page_size as UINT, pulScrollLines);
        let mut cLineScroll: INT = (pulScrollLines as f32 * (*descr).wheel_remain as f32 / WHEEL_DELTA as f32) as INT;
        (*descr).wheel_remain -= WHEEL_DELTA as INT * cLineScroll / pulScrollLines as INT;
        if cLineScroll < 0 {
            cLineScroll -= (*descr).page_size;
        }
        LISTBOX_SetTopItem(descr, (*descr).top_item - cLineScroll, 1);
    }
    0
}

unsafe fn LISTBOX_HandleLButtonDown( descr: *mut LB_DESCR, keys: DWORD, x: INT, y: INT ) -> LRESULT
{
    let index: INT = LISTBOX_GetItemFromPoint( descr, x, y );

    if (*descr).caret_on == 0 && (*descr).in_focus != 0 { return 0; }

    if (*descr).in_focus == 0
    {
        if (*descr).lphc.is_null() { SetFocus( (*descr).self_ ); }
        else {
            SetFocus( if !(*(*descr).lphc).hWndEdit.is_null() { (*(*descr).lphc).hWndEdit } else { (*(*descr).lphc).self_ } );
        }
    }

    if index == -1 { return 0; }

    if (*descr).lphc.is_null()
    {
        if (*descr).style & LBS_NOTIFY != 0
        {
            SendMessageW( (*descr).owner, WM_LBTRACKPOINT, index as WPARAM,
                            crate::internal::MAKELPARAM( x as WORD, y as WORD ) );
        }
    }

    (*descr).captured = TRUE as BOOL;
    SetCapture( (*descr).self_ );

    if (*descr).style & (LBS_EXTENDEDSEL | LBS_MULTIPLESEL) != 0
    {
        if keys & MK_SHIFT == 0 { (*descr).anchor_item = index; }
        if keys & MK_CONTROL != 0
        {
            LISTBOX_SetCaretIndex( descr, index, FALSE as BOOL );
            LISTBOX_SetSelection( descr, index,
                                  ((*(*descr).items.offset(index as isize)).selected == 0) as BOOL,
                                  ((*descr).style & LBS_NOTIFY != 0) as BOOL);
        }
        else
        {
            LISTBOX_MoveCaret( descr, index, FALSE as BOOL );

            if (*descr).style & LBS_EXTENDEDSEL != 0
            {
                LISTBOX_SetSelection( descr, index,
                               (*(*descr).items.offset(index as isize)).selected,
                              ((*descr).style & LBS_NOTIFY != 0) as BOOL );
            }
            else
            {
                LISTBOX_SetSelection( descr, index,
                               ((*(*descr).items.offset(index as isize)).selected == 0) as BOOL,
                              ((*descr).style & LBS_NOTIFY != 0) as BOOL );
            }
        }
    }
    else
    {
        (*descr).anchor_item = index;
        LISTBOX_MoveCaret( descr, index, FALSE as BOOL );
        LISTBOX_SetSelection( descr, index,
                              TRUE as BOOL, ((*descr).style & LBS_NOTIFY != 0) as BOOL );
    }

    if (*descr).lphc.is_null()
    {
        if GetWindowLongPtrW( (*descr).self_, GWL_EXSTYLE ) & WS_EX_DRAGDETECT as LONG_PTR != 0
        {
            let mut pt: POINT = core::mem::zeroed();

            pt.x = x;
            pt.y = y;

            if DragDetect( (*descr).self_, pt ) != 0
            {
                SendMessageW( (*descr).owner, WM_BEGINDRAG, 0, 0 );
            }
        }
    }
    0
}

unsafe fn LISTBOX_HandleLButtonDownCombo( descr: *mut LB_DESCR, msg: UINT, keys: DWORD, x: INT, y: INT) -> LRESULT
{
    let mut clientRect: RECT = core::mem::zeroed();
    let mut screenRect: RECT = core::mem::zeroed();
    let mut mousePos: POINT = core::mem::zeroed();

    mousePos.x = x;
    mousePos.y = y;

    GetClientRect((*descr).self_, &mut clientRect);

    if PtInRect(&clientRect, mousePos) != 0
    {
        if msg == WM_LBUTTONDOWN
        {
            (*(*descr).lphc).droppedIndex = if (*descr).nb_items != 0 { (*descr).selected_item } else { -1 };
            return LISTBOX_HandleLButtonDown( descr, keys, x, y);
        }
        else if (*descr).style & LBS_NOTIFY != 0
        {
            SendMessageW( (*descr).owner, WM_COMMAND,
                crate::internal::MAKEWPARAM( GetWindowLongPtrW((*descr).self_, GWLP_ID) as WORD, LBN_DBLCLK as WORD) as WPARAM,
                (*descr).self_ as LPARAM );
        }
    }
    else
    {
        let mut screenMousePos: POINT;
        let hWndOldCapture: HWND;

        screenMousePos = mousePos;
        hWndOldCapture = GetCapture();
        ReleaseCapture();
        GetWindowRect((*descr).self_, &mut screenRect);
        ClientToScreen((*descr).self_, &mut screenMousePos);

        if PtInRect(&screenRect, screenMousePos) == 0
        {
            LISTBOX_SetCaretIndex( descr, (*(*descr).lphc).droppedIndex, FALSE as BOOL );
            LISTBOX_SetSelection( descr, (*(*descr).lphc).droppedIndex, FALSE as BOOL, FALSE as BOOL );
            COMBO_FlipListbox( (*descr).lphc, FALSE as BOOL, FALSE as BOOL );
        }
        else
        {
            let mut nHitTestType: INT = 0;
            let style: LONG = GetWindowLongPtrW( (*descr).self_, GWL_STYLE ) as LONG;
            if style & WS_VSCROLL as LONG != 0
            {
                clientRect.right += GetSystemMetrics(SM_CXVSCROLL as i32);
                if PtInRect( &clientRect, mousePos ) != 0
                {
                    nHitTestType = HTVSCROLL as INT;
                }
            }
            if style & WS_HSCROLL as LONG != 0
            {
                clientRect.bottom += GetSystemMetrics(SM_CYHSCROLL as i32);
                if PtInRect( &clientRect, mousePos ) != 0
                {
                    nHitTestType = HTHSCROLL as INT;
                }
            }

            if nHitTestType != 0
            {
                SendMessageW((*descr).self_, WM_NCLBUTTONDOWN, nHitTestType as WPARAM,
                             crate::internal::MAKELONG(screenMousePos.x as WORD, screenMousePos.y as WORD) as LPARAM);
            }
            if hWndOldCapture != core::ptr::null_mut()
            {
                SetCapture(hWndOldCapture);
            }
        }
    }
    0
}

unsafe fn LISTBOX_HandleLButtonUp(descr: *mut LB_DESCR) -> LRESULT {
    if LISTBOX_Timer != LB_TIMER_NONE {
        KillSystemTimer((*descr).self_, LB_TIMER_ID as UINT_PTR);
    }
    LISTBOX_Timer = LB_TIMER_NONE;
    if (*descr).captured != 0 {
        (*descr).captured = 0;
        if GetCapture() == (*descr).self_ {
            ReleaseCapture();
        }
        if ((*descr).style & LBS_NOTIFY) != 0 && (*descr).nb_items != 0 {
            SendMessageW(
                (*descr).owner,
                WM_COMMAND,
                crate::internal::MAKEWPARAM(
                    GetWindowLongPtrW((*descr).self_, GWLP_ID) as u16,
                    LBN_SELCHANGE as u16,
                ) as WPARAM,
                (*descr).self_ as LPARAM,
            );
        }
    }
    0
}

unsafe fn LISTBOX_HandleTimer(descr: *mut LB_DESCR, mut index: INT, dir: TIMER_DIRECTION) -> LRESULT {
    match dir {
        LB_TIMER_UP => {
            if (*descr).top_item != 0 {
                index = (*descr).top_item - 1;
            } else {
                index = 0;
            }
        }
        LB_TIMER_LEFT => {
            if (*descr).top_item != 0 {
                index -= (*descr).page_size;
            }
        }
        LB_TIMER_DOWN => {
            index = (*descr).top_item + LISTBOX_GetCurrentPageSize(descr);
            if index == (*descr).focus_item {
                index += 1;
            }
            if index >= (*descr).nb_items {
                index = (*descr).nb_items - 1;
            }
        }
        LB_TIMER_RIGHT => {
            if index + (*descr).page_size < (*descr).nb_items {
                index += (*descr).page_size;
            }
        }
        LB_TIMER_NONE => {}
        _ => {}
    }
    if index == (*descr).focus_item {
        return 0 as LRESULT;
    }
    LISTBOX_MoveCaret(descr, index, 0);
    1 as LRESULT
}

unsafe fn LISTBOX_HandleSystemTimer(descr: *mut LB_DESCR) -> LRESULT {
    if LISTBOX_HandleTimer(descr, (*descr).focus_item, LISTBOX_Timer) == 0 {
        KillSystemTimer((*descr).self_, LB_TIMER_ID as UINT_PTR);
        LISTBOX_Timer = LB_TIMER_NONE;
    }
    0
}

unsafe fn LISTBOX_HandleMouseMove( descr: *mut LB_DESCR, mut x: INT, mut y: INT )
{
    let index: INT;
    let mut dir: TIMER_DIRECTION = LB_TIMER_NONE;

    if (*descr).captured == 0 { return; }

    if ((*descr).style & LBS_MULTICOLUMN) != 0
    {
        if y < 0 { y = 0; }
        else if y >= (*descr).item_height * (*descr).page_size {
            y = (*descr).item_height * (*descr).page_size - 1;
        }

        if x < 0
        {
            dir = LB_TIMER_LEFT;
            x = 0;
        }
        else if x >= (*descr).width
        {
            dir = LB_TIMER_RIGHT;
            x = (*descr).width - 1;
        }
    }
    else
    {
        if y < 0 { dir = LB_TIMER_UP; }
        else if y >= (*descr).height { dir = LB_TIMER_DOWN; }
    }

    index = {
        let mut idx = LISTBOX_GetItemFromPoint( descr, x, y );
        if idx == -1 { idx = (*descr).focus_item; }
        idx
    };
    if LISTBOX_HandleTimer( descr, index, dir ) == 0 { dir = LB_TIMER_NONE; }

    if dir != LB_TIMER_NONE {
        SetSystemTimer( (*descr).self_, LB_TIMER_ID as UINT_PTR, LB_SCROLL_TIMEOUT as UINT, None );
    } else if LISTBOX_Timer != LB_TIMER_NONE {
        KillSystemTimer( (*descr).self_, LB_TIMER_ID as UINT_PTR );
    }
    LISTBOX_Timer = dir;
}

unsafe fn LISTBOX_HandleKeyDown(descr: *mut LB_DESCR, key: DWORD) -> LRESULT {
    let mut caret: INT = -1;
    let mut bForceSelection: BOOL = 1;
    if (((*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL)) != 0
        && ((*descr).style & LBS_NOSEL) == 0)
        || ((*descr).selected_item == (*descr).focus_item)
    {
        bForceSelection = 0;
    }

    if (*descr).style & LBS_WANTKEYBOARDINPUT != 0 {
        caret = SendMessageW(
            (*descr).owner,
            WM_VKEYTOITEM,
            crate::internal::MAKEWPARAM(crate::internal::LOWORD(key), (*descr).focus_item as WORD) as WPARAM,
            (*descr).self_ as LPARAM,
        ) as INT;
        if caret == -2 {
            return 0;
        }
    }

    if caret == -1 {
        loop {
            match key {
                VK_LEFT => {
                    if (*descr).style & LBS_MULTICOLUMN != 0 {
                        bForceSelection = 0;
                        if (*descr).focus_item >= (*descr).page_size {
                            caret = (*descr).focus_item - (*descr).page_size;
                        }
                        break;
                    }
                    caret = (*descr).focus_item - 1;
                    if caret < 0 {
                        caret = 0;
                    }
                    break;
                }
                VK_UP => {
                    caret = (*descr).focus_item - 1;
                    if caret < 0 {
                        caret = 0;
                    }
                    break;
                }
                VK_RIGHT => {
                    if (*descr).style & LBS_MULTICOLUMN != 0 {
                        bForceSelection = 0;
                        if (*descr).focus_item + (*descr).page_size < (*descr).nb_items {
                            caret = (*descr).focus_item + (*descr).page_size;
                        }
                        break;
                    }
                    caret = (*descr).focus_item + 1;
                    if caret >= (*descr).nb_items {
                        caret = (*descr).nb_items - 1;
                    }
                    break;
                }
                VK_DOWN => {
                    caret = (*descr).focus_item + 1;
                    if caret >= (*descr).nb_items {
                        caret = (*descr).nb_items - 1;
                    }
                    break;
                }
                VK_PRIOR => {
                    if (*descr).style & LBS_MULTICOLUMN != 0 {
                        let mut page = (*descr).width / (*descr).column_width;
                        if page < 1 {
                            page = 1;
                        }
                        caret = (*descr).focus_item - (page * (*descr).page_size) + 1;
                    } else {
                        caret = (*descr).focus_item - LISTBOX_GetCurrentPageSize(descr) + 1;
                    }
                    if caret < 0 {
                        caret = 0;
                    }
                    break;
                }
                VK_NEXT => {
                    if (*descr).style & LBS_MULTICOLUMN != 0 {
                        let mut page = (*descr).width / (*descr).column_width;
                        if page < 1 {
                            page = 1;
                        }
                        caret = (*descr).focus_item + (page * (*descr).page_size) - 1;
                    } else {
                        caret = (*descr).focus_item + LISTBOX_GetCurrentPageSize(descr) - 1;
                    }
                    if caret >= (*descr).nb_items {
                        caret = (*descr).nb_items - 1;
                    }
                    break;
                }
                VK_HOME => {
                    caret = 0;
                    break;
                }
                VK_END => {
                    caret = (*descr).nb_items - 1;
                    break;
                }
                VK_SPACE => {
                    if (*descr).style & LBS_EXTENDEDSEL != 0 {
                        caret = (*descr).focus_item;
                    } else if (*descr).style & LBS_MULTIPLESEL != 0 {
                        LISTBOX_SetSelection(
                            descr,
                            (*descr).focus_item,
                            ((*(*descr).items.offset((*descr).focus_item as isize)).selected == 0)
                                as BOOL,
                            ((*descr).style & LBS_NOTIFY != 0) as BOOL,
                        );
                    }
                    break;
                }
                _ => {
                    bForceSelection = 0;
                    break;
                }
            }
        }
    }

    if bForceSelection != 0 {
        caret = (*descr).focus_item;
    }
    if caret >= 0 {
        if (((*descr).style & LBS_EXTENDEDSEL != 0)
            && (GetKeyState(VK_SHIFT as i32) as i32 & 0x8000u16 as i16 as i32) == 0)
            || !(((*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL)) != 0
                && ((*descr).style & LBS_NOSEL) == 0)
        {
            (*descr).anchor_item = caret;
        }
        LISTBOX_MoveCaret(descr, caret, 1);

        if (*descr).style & LBS_MULTIPLESEL != 0 {
            (*descr).selected_item = caret;
        } else {
            LISTBOX_SetSelection(descr, caret, 1, 0);
        }
        if (*descr).style & LBS_NOTIFY != 0 {
            if !(*descr).lphc.is_null() && IsWindowVisible((*descr).self_) != 0 {
                (*(*descr).lphc).wState |= CBF_NOROLLUP;
            }
            if (*descr).nb_items != 0 {
                SendMessageW(
                    (*descr).owner,
                    WM_COMMAND,
                    crate::internal::MAKEWPARAM(
                        GetWindowLongPtrW((*descr).self_, GWLP_ID) as WORD,
                        LBN_SELCHANGE as WORD,
                    ) as WPARAM,
                    (*descr).self_ as LPARAM,
                );
            }
        }
    }
    0
}

unsafe fn LISTBOX_HandleChar(descr: *mut LB_DESCR, charW: WCHAR) -> LRESULT {
    let mut caret: INT = -1;
    let mut str: [WCHAR; 2] = [0; 2];

    str[0] = charW;
    str[1] = 0;

    if (*descr).style & LBS_WANTKEYBOARDINPUT != 0 {
        caret = SendMessageW(
            (*descr).owner,
            WM_CHARTOITEM,
            crate::internal::MAKEWPARAM(charW as WORD, (*descr).focus_item as WORD) as WPARAM,
            (*descr).self_ as LPARAM,
        ) as INT;
        if caret == -2 {
            return 0;
        }
    }
    if caret == -1 {
        caret = LISTBOX_FindString(descr, (*descr).focus_item, str.as_ptr(), 0);
    }
    if caret != -1 {
        if (IS_MULTISELECT(descr) == 0) && (*descr).selected_item == -1 {
            LISTBOX_SetSelection(descr, caret, 1, 0);
        }
        LISTBOX_MoveCaret(descr, caret, 1);
        if ((*descr).style & LBS_NOTIFY != 0) && (*descr).nb_items != 0 {
            SendMessageW(
                (*descr).owner,
                WM_COMMAND,
                crate::internal::MAKEWPARAM(
                    GetWindowLongPtrW((*descr).self_, GWLP_ID) as WORD,
                    LBN_SELCHANGE as WORD,
                ) as WPARAM,
                (*descr).self_ as LPARAM,
            );
        }
    }
    0
}

unsafe fn LISTBOX_update_uistate(descr: *mut LB_DESCR) -> BOOL {
    let prev_flags: LONG;

    prev_flags = (*descr).UIState;
    (*descr).UIState = DefWindowProcW((*descr).self_, WM_QUERYUISTATE, 0, 0) as LONG;
    (prev_flags != (*descr).UIState) as BOOL
}

unsafe fn LISTBOX_Create(hwnd: HWND, lphc: LPHEADCOMBO) -> BOOL {
    let descr: *mut LB_DESCR;
    let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();

    descr = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<LB_DESCR>()) as *mut LB_DESCR;
    if descr.is_null() {
        return 0u8 as BOOL;
    }

    GetClientRect(hwnd, &mut rect);
    (*descr).self_ = hwnd;
    (*descr).owner = GetParent((*descr).self_);
    (*descr).style = GetWindowLongPtrW((*descr).self_, GWL_STYLE) as UINT;
    (*descr).width = rect.right - rect.left;
    (*descr).height = rect.bottom - rect.top;
    (*descr).items = core::ptr::null_mut();
    (*descr).nb_items = 0;
    (*descr).top_item = 0;
    (*descr).selected_item = -1;
    (*descr).focus_item = 0;
    (*descr).anchor_item = -1;
    (*descr).item_height = 1;
    (*descr).page_size = 1;
    (*descr).column_width = 150;
    (*descr).horz_extent = 0;
    (*descr).horz_pos = 0;
    (*descr).nb_tabs = 0;
    (*descr).tabs = core::ptr::null_mut();
    (*descr).wheel_remain = 0;
    (*descr).caret_on = if lphc.is_null() { 1 } else { 0 };
    if (*descr).style & LBS_NOSEL != 0 {
        (*descr).caret_on = 0;
    }
    (*descr).in_focus = 0;
    (*descr).captured = 0;
    (*descr).font = 0 as HFONT;
    (*descr).locale = GetUserDefaultLCID();
    (*descr).lphc = lphc;

    if !lphc.is_null() {
        TRACE!(
            b"[%p]: resetting owner %p -> %p\n\0".as_ptr() as *const i8,
            (*descr).self_,
            (*descr).owner,
            (*lphc).self_,
        );
        (*descr).owner = (*lphc).self_;
    }

    SetWindowLongPtrW((*descr).self_, 0, descr as LONG_PTR);

    LISTBOX_update_uistate(descr);

    if (*descr).style & LBS_EXTENDEDSEL != 0 {
        (*descr).style |= LBS_MULTIPLESEL;
    }
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        (*descr).style &= !LBS_OWNERDRAWVARIABLE;
    }
    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        (*descr).style |= LBS_NOINTEGRALHEIGHT;
    }

    if (*descr).style & LBS_NODATA != 0
        && (((*descr).style & LBS_OWNERDRAWFIXED) == 0
            || (*descr).style & (LBS_HASSTRINGS | LBS_SORT) != 0)
    {
        (*descr).style &= !LBS_NODATA;
    }

    (*descr).item_height = LISTBOX_SetFont(descr, 0 as HFONT);

    if (*descr).style & LBS_OWNERDRAWFIXED != 0 {
        if !(*descr).lphc.is_null() && ((*(*descr).lphc).dwStyle & CBS_DROPDOWN) != 0 {
            (*descr).item_height = (*lphc).fixedOwnerDrawHeight as INT;
        } else {
            let id = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;
            mis.CtlType = ODT_LISTBOX as UINT;
            mis.CtlID = id;
            mis.itemID = -1i32 as UINT;
            mis.itemWidth = 0;
            mis.itemData = 0;
            mis.itemHeight = (*descr).item_height as UINT;
            SendMessageW((*descr).owner, WM_MEASUREITEM, id as WPARAM, &mut mis as *mut _ as LPARAM);
            (*descr).item_height = if mis.itemHeight != 0 {
                mis.itemHeight as INT
            } else {
                1
            };
        }
    }

    TRACE!(
        b"owner: %p, style: %08x, width: %d, height: %d\n\0".as_ptr() as *const i8,
        (*descr).owner,
        (*descr).style,
        (*descr).width,
        (*descr).height,
    );
    1u8 as BOOL
}

unsafe fn LISTBOX_Destroy(descr: *mut LB_DESCR) -> BOOL {
    LISTBOX_ResetContent(descr);
    SetWindowLongPtrW((*descr).self_, 0, 0);
    HeapFree(GetProcessHeap(), 0, descr as *mut _);
    1
}

#[no_mangle]
pub unsafe extern "system" fn ListBoxWndProc_common(
    hwnd: HWND,
    msg: UINT,
    mut wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let descr: *mut LB_DESCR = GetWindowLongPtrW(hwnd, 0) as *mut LB_DESCR;
    let mut lphc: LPHEADCOMBO = core::ptr::null_mut();
    let mut ret: LRESULT;

    let pWnd: PWND = crate::internal::ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_LISTBOX as WORD);
        } else {
            if (*pWnd).fnid != FNID_LISTBOX as DWORD {
                ERR!("Wrong window class for listbox! fnId 0x%x\n", (*pWnd).fnid as UINT);
                return 0;
            }
        }
    }

    if descr.is_null() {
        if IsWindow(hwnd) == 0 {
            return 0;
        }

        if msg == WM_CREATE {
            let lpcs = lParam as *mut CREATESTRUCTW;
            if (*lpcs).style & LBS_COMBOBOX as LONG != 0 {
                lphc = (*lpcs).lpCreateParams as LPHEADCOMBO;
            }
            if LISTBOX_Create(hwnd, lphc) == 0 {
                return -1;
            }
            return 0;
        }
        return if unicode != 0 {
            DefWindowProcW(hwnd, msg, wParam, lParam)
        } else {
            DefWindowProcA(hwnd, msg, wParam, lParam)
        };
    }

    if (*descr).style & LBS_COMBOBOX != 0 {
        lphc = (*descr).lphc;
    }

    match msg {
        LB_RESETCONTENT => {
            LISTBOX_ResetContent(descr);
            LISTBOX_UpdateScroll(descr);
            InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
            return 0;
        }

        LB_ADDSTRING | LB_ADDSTRING_LOWER | LB_ADDSTRING_UPPER => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                } else {
                    return LB_ERRSPACE as LRESULT;
                }
            }
            if msg == LB_ADDSTRING_LOWER {
                strlwrW(textW);
            } else if msg == LB_ADDSTRING_UPPER {
                struprW(textW);
            }
            wParam = LISTBOX_FindStringPos(descr, textW, 0) as WPARAM;
            ret = LISTBOX_InsertString(descr, wParam as INT, textW) as INT;
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_INSERTSTRING | LB_INSERTSTRING_UPPER | LB_INSERTSTRING_LOWER => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                } else {
                    return LB_ERRSPACE as LRESULT;
                }
            }
            if msg == LB_INSERTSTRING_LOWER {
                strlwrW(textW);
            } else if msg == LB_INSERTSTRING_UPPER {
                struprW(textW);
            }
            ret = LISTBOX_InsertString(descr, wParam as INT, textW) as INT;
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_ADDFILE => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                } else {
                    return LB_ERRSPACE as LRESULT;
                }
            }
            wParam = LISTBOX_FindFileStrPos(descr, textW) as WPARAM;
            ret = LISTBOX_InsertString(descr, wParam as INT, textW) as INT;
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_DELETESTRING => {
            if LISTBOX_RemoveItem(descr, wParam as INT) != LB_ERR as LRESULT {
                return (*descr).nb_items as LRESULT;
            } else {
                SetLastError(ERROR_INVALID_INDEX);
                return LB_ERR as LRESULT;
            }
        }

        LB_GETITEMDATA => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX);
                return LB_ERR as LRESULT;
            }
            return (*(*descr).items.offset(wParam as isize)).data as LRESULT;
        }

        LB_SETITEMDATA => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX);
                return LB_ERR as LRESULT;
            }
            (*(*descr).items.offset(wParam as isize)).data = lParam as ULONG_PTR;
            return 1;
        }

        LB_GETCOUNT => {
            return (*descr).nb_items as LRESULT;
        }

        LB_GETTEXT => {
            return LISTBOX_GetText(descr, wParam as INT, lParam as LPWSTR, unicode);
        }

        LB_GETTEXTLEN => {
            if (wParam as INT) >= (*descr).nb_items || (wParam as INT) < 0 {
                SetLastError(ERROR_INVALID_INDEX);
                return LB_ERR as LRESULT;
            }
            if HAS_STRINGS(descr) == 0 {
                return core::mem::size_of::<DWORD>() as LRESULT;
            }
            if unicode != 0 {
                return strlenW((*(*descr).items.offset(wParam as isize)).str_) as LRESULT;
            }
            return WideCharToMultiByte(
                CP_ACP, 0,
                (*(*descr).items.offset(wParam as isize)).str_,
                strlenW((*(*descr).items.offset(wParam as isize)).str_) as INT,
                core::ptr::null_mut(), 0,
                core::ptr::null(), core::ptr::null_mut(),
            ) as LRESULT;
        }

        LB_GETCURSEL => {
            if (*descr).nb_items == 0 {
                return LB_ERR as LRESULT;
            }
            if IS_MULTISELECT(descr) == 0 {
                return (*descr).selected_item as LRESULT;
            }
            if (*descr).selected_item != -1 {
                return (*descr).selected_item as LRESULT;
            }
            return (*descr).focus_item as LRESULT;
        }

        LB_GETTOPINDEX => {
            return (*descr).top_item as LRESULT;
        }

        LB_GETITEMHEIGHT => {
            return LISTBOX_GetItemHeight(descr, wParam as INT);
        }

        LB_SETITEMHEIGHT => {
            return LISTBOX_SetItemHeight(descr, wParam as INT, lParam as INT, 1);
        }

        LB_ITEMFROMPOINT => {
            let mut pt: POINT = core::mem::zeroed();
            let mut rect: RECT = core::mem::zeroed();
            let mut index: i32;
            let mut hit: BOOL = 1;

            if (*descr).nb_items == 0 {
                return 0x1ffff;
            }

            pt.x = (crate::internal::LOWORD(lParam as DWORD) as i16) as LONG;
            pt.y = (crate::internal::HIWORD(lParam as DWORD) as i16) as LONG;

            SetRect(&mut rect, 0, 0, (*descr).width, (*descr).height);

            if PtInRect(&rect, pt) == 0 {
                pt.x = min(pt.x, rect.right - 1);
                pt.x = max(pt.x, 0);
                pt.y = min(pt.y, rect.bottom - 1);
                pt.y = max(pt.y, 0);
                hit = 0;
            }

            index = LISTBOX_GetItemFromPoint(descr, pt.x, pt.y);

            if index == -1 {
                index = (*descr).nb_items - 1;
                hit = 0;
            }
            return crate::internal::MAKELONG(index as WORD, if hit != 0 { 0 } else { 1 }) as LRESULT;
        }

        LB_SETCARETINDEX => {
            if IS_MULTISELECT(descr) == 0 && (*descr).selected_item != -1 {
                return LB_ERR as LRESULT;
            }
            if LISTBOX_SetCaretIndex(descr, wParam as INT, (lParam == 0) as INT) == LB_ERR as LRESULT {
                return LB_ERR as LRESULT;
            } else if ISWIN31() {
                return wParam as LRESULT;
            } else {
                return LB_OKAY as LRESULT;
            }
        }

        LB_GETCARETINDEX => {
            return (*descr).focus_item as LRESULT;
        }

        LB_SETTOPINDEX => {
            return LISTBOX_SetTopItem(descr, wParam as INT, 1);
        }

        LB_SETCOLUMNWIDTH => {
            return LISTBOX_SetColumnWidth(descr, wParam as INT);
        }

        LB_GETITEMRECT => {
            return LISTBOX_GetItemRect(descr, wParam as INT, lParam as *mut RECT);
        }

        LB_FINDSTRING => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                }
            }
            ret = LISTBOX_FindString(descr, wParam as INT, textW, 0);
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_FINDSTRINGEXACT => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                }
            }
            ret = LISTBOX_FindString(descr, wParam as INT, textW, 1);
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_SELECTSTRING => {
            let index: INT;
            let textW: LPWSTR;
            if unicode != 0 || HAS_STRINGS(descr) == 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                }
            }
            index = LISTBOX_FindString(descr, wParam as INT, textW, 0);
            if unicode == 0 && HAS_STRINGS(descr) != 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            if index != LB_ERR {
                LISTBOX_MoveCaret(descr, index, 1);
                LISTBOX_SetSelection(descr, index, 1, 0);
            }
            return index as LRESULT;
        }

        LB_GETSEL => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                return LB_ERR as LRESULT;
            }
            return (*(*descr).items.offset(wParam as isize)).selected as LRESULT;
        }

        LB_SETSEL => {
            return LISTBOX_SetSelection(descr, lParam as INT, wParam as INT, 0);
        }

        LB_SETCURSEL => {
            if IS_MULTISELECT(descr) != 0 {
                return LB_ERR as LRESULT;
            }
            LISTBOX_SetCaretIndex(descr, wParam as INT, 1);
            ret = LISTBOX_SetSelection(descr, wParam as INT, 1, 0);
            if ret != LB_ERR as LRESULT {
                ret = (*descr).selected_item as LRESULT;
            }
            return ret;
        }

        LB_GETSELCOUNT => {
            return LISTBOX_GetSelCount(descr);
        }

        LB_GETSELITEMS => {
            return LISTBOX_GetSelItems(descr, wParam as INT, lParam as LPINT);
        }

        LB_SELITEMRANGE => {
            if crate::internal::LOWORD(lParam as DWORD) <= crate::internal::HIWORD(lParam as DWORD) {
                return LISTBOX_SelectItemRange(descr, crate::internal::LOWORD(lParam as DWORD) as INT, crate::internal::HIWORD(lParam as DWORD) as INT, wParam as INT);
            } else {
                return LISTBOX_SelectItemRange(descr, crate::internal::HIWORD(lParam as DWORD) as INT, crate::internal::LOWORD(lParam as DWORD) as INT, wParam as INT);
            }
        }

        LB_SELITEMRANGEEX => {
            if (lParam as INT) >= (wParam as INT) {
                return LISTBOX_SelectItemRange(descr, wParam as INT, lParam as INT, 1);
            } else {
                return LISTBOX_SelectItemRange(descr, lParam as INT, wParam as INT, 0);
            }
        }

        LB_GETHORIZONTALEXTENT => {
            return (*descr).horz_extent as LRESULT;
        }

        LB_SETHORIZONTALEXTENT => {
            return LISTBOX_SetHorizontalExtent(descr, wParam as INT);
        }

        LB_GETANCHORINDEX => {
            return (*descr).anchor_item as LRESULT;
        }

        LB_SETANCHORINDEX => {
            if (wParam as INT) < -1 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX);
                return LB_ERR as LRESULT;
            }
            (*descr).anchor_item = wParam as INT;
            return LB_OKAY as LRESULT;
        }

        LB_DIR => {
            let ret: INT;
            let textW: LPWSTR;
            if unicode != 0 {
                textW = lParam as LPWSTR;
            } else {
                let textA = lParam as LPSTR;
                let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                if !textW.is_null() {
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                }
            }
            ret = LISTBOX_Directory(descr, wParam as UINT, textW, (msg == LB_DIR) as INT) as INT;
            if unicode == 0 {
                HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
            }
            return ret as LRESULT;
        }

        LB_GETLOCALE => {
            return (*descr).locale as LRESULT;
        }

        LB_SETLOCALE => {
            let ret: LCID;
            if IsValidLocale(wParam as LCID, LCID_INSTALLED) == 0 {
                return LB_ERR as LRESULT;
            }
            ret = (*descr).locale;
            (*descr).locale = wParam as LCID;
            return ret as LRESULT;
        }

        LB_INITSTORAGE => {
            return LISTBOX_InitStorage(descr, wParam as INT);
        }

        LB_SETCOUNT => {
            return LISTBOX_SetCount(descr, wParam as INT);
        }

        LB_SETTABSTOPS => {
            return LISTBOX_SetTabStops(descr, wParam as INT, lParam as LPINT) as LRESULT;
        }

        LB_CARETON => {
            if (*descr).caret_on != 0 {
                return LB_OKAY as LRESULT;
            }
            (*descr).caret_on = 1;
            if (*descr).focus_item != -1 && (*descr).in_focus != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            return LB_OKAY as LRESULT;
        }

        LB_CARETOFF => {
            if (*descr).caret_on == 0 {
                return LB_OKAY as LRESULT;
            }
            (*descr).caret_on = 0;
            if (*descr).focus_item != -1 && (*descr).in_focus != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            return LB_OKAY as LRESULT;
        }

        LB_GETLISTBOXINFO => {
            return (*descr).page_size as LRESULT;
        }

        WM_DESTROY => {
            return LISTBOX_Destroy(descr) as LRESULT;
        }

        WM_ENABLE => {
            InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
            return 0;
        }

        WM_SETREDRAW => {
            LISTBOX_SetRedraw(descr, (wParam != 0) as BOOL);
            return 0;
        }

        WM_GETDLGCODE => {
            return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hdc: HDC = if wParam != 0 {
                wParam as HDC
            } else {
                BeginPaint((*descr).self_, &mut ps)
            };
            ret = LISTBOX_Paint(descr, hdc);
            if wParam == 0 {
                EndPaint((*descr).self_, &mut ps);
            }
            return ret;
        }

        WM_SIZE => {
            LISTBOX_UpdateSize(descr);
            return 0;
        }

        WM_GETFONT => {
            return (*descr).font as LRESULT;
        }

        WM_SETFONT => {
            LISTBOX_SetFont(descr, wParam as HFONT);
            if lParam != 0 {
                InvalidateRect((*descr).self_, core::ptr::null_mut(), 1);
            }
            return 0;
        }

        WM_SETFOCUS => {
            (*descr).in_focus = 1;
            (*descr).caret_on = 1;
            if (*descr).focus_item != -1 {
                LISTBOX_DrawFocusRect(descr, 1);
            }
            SEND_NOTIFICATION(descr, LBN_SETFOCUS as INT);
            return 0;
        }

        WM_KILLFOCUS => {
            LISTBOX_HandleLButtonUp(descr);
            (*descr).in_focus = 0;
            (*descr).wheel_remain = 0;
            if (*descr).focus_item != -1 && (*descr).caret_on != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            SEND_NOTIFICATION(descr, LBN_KILLFOCUS as INT);
            return 0;
        }

        WM_HSCROLL => {
            return LISTBOX_HandleHScroll(descr, crate::internal::LOWORD(wParam as DWORD), crate::internal::HIWORD(wParam as DWORD));
        }

        WM_VSCROLL => {
            return LISTBOX_HandleVScroll(descr, crate::internal::LOWORD(wParam as DWORD), crate::internal::HIWORD(wParam as DWORD));
        }

        WM_MOUSEWHEEL => {
            if wParam & (MK_SHIFT | MK_CONTROL) as WPARAM != 0 {
                return DefWindowProcW((*descr).self_, msg, wParam, lParam);
            }
            return LISTBOX_HandleMouseWheel(descr, crate::internal::HIWORD(wParam as DWORD) as SHORT);
        }

        WM_LBUTTONDOWN => {
            if !lphc.is_null() {
                return LISTBOX_HandleLButtonDownCombo(descr, msg, wParam as DWORD, crate::internal::LOWORD(lParam as DWORD) as i16 as INT, crate::internal::HIWORD(lParam as DWORD) as i16 as INT);
            }
            return LISTBOX_HandleLButtonDown(descr, wParam as DWORD, crate::internal::LOWORD(lParam as DWORD) as i16 as INT, crate::internal::HIWORD(lParam as DWORD) as i16 as INT);
        }

        WM_LBUTTONDBLCLK => {
            if !lphc.is_null() {
                return LISTBOX_HandleLButtonDownCombo(descr, msg, wParam as DWORD, crate::internal::LOWORD(lParam as DWORD) as i16 as INT, crate::internal::HIWORD(lParam as DWORD) as i16 as INT);
            }
            if (*descr).style & LBS_NOTIFY != 0 {
                SEND_NOTIFICATION(descr, LBN_DBLCLK as INT);
            }
            return 0;
        }

        WM_MOUSEMOVE => {
            if !lphc.is_null() && ((*lphc).dwStyle & CBS_DROPDOWNLIST) != CBS_SIMPLE {
                let captured: BOOL = (*descr).captured;
                let mut mousePos: POINT = core::mem::zeroed();
                let mut clientRect: RECT = core::mem::zeroed();

                mousePos.x = (crate::internal::LOWORD(lParam as DWORD) as i16) as LONG;
                mousePos.y = (crate::internal::HIWORD(lParam as DWORD) as i16) as LONG;

                if GetClientRect((*descr).self_, &mut clientRect) != 0 && PtInRect(&clientRect, mousePos) != 0 {
                    (*descr).captured = 1;
                }

                LISTBOX_HandleMouseMove(descr, mousePos.x, mousePos.y);

                (*descr).captured = captured;
            } else if GetCapture() == (*descr).self_ {
                LISTBOX_HandleMouseMove(descr, crate::internal::LOWORD(lParam as DWORD) as i16 as INT, crate::internal::HIWORD(lParam as DWORD) as i16 as INT);
            }
            return 0;
        }

        WM_LBUTTONUP => {
            if !lphc.is_null() {
                let mut mousePos: POINT = core::mem::zeroed();
                let mut clientRect: RECT = core::mem::zeroed();

                mousePos.x = (crate::internal::LOWORD(lParam as DWORD) as i16) as LONG;
                mousePos.y = (crate::internal::HIWORD(lParam as DWORD) as i16) as LONG;

                GetClientRect((*descr).self_, &mut clientRect);

                if lParam == -1isize as LPARAM || PtInRect(&clientRect, mousePos) == 0 {
                    LISTBOX_MoveCaret(descr, (*lphc).droppedIndex, 0);
                }
            }
            return LISTBOX_HandleLButtonUp(descr);
        }

        WM_KEYDOWN => {
            if !lphc.is_null() && ((*lphc).dwStyle & CBS_DROPDOWNLIST) != CBS_SIMPLE {
                if (((*lphc).wState & CBF_EUI) == 0 && wParam == VK_F4 as WPARAM)
                    || (((*lphc).wState & CBF_EUI) != 0 && ((*lphc).wState & CBF_DROPPED) == 0
                        && (wParam == VK_DOWN as WPARAM || wParam == VK_UP as WPARAM))
                {
                    COMBO_FlipListbox(lphc, 0, 0);
                    return 0;
                }
            }
            return LISTBOX_HandleKeyDown(descr, wParam as DWORD);
        }

        WM_CHAR => {
            let mut charW: WCHAR;
            if unicode != 0 {
                charW = wParam as WCHAR;
            } else {
                let charA: CHAR = wParam as CHAR;
                charW = 0;
                MultiByteToWideChar(CP_ACP, 0, &charA, 1, &mut charW, 1);
            }
            return LISTBOX_HandleChar(descr, charW);
        }

        WM_SYSTIMER => {
            return LISTBOX_HandleSystemTimer(descr);
        }

        WM_ERASEBKGND => {
            if IS_OWNERDRAW(descr) != 0 && (*descr).style & (LBS_DISPLAYCHANGED as UINT) == 0 {
                let mut rect: RECT = core::mem::zeroed();
                let mut hbrush: HBRUSH = GetControlColor((*descr).owner, (*descr).self_, wParam as HDC, WM_CTLCOLORLISTBOX);
                if hbrush.is_null() {
                    hbrush = GetSysColorBrush(COLOR_WINDOW as INT);
                }
                if !hbrush.is_null() {
                    GetClientRect((*descr).self_, &mut rect);
                    FillRect(wParam as HDC, &rect, hbrush);
                }
            }
            return 1;
        }

        WM_DROPFILES => {
            if !lphc.is_null() {
                return 0;
            }
            return if unicode != 0 {
                SendMessageW((*descr).owner, msg, wParam, lParam)
            } else {
                SendMessageA((*descr).owner, msg, wParam, lParam)
            };
        }

        WM_NCDESTROY => {
            if !lphc.is_null() && ((*lphc).dwStyle & CBS_DROPDOWNLIST) != CBS_SIMPLE {
                (*lphc).hWndLBox = core::ptr::null_mut();
            }
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as WORD);
        }

        WM_NCACTIVATE => {
            if !lphc.is_null() {
                return 0;
            }
        }

        WM_UPDATEUISTATE => {
            if unicode != 0 {
                DefWindowProcW((*descr).self_, msg, wParam, lParam);
            } else {
                DefWindowProcA((*descr).self_, msg, wParam, lParam);
            }

            if LISTBOX_update_uistate(descr) != 0 {
                if (*descr).focus_item != -1 {
                    LISTBOX_DrawFocusRect(descr, (*descr).in_focus);
                }
            }
        }

        _ => {
            if msg >= WM_USER && msg < 0xc000 {
                WARN!(b"[%p]: unknown msg %04x wp %08lx lp %08lx\n\0".as_ptr() as *const i8, hwnd, msg, wParam, lParam);
            }
        }
    }

    return if unicode != 0 {
        DefWindowProcW(hwnd, msg, wParam, lParam)
    } else {
        DefWindowProcA(hwnd, msg, wParam, lParam)
    };
}

#[no_mangle]
pub unsafe extern "system" fn ListBoxWndProcA(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    ListBoxWndProc_common(hwnd, msg, wParam, lParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn ListBoxWndProcW(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    ListBoxWndProc_common(hwnd, msg, wParam, lParam, 1)
}
