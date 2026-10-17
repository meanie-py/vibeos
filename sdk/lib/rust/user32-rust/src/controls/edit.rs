use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::wow64::DbgPrint;
use crate::windows::message::GlobalAlloc;
use crate::windows::window::HeapAlloc;
use crate::windows::message::HeapReAlloc;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
use crate::windows::rect::IsRectEmpty;
use crate::controls::scrollbar::SetScrollInfo;
use core::ffi::c_void;
// NOTE: no `use ntkrnl_sys::ddk::*` here — `tm` in this file is always a local
// TEXTMETRICW variable, and ntkrnl_sys is not a dependency of user32-rust.

// Word/coordinate macros + ValidateHwnd (FASTCALL) live in crate::internal.
use crate::internal::{LOWORD, HIWORD, MAKELONG, MAKEWPARAM, ValidateHwnd, NtCurrentTeb};
// GetControlBrush is an NtUser* wrapper (crate::ntwrapper).
use crate::ntwrapper::GetControlBrush;
// These are NtUser* syscall forwarders (ntwrapper.h EXTINLINE wrappers), not linkable
// user32 exports — call through the crate's wrappers.
use crate::ntwrapper::{
    BeginPaint, CloseClipboard, CreateCaret, DestroyMenu, EmptyClipboard, EndPaint, GetDC,
    InvalidateRect, InvalidateRgn, IsClipboardFormatAvailable, KillTimer, SetCapture, SetFocus,
    SetTimer, ShowCaret,
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

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── user32 own-exports + gdi32/imm32/usp10 imports (extern "system"). ──────────
extern "system" {
    // user32 own-exports / imports
    fn GetWindowLongW(hWnd: HWND, nIndex: i32) -> LONG;
    fn SetWindowLongW(hWnd: HWND, nIndex: i32, dwNewLong: LONG) -> LONG;
    fn DefWindowProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PostMessageW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn GetParent(hWnd: HWND) -> HWND;
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: INT) -> HWND;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn MapWindowPoints(hWndFrom: HWND, hWndTo: HWND, lpPoints: *mut POINT, cPoints: UINT) -> INT;
    fn GetScrollInfo(hwnd: HWND, fnBar: INT, lpsi: *mut SCROLLINFO) -> BOOL;
    fn GetScrollPos(hWnd: HWND, nBar: INT) -> INT;
    fn ScrollWindowEx(hWnd: HWND, dx: INT, dy: INT, prcScroll: *const RECT, prcClip: *const RECT,
                      hrgnUpdate: HRGN, prcUpdate: LPRECT, flags: UINT) -> INT;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn GetKeyState(nVirtKey: INT) -> SHORT;
    fn GetKeyboardLayout(idThread: DWORD) -> HKL;
    fn GetCapture() -> HWND;
    fn ReleaseCapture() -> BOOL;
    fn DestroyCaret() -> BOOL;
    fn SetCaretPos(x: INT, y: INT) -> BOOL;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> INT;
    fn GetSysColor(nIndex: INT) -> DWORD;
    fn GetSysColorBrush(nIndex: INT) -> HBRUSH;
    fn IntersectRect(lprcDst: LPRECT, lprcSrc1: *const RECT, lprcSrc2: *const RECT) -> BOOL;
    fn SetRectEmpty(lprc: LPRECT) -> BOOL;
    fn CopyRect(lprcDst: LPRECT, lprcSrc: *const RECT) -> BOOL;
    fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: *mut c_void, fWinIni: UINT) -> BOOL;
    fn LoadMenuA(hInstance: HINSTANCE, lpMenuName: LPCSTR) -> HMENU;
    fn GetSubMenu(hMenu: HMENU, nPos: INT) -> HMENU;
    fn EnableMenuItem(hMenu: HMENU, uIDEnableItem: UINT, uEnable: UINT) -> BOOL;
    fn TrackPopupMenu(hMenu: HMENU, uFlags: UINT, x: INT, y: INT, nReserved: INT,
                      hWnd: HWND, prcRect: *const RECT) -> BOOL;
    fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    fn GetClipboardData(uFormat: UINT) -> HANDLE;
    fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
    fn CharUpperBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD;
    fn CharLowerBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD;
    fn GetProcessVersion(ProcessId: DWORD) -> DWORD;
    fn GetVersionExW(lpVersionInformation: *mut OSVERSIONINFOW) -> BOOL;
    // imm32 IMM functions are NOT direct imports — controls/edit.c calls them via the
    // IMM_FN(name) macro = gImmApiEntries.pname (a runtime-loaded dispatch table in the
    // still-C misc/imm.c). Replicated below as the Imm32ApiTable mirror + inline wrappers.
    // gdi32
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn GetStockObject(i: INT) -> HGDIOBJ;
    fn GetObjectW(h: HGDIOBJ, c: INT, pv: *mut c_void) -> INT;
    fn CreateRectRgn(x1: INT, y1: INT, x2: INT, y2: INT) -> HRGN;
    fn SetRectRgn(hrgn: HRGN, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn CombineRgn(hrgnDst: HRGN, hrgnSrc1: HRGN, hrgnSrc2: HRGN, iMode: INT) -> INT;
    fn IntersectClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> INT;
    fn GetClipBox(hdc: HDC, lprect: LPRECT) -> INT;
    fn PatBlt(hdc: HDC, x: INT, y: INT, w: INT, h: INT, rop: DWORD) -> BOOL;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn GetTextMetricsW(hdc: HDC, lptm: *mut TEXTMETRICW) -> BOOL;
    // usp10 (Uniscribe)
    fn ScriptBreak(pwcChars: *const WCHAR, cChars: INT, psa: *const SCRIPT_ANALYSIS,
                   psla: *mut SCRIPT_LOGATTR) -> HRESULT;
    fn ScriptStringAnalyse(hdc: HDC, pString: *const c_void, cString: INT, cGlyphs: INT,
                           iCharset: INT, dwFlags: DWORD, iReqWidth: INT,
                           psControl: *mut c_void, psState: *mut c_void, piDx: *const INT,
                           pTabdef: *mut SCRIPT_TABDEF, pbInClass: *const BYTE,
                           pssa: *mut SCRIPT_STRING_ANALYSIS) -> HRESULT;
    fn ScriptStringFree(pssa: *mut SCRIPT_STRING_ANALYSIS) -> HRESULT;
    fn ScriptStringCPtoX(ssa: SCRIPT_STRING_ANALYSIS, icp: INT, fTrailing: BOOL, pX: *mut INT) -> HRESULT;
    fn ScriptStringXtoCP(ssa: SCRIPT_STRING_ANALYSIS, iX: INT, piCh: *mut INT, piTrailing: *mut INT) -> HRESULT;
    fn ScriptStringGetLogicalWidths(ssa: SCRIPT_STRING_ANALYSIS, piDx: *mut INT) -> HRESULT;
    fn ScriptStringOut(ssa: SCRIPT_STRING_ANALYSIS, iX: INT, iY: INT, uOptions: UINT,
                       prc: *const RECT, iMinSel: INT, iMaxSel: INT, fDisabled: BOOL) -> HRESULT;
    fn ScriptString_pSize(ssa: SCRIPT_STRING_ANALYSIS) -> *const SIZE;
    fn ScriptString_pcOutChars(ssa: SCRIPT_STRING_ANALYSIS) -> *const INT;
    // gdi32 — text painting (EDIT_PaintText)
    fn GetBkMode(hdc: HDC) -> INT;
    fn GetBkColor(hdc: HDC) -> COLORREF;
    fn GetTextColor(hdc: HDC) -> COLORREF;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkMode(hdc: HDC, mode: INT) -> INT;
    fn TextOutW(hdc: HDC, x: INT, y: INT, lpString: LPCWSTR, c: INT) -> BOOL;
    fn TabbedTextOutW(hdc: HDC, x: INT, y: INT, lpString: LPCWSTR, chCount: INT,
                      nTabPositions: INT, lpnTabStopPositions: *const INT, nTabOrigin: INT) -> LONG;
    fn GetTextExtentPoint32W(hdc: HDC, lpString: LPCWSTR, c: INT, psizl: *mut SIZE) -> BOOL;
}

extern "C" {
}

// ── kernel32 heap / global / multibyte (extern "system"). ─────────────────────
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    fn LocalReAlloc(hMem: HLOCAL, uBytes: SIZE_T, uFlags: UINT) -> HLOCAL;
    fn LocalLock(hMem: HLOCAL) -> *mut c_void;
    fn LocalUnlock(hMem: HLOCAL) -> BOOL;
    fn LocalSize(hMem: HLOCAL) -> SIZE_T;
    fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT,
                           lpMultiByteStr: LPSTR, cbMultiByte: INT,
                           lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL) -> INT;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT,
                           lpWideCharStr: LPWSTR, cchWideChar: INT) -> INT;
}

// ── user32-internal C symbols (cohab — surviving C objects). ───────────────────
extern "C" {
    fn SPY_GetMsgName(msg: UINT, hWnd: HWND) -> *const CHAR;
    static User32Instance: HINSTANCE; // dllmain.c
    // CRT (msvcrt)
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const CHAR) -> usize;
}

// ── file-local helpers (macros/inlines the C uses; not shared symbols) ─────────

#[inline] fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline] fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
#[inline] fn abs(v: INT) -> INT { if v < 0 { -v } else { v } }

/// `ROUND_TO_GROW(size)` — edit.c macro: round up to a multiple of GROWLENGTH.
#[inline] fn ROUND_TO_GROW(size: UINT) -> UINT { (size + (GROWLENGTH - 1)) & !(GROWLENGTH - 1) }
/// `FAILED(hr)` — winerror.h macro.
#[inline] fn FAILED(hr: HRESULT) -> bool { hr < 0 }
/// `PRIMARYLANGID(lgid)` — winnt.h macro.
#[inline] fn PRIMARYLANGID(lgid: WORD) -> WORD { lgid & 0x3ff }
/// `GET_WHEEL_DELTA_WPARAM(wParam)` — winuser.h macro.
#[inline] fn GET_WHEEL_DELTA_WPARAM(wParam: WPARAM) -> i32 { (HIWORD(wParam as DWORD) as i16) as i32 }

/// `GetWin32ClientInfo()` — user_x.h FORCEINLINE over the TEB.
#[inline] unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

/// `strlenW` — wcslen over the UTF-16 buffer (user32 internal inline).
#[inline] unsafe fn strlenW(mut s: *const WCHAR) -> UINT {
    if s.is_null() { return 0; }
    let mut n: UINT = 0;
    while *s != 0 { s = s.offset(1); n += 1; }
    n
}
/// `strcpyW(dst, src)` — wcscpy (returns dst).
#[inline] unsafe fn strcpyW(dst: LPWSTR, mut src: LPCWSTR) -> LPWSTR {
    let mut p = dst;
    while *src != 0 { *p = *src; p = p.offset(1); src = src.offset(1); }
    *p = 0;
    dst
}
/// `lstrcpynW(dst, src, n)` — copy at most n-1 WCHARs + NUL.
#[inline] unsafe fn lstrcpynW(dst: LPWSTR, mut src: LPCWSTR, n: INT) -> LPWSTR {
    if n <= 0 { return dst; }
    let mut p = dst;
    let mut i = 0;
    while i < n - 1 && *src != 0 { *p = *src; p = p.offset(1); src = src.offset(1); i += 1; }
    *p = 0;
    dst
}

// ── USP10 (Uniscribe) types + flags — not in win32k-sys; edit.rs is the sole user.
pub type SCRIPT_STRING_ANALYSIS = *mut c_void;

pub const SCRIPT_UNDEFINED: WORD = 0;
pub const SSA_PASSWORD: DWORD = 0x00000001;
pub const SSA_TAB: DWORD = 0x00000002;
pub const SSA_FALLBACK: DWORD = 0x00000020;
pub const SSA_GLYPHS: DWORD = 0x00000080;
pub const SSA_LINK: DWORD = 0x00001000;

// ── constants not surfaced by win32k-sys (kernel32 mem flags, winnls, combo id). ─
pub const CP_ACP: UINT = 0;
pub const LMEM_MOVEABLE: UINT = 0x0002;
pub const LMEM_ZEROINIT: UINT = 0x0040;
pub const GMEM_MOVEABLE: UINT = 0x0002;
pub const GMEM_DDESHARE: UINT = 0x2000;
pub const ID_CB_LISTBOX: INT = 1000;

/// `SCRIPT_ANALYSIS` (usp10.h) — 4 bytes (WORD bitfields + SCRIPT_STATE). Only the
/// `eScript` field (low 10 bits of word 0) is touched here; modeled directly.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SCRIPT_ANALYSIS {
    pub eScript: WORD,
    pub s: WORD,
}

/// `SCRIPT_TABDEF` (usp10.h).
#[repr(C)]
pub struct SCRIPT_TABDEF {
    pub cTabStops: INT,
    pub iScale: INT,
    pub pTabStops: *mut INT,
    pub iTabOrigin: INT,
}

/// `SCRIPT_LOGATTR` (usp10.h) — one byte of bitfields. Accessors match the C
/// member-read sites (`fSoftBreak`, `fWhiteSpace`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SCRIPT_LOGATTR(pub BYTE);
impl SCRIPT_LOGATTR {
    #[inline] pub fn fSoftBreak(&self) -> INT { (self.0 & 0x01) as INT }
    #[inline] pub fn fWhiteSpace(&self) -> INT { ((self.0 >> 1) & 0x01) as INT }
}

// ---- file-scope preamble (translated) ----
pub const BUFLIMIT_INITIAL: UINT = 30000;
pub const GROWLENGTH: UINT = 32;
pub const HSCROLL_FRACTION: INT = 3;

pub const EF_MODIFIED: WORD = 0x0001;
pub const EF_FOCUSED: WORD = 0x0002;
pub const EF_UPDATE: WORD = 0x0004;
pub const EF_VSCROLL_TRACK: WORD = 0x0008;
pub const EF_HSCROLL_TRACK: WORD = 0x0010;
pub const EF_AFTER_WRAP: WORD = 0x0080;
pub const EF_USE_SOFTBRK: WORD = 0x0100;
pub const EF_DIALOGMODE: WORD = 0x0200;

pub const END_0: u32 = 0;
pub const END_WRAP: u32 = 1;
pub const END_HARD: u32 = 2;
pub const END_SOFT: u32 = 3;
pub const END_RICH: u32 = 4;
pub type LINE_END = u32;

#[repr(C)]
pub struct tagLINEDEF {
    pub length: INT,
    pub net_length: INT,
    pub ending: LINE_END,
    pub width: INT,
    pub index: INT,
    pub ssa: SCRIPT_STRING_ANALYSIS,
    pub next: *mut tagLINEDEF,
}
pub type LINEDEF = tagLINEDEF;

#[repr(C)]
pub struct EDITSTATE {
    pub is_unicode: BOOL,
    pub text: LPWSTR,
    pub text_length: UINT,
    pub buffer_size: UINT,
    pub buffer_limit: UINT,
    pub font: HFONT,
    pub x_offset: INT,
    pub dwCaretWidth: DWORD,
    pub line_height: INT,
    pub char_width: INT,
    pub style: DWORD,
    pub flags: WORD,
    pub undo_insert_count: INT,
    pub undo_position: UINT,
    pub undo_text: LPWSTR,
    pub undo_buffer_size: UINT,
    pub selection_start: INT,
    pub selection_end: INT,
    pub password_char: WCHAR,
    pub left_margin: INT,
    pub right_margin: INT,
    pub format_rect: RECT,
    pub text_width: INT,
    pub region_posx: INT,
    pub region_posy: INT,
    pub word_break_proc: *mut core::ffi::c_void,
    pub line_count: INT,
    pub y_offset: INT,
    pub bCaptureState: BOOL,
    pub bEnableState: BOOL,
    pub hwndSelf: HWND,
    pub hwndParent: HWND,
    pub hwndListBox: HWND,
    pub wheelDeltaRemainder: INT,
    pub lock_count: INT,
    pub tabs_count: INT,
    pub tabs: LPINT,
    pub first_line_def: *mut LINEDEF,
    pub hloc32W: HLOCAL,
    pub hloc32A: HLOCAL,
    pub hlocapp: HLOCAL,
    pub logAttr: *mut SCRIPT_LOGATTR,
    pub ssa: SCRIPT_STRING_ANALYSIS,
}
pub type PEDITSTATE = *mut EDITSTATE;

pub static empty_stringW: [WCHAR; 1] = [0];

pub static editW: [WCHAR; 5] = ['E' as WCHAR, 'd' as WCHAR, 'i' as WCHAR, 't' as WCHAR, 0];

unsafe fn notify_parent(es: *const EDITSTATE, code: INT) -> BOOL {
    let hwnd = (*es).hwndSelf;
    SendMessageW(
        (*es).hwndParent,
        WM_COMMAND,
        MAKEWPARAM(GetWindowLongPtrW((*es).hwndSelf, GWLP_ID) as _, code as _),
        (*es).hwndSelf as LPARAM,
    );
    IsWindow(hwnd)
}

unsafe fn EDIT_EM_CanUndo(es: *const EDITSTATE) -> BOOL {
    (((*es).undo_insert_count != 0) || (strlenW((*es).undo_text) != 0)) as BOOL
}

unsafe fn EDIT_EM_EmptyUndoBuffer(es: *mut EDITSTATE) {
    (*es).undo_insert_count = 0;
    *(*es).undo_text = 0;
}

unsafe fn get_app_version() -> DWORD {
    static mut version: DWORD = 0;
    if version == 0 {
        let dwEmulatedVersion: DWORD;
        let mut info: OSVERSIONINFOW = core::mem::zeroed();
        let dwProcVersion: DWORD = GetProcessVersion(0);

        info.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;
        GetVersionExW(&mut info);
        dwEmulatedVersion = MAKELONG(info.dwMinorVersion as _, info.dwMajorVersion as _) as DWORD;
        version = if dwProcVersion < dwEmulatedVersion { dwProcVersion } else { dwEmulatedVersion };
    }
    version
}

unsafe fn EDIT_NotifyCtlColor(es: *mut EDITSTATE, hdc: HDC) -> HBRUSH {
    let hbrush: HBRUSH;
    let msg: UINT;

    if get_app_version() >= 0x40000
        && ((*es).bEnableState == 0 || ((*es).style & ES_READONLY) != 0)
    {
        msg = WM_CTLCOLORSTATIC;
    } else {
        msg = WM_CTLCOLOREDIT;
    }

    hbrush = GetControlBrush((*es).hwndSelf, hdc, msg);

    hbrush
}

unsafe fn get_text_length(es: *mut EDITSTATE) -> UINT {
    if (*es).text_length == (-1i32) as UINT {
        (*es).text_length = strlenW((*es).text);
    }
    (*es).text_length
}

unsafe fn EDIT_WordBreakProc(es: *mut EDITSTATE, s: LPWSTR, mut index: INT, count: INT, action: INT) -> INT {
    let mut ret: INT = 0;

    if s.is_null() {
        return 0;
    }

    if (*es).logAttr.is_null() {
        let mut psa: SCRIPT_ANALYSIS = core::mem::zeroed();

        core::ptr::write_bytes(&mut psa as *mut SCRIPT_ANALYSIS, 0, 1);
        psa.eScript = SCRIPT_UNDEFINED as _;

        (*es).logAttr = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<SCRIPT_LOGATTR>() as UINT * get_text_length(es)) as _) as *mut SCRIPT_LOGATTR;
        ScriptBreak((*es).text, get_text_length(es) as INT, &mut psa, (*es).logAttr);
    }

    match action as u32 {
        WB_LEFT => {
            if index != 0 {
                index -= 1;
            }
            while index != 0 && (*(*es).logAttr.offset(index as isize)).fSoftBreak() == 0 {
                index -= 1;
            }
            ret = index;
        }
        WB_RIGHT => {
            if count == 0 {
            } else {
                while index < count && *s.offset(index as isize) != 0 && (*(*es).logAttr.offset(index as isize)).fSoftBreak() == 0 {
                    index += 1;
                }
                ret = index;
            }
        }
        WB_ISDELIMITER => {
            ret = (*(*es).logAttr.offset(index as isize)).fWhiteSpace() as INT;
        }
        _ => {
        }
    }
    ret
}

unsafe fn EDIT_CallWordBreakProc(es: *mut EDITSTATE, start: INT, index: INT, count: INT, action: INT) -> INT {
    let ret: INT;

    if !(*es).word_break_proc.is_null() {
        if (*es).is_unicode != 0 {
            let wbpW: EDITWORDBREAKPROCW =
                core::mem::transmute((*es).word_break_proc);
            ret = (wbpW.unwrap())((*es).text.offset(start as isize), index, count, action);
        } else {
            let wbpA: EDITWORDBREAKPROCA =
                core::mem::transmute((*es).word_break_proc);
            let countA: INT;
            let textA: *mut CHAR;

            countA = WideCharToMultiByte(CP_ACP, 0, (*es).text.offset(start as isize), count, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
            textA = HeapAlloc(GetProcessHeap(), 0, countA as SIZE_T) as *mut CHAR;
            if textA.is_null() {
                return 0;
            }
            WideCharToMultiByte(CP_ACP, 0, (*es).text.offset(start as isize), count, textA, countA, core::ptr::null_mut(), core::ptr::null_mut());
            ret = (wbpA.unwrap())(textA, index, countA, action);
            HeapFree(GetProcessHeap(), 0, textA as *mut _);
        }
    } else {
        ret = EDIT_WordBreakProc(es, (*es).text, index + start, count + start, action) - start;
    }

    ret
}

unsafe fn EDIT_InvalidateUniscribeData_linedef(line_def: *mut LINEDEF) {
    if !(*line_def).ssa.is_null() {
        ScriptStringFree(core::ptr::addr_of_mut!((*line_def).ssa));
        (*line_def).ssa = core::ptr::null_mut();
    }
}

unsafe fn EDIT_InvalidateUniscribeData(es: *mut EDITSTATE) {
    let mut line_def: *mut LINEDEF = (*es).first_line_def;
    while !line_def.is_null() {
        EDIT_InvalidateUniscribeData_linedef(line_def);
        line_def = (*line_def).next;
    }
    if !(*es).ssa.is_null() {
        ScriptStringFree(core::ptr::addr_of_mut!((*es).ssa));
        (*es).ssa = core::ptr::null_mut();
    }
}

unsafe fn EDIT_UpdateUniscribeData_linedef(es: *mut EDITSTATE, dc: HDC, line_def: *mut LINEDEF) -> SCRIPT_STRING_ANALYSIS {
    if line_def.is_null() {
        return core::ptr::null_mut();
    }

    if (*line_def).net_length != 0 && (*line_def).ssa.is_null() {
        let index = (*line_def).index;
        let mut old_font: HFONT = core::ptr::null_mut();
        let mut udc: HDC = dc;
        let mut tabdef: SCRIPT_TABDEF = core::mem::zeroed();
        let hr: HRESULT;

        if udc.is_null() {
            udc = GetDC((*es).hwndSelf);
        }
        if !(*es).font.is_null() {
            old_font = SelectObject(udc, (*es).font as *mut _) as HFONT;
        }

        tabdef.cTabStops = (*es).tabs_count;
        tabdef.iScale = 0;
        tabdef.pTabStops = (*es).tabs;
        tabdef.iTabOrigin = 0;

        hr = ScriptStringAnalyse(
            udc,
            (*es).text.offset(index as isize) as *const c_void,
            (*line_def).net_length,
            3 * (*line_def).net_length / 2 + 16,
            -1,
            SSA_LINK | SSA_FALLBACK | SSA_GLYPHS | SSA_TAB,
            -1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(tabdef),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*line_def).ssa),
        );
        if FAILED(hr) {
            (*line_def).ssa = core::ptr::null_mut();
        }

        if !(*es).font.is_null() {
            SelectObject(udc, old_font as *mut _);
        }
        if udc != dc {
            ReleaseDC((*es).hwndSelf, udc);
        }
    }

    (*line_def).ssa
}

unsafe fn EDIT_UpdateUniscribeData(es: *mut EDITSTATE, dc: HDC, mut line: INT) -> SCRIPT_STRING_ANALYSIS {
    let mut line_def: *mut LINEDEF;

    if (*es).style & ES_MULTILINE == 0 {
        if (*es).ssa.is_null() {
            let length: INT = get_text_length(es) as INT;
            let mut old_font: HFONT = core::ptr::null_mut();
            let mut udc: HDC = dc;

            if udc.is_null() {
                udc = GetDC((*es).hwndSelf);
            }
            if !(*es).font.is_null() {
                old_font = SelectObject(udc, (*es).font as *mut _) as HFONT;
            }

            if (*es).style & ES_PASSWORD != 0 {
                ScriptStringAnalyse(udc, core::ptr::addr_of_mut!((*es).password_char) as *const core::ffi::c_void, length, 3 * length / 2 + 16, -1,
                    (SSA_LINK | SSA_FALLBACK | SSA_GLYPHS | SSA_PASSWORD) as DWORD, -1,
                    core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*es).ssa));
            } else {
                ScriptStringAnalyse(udc, (*es).text as *const core::ffi::c_void, length, 3 * length / 2 + 16, -1,
                    (SSA_LINK | SSA_FALLBACK | SSA_GLYPHS) as DWORD, -1,
                    core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*es).ssa));
            }

            if !(*es).font.is_null() {
                SelectObject(udc, old_font as *mut _);
            }
            if udc != dc {
                ReleaseDC((*es).hwndSelf, udc);
            }
        }
        return (*es).ssa;
    } else {
        line_def = (*es).first_line_def;
        while !line_def.is_null() && line != 0 {
            line_def = (*line_def).next;
            line -= 1;
        }

        return EDIT_UpdateUniscribeData_linedef(es, dc, line_def);
    }
}

unsafe fn get_vertical_line_count(es: *mut EDITSTATE) -> INT {
    let vlc: INT = ((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
    core::cmp::max(1, vlc)
}

unsafe fn EDIT_BuildLineDefs_ML(es: *mut EDITSTATE, mut istart: INT, iend: INT, delta: INT, hrgn: HRGN) {
    let mut current_position: LPWSTR;
    let mut cp: LPWSTR;
    let fw: INT;
    let mut current_line: *mut LINEDEF;
    let mut previous_line: *mut LINEDEF;
    let start_line: *mut LINEDEF;
    let mut line_index: INT = 0;
    let mut nstart_line: INT;
    let mut nstart_index: INT;
    let line_count: INT = (*es).line_count;
    let mut orig_net_length: INT;
    let mut rc: RECT = core::mem::zeroed();
    let vlc: INT;

    if istart == iend && delta == 0 {
        return;
    }

    previous_line = core::ptr::null_mut();
    current_line = (*es).first_line_def;

    /* Find starting line. */
    loop {
        if istart < (*current_line).index + (*current_line).length
            || (*current_line).ending == END_0
        {
            break;
        }

        previous_line = current_line;
        current_line = (*current_line).next;
        line_index += 1;

        if current_line.is_null() {
            break;
        }
    }

    if current_line.is_null() {
        FIXME!(" modification occurred outside buffer\n");
        return;
    }

    nstart_line = line_index;
    nstart_index = (*current_line).index;

    if ((*es).style & ES_AUTOHSCROLL) == 0 && line_index > 0 {
        line_index -= 1;
        current_line = previous_line;
    }
    start_line = current_line;

    fw = (*es).format_rect.right - (*es).format_rect.left;
    current_position = (*es).text.offset((*current_line).index as isize);
    vlc = get_vertical_line_count(es);

    loop {
        if current_line != start_line {
            if current_line.is_null()
                || (*current_line).index + delta
                    > current_position.offset_from((*es).text) as INT
            {
                let new_line = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    core::mem::size_of::<LINEDEF>() as SIZE_T,
                ) as *mut LINEDEF;
                if new_line.is_null() {
                    break;
                }
                (*new_line).next = (*previous_line).next;
                (*previous_line).next = new_line;
                current_line = new_line;
                (*es).line_count += 1;
            } else if (*current_line).index + delta
                < current_position.offset_from((*es).text) as INT
            {
                (*previous_line).next = (*current_line).next;
                HeapFree(GetProcessHeap(), 0, current_line as *mut core::ffi::c_void);
                current_line = (*previous_line).next;
                (*es).line_count -= 1;
                continue;
            } else {
                if current_position.offset_from((*es).text) as INT > iend {
                    break;
                }
            }
        }

        (*current_line).index = current_position.offset_from((*es).text) as INT;
        orig_net_length = (*current_line).net_length;

        cp = current_position;
        while *cp != 0 {
            if *cp == '\n' as u16 {
                break;
            }
            if *cp == '\r' as u16 && *cp.offset(1) == '\n' as u16 {
                break;
            }
            cp = cp.offset(1);
        }

        if *cp == 0 {
            (*current_line).ending = END_0;
            (*current_line).net_length = strlenW(current_position) as INT;
        } else if cp > current_position && *cp.offset(-1) == '\r' as u16 {
            (*current_line).ending = END_SOFT;
            (*current_line).net_length = cp.offset_from(current_position) as INT - 1;
        } else if *cp == '\n' as u16 {
            (*current_line).ending = END_RICH;
            (*current_line).net_length = cp.offset_from(current_position) as INT;
        } else {
            (*current_line).ending = END_HARD;
            (*current_line).net_length = cp.offset_from(current_position) as INT;
        }

        if (*current_line).net_length != 0 {
            let sz: *const SIZE;
            EDIT_InvalidateUniscribeData_linedef(current_line);
            EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut(), current_line);
            if !(*current_line).ssa.is_null() {
                sz = ScriptString_pSize((*current_line).ssa);
                (*current_line).width = (*sz).cx;
            } else {
                (*current_line).width = (*es).char_width * (*current_line).net_length;
            }
        } else {
            (*current_line).width = 0;
        }

        if ((*es).style & ES_AUTOHSCROLL) == 0 {
            if (*current_line).width > fw && fw > (*es).char_width {
                let mut prev: INT;
                let mut next: INT;
                let w: INT;
                let mut sz: *const SIZE;
                let mut d: f32;

                prev = (*current_line).net_length - 1;
                w = (*current_line).net_length;
                d = (*current_line).width as f32 / fw as f32;
                if d > 1.2f32 {
                    d -= 0.2f32;
                }
                next = (prev as f32 / d) as INT;
                if next >= prev {
                    next = prev - 1;
                }
                loop {
                    prev = EDIT_CallWordBreakProc(
                        es,
                        current_position.offset_from((*es).text) as INT,
                        next,
                        (*current_line).net_length,
                        WB_LEFT as INT,
                    );
                    (*current_line).net_length = prev;
                    EDIT_InvalidateUniscribeData_linedef(current_line);
                    EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut(), current_line);
                    if !(*current_line).ssa.is_null() {
                        sz = ScriptString_pSize((*current_line).ssa);
                    } else {
                        sz = core::ptr::null();
                    }
                    if !sz.is_null() {
                        (*current_line).width = (*sz).cx;
                    } else {
                        prev = 0;
                    }
                    next = prev - 1;
                    if !(prev != 0 && (*current_line).width > fw) {
                        break;
                    }
                }
                (*current_line).net_length = w;

                if prev == 0 {
                    let piDx: *mut INT;
                    let count: *const INT;

                    EDIT_InvalidateUniscribeData_linedef(current_line);
                    EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut(), current_line);

                    if !(*current_line).ssa.is_null() {
                        count = ScriptString_pcOutChars((*current_line).ssa);
                        piDx = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (core::mem::size_of::<INT>() * (*count) as usize) as SIZE_T,
                        ) as *mut INT;
                        ScriptStringGetLogicalWidths((*current_line).ssa, piDx);

                        prev = (*current_line).net_length - 1;
                        loop {
                            (*current_line).width -= *piDx.offset(prev as isize);
                            prev -= 1;
                            if !(prev > 0 && (*current_line).width > fw) {
                                break;
                            }
                        }
                        if prev <= 0 {
                            prev = 1;
                        }
                        HeapFree(GetProcessHeap(), 0, piDx as *mut core::ffi::c_void);
                    } else {
                        prev = fw / (*es).char_width;
                    }
                }

                if (*current_line).index == nstart_index
                    && istart > (*current_line).index + prev
                {
                    istart = (*current_line).index + prev;
                } else if current_line == start_line
                    && (*current_line).index != nstart_index
                    && orig_net_length < prev
                {
                    nstart_line = line_index;
                    nstart_index = (*current_line).index;
                    istart = (*current_line).index + orig_net_length;
                }

                (*current_line).net_length = prev;
                (*current_line).ending = END_WRAP;

                if (*current_line).net_length > 0 {
                    EDIT_UpdateUniscribeData_linedef(es, core::ptr::null_mut(), current_line);
                    if !(*current_line).ssa.is_null() {
                        let sz2 = ScriptString_pSize((*current_line).ssa);
                        (*current_line).width = (*sz2).cx;
                    } else {
                        (*current_line).width = 0;
                    }
                } else {
                    (*current_line).width = 0;
                }
            } else if current_line == start_line
                && (*current_line).index != nstart_index
                && orig_net_length < (*current_line).net_length
            {
                nstart_line = line_index;
                nstart_index = (*current_line).index;
                istart = (*current_line).index + orig_net_length;
            }
        }

        match (*current_line).ending {
            END_SOFT => {
                (*current_line).length = (*current_line).net_length + 3;
            }
            END_RICH => {
                (*current_line).length = (*current_line).net_length + 1;
            }
            END_HARD => {
                (*current_line).length = (*current_line).net_length + 2;
            }
            END_WRAP | END_0 => {
                (*current_line).length = (*current_line).net_length;
            }
            _ => {}
        }
        (*es).text_width = max((*es).text_width, (*current_line).width);
        current_position = current_position.offset((*current_line).length as isize);
        previous_line = current_line;

        if (line_index < (*es).y_offset) || (line_index > (*es).y_offset + vlc) {
            EDIT_InvalidateUniscribeData_linedef(current_line);
        }

        current_line = (*current_line).next;
        line_index += 1;

        if (*previous_line).ending == END_0 {
            break;
        }
    }

    if (*previous_line).ending == END_0 {
        let mut pnext: *mut LINEDEF;

        (*previous_line).next = core::ptr::null_mut();
        while !current_line.is_null() {
            pnext = (*current_line).next;
            EDIT_InvalidateUniscribeData_linedef(current_line);
            HeapFree(GetProcessHeap(), 0, current_line as *mut core::ffi::c_void);
            current_line = pnext;
            (*es).line_count -= 1;
        }
    } else if delta != 0 {
        while !current_line.is_null() {
            (*current_line).index += delta;
            current_line = (*current_line).next;
        }
    }

    if !hrgn.is_null() {
        let tmphrgn: HRGN;
        rc.top = (*es).format_rect.top + nstart_line * (*es).line_height
            - ((*es).y_offset * (*es).line_height);
        rc.bottom = rc.top + (*es).line_height;
        if ((*es).style & ES_CENTER) != 0 || ((*es).style & ES_RIGHT) != 0 {
            rc.left = (*es).format_rect.left;
        } else {
            rc.left = LOWORD(EDIT_EM_PosFromChar(es, nstart_index, 0) as u32) as i16 as INT;
        }
        rc.right = (*es).format_rect.right;
        SetRectRgn(hrgn, rc.left, rc.top, rc.right, rc.bottom);

        rc.top = rc.bottom;
        rc.left = (*es).format_rect.left;
        rc.right = (*es).format_rect.right;

        if line_count < (*es).line_count {
            rc.bottom = (*es).line_count * (*es).line_height;
        } else if line_count > (*es).line_count {
            rc.bottom = line_count * (*es).line_height;
        } else {
            rc.bottom = line_index * (*es).line_height;
        }
        rc.bottom += (*es).format_rect.top;
        rc.bottom -= (*es).y_offset * (*es).line_height;
        tmphrgn = CreateRectRgn(rc.left, rc.top, rc.right, rc.bottom);
        CombineRgn(hrgn, hrgn, tmphrgn, RGN_OR as INT);
        DeleteObject(tmphrgn as *mut core::ffi::c_void);
    }
}

unsafe fn EDIT_CalcLineWidth_SL(es: *mut EDITSTATE) {
    EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), 0);
    if !(*es).ssa.is_null() {
        let size: *const SIZE = ScriptString_pSize((*es).ssa);
        (*es).text_width = (*size).cx;
    } else {
        (*es).text_width = 0;
    }
}

unsafe fn EDIT_CharFromPos(es: *mut EDITSTATE, mut x: INT, y: INT, after_wrap: LPBOOL) -> INT {
    let index: INT;

    if (*es).style & ES_MULTILINE != 0 {
        let mut trailing: INT = 0;
        let mut line: INT = (y - (*es).format_rect.top) / (*es).line_height + (*es).y_offset;
        let mut line_index: INT = 0;
        let mut line_def: *mut LINEDEF = (*es).first_line_def;
        EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), line);
        while line > 0 && !(*line_def).next.is_null() {
            line_index += (*line_def).length;
            line_def = (*line_def).next;
            line -= 1;
        }

        x += (*es).x_offset - (*es).format_rect.left;
        if (*es).style & ES_RIGHT != 0 {
            x -= ((*es).format_rect.right - (*es).format_rect.left) - (*line_def).width;
        } else if (*es).style & ES_CENTER != 0 {
            x -= (((*es).format_rect.right - (*es).format_rect.left) - (*line_def).width) / 2;
        }
        if x >= (*line_def).width {
            if !after_wrap.is_null() {
                *after_wrap = ((*line_def).ending == END_WRAP) as BOOL;
            }
            return line_index + (*line_def).net_length;
        }
        if x <= 0 || (*line_def).ssa.is_null() {
            if !after_wrap.is_null() {
                *after_wrap = 0;
            }
            return line_index;
        }

        let mut idx: INT = 0;
        ScriptStringXtoCP((*line_def).ssa, x, &mut idx, &mut trailing);
        if trailing != 0 {
            idx += 1;
        }
        idx += line_index;
        index = idx;
        if !after_wrap.is_null() {
            *after_wrap = ((index == line_index + (*line_def).net_length)
                && ((*line_def).ending == END_WRAP)) as BOOL;
        }
    } else {
        let mut xoff: INT = 0;
        let mut trailing: INT = 0;
        if !after_wrap.is_null() {
            *after_wrap = 0;
        }
        x -= (*es).format_rect.left;
        if x == 0 {
            return (*es).x_offset;
        }

        if (*es).x_offset == 0 {
            let indent: INT = ((*es).format_rect.right - (*es).format_rect.left) - (*es).text_width;
            if (*es).style & ES_RIGHT != 0 {
                x -= indent;
            } else if (*es).style & ES_CENTER != 0 {
                x -= indent / 2;
            }
        }

        EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), 0);
        if (*es).x_offset != 0 {
            if !(*es).ssa.is_null() {
                if (*es).x_offset as UINT >= get_text_length(es) {
                    let size: *const SIZE = ScriptString_pSize((*es).ssa);
                    xoff = (*size).cx;
                }
                ScriptStringCPtoX((*es).ssa, (*es).x_offset, 0, &mut xoff);
            } else {
                xoff = 0;
            }
        }
        let mut idx: INT;
        if x < 0 {
            if x + xoff > 0 || (*es).ssa.is_null() {
                idx = 0;
                ScriptStringXtoCP((*es).ssa, x + xoff, &mut idx, &mut trailing);
                if trailing != 0 {
                    idx += 1;
                }
            } else {
                idx = 0;
            }
        } else {
            if x != 0 {
                let mut size: *const SIZE = core::ptr::null();
                if !(*es).ssa.is_null() {
                    size = ScriptString_pSize((*es).ssa);
                }
                if size.is_null() {
                    idx = 0;
                } else if x > (*size).cx {
                    idx = get_text_length(es) as INT;
                } else if !(*es).ssa.is_null() {
                    idx = 0;
                    ScriptStringXtoCP((*es).ssa, x + xoff, &mut idx, &mut trailing);
                    if trailing != 0 {
                        idx += 1;
                    }
                } else {
                    idx = 0;
                }
            } else {
                idx = (*es).x_offset;
            }
        }
        index = idx;
    }
    index
}

unsafe fn EDIT_ConfinePoint(es: *const EDITSTATE, x: LPINT, y: LPINT) {
    *x = core::cmp::min(core::cmp::max(*x, (*es).format_rect.left), (*es).format_rect.right - 1);
    *y = core::cmp::min(core::cmp::max(*y, (*es).format_rect.top), (*es).format_rect.bottom - 1);
}

unsafe fn EDIT_EM_LineFromChar(es: *mut EDITSTATE, mut index: INT) -> INT {
	let mut line: INT;
	let mut line_def: *mut LINEDEF;

	if ((*es).style & ES_MULTILINE) == 0 {
		return 0;
	}
	if index > get_text_length(es) as INT {
		return (*es).line_count - 1;
	}
	if index == -1 {
		index = core::cmp::min((*es).selection_start, (*es).selection_end);
	}

	line = 0;
	line_def = (*es).first_line_def;
	index -= (*line_def).length;
	while index >= 0 && !(*line_def).next.is_null() {
		line += 1;
		line_def = (*line_def).next;
		index -= (*line_def).length;
	}
	line
}

unsafe fn EDIT_EM_LineIndex(es: *const EDITSTATE, mut line: INT) -> INT {
    let mut line_index: INT;
    let mut line_def: *const LINEDEF;

    if ((*es).style & ES_MULTILINE) == 0 {
        return 0;
    }
    if line >= (*es).line_count {
        return -1;
    }

    line_index = 0;
    line_def = (*es).first_line_def;
    if line == -1 {
        let mut index: INT = (*es).selection_end - (*line_def).length;
        while index >= 0 && !(*line_def).next.is_null() {
            line_index += (*line_def).length;
            line_def = (*line_def).next;
            index -= (*line_def).length;
        }
    } else {
        while line > 0 {
            line_index += (*line_def).length;
            line_def = (*line_def).next;
            line -= 1;
        }
    }
    line_index
}

unsafe fn EDIT_EM_LineLength(es: *mut EDITSTATE, mut index: INT) -> INT {
    let mut line_def: *mut LINEDEF;

    if (*es).style & ES_MULTILINE == 0 {
        return get_text_length(es) as INT;
    }

    if index == -1 {
        let mut l: INT32;
        let li: INT32;
        let mut count: INT32;
        l = EDIT_EM_LineFromChar(es, (*es).selection_start);
        count = (*es).selection_start - EDIT_EM_LineIndex(es, l);
        l = EDIT_EM_LineFromChar(es, (*es).selection_end);
        li = EDIT_EM_LineIndex(es, l);
        count += li + EDIT_EM_LineLength(es, li) - (*es).selection_end;
        return count;
    }
    line_def = (*es).first_line_def;
    index -= (*line_def).length;
    while index >= 0 && !(*line_def).next.is_null() {
        line_def = (*line_def).next;
        index -= (*line_def).length;
    }
    (*line_def).net_length
}

unsafe fn EDIT_EM_PosFromChar(es: *mut EDITSTATE, mut index: INT, after_wrap: BOOL) -> LRESULT {
    let len: INT = get_text_length(es) as INT;
    let mut l: INT;
    let mut li: INT;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut w: INT;
    let mut lw: INT;
    let mut line_def: *mut LINEDEF;

    index = min(index, len);
    if ((*es).style & ES_MULTILINE) != 0 {
        l = EDIT_EM_LineFromChar(es, index);
        EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), l);

        y = (l - (*es).y_offset) * (*es).line_height;
        li = EDIT_EM_LineIndex(es, l);
        if after_wrap != 0 && (li == index) && l != 0 {
            let mut l2: INT = l - 1;
            line_def = (*es).first_line_def;
            while l2 != 0 {
                line_def = (*line_def).next;
                l2 -= 1;
            }
            if (*line_def).ending == END_WRAP {
                l -= 1;
                y -= (*es).line_height;
                li = EDIT_EM_LineIndex(es, l);
            }
        }

        line_def = (*es).first_line_def;
        while (*line_def).index != li {
            line_def = (*line_def).next;
        }

        lw = (*line_def).width;
        w = (*es).format_rect.right - (*es).format_rect.left;
        if !(*line_def).ssa.is_null() {
            ScriptStringCPtoX((*line_def).ssa, (index - 1) - li, 1, &mut x);
            x -= (*es).x_offset;
        } else {
            x = if lw > 0 { (*es).x_offset } else { x - (*es).x_offset };
        }

        if ((*es).style & ES_RIGHT) != 0 {
            x = w - (lw - x);
        } else if ((*es).style & ES_CENTER) != 0 {
            x += (w - lw) / 2;
        }
    } else {
        let mut xoff: INT = 0;
        let mut xi: INT = 0;
        EDIT_UpdateUniscribeData(es, core::ptr::null_mut(), 0);
        if (*es).x_offset != 0 {
            if !(*es).ssa.is_null() {
                if (*es).x_offset >= get_text_length(es) as INT {
                    let leftover: INT = (*es).x_offset - get_text_length(es) as INT;
                    if !(*es).ssa.is_null() {
                        let size: *const SIZE = ScriptString_pSize((*es).ssa);
                        xoff = (*size).cx;
                    } else {
                        xoff = 0;
                    }
                    xoff += (*es).char_width * leftover;
                } else {
                    ScriptStringCPtoX((*es).ssa, (*es).x_offset, 0, &mut xoff);
                }
            } else {
                xoff = 0;
            }
        }
        if index != 0 {
            if index >= get_text_length(es) as INT {
                if !(*es).ssa.is_null() {
                    let size: *const SIZE = ScriptString_pSize((*es).ssa);
                    xi = (*size).cx;
                } else {
                    xi = 0;
                }
            } else if !(*es).ssa.is_null() {
                ScriptStringCPtoX((*es).ssa, index, 0, &mut xi);
            } else {
                xi = 0;
            }
        }
        x = xi - xoff;

        if index >= (*es).x_offset {
            if (*es).x_offset == 0 && ((*es).style & (ES_RIGHT | ES_CENTER)) != 0 {
                w = (*es).format_rect.right - (*es).format_rect.left;
                if w > (*es).text_width {
                    if ((*es).style & ES_RIGHT) != 0 {
                        x += w - (*es).text_width;
                    } else if ((*es).style & ES_CENTER) != 0 {
                        x += (w - (*es).text_width) / 2;
                    }
                }
            }
        }
        y = 0;
    }
    x += (*es).format_rect.left;
    y += (*es).format_rect.top;
    MAKELONG(x as INT16 as u16, y as INT16 as u16) as LRESULT
}

unsafe fn EDIT_GetLineRect(es: *mut EDITSTATE, mut line: INT, scol: INT, ecol: INT, rc: LPRECT) {
    let ssa: SCRIPT_STRING_ANALYSIS;
    let mut line_index: INT = 0;
    let pt1: INT;
    let pt2: INT;
    let pt3: INT;

    if (*es).style & ES_MULTILINE != 0 {
        let mut line_def: *const LINEDEF;
        (*rc).top = (*es).format_rect.top + (line - (*es).y_offset) * (*es).line_height;
        if line >= (*es).line_count {
            return;
        }

        line_def = (*es).first_line_def;
        if line == -1 {
            let mut index: INT = (*es).selection_end - (*line_def).length;
            while index >= 0 && !(*line_def).next.is_null() {
                line_index += (*line_def).length;
                line_def = (*line_def).next;
                index -= (*line_def).length;
            }
        } else {
            while line > 0 {
                line_index += (*line_def).length;
                line_def = (*line_def).next;
                line -= 1;
            }
        }
        ssa = (*line_def).ssa;
    } else {
        line_index = 0;
        (*rc).top = (*es).format_rect.top;
        ssa = (*es).ssa;
    }

    (*rc).bottom = (*rc).top + (*es).line_height;
    pt1 = if scol == 0 {
        (*es).format_rect.left
    } else {
        LOWORD(EDIT_EM_PosFromChar(es, line_index + scol, 1) as DWORD) as i16 as INT
    };
    pt2 = if ecol == -1 {
        (*es).format_rect.right
    } else {
        LOWORD(EDIT_EM_PosFromChar(es, line_index + ecol, 1) as DWORD) as i16 as INT
    };
    if !ssa.is_null() {
        let mut pt3_v: INT = 0;
        ScriptStringCPtoX(ssa, scol, 0, &mut pt3_v);
        pt3 = pt3_v + (*es).format_rect.left;
    } else {
        pt3 = pt1;
    }
    (*rc).right = max(max(pt1, pt2), pt3);
    (*rc).left = min(min(pt1, pt2), pt3);
}

unsafe fn text_buffer_changed(es: *mut EDITSTATE) {
    (*es).text_length = (-1i32) as UINT;

    HeapFree(GetProcessHeap(), 0, (*es).logAttr as *mut core::ffi::c_void);
    (*es).logAttr = core::ptr::null_mut();
    EDIT_InvalidateUniscribeData(es);
}

/// `EDIT_LockBuffer` (edit.c, `static`). ReactOS `__REACTOS__` path only.
unsafe fn EDIT_LockBuffer(es: *mut EDITSTATE) {
    if (*es).text.is_null() {
        // ReactOS Hacked! r45670
        let mut textA: *mut CHAR = core::ptr::null_mut();

        if !(*es).hloc32W.is_null() {
            if !(*es).hloc32A.is_null() {
                TRACE!();
                textA = LocalLock((*es).hloc32A) as *mut CHAR;
            }
        } else {
            ERR!("no buffer ... please report\n");
            return;
        }

        if !textA.is_null() {
            let hloc32W_new: HLOCAL;
            let countW_new: UINT = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0) as UINT;
            if countW_new > (*es).buffer_size + 1 {
                let alloc_size: UINT = ROUND_TO_GROW(countW_new * core::mem::size_of::<WCHAR>() as UINT);
                hloc32W_new = LocalReAlloc((*es).hloc32W, alloc_size as _, (LMEM_MOVEABLE | LMEM_ZEROINIT) as _);
                if !hloc32W_new.is_null() {
                    (*es).hloc32W = hloc32W_new;
                    (*es).buffer_size = (LocalSize(hloc32W_new) as UINT) / core::mem::size_of::<WCHAR>() as UINT - 1;
                } else {
                    WARN!();
                }
            }
            (*es).text = LocalLock((*es).hloc32W) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, textA, -1, (*es).text, ((*es).buffer_size + 1) as INT);
            LocalUnlock((*es).hloc32A);
        } else {
            (*es).text = LocalLock((*es).hloc32W) as LPWSTR;
        }

        /* Every caller assumes es->text is valid after a Lock (that is the whole
         * contract), and dereferences it without checking — EDIT_EM_ReplaceSel
         * writes through it immediately. If the lock failed, say so HERE rather
         * than letting it surface as an AV at address 0 three frames away. */
        if (*es).text.is_null() {
            DbgPrint(
                b"[EDIT] LockBuffer: LocalLock(hloc32W=%p) returned NULL (buffer_size=%u) \
                  - the edit buffer is unusable\n\0"
                    .as_ptr() as *const i8,
                (*es).hloc32W,
                (*es).buffer_size,
            );
        }
    }
    (*es).lock_count += 1;
}

unsafe fn EDIT_UnlockBuffer(es: *mut EDITSTATE, force: BOOL) {
	if IsWindow((*es).hwndSelf) == 0 {
		return;
	}

	if (*es).lock_count == 0 {
		return;
	}
	if (*es).text.is_null() {
		return;
	}

	if force != 0 || ((*es).lock_count == 1) {
		if !(*es).hloc32W.is_null() {
			let mut countA: UINT = 0;
			let countW: UINT = get_text_length(es) + 1;

			if !(*es).hloc32A.is_null() {
				let countA_new: UINT = WideCharToMultiByte(CP_ACP, 0, (*es).text, countW as i32, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
				countA = LocalSize((*es).hloc32A) as UINT;
				if countA_new > countA {
					let alloc_size: UINT = ROUND_TO_GROW(countA_new);
					let hloc32A_new = LocalReAlloc((*es).hloc32A, alloc_size as _, (LMEM_MOVEABLE | LMEM_ZEROINIT) as _);
					if !hloc32A_new.is_null() {
						(*es).hloc32A = hloc32A_new;
						countA = LocalSize(hloc32A_new) as UINT;
					}
				}
				WideCharToMultiByte(CP_ACP, 0, (*es).text, countW as i32,
					LocalLock((*es).hloc32A) as _, countA as i32, core::ptr::null_mut(), core::ptr::null_mut());
				LocalUnlock((*es).hloc32A);
			}

			LocalUnlock((*es).hloc32W);
			(*es).text = core::ptr::null_mut();
		} else {
			return;
		}
	}
	(*es).lock_count -= 1;
}

unsafe fn EDIT_MakeFit(es: *mut EDITSTATE, size: UINT) -> BOOL {
    let hNew32W: HLOCAL;

    if size <= (*es).buffer_size {
        return 1;
    }

    EDIT_UnlockBuffer(es, 1);

    if !(*es).hloc32W.is_null() {
        let alloc_size = ROUND_TO_GROW((size + 1) * core::mem::size_of::<WCHAR>() as UINT);
        hNew32W = LocalReAlloc((*es).hloc32W, alloc_size as _, (LMEM_MOVEABLE | LMEM_ZEROINIT) as _);
        if !hNew32W.is_null() {
            (*es).hloc32W = hNew32W;
            (*es).buffer_size = (LocalSize(hNew32W) as UINT) / core::mem::size_of::<WCHAR>() as UINT - 1;
        }
    }

    EDIT_LockBuffer(es);

    if (*es).buffer_size < size {
        notify_parent(es, EN_ERRSPACE as INT);
        0
    } else {
        1
    }
}

unsafe fn EDIT_MakeUndoFit(es: *mut EDITSTATE, size: UINT) -> BOOL {
    let alloc_size: UINT;

    if size <= (*es).undo_buffer_size {
        return 1;
    }

    alloc_size = ROUND_TO_GROW((size + 1) * core::mem::size_of::<WCHAR>() as UINT);
    (*es).undo_text = HeapReAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*es).undo_text as *mut core::ffi::c_void,
        alloc_size as SIZE_T,
    ) as LPWSTR;
    if !(*es).undo_text.is_null() {
        (*es).undo_buffer_size = alloc_size / core::mem::size_of::<WCHAR>() as UINT - 1;
        1
    } else {
        0
    }
}

unsafe fn EDIT_UpdateTextRegion(es: *mut EDITSTATE, hrgn: HRGN, bErase: BOOL) {
    if (*es).flags as u32 & EF_UPDATE as u32 != 0 {
        (*es).flags &= !(EF_UPDATE as u16);
        if notify_parent(es, EN_UPDATE as INT) == 0 {
            return;
        }
    }
    InvalidateRgn((*es).hwndSelf, hrgn, bErase);
}

unsafe fn EDIT_UpdateText(es: *mut EDITSTATE, rc: *const RECT, bErase: BOOL) {
    if (*es).flags as u32 & EF_UPDATE as u32 != 0 {
        (*es).flags &= !(EF_UPDATE as u16);
        if notify_parent(es, EN_UPDATE as INT) == 0 {
            return;
        }
    }
    InvalidateRect((*es).hwndSelf, rc, bErase);
}

unsafe fn EDIT_SL_InvalidateText(es: *mut EDITSTATE, start: INT, end: INT) {
    let mut line_rect: RECT = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();

    EDIT_GetLineRect(es, 0, start, end, &mut line_rect);
    if IntersectRect(&mut rc, &line_rect, core::ptr::addr_of!((*es).format_rect)) != 0 {
        EDIT_UpdateText(es, &rc, 1);
    }
}

unsafe fn EDIT_ML_InvalidateText(es: *mut EDITSTATE, start: INT, end: INT) {
    let vlc: INT = get_vertical_line_count(es);
    let mut sl: INT = EDIT_EM_LineFromChar(es, start);
    let mut el: INT = EDIT_EM_LineFromChar(es, end);
    let mut sc: INT;
    let mut ec: INT;
    let mut rc1: RECT = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();
    let mut rcLine: RECT = core::mem::zeroed();
    let mut rcUpdate: RECT = core::mem::zeroed();
    let mut l: INT;

    if (el < (*es).y_offset) || (sl > (*es).y_offset + vlc) {
        return;
    }

    sc = start - EDIT_EM_LineIndex(es, sl);
    ec = end - EDIT_EM_LineIndex(es, el);
    if sl < (*es).y_offset {
        sl = (*es).y_offset;
        sc = 0;
    }
    if el > (*es).y_offset + vlc {
        el = (*es).y_offset + vlc;
        ec = EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, el));
    }
    GetClientRect((*es).hwndSelf, &mut rc1);
    IntersectRect(&mut rcWnd, &rc1, core::ptr::addr_of!((*es).format_rect));
    if sl == el {
        EDIT_GetLineRect(es, sl, sc, ec, &mut rcLine);
        if IntersectRect(&mut rcUpdate, &rcWnd, &rcLine) != 0 {
            EDIT_UpdateText(es, &rcUpdate, 1);
        }
    } else {
        EDIT_GetLineRect(es, sl, sc,
            EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, sl)),
            &mut rcLine);
        if IntersectRect(&mut rcUpdate, &rcWnd, &rcLine) != 0 {
            EDIT_UpdateText(es, &rcUpdate, 1);
        }
        l = sl + 1;
        while l < el {
            EDIT_GetLineRect(es, l, 0,
                EDIT_EM_LineLength(es, EDIT_EM_LineIndex(es, l)),
                &mut rcLine);
            if IntersectRect(&mut rcUpdate, &rcWnd, &rcLine) != 0 {
                EDIT_UpdateText(es, &rcUpdate, 1);
            }
            l += 1;
        }
        EDIT_GetLineRect(es, el, 0, ec, &mut rcLine);
        if IntersectRect(&mut rcUpdate, &rcWnd, &rcLine) != 0 {
            EDIT_UpdateText(es, &rcUpdate, 1);
        }
    }
}

unsafe fn EDIT_InvalidateText(es: *mut EDITSTATE, mut start: INT, mut end: INT) {
    if end == start {
        return;
    }

    if end == -1 {
        end = get_text_length(es) as INT;
    }

    if end < start {
        let tmp = start;
        start = end;
        end = tmp;
    }

    if (*es).style & ES_MULTILINE != 0 {
        EDIT_ML_InvalidateText(es, start, end);
    } else {
        EDIT_SL_InvalidateText(es, start, end);
    }
}

unsafe fn EDIT_EM_SetSel(es: *mut EDITSTATE, mut start: UINT, mut end: UINT, after_wrap: BOOL) {
	let mut old_start: UINT = (*es).selection_start as UINT;
	let mut old_end: UINT = (*es).selection_end as UINT;
	let len: UINT = get_text_length(es);

	if start == (-1i32) as UINT {
		start = (*es).selection_end as UINT;
		end = (*es).selection_end as UINT;
	} else {
		start = core::cmp::min(start, len);
		end = core::cmp::min(end, len);
	}
	(*es).selection_start = start as INT;
	(*es).selection_end = end as INT;
	if after_wrap != 0 {
		(*es).flags |= EF_AFTER_WRAP as WORD;
	} else {
		(*es).flags &= !(EF_AFTER_WRAP as WORD);
	}

	if end < old_end {
		core::mem::swap(&mut end, &mut old_end);
	}
	if start < old_start {
		core::mem::swap(&mut start, &mut old_start);
	}
	if old_end < old_start {
		core::mem::swap(&mut old_start, &mut old_end);
	}
	if end < start {
		core::mem::swap(&mut start, &mut end);
	}

	if end != old_start {
		if old_start > end {
			EDIT_InvalidateText(es, start as INT, end as INT);
			EDIT_InvalidateText(es, old_start as INT, old_end as INT);
		} else {
			EDIT_InvalidateText(es, start as INT, old_start as INT);
			EDIT_InvalidateText(es, end as INT, old_end as INT);
		}
	} else {
		EDIT_InvalidateText(es, start as INT, old_end as INT);
	}
}

unsafe fn EDIT_UpdateScrollInfo(es: *mut EDITSTATE) {
    if ((*es).style & WS_VSCROLL != 0) && ((*es).flags as u32 & EF_VSCROLL_TRACK as u32 == 0) {
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_PAGE | SIF_POS | SIF_RANGE | SIF_DISABLENOSCROLL;
        si.nMin = 0;
        si.nMax = (*es).line_count - 1;
        si.nPage = (((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height) as UINT;
        si.nPos = (*es).y_offset;
        SetScrollInfo((*es).hwndSelf, SB_VERT as i32, &mut si, 1);
    }

    if ((*es).style & WS_HSCROLL != 0) && ((*es).flags as u32 & EF_HSCROLL_TRACK as u32 == 0) {
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_PAGE | SIF_POS | SIF_RANGE | SIF_DISABLENOSCROLL;
        si.nMin = 0;
        si.nMax = (*es).text_width - 1;
        si.nPage = ((*es).format_rect.right - (*es).format_rect.left) as UINT;
        si.nPos = (*es).x_offset;
        SetScrollInfo((*es).hwndSelf, SB_HORZ as i32, &mut si, 1);
    }
}

unsafe fn EDIT_EM_LineScroll_internal(es: *mut EDITSTATE, mut dx: INT, mut dy: INT) -> BOOL {
    let nyoff: INT;
    let x_offset_in_pixels: INT;
    let lines_per_page: INT = ((*es).format_rect.bottom - (*es).format_rect.top) /
        (*es).line_height;

    if (*es).style & ES_MULTILINE != 0 {
        x_offset_in_pixels = (*es).x_offset;
    } else {
        dy = 0;
        x_offset_in_pixels = (LOWORD(EDIT_EM_PosFromChar(es, (*es).x_offset, FALSE as BOOL) as DWORD)) as i16 as INT;
    }

    if -dx > x_offset_in_pixels {
        dx = -x_offset_in_pixels;
    }
    if dx > (*es).text_width - x_offset_in_pixels {
        dx = (*es).text_width - x_offset_in_pixels;
    }
    nyoff = max(0, (*es).y_offset + dy);
    let mut nyoff = nyoff;
    if nyoff >= (*es).line_count - lines_per_page {
        nyoff = max(0, (*es).line_count - lines_per_page);
    }
    dy = ((*es).y_offset - nyoff) * (*es).line_height;
    if dx != 0 || dy != 0 {
        let mut rc1: RECT = core::mem::zeroed();
        let mut rc: RECT = core::mem::zeroed();

        (*es).y_offset = nyoff;
        if (*es).style & ES_MULTILINE != 0 {
            (*es).x_offset += dx;
        } else {
            (*es).x_offset += dx / (*es).char_width;
        }

        GetClientRect((*es).hwndSelf, &mut rc1);
        IntersectRect(&mut rc, &rc1, &(*es).format_rect);
        ScrollWindowEx((*es).hwndSelf, -dx, dy,
            core::ptr::null_mut(), &rc, core::ptr::null_mut(), core::ptr::null_mut(), SW_INVALIDATE as UINT);
        EDIT_UpdateScrollInfo(es);
    }
    if dx != 0 && (*es).flags & EF_HSCROLL_TRACK == 0 {
        notify_parent(es, EN_HSCROLL as INT);
    }
    if dy != 0 && (*es).flags & EF_VSCROLL_TRACK == 0 {
        notify_parent(es, EN_VSCROLL as INT);
    }
    TRUE as BOOL
}

unsafe fn EDIT_EM_LineScroll(es: *mut EDITSTATE, mut dx: INT, dy: INT) -> BOOL {
    if ((*es).style & ES_MULTILINE) == 0 {
        return 0u8 as BOOL;
    }

    dx *= (*es).char_width;
    EDIT_EM_LineScroll_internal(es, dx, dy)
}

unsafe fn EDIT_EM_Scroll(es: *mut EDITSTATE, action: INT) -> LRESULT {
    let mut dy: INT;

    if ((*es).style & ES_MULTILINE) == 0 {
        return 0 as LRESULT;
    }

    dy = 0;

    match action as u32 {
        SB_LINEUP => {
            if (*es).y_offset != 0 {
                dy = -1;
            }
        }
        SB_LINEDOWN => {
            if (*es).y_offset < (*es).line_count - 1 {
                dy = 1;
            }
        }
        SB_PAGEUP => {
            if (*es).y_offset != 0 {
                dy = -((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
            }
        }
        SB_PAGEDOWN => {
            if (*es).y_offset < (*es).line_count - 1 {
                dy = ((*es).format_rect.bottom - (*es).format_rect.top) / (*es).line_height;
            }
        }
        _ => {
            return 0 as LRESULT;
        }
    }

    if dy != 0 {
        let vlc: INT = get_vertical_line_count(es);
        if (*es).y_offset + dy > (*es).line_count - vlc {
            dy = max((*es).line_count - vlc, 0) - (*es).y_offset;
        }

        if dy != 0 {
            EDIT_EM_LineScroll(es, 0, dy);
            return MAKELONG(dy as u16, 1u16) as LRESULT;
        }
    }
    0 as LRESULT
}

unsafe fn EDIT_ImmSetCompositionWindow(es: *mut EDITSTATE, pt: POINT) {
    let mut CompForm: COMPOSITIONFORM = core::mem::zeroed();
    let hIMC = ImmGetContext((*es).hwndSelf);
    if hIMC.is_null() {
        ERR!("!hIMC\n");
        return;
    }

    CompForm.ptCurrentPos = core::mem::transmute_copy(&pt);
    if (*es).style & ES_MULTILINE != 0 {
        CompForm.dwStyle = CFS_RECT;
        CompForm.rcArea = core::mem::transmute_copy(&(*es).format_rect);
    } else {
        CompForm.dwStyle = CFS_POINT;
        SetRectEmpty(core::ptr::addr_of_mut!(CompForm.rcArea) as LPRECT);
    }

    ImmSetCompositionWindow(hIMC, core::ptr::addr_of_mut!(CompForm));
    ImmReleaseContext((*es).hwndSelf, hIMC);
}

unsafe fn EDIT_SetCaretPos(es: *mut EDITSTATE, pos: INT, after_wrap: BOOL) {
    let res = EDIT_EM_PosFromChar(es, pos, after_wrap);
    let hKL = GetKeyboardLayout(0);
    let pt = POINT {
        x: (LOWORD(res as u32) as i16) as i32,
        y: (HIWORD(res as u32) as i16) as i32,
    };

    if ((*es).flags as u32 & EF_FOCUSED as u32) == 0 {
        return;
    }

    SetCaretPos(pt.x, pt.y);

    if ImmIsIME(hKL) != 0 {
        EDIT_ImmSetCompositionWindow(es, pt);
    }
}

unsafe fn EDIT_EM_ScrollCaret(es: *mut EDITSTATE) {
    if (*es).style & ES_MULTILINE != 0 {
        let l: INT;
        let vlc: INT;
        let ww: INT;
        let cw: INT = (*es).char_width;
        let x: INT;
        let mut dy: INT = 0;
        let mut dx: INT = 0;

        l = EDIT_EM_LineFromChar(es, (*es).selection_end);
        x = LOWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, ((*es).flags & EF_AFTER_WRAP as u16) as BOOL) as DWORD) as i16 as INT;
        vlc = get_vertical_line_count(es);
        if l >= (*es).y_offset + vlc {
            dy = l - vlc + 1 - (*es).y_offset;
        }
        if l < (*es).y_offset {
            dy = l - (*es).y_offset;
        }
        ww = (*es).format_rect.right - (*es).format_rect.left;
        if x < (*es).format_rect.left {
            dx = x - (*es).format_rect.left - ww / HSCROLL_FRACTION / cw * cw;
        }
        if x > (*es).format_rect.right {
            dx = x - (*es).format_rect.left - (HSCROLL_FRACTION - 1) * ww / HSCROLL_FRACTION / cw * cw;
        }
        if dy != 0 || dx != 0 || ((*es).y_offset != 0 && ((*es).line_count - (*es).y_offset < vlc)) {
            if (*es).x_offset + dx + ww > (*es).text_width {
                dx = (*es).text_width - ww - (*es).x_offset;
            }
            if dx != 0 || dy != 0 || ((*es).y_offset != 0 && ((*es).line_count - (*es).y_offset < vlc)) {
                EDIT_EM_LineScroll_internal(es, dx, dy);
            }
        }
    } else {
        let mut x: INT;
        let goal: INT;
        let format_width: INT;

        x = LOWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, 0u8 as BOOL) as DWORD) as i16 as INT;
        format_width = (*es).format_rect.right - (*es).format_rect.left;
        if x < (*es).format_rect.left {
            goal = (*es).format_rect.left + format_width / HSCROLL_FRACTION;
            loop {
                (*es).x_offset -= 1;
                x = LOWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, 0u8 as BOOL) as DWORD) as i16 as INT;
                if !((x < goal) && (*es).x_offset != 0) {
                    break;
                }
            }
            EDIT_UpdateText(es, core::ptr::null_mut(), 1u8 as BOOL);
        } else if x > (*es).format_rect.right {
            let mut x_last: INT;
            let len: INT = get_text_length(es) as INT;
            goal = (*es).format_rect.right - format_width / HSCROLL_FRACTION;
            loop {
                (*es).x_offset += 1;
                x = LOWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, 0u8 as BOOL) as DWORD) as i16 as INT;
                x_last = LOWORD(EDIT_EM_PosFromChar(es, len, 0u8 as BOOL) as DWORD) as i16 as INT;
                if !((x > goal) && (x_last > (*es).format_rect.right)) {
                    break;
                }
            }
            EDIT_UpdateText(es, core::ptr::null_mut(), 1u8 as BOOL);
        }
    }

    if (*es).flags & EF_FOCUSED as u16 != 0 {
        EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags & EF_AFTER_WRAP as u16) as BOOL);
    }
}

unsafe fn EDIT_MoveBackward(es: *mut EDITSTATE, extend: BOOL) {
	let mut e: INT = (*es).selection_end;

	if e != 0 {
		e -= 1;
		if ((*es).style & ES_MULTILINE as u32) != 0 && e != 0 &&
				*(*es).text.offset((e - 1) as isize) == b'\r' as u16 &&
				*(*es).text.offset(e as isize) == b'\n' as u16 {
			e -= 1;
			if e != 0 && *(*es).text.offset((e - 1) as isize) == b'\r' as u16 {
				e -= 1;
			}
		}
	}
	EDIT_EM_SetSel(es, if extend != 0 { (*es).selection_start as UINT } else { e as UINT }, e as UINT, 0);
	EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveDown_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL;
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x: INT = ((pos & 0xffff) as u16 as i16) as INT;
    let y: INT = (((pos >> 16) & 0xffff) as u16 as i16) as INT;

    e = EDIT_CharFromPos(es, x, y + (*es).line_height, &mut after_wrap);
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveEnd(es: *mut EDITSTATE, extend: BOOL, ctrl: BOOL) {
    let mut after_wrap: BOOL = 0u8 as BOOL;
    let e: INT;

    if ctrl == 0 && ((*es).style & ES_MULTILINE) != 0 {
        e = EDIT_CharFromPos(
            es,
            0x3fffffff,
            HIWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL) as DWORD) as INT,
            &mut after_wrap,
        );
    } else {
        e = get_text_length(es) as INT;
    }
    EDIT_EM_SetSel(es, if extend != 0 { (*es).selection_start as UINT } else { e as UINT }, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveForward(es: *mut EDITSTATE, extend: BOOL) {
	let mut e: INT = (*es).selection_end;

	if *(*es).text.offset(e as isize) != 0 {
		e += 1;
		if ((*es).style & ES_MULTILINE) != 0 && *(*es).text.offset((e - 1) as isize) == '\r' as u16 {
			if *(*es).text.offset(e as isize) == '\n' as u16 {
				e += 1;
			} else if *(*es).text.offset(e as isize) == '\r' as u16
				&& *(*es).text.offset((e + 1) as isize) == '\n' as u16
			{
				e += 2;
			}
		}
	}
	EDIT_EM_SetSel(es, if extend != 0 { (*es).selection_start as UINT } else { e as UINT }, e as UINT, 0);
	EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveHome(es: *mut EDITSTATE, extend: BOOL, ctrl: BOOL) {
    let e: INT;

    if ctrl == 0 && ((*es).style & ES_MULTILINE) != 0 {
        e = EDIT_CharFromPos(
            es,
            -(*es).x_offset,
            HIWORD(EDIT_EM_PosFromChar(es, (*es).selection_end, ((*es).flags as DWORD & EF_AFTER_WRAP as u32) as BOOL) as DWORD) as INT,
            core::ptr::null_mut(),
        );
    } else {
        e = 0;
    }
    EDIT_EM_SetSel(es, if extend != 0 { (*es).selection_start as UINT } else { e as UINT }, e as UINT, 0);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MovePageDown_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL;
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x: INT = (pos & 0xffff) as i16 as INT;
    let y: INT = ((pos >> 16) & 0xffff) as i16 as INT;

    e = EDIT_CharFromPos(es, x,
        y + ((*es).format_rect.bottom - (*es).format_rect.top),
        core::ptr::addr_of_mut!(after_wrap));
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MovePageUp_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s = (*es).selection_start;
    let mut e = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags & EF_AFTER_WRAP as u16) as BOOL;
    let pos = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x = (pos & 0xffff) as i16 as INT;
    let y = ((pos >> 16) & 0xffff) as i16 as INT;

    e = EDIT_CharFromPos(es, x,
        y - ((*es).format_rect.bottom - (*es).format_rect.top),
        &mut after_wrap);
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveUp_ML(es: *mut EDITSTATE, extend: BOOL) {
    let mut s: INT = (*es).selection_start;
    let mut e: INT = (*es).selection_end;
    let mut after_wrap: BOOL = ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL;
    let pos: LRESULT = EDIT_EM_PosFromChar(es, e, after_wrap);
    let x: INT = (pos & 0xffff) as i16 as INT;
    let y: INT = ((pos >> 16) & 0xffff) as i16 as INT;

    e = EDIT_CharFromPos(es, x, y - (*es).line_height, &mut after_wrap);
    if extend == 0 {
        s = e;
    }
    EDIT_EM_SetSel(es, s as UINT, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveWordBackward(es: *mut EDITSTATE, extend: BOOL) {
	let mut s: INT = (*es).selection_start;
	let mut e: INT = (*es).selection_end;
	let l: INT;
	let ll: INT;
	let mut li: INT;

	l = EDIT_EM_LineFromChar(es, e);
	ll = EDIT_EM_LineLength(es, e);
	li = EDIT_EM_LineIndex(es, l);
	if e - li == 0 {
		if l != 0 {
			li = EDIT_EM_LineIndex(es, l - 1);
			e = li + EDIT_EM_LineLength(es, li);
		}
	} else {
		e = li + EDIT_CallWordBreakProc(es, li, e - li, ll, WB_LEFT as INT);
	}
	if extend == 0 {
		s = e;
	}
	EDIT_EM_SetSel(es, s as UINT, e as UINT, 0);
	EDIT_EM_ScrollCaret(es);
}

unsafe fn EDIT_MoveWordForward(es: *mut EDITSTATE, extend: BOOL) {
	let mut s: INT = (*es).selection_start;
	let mut e: INT = (*es).selection_end;
	let l: INT;
	let ll: INT;
	let li: INT;

	l = EDIT_EM_LineFromChar(es, e);
	ll = EDIT_EM_LineLength(es, e);
	li = EDIT_EM_LineIndex(es, l);
	if e - li == ll {
		if ((*es).style & ES_MULTILINE != 0) && (l != (*es).line_count - 1) {
			e = EDIT_EM_LineIndex(es, l + 1);
		}
	} else {
		e = li + EDIT_CallWordBreakProc(es, li, e - li + 1, ll, WB_RIGHT as INT);
	}
	if extend == 0 {
		s = e;
	}
	EDIT_EM_SetSel(es, s as UINT, e as UINT, 0);
	EDIT_EM_ScrollCaret(es);
}

/// `EDIT_PaintText` (edit.c, `static`). `__REACTOS__` build: the `rev` block always
/// takes the `if (TRUE)` highlight path (composition-underline path is compiled out).
unsafe fn EDIT_PaintText(es: *mut EDITSTATE, dc: HDC, x: INT, y: INT, line: INT, col: INT, count: INT, rev: BOOL) -> INT {
    let BkColor: COLORREF;
    let TextColor: COLORREF;
    let ret: INT;
    let li: INT;
    let BkMode: INT;
    let mut size: SIZE = core::mem::zeroed();

    if count == 0 {
        return 0;
    }
    BkMode = GetBkMode(dc);
    BkColor = GetBkColor(dc);
    TextColor = GetTextColor(dc);
    if rev != 0 {
        /* __REACTOS__: if (TRUE) */
        SetBkColor(dc, GetSysColor(COLOR_HIGHLIGHT as INT));
        SetTextColor(dc, GetSysColor(COLOR_HIGHLIGHTTEXT as INT));
        SetBkMode(dc, OPAQUE as INT);
    }
    li = EDIT_EM_LineIndex(es, line);
    if (*es).style & ES_MULTILINE != 0 {
        ret = LOWORD(TabbedTextOutW(dc, x, y, (*es).text.offset((li + col) as isize), count,
                    (*es).tabs_count, (*es).tabs, (*es).format_rect.left - (*es).x_offset) as DWORD) as INT;
    } else {
        TextOutW(dc, x, y, (*es).text.offset((li + col) as isize), count);
        GetTextExtentPoint32W(dc, (*es).text.offset((li + col) as isize), count, &mut size);
        ret = size.cx;
    }
    if rev != 0 {
        /* __REACTOS__: if (TRUE) */
        SetBkColor(dc, BkColor);
        SetTextColor(dc, TextColor);
        SetBkMode(dc, BkMode);
    }
    ret
}

unsafe fn EDIT_PaintLine(es: *mut EDITSTATE, dc: HDC, line: INT, rev: BOOL) {
    let mut s: INT = 0;
    let mut e: INT = 0;
    let mut li: INT = 0;
    let mut ll: INT = 0;
    let mut x: INT;
    let y: INT;
    let pos: LRESULT;
    let ssa: SCRIPT_STRING_ANALYSIS;

    if (*es).style & ES_MULTILINE != 0 {
        let vlc = get_vertical_line_count(es);

        if (line < (*es).y_offset) || (line > (*es).y_offset + vlc) || (line >= (*es).line_count) {
            return;
        }
    } else if line != 0 {
        return;
    }

    ssa = EDIT_UpdateUniscribeData(es, dc, line);
    pos = EDIT_EM_PosFromChar(es, EDIT_EM_LineIndex(es, line), FALSE as BOOL);
    x = LOWORD(pos as u32) as i16 as INT;
    y = HIWORD(pos as u32) as i16 as INT;

    if (*es).style & ES_MULTILINE != 0 {
        let mut line_idx = line;
        x = -(*es).x_offset;
        if (*es).style & ES_RIGHT != 0 || (*es).style & ES_CENTER != 0 {
            let mut line_def = (*es).first_line_def;
            let w: i32;
            let lw: i32;

            while !line_def.is_null() && line_idx != 0 {
                line_def = (*line_def).next;
                line_idx -= 1;
            }
            w = (*es).format_rect.right - (*es).format_rect.left;
            lw = (*line_def).width;

            if (*es).style & ES_RIGHT != 0 {
                x = w - (lw - x);
            } else if (*es).style & ES_CENTER != 0 {
                x += (w - lw) / 2;
            }
        }
        x += (*es).format_rect.left;
    }

    if rev != 0 {
        li = EDIT_EM_LineIndex(es, line);
        ll = EDIT_EM_LineLength(es, li);
        s = min((*es).selection_start, (*es).selection_end);
        e = max((*es).selection_start, (*es).selection_end);
        s = min(li + ll, max(li, s));
        e = min(li + ll, max(li, e));
    }

    if !ssa.is_null() {
        ScriptStringOut(ssa, x, y, 0, core::ptr::addr_of!((*es).format_rect), s - li, e - li, FALSE as BOOL);
    } else if rev != 0 && (s != e) &&
            (((*es).flags as u32 & EF_FOCUSED as u32 != 0) || ((*es).style & ES_NOHIDESEL != 0)) {
        x += EDIT_PaintText(es, dc, x, y, line, 0, s - li, FALSE as BOOL);
        x += EDIT_PaintText(es, dc, x, y, line, s - li, e - s, TRUE as BOOL);
        x += EDIT_PaintText(es, dc, x, y, line, e - li, li + ll - e, FALSE as BOOL);
    } else {
        x += EDIT_PaintText(es, dc, x, y, line, 0, ll, FALSE as BOOL);
    }
}

unsafe fn EDIT_AdjustFormatRect(es: *mut EDITSTATE) {
    let mut ClientRect: RECT = core::mem::zeroed();

    let left_plus = (*es).format_rect.left + (*es).char_width;
    if left_plus > (*es).format_rect.right {
        (*es).format_rect.right = left_plus;
    }

    if (*es).style & ES_MULTILINE != 0 {
        let vlc = get_vertical_line_count(es);
        (*es).format_rect.bottom = (*es).format_rect.top + vlc * (*es).line_height;

        let fw = (*es).format_rect.right - (*es).format_rect.left;
        let mut max_x_offset = (*es).text_width - fw;
        if max_x_offset < 0 {
            max_x_offset = 0;
        }
        if (*es).x_offset > max_x_offset {
            (*es).x_offset = max_x_offset;
        }

        let mut max_y_offset = (*es).line_count - vlc;
        if max_y_offset < 0 {
            max_y_offset = 0;
        }
        if (*es).y_offset > max_y_offset {
            (*es).y_offset = max_y_offset;
        }

        EDIT_UpdateScrollInfo(es);
    } else {
        (*es).format_rect.bottom = (*es).format_rect.top + (*es).line_height;
    }

    GetClientRect((*es).hwndSelf, &mut ClientRect);
    if ClientRect.bottom < (*es).format_rect.bottom {
        (*es).format_rect.bottom = ClientRect.bottom;
    }

    if ((*es).style & ES_MULTILINE != 0) && ((*es).style & ES_AUTOHSCROLL == 0) {
        EDIT_BuildLineDefs_ML(es, 0, get_text_length(es) as INT, 0, core::ptr::null_mut());
    }

    EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags & EF_AFTER_WRAP) as BOOL);
}

unsafe fn EDIT_SetRectNP(es: *mut EDITSTATE, rc: *const RECT) {
    let ExStyle: LONG_PTR = GetWindowLongPtrW((*es).hwndSelf, GWL_EXSTYLE as i32);

    CopyRect(core::ptr::addr_of_mut!((*es).format_rect), rc);

    if ExStyle & (WS_EX_CLIENTEDGE as LONG_PTR) != 0 {
        (*es).format_rect.left += 1;
        (*es).format_rect.right -= 1;

        if (*es).format_rect.bottom - (*es).format_rect.top >= (*es).line_height + 2 {
            (*es).format_rect.top += 1;
            (*es).format_rect.bottom -= 1;
        }
    } else if (*es).style & WS_BORDER != 0 {
        let bw: INT = GetSystemMetrics(SM_CXBORDER as i32) + 1;
        let bh: INT = GetSystemMetrics(SM_CYBORDER as i32) + 1;
        (*es).format_rect.left += bw;
        (*es).format_rect.right -= bw;
        if (*es).format_rect.bottom - (*es).format_rect.top >= (*es).line_height + 2 * bh {
            (*es).format_rect.top += bh;
            (*es).format_rect.bottom -= bh;
        }
    }

    (*es).format_rect.left += (*es).left_margin;
    (*es).format_rect.right -= (*es).right_margin;
    EDIT_AdjustFormatRect(es);
}

unsafe fn EDIT_EM_CharFromPos(es: *mut EDITSTATE, x: INT, y: INT) -> LRESULT {
    let mut pt: POINT = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();
    let index: INT;

    pt.x = x;
    pt.y = y;
    GetClientRect((*es).hwndSelf, &mut rc);
    if PtInRect(&rc, pt) == 0 {
        return -1 as LRESULT;
    }

    index = EDIT_CharFromPos(es, x, y, core::ptr::null_mut());
    MAKELONG(index as WORD, EDIT_EM_LineFromChar(es, index) as WORD) as LRESULT
}

unsafe fn EDIT_EM_FmtLines(es: *mut EDITSTATE, add_eol: BOOL) -> BOOL {
    (*es).flags &= !(EF_USE_SOFTBRK as u16);
    if add_eol != 0 {
        (*es).flags |= EF_USE_SOFTBRK as u16;
    }
    add_eol
}

unsafe fn EDIT_EM_GetHandle(es: *mut EDITSTATE) -> HLOCAL {
    let hLocal: HLOCAL;

    if (*es).style & ES_MULTILINE == 0 {
        return core::ptr::null_mut();
    }

    if (*es).is_unicode != 0 {
        hLocal = (*es).hloc32W;
    } else {
        if (*es).hloc32A.is_null() {
            let textA: *mut CHAR;
            let countA: UINT;
            let alloc_size: UINT;
            countA = WideCharToMultiByte(CP_ACP, 0, (*es).text, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
            alloc_size = ROUND_TO_GROW(countA);
            (*es).hloc32A = LocalAlloc(LMEM_MOVEABLE | LMEM_ZEROINIT, alloc_size as SIZE_T);
            if (*es).hloc32A.is_null() {
                return core::ptr::null_mut();
            }
            textA = LocalLock((*es).hloc32A) as *mut CHAR;
            WideCharToMultiByte(CP_ACP, 0, (*es).text, -1, textA, countA as i32, core::ptr::null_mut(), core::ptr::null_mut());
            LocalUnlock((*es).hloc32A);
        }
        hLocal = (*es).hloc32A;
    }

    EDIT_UnlockBuffer(es, 1);

    (*es).hlocapp = hLocal;

    hLocal
}

unsafe fn EDIT_EM_GetLine(es: *mut EDITSTATE, mut line: INT, dst: LPWSTR, unicode: BOOL) -> INT {
    let src: LPWSTR;
    let line_len: INT;
    let dst_len: INT;
    let i: INT;

    if (*es).style & ES_MULTILINE as u32 != 0 {
        if line >= (*es).line_count {
            return 0;
        }
    } else {
        line = 0;
    }
    i = EDIT_EM_LineIndex(es, line);
    src = (*es).text.add(i as usize);
    line_len = EDIT_EM_LineLength(es, i);
    dst_len = *(dst as *mut u16) as INT;
    if unicode != 0 {
        if dst_len <= line_len {
            memcpy(
                dst as *mut core::ffi::c_void,
                src as *const core::ffi::c_void,
                (dst_len as usize) * core::mem::size_of::<u16>(),
            );
            dst_len
        } else {
            memcpy(
                dst as *mut core::ffi::c_void,
                src as *const core::ffi::c_void,
                (line_len as usize) * core::mem::size_of::<u16>(),
            );
            *dst.add(line_len as usize) = 0;
            line_len
        }
    } else {
        let ret: INT = WideCharToMultiByte(
            CP_ACP,
            0,
            src,
            line_len,
            dst as LPSTR,
            dst_len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ret == 0 && line_len != 0 {
            return dst_len;
        }
        if ret < dst_len {
            *(dst as LPSTR).add(ret as usize) = 0;
        }
        ret
    }
}

unsafe fn EDIT_EM_GetSel(es: *const EDITSTATE, start: PUINT, end: PUINT) -> LRESULT {
    let mut s: UINT = (*es).selection_start as UINT;
    let mut e: UINT = (*es).selection_end as UINT;

    if e < s {
        let temp = s;
        s = e;
        e = temp;
    }
    if !start.is_null() {
        *start = s;
    }
    if !end.is_null() {
        *end = e;
    }
    (((s & 0xffff) | ((e & 0xffff) << 16)) as u32) as LRESULT
}

unsafe fn EDIT_EM_ReplaceSel(es: *mut EDITSTATE, can_undo: BOOL, lpsz_replace: LPCWSTR, send_update: BOOL, honor_limit: BOOL) {
    let mut strl: UINT = strlenW(lpsz_replace) as UINT;
    let mut tl: UINT = get_text_length(es);
    let utl: UINT;
    let mut s: UINT;
    let mut e: UINT;
    let mut i: UINT;
    let size: UINT;
    let mut p: LPWSTR;
    let mut hrgn: HRGN = core::ptr::null_mut();
    let mut buf: LPWSTR = core::ptr::null_mut();
    let bufl: UINT;

    s = (*es).selection_start as UINT;
    e = (*es).selection_end as UINT;

    EDIT_InvalidateUniscribeData(es);
    if (s == e) && strl == 0 {
        return;
    }

    if e < s {
        let tmp = s;
        s = e;
        e = tmp;
    }

    size = tl - (e - s) + strl;
    if size == 0 {
        (*es).text_width = 0;
    }

    if honor_limit != 0 && size > (*es).buffer_limit {
        if notify_parent(es, EN_MAXTEXT as INT) == 0 {
            return;
        }
        if (*es).buffer_limit < (tl - (e - s)) {
            strl = 0;
        } else {
            let lim = (*es).buffer_limit - (tl - (e - s));
            strl = if strl < lim { strl } else { lim };
        }
    }

    if EDIT_MakeFit(es, tl - (e - s) + strl) == 0 {
        return;
    }

    if e != s {
        bufl = e - s;
        buf = HeapAlloc(GetProcessHeap(), 0, ((bufl + 1) as usize * core::mem::size_of::<u16>()) as _) as LPWSTR;
        if buf.is_null() {
            return;
        }
        memcpy(buf as _, (*es).text.add(s as usize) as _, (bufl as usize * core::mem::size_of::<u16>()) as _);
        *buf.add(bufl as usize) = 0;
        strcpyW((*es).text.add(s as usize), (*es).text.add(e as usize));
        text_buffer_changed(es);
    }
    if strl != 0 {
        tl = get_text_length(es);
        p = (*es).text.add(tl as usize);
        while p >= (*es).text.add(s as usize) {
            *p.add(strl as usize) = *p;
            p = p.sub(1);
        }
        i = 0;
        p = (*es).text.add(s as usize);
        while i < strl {
            *p.add(i as usize) = *lpsz_replace.add(i as usize);
            i += 1;
        }
        if (*es).style & ES_UPPERCASE != 0 {
            CharUpperBuffW(p, strl);
        } else if (*es).style & ES_LOWERCASE != 0 {
            CharLowerBuffW(p, strl);
        }
        text_buffer_changed(es);
    }
    if (*es).style & ES_MULTILINE != 0 {
        let st: INT = core::cmp::min((*es).selection_start, (*es).selection_end);
        let vlc: INT = get_vertical_line_count(es);

        hrgn = CreateRectRgn(0, 0, 0, 0);
        EDIT_BuildLineDefs_ML(es, st, st + strl as INT,
            strl as INT - abs((*es).selection_end - (*es).selection_start), hrgn);
        if honor_limit != 0 && (*es).style & ES_AUTOVSCROLL == 0 && (*es).line_count > vlc {
            if strl != 0 {
                strcpyW((*es).text.add(e as usize), (*es).text.add((e + strl) as usize));
            }
            if e != s {
                i = 0;
                p = (*es).text;
                while i < e - s {
                    *p.add((i + s) as usize) = *buf.add(i as usize);
                    i += 1;
                }
            }
            text_buffer_changed(es);
            EDIT_BuildLineDefs_ML(es, s as INT, e as INT,
                abs((*es).selection_end - (*es).selection_start) - strl as INT, hrgn);
            strl = 0;
            e = s;
            SetRectRgn(hrgn, 0, 0, 0, 0);
            if notify_parent(es, EN_MAXTEXT as INT) == 0 {
                return;
            }
        }
    } else {
        let fw: INT = (*es).format_rect.right - (*es).format_rect.left;
        EDIT_InvalidateUniscribeData(es);
        EDIT_CalcLineWidth_SL(es);
        if honor_limit != 0 && (*es).style & ES_AUTOHSCROLL == 0 && (*es).text_width > fw {
            while (*es).text_width > fw && s + strl >= s {
                strcpyW((*es).text.add((s + strl - 1) as usize), (*es).text.add((s + strl) as usize));
                strl -= 1;
                (*es).text_length = -1i32 as UINT;
                EDIT_InvalidateUniscribeData(es);
                EDIT_CalcLineWidth_SL(es);
            }
            text_buffer_changed(es);
            if notify_parent(es, EN_MAXTEXT as INT) == 0 {
                return;
            }
        }
    }

    if e != s {
        if can_undo != 0 {
            utl = strlenW((*es).undo_text) as UINT;
            if (*es).undo_insert_count == 0 && (*(*es).undo_text != 0 && s == (*es).undo_position) {
                EDIT_MakeUndoFit(es, utl + e - s);
                memcpy((*es).undo_text.add(utl as usize) as _, buf as _, ((e - s) as usize * core::mem::size_of::<u16>()) as _);
                *(*es).undo_text.add(utl as usize).add((e - s) as usize) = 0;
            } else if (*es).undo_insert_count == 0 && (*(*es).undo_text != 0 && e == (*es).undo_position) {
                EDIT_MakeUndoFit(es, utl + e - s);
                p = (*es).undo_text.add(utl as usize);
                while p >= (*es).undo_text {
                    *p.add((e - s) as usize) = *p;
                    p = p.sub(1);
                }
                i = 0;
                p = (*es).undo_text;
                while i < e - s {
                    *p.add(i as usize) = *buf.add(i as usize);
                    i += 1;
                }
                (*es).undo_position = s;
            } else {
                EDIT_MakeUndoFit(es, e - s);
                memcpy((*es).undo_text as _, buf as _, ((e - s) as usize * core::mem::size_of::<u16>()) as _);
                *(*es).undo_text.add((e - s) as usize) = 0;
                (*es).undo_position = s;
            }
            (*es).undo_insert_count = 0;
        } else {
            EDIT_EM_EmptyUndoBuffer(es);
        }
    }
    if strl != 0 {
        if can_undo != 0 {
            if s == (*es).undo_position ||
                ((*es).undo_insert_count != 0 &&
                 s == (*es).undo_position + (*es).undo_insert_count as UINT) {
                (*es).undo_insert_count += strl as INT;
            } else {
                (*es).undo_position = s;
                (*es).undo_insert_count = strl as INT;
                *(*es).undo_text = 0;
            }
        } else {
            EDIT_EM_EmptyUndoBuffer(es);
        }
    }

    HeapFree(GetProcessHeap(), 0, buf as _);

    s += strl;

    if (*es).style & (ES_RIGHT | ES_CENTER) != 0 {
        let delta: INT = strl as INT - abs((*es).selection_end - (*es).selection_start);

        if delta < 0 && (*es).x_offset != 0 {
            if abs(delta) > (*es).x_offset {
                (*es).x_offset = 0;
            } else {
                (*es).x_offset += delta;
            }
        }
    }

    EDIT_EM_SetSel(es, s, s, 0);
    (*es).flags |= EF_MODIFIED as u16;
    if send_update != 0 {
        (*es).flags |= EF_UPDATE as u16;
    }
    if !hrgn.is_null() {
        EDIT_UpdateTextRegion(es, hrgn, 1);
        DeleteObject(hrgn as _);
    } else {
        EDIT_UpdateText(es, core::ptr::null(), 1);
    }

    EDIT_EM_ScrollCaret(es);

    EDIT_UpdateScrollInfo(es);

    if send_update != 0 || (*es).flags & EF_UPDATE as u16 != 0 {
        (*es).flags &= !(EF_UPDATE as u16);
        if notify_parent(es, EN_CHANGE as INT) == 0 {
            return;
        }
    }
    EDIT_InvalidateUniscribeData(es);
}

unsafe fn EDIT_EM_SetHandle(es: *mut EDITSTATE, hloc: HLOCAL) {
    if ((*es).style & ES_MULTILINE == 0) {
        return;
    }

    if hloc.is_null() {
        return;
    }

    if !(*es).text.is_null() {
        EDIT_UnlockBuffer(es, 1);
    }

    if (*es).is_unicode != 0 {
        if !(*es).hloc32A.is_null() {
            LocalFree((*es).hloc32A);
            (*es).hloc32A = core::ptr::null_mut();
        }
        (*es).hloc32W = hloc;
    } else {
        let countW: INT;
        let countA: INT;
        let hloc32W_new: HLOCAL;
        let textW: *mut WCHAR;
        let textA: *mut CHAR;

        countA = LocalSize(hloc) as INT;
        textA = LocalLock(hloc) as *mut CHAR;
        countW = MultiByteToWideChar(CP_ACP, 0, textA, countA, core::ptr::null_mut(), 0);
        hloc32W_new = LocalAlloc(LMEM_MOVEABLE | LMEM_ZEROINIT, ((countW as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T);
        if hloc32W_new.is_null() {
            return;
        }
        textW = LocalLock(hloc32W_new) as *mut WCHAR;
        MultiByteToWideChar(CP_ACP, 0, textA, countA, textW, countW);
        LocalUnlock(hloc32W_new);
        LocalUnlock(hloc);

        if !(*es).hloc32W.is_null() {
            LocalFree((*es).hloc32W);
        }

        (*es).hloc32W = hloc32W_new;
        (*es).hloc32A = hloc;
    }

    (*es).buffer_size = (LocalSize((*es).hloc32W) / core::mem::size_of::<WCHAR>() as SIZE_T - 1) as UINT;

    (*es).hlocapp = core::ptr::null_mut();

    EDIT_LockBuffer(es);
    text_buffer_changed(es);

    (*es).x_offset = 0;
    (*es).y_offset = 0;
    (*es).selection_start = 0;
    (*es).selection_end = 0;
    EDIT_EM_EmptyUndoBuffer(es);
    (*es).flags &= !(EF_MODIFIED as WORD);
    (*es).flags &= !(EF_UPDATE as WORD);
    EDIT_BuildLineDefs_ML(es, 0, get_text_length(es) as INT, 0, core::ptr::null_mut());
    EDIT_UpdateText(es, core::ptr::null(), 1);
    EDIT_EM_ScrollCaret(es);
    EDIT_UpdateScrollInfo(es);
}

unsafe fn EDIT_EM_SetLimitText(es: *mut EDITSTATE, mut limit: UINT) {
    if limit == 0 {
        limit = !0u32;
    }
    if (*es).style & ES_MULTILINE == 0 {
        limit = core::cmp::min(limit, 0x7ffffffe);
    }
    (*es).buffer_limit = limit;
}

unsafe fn is_cjk(charset: UINT) -> BOOL {
    match charset {
        x if x == SHIFTJIS_CHARSET as UINT
            || x == HANGUL_CHARSET as UINT
            || x == GB2312_CHARSET as UINT
            || x == CHINESEBIG5_CHARSET as UINT =>
        {
            return 1;
        }
        _ => {}
    }
    0
}

unsafe fn EDIT_EM_SetMargins(es: *mut EDITSTATE, action: INT, left: WORD, right: WORD, repaint: BOOL) {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut default_left_margin: INT = 0;
    let mut default_right_margin: INT = 0;

    if !(*es).font.is_null() && (left as u32 == EC_USEFONTINFO || right as u32 == EC_USEFONTINFO) {
        let dc = GetDC((*es).hwndSelf);
        let old_font = SelectObject(dc, (*es).font as _);
        let width = GdiGetCharDimensions(dc, &mut tm, core::ptr::null_mut());
        let mut rc: RECT = core::mem::zeroed();

        if (tm.tmPitchAndFamily & ((TMPF_VECTOR | TMPF_TRUETYPE) as u8)) != 0 {
            if is_cjk(tm.tmCharSet as UINT) == 0 {
                default_left_margin = width / 2;
                default_right_margin = width / 2;

                GetClientRect((*es).hwndSelf, &mut rc);
                if rc.right - rc.left < (width / 2 + width) * 2 &&
                    (width >= 28 || IsRectEmpty(&rc) == 0) {
                    default_left_margin = (*es).left_margin;
                    default_right_margin = (*es).right_margin;
                }
            } else {
                default_left_margin = width / 2;
                default_right_margin = width / 2;
            }
        }
        SelectObject(dc, old_font);
        ReleaseDC((*es).hwndSelf, dc);
    }

    if action & (EC_LEFTMARGIN as INT) != 0 {
        (*es).format_rect.left -= (*es).left_margin;
        if left as u32 != EC_USEFONTINFO {
            (*es).left_margin = left as INT;
        } else {
            (*es).left_margin = default_left_margin;
        }
        (*es).format_rect.left += (*es).left_margin;
    }

    if action & (EC_RIGHTMARGIN as INT) != 0 {
        (*es).format_rect.right += (*es).right_margin;
        if right as u32 != EC_USEFONTINFO {
            (*es).right_margin = right as INT;
        } else {
            (*es).right_margin = default_right_margin;
        }
        (*es).format_rect.right -= (*es).right_margin;
    }

    if action & ((EC_LEFTMARGIN | EC_RIGHTMARGIN) as INT) != 0 {
        EDIT_AdjustFormatRect(es);
        if repaint != 0 {
            EDIT_UpdateText(es, core::ptr::null(), 1);
        }
    }
}

unsafe fn EDIT_EM_SetPasswordChar(es: *mut EDITSTATE, c: WCHAR) {

    let style: LONG;

    if (*es).style & ES_MULTILINE != 0 {
        return;
    }

    if (*es).password_char == c {
        return;
    }

    style = GetWindowLongW((*es).hwndSelf, GWL_STYLE as i32);
    (*es).password_char = c;
    if c != 0 {
        SetWindowLongW((*es).hwndSelf, GWL_STYLE as i32, style | ES_PASSWORD as LONG);
        (*es).style |= ES_PASSWORD;
    } else {
        SetWindowLongW((*es).hwndSelf, GWL_STYLE as i32, style & !(ES_PASSWORD as LONG));
        (*es).style &= !ES_PASSWORD;
    }
    EDIT_InvalidateUniscribeData(es);
    EDIT_UpdateText(es, core::ptr::null_mut(), 1);
}

unsafe fn EDIT_EM_SetTabStops(es: *mut EDITSTATE, count: INT, tabs: *const INT) -> BOOL {
    if ((*es).style & ES_MULTILINE) == 0 {
        return FALSE;
    }
    HeapFree(GetProcessHeap(), 0, (*es).tabs as *mut core::ffi::c_void);
    (*es).tabs_count = count;
    if count == 0 {
        (*es).tabs = core::ptr::null_mut();
    } else {
        (*es).tabs = HeapAlloc(GetProcessHeap(), 0, ((count as usize) * core::mem::size_of::<INT>()) as SIZE_T) as LPINT;
        if (*es).tabs.is_null() {
            (*es).tabs_count = 0;
            return FALSE;
        }
        memcpy(
            (*es).tabs as *mut core::ffi::c_void,
            tabs as *const core::ffi::c_void,
            (count as usize) * core::mem::size_of::<INT>(),
        );
    }
    EDIT_InvalidateUniscribeData(es);
    TRUE
}

unsafe fn EDIT_EM_SetWordBreakProc(es: *mut EDITSTATE, wbp: *mut core::ffi::c_void) {
    if (*es).word_break_proc == wbp {
        return;
    }

    (*es).word_break_proc = wbp;

    if ((*es).style & ES_MULTILINE != 0) && ((*es).style & ES_AUTOHSCROLL == 0) {
        EDIT_BuildLineDefs_ML(es, 0, get_text_length(es) as INT, 0, core::ptr::null_mut());
        EDIT_UpdateText(es, core::ptr::null(), 1);
    }
}

unsafe fn EDIT_EM_Undo(es: *mut EDITSTATE) -> BOOL {
    let ulength: INT;
    let utext: LPWSTR;

    if (*es).style & ES_READONLY != 0 {
        return if (*es).style & ES_MULTILINE != 0 { 0u8 as BOOL } else { 1u8 as BOOL };
    }

    ulength = strlenW((*es).undo_text) as INT;

    utext = HeapAlloc(GetProcessHeap(), 0, ((ulength + 1) as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
    if utext.is_null() {
        return 0u8 as BOOL;
    }

    strcpyW(utext, (*es).undo_text);

    EDIT_EM_SetSel(es, (*es).undo_position, ((*es).undo_position as INT + (*es).undo_insert_count) as UINT, 0u8 as BOOL);
    EDIT_EM_EmptyUndoBuffer(es);
    EDIT_EM_ReplaceSel(es, 1u8 as BOOL, utext, 1u8 as BOOL, 1u8 as BOOL);
    EDIT_EM_SetSel(es, (*es).undo_position, ((*es).undo_position as INT + (*es).undo_insert_count) as UINT, 0u8 as BOOL);
    if notify_parent(es, EN_CHANGE as INT) == 0 {
        return 1u8 as BOOL;
    }
    EDIT_EM_ScrollCaret(es);
    HeapFree(GetProcessHeap(), 0, utext as _);

    1u8 as BOOL
}

unsafe fn EDIT_IsInsideDialog(es: *mut EDITSTATE) -> BOOL {
    ((*es).flags as u32 & EF_DIALOGMODE as u32) as BOOL
}

unsafe fn EDIT_WM_Paste(es: *mut EDITSTATE) {
    let hsrc: HGLOBAL;
    let src: LPWSTR;

    if (*es).style & ES_READONLY != 0 {
        return;
    }

    OpenClipboard((*es).hwndSelf);
    hsrc = GetClipboardData(CF_UNICODETEXT);
    if !hsrc.is_null() {
        src = GlobalLock(hsrc) as LPWSTR;
        EDIT_EM_ReplaceSel(es, 1, src, 1, 1);
        GlobalUnlock(hsrc);
    } else if (*es).style & ES_PASSWORD != 0 {
        EDIT_EM_ReplaceSel(es, 1, empty_stringW.as_ptr(), 1, 1);
    }
    CloseClipboard();
}

unsafe fn EDIT_WM_Copy(es: *mut EDITSTATE) {
    let s = core::cmp::min((*es).selection_start, (*es).selection_end);
    let e = core::cmp::max((*es).selection_start, (*es).selection_end);
    let hdst: HGLOBAL;
    let dst: LPWSTR;
    let len: DWORD;

    if e == s {
        return;
    }

    len = (e - s) as DWORD;
    hdst = GlobalAlloc(GMEM_MOVEABLE | GMEM_DDESHARE, ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as _);
    dst = GlobalLock(hdst) as LPWSTR;
    core::ptr::copy_nonoverlapping((*es).text.add(s as usize), dst, len as usize);
    *dst.add(len as usize) = 0;
    GlobalUnlock(hdst);
    OpenClipboard((*es).hwndSelf);
    EmptyClipboard();
    SetClipboardData(CF_UNICODETEXT as _, hdst);
    CloseClipboard();
}

unsafe fn EDIT_WM_Clear(es: *mut EDITSTATE) {
    if (*es).style & ES_READONLY != 0 {
        return;
    }

    EDIT_EM_ReplaceSel(es, 1, empty_stringW.as_ptr(), 1, 1);
}

unsafe fn EDIT_WM_Cut(es: *mut EDITSTATE) {
    EDIT_WM_Copy(es);
    EDIT_WM_Clear(es);
}

unsafe fn EDIT_WM_Char(es: *mut EDITSTATE, c: WCHAR) -> LRESULT {
    let control: BOOL;

    if (*es).bCaptureState != 0 {
        return 0;
    }

    control = ((GetKeyState(VK_CONTROL as i32) as i32) & 0x8000) as BOOL;

    'sw: {
        match c as u32 {
            0x0D /* '\r' */ => {
                if ((*es).style & ES_MULTILINE != 0) && ((*es).style & ES_WANTRETURN == 0) {
                    if EDIT_IsInsideDialog(es) != 0 {
                        break 'sw;
                    }
                }
                /* fall through to '\n' */
                if (*es).style & ES_MULTILINE != 0 {
                    if (*es).style & ES_READONLY != 0 {
                        EDIT_MoveHome(es, 0, 0);
                        EDIT_MoveDown_ML(es, 0);
                    } else {
                        let cr_lfW: [WCHAR; 3] = ['\r' as u16, '\n' as u16, 0];
                        EDIT_EM_ReplaceSel(es, 1, cr_lfW.as_ptr(), 1, 1);
                    }
                }
            }
            0x0A /* '\n' */ => {
                if (*es).style & ES_MULTILINE != 0 {
                    if (*es).style & ES_READONLY != 0 {
                        EDIT_MoveHome(es, 0, 0);
                        EDIT_MoveDown_ML(es, 0);
                    } else {
                        let cr_lfW: [WCHAR; 3] = ['\r' as u16, '\n' as u16, 0];
                        EDIT_EM_ReplaceSel(es, 1, cr_lfW.as_ptr(), 1, 1);
                    }
                }
            }
            0x09 /* '\t' */ => {
                if ((*es).style & ES_MULTILINE != 0) && ((*es).style & ES_READONLY == 0) {
                    let tabW: [WCHAR; 2] = ['\t' as u16, 0];
                    if EDIT_IsInsideDialog(es) != 0 {
                        break 'sw;
                    }
                    EDIT_EM_ReplaceSel(es, 1, tabW.as_ptr(), 1, 1);
                }
            }
            x if x == VK_BACK as u32 => {
                if ((*es).style & ES_READONLY == 0) && control == 0 {
                    if (*es).selection_start != (*es).selection_end {
                        EDIT_WM_Clear(es);
                    } else {
                        EDIT_EM_SetSel(es, (-1i32) as UINT, 0, 0);
                        EDIT_MoveBackward(es, 1);
                        EDIT_WM_Clear(es);
                    }
                }
            }
            0x03 => {
                if (*es).style & ES_PASSWORD == 0 {
                    SendMessageW((*es).hwndSelf, WM_COPY, 0, 0);
                }
            }
            0x16 => {
                if (*es).style & ES_READONLY == 0 {
                    SendMessageW((*es).hwndSelf, WM_PASTE, 0, 0);
                }
            }
            0x18 => {
                if !(((*es).style & ES_READONLY != 0) || ((*es).style & ES_PASSWORD != 0)) {
                    SendMessageW((*es).hwndSelf, WM_CUT, 0, 0);
                }
            }
            0x1A => {
                if (*es).style & ES_READONLY == 0 {
                    SendMessageW((*es).hwndSelf, WM_UNDO, 0, 0);
                }
            }
            _ => {
                if ((*es).style & ES_NUMBER != 0) && !(c >= '0' as u16 && c <= '9' as u16) {
                    break 'sw;
                }

                if ((*es).style & ES_READONLY == 0) && (c >= ' ' as u16) && (c != 127) {
                    let mut str: [WCHAR; 2] = [0; 2];
                    str[0] = c;
                    str[1] = 0;
                    EDIT_EM_ReplaceSel(es, 1, str.as_ptr(), 1, 1);
                }
            }
        }
    }
    1
}

unsafe fn EDIT_ContextMenuCommand(es: *mut EDITSTATE, id: UINT) {
    match id {
        EM_UNDO => {
            SendMessageW((*es).hwndSelf, WM_UNDO, 0, 0);
        }
        WM_CUT => {
            SendMessageW((*es).hwndSelf, WM_CUT, 0, 0);
        }
        WM_COPY => {
            SendMessageW((*es).hwndSelf, WM_COPY, 0, 0);
        }
        WM_PASTE => {
            SendMessageW((*es).hwndSelf, WM_PASTE, 0, 0);
        }
        WM_CLEAR => {
            SendMessageW((*es).hwndSelf, WM_CLEAR, 0, 0);
        }
        EM_SETSEL => {
            SendMessageW((*es).hwndSelf, EM_SETSEL, 0, -1i32 as LPARAM);
        }
        _ => {
            ERR!("unknown menu item, please report\n");
        }
    }
}

unsafe fn EDIT_WM_ContextMenu(es: *mut EDITSTATE, mut x: INT, mut y: INT) {
    let menu: HMENU = LoadMenuA(User32Instance, b"EDITMENU\0".as_ptr() as LPCSTR);
    let popup: HMENU = GetSubMenu(menu, 0);
    let mut start: UINT = (*es).selection_start as UINT;
    let mut end: UINT = (*es).selection_end as UINT;
    let cmd: UINT;

    if end < start {
        let temp = start;
        start = end;
        end = temp;
    }

    /* undo */
    EnableMenuItem(popup, 0, MF_BYPOSITION | (if EDIT_EM_CanUndo(es) != 0 && ((*es).style & ES_READONLY) == 0 { MF_ENABLED } else { MF_GRAYED }));
    /* cut */
    EnableMenuItem(popup, 2, MF_BYPOSITION | (if (end - start) != 0 && ((*es).style & ES_PASSWORD) == 0 && ((*es).style & ES_READONLY) == 0 { MF_ENABLED } else { MF_GRAYED }));
    /* copy */
    EnableMenuItem(popup, 3, MF_BYPOSITION | (if (end - start) != 0 && ((*es).style & ES_PASSWORD) == 0 { MF_ENABLED } else { MF_GRAYED }));
    /* paste */
    EnableMenuItem(popup, 4, MF_BYPOSITION | (if IsClipboardFormatAvailable(CF_UNICODETEXT) != 0 && ((*es).style & ES_READONLY) == 0 { MF_ENABLED } else { MF_GRAYED }));
    /* delete */
    EnableMenuItem(popup, 5, MF_BYPOSITION | (if (end - start) != 0 && ((*es).style & ES_READONLY) == 0 { MF_ENABLED } else { MF_GRAYED }));
    /* select all */
    EnableMenuItem(popup, 7, MF_BYPOSITION | (if start != 0 || (end != get_text_length(es)) { MF_ENABLED } else { MF_GRAYED }));

    if x == -1 && y == -1 {
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*es).hwndSelf, &mut rc);
        MapWindowPoints((*es).hwndSelf, core::ptr::null_mut(), &mut rc as *mut RECT as *mut POINT, 2);
        x = rc.left + (rc.right - rc.left) / 2;
        y = rc.top + (rc.bottom - rc.top) / 2;
    }

    if ((*es).flags as u32 & EF_FOCUSED as u32) == 0 {
        SetFocus((*es).hwndSelf);
    }

    cmd = TrackPopupMenu(popup, TPM_LEFTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
                         x, y, 0, (*es).hwndSelf, core::ptr::null_mut()) as UINT;

    if cmd != 0 {
        EDIT_ContextMenuCommand(es, cmd);
    }

    DestroyMenu(menu);
}

unsafe fn EDIT_WM_GetText(es: *const EDITSTATE, count: INT, dst: LPWSTR, unicode: BOOL) -> INT {
    if count == 0 {
        return 0;
    }

    if unicode != 0 {
        lstrcpynW(dst, (*es).text, count);
        strlenW(dst) as INT
    } else {
        let textA = dst as LPSTR;
        if WideCharToMultiByte(CP_ACP, 0, (*es).text, -1, textA, count, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
            *textA.offset((count - 1) as isize) = 0;
        }
        strlen(textA) as INT
    }
}

unsafe fn EDIT_CheckCombo(es: *mut EDITSTATE, msg: UINT, mut key: INT) -> BOOL {
    let hLBox: HWND = (*es).hwndListBox;
    let hCombo: HWND;
    let mut bDropped: BOOL;
    let mut nEUI: i32;

    if hLBox.is_null() {
        return 0;
    }

    hCombo = GetParent((*es).hwndSelf);
    bDropped = 1;
    nEUI = 0;

    if key == VK_UP as INT || key == VK_DOWN as INT {
        if SendMessageW(hCombo, CB_GETEXTENDEDUI, 0, 0) != 0 {
            nEUI = 1;
        }

        if msg == WM_KEYDOWN || nEUI != 0 {
            bDropped = SendMessageW(hCombo, CB_GETDROPPEDSTATE, 0, 0) as BOOL;
        }
    }

    match msg {
        WM_KEYDOWN => {
            if bDropped == 0 && nEUI != 0 && (key == VK_UP as INT || key == VK_DOWN as INT) {
                SendMessageW(hCombo, CB_SETEXTENDEDUI, 0, 0);
                key = VK_F4 as INT;
                nEUI = 2;
            }

            SendMessageW(hLBox, WM_KEYDOWN, key as WPARAM, 0);
        }
        WM_SYSKEYDOWN => {
            if nEUI != 0 {
                SendMessageW(hCombo, CB_SHOWDROPDOWN, (bDropped == 0) as WPARAM, 0);
            } else {
                SendMessageW(hLBox, WM_KEYDOWN, VK_F4 as WPARAM, 0);
            }
        }
        _ => {}
    }

    if nEUI == 2 {
        SendMessageW(hCombo, CB_SETEXTENDEDUI, 1, 0);
    }

    1
}

unsafe fn EDIT_WM_KeyDown(es: *mut EDITSTATE, key: INT) -> LRESULT {
    let shift: BOOL;
    let control: BOOL;

    if GetKeyState(VK_MENU as INT) as i32 & 0x8000 != 0 {
        return 0;
    }

    shift = (GetKeyState(VK_SHIFT as INT) as i32 & 0x8000 != 0) as BOOL;
    control = (GetKeyState(VK_CONTROL as INT) as i32 & 0x8000 != 0) as BOOL;

    loop {
        if key == VK_F4 as INT || key == VK_UP as INT {
            if EDIT_CheckCombo(es, WM_KEYDOWN, key) != 0 || key == VK_F4 as INT {
                break;
            }
            // fall through to VK_LEFT
            if ((*es).style & ES_MULTILINE != 0) && (key == VK_UP as INT) {
                EDIT_MoveUp_ML(es, shift);
            } else if control != 0 {
                EDIT_MoveWordBackward(es, shift);
            } else {
                EDIT_MoveBackward(es, shift);
            }
            break;
        }

        match key as u32 {
            x if x == VK_LEFT => {
                if ((*es).style & ES_MULTILINE != 0) && (key == VK_UP as INT) {
                    EDIT_MoveUp_ML(es, shift);
                } else if control != 0 {
                    EDIT_MoveWordBackward(es, shift);
                } else {
                    EDIT_MoveBackward(es, shift);
                }
            }
            x if x == VK_DOWN => {
                if EDIT_CheckCombo(es, WM_KEYDOWN, key) != 0 {
                    break;
                }
                // fall through to VK_RIGHT
                if ((*es).style & ES_MULTILINE != 0) && (key == VK_DOWN as INT) {
                    EDIT_MoveDown_ML(es, shift);
                } else if control != 0 {
                    EDIT_MoveWordForward(es, shift);
                } else {
                    EDIT_MoveForward(es, shift);
                }
            }
            x if x == VK_RIGHT => {
                if ((*es).style & ES_MULTILINE != 0) && (key == VK_DOWN as INT) {
                    EDIT_MoveDown_ML(es, shift);
                } else if control != 0 {
                    EDIT_MoveWordForward(es, shift);
                } else {
                    EDIT_MoveForward(es, shift);
                }
            }
            x if x == VK_HOME => {
                EDIT_MoveHome(es, shift, control);
            }
            x if x == VK_END => {
                EDIT_MoveEnd(es, shift, control);
            }
            x if x == VK_PRIOR => {
                if (*es).style & ES_MULTILINE != 0 {
                    EDIT_MovePageUp_ML(es, shift);
                } else {
                    EDIT_CheckCombo(es, WM_KEYDOWN, key);
                }
            }
            x if x == VK_NEXT => {
                if (*es).style & ES_MULTILINE != 0 {
                    EDIT_MovePageDown_ML(es, shift);
                } else {
                    EDIT_CheckCombo(es, WM_KEYDOWN, key);
                }
            }
            x if x == VK_DELETE => {
                if (*es).style & ES_READONLY == 0 && !(shift != 0 && control != 0) {
                    if (*es).selection_start != (*es).selection_end {
                        if shift != 0 {
                            EDIT_WM_Cut(es);
                        } else {
                            EDIT_WM_Clear(es);
                        }
                    } else if shift != 0 {
                        EDIT_EM_SetSel(es, (-1i32) as UINT, 0, 0);
                        EDIT_MoveBackward(es, 1);
                        EDIT_WM_Clear(es);
                    } else if control != 0 {
                        EDIT_EM_SetSel(es, (-1i32) as UINT, 0, 0);
                        EDIT_MoveEnd(es, 1, 0);
                        EDIT_WM_Clear(es);
                    } else {
                        EDIT_EM_SetSel(es, (-1i32) as UINT, 0, 0);
                        EDIT_MoveForward(es, 1);
                        EDIT_WM_Clear(es);
                    }
                }
            }
            x if x == VK_INSERT => {
                if shift != 0 {
                    if (*es).style & ES_READONLY == 0 {
                        EDIT_WM_Paste(es);
                    }
                } else if control != 0 {
                    EDIT_WM_Copy(es);
                }
            }
            x if x == VK_RETURN => {
                if ((*es).style & ES_MULTILINE == 0) || ((*es).style & ES_WANTRETURN == 0) {
                    let dw: DWORD;

                    if EDIT_IsInsideDialog(es) == 0 {
                        break;
                    }
                    if control != 0 {
                        break;
                    }
                    dw = SendMessageW((*es).hwndParent, DM_GETDEFID, 0, 0) as DWORD;
                    if HIWORD(dw) as u32 == DC_HASDEFID {
                        let hwDefCtrl = GetDlgItem((*es).hwndParent, LOWORD(dw) as INT);
                        if !hwDefCtrl.is_null() {
                            SendMessageW((*es).hwndParent, WM_NEXTDLGCTL, hwDefCtrl as WPARAM, 1);
                            PostMessageW(hwDefCtrl, WM_KEYDOWN, VK_RETURN as WPARAM, 0);
                        }
                    }
                }
            }
            x if x == VK_ESCAPE => {
                if ((*es).style & ES_MULTILINE != 0) && EDIT_IsInsideDialog(es) != 0 {
                    PostMessageW((*es).hwndParent, WM_CLOSE, 0, 0);
                }
            }
            x if x == VK_TAB => {
                if ((*es).style & ES_MULTILINE != 0) && EDIT_IsInsideDialog(es) != 0 {
                    SendMessageW((*es).hwndParent, WM_NEXTDLGCTL, shift as WPARAM, 0);
                }
            }
            x if x == VK_BACK => {
                if control != 0 {
                    FIXME!("Ctrl+Backspace\n");
                }
            }
            _ => {}
        }
        break;
    }
    1
}

unsafe fn EDIT_WM_KillFocus(es: *mut EDITSTATE) -> LRESULT {
    let hCombo: HWND;
    let lStyles: LONG;

    (*es).flags &= !(EF_FOCUSED as WORD);
    DestroyCaret();
    if ((*es).style & ES_NOHIDESEL) == 0 {
        EDIT_InvalidateText(es, (*es).selection_start, (*es).selection_end);
    }

    (*es).wheelDeltaRemainder = 0;

    if (*es).hwndListBox.is_null() {
        if notify_parent(es, EN_KILLFOCUS as INT) == 0 {
            return 0;
        }
    } else {
        hCombo = GetParent((*es).hwndSelf);
        lStyles = GetWindowLongW(hCombo, GWL_STYLE as INT);
        if (lStyles & CBS_DROPDOWN as LONG) != 0 || (lStyles & CBS_SIMPLE as LONG) != 0 {
            SendMessageW(hCombo, WM_CBLOSTTEXTFOCUS, 0, 0);
        }
    }

    0
}

unsafe fn EDIT_WM_LButtonDblClk(es: *mut EDITSTATE) -> LRESULT {
	let s: INT;
	let mut e: INT = (*es).selection_end;
	let l: INT;
	let li: INT;
	let ll: INT;

	(*es).bCaptureState = 1;
	SetCapture((*es).hwndSelf);

	l = EDIT_EM_LineFromChar(es, e);
	li = EDIT_EM_LineIndex(es, l);
	ll = EDIT_EM_LineLength(es, e);
	s = li + EDIT_CallWordBreakProc(es, li, e - li, ll, WB_LEFT as INT);
	e = li + EDIT_CallWordBreakProc(es, li, e - li, ll, WB_RIGHT as INT);
	EDIT_EM_SetSel(es, s as UINT, e as UINT, 0);
	EDIT_EM_ScrollCaret(es);
	(*es).region_posy = 0;
	(*es).region_posx = (*es).region_posy;
	SetTimer((*es).hwndSelf, 0, 100, None);
	0
}

unsafe fn EDIT_WM_LButtonDown(es: *mut EDITSTATE, keys: DWORD, mut x: INT, mut y: INT) -> LRESULT {
    let e: INT;
    let mut after_wrap: BOOL = 0;

    (*es).bCaptureState = 1;
    SetCapture((*es).hwndSelf);
    EDIT_ConfinePoint(es, &mut x, &mut y);
    e = EDIT_CharFromPos(es, x, y, &mut after_wrap);
    EDIT_EM_SetSel(es, if keys & MK_SHIFT != 0 { (*es).selection_start as UINT } else { e as UINT }, e as UINT, after_wrap);
    EDIT_EM_ScrollCaret(es);
    (*es).region_posy = 0;
    (*es).region_posx = (*es).region_posy;
    SetTimer((*es).hwndSelf, 0, 100, None);

    if (*es).flags as u32 & EF_FOCUSED as u32 == 0 {
        SetFocus((*es).hwndSelf);
    }

    0
}

unsafe fn EDIT_WM_LButtonUp(es: *mut EDITSTATE) -> LRESULT {
    if (*es).bCaptureState != 0 {
        KillTimer((*es).hwndSelf, 0);
        if GetCapture() == (*es).hwndSelf {
            ReleaseCapture();
        }
    }
    (*es).bCaptureState = 0;
    0
}

unsafe fn EDIT_WM_MButtonDown(es: *mut EDITSTATE) -> LRESULT {
    SendMessageW((*es).hwndSelf, WM_PASTE, 0, 0);
    0
}

unsafe fn EDIT_WM_MouseMove(es: *mut EDITSTATE, mut x: INT, mut y: INT) -> LRESULT {
	let e: INT;
	let mut after_wrap: BOOL = 0;
	let prex: INT;
	let prey: INT;

	if (*es).bCaptureState == 0 || GetCapture() != (*es).hwndSelf {
		return 0;
	}

	prex = x;
	prey = y;
	EDIT_ConfinePoint(es, core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(y));
	(*es).region_posx = if prex < x { -1 } else if prex > x { 1 } else { 0 };
	(*es).region_posy = if prey < y { -1 } else if prey > y { 1 } else { 0 };
	e = EDIT_CharFromPos(es, x, y, core::ptr::addr_of_mut!(after_wrap));
	EDIT_EM_SetSel(es, (*es).selection_start as UINT, e as UINT, after_wrap);
	EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL);
	0
}

unsafe fn EDIT_WM_Paint(es: *mut EDITSTATE, hdc: HDC) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut i: INT;
    let dc: HDC;
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut rcLine: RECT = core::mem::zeroed();
    let mut rcRgn: RECT = core::mem::zeroed();
    let brush: HBRUSH;
    let old_brush: HBRUSH;
    let bw: INT;
    let bh: INT;
    let rev: BOOL = ((*es).bEnableState != 0
        && (((*es).flags as u32 & EF_FOCUSED as u32) != 0
            || ((*es).style & ES_NOHIDESEL as u32) != 0)) as BOOL;

    dc = if !hdc.is_null() {
        hdc
    } else {
        BeginPaint((*es).hwndSelf, &mut ps)
    };

    EDIT_InvalidateUniscribeData(es);
    GetClientRect((*es).hwndSelf, &mut rcClient);

    brush = EDIT_NotifyCtlColor(es, dc);

    IntersectClipRect(dc, rcClient.left, rcClient.top, rcClient.right, rcClient.bottom);

    if (*es).style & WS_BORDER as u32 != 0 {
        bw = GetSystemMetrics(SM_CXBORDER as INT);
        bh = GetSystemMetrics(SM_CYBORDER as INT);
        rc = rcClient;
        if (*es).style & ES_MULTILINE as u32 != 0 {
            if (*es).style & WS_HSCROLL as u32 != 0 {
                rc.bottom += bh;
            }
            if (*es).style & WS_VSCROLL as u32 != 0 {
                rc.right += bw;
            }
        }

        old_brush = SelectObject(dc, GetSysColorBrush(COLOR_WINDOWFRAME as INT) as *mut _) as HBRUSH;
        PatBlt(dc, rc.left, rc.top, rc.right - rc.left, bh, PATCOPY);
        PatBlt(dc, rc.left, rc.top, bw, rc.bottom - rc.top, PATCOPY);
        PatBlt(dc, rc.left, rc.bottom - 1, rc.right - rc.left, -bw, PATCOPY);
        PatBlt(dc, rc.right - 1, rc.top, -bw, rc.bottom - rc.top, PATCOPY);
        SelectObject(dc, old_brush as *mut _);

        IntersectClipRect(
            dc,
            rc.left + bw,
            rc.top + bh,
            max(rc.right - bw, rc.left + bw),
            max(rc.bottom - bh, rc.top + bh),
        );
    }

    GetClipBox(dc, &mut rc);
    FillRect(dc, &rc, brush);

    IntersectClipRect(
        dc,
        (*es).format_rect.left,
        (*es).format_rect.top,
        (*es).format_rect.right,
        (*es).format_rect.bottom,
    );
    if (*es).style & ES_MULTILINE as u32 != 0 {
        rc = rcClient;
        IntersectClipRect(dc, rc.left, rc.top, rc.right, rc.bottom);
    }
    if !(*es).font.is_null() {
        old_font = SelectObject(dc, (*es).font as *mut _) as HFONT;
    }

    if (*es).bEnableState == 0 {
        SetTextColor(dc, GetSysColor(COLOR_GRAYTEXT as INT));
    }
    GetClipBox(dc, &mut rcRgn);
    if (*es).style & ES_MULTILINE as u32 != 0 {
        let vlc: INT = get_vertical_line_count(es);
        i = (*es).y_offset;
        while i <= min((*es).y_offset + vlc, (*es).y_offset + (*es).line_count - 1) {
            EDIT_UpdateUniscribeData(es, dc, i);
            EDIT_GetLineRect(es, i, 0, -1, &mut rcLine);
            if IntersectRect(&mut rc, &rcRgn, &rcLine) != 0 {
                EDIT_PaintLine(es, dc, i, rev);
            }
            i += 1;
        }
    } else {
        EDIT_UpdateUniscribeData(es, dc, 0);
        EDIT_GetLineRect(es, 0, 0, -1, &mut rcLine);
        if IntersectRect(&mut rc, &rcRgn, &rcLine) != 0 {
            EDIT_PaintLine(es, dc, 0, rev);
        }
    }
    if !(*es).font.is_null() {
        SelectObject(dc, old_font as *mut _);
    }

    if hdc.is_null() {
        EndPaint((*es).hwndSelf, &ps);
    }
}

unsafe fn EDIT_WM_SetFocus(es: *mut EDITSTATE) {
    (*es).flags |= EF_FOCUSED as WORD;

    if ((*es).style & ES_NOHIDESEL) == 0 {
        EDIT_InvalidateText(es, (*es).selection_start, (*es).selection_end);
    }

    if IsWindowVisible((*es).hwndSelf) != 0 && ((*es).style & ES_MULTILINE) == 0 {
        let hdc = GetDC((*es).hwndSelf);
        EDIT_WM_Paint(es, hdc);
        ReleaseDC((*es).hwndSelf, hdc);
    }

    SystemParametersInfoW(SPI_GETCARETWIDTH, 0, core::ptr::addr_of_mut!((*es).dwCaretWidth) as *mut core::ffi::c_void, 0);
    CreateCaret((*es).hwndSelf, core::ptr::null_mut(), (*es).dwCaretWidth as INT, (*es).line_height);

    EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags & EF_AFTER_WRAP as WORD) as BOOL);
    ShowCaret((*es).hwndSelf);
    notify_parent(es, EN_SETFOCUS as INT);
}

unsafe fn EDIT_WM_SetFont(es: *mut EDITSTATE, mut font: HFONT, redraw: BOOL) {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let dc: HDC;
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut clientRect: RECT = core::mem::zeroed();

    (*es).font = font;
    EDIT_InvalidateUniscribeData(es);
    dc = GetDC((*es).hwndSelf);
    if !font.is_null() {
        old_font = SelectObject(dc, font as *mut core::ffi::c_void) as HFONT;
    }
    GetTextMetricsW(dc, &mut tm);
    (*es).line_height = tm.tmHeight;
    (*es).char_width = tm.tmAveCharWidth;
    if !font.is_null() {
        SelectObject(dc, old_font as *mut core::ffi::c_void);
    }
    ReleaseDC((*es).hwndSelf, dc);

    GetClientRect((*es).hwndSelf, &mut clientRect);
    EDIT_SetRectNP(es, &clientRect);
    EDIT_EM_SetMargins(es, (EC_LEFTMARGIN | EC_RIGHTMARGIN) as INT,
                       EC_USEFONTINFO as WORD, EC_USEFONTINFO as WORD, 0);

    if (*es).style & ES_MULTILINE != 0 {
        EDIT_BuildLineDefs_ML(es, 0, get_text_length(es) as INT, 0, core::ptr::null_mut());
    } else {
        EDIT_CalcLineWidth_SL(es);
    }

    if redraw != 0 {
        EDIT_UpdateText(es, core::ptr::null(), 1);
    }
    if (*es).flags & EF_FOCUSED as WORD != 0 {
        DestroyCaret();
        CreateCaret((*es).hwndSelf, core::ptr::null_mut(), (*es).dwCaretWidth as INT, (*es).line_height);
        EDIT_SetCaretPos(es, (*es).selection_end, ((*es).flags & EF_AFTER_WRAP as WORD) as BOOL);
        ShowCaret((*es).hwndSelf);
    }

    if ImmIsIME(GetKeyboardLayout(0)) != 0 {
        let mut lf: LOGFONTW = core::mem::zeroed();
        let hIMC: HIMC = ImmGetContext((*es).hwndSelf);
        if font.is_null() {
            font = GetStockObject(DEFAULT_GUI_FONT as INT) as HFONT;
        }
        GetObjectW(font as *mut core::ffi::c_void, core::mem::size_of::<LOGFONTW>() as INT, &mut lf as *mut _ as *mut core::ffi::c_void);
        ImmSetCompositionFontW(hIMC, &mut lf);
        ImmReleaseContext((*es).hwndSelf, hIMC);
    }
}

unsafe fn EDIT_WM_SetText(es: *mut EDITSTATE, mut text: LPCWSTR, unicode: BOOL) {
    let mut textW: LPWSTR = core::ptr::null_mut();
    if unicode == 0 && !text.is_null() {
        let textA = text as LPCSTR;
        let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
        textW = HeapAlloc(GetProcessHeap(), 0, (countW as usize * core::mem::size_of::<u16>()) as _) as LPWSTR;
        if !textW.is_null() {
            MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
        }
        text = textW;
    }

    EDIT_EM_SetSel(es, 0, (-1i32) as UINT, 0);
    if !text.is_null() {
        EDIT_EM_ReplaceSel(es, 0, text, 0, 0);
        if unicode == 0 {
            HeapFree(GetProcessHeap(), 0, textW as _);
        }
    } else {
        EDIT_EM_ReplaceSel(es, 0, empty_stringW.as_ptr(), 0, 0);
    }
    (*es).x_offset = 0;
    (*es).flags &= !(EF_MODIFIED as u16);
    EDIT_EM_SetSel(es, 0, 0, 0);

    if !(((*es).style & ES_MULTILINE != 0) || !(*es).hwndListBox.is_null()) {
        if notify_parent(es, EN_UPDATE as INT) == 0 {
            return;
        }
        if notify_parent(es, EN_CHANGE as INT) == 0 {
            return;
        }
    }
    EDIT_EM_ScrollCaret(es);
    EDIT_UpdateScrollInfo(es);
    EDIT_InvalidateUniscribeData(es);
}

unsafe fn EDIT_WM_Size(es: *mut EDITSTATE, action: UINT) {
    if (action == SIZE_MAXIMIZED as UINT) || (action == SIZE_RESTORED as UINT) {
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*es).hwndSelf, &mut rc);
        EDIT_SetRectNP(es, &rc);
        EDIT_UpdateText(es, core::ptr::null(), 1);
    }
}

unsafe fn EDIT_WM_StyleChanged(es: *mut EDITSTATE, which: WPARAM, style: *const STYLESTRUCT) -> LRESULT {
    if which == GWL_STYLE as WPARAM {
        let mut style_change_mask: DWORD = (ES_UPPERCASE | ES_LOWERCASE | ES_NUMBER) as DWORD;
        if (*es).style & ES_MULTILINE != 0 {
            style_change_mask |= ES_WANTRETURN as DWORD;
        }

        let mut new_style: DWORD = (*style).styleNew & style_change_mask;

        if new_style & ES_NUMBER as DWORD != 0 {
            /* do not override the ES_NUMBER */
        } else if new_style & ES_LOWERCASE as DWORD != 0 {
            new_style &= !(ES_UPPERCASE as DWORD);
        }

        (*es).style = ((*es).style & !style_change_mask) | new_style;
    } else if which == GWL_EXSTYLE as WPARAM {
        /* FIXME - what is needed here */
    }

    0
}

unsafe fn EDIT_WM_SysKeyDown(es: *mut EDITSTATE, key: INT, key_data: DWORD) -> LRESULT {
    if (key == VK_BACK as INT) && (key_data & 0x2000) != 0 {
        if EDIT_EM_CanUndo(es) != 0 {
            EDIT_EM_Undo(es);
        }
        return 0;
    } else if key == VK_UP as INT || key == VK_DOWN as INT {
        if EDIT_CheckCombo(es, WM_SYSKEYDOWN, key) != 0 {
            return 0;
        }
    }
    DefWindowProcW((*es).hwndSelf, WM_SYSKEYDOWN, key as WPARAM, key_data as LPARAM)
}

unsafe fn EDIT_WM_Timer(es: *mut EDITSTATE) {
	if (*es).region_posx < 0 {
		EDIT_MoveBackward(es, 1);
	} else if (*es).region_posx > 0 {
		EDIT_MoveForward(es, 1);
	}
}

unsafe fn EDIT_WM_HScroll(es: *mut EDITSTATE, action: INT, pos: INT) -> LRESULT {
    let mut dx: INT;
    let fw: INT;

    if ((*es).style & ES_MULTILINE) == 0 {
        return 0;
    }

    if ((*es).style & ES_AUTOHSCROLL) == 0 {
        return 0;
    }

    dx = 0;
    fw = (*es).format_rect.right - (*es).format_rect.left;
    match action as u32 {
        SB_LINELEFT => {
            if (*es).x_offset != 0 {
                dx = -(*es).char_width;
            }
        }
        SB_LINERIGHT => {
            if (*es).x_offset < (*es).text_width {
                dx = (*es).char_width;
            }
        }
        SB_PAGELEFT => {
            if (*es).x_offset != 0 {
                dx = -fw / HSCROLL_FRACTION / (*es).char_width * (*es).char_width;
            }
        }
        SB_PAGERIGHT => {
            if (*es).x_offset < (*es).text_width {
                dx = fw / HSCROLL_FRACTION / (*es).char_width * (*es).char_width;
            }
        }
        SB_LEFT => {
            if (*es).x_offset != 0 {
                dx = -(*es).x_offset;
            }
        }
        SB_RIGHT => {
            if (*es).x_offset < (*es).text_width {
                dx = (*es).text_width - (*es).x_offset;
            }
        }
        SB_THUMBTRACK => {
            (*es).flags |= EF_HSCROLL_TRACK as u16;
            if ((*es).style & WS_HSCROLL) != 0 {
                dx = pos - (*es).x_offset;
            } else {
                let fw_local: INT;
                let new_x: INT;
                if pos < 0 || pos > 100 {
                    return 0;
                }
                fw_local = (*es).format_rect.right - (*es).format_rect.left;
                new_x = pos * ((*es).text_width - fw_local) / 100;
                dx = if (*es).text_width != 0 { new_x - (*es).x_offset } else { 0 };
            }
        }
        SB_THUMBPOSITION => {
            (*es).flags &= !(EF_HSCROLL_TRACK as u16);
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE as i32) & WS_HSCROLL as i32) != 0 {
                dx = pos - (*es).x_offset;
            } else {
                let fw_local: INT;
                let new_x: INT;
                if pos < 0 || pos > 100 {
                    return 0;
                }
                fw_local = (*es).format_rect.right - (*es).format_rect.left;
                new_x = pos * ((*es).text_width - fw_local) / 100;
                dx = if (*es).text_width != 0 { new_x - (*es).x_offset } else { 0 };
            }
            if dx == 0 {
                EDIT_UpdateScrollInfo(es);
                notify_parent(es, EN_HSCROLL as INT);
            }
        }
        SB_ENDSCROLL => {}
        EM_GETTHUMB => {
            let ret: LRESULT;
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE as i32) & WS_HSCROLL as i32) != 0 {
                ret = GetScrollPos((*es).hwndSelf, SB_HORZ as i32) as LRESULT;
            } else {
                let fw_local: INT = (*es).format_rect.right - (*es).format_rect.left;
                ret = if (*es).text_width != 0 {
                    ((*es).x_offset * 100 / ((*es).text_width - fw_local)) as LRESULT
                } else {
                    0
                };
            }
            return ret;
        }
        EM_LINESCROLL => {
            dx = pos;
        }
        _ => {
            return 0;
        }
    }
    if dx != 0 {
        let fw_local: INT = (*es).format_rect.right - (*es).format_rect.left;
        if (*es).x_offset + dx + fw_local > (*es).text_width {
            dx = (*es).text_width - fw_local - (*es).x_offset;
        }
        if dx != 0 {
            EDIT_EM_LineScroll_internal(es, dx, 0);
        }
    }
    0
}

unsafe fn EDIT_WM_VScroll(es: *mut EDITSTATE, action: INT, pos: INT) -> LRESULT {
    let mut dy: INT;

    if ((*es).style & ES_MULTILINE) == 0 {
        return 0;
    }

    if ((*es).style & ES_AUTOVSCROLL) == 0 {
        return 0;
    }

    dy = 0;
    match action as u32 {
        SB_LINEUP | SB_LINEDOWN | SB_PAGEUP | SB_PAGEDOWN => {
            EDIT_EM_Scroll(es, action);
            return 0;
        }
        SB_TOP => {
            dy = -(*es).y_offset;
        }
        SB_BOTTOM => {
            dy = (*es).line_count - 1 - (*es).y_offset;
        }
        SB_THUMBTRACK => {
            (*es).flags |= EF_VSCROLL_TRACK as u16;
            if ((*es).style & WS_VSCROLL) != 0 {
                dy = pos - (*es).y_offset;
            } else {
                if pos < 0 || pos > 100 {
                    return 0;
                }
                let vlc = get_vertical_line_count(es);
                let new_y = pos * ((*es).line_count - vlc) / 100;
                dy = if (*es).line_count != 0 { new_y - (*es).y_offset } else { 0 };
            }
        }
        SB_THUMBPOSITION => {
            (*es).flags &= !(EF_VSCROLL_TRACK as u16);
            if ((*es).style & WS_VSCROLL) != 0 {
                dy = pos - (*es).y_offset;
            } else {
                if pos < 0 || pos > 100 {
                    return 0;
                }
                let vlc = get_vertical_line_count(es);
                let new_y = pos * ((*es).line_count - vlc) / 100;
                dy = if (*es).line_count != 0 { new_y - (*es).y_offset } else { 0 };
            }
            if dy == 0 {
                EDIT_UpdateScrollInfo(es);
                notify_parent(es, EN_VSCROLL as INT);
            }
        }
        SB_ENDSCROLL => {
        }
        EM_GETTHUMB => {
            let ret: LRESULT;
            if (GetWindowLongW((*es).hwndSelf, GWL_STYLE as i32) & WS_VSCROLL as i32) != 0 {
                ret = GetScrollPos((*es).hwndSelf, SB_VERT as i32) as LRESULT;
            } else {
                let vlc = get_vertical_line_count(es);
                ret = if (*es).line_count != 0 {
                    ((*es).y_offset * 100 / ((*es).line_count - vlc)) as LRESULT
                } else {
                    0
                };
            }
            return ret;
        }
        EM_LINESCROLL => {
            dy = pos;
        }
        _ => {
            return 0;
        }
    }
    if dy != 0 {
        EDIT_EM_LineScroll(es, 0, dy);
    }
    0
}

unsafe fn EDIT_EM_GetThumb(es: *mut EDITSTATE) -> LRESULT {
    let lo = EDIT_WM_VScroll(es, EM_GETTHUMB as INT, 0) as u16 as u32;
    let hi = EDIT_WM_HScroll(es, EM_GETTHUMB as INT, 0) as u16 as u32;
    ((lo | (hi << 16)) as i32) as LRESULT
}

unsafe fn EDIT_GetCompositionStr(hIMC: HIMC, CompFlag: LPARAM, es: *mut EDITSTATE) {
    let mut buflen: LONG;
    let lpCompStr: LPWSTR;
    let mut lpCompStrAttr: LPSTR = core::ptr::null_mut();
    let mut dwBufLenAttr: DWORD;

    buflen = ImmGetCompositionStringW(hIMC, GCS_COMPSTR as _, core::ptr::null_mut(), 0) as LONG;

    if buflen < 0 {
        return;
    }

    lpCompStr = HeapAlloc(GetProcessHeap(), 0, (buflen as usize + core::mem::size_of::<u16>()) as _) as LPWSTR;
    if lpCompStr.is_null() {
        ERR!("Unable to allocate IME CompositionString\n");
        return;
    }

    if buflen != 0 {
        ImmGetCompositionStringW(hIMC, GCS_COMPSTR as _, lpCompStr as _, buflen as _);
    }
    *lpCompStr.offset((buflen as usize / core::mem::size_of::<u16>()) as isize) = 0;

    if (CompFlag & GCS_COMPATTR as LPARAM) != 0 {
        dwBufLenAttr = ImmGetCompositionStringW(hIMC, GCS_COMPATTR as _, core::ptr::null_mut(), 0) as DWORD;
        if dwBufLenAttr != 0 {
            dwBufLenAttr += 1;
            lpCompStrAttr = HeapAlloc(GetProcessHeap(), 0, (dwBufLenAttr + 1) as _) as LPSTR;
            if lpCompStrAttr.is_null() {
                ERR!("Unable to allocate IME Attribute String\n");
                HeapFree(GetProcessHeap(), 0, lpCompStr as _);
                return;
            }
            ImmGetCompositionStringW(hIMC, GCS_COMPATTR as _, lpCompStrAttr as _, dwBufLenAttr);
            *lpCompStrAttr.offset(dwBufLenAttr as isize) = 0;
        }
    }

    HeapFree(GetProcessHeap(), 0, lpCompStrAttr as _);
    HeapFree(GetProcessHeap(), 0, lpCompStr as _);
}

unsafe fn EDIT_GetResultStr(hIMC: HIMC, es: *mut EDITSTATE) {
    let buflen: LONG = ImmGetCompositionStringW(hIMC, GCS_RESULTSTR as _, core::ptr::null_mut(), 0);
    if buflen <= 0 {
        return;
    }

    let lpResultStr: LPWSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        (buflen as usize + core::mem::size_of::<u16>()) as _,
    ) as LPWSTR;
    if lpResultStr.is_null() {
        ERR!("Unable to alloc buffer for IME string\n");
        return;
    }

    ImmGetCompositionStringW(hIMC, GCS_RESULTSTR as _, lpResultStr as _, buflen as DWORD);
    *lpResultStr.offset((buflen as usize / core::mem::size_of::<u16>()) as isize) = 0;

    HeapFree(GetProcessHeap(), 0, lpResultStr as _);
}

unsafe fn EDIT_ImeComposition(hwnd: HWND, CompFlag: LPARAM, es: *mut EDITSTATE) {
    let hIMC: HIMC;
    let cursor: core::ffi::c_int;

    if (*es).selection_start != (*es).selection_end {
        EDIT_EM_ReplaceSel(es, 1, empty_stringW.as_ptr(), 1, 1);
    }

    hIMC = ImmGetContext(hwnd);
    if hIMC.is_null() {
        return;
    }

    if (CompFlag & GCS_RESULTSTR as LPARAM) != 0 {
        EDIT_GetResultStr(hIMC, es);
        cursor = 0;
    } else {
        if (CompFlag & GCS_COMPSTR as LPARAM) != 0 {
            EDIT_GetCompositionStr(hIMC, CompFlag, es);
        }
        cursor = 0;
    }
    ImmReleaseContext(hwnd, hIMC);
    EDIT_SetCaretPos(es, (*es).selection_start + cursor, ((*es).flags as u32 & EF_AFTER_WRAP as u32) as BOOL);
}

unsafe fn EDIT_WM_NCCreate(hwnd: HWND, lpcs: LPCREATESTRUCTW, unicode: BOOL) -> LRESULT {
    let es: *mut EDITSTATE;
    let alloc_size: UINT;

    es = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<EDITSTATE>() as _) as *mut EDITSTATE;
    if es.is_null() {
        return 0 as LRESULT;
    }
    SetWindowLongPtrW(hwnd, 0, es as LONG_PTR);

    (*es).is_unicode = unicode;
    (*es).style = (*lpcs).style as DWORD;

    (*es).bEnableState = if ((*es).style & WS_DISABLED) != 0 { 0 } else { 1 };

    (*es).hwndSelf = hwnd;
    (*es).hwndParent = (*lpcs).hwndParent;

    if ((*es).style & ES_COMBO) != 0 {
        (*es).hwndListBox = GetDlgItem((*es).hwndParent, ID_CB_LISTBOX);
    }

    if ((*lpcs).dwExStyle & WS_EX_RIGHT) != 0 {
        (*es).style |= ES_RIGHT;
    }

    if ((*es).style & ES_NUMBER) != 0 {
        /* do not override the ES_NUMBER */
    } else if ((*es).style & ES_LOWERCASE) != 0 {
        (*es).style &= !ES_UPPERCASE;
    }

    if ((*es).style & ES_MULTILINE) != 0 {
        (*es).buffer_limit = BUFLIMIT_INITIAL;
        if ((*es).style & WS_VSCROLL) != 0 {
            (*es).style |= ES_AUTOVSCROLL;
        }
        if ((*es).style & WS_HSCROLL) != 0 {
            (*es).style |= ES_AUTOHSCROLL;
        }
        (*es).style &= !ES_PASSWORD;
        if ((*es).style & ES_CENTER) != 0 || ((*es).style & ES_RIGHT) != 0 {
            if ((*es).style & ES_RIGHT) != 0 {
                (*es).style &= !ES_CENTER;
            }
            (*es).style &= !WS_HSCROLL;
            (*es).style &= !ES_AUTOHSCROLL;
        }
    } else {
        (*es).buffer_limit = BUFLIMIT_INITIAL;
        if ((*es).style & ES_RIGHT) != 0 && ((*es).style & ES_CENTER) != 0 {
            (*es).style &= !ES_CENTER;
        }
        (*es).style &= !WS_HSCROLL;
        (*es).style &= !WS_VSCROLL;
        if ((*es).style & ES_PASSWORD) != 0 {
            (*es).password_char = '*' as WCHAR;
        }
    }

    'cleanup: loop {
        alloc_size = ROUND_TO_GROW(((*es).buffer_size + 1) * core::mem::size_of::<WCHAR>() as UINT);
        (*es).hloc32W = LocalAlloc(LMEM_MOVEABLE | LMEM_ZEROINIT, alloc_size as _);
        if (*es).hloc32W.is_null() {
            break 'cleanup;
        }
        (*es).buffer_size = (LocalSize((*es).hloc32W) as UINT) / core::mem::size_of::<WCHAR>() as UINT - 1;

        (*es).undo_text = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (((*es).buffer_size + 1) * core::mem::size_of::<WCHAR>() as UINT) as _) as LPWSTR;
        if (*es).undo_text.is_null() {
            break 'cleanup;
        }
        (*es).undo_buffer_size = (*es).buffer_size;

        if ((*es).style & ES_MULTILINE) != 0 {
            (*es).first_line_def = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<LINEDEF>() as _) as *mut LINEDEF;
            if (*es).first_line_def.is_null() {
                break 'cleanup;
            }
        }
        (*es).line_count = 1;

        if ((*lpcs).dwExStyle & WS_EX_CLIENTEDGE) != 0 {
            (*es).style &= !WS_BORDER;
        } else if ((*es).style & WS_BORDER) != 0 {
            SetWindowLongW(hwnd, GWL_STYLE, ((*es).style & !WS_BORDER) as _);
        }

        return 1 as LRESULT;
    }

    SetWindowLongPtrW((*es).hwndSelf, 0, 0);
    EDIT_InvalidateUniscribeData(es);
    HeapFree(GetProcessHeap(), 0, (*es).first_line_def as _);
    HeapFree(GetProcessHeap(), 0, (*es).undo_text as _);
    if !(*es).hloc32W.is_null() {
        LocalFree((*es).hloc32W);
    }
    HeapFree(GetProcessHeap(), 0, (*es).logAttr as _);
    HeapFree(GetProcessHeap(), 0, es as _);
    0 as LRESULT
}

unsafe fn EDIT_WM_Create(es: *mut EDITSTATE, name: LPCWSTR) -> LRESULT {
    let mut clientRect: RECT = core::mem::zeroed();

    EDIT_WM_SetFont(es, core::ptr::null_mut(), 0);
    EDIT_EM_EmptyUndoBuffer(es);

    GetClientRect((*es).hwndSelf, &mut clientRect);
    EDIT_SetRectNP(es, &clientRect);

    if !name.is_null() && *name != 0 {
        EDIT_EM_ReplaceSel(es, 0, name, 0, 0);
        (*es).selection_end = 0;
        (*es).selection_start = 0;
        EDIT_EM_ScrollCaret(es);
    }
    EDIT_UpdateScrollInfo(es);
    1
}

unsafe fn EDIT_WM_NCDestroy(es: *mut EDITSTATE) -> LRESULT {
    let mut pc: *mut LINEDEF;
    let pp: *mut LINEDEF;

    if !(*es).hloc32W.is_null() && (*es).hloc32W != (*es).hlocapp {
        LocalFree((*es).hloc32W);
    }
    if !(*es).hloc32A.is_null() && (*es).hloc32A != (*es).hlocapp {
        LocalFree((*es).hloc32A);
    }
    EDIT_InvalidateUniscribeData(es);
    pc = (*es).first_line_def;
    while !pc.is_null() {
        let pp = (*pc).next;
        HeapFree(GetProcessHeap(), 0, pc as *mut core::ffi::c_void);
        pc = pp;
    }

    SetWindowLongPtrW((*es).hwndSelf, 0, 0);
    HeapFree(GetProcessHeap(), 0, (*es).undo_text as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, es as *mut core::ffi::c_void);

    0
}

unsafe fn DefWindowProcT(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, unicode: BOOL) -> LRESULT {
    if unicode != 0 {
        DefWindowProcW(hwnd, msg, wParam, lParam)
    } else {
        DefWindowProcA(hwnd, msg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn EditWndProc_common(hwnd: HWND, msg: UINT, mut wParam: WPARAM, mut lParam: LPARAM, unicode: BOOL) -> LRESULT {
    let mut es: *mut EDITSTATE = GetWindowLongPtrW(hwnd, 0) as *mut EDITSTATE;
    let mut result: LRESULT = 0;

    let pWnd: PWND = ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_EDIT as u16);
        } else {
            if (*pWnd).fnid != FNID_EDIT {
                ERR!("Wrong window class for Edit! fnId 0x%x\n", (*pWnd).fnid as u32);
                return 0;
            }
        }
    }

    TRACE!(b"hwnd=%p msg=%x (%s) wparam=%lx lparam=%lx\n\0".as_ptr() as *const i8,
          hwnd, msg, SPY_GetMsgName(msg, hwnd), wParam, lParam);

    if es.is_null() && msg != WM_NCCREATE {
        return DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
    }

    if !es.is_null() && msg != WM_NCDESTROY {
        EDIT_LockBuffer(es);
    }

    match msg {
        EM_GETSEL => {
            result = EDIT_EM_GetSel(es, wParam as PUINT, lParam as PUINT);
        }
        EM_SETSEL => {
            EDIT_EM_SetSel(es, wParam as UINT, lParam as UINT, 0);
            EDIT_EM_ScrollCaret(es);
            result = 1;
        }
        EM_GETRECT => {
            if lParam != 0 {
                CopyRect(lParam as LPRECT, core::ptr::addr_of!((*es).format_rect));
            }
        }
        EM_SETRECT => {
            if ((*es).style & ES_MULTILINE) != 0 && lParam != 0 {
                EDIT_SetRectNP(es, lParam as LPRECT);
                EDIT_UpdateText(es, core::ptr::null_mut(), 1);
            }
        }
        EM_SETRECTNP => {
            if ((*es).style & ES_MULTILINE) != 0 && lParam != 0 {
                EDIT_SetRectNP(es, lParam as LPRECT);
            }
        }
        EM_SCROLL => {
            result = EDIT_EM_Scroll(es, wParam as INT);
        }
        EM_LINESCROLL => {
            result = EDIT_EM_LineScroll(es, wParam as INT, lParam as INT) as LRESULT;
        }
        EM_SCROLLCARET => {
            EDIT_EM_ScrollCaret(es);
            result = 1;
        }
        EM_GETMODIFY => {
            result = (((*es).flags & EF_MODIFIED) != 0) as LRESULT;
        }
        EM_SETMODIFY => {
            if wParam != 0 {
                (*es).flags |= EF_MODIFIED;
            } else {
                (*es).flags &= !((EF_MODIFIED | EF_UPDATE) as u16);
            }
        }
        EM_GETLINECOUNT => {
            result = if ((*es).style & ES_MULTILINE) != 0 { (*es).line_count as LRESULT } else { 1 };
        }
        EM_LINEINDEX => {
            result = EDIT_EM_LineIndex(es, wParam as INT) as LRESULT;
        }
        EM_SETHANDLE => {
            EDIT_EM_SetHandle(es, wParam as HLOCAL);
        }
        EM_GETHANDLE => {
            result = EDIT_EM_GetHandle(es) as LRESULT;
        }
        EM_GETTHUMB => {
            result = EDIT_EM_GetThumb(es);
        }
        0x00bf | 0x00c0 | 0x00c3 | 0x00ca => {
            FIXME!("undocumented message 0x%x, please report\n", msg);
            result = DefWindowProcW(hwnd, msg, wParam, lParam);
        }
        EM_LINELENGTH => {
            result = EDIT_EM_LineLength(es, wParam as INT) as LRESULT;
        }
        EM_REPLACESEL => {
            'block: {
                let textW: LPWSTR;
                if unicode != 0 {
                    textW = lParam as LPWSTR;
                } else {
                    let textA = lParam as LPSTR;
                    let countW = MultiByteToWideChar(CP_ACP, 0, textA, -1, core::ptr::null_mut(), 0);
                    textW = HeapAlloc(GetProcessHeap(), 0, ((countW as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                    if textW.is_null() {
                        break 'block;
                    }
                    MultiByteToWideChar(CP_ACP, 0, textA, -1, textW, countW);
                }

                EDIT_EM_ReplaceSel(es, wParam as BOOL, textW, 1, 1);
                result = 1;

                if unicode == 0 {
                    HeapFree(GetProcessHeap(), 0, textW as *mut core::ffi::c_void);
                }
            }
        }
        EM_GETLINE => {
            result = EDIT_EM_GetLine(es, wParam as INT, lParam as LPWSTR, unicode) as LRESULT;
        }
        EM_SETLIMITTEXT => {
            EDIT_EM_SetLimitText(es, wParam as UINT);
        }
        EM_CANUNDO => {
            result = EDIT_EM_CanUndo(es) as LRESULT;
        }
        EM_UNDO | WM_UNDO => {
            result = EDIT_EM_Undo(es) as LRESULT;
        }
        EM_FMTLINES => {
            result = EDIT_EM_FmtLines(es, wParam as BOOL) as LRESULT;
        }
        EM_LINEFROMCHAR => {
            result = EDIT_EM_LineFromChar(es, wParam as INT) as LRESULT;
        }
        EM_SETTABSTOPS => {
            result = EDIT_EM_SetTabStops(es, wParam as INT, lParam as LPINT) as LRESULT;
        }
        EM_SETPASSWORDCHAR => {
            let mut charW: WCHAR = 0;
            if unicode != 0 {
                charW = wParam as WCHAR;
            } else {
                let charA: CHAR = wParam as CHAR;
                MultiByteToWideChar(CP_ACP, 0, core::ptr::addr_of!(charA), 1, core::ptr::addr_of_mut!(charW), 1);
            }
            EDIT_EM_SetPasswordChar(es, charW);
        }
        EM_EMPTYUNDOBUFFER => {
            EDIT_EM_EmptyUndoBuffer(es);
        }
        EM_GETFIRSTVISIBLELINE => {
            result = if ((*es).style & ES_MULTILINE) != 0 { (*es).y_offset as LRESULT } else { (*es).x_offset as LRESULT };
        }
        EM_SETREADONLY => {
            let old_style: DWORD = (*es).style;
            if wParam != 0 {
                SetWindowLongW(hwnd, GWL_STYLE, GetWindowLongW(hwnd, GWL_STYLE) | ES_READONLY as i32);
                (*es).style |= ES_READONLY;
            } else {
                SetWindowLongW(hwnd, GWL_STYLE, GetWindowLongW(hwnd, GWL_STYLE) & !(ES_READONLY as i32));
                (*es).style &= !ES_READONLY;
            }
            if (old_style ^ (*es).style) != 0 {
                InvalidateRect((*es).hwndSelf, core::ptr::null(), 1);
            }
            result = 1;
        }
        EM_SETWORDBREAKPROC => {
            EDIT_EM_SetWordBreakProc(es, lParam as *mut core::ffi::c_void);
        }
        EM_GETWORDBREAKPROC => {
            result = (*es).word_break_proc as LRESULT;
        }
        EM_GETPASSWORDCHAR => {
            if unicode != 0 {
                result = (*es).password_char as LRESULT;
            } else {
                let charW: WCHAR = (*es).password_char;
                let mut charA: CHAR = 0;
                WideCharToMultiByte(CP_ACP, 0, core::ptr::addr_of!(charW), 1, core::ptr::addr_of_mut!(charA), 1, core::ptr::null(), core::ptr::null_mut());
                result = charA as LRESULT;
            }
        }
        EM_SETMARGINS => {
            EDIT_EM_SetMargins(es, wParam as INT, LOWORD(lParam as DWORD), HIWORD(lParam as DWORD), 1);
        }
        EM_GETMARGINS => {
            result = MAKELONG((*es).left_margin as WORD, (*es).right_margin as WORD) as LRESULT;
        }
        EM_GETLIMITTEXT => {
            result = (*es).buffer_limit as LRESULT;
        }
        EM_POSFROMCHAR => {
            if (wParam as INT) >= get_text_length(es) as INT {
                result = -1;
            } else {
                result = EDIT_EM_PosFromChar(es, wParam as INT, 0);
            }
        }
        EM_CHARFROMPOS => {
            result = EDIT_EM_CharFromPos(es, LOWORD(lParam as DWORD) as i16 as INT, HIWORD(lParam as DWORD) as i16 as INT);
        }
        WM_NCCREATE => {
            result = EDIT_WM_NCCreate(hwnd, lParam as LPCREATESTRUCTW, unicode);
        }
        WM_NCDESTROY => {
            result = EDIT_WM_NCDestroy(es);
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
            es = core::ptr::null_mut();
        }
        WM_GETDLGCODE => {
            result = (DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS) as LRESULT;

            if ((*es).style & ES_MULTILINE) != 0 {
                result |= DLGC_WANTALLKEYS as LRESULT;
            }

            if lParam != 0 {
                (*es).flags |= EF_DIALOGMODE as u16;

                if (*(lParam as LPMSG)).message == WM_KEYDOWN {
                    let vk = (*(lParam as LPMSG)).wParam as i32;
                    if !(*es).hwndListBox.is_null() {
                        if vk == VK_RETURN as i32 || vk == VK_ESCAPE as i32 {
                            if SendMessageW(GetParent(hwnd), CB_GETDROPPEDSTATE, 0, 0) != 0 {
                                result |= DLGC_WANTMESSAGE as LRESULT;
                            }
                        }
                    }
                }
            }
        }
        WM_IME_CHAR => {
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_CHAR => {
            'block: {
                let charW: WCHAR;
                if unicode != 0 {
                    charW = wParam as WCHAR;
                } else {
                    let charA: CHAR = wParam as CHAR;
                    let mut tmp: WCHAR = 0;
                    MultiByteToWideChar(CP_ACP, 0, core::ptr::addr_of!(charA), 1, core::ptr::addr_of_mut!(tmp), 1);
                    charW = tmp;
                }

                if !(*es).hwndListBox.is_null() {
                    if charW as u32 == VK_RETURN || charW as u32 == VK_ESCAPE {
                        if SendMessageW(GetParent(hwnd), CB_GETDROPPEDSTATE, 0, 0) != 0 {
                            SendMessageW(GetParent(hwnd), WM_KEYDOWN, charW as WPARAM, 0);
                        }
                        break 'block;
                    }
                }
                result = EDIT_WM_Char(es, charW);
            }
        }
        WM_UNICHAR => {
            if unicode != 0 {
                if wParam == UNICODE_NOCHAR as WPARAM {
                    return 1;
                }
                if wParam <= 0x000fffff {
                    if wParam > 0xffff {
                        wParam -= 0x10000;
                        EDIT_WM_Char(es, ((wParam >> 10) + 0xd800) as WCHAR);
                        EDIT_WM_Char(es, ((wParam & 0x03ff) + 0xdc00) as WCHAR);
                    } else {
                        EDIT_WM_Char(es, wParam as WCHAR);
                    }
                }
                return 0;
            }
        }
        WM_CLEAR => {
            EDIT_WM_Clear(es);
        }
        WM_CONTEXTMENU => {
            EDIT_WM_ContextMenu(es, LOWORD(lParam as DWORD) as i16 as INT, HIWORD(lParam as DWORD) as i16 as INT);
        }
        WM_COPY => {
            EDIT_WM_Copy(es);
        }
        WM_CREATE => {
            if unicode != 0 {
                result = EDIT_WM_Create(es, (*(lParam as LPCREATESTRUCTW)).lpszName);
            } else {
                let nameA: LPCSTR = (*(lParam as LPCREATESTRUCTA)).lpszName;
                let mut nameW: LPWSTR = core::ptr::null_mut();
                if !nameA.is_null() {
                    let countW = MultiByteToWideChar(CP_ACP, 0, nameA, -1, core::ptr::null_mut(), 0);
                    nameW = HeapAlloc(GetProcessHeap(), 0, ((countW as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                    if !nameW.is_null() {
                        MultiByteToWideChar(CP_ACP, 0, nameA, -1, nameW, countW);
                    }
                }
                result = EDIT_WM_Create(es, nameW);
                HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
            }
        }
        WM_CUT => {
            EDIT_WM_Cut(es);
        }
        WM_ENABLE => {
            (*es).bEnableState = wParam as BOOL;
            EDIT_UpdateText(es, core::ptr::null_mut(), 1);
        }
        WM_ERASEBKGND => {
            result = 1;
        }
        WM_GETFONT => {
            result = (*es).font as LRESULT;
        }
        WM_GETTEXT => {
            result = EDIT_WM_GetText(es, wParam as INT, lParam as LPWSTR, unicode) as LRESULT;
        }
        WM_GETTEXTLENGTH => {
            if unicode != 0 {
                result = get_text_length(es) as LRESULT;
            } else {
                result = WideCharToMultiByte(CP_ACP, 0, (*es).text, get_text_length(es) as i32,
                                             core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as LRESULT;
            }
        }
        WM_HSCROLL => {
            result = EDIT_WM_HScroll(es, LOWORD(wParam as DWORD) as INT, HIWORD(wParam as DWORD) as i16 as INT);
        }
        WM_KEYDOWN => {
            result = EDIT_WM_KeyDown(es, wParam as INT);
        }
        WM_KILLFOCUS => {
            result = EDIT_WM_KillFocus(es);
        }
        WM_LBUTTONDBLCLK => {
            result = EDIT_WM_LButtonDblClk(es);
        }
        WM_LBUTTONDOWN => {
            result = EDIT_WM_LButtonDown(es, wParam as DWORD, LOWORD(lParam as DWORD) as i16 as INT, HIWORD(lParam as DWORD) as i16 as INT);
        }
        WM_LBUTTONUP => {
            result = EDIT_WM_LButtonUp(es);
        }
        WM_MBUTTONDOWN => {
            result = EDIT_WM_MButtonDown(es);
        }
        WM_MOUSEMOVE => {
            result = EDIT_WM_MouseMove(es, LOWORD(lParam as DWORD) as i16 as INT, HIWORD(lParam as DWORD) as i16 as INT);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            EDIT_WM_Paint(es, wParam as HDC);
        }
        WM_PASTE => {
            EDIT_WM_Paste(es);
        }
        WM_SETFOCUS => {
            EDIT_WM_SetFocus(es);
        }
        WM_SETFONT => {
            EDIT_WM_SetFont(es, wParam as HFONT, (LOWORD(lParam as DWORD) != 0) as BOOL);
        }
        WM_SETREDRAW => {
        }
        WM_SETTEXT => {
            EDIT_WM_SetText(es, lParam as LPCWSTR, unicode);
            result = 1;
        }
        WM_SIZE => {
            EDIT_WM_Size(es, wParam as UINT);
        }
        WM_STYLECHANGED => {
            result = EDIT_WM_StyleChanged(es, wParam, lParam as *const STYLESTRUCT);
        }
        WM_STYLECHANGING => {
            result = 0;
        }
        WM_SYSKEYDOWN => {
            result = EDIT_WM_SysKeyDown(es, wParam as INT, lParam as DWORD);
        }
        WM_TIMER => {
            EDIT_WM_Timer(es);
        }
        WM_VSCROLL => {
            /* Upstream 1dc2e6de728 (CORE-19305): wParam's HIWORD is only 16 bits, so a
             * thumb position past line 32767 wrapped negative. Take the real 32-bit
             * tracking position from the scroll bar instead. */
            let mut si: SCROLLINFO = core::mem::zeroed();
            si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
            si.fMask = SIF_TRACKPOS;
            if GetScrollInfo(hwnd, SB_VERT as INT, &mut si) == 0 {
                result = 1;
            } else {
                result = EDIT_WM_VScroll(es, LOWORD(wParam as DWORD) as INT, si.nTrackPos);
            }
        }
        WM_MOUSEWHEEL => {
            'block: {
                let wheelDelta: i32;
                let mut pulScrollLines: UINT = 3;
                SystemParametersInfoW(SPI_GETWHEELSCROLLLINES, 0, core::ptr::addr_of_mut!(pulScrollLines) as *mut core::ffi::c_void, 0);

                if (wParam & (MK_SHIFT | MK_CONTROL) as WPARAM) != 0 {
                    result = DefWindowProcW(hwnd, msg, wParam, lParam);
                    break 'block;
                }
                wheelDelta = GET_WHEEL_DELTA_WPARAM(wParam);
                if (wheelDelta < 0 && (*es).wheelDeltaRemainder < 0) ||
                   (wheelDelta > 0 && (*es).wheelDeltaRemainder > 0) {
                    (*es).wheelDeltaRemainder += wheelDelta;
                } else {
                    (*es).wheelDeltaRemainder = wheelDelta;
                }
                if (*es).wheelDeltaRemainder != 0 && pulScrollLines != 0 {
                    let cLineScroll: i32;
                    pulScrollLines = min((*es).line_count as UINT, pulScrollLines);
                    cLineScroll = (pulScrollLines as f32 * (*es).wheelDeltaRemainder as f32 / WHEEL_DELTA as f32) as i32;
                    (*es).wheelDeltaRemainder -= WHEEL_DELTA as i32 * cLineScroll / pulScrollLines as i32;
                    result = EDIT_EM_LineScroll(es, 0, -cLineScroll) as LRESULT;
                }
            }
        }
        WM_IME_SETCONTEXT => {
            let hKL: HKL = GetKeyboardLayout(0);

            if PRIMARYLANGID(LOWORD(hKL as DWORD)) == LANG_KOREAN as u16 {
                lParam &= !(ISC_SHOWUICOMPOSITIONWINDOW as LPARAM);
            }

            if wParam != 0 {
                let hIMC: HIMC = ImmGetContext(hwnd);
                let pIC: LPINPUTCONTEXTDX = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
                if !pIC.is_null() {
                    (*pIC).dwUIFlags &= !0x40000;
                    ImmUnlockIMC(hIMC);
                }
                if (*GetWin32ClientInfo()).CI_flags & CI_WOW as ULONG_PTR != 0 {
                    ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_CANCEL, 0);
                }
                ImmReleaseContext(hwnd, hIMC);
            }

            result = DefWindowProcT(hwnd, WM_IME_SETCONTEXT, wParam, lParam, unicode);
        }
        WM_IME_STARTCOMPOSITION => {
            let hKL: HKL = GetKeyboardLayout(0);

            if PRIMARYLANGID(LOWORD(hKL as DWORD)) == LANG_KOREAN as u16 {
                return 1;
            }

            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_IME_COMPOSITION => {
            EDIT_ImeComposition(hwnd, lParam, es);
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_IME_ENDCOMPOSITION => {
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_IME_COMPOSITIONFULL => {
        }
        WM_IME_SELECT => {
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_IME_CONTROL => {
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
        WM_IME_REQUEST => {
            match wParam {
                x if x == IMR_QUERYCHARPOSITION as WPARAM => {
                    let pos: LRESULT;
                    let chpos = lParam as *mut IMECHARPOSITION;

                    pos = EDIT_EM_PosFromChar(es, (*es).selection_start + (*chpos).dwCharPos as INT, 0);
                    (*chpos).pt.x = LOWORD(pos as DWORD) as i32;
                    (*chpos).pt.y = HIWORD(pos as DWORD) as i32;
                    (*chpos).cLineHeight = (*es).line_height as u32;
                    (*chpos).rcDocument = core::mem::transmute_copy(&(*es).format_rect);
                    MapWindowPoints(hwnd, core::ptr::null_mut(), core::ptr::addr_of_mut!((*chpos).pt) as *mut POINT, 1);
                    MapWindowPoints(hwnd, core::ptr::null_mut(), core::ptr::addr_of_mut!((*chpos).rcDocument) as *mut POINT, 2);
                    result = 1;
                }
                _ => {}
            }
        }
        _ => {
            result = DefWindowProcT(hwnd, msg, wParam, lParam, unicode);
        }
    }

    if IsWindow(hwnd) != 0 && !es.is_null() && msg != EM_GETHANDLE && GetWindowLongPtrW(hwnd, 0) != 0 {
        EDIT_UnlockBuffer(es, 0);
    }

    TRACE!(b"hwnd=%p msg=%x (%s) -- 0x%08lx\n\0".as_ptr() as *const i8, hwnd, msg, SPY_GetMsgName(msg, hwnd), result);

    result
}

#[no_mangle]
pub unsafe extern "system" fn EditWndProcA(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    EditWndProc_common(hWnd, uMsg, wParam, lParam, 0u8 as BOOL)
}

#[no_mangle]
pub unsafe extern "system" fn EditWndProcW(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    EditWndProc_common(hWnd, uMsg, wParam, lParam, 1)
}

// ─── EDIT_builtin_class — cross-TU data symbol read by controls/regcontrol.c ───
// Faithful to controls/edit.c:5532 (extra = sizeof(EDITSTATE*) [+ sizeof(WORD) on !_WIN64]).
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

#[cfg(target_arch = "x86")]
const EDIT_BUILTIN_EXTRA: INT =
    (core::mem::size_of::<PEDITSTATE>() + core::mem::size_of::<u16>()) as INT;
#[cfg(not(target_arch = "x86"))]
const EDIT_BUILTIN_EXTRA: INT = core::mem::size_of::<PEDITSTATE>() as INT;

#[no_mangle]
pub static EDIT_builtin_class: builtin_class_descr = builtin_class_descr {
    name: editW.as_ptr(),
    style: CS_DBLCLKS | CS_PARENTDC,
    procA: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(EditWndProcA)
    },
    procW: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(EditWndProcW)
    },
    extra: EDIT_BUILTIN_EXTRA,
    cursor: 32513 as LPCWSTR, // IDC_IBEAM
    brush: 0 as HBRUSH,
};

// ─── IMM dynamic dispatch (controls/edit.c: IMM_FN(name) == gImmApiEntries.pname) ─────
// imm32 entry points live in the runtime-loaded global `gImmApiEntries` (Imm32ApiTable)
// in the still-C misc/imm.c; user32 does NOT import imm32 directly. Mirror the table only
// up to the highest field edit uses — ImmUnlockIMC, index 28 (immtable.h order); earlier
// and unused slots are pointer-sized pads so the used fields land at the correct offsets.
#[repr(C)]
struct Imm32ApiTable {
    _pad0_13: [*mut c_void; 14], // 0..13: ImmWINNLS*/ImmSendIMEMsg*/ImmIMP*/ImmAssoc/ImmEscape*/GetCompStrA
    pImmGetCompositionStringW:
        Option<unsafe extern "system" fn(HIMC, DWORD, *mut c_void, DWORD) -> LONG>, // 14
    _pad15: *mut c_void,                                                             // 15 GetCompWindow
    pImmGetContext: Option<unsafe extern "system" fn(HWND) -> HIMC>,                 // 16
    _pad17: *mut c_void,                                                             // 17 GetDefaultIMEWnd
    pImmIsIME: Option<unsafe extern "system" fn(HKL) -> BOOL>,                       // 18
    pImmReleaseContext: Option<unsafe extern "system" fn(HWND, HIMC) -> BOOL>,       // 19
    _pad20_22: [*mut c_void; 3], // 20..22 RegisterClient/GetCompFontW/GetCompFontA
    pImmSetCompositionFontW: Option<unsafe extern "system" fn(HIMC, *mut LOGFONTW) -> BOOL>, // 23
    _pad24: *mut c_void,                                                             // 24 SetCompFontA
    pImmSetCompositionWindow:
        Option<unsafe extern "system" fn(HIMC, *mut COMPOSITIONFORM) -> BOOL>, // 25
    pImmNotifyIME: Option<unsafe extern "system" fn(HIMC, DWORD, DWORD, DWORD_PTR) -> BOOL>, // 26
    pImmLockIMC: Option<unsafe extern "system" fn(HIMC) -> *mut c_void>,             // 27
    pImmUnlockIMC: Option<unsafe extern "system" fn(HIMC) -> BOOL>,                  // 28
}
extern "C" {
    static gImmApiEntries: Imm32ApiTable;
}
#[inline]
unsafe fn ImmGetContext(hwnd: HWND) -> HIMC {
    (gImmApiEntries.pImmGetContext.unwrap())(hwnd)
}
#[inline]
unsafe fn ImmReleaseContext(hwnd: HWND, h: HIMC) -> BOOL {
    (gImmApiEntries.pImmReleaseContext.unwrap())(hwnd, h)
}
#[inline]
unsafe fn ImmIsIME(hkl: HKL) -> BOOL {
    (gImmApiEntries.pImmIsIME.unwrap())(hkl)
}
#[inline]
unsafe fn ImmSetCompositionWindow(h: HIMC, f: *mut COMPOSITIONFORM) -> BOOL {
    (gImmApiEntries.pImmSetCompositionWindow.unwrap())(h, f)
}
#[inline]
unsafe fn ImmSetCompositionFontW(h: HIMC, lf: *mut LOGFONTW) -> BOOL {
    (gImmApiEntries.pImmSetCompositionFontW.unwrap())(h, lf)
}
#[inline]
unsafe fn ImmGetCompositionStringW(h: HIMC, idx: DWORD, buf: *mut c_void, len: DWORD) -> LONG {
    (gImmApiEntries.pImmGetCompositionStringW.unwrap())(h, idx, buf, len)
}
#[inline]
unsafe fn ImmLockIMC(h: HIMC) -> *mut c_void {
    (gImmApiEntries.pImmLockIMC.unwrap())(h)
}
#[inline]
unsafe fn ImmUnlockIMC(h: HIMC) -> BOOL {
    (gImmApiEntries.pImmUnlockIMC.unwrap())(h)
}
#[inline]
unsafe fn ImmNotifyIME(h: HIMC, action: DWORD, idx: DWORD, val: DWORD) -> BOOL {
    (gImmApiEntries.pImmNotifyIME.unwrap())(h, action, idx, val as DWORD_PTR)
}
