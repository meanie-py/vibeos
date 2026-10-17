use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::winbase::ExtTextOutW;
use crate::windows::window::HeapAlloc;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
use crate::windows::rect::IsRectEmpty;
use crate::misc::display::MonitorFromRect;
#[allow(unused_imports)]
use core::ffi::c_void;

// Shared combo/listbox surface (HEADCOMBO, LPHEADCOMBO, CBF_*, LB_*_LOWER/UPPER,
// LB_CARETON/OFF) — sourced from win32ss/user/user32/include/controls.h.
use crate::controls::shared::*;
// Explicit single-name imports win over the two `*` glob imports above, resolving
// the LB_CARETON/LB_CARETOFF ambiguity (both reach this module via glob).
use win32k_sys::win32k::{LB_CARETOFF, LB_CARETON};

// internal helpers (LOWORD/HIWORD/MAKEWPARAM/ValidateHwnd) + control-color wrapper.
use crate::internal::ValidateHwnd;
use crate::ntwrapper::GetControlColor;
// NtUser* syscall forwarders (ntwrapper.h EXTINLINE wrappers), not linkable exports.
use crate::ntwrapper::{
    BeginPaint, DestroyWindow, EndPaint, GetDC, InvalidateRect, RedrawWindow, SetCapture, SetFocus,
    SetParent, SetWindowPos, ShowWindow,
};
// SetWindowLongPtrW is a winuser.h macro for SetWindowLongW on x86; class.rs only
// defines a real (isize-typed) SetWindowLongPtrW for non-x86. cfg-split.
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

// Wine debug-channel stubs (WINE_DEFAULT_DEBUG_CHANNEL(combo)) — no-ops.
// debugstr_w / wine_dbgstr_rect — used only inside the no-op trace macros; provide
// dummy implementations so any stray call site still type-checks.
#[inline]
unsafe fn debugstr_w(_s: LPCWSTR) -> *const i8 { b"\0".as_ptr() as *const i8 }
#[inline]
unsafe fn wine_dbgstr_rect(_r: *const RECT) -> *const i8 { b"\0".as_ptr() as *const i8 }

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const CP_ACP: UINT = 0;

// controls.h listbox-child control IDs.
const ID_CB_LISTBOX: i32 = 1000;
const ID_CB_EDIT: i32 = 1001;

// winuser.h helper macros not present in the generated bindings.
#[inline]
fn MAKEINTRESOURCEW(i: WORD) -> LPCWSTR {
    i as usize as LPCWSTR
}
#[inline]
fn GET_WHEEL_DELTA_WPARAM(wParam: WPARAM) -> i16 {
    crate::internal::HIWORD(wParam as DWORD) as i16
}
// WM_CTLCOLOR (legacy, 0x0019) — not emitted by the generated bindings.
const WM_CTLCOLOR: UINT = 0x0019;
// Special HWND sentinels.
const HWND_DESKTOP: HWND = 0 as HWND;
const HWND_TOPMOST: HWND = (-1isize) as HWND;
// IDC_ARROW resource id.
const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;

#[inline]
fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } }
#[inline]
fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }

// ───────────────────────── Win32 API imports ─────────────────────────────────
// Resolved at link via the user32/gdi32/kernel32/ntdll import libs. stdcall
// WINAPI everywhere → extern "system" (i386 → _Name@N).
extern "system" {
    // kernel32 — process heap + string helpers
    fn GetProcessHeap() -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void) -> i32;
    fn GetVersion() -> DWORD;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: INT) -> LPSTR;

    // user32 own-exports
    fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: INT, Y: INT, nWidth: INT, nHeight: INT, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: INT, Y: INT, nWidth: INT, nHeight: INT, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    fn UpdateWindow(hWnd: HWND) -> BOOL;
    fn GetCapture() -> HWND;
    fn ReleaseCapture() -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT;
    fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    fn MapWindowPoints(hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT) -> INT;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn GetSysColor(nIndex: i32) -> COLORREF;
    fn GetSysColorBrush(nIndex: i32) -> HBRUSH;
    fn LoadBitmapW(hInstance: HINSTANCE, lpBitmapName: LPCWSTR) -> HBITMAP;
    fn CharLowerW(lpsz: LPWSTR) -> LPWSTR;
    fn CharUpperW(lpsz: LPWSTR) -> LPWSTR;
    fn CharLowerA(lpsz: LPSTR) -> LPSTR;
    fn CharUpperA(lpsz: LPSTR) -> LPSTR;
    fn GetMonitorInfoW(hMonitor: HMONITOR, lpmi: *mut MONITORINFO) -> BOOL;

    // rect helpers (user32)
    fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;
    fn SetRect(lprc: *mut RECT, xLeft: i32, yTop: i32, xRight: i32, yBottom: i32) -> BOOL;
    fn InflateRect(lprc: *mut RECT, dx: i32, dy: i32) -> BOOL;
    fn CopyRect(lprcDst: *mut RECT, lprcSrc: *const RECT) -> BOOL;
    fn UnionRect(lprcDst: *mut RECT, lprcSrc1: *const RECT, lprcSrc2: *const RECT) -> BOOL;
    fn DrawFocusRect(hDC: HDC, lprc: *const RECT) -> BOOL;
    fn DrawFrameControl(hdc: HDC, lprc: *mut RECT, uType: UINT, uState: UINT) -> BOOL;
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn FrameRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;

    // gdi32 imports
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn SelectObject(hdc: HDC, h: *mut c_void) -> *mut c_void;
    fn DeleteObject(ho: *mut c_void) -> i32;
    fn GetObjectW(h: *mut c_void, c: i32, pv: *mut c_void) -> i32;
    fn GetTextMetricsW(hdc: HDC, lptm: *mut TEXTMETRICW) -> BOOL;
    fn InvertRect(hDC: HDC, lprc: *const RECT) -> BOOL;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> i32;
}

// `set_control_clipping` — surviving C symbol in user32 (controls/uitools.c), cdecl.
// `strlen`/`memcpy` — CRT, cdecl.
extern "C" {
    fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN;
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// ---- combo-local helper macros (combo.c top + include/controls.h) ----
// CB_NOTIFY( lphc, code )
#[inline]
unsafe fn CB_NOTIFY(lphc: LPHEADCOMBO, code: INT) -> LRESULT {
    SendMessageW(
        (*lphc).owner,
        WM_COMMAND,
        crate::internal::MAKEWPARAM(
            GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
            code as WORD,
        ) as WPARAM,
        (*lphc).self_ as LPARAM,
    )
}
// CB_DISABLED( lphc )   (!IsWindowEnabled((lphc)->self))
#[inline]
unsafe fn CB_DISABLED(lphc: LPHEADCOMBO) -> BOOL {
    (IsWindowEnabled((*lphc).self_) == 0) as BOOL
}
// CB_OWNERDRAWN( lphc ) ((lphc)->dwStyle & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE))
#[inline]
unsafe fn CB_OWNERDRAWN(lphc: LPHEADCOMBO) -> UINT {
    (*lphc).dwStyle as UINT & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE)
}
// CB_HASSTRINGS( lphc ) ((lphc)->dwStyle & CBS_HASSTRINGS)
#[inline]
unsafe fn CB_HASSTRINGS(lphc: LPHEADCOMBO) -> UINT {
    (*lphc).dwStyle as UINT & CBS_HASSTRINGS
}
// CB_GETTYPE( lphc )    ((lphc)->dwStyle & (CBS_DROPDOWNLIST))   (include/controls.h)
#[inline]
unsafe fn CB_GETTYPE(lphc: *const HEADCOMBO) -> UINT {
    (*lphc).dwStyle as UINT & CBS_DROPDOWNLIST
}
// Look-and-feel dependent "constants"
#[inline]
fn COMBO_XBORDERSIZE() -> INT {
    2
}
#[inline]
fn COMBO_YBORDERSIZE() -> INT {
    2
}
#[inline]
fn COMBO_EDITBUTTONSPACE() -> INT {
    0
}
#[inline]
fn EDIT_CONTROL_PADDING() -> INT {
    1
}
// ISWIN31 := (LOWORD(GetVersion()) == 0x0a03)
#[inline]
unsafe fn ISWIN31() -> bool {
    crate::internal::LOWORD(GetVersion()) == 0x0a03
}
// WIN_GetFullHandle(h) := ((HWND)(h))   (user_x.h)
#[no_mangle]
pub unsafe extern "C" fn WIN_GetFullHandle(h: HWND) -> HWND {
    h
}

// ---- file-scope preamble (translated) ----
pub static mut hComboBmp: HBITMAP = 0 as HBITMAP;
pub static mut CBitHeight: UINT = 0;
pub static mut CBitWidth: UINT = 0;

pub const COMBO_YBORDERGAP: i32 = 5;

pub static comboboxW: [WCHAR; 9] = [
    'C' as WCHAR, 'o' as WCHAR, 'm' as WCHAR, 'b' as WCHAR, 'o' as WCHAR,
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

// ComboWndProcA/W are #[no_mangle] extern "system" (the required export ABI).
// `WNDPROC` is spelled extern "stdcall" on i386 / extern "C" on amd64+arm64 by
// bindgen; the calling convention is identical on each arch, so reinterpret the
// fn pointer into the bindgen WNDPROC type.
type CbWndProcFn = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
const CB_WNDPROC_A: WNDPROC =
    unsafe { core::mem::transmute::<CbWndProcFn, WNDPROC>(ComboWndProcA) };
const CB_WNDPROC_W: WNDPROC =
    unsafe { core::mem::transmute::<CbWndProcFn, WNDPROC>(ComboWndProcW) };

#[no_mangle]
pub static COMBO_builtin_class: builtin_class_descr = builtin_class_descr {
    name: comboboxW.as_ptr(),
    style: CS_PARENTDC | CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW,
    procA: CB_WNDPROC_A,
    procW: CB_WNDPROC_W,
    extra: core::mem::size_of::<*mut HEADCOMBO>() as INT,
    cursor: IDC_ARROW,
    brush: 0 as HBRUSH,
};

unsafe fn COMBO_Init() -> BOOL {
    let hDC: HDC;

    if hComboBmp != 0 as HBITMAP {
        return 1;
    }
    hDC = CreateCompatibleDC(0 as HDC);
    if hDC != 0 as HDC {
        let mut bRet: BOOL = 0;
        hComboBmp = LoadBitmapW(0 as HINSTANCE, MAKEINTRESOURCEW(OBM_COMBO as WORD));
        if hComboBmp != 0 as HBITMAP {
            let mut bm: BITMAP = core::mem::zeroed();
            let hPrevB: HBITMAP;
            let mut r: RECT = core::mem::zeroed();

            GetObjectW(
                hComboBmp as *mut core::ffi::c_void,
                core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
            );
            CBitHeight = bm.bmHeight as UINT;
            CBitWidth = bm.bmWidth as UINT;

            TRACE!(b"combo bitmap [%i,%i]\n\0".as_ptr() as *const i8, CBitWidth, CBitHeight);

            hPrevB = SelectObject(hDC, hComboBmp as *mut core::ffi::c_void) as HBITMAP;
            SetRect(core::ptr::addr_of_mut!(r), 0, 0, CBitWidth as core::ffi::c_int, CBitHeight as core::ffi::c_int);
            InvertRect(hDC, core::ptr::addr_of_mut!(r));
            SelectObject(hDC, hPrevB as *mut core::ffi::c_void);
            bRet = 1;
        }
        DeleteDC(hDC);
        return bRet;
    }
    0
}

unsafe fn COMBO_update_uistate(lphc: LPHEADCOMBO) -> BOOL {
    let prev_flags: LONG = (*lphc).UIState;
    (*lphc).UIState = DefWindowProcW((*lphc).self_, WM_QUERYUISTATE, 0, 0) as LONG;
    (prev_flags != (*lphc).UIState) as BOOL
}

unsafe fn COMBO_NCCreate(hwnd: HWND, style: LONG) -> LRESULT {
    let lphc: LPHEADCOMBO;

    if COMBO_Init() != 0 && {
        lphc = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<HEADCOMBO>()) as SIZE_T) as LPHEADCOMBO;
        !lphc.is_null()
    } {
        (*lphc).self_ = hwnd;
        SetWindowLongPtrW(hwnd, 0, lphc as LONG_PTR);

        COMBO_update_uistate(lphc);

        /* some braindead apps do try to use scrollbar/border flags */

        (*lphc).dwStyle = style as u32 & !(WS_BORDER | WS_HSCROLL | WS_VSCROLL);
        SetWindowLongPtrW(hwnd, GWL_STYLE, (style as u32 & !(WS_BORDER | WS_HSCROLL | WS_VSCROLL)) as LONG_PTR);

        /*
         * We also have to remove the client edge style to make sure
         * we don't end-up with a non client area.
         */
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE,
            GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & !(WS_EX_CLIENTEDGE as LONG_PTR));

        if style as u32 & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE) == 0 {
            (*lphc).dwStyle |= CBS_HASSTRINGS as UINT;
        }
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_NOPARENTNOTIFY as LONG_PTR == 0 {
            (*lphc).wState |= CBF_NOTIFY;
        }

        return TRUE as LRESULT;
    }
    FALSE as LRESULT
}

unsafe fn COMBO_NCDestroy(lphc: LPHEADCOMBO) -> LRESULT {
    if !lphc.is_null() {
        if (CB_GETTYPE(lphc) != CBS_SIMPLE) && !(*lphc).hWndLBox.is_null() {
            DestroyWindow((*lphc).hWndLBox);
        }

        SetWindowLongPtrW((*lphc).self_, 0, 0);
        HeapFree(GetProcessHeap(), 0, lphc as *mut core::ffi::c_void);
    }
    0
}

unsafe fn CBGetTextAreaHeight(hwnd: HWND, lphc: LPHEADCOMBO) -> INT {
    let mut iTextItemHeight: INT;

    if (*lphc).editHeight != 0 {
        iTextItemHeight = (*lphc).editHeight;
    } else {
        let mut tm: TEXTMETRICW = core::mem::zeroed();
        let hDC: HDC = GetDC(hwnd);
        let mut hPrevFont: HFONT = core::ptr::null_mut();
        let baseUnitY: INT;

        if !(*lphc).hFont.is_null() {
            hPrevFont = SelectObject(hDC, (*lphc).hFont as *mut _) as HFONT;
        }

        GetTextMetricsW(hDC, &mut tm);

        baseUnitY = tm.tmHeight;

        if !hPrevFont.is_null() {
            SelectObject(hDC, hPrevFont as *mut _);
        }

        ReleaseDC(hwnd, hDC);

        iTextItemHeight = baseUnitY + 4;
    }

    if CB_OWNERDRAWN(lphc) != 0 && ((*lphc).wState & CBF_MEASUREITEM) != 0 {
        let mut measureItem: MEASUREITEMSTRUCT = core::mem::zeroed();
        let mut clientRect: RECT = core::mem::zeroed();
        let originalItemHeight: INT = iTextItemHeight;
        let id: UINT = GetWindowLongPtrW((*lphc).self_, GWLP_ID as i32) as UINT;

        GetClientRect(hwnd, &mut clientRect);

        (*lphc).wState &= !CBF_MEASUREITEM;

        measureItem.CtlType = ODT_COMBOBOX as UINT;
        measureItem.CtlID = id;
        measureItem.itemID = -1i32 as UINT;
        measureItem.itemWidth = clientRect.right as UINT;
        measureItem.itemHeight = (iTextItemHeight - 6) as UINT;
        measureItem.itemData = 0;
        SendMessageW((*lphc).owner, WM_MEASUREITEM, id as WPARAM, &mut measureItem as *mut _ as LPARAM);
        iTextItemHeight = 6 + measureItem.itemHeight as INT;

        if ((*lphc).dwStyle & CBS_OWNERDRAWFIXED) != 0 {
            measureItem.CtlType = ODT_COMBOBOX as UINT;
            measureItem.CtlID = id;
            measureItem.itemID = 0;
            measureItem.itemWidth = clientRect.right as UINT;
            measureItem.itemHeight = originalItemHeight as UINT;
            measureItem.itemData = 0;
            SendMessageW((*lphc).owner, WM_MEASUREITEM, id as WPARAM, &mut measureItem as *mut _ as LPARAM);
            (*lphc).fixedOwnerDrawHeight = measureItem.itemHeight as INT;
        }

        (*lphc).editHeight = iTextItemHeight;
    }

    iTextItemHeight
}

unsafe fn CBForceDummyResize(lphc: LPHEADCOMBO) {
    let mut windowRect: RECT = core::mem::zeroed();
    let newComboHeight: INT;

    newComboHeight = CBGetTextAreaHeight((*lphc).self_, lphc) + 2 * COMBO_YBORDERSIZE();

    GetWindowRect((*lphc).self_, &mut windowRect);

    SetWindowPos(
        (*lphc).self_,
        core::ptr::null_mut(),
        0,
        0,
        windowRect.right - windowRect.left,
        newComboHeight,
        (SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
    );
}

unsafe fn CBCalcPlacement(
    hwnd: HWND,
    lphc: LPHEADCOMBO,
    lprEdit: LPRECT,
    lprButton: LPRECT,
    lprLB: LPRECT,
) {
    GetClientRect(hwnd, lprEdit);

    InflateRect(lprEdit, -(COMBO_XBORDERSIZE() as INT), -(COMBO_YBORDERSIZE() as INT));

    (*lprEdit).bottom = (*lprEdit).top + CBGetTextAreaHeight(hwnd, lphc);

    CopyRect(lprButton, lprEdit);

    if CB_GETTYPE(lphc) == CBS_SIMPLE as u32 {
        (*lprButton).left = 0;
        (*lprButton).right = 0;
        (*lprButton).bottom = 0;
    } else {
        (*lprButton).left = (*lprButton).right - GetSystemMetrics(SM_CXVSCROLL as INT);
        (*lprEdit).right = (*lprButton).left;
    }

    if CB_GETTYPE(lphc) == CBS_DROPDOWN as u32 {
        (*lprEdit).right -= COMBO_EDITBUTTONSPACE() as INT;
    }

    if CB_GETTYPE(lphc) != CBS_DROPDOWNLIST as u32 {
        InflateRect(lprEdit, -(EDIT_CONTROL_PADDING() as INT), -(EDIT_CONTROL_PADDING() as INT));
    }

    if CB_GETTYPE(lphc) == CBS_SIMPLE as u32 {
        GetClientRect(hwnd, lprLB);

        (*lprLB).top = (*lprEdit).bottom + COMBO_YBORDERSIZE() as INT;
    } else {
        if (*lphc).droppedWidth < ((*lprButton).right + COMBO_XBORDERSIZE() as INT) {
            (*lprLB).right = (*lprLB).left + ((*lprButton).right + COMBO_XBORDERSIZE() as INT);

            if CB_GETTYPE(lphc) == CBS_DROPDOWN as u32 {
                (*lprLB).right -= COMBO_EDITBUTTONSPACE() as INT;
            }
        } else {
            (*lprLB).right = (*lprLB).left + (*lphc).droppedWidth;
        }
    }

    if (*lprEdit).right < (*lprEdit).left {
        (*lprEdit).right = (*lprEdit).left;
    }
}

unsafe fn CBGetDroppedControlRect(lphc: LPHEADCOMBO, lpRect: LPRECT) {
    GetWindowRect((*lphc).self_, lpRect);

    (*lpRect).right = (*lpRect).left + (*lphc).droppedRect.right - (*lphc).droppedRect.left;
    (*lpRect).bottom = (*lpRect).top + (*lphc).droppedRect.bottom - (*lphc).droppedRect.top;
}

unsafe fn COMBO_Create(
    hwnd: HWND,
    lphc: LPHEADCOMBO,
    hwndParent: HWND,
    style: LONG,
    unicode: BOOL,
) -> LRESULT {
    static clbName: [WCHAR; 10] = [
        'C' as WCHAR, 'o' as WCHAR, 'm' as WCHAR, 'b' as WCHAR, 'o' as WCHAR,
        'L' as WCHAR, 'B' as WCHAR, 'o' as WCHAR, 'x' as WCHAR,
     0,];
    static editName: [WCHAR; 5] = [
        'E' as WCHAR, 'd' as WCHAR, 'i' as WCHAR, 't' as WCHAR, 0,
    ];

    if CB_GETTYPE(lphc) == 0 {
        (*lphc).dwStyle |= CBS_SIMPLE as u32;
    }
    if CB_GETTYPE(lphc) != CBS_DROPDOWNLIST {
        (*lphc).wState |= CBF_EDIT;
    }

    (*lphc).owner = hwndParent;

    (*lphc).droppedWidth = 0;
    (*lphc).editHeight = 0;

    (*lphc).wState |= CBF_MEASUREITEM;

    if true {
        let mut lbeStyle: UINT = 0;
        let mut lbeExStyle: UINT = 0;

        GetClientRect(hwnd, core::ptr::addr_of_mut!((*lphc).droppedRect));
        CBCalcPlacement(
            hwnd,
            lphc,
            core::ptr::addr_of_mut!((*lphc).textRect),
            core::ptr::addr_of_mut!((*lphc).buttonRect),
            core::ptr::addr_of_mut!((*lphc).droppedRect),
        );

        if CB_GETTYPE(lphc) != CBS_SIMPLE {
            (*lphc).droppedRect.top = (*lphc).textRect.bottom + COMBO_YBORDERSIZE();

            if CB_GETTYPE(lphc) == CBS_DROPDOWN {
                (*lphc).droppedRect.left += COMBO_EDITBUTTONSPACE();
            }

            if (*lphc).droppedRect.bottom < (*lphc).droppedRect.top {
                (*lphc).droppedRect.bottom = (*lphc).droppedRect.top;
            }
            if (*lphc).droppedRect.right < (*lphc).droppedRect.left {
                (*lphc).droppedRect.right = (*lphc).droppedRect.left;
            }
            MapWindowPoints(
                hwnd,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*lphc).droppedRect) as LPPOINT,
                2,
            );
        }

        lbeStyle = (LBS_NOTIFY | LBS_COMBOBOX | WS_BORDER | WS_CLIPSIBLINGS | WS_CHILD)
            | (style as UINT & (WS_VSCROLL | CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE));

        if (*lphc).dwStyle & CBS_SORT as u32 != 0 {
            lbeStyle |= LBS_SORT;
        }
        if (*lphc).dwStyle & CBS_HASSTRINGS as u32 != 0 {
            lbeStyle |= LBS_HASSTRINGS;
        }
        if (*lphc).dwStyle & CBS_NOINTEGRALHEIGHT as u32 != 0 {
            lbeStyle |= LBS_NOINTEGRALHEIGHT;
        }
        if (*lphc).dwStyle & CBS_DISABLENOSCROLL as u32 != 0 {
            lbeStyle |= LBS_DISABLENOSCROLL;
        }

        if CB_GETTYPE(lphc) == CBS_SIMPLE {
            lbeStyle |= WS_VISIBLE;
            lbeStyle &= !WS_BORDER;
            lbeExStyle |= WS_EX_CLIENTEDGE;
        } else {
            lbeExStyle |= WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        }

        if unicode != 0 {
            (*lphc).hWndLBox = CreateWindowExW(
                lbeExStyle,
                clbName.as_ptr(),
                core::ptr::null_mut(),
                lbeStyle,
                (*lphc).droppedRect.left,
                (*lphc).droppedRect.top,
                (*lphc).droppedRect.right - (*lphc).droppedRect.left,
                (*lphc).droppedRect.bottom - (*lphc).droppedRect.top,
                hwnd,
                ID_CB_LISTBOX as HMENU,
                GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
                lphc as *mut core::ffi::c_void,
            );
        } else {
            (*lphc).hWndLBox = CreateWindowExA(
                lbeExStyle,
                b"ComboLBox\0".as_ptr() as LPCSTR,
                core::ptr::null_mut(),
                lbeStyle,
                (*lphc).droppedRect.left,
                (*lphc).droppedRect.top,
                (*lphc).droppedRect.right - (*lphc).droppedRect.left,
                (*lphc).droppedRect.bottom - (*lphc).droppedRect.top,
                hwnd,
                ID_CB_LISTBOX as HMENU,
                GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
                lphc as *mut core::ffi::c_void,
            );
        }

        if !(*lphc).hWndLBox.is_null() {
            let mut bEdit: BOOL = 1;
            lbeStyle = WS_CHILD | WS_VISIBLE | ES_NOHIDESEL | ES_LEFT | ES_COMBO;

            if (*lphc).wState & CBF_EDIT != 0 {
                if (*lphc).dwStyle & CBS_OEMCONVERT as u32 != 0 {
                    lbeStyle |= ES_OEMCONVERT;
                }
                if (*lphc).dwStyle & CBS_AUTOHSCROLL as u32 != 0 {
                    lbeStyle |= ES_AUTOHSCROLL;
                }
                if (*lphc).dwStyle & CBS_LOWERCASE as u32 != 0 {
                    lbeStyle |= ES_LOWERCASE;
                } else if (*lphc).dwStyle & CBS_UPPERCASE as u32 != 0 {
                    lbeStyle |= ES_UPPERCASE;
                }

                if IsWindowEnabled(hwnd) == 0 {
                    lbeStyle |= WS_DISABLED;
                }

                if unicode != 0 {
                    (*lphc).hWndEdit = CreateWindowExW(
                        0,
                        editName.as_ptr(),
                        core::ptr::null_mut(),
                        lbeStyle,
                        (*lphc).textRect.left,
                        (*lphc).textRect.top,
                        (*lphc).textRect.right - (*lphc).textRect.left,
                        (*lphc).textRect.bottom - (*lphc).textRect.top,
                        hwnd,
                        ID_CB_EDIT as HMENU,
                        GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
                        core::ptr::null_mut(),
                    );
                } else {
                    (*lphc).hWndEdit = CreateWindowExA(
                        0,
                        b"Edit\0".as_ptr() as LPCSTR,
                        core::ptr::null_mut(),
                        lbeStyle,
                        (*lphc).textRect.left,
                        (*lphc).textRect.top,
                        (*lphc).textRect.right - (*lphc).textRect.left,
                        (*lphc).textRect.bottom - (*lphc).textRect.top,
                        hwnd,
                        ID_CB_EDIT as HMENU,
                        GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
                        core::ptr::null_mut(),
                    );
                }

                if (*lphc).hWndEdit.is_null() {
                    bEdit = 0;
                }
            }

            if bEdit != 0 {
                if CB_GETTYPE(lphc) != CBS_SIMPLE {
                    SetParent((*lphc).hWndLBox, HWND_DESKTOP);
                    CBForceDummyResize(lphc);
                }

                TRACE!(b"init done\n\0".as_ptr());
                return 0;
            }
            ERR!("edit control failure.\n");
        } else {
            ERR!("listbox failure.\n");
        }
    } else {
        ERR!("no owner for visible combo.\n");
    }

    -1
}

unsafe fn CBPaintButton(lphc: LPHEADCOMBO, hdc: HDC, mut rectButton: RECT) {
    let mut buttonState: UINT = DFCS_SCROLLCOMBOBOX;

    if (*lphc).wState & CBF_NOREDRAW != 0 {
        return;
    }

    if (*lphc).wState & CBF_BUTTONDOWN != 0 {
        buttonState |= DFCS_PUSHED;
    }

    if CB_DISABLED(lphc) != 0 {
        buttonState |= DFCS_INACTIVE;
    }

    DrawFrameControl(hdc, core::ptr::addr_of_mut!(rectButton), DFC_SCROLL, buttonState);
}

unsafe fn COMBO_PrepareColors(lphc: LPHEADCOMBO, hDC: HDC) -> HBRUSH {
    let mut hBkgBrush: HBRUSH;

    if CB_DISABLED(lphc) != 0 {
        hBkgBrush = GetControlColor((*lphc).owner, (*lphc).self_, hDC, WM_CTLCOLORSTATIC);
        SetTextColor(hDC, GetSysColor(COLOR_GRAYTEXT as i32));
    } else {
        hBkgBrush = GetControlColor((*lphc).owner, (*lphc).self_, hDC, WM_CTLCOLOREDIT);
    }

    if hBkgBrush.is_null() {
        hBkgBrush = GetSysColorBrush(COLOR_WINDOW as i32);
    }

    hBkgBrush
}

unsafe fn CBPaintText(lphc: LPHEADCOMBO, hdc_paint: HDC) {
    let mut rectEdit: RECT = (*lphc).textRect;
    let mut id: INT;
    let mut size: INT = 0;
    let mut pText: LPWSTR = core::ptr::null_mut();

    TRACE!(b"\n\0".as_ptr() as *const i8);

    id = SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as INT;
    if id != LB_ERR {
        size = SendMessageW((*lphc).hWndLBox, LB_GETTEXTLEN, id as WPARAM, 0) as INT;
        if size == LB_ERR {
            FIXME!("LB_ERR probably not handled yet\n");
        }
        pText = HeapAlloc(
            GetProcessHeap(),
            0,
            ((size + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if !pText.is_null() {
            size = SendMessageW(
                (*lphc).hWndLBox,
                LB_GETTEXT,
                id as WPARAM,
                pText as LPARAM,
            ) as INT;
            *pText.offset(size as isize) = 0;
        } else {
            return;
        }
    }

    if ((*lphc).wState & CBF_EDIT as u32) != 0 {
        static empty_stringW: [u16; 1] = [0];
        if CB_HASSTRINGS(lphc) != 0 {
            SetWindowTextW(
                (*lphc).hWndEdit,
                if !pText.is_null() {
                    pText
                } else {
                    empty_stringW.as_ptr() as LPWSTR
                },
            );
        }
        if ((*lphc).wState & CBF_FOCUSED as u32) != 0 {
            SendMessageW((*lphc).hWndEdit, EM_SETSEL, 0, MAXLONG as LPARAM);
        }
    } else if ((*lphc).wState & CBF_NOREDRAW as u32) == 0 && IsWindowVisible((*lphc).self_) != 0 {
        let hdc: HDC = if !hdc_paint.is_null() {
            hdc_paint
        } else {
            GetDC((*lphc).self_)
        };
        let mut itemState: UINT = ODS_COMBOBOXEDIT;
        let hPrevFont: HFONT = if !(*lphc).hFont.is_null() {
            SelectObject(hdc, (*lphc).hFont as *mut core::ffi::c_void) as HFONT
        } else {
            core::ptr::null_mut()
        };
        let hPrevBrush: HBRUSH;
        let hBkgBrush: HBRUSH;

        InflateRect(&mut rectEdit, -1, -1);

        hBkgBrush = COMBO_PrepareColors(lphc, hdc);
        hPrevBrush = SelectObject(hdc, hBkgBrush as *mut core::ffi::c_void) as HBRUSH;
        FillRect(hdc, &rectEdit, hBkgBrush);

        if CB_OWNERDRAWN(lphc) != 0 {
            let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
            let clipRegion: HRGN;
            let ctlid: UINT = GetWindowLongPtrW((*lphc).self_, GWLP_ID) as UINT;

            if ((*lphc).wState & CBF_FOCUSED as u32) != 0
                && ((*lphc).wState & CBF_DROPPED as u32) == 0
            {
                itemState |= ODS_SELECTED | ODS_FOCUS;
            }

            if IsWindowEnabled((*lphc).self_) == 0 {
                itemState |= ODS_DISABLED;
            }

            dis.CtlType = ODT_COMBOBOX;
            dis.CtlID = ctlid;
            dis.hwndItem = (*lphc).self_;
            dis.itemAction = ODA_DRAWENTIRE;
            dis.itemID = id as UINT;
            dis.itemState = itemState;
            dis.hDC = hdc;
            // DRAWITEMSTRUCT.rcItem is RECTL; rectEdit is the layout-identical tagRECT.
            dis.rcItem = *(&rectEdit as *const RECT as *const RECTL);
            dis.itemData =
                SendMessageW((*lphc).hWndLBox, LB_GETITEMDATA, id as WPARAM, 0) as ULONG_PTR;

            clipRegion = set_control_clipping(hdc, &rectEdit);

            SendMessageW(
                (*lphc).owner,
                WM_DRAWITEM,
                ctlid as WPARAM,
                core::ptr::addr_of_mut!(dis) as LPARAM,
            );

            SelectClipRgn(hdc, clipRegion);
            if !clipRegion.is_null() {
                DeleteObject(clipRegion as *mut core::ffi::c_void);
            }
        } else {
            static empty_stringW: [u16; 1] = [0];

            if ((*lphc).wState & CBF_FOCUSED as u32) != 0
                && ((*lphc).wState & CBF_DROPPED as u32) == 0
            {
                FillRect(hdc, &rectEdit, GetSysColorBrush(COLOR_HIGHLIGHT as i32));
                SetBkColor(hdc, GetSysColor(COLOR_HIGHLIGHT as i32));
                SetTextColor(hdc, GetSysColor(COLOR_HIGHLIGHTTEXT as i32));
            }

            ExtTextOutW(
                hdc,
                rectEdit.left + 1,
                rectEdit.top + 1,
                ETO_OPAQUE | ETO_CLIPPED,
                &rectEdit,
                if !pText.is_null() {
                    pText
                } else {
                    empty_stringW.as_ptr() as LPWSTR
                },
                size as UINT,
                core::ptr::null_mut(),
            );

            if ((*lphc).wState & CBF_FOCUSED as u32) != 0
                && ((*lphc).wState & CBF_DROPPED as u32) == 0
                && ((*lphc).UIState & UISF_HIDEFOCUS as i32) == 0
            {
                DrawFocusRect(hdc, &rectEdit);
            }
        }

        if !hPrevFont.is_null() {
            SelectObject(hdc, hPrevFont as *mut core::ffi::c_void);
        }

        if !hPrevBrush.is_null() {
            SelectObject(hdc, hPrevBrush as *mut core::ffi::c_void);
        }

        if hdc_paint.is_null() {
            ReleaseDC((*lphc).self_, hdc);
        }
    }

    if !pText.is_null() {
        HeapFree(GetProcessHeap(), 0, pText as *mut core::ffi::c_void);
    }
}

unsafe fn CBPaintBorder(hwnd: HWND, lphc: *const HEADCOMBO, hdc: HDC) {
    let mut clientRect: RECT = core::mem::zeroed();

    if CB_GETTYPE(lphc) != CBS_SIMPLE {
        GetClientRect(hwnd, &mut clientRect);
    } else {
        CopyRect(&mut clientRect, &(*lphc).textRect);

        InflateRect(&mut clientRect, EDIT_CONTROL_PADDING(), EDIT_CONTROL_PADDING());
        InflateRect(&mut clientRect, COMBO_XBORDERSIZE(), COMBO_YBORDERSIZE());
    }

    DrawEdge(hdc, &mut clientRect, EDGE_SUNKEN, BF_RECT);
}

unsafe fn COMBO_Paint(lphc: LPHEADCOMBO, hParamDC: HDC) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let hDC: HDC;

    hDC = if !hParamDC.is_null() {
        hParamDC
    } else {
        BeginPaint((*lphc).self_, &mut ps)
    };

    TRACE!(b"hdc=%p\n\0".as_ptr() as *const i8, hDC);

    if !hDC.is_null() && ((*lphc).wState & CBF_NOREDRAW) == 0 {
        let hPrevBrush: HBRUSH;
        let hBkgBrush: HBRUSH;

        hBkgBrush = COMBO_PrepareColors(lphc, hDC);
        hPrevBrush = SelectObject(hDC, hBkgBrush as _) as HBRUSH;
        if ((*lphc).wState & CBF_EDIT) == 0 {
            FillRect(hDC, core::ptr::addr_of_mut!((*lphc).textRect), hBkgBrush);
        }

        CBPaintBorder((*lphc).self_, lphc, hDC);

        if IsRectEmpty(core::ptr::addr_of_mut!((*lphc).buttonRect)) == 0 {
            CBPaintButton(lphc, hDC, (*lphc).buttonRect);
        }

        if CB_GETTYPE(lphc) != CBS_DROPDOWNLIST {
            let mut rPadEdit: RECT = (*lphc).textRect;

            InflateRect(
                &mut rPadEdit,
                EDIT_CONTROL_PADDING(),
                EDIT_CONTROL_PADDING(),
            );

            FrameRect(hDC, &mut rPadEdit, GetSysColorBrush(COLOR_WINDOW as i32));
        }

        if ((*lphc).wState & CBF_EDIT) == 0 {
            CBPaintText(lphc, hDC);
        }

        if !hPrevBrush.is_null() {
            SelectObject(hDC, hPrevBrush as _);
        }
    }

    if hParamDC.is_null() {
        EndPaint((*lphc).self_, &mut ps);
    }

    0
}

unsafe fn CBUpdateLBox(lphc: LPHEADCOMBO, bSelect: BOOL) -> INT {
    let mut length: INT;
    let mut idx: INT;
    let mut pText: LPWSTR = core::ptr::null_mut();

    idx = LB_ERR;
    length = SendMessageW((*lphc).hWndEdit, WM_GETTEXTLENGTH, 0, 0) as INT;

    if length > 0 {
        pText = HeapAlloc(
            GetProcessHeap(),
            0,
            ((length + 1) as usize * core::mem::size_of::<WCHAR>()) as _,
        ) as LPWSTR;
    }

    TRACE!(b"\t edit text length %i\n\0".as_ptr() as *const i8, length);

    if !pText.is_null() {
        GetWindowTextW((*lphc).hWndEdit, pText, length + 1);
        idx = SendMessageW(
            (*lphc).hWndLBox,
            LB_FINDSTRING,
            (-1i32) as WPARAM,
            pText as LPARAM,
        ) as INT;
        HeapFree(GetProcessHeap(), 0, pText as _);
    }

    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETCURSEL,
        (if bSelect != 0 { idx } else { -1 }) as WPARAM,
        0,
    );

    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETCARETINDEX,
        (if idx < 0 { 0 } else { idx }) as WPARAM,
        0,
    );
    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETTOPINDEX,
        (if idx < 0 { 0 } else { idx }) as WPARAM,
        0,
    );

    idx
}

unsafe fn CBUpdateEdit(lphc: LPHEADCOMBO, index: INT) {
    let mut length: INT;
    let mut pText: LPWSTR = core::ptr::null_mut();
    static empty_stringW: [WCHAR; 1] = [0];

    if index >= 0 {
        length = SendMessageW((*lphc).hWndLBox, LB_GETTEXTLEN, index as WPARAM, 0) as INT;
        if length != LB_ERR {
            pText = HeapAlloc(GetProcessHeap(), 0, ((length + 1) as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
            if !pText.is_null() {
                SendMessageW((*lphc).hWndLBox, LB_GETTEXT, index as WPARAM, pText as LPARAM);
            }
        }
    }

    if CB_HASSTRINGS(lphc) != 0 {
        (*lphc).wState |= (CBF_NOEDITNOTIFY | CBF_NOLBSELECT) as UINT;
        SendMessageW(
            (*lphc).hWndEdit,
            WM_SETTEXT,
            0,
            if !pText.is_null() { pText as LPARAM } else { empty_stringW.as_ptr() as LPARAM },
        );
        (*lphc).wState &= !((CBF_NOEDITNOTIFY | CBF_NOLBSELECT) as u32);
    }

    if (*lphc).wState & (CBF_FOCUSED as u32) != 0 {
        SendMessageW((*lphc).hWndEdit, EM_SETSEL, 0, -1i32 as LPARAM);
    }

    HeapFree(GetProcessHeap(), 0, pText as _);
}

unsafe fn CBDropDown(lphc: LPHEADCOMBO) {
    let monitor: HMONITOR;
    let mut mon_info: MONITORINFO = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let mut nItems: i32 = 0;
    let mut nDroppedHeight: i32;

    TRACE!(b"[%p]: drop down\n\0".as_ptr() as *const i8, (*lphc).self_);

    CB_NOTIFY(lphc, CBN_DROPDOWN as i32);

    /* set selection */

    (*lphc).wState |= CBF_DROPPED as UINT;
    if CB_GETTYPE(lphc) == CBS_SIMPLE {
        // placeholder, never reached - CB_GETTYPE compares against CBS_DROPDOWN below
    }
    if CB_GETTYPE(lphc) == CBS_DROPDOWN {
        (*lphc).droppedIndex = CBUpdateLBox(lphc, 1);

        /* Update edit only if item is in the list */
        if ((*lphc).wState & CBF_CAPTURE as UINT) == 0 && (*lphc).droppedIndex >= 0 {
            CBUpdateEdit(lphc, (*lphc).droppedIndex);
        }
    } else {
        (*lphc).droppedIndex =
            SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as INT;

        SendMessageW(
            (*lphc).hWndLBox,
            LB_SETTOPINDEX,
            (if (*lphc).droppedIndex == LB_ERR { 0 } else { (*lphc).droppedIndex }) as WPARAM,
            0,
        );
        SendMessageW((*lphc).hWndLBox, LB_CARETON, 0, 0);
    }

    /* now set popup position */
    GetWindowRect((*lphc).self_, &mut rect);

    if CB_GETTYPE(lphc) == CBS_DROPDOWN {
        rect.left += COMBO_EDITBUTTONSPACE();
    }

    nDroppedHeight = (*lphc).droppedRect.bottom - (*lphc).droppedRect.top;
    GetWindowRect((*lphc).hWndLBox, &mut r);
    if nDroppedHeight < r.bottom - r.top {
        nDroppedHeight = r.bottom - r.top;
    }
    nItems = SendMessageW((*lphc).hWndLBox, LB_GETCOUNT, 0, 0) as i32;

    if nItems > 0 {
        let nHeight: i32;
        let nIHeight: i32;

        nIHeight = SendMessageW((*lphc).hWndLBox, LB_GETITEMHEIGHT, 0, 0) as i32;

        nHeight = nIHeight * nItems;

        if nHeight < nDroppedHeight - COMBO_YBORDERSIZE() {
            nDroppedHeight = nHeight + COMBO_YBORDERSIZE();
        }
    }

    r.left = rect.left;
    r.top = rect.bottom;
    r.right = r.left + (*lphc).droppedRect.right - (*lphc).droppedRect.left;
    r.bottom = r.top + nDroppedHeight;

    monitor = MonitorFromRect(&mut rect, MONITOR_DEFAULTTOPRIMARY);
    mon_info.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
    GetMonitorInfoW(monitor, &mut mon_info);

    if r.bottom > mon_info.rcWork.bottom {
        r.top = max(rect.top - nDroppedHeight, mon_info.rcWork.top);
        r.bottom = min(r.top + nDroppedHeight, mon_info.rcWork.bottom);
    }

    SetWindowPos(
        (*lphc).hWndLBox,
        HWND_TOPMOST,
        r.left,
        r.top,
        r.right - r.left,
        r.bottom - r.top,
        (SWP_NOACTIVATE | SWP_SHOWWINDOW) as UINT,
    );

    if ((*lphc).wState & CBF_NOREDRAW as UINT) == 0 {
        RedrawWindow(
            (*lphc).self_,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW) as UINT,
        );
    }

    EnableWindow((*lphc).hWndLBox, 1);
    if GetCapture() != (*lphc).self_ {
        SetCapture((*lphc).hWndLBox);
    }
}

unsafe fn CBRollUp(lphc: LPHEADCOMBO, ok: BOOL, mut bButton: BOOL) {
    let hWnd: HWND = (*lphc).self_;

    CB_NOTIFY(lphc, if ok != 0 { CBN_SELENDOK as INT } else { CBN_SELENDCANCEL as INT });

    if IsWindow(hWnd) != 0 && CB_GETTYPE(lphc) != CBS_SIMPLE {
        if (*lphc).wState & CBF_DROPPED != 0 {
            let mut rect: RECT = core::mem::zeroed();

            (*lphc).wState &= !CBF_DROPPED;
            ShowWindow((*lphc).hWndLBox, SW_HIDE as i32);

            if GetCapture() == (*lphc).hWndLBox {
                ReleaseCapture();
            }

            if CB_GETTYPE(lphc) == CBS_DROPDOWN {
                rect = (*lphc).buttonRect;
            } else {
                if bButton != 0 {
                    UnionRect(
                        &mut rect,
                        &(*lphc).buttonRect,
                        &(*lphc).textRect,
                    );
                } else {
                    rect = (*lphc).textRect;
                }

                bButton = TRUE as BOOL;
            }

            if bButton != 0 && (*lphc).wState & CBF_NOREDRAW == 0 {
                RedrawWindow(
                    hWnd,
                    &rect,
                    core::ptr::null_mut(),
                    RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN,
                );
            }
            CB_NOTIFY(lphc, CBN_CLOSEUP as INT);
        }
    }
}

// Cross-TU: referenced by controls/listbox.c (extern in include/controls.h) and
// listbox.rs — must be a `#[no_mangle] extern "system"` C symbol (i386 → _Name@N).
#[no_mangle]
pub unsafe extern "system" fn COMBO_FlipListbox(lphc: LPHEADCOMBO, ok: BOOL, bRedrawButton: BOOL) -> BOOL {
    if (*lphc).wState & CBF_DROPPED != 0 {
        CBRollUp(lphc, ok, bRedrawButton);
        return 0;
    }

    CBDropDown(lphc);
    1
}

unsafe fn CBRepaintButton(lphc: LPHEADCOMBO) {
    InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).buttonRect), 1u8 as BOOL);
    UpdateWindow((*lphc).self_);
}

unsafe fn COMBO_SetFocus(lphc: LPHEADCOMBO) {
    if (*lphc).wState & CBF_FOCUSED == 0 {
        if CB_GETTYPE(lphc) == CBS_DROPDOWNLIST {
            SendMessageW((*lphc).hWndLBox, LB_CARETON, 0, 0);
        }

        if (*lphc).wState & CBF_EDIT == 0 {
            InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).textRect), TRUE);
        }

        CB_NOTIFY(lphc, CBN_SETFOCUS as INT);
        (*lphc).wState |= CBF_FOCUSED;
    }
}

unsafe fn COMBO_KillFocus(lphc: LPHEADCOMBO) {
    let hWnd: HWND = (*lphc).self_;

    if (*lphc).wState & CBF_FOCUSED != 0 {
        CBRollUp(lphc, FALSE as BOOL, TRUE as BOOL);
        if IsWindow(hWnd) != 0 {
            if CB_GETTYPE(lphc) == CBS_DROPDOWNLIST {
                SendMessageW((*lphc).hWndLBox, LB_CARETOFF, 0, 0);
            }

            (*lphc).wState &= !CBF_FOCUSED;

            if (*lphc).wState & CBF_EDIT == 0 {
                InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).textRect), TRUE as BOOL);
            }

            CB_NOTIFY(lphc, CBN_KILLFOCUS as i32);
        }
    }
}

unsafe fn COMBO_Command(lphc: LPHEADCOMBO, wParam: WPARAM, hWnd: HWND) -> LRESULT {
    if (*lphc).wState & CBF_EDIT as u32 != 0 && (*lphc).hWndEdit == hWnd {
        match (crate::internal::HIWORD(wParam as DWORD) >> 8) as u32 {
            x if x == (EN_SETFOCUS >> 8) as u32 => {
                COMBO_SetFocus(lphc);
            }
            x if x == (EN_KILLFOCUS >> 8) as u32 => {
                COMBO_KillFocus(lphc);
            }
            x if x == (EN_CHANGE >> 8) as u32 => {
                if (*lphc).wState & CBF_NOLBSELECT as u32 != 0 {
                    (*lphc).wState &= !(CBF_NOLBSELECT as u32);
                } else {
                    CBUpdateLBox(lphc, ((*lphc).wState & CBF_DROPPED as u32) as BOOL);
                }

                if (*lphc).wState & CBF_NOEDITNOTIFY as u32 == 0 {
                    SendMessageW(
                        (*lphc).owner,
                        WM_COMMAND,
                        crate::internal::MAKEWPARAM(
                            GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                            CBN_EDITCHANGE as WORD,
                        ) as WPARAM,
                        (*lphc).self_ as LPARAM,
                    );
                }
            }
            x if x == (EN_UPDATE >> 8) as u32 => {
                if (*lphc).wState & CBF_NOEDITNOTIFY as u32 == 0 {
                    SendMessageW(
                        (*lphc).owner,
                        WM_COMMAND,
                        crate::internal::MAKEWPARAM(
                            GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                            CBN_EDITUPDATE as WORD,
                        ) as WPARAM,
                        (*lphc).self_ as LPARAM,
                    );
                }
            }
            x if x == (EN_ERRSPACE >> 8) as u32 => {
                SendMessageW(
                    (*lphc).owner,
                    WM_COMMAND,
                    crate::internal::MAKEWPARAM(
                        GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                        CBN_ERRSPACE as WORD,
                    ) as WPARAM,
                    (*lphc).self_ as LPARAM,
                );
            }
            _ => {}
        }
    } else if (*lphc).hWndLBox == hWnd {
        match crate::internal::HIWORD(wParam as DWORD) as i16 as i32 {
            x if x == LBN_ERRSPACE as i32 => {
                SendMessageW(
                    (*lphc).owner,
                    WM_COMMAND,
                    crate::internal::MAKEWPARAM(
                        GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                        CBN_ERRSPACE as WORD,
                    ) as WPARAM,
                    (*lphc).self_ as LPARAM,
                );
            }
            x if x == LBN_DBLCLK as i32 => {
                SendMessageW(
                    (*lphc).owner,
                    WM_COMMAND,
                    crate::internal::MAKEWPARAM(
                        GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                        CBN_DBLCLK as WORD,
                    ) as WPARAM,
                    (*lphc).self_ as LPARAM,
                );
            }
            x if x == LBN_SELCHANGE as i32 || x == LBN_SELCANCEL as i32 => {
                if (*lphc).wState & CBF_NOROLLUP as u32 == 0 {
                    CBRollUp(
                        lphc,
                        (crate::internal::HIWORD(wParam as DWORD) as i32 == LBN_SELCHANGE as i32) as BOOL,
                        1,
                    );
                } else {
                    (*lphc).wState &= !(CBF_NOROLLUP as u32);
                }

                SendMessageW(
                    (*lphc).owner,
                    WM_COMMAND,
                    crate::internal::MAKEWPARAM(
                        GetWindowLongPtrW((*lphc).self_, GWLP_ID) as WORD,
                        CBN_SELCHANGE as WORD,
                    ) as WPARAM,
                    (*lphc).self_ as LPARAM,
                );

                if crate::internal::HIWORD(wParam as DWORD) as i32 == LBN_SELCHANGE as i32 {
                    if (*lphc).wState & CBF_EDIT as u32 != 0 {
                        let index = SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as INT;
                        (*lphc).wState |= CBF_NOLBSELECT as u32;
                        CBUpdateEdit(lphc, index);
                        SendMessageW((*lphc).hWndEdit, EM_SETSEL, 0, -1 as LPARAM);
                    } else {
                        InvalidateRect((*lphc).self_, &mut (*lphc).textRect, 1);
                        UpdateWindow((*lphc).self_);
                    }
                }
            }
            x if x == LBN_SETFOCUS as i32 || x == LBN_KILLFOCUS as i32 => {}
            _ => {}
        }
    }
    0
}

unsafe fn COMBO_ItemOp(lphc: LPHEADCOMBO, msg: UINT, lParam: LPARAM) -> LRESULT {
    let hWnd: HWND = (*lphc).self_;
    let id: UINT = GetWindowLongPtrW(hWnd, GWLP_ID as i32) as UINT;

    TRACE!(b"[%p]: ownerdraw op %04x\n\0".as_ptr(), (*lphc).self_, msg);

    match msg {
        x if x == WM_DELETEITEM => {
            let lpIS = lParam as *mut DELETEITEMSTRUCT;
            (*lpIS).CtlType = ODT_COMBOBOX;
            (*lpIS).CtlID = id;
            (*lpIS).hwndItem = hWnd;
        }
        x if x == WM_DRAWITEM => {
            let lpIS = lParam as *mut DRAWITEMSTRUCT;
            (*lpIS).CtlType = ODT_COMBOBOX;
            (*lpIS).CtlID = id;
            (*lpIS).hwndItem = hWnd;
        }
        x if x == WM_COMPAREITEM => {
            let lpIS = lParam as *mut COMPAREITEMSTRUCT;
            (*lpIS).CtlType = ODT_COMBOBOX;
            (*lpIS).CtlID = id;
            (*lpIS).hwndItem = hWnd;
        }
        x if x == WM_MEASUREITEM => {
            let lpIS = lParam as *mut MEASUREITEMSTRUCT;
            (*lpIS).CtlType = ODT_COMBOBOX;
            (*lpIS).CtlID = id;
        }
        _ => {}
    }

    SendMessageW((*lphc).owner, msg, id as WPARAM, lParam)
}

unsafe fn COMBO_GetTextW(lphc: LPHEADCOMBO, count: INT, buf: LPWSTR) -> LRESULT {
    let mut length: INT;

    if (*lphc).wState & CBF_EDIT as u32 != 0 {
        return SendMessageW((*lphc).hWndEdit, WM_GETTEXT, count as WPARAM, buf as LPARAM);
    }

    /* get it from the listbox */

    if count == 0 || buf.is_null() {
        return 0;
    }

    'error: {
        if !(*lphc).hWndLBox.is_null() {
            let idx = SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as INT;
            if idx == LB_ERR {
                break 'error;
            }
            length = SendMessageW((*lphc).hWndLBox, LB_GETTEXTLEN, idx as WPARAM, 0) as INT;
            if length == LB_ERR {
                break 'error;
            }

            /* 'length' is without the terminating character */
            if length >= count {
                let lpBuffer = HeapAlloc(GetProcessHeap(), 0, (((length + 1) as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                if lpBuffer.is_null() {
                    break 'error;
                }
                length = SendMessageW((*lphc).hWndLBox, LB_GETTEXT, idx as WPARAM, lpBuffer as LPARAM) as INT;

                /* truncate if buffer is too short */
                if length != LB_ERR {
                    lstrcpynW(buf, lpBuffer, count);
                    length = count;
                }
                HeapFree(GetProcessHeap(), 0, lpBuffer as *mut core::ffi::c_void);
            } else {
                length = SendMessageW((*lphc).hWndLBox, LB_GETTEXT, idx as WPARAM, buf as LPARAM) as INT;
            }

            if length == LB_ERR {
                return 0;
            }
            return length as LRESULT;
        }
    }

    /* error - truncate string, return zero */
    *buf = 0;
    0
}

unsafe fn COMBO_GetTextA( lphc: LPHEADCOMBO, count: INT, buf: LPSTR ) -> LRESULT
{
    let mut length: INT;

    if (*lphc).wState & CBF_EDIT as u32 != 0 {
        return SendMessageA( (*lphc).hWndEdit, WM_GETTEXT, count as WPARAM, buf as LPARAM );
    }

    /* get it from the listbox */

    if count == 0 || buf.is_null() { return 0; }

    'error: {
        if !(*lphc).hWndLBox.is_null()
        {
            let idx = SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as INT;
            if idx == LB_ERR { break 'error; }
            length = SendMessageA((*lphc).hWndLBox, LB_GETTEXTLEN, idx as WPARAM, 0 ) as INT;
            if length == LB_ERR { break 'error; }

            /* 'length' is without the terminating character */
            if length >= count
            {
                let lpBuffer = HeapAlloc(GetProcessHeap(), 0, (length + 1) as SIZE_T) as LPSTR;
                if lpBuffer.is_null() { break 'error; }
                length = SendMessageA((*lphc).hWndLBox, LB_GETTEXT, idx as WPARAM, lpBuffer as LPARAM) as INT;

                /* truncate if buffer is too short */
                if length != LB_ERR
                {
                    lstrcpynA( buf, lpBuffer, count );
                    length = count;
                }
                HeapFree( GetProcessHeap(), 0, lpBuffer as *mut _ );
            }
            else { length = SendMessageA((*lphc).hWndLBox, LB_GETTEXT, idx as WPARAM, buf as LPARAM) as INT; }

            if length == LB_ERR { return 0; }
            return length as LRESULT;
        }
    }

    /* error - truncate string, return zero */
    *buf = 0;
    0
}

unsafe fn CBResetPos(
    lphc: LPHEADCOMBO,
    rectEdit: *const RECT,
    rectLB: *const RECT,
    bRedraw: BOOL,
) {
    let bDrop: BOOL = (CB_GETTYPE(lphc) != CBS_SIMPLE as u32) as BOOL;

    if (*lphc).wState & CBF_EDIT != 0 {
        SetWindowPos(
            (*lphc).hWndEdit,
            0 as HWND,
            (*rectEdit).left,
            (*rectEdit).top,
            (*rectEdit).right - (*rectEdit).left,
            (*rectEdit).bottom - (*rectEdit).top,
            SWP_NOZORDER | SWP_NOACTIVATE | (if bDrop != 0 { SWP_NOREDRAW } else { 0 }),
        );
    }

    SetWindowPos(
        (*lphc).hWndLBox,
        0 as HWND,
        (*rectLB).left,
        (*rectLB).top,
        (*rectLB).right - (*rectLB).left,
        (*rectLB).bottom - (*rectLB).top,
        SWP_NOACTIVATE | SWP_NOZORDER | (if bDrop != 0 { SWP_NOREDRAW } else { 0 }),
    );

    if bDrop != 0 {
        if (*lphc).wState & CBF_DROPPED != 0 {
            (*lphc).wState &= !CBF_DROPPED;
            ShowWindow((*lphc).hWndLBox, SW_HIDE as i32);
        }

        if bRedraw != 0 && (*lphc).wState & CBF_NOREDRAW == 0 {
            RedrawWindow(
                (*lphc).self_,
                core::ptr::null_mut(),
                0 as HRGN,
                RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW,
            );
        }
    }
}

unsafe fn COMBO_Size(lphc: LPHEADCOMBO) {
    if CB_GETTYPE(lphc) != CBS_SIMPLE {
        let mut rc: RECT = core::mem::zeroed();

        GetWindowRect((*lphc).self_, &mut rc);
        let curComboHeight = rc.bottom - rc.top;
        let curComboWidth = rc.right - rc.left;
        let newComboHeight = CBGetTextAreaHeight((*lphc).self_, lphc) + 2 * COMBO_YBORDERSIZE();

        if curComboHeight > newComboHeight {
            (*lphc).droppedRect.bottom =
                (*lphc).droppedRect.top + curComboHeight - newComboHeight;
        }

        if curComboHeight != newComboHeight {
            SetWindowPos(
                (*lphc).self_,
                0 as HWND,
                0,
                0,
                curComboWidth,
                newComboHeight,
                (SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOREDRAW) as UINT,
            );
        }
    }

    CBCalcPlacement(
        (*lphc).self_,
        lphc,
        core::ptr::addr_of_mut!((*lphc).textRect),
        core::ptr::addr_of_mut!((*lphc).buttonRect),
        core::ptr::addr_of_mut!((*lphc).droppedRect),
    );

    CBResetPos(
        lphc,
        core::ptr::addr_of!((*lphc).textRect),
        core::ptr::addr_of!((*lphc).droppedRect),
        FALSE as BOOL,
    );
}

unsafe fn COMBO_Font(lphc: LPHEADCOMBO, hFont: HFONT, bRedraw: BOOL) {
    (*lphc).hFont = hFont;

    if (*lphc).wState & CBF_EDIT != 0 {
        SendMessageW((*lphc).hWndEdit, WM_SETFONT, hFont as WPARAM, bRedraw as LPARAM);
    }
    SendMessageW((*lphc).hWndLBox, WM_SETFONT, hFont as WPARAM, bRedraw as LPARAM);

    if CB_GETTYPE(lphc) == CBS_SIMPLE {
        CBCalcPlacement(
            (*lphc).self_,
            lphc,
            core::ptr::addr_of_mut!((*lphc).textRect),
            core::ptr::addr_of_mut!((*lphc).buttonRect),
            core::ptr::addr_of_mut!((*lphc).droppedRect),
        );

        CBResetPos(
            lphc,
            core::ptr::addr_of!((*lphc).textRect),
            core::ptr::addr_of!((*lphc).droppedRect),
            1 as BOOL,
        );
    } else {
        CBForceDummyResize(lphc);
    }
}

unsafe fn COMBO_SetItemHeight(lphc: LPHEADCOMBO, index: INT, height: INT) -> LRESULT {
    let mut lRet: LRESULT = CB_ERR as LRESULT;

    if index == -1 {
        if height < 32768 {
            (*lphc).editHeight = (height + 2) as _;

            if CB_GETTYPE(lphc) == CBS_SIMPLE {
                CBCalcPlacement(
                    (*lphc).self_,
                    lphc,
                    core::ptr::addr_of_mut!((*lphc).textRect),
                    core::ptr::addr_of_mut!((*lphc).buttonRect),
                    core::ptr::addr_of_mut!((*lphc).droppedRect),
                );

                CBResetPos(
                    lphc,
                    core::ptr::addr_of!((*lphc).textRect),
                    core::ptr::addr_of!((*lphc).droppedRect),
                    1,
                );
            } else {
                CBForceDummyResize(lphc);
            }

            lRet = height as LRESULT;
        }
    } else if CB_OWNERDRAWN(lphc) != 0 {
        lRet = SendMessageW(
            (*lphc).hWndLBox,
            LB_SETITEMHEIGHT,
            index as WPARAM,
            height as LPARAM,
        );
    }
    lRet
}

unsafe fn COMBO_SelectString(lphc: LPHEADCOMBO, start: INT, pText: LPARAM, unicode: BOOL) -> LRESULT {
    let index: INT = if unicode != 0 {
        SendMessageW((*lphc).hWndLBox, LB_SELECTSTRING, start as WPARAM, pText) as INT
    } else {
        SendMessageA((*lphc).hWndLBox, LB_SELECTSTRING, start as WPARAM, pText) as INT
    };
    if index >= 0 {
        if (*lphc).wState & CBF_EDIT as u32 != 0 {
            CBUpdateEdit(lphc, index);
        } else {
            InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).textRect), TRUE as BOOL);
        }
    }
    index as LRESULT
}

unsafe fn COMBO_LButtonDown(lphc: LPHEADCOMBO, lParam: LPARAM) {
    let mut pt: POINT = core::mem::zeroed();
    let bButton: BOOL;
    let hWnd: HWND = (*lphc).self_;

    pt.x = crate::internal::LOWORD(lParam as DWORD) as i16 as LONG;
    pt.y = crate::internal::HIWORD(lParam as DWORD) as i16 as LONG;
    bButton = PtInRect(core::ptr::addr_of_mut!((*lphc).buttonRect), pt);

    if (CB_GETTYPE(lphc) == CBS_DROPDOWNLIST)
        || (bButton != 0 && (CB_GETTYPE(lphc) == CBS_DROPDOWN))
    {
        (*lphc).wState |= CBF_BUTTONDOWN;
        if (*lphc).wState & CBF_DROPPED != 0 {
            (*lphc).wState &= !CBF_BUTTONDOWN;
            CBRollUp(lphc, 1, 0);
            if IsWindow(hWnd) == 0 {
                return;
            }

            if (*lphc).wState & CBF_CAPTURE != 0 {
                (*lphc).wState &= !CBF_CAPTURE;
                ReleaseCapture();
            }
        } else {
            (*lphc).wState |= CBF_CAPTURE;
            SetCapture(hWnd);
            CBDropDown(lphc);
        }
        if bButton != 0 {
            CBRepaintButton(lphc);
        }
    }
}

unsafe fn COMBO_LButtonUp(lphc: LPHEADCOMBO) {
    if (*lphc).wState & CBF_CAPTURE != 0 {
        (*lphc).wState &= !CBF_CAPTURE;
        if CB_GETTYPE(lphc) == CBS_DROPDOWN {
            let index = CBUpdateLBox(lphc, 1);
            if index >= 0 {
                (*lphc).wState |= CBF_NOLBSELECT;
                CBUpdateEdit(lphc, index);
                (*lphc).wState &= !CBF_NOLBSELECT;
            }
        }
        ReleaseCapture();
        SetCapture((*lphc).hWndLBox);
    }

    if (*lphc).wState & CBF_BUTTONDOWN != 0 {
        (*lphc).wState &= !CBF_BUTTONDOWN;
        CBRepaintButton(lphc);
    }
}

unsafe fn COMBO_MouseMove(lphc: LPHEADCOMBO, wParam: WPARAM, lParam: LPARAM) {
    let mut pt: POINT = core::mem::zeroed();
    let mut lbRect: RECT = core::mem::zeroed();

    pt.x = (crate::internal::LOWORD(lParam as DWORD) as i16) as LONG;
    pt.y = (crate::internal::HIWORD(lParam as DWORD) as i16) as LONG;

    if (*lphc).wState & CBF_BUTTONDOWN != 0 {
        let bButton = PtInRect(core::ptr::addr_of_mut!((*lphc).buttonRect), pt);

        if bButton == 0 {
            (*lphc).wState &= !CBF_BUTTONDOWN;
            CBRepaintButton(lphc);
        }
    }

    GetClientRect((*lphc).hWndLBox, &mut lbRect);
    MapWindowPoints((*lphc).self_, (*lphc).hWndLBox, &mut pt, 1);
    if PtInRect(&mut lbRect, pt) != 0 {
        (*lphc).wState &= !CBF_CAPTURE;
        ReleaseCapture();
        if CB_GETTYPE(lphc) == CBS_DROPDOWN {
            CBUpdateLBox(lphc, 1);
        }

        SendMessageW((*lphc).hWndLBox, WM_LBUTTONDOWN, wParam, lParam);
    }
}

unsafe fn COMBO_GetComboBoxInfo(lphc: *const HEADCOMBO, pcbi: *mut COMBOBOXINFO) -> LRESULT {
    if pcbi.is_null() || ((*pcbi).cbSize as usize) < core::mem::size_of::<COMBOBOXINFO>() {
        return 0;
    }

    // COMBOBOXINFO rcItem/rcButton are RECTL; textRect/buttonRect are layout-identical tagRECT.
    (*pcbi).rcItem = *(&(*lphc).textRect as *const RECT as *const RECTL);
    (*pcbi).rcButton = *(&(*lphc).buttonRect as *const RECT as *const RECTL);
    (*pcbi).stateButton = 0;
    if (*lphc).wState & CBF_BUTTONDOWN != 0 {
        (*pcbi).stateButton |= STATE_SYSTEM_PRESSED;
    }
    if IsRectEmpty(core::ptr::addr_of!((*lphc).buttonRect)) != 0 {
        (*pcbi).stateButton |= STATE_SYSTEM_INVISIBLE;
    }
    (*pcbi).hwndCombo = (*lphc).self_;
    (*pcbi).hwndItem = (*lphc).hWndEdit;
    (*pcbi).hwndList = (*lphc).hWndLBox;
    1
}

unsafe fn strdupA(str: LPCSTR) -> *mut core::ffi::c_char {
    if str.is_null() {
        return core::ptr::null_mut();
    }

    let len = strlen(str) as DWORD;
    let ret = HeapAlloc(GetProcessHeap(), 0, (len + 1) as SIZE_T) as *mut core::ffi::c_char;
    if !ret.is_null() {
        memcpy(ret as *mut _, str as *const _, (len + 1) as usize);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ComboWndProc_common(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    mut lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let lphc: LPHEADCOMBO = GetWindowLongPtrW(hwnd, 0) as LPHEADCOMBO;
    // __REACTOS__
    let pWnd: PWND;

    pWnd = ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_COMBOBOX as WORD);
        } else {
            if (*pWnd).fnid != FNID_COMBOBOX as u32 {
                ERR!(
                    "Wrong window class for ComboBox! fnId 0x%x\n",
                    (*pWnd).fnid as UINT,
                );
                return 0;
            }
        }
    }

    TRACE!(b"[%p]: msg %s wp %08lx lp %08lx\n\0".as_ptr() as *const i8, hwnd);

    if !lphc.is_null() || message == WM_NCCREATE {
        match message {
            x if x == WM_NCCREATE => {
                let style: LONG = if unicode != 0 {
                    (*(lParam as LPCREATESTRUCTW)).style
                } else {
                    (*(lParam as LPCREATESTRUCTA)).style
                };
                return COMBO_NCCreate(hwnd, style);
            }
            x if x == WM_NCDESTROY => {
                COMBO_NCDestroy(lphc);
                // __REACTOS__
                NtUserSetWindowFNID(hwnd, FNID_DESTROY as WORD);
                // break -> DefWindowProc
            }
            x if x == WM_CREATE => {
                let hwndParent: HWND;
                let style: LONG;
                if unicode != 0 {
                    hwndParent = (*(lParam as LPCREATESTRUCTW)).hwndParent;
                    style = (*(lParam as LPCREATESTRUCTW)).style;
                } else {
                    hwndParent = (*(lParam as LPCREATESTRUCTA)).hwndParent;
                    style = (*(lParam as LPCREATESTRUCTA)).style;
                }
                return COMBO_Create(hwnd, lphc, hwndParent, style, unicode);
            }
            x if x == WM_PRINTCLIENT || x == WM_PAINT => {
                /* wParam may contain a valid HDC! */
                return COMBO_Paint(lphc, wParam as HDC);
            }
            x if x == WM_ERASEBKGND => {
                /* do all painting in WM_PAINT like Windows does */
                return 1;
            }
            x if x == WM_GETDLGCODE => {
                let mut result: LRESULT = (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
                if lParam != 0 && (*(lParam as LPMSG)).message == WM_KEYDOWN {
                    let vk: i32 = (*(lParam as LPMSG)).wParam as i32;

                    if (vk == VK_RETURN as i32 || vk == VK_ESCAPE as i32)
                        && ((*lphc).wState & CBF_DROPPED) != 0
                    {
                        result |= DLGC_WANTMESSAGE as LRESULT;
                    }
                }
                return result;
            }
            x if x == WM_SIZE => {
                if !(*lphc).hWndLBox.is_null() && ((*lphc).wState & CBF_NORESIZE) == 0 {
                    COMBO_Size(lphc);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_SETFONT => {
                COMBO_Font(lphc, wParam as HFONT, lParam as BOOL);
                return TRUE as LRESULT;
            }
            x if x == WM_GETFONT => {
                return (*lphc).hFont as LRESULT;
            }
            x if x == WM_SETFOCUS => {
                if (*lphc).wState & CBF_EDIT != 0 {
                    SetFocus((*lphc).hWndEdit);
                    /* The first time focus is received, select all the text */
                    if ((*lphc).wState & CBF_BEENFOCUSED) == 0 {
                        SendMessageW((*lphc).hWndEdit, EM_SETSEL, 0, -1i32 as LPARAM);
                        (*lphc).wState |= CBF_BEENFOCUSED;
                    }
                } else {
                    COMBO_SetFocus(lphc);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_KILLFOCUS => {
                let hwndFocus: HWND = WIN_GetFullHandle(wParam as HWND);
                if hwndFocus.is_null()
                    || (hwndFocus != (*lphc).hWndEdit && hwndFocus != (*lphc).hWndLBox)
                {
                    COMBO_KillFocus(lphc);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_COMMAND => {
                return COMBO_Command(lphc, wParam, WIN_GetFullHandle(lParam as HWND));
            }
            x if x == WM_GETTEXT => {
                return if unicode != 0 {
                    COMBO_GetTextW(lphc, wParam as INT, lParam as LPWSTR)
                } else {
                    COMBO_GetTextA(lphc, wParam as INT, lParam as LPSTR)
                };
            }
            x if x == WM_SETTEXT || x == WM_GETTEXTLENGTH || x == WM_CLEAR => {
                if message == WM_GETTEXTLENGTH && !ISWIN31() && ((*lphc).wState & CBF_EDIT) == 0 {
                    let j: i32 = SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0) as i32;
                    if j == -1 {
                        return 0;
                    }
                    return if unicode != 0 {
                        SendMessageW((*lphc).hWndLBox, LB_GETTEXTLEN, j as WPARAM, 0)
                    } else {
                        SendMessageA((*lphc).hWndLBox, LB_GETTEXTLEN, j as WPARAM, 0)
                    };
                } else if (*lphc).wState & CBF_EDIT != 0 {
                    let ret: LRESULT;
                    (*lphc).wState |= CBF_NOEDITNOTIFY;
                    ret = if unicode != 0 {
                        SendMessageW((*lphc).hWndEdit, message, wParam, lParam)
                    } else {
                        SendMessageA((*lphc).hWndEdit, message, wParam, lParam)
                    };
                    (*lphc).wState &= !CBF_NOEDITNOTIFY;
                    return ret;
                } else {
                    return CB_ERR as LRESULT;
                }
            }
            x if x == WM_CUT || x == WM_PASTE || x == WM_COPY => {
                if (*lphc).wState & CBF_EDIT != 0 {
                    return if unicode != 0 {
                        SendMessageW((*lphc).hWndEdit, message, wParam, lParam)
                    } else {
                        SendMessageA((*lphc).hWndEdit, message, wParam, lParam)
                    };
                } else {
                    return CB_ERR as LRESULT;
                }
            }
            x if x == WM_DRAWITEM
                || x == WM_DELETEITEM
                || x == WM_COMPAREITEM
                || x == WM_MEASUREITEM =>
            {
                return COMBO_ItemOp(lphc, message, lParam);
            }
            x if x == WM_ENABLE => {
                if (*lphc).wState & CBF_EDIT != 0 {
                    EnableWindow((*lphc).hWndEdit, wParam as BOOL);
                }
                EnableWindow((*lphc).hWndLBox, wParam as BOOL);

                /* Force the control to repaint when the enabled state changes. */
                InvalidateRect((*lphc).self_, core::ptr::null_mut(), TRUE);
                return TRUE as LRESULT;
            }
            x if x == WM_SETREDRAW => {
                if wParam != 0 {
                    (*lphc).wState &= !CBF_NOREDRAW;
                } else {
                    (*lphc).wState |= CBF_NOREDRAW;
                }

                if (*lphc).wState & CBF_EDIT != 0 {
                    SendMessageW((*lphc).hWndEdit, message, wParam, lParam);
                }
                SendMessageW((*lphc).hWndLBox, message, wParam, lParam);
                return 0;
            }
            x if x == WM_SYSKEYDOWN => {
                // __REACTOS__: KF_ALTDOWN & HIWORD(lParam)
                if (KF_ALTDOWN & crate::internal::HIWORD(lParam as DWORD) as u32) != 0 {
                    if wParam == VK_UP as WPARAM || wParam == VK_DOWN as WPARAM {
                        COMBO_FlipListbox(lphc, FALSE as BOOL, FALSE as BOOL);
                        return 0;
                    }
                }
                // break
            }
            x if x == WM_KEYDOWN
                || x == WM_CHAR
                || x == WM_IME_CHAR =>
            {
                // WM_KEYDOWN has special handling that may fall through to WM_CHAR.
                if message == WM_KEYDOWN {
                    if (wParam == VK_RETURN as WPARAM || wParam == VK_ESCAPE as WPARAM)
                        && ((*lphc).wState & CBF_DROPPED) != 0
                    {
                        CBRollUp(lphc, (wParam == VK_RETURN as WPARAM) as BOOL, FALSE as BOOL);
                        return TRUE as LRESULT;
                    } else if wParam == VK_F4 as WPARAM && ((*lphc).wState & CBF_EUI) == 0 {
                        COMBO_FlipListbox(lphc, FALSE as BOOL, FALSE as BOOL);
                        return TRUE as LRESULT;
                    }
                    /* fall through */
                }
                let hwndTarget: HWND;

                // __REACTOS__
                if (*lphc).wState & CBF_DROPPED != 0 {
                    (*lphc).wState |= CBF_NOROLLUP;
                }
                if (*lphc).wState & CBF_EDIT != 0 {
                    hwndTarget = (*lphc).hWndEdit;
                } else {
                    hwndTarget = (*lphc).hWndLBox;
                }

                return if unicode != 0 {
                    SendMessageW(hwndTarget, message, wParam, lParam)
                } else {
                    SendMessageA(hwndTarget, message, wParam, lParam)
                };
            }
            x if x == WM_LBUTTONDOWN => {
                if ((*lphc).wState & CBF_FOCUSED) == 0 {
                    SetFocus((*lphc).self_);
                }
                if (*lphc).wState & CBF_FOCUSED != 0 {
                    COMBO_LButtonDown(lphc, lParam);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_LBUTTONUP => {
                COMBO_LButtonUp(lphc);
                return TRUE as LRESULT;
            }
            x if x == WM_MOUSEMOVE => {
                if (*lphc).wState & CBF_CAPTURE != 0 {
                    COMBO_MouseMove(lphc, wParam, lParam);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_MOUSEWHEEL => {
                if wParam & (MK_SHIFT | MK_CONTROL) as WPARAM != 0 {
                    return if unicode != 0 {
                        DefWindowProcW(hwnd, message, wParam, lParam)
                    } else {
                        DefWindowProcA(hwnd, message, wParam, lParam)
                    };
                }

                if GET_WHEEL_DELTA_WPARAM(wParam) > 0 {
                    return SendMessageW(hwnd, WM_KEYDOWN, VK_UP as WPARAM, 0);
                }
                if GET_WHEEL_DELTA_WPARAM(wParam) < 0 {
                    return SendMessageW(hwnd, WM_KEYDOWN, VK_DOWN as WPARAM, 0);
                }
                return TRUE as LRESULT;
            }
            x if x == WM_CTLCOLOR
                || x == WM_CTLCOLORMSGBOX
                || x == WM_CTLCOLOREDIT
                || x == WM_CTLCOLORLISTBOX
                || x == WM_CTLCOLORBTN
                || x == WM_CTLCOLORDLG
                || x == WM_CTLCOLORSCROLLBAR
                || x == WM_CTLCOLORSTATIC =>
            {
                // __REACTOS__: Must be Win 4.0 and above.
                if !pWnd.is_null() && ((*pWnd).state2 & WNDS2_WIN40COMPAT) == 0 {
                    // break
                } else if !(*lphc).owner.is_null() {
                    return SendMessageW((*lphc).owner, message, wParam, lParam);
                }
                // break
            }

            /* Combo messages */
            x if x == CB_ADDSTRING => {
                if unicode != 0 {
                    if (*lphc).dwStyle & CBS_LOWERCASE as u32 != 0 {
                        CharLowerW(lParam as LPWSTR);
                    } else if (*lphc).dwStyle & CBS_UPPERCASE as u32 != 0 {
                        CharUpperW(lParam as LPWSTR);
                    }
                    return SendMessageW((*lphc).hWndLBox, LB_ADDSTRING, 0, lParam);
                } else {
                    /* unlike the unicode version, the ansi version does not overwrite
                    the string if converting case */
                    let mut string: *mut core::ffi::c_char = core::ptr::null_mut();
                    let ret: LRESULT;
                    if (*lphc).dwStyle & CBS_LOWERCASE as u32 != 0 {
                        string = strdupA(lParam as LPSTR);
                        CharLowerA(string);
                    } else if (*lphc).dwStyle & CBS_UPPERCASE as u32 != 0 {
                        string = strdupA(lParam as LPSTR);
                        CharUpperA(string);
                    }

                    ret = SendMessageA(
                        (*lphc).hWndLBox,
                        LB_ADDSTRING,
                        0,
                        if !string.is_null() { string as LPARAM } else { lParam },
                    );
                    HeapFree(GetProcessHeap(), 0, string as *mut core::ffi::c_void);
                    return ret;
                }
            }
            x if x == CB_INSERTSTRING => {
                if unicode != 0 {
                    if (*lphc).dwStyle & CBS_LOWERCASE as u32 != 0 {
                        CharLowerW(lParam as LPWSTR);
                    } else if (*lphc).dwStyle & CBS_UPPERCASE as u32 != 0 {
                        CharUpperW(lParam as LPWSTR);
                    }
                    return SendMessageW((*lphc).hWndLBox, LB_INSERTSTRING, wParam, lParam);
                } else {
                    if (*lphc).dwStyle & CBS_LOWERCASE as u32 != 0 {
                        CharLowerA(lParam as LPSTR);
                    } else if (*lphc).dwStyle & CBS_UPPERCASE as u32 != 0 {
                        CharUpperA(lParam as LPSTR);
                    }
                    return SendMessageA((*lphc).hWndLBox, LB_INSERTSTRING, wParam, lParam);
                }
            }
            x if x == CB_DELETESTRING => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_DELETESTRING, wParam, 0)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_DELETESTRING, wParam, 0)
                };
            }
            x if x == CB_SELECTSTRING => {
                return COMBO_SelectString(lphc, wParam as INT, lParam, unicode);
            }
            x if x == CB_FINDSTRING => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_FINDSTRING, wParam, lParam)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_FINDSTRING, wParam, lParam)
                };
            }
            x if x == CB_FINDSTRINGEXACT => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_FINDSTRINGEXACT, wParam, lParam)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_FINDSTRINGEXACT, wParam, lParam)
                };
            }
            x if x == CB_SETITEMHEIGHT => {
                return COMBO_SetItemHeight(lphc, wParam as INT, lParam as INT);
            }
            x if x == CB_GETITEMHEIGHT => {
                if wParam as INT >= 0 {
                    /* listbox item */
                    return SendMessageW((*lphc).hWndLBox, LB_GETITEMHEIGHT, wParam, 0);
                }
                return CBGetTextAreaHeight(hwnd, lphc) as LRESULT;
            }
            x if x == CB_RESETCONTENT => {
                SendMessageW((*lphc).hWndLBox, LB_RESETCONTENT, 0, 0);
                if ((*lphc).wState & CBF_EDIT) != 0 && CB_HASSTRINGS(lphc) != 0 {
                    static empty_stringW: [WCHAR; 1] = [0];
                    SendMessageW(
                        (*lphc).hWndEdit,
                        WM_SETTEXT,
                        0,
                        empty_stringW.as_ptr() as LPARAM,
                    );
                } else {
                    InvalidateRect((*lphc).self_, core::ptr::null_mut(), TRUE);
                }
                return TRUE as LRESULT;
            }
            x if x == CB_INITSTORAGE => {
                return SendMessageW((*lphc).hWndLBox, LB_INITSTORAGE, wParam, lParam);
            }
            x if x == CB_GETHORIZONTALEXTENT => {
                return SendMessageW((*lphc).hWndLBox, LB_GETHORIZONTALEXTENT, 0, 0);
            }
            x if x == CB_SETHORIZONTALEXTENT => {
                return SendMessageW((*lphc).hWndLBox, LB_SETHORIZONTALEXTENT, wParam, 0);
            }
            x if x == CB_GETTOPINDEX => {
                return SendMessageW((*lphc).hWndLBox, LB_GETTOPINDEX, 0, 0);
            }
            x if x == CB_GETLOCALE => {
                return SendMessageW((*lphc).hWndLBox, LB_GETLOCALE, 0, 0);
            }
            x if x == CB_SETLOCALE => {
                return SendMessageW((*lphc).hWndLBox, LB_SETLOCALE, wParam, 0);
            }
            x if x == CB_SETDROPPEDWIDTH => {
                if CB_GETTYPE(lphc) == CBS_SIMPLE || wParam as INT >= 32768 {
                    return CB_ERR as LRESULT;
                }
                /* new value must be higher than combobox width */
                if wParam as INT >= (*lphc).droppedRect.right - (*lphc).droppedRect.left {
                    (*lphc).droppedWidth = wParam as INT;
                } else if wParam != 0 {
                    (*lphc).droppedWidth = 0;
                }

                /* recalculate the combobox area */
                CBCalcPlacement(
                    hwnd,
                    lphc,
                    core::ptr::addr_of_mut!((*lphc).textRect),
                    core::ptr::addr_of_mut!((*lphc).buttonRect),
                    core::ptr::addr_of_mut!((*lphc).droppedRect),
                );

                /* fall through */
                if (*lphc).droppedWidth != 0 {
                    return (*lphc).droppedWidth as LRESULT;
                }
                return ((*lphc).droppedRect.right - (*lphc).droppedRect.left) as LRESULT;
            }
            x if x == CB_GETDROPPEDWIDTH => {
                if (*lphc).droppedWidth != 0 {
                    return (*lphc).droppedWidth as LRESULT;
                }
                return ((*lphc).droppedRect.right - (*lphc).droppedRect.left) as LRESULT;
            }
            x if x == CB_GETDROPPEDCONTROLRECT => {
                if lParam != 0 {
                    CBGetDroppedControlRect(lphc, lParam as LPRECT);
                }
                return CB_OKAY as LRESULT;
            }
            x if x == CB_GETDROPPEDSTATE => {
                return (((*lphc).wState & CBF_DROPPED) != 0) as LRESULT;
            }
            x if x == CB_DIR => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_DIR, wParam, lParam)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_DIR, wParam, lParam)
                };
            }
            x if x == CB_SHOWDROPDOWN => {
                if CB_GETTYPE(lphc) != CBS_SIMPLE {
                    if wParam != 0 {
                        if ((*lphc).wState & CBF_DROPPED) == 0 {
                            CBDropDown(lphc);
                        }
                    } else if (*lphc).wState & CBF_DROPPED != 0 {
                        CBRollUp(lphc, FALSE as BOOL, TRUE as BOOL);
                    }
                }
                return TRUE as LRESULT;
            }
            x if x == CB_GETCOUNT => {
                return SendMessageW((*lphc).hWndLBox, LB_GETCOUNT, 0, 0);
            }
            x if x == CB_GETCURSEL => {
                return SendMessageW((*lphc).hWndLBox, LB_GETCURSEL, 0, 0);
            }
            x if x == CB_SETCURSEL => {
                lParam = SendMessageW((*lphc).hWndLBox, LB_SETCURSEL, wParam, 0);
                if lParam >= 0 {
                    SendMessageW((*lphc).hWndLBox, LB_SETTOPINDEX, wParam, 0);
                }

                /* no LBN_SELCHANGE in this case, update manually */
                CBPaintText(lphc, core::ptr::null_mut());
                (*lphc).wState &= !CBF_SELCHANGE;
                return lParam;
            }
            x if x == CB_GETLBTEXT => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_GETTEXT, wParam, lParam)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_GETTEXT, wParam, lParam)
                };
            }
            x if x == CB_GETLBTEXTLEN => {
                return if unicode != 0 {
                    SendMessageW((*lphc).hWndLBox, LB_GETTEXTLEN, wParam, 0)
                } else {
                    SendMessageA((*lphc).hWndLBox, LB_GETTEXTLEN, wParam, 0)
                };
            }
            x if x == CB_GETITEMDATA => {
                return SendMessageW((*lphc).hWndLBox, LB_GETITEMDATA, wParam, 0);
            }
            x if x == CB_SETITEMDATA => {
                return SendMessageW((*lphc).hWndLBox, LB_SETITEMDATA, wParam, lParam);
            }
            x if x == CB_GETEDITSEL => {
                /* Edit checks passed parameters itself */
                if (*lphc).wState & CBF_EDIT != 0 {
                    return SendMessageW((*lphc).hWndEdit, EM_GETSEL, wParam, lParam);
                }
                return CB_ERR as LRESULT;
            }
            x if x == CB_SETEDITSEL => {
                if (*lphc).wState & CBF_EDIT != 0 {
                    return SendMessageW(
                        (*lphc).hWndEdit,
                        EM_SETSEL,
                        crate::internal::LOWORD(lParam as DWORD) as i16 as i32 as WPARAM,
                        crate::internal::HIWORD(lParam as DWORD) as i16 as i32 as LPARAM,
                    );
                }
                return CB_ERR as LRESULT;
            }
            x if x == CB_SETEXTENDEDUI => {
                if CB_GETTYPE(lphc) == CBS_SIMPLE {
                    return CB_ERR as LRESULT;
                }
                if wParam != 0 {
                    (*lphc).wState |= CBF_EUI;
                } else {
                    (*lphc).wState &= !CBF_EUI;
                }
                return CB_OKAY as LRESULT;
            }
            x if x == CB_GETEXTENDEDUI => {
                return (((*lphc).wState & CBF_EUI) != 0) as LRESULT;
            }
            x if x == CB_GETCOMBOBOXINFO => {
                return COMBO_GetComboBoxInfo(lphc, lParam as *mut COMBOBOXINFO);
            }
            x if x == CB_LIMITTEXT => {
                if (*lphc).wState & CBF_EDIT != 0 {
                    return SendMessageW((*lphc).hWndEdit, EM_LIMITTEXT, wParam, lParam);
                }
                return TRUE as LRESULT;
            }

            // __REACTOS__
            x if x == WM_UPDATEUISTATE => {
                if unicode != 0 {
                    DefWindowProcW((*lphc).self_, message, wParam, lParam);
                } else {
                    DefWindowProcA((*lphc).self_, message, wParam, lParam);
                }

                if COMBO_update_uistate(lphc) != 0 {
                    /* redraw text */
                    if ((*lphc).wState & CBF_EDIT) == 0 {
                        NtUserInvalidateRect(
                            (*lphc).self_,
                            core::ptr::addr_of!((*lphc).textRect) as _,
                            TRUE,
                        );
                    }
                }
                // break
            }

            x if x == WM_CBLOSTTEXTFOCUS => {
                /* undocumented message - deselects the text when focus is lost */
                if !(*lphc).hWndEdit.is_null() {
                    SendMessageW((*lphc).self_, WM_LBUTTONUP, 0, 0xFFFFFFFFu32 as LPARAM);
                    SendMessageW((*lphc).hWndEdit, EM_SETSEL, 0, 0);
                    (*lphc).wState &= !(CBF_FOCUSED | CBF_BEENFOCUSED);
                    CB_NOTIFY(lphc, CBN_KILLFOCUS as INT);
                }
                return TRUE as LRESULT;
            }

            _ => {
                if message >= WM_USER {
                    WARN!(
                        b"unknown msg WM_USER+%04x wp=%04lx lp=%08lx\n\0".as_ptr() as *const i8,
                    );
                }
                // break
            }
        }
    }
    if unicode != 0 {
        DefWindowProcW(hwnd, message, wParam, lParam)
    } else {
        DefWindowProcA(hwnd, message, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn ComboWndProcA(hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hwnd) == 0 {
        return 0;
    }
    ComboWndProc_common(hwnd, message, wParam, lParam, FALSE)
}

#[no_mangle]
pub unsafe extern "system" fn ComboWndProcW(hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hwnd) == 0 {
        return 0;
    }
    ComboWndProc_common(hwnd, message, wParam, lParam, TRUE)
}

#[no_mangle]
pub unsafe extern "system" fn GetComboBoxInfo(
    hwndCombo: HWND,
    pcbi: PCOMBOBOXINFO,
) -> BOOL {
    NtUserGetComboBoxInfo(hwndCombo, pcbi)
}
