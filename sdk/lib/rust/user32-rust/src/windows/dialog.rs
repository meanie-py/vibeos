use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::cursoricon::FindResourceW;
use crate::windows::window::HeapAlloc;
use crate::windows::cursoricon::LoadResource;
// The crate's own functions: the definition is the declaration.
use crate::misc::display::MonitorFromRect;

use core::ffi::c_void;

// ValidateHwnd lives in crate::internal (correctly fastcall-split on i386).
// LOWORD/HIWORD/MAKELONG/MAKEWPARAM are macros — use crate::internal::*.
use crate::internal::{ValidateHwnd, HIWORD, LOWORD, MAKELONG, MAKEWPARAM};
// NtUserx*/GetControlColor and the EXTINLINE ntwrapper.h wrappers (GetDC, SetFocus,
// ShowWindow, SetWindowPos, DestroyWindow, DestroyMenu, SetActiveWindow,
// GetForegroundWindow, NtUserxSetDialogPointer) come from crate::ntwrapper.
use crate::ntwrapper::*;
// GetWindowLong{,Ptr}{A,W} / SetWindowLongPtr{A,W} are user32's own exports defined
// in class.rs (i386 winuser.h macro forms over *LongW; LONG_PTR return matches the
// DWLP_DLGPROC transmute). Call via the crate path (no extern decl).
use crate::windows::class::{GetWindowLongA, GetWindowLongPtrA, GetWindowLongPtrW, GetWindowLongW};
// SetWindowLongPtr{A,W} are _WIN64-only in C (class.rs gates them off on x86). On
// Win32 the winuser.h `SetWindowLongPtr` macro expands to `SetWindowLong`, so on x86
// forward to SetWindowLong{A,W} (LONG_PTR == LONG == i32) to stay faithful.
#[cfg(not(target_arch = "x86"))]
use crate::windows::class::{SetWindowLongPtrA, SetWindowLongPtrW};
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrA(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: isize) -> isize {
    crate::windows::class::SetWindowLongA(hWnd, nIndex, dwNewLong as core::ffi::c_long) as isize
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: isize) -> isize {
    crate::windows::class::SetWindowLongW(hWnd, nIndex, dwNewLong as core::ffi::c_long) as isize
}

// CRT (msvcrt).
extern "C" {
    fn strtol(nptr: *const core::ffi::c_char, endptr: *mut *mut core::ffi::c_char, base: core::ffi::c_int) -> core::ffi::c_long;
    fn strtoul(nptr: *const core::ffi::c_char, endptr: *mut *mut core::ffi::c_char, base: core::ffi::c_int) -> core::ffi::c_ulong;
    fn sprintf(str_: *mut core::ffi::c_char, format: *const core::ffi::c_char, ...) -> core::ffi::c_int;
    fn wcslen(s: *const u16) -> usize;
}

// user32-internal FASTCALL helper (user32p.h). i386 uses the @Name@N decoration;
// amd64/arm64 degrade FASTCALL to the C ABI.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn TestWindowProcess(pWnd: PWND) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn TestWindowProcess(pWnd: PWND) -> BOOL;
}

// user32-internal cdecl helper (windows/mdi.c).
extern "C" {
    fn WIN_ListChildren(hWndparent: HWND) -> *mut HWND;
}

// kernel32 imports (extern "system").
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: LPVOID) -> i32;
    fn SetLastError(dwErrCode: DWORD);
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: LPCSTR, cbMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32) -> i32;
    fn WideCharToMultiByte(CodePage: u32, dwFlags: u32, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut i32) -> i32;
    pub fn FindResourceA(hModule: HINSTANCE, lpName: LPCSTR, lpType: LPCSTR) -> HRSRC;
    fn SetCurrentDirectoryW(lpPathName: LPCWSTR) -> BOOL;
    fn GetCurrentDirectoryW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD;
}

// gdi32 imports (extern "system").
extern "system" {
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
    fn GetDeviceCaps(hdc: HDC, index: core::ffi::c_int) -> core::ffi::c_int;
    pub fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    fn CreateFontW(cHeight: INT, cWidth: INT, cEscapement: INT, cOrientation: INT, cWeight: INT, bItalic: DWORD, bUnderline: DWORD, bStrikeOut: DWORD, iCharSet: DWORD, iOutPrecision: DWORD, iClipPrecision: DWORD, iQuality: DWORD, iPitchAndFamily: DWORD, pszFaceName: LPCWSTR) -> HFONT;
    fn SelectObject(hdc: HDC, h: LPVOID) -> LPVOID;
    fn DeleteObject(ho: LPVOID) -> BOOL;
    fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: INT) -> BOOL;
}

// user32 public exports referenced here (extern "system"; resolve to user32's own
// export table / sibling object files within this DLL).
extern "system" {
    fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, nWidth: core::ffi::c_int, nHeight: core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, x: core::ffi::c_int, y: core::ffi::c_int, nWidth: core::ffi::c_int, nHeight: core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn PostQuitMessage(nExitCode: INT);
    fn GetMessageW(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn PeekMessageW(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn CallMsgFilterW(lpMsg: LPMSG, nCode: INT) -> BOOL;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn CallWindowProcA(lpPrevWndFunc: WNDPROC, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn CallWindowProcW(lpPrevWndFunc: WNDPROC, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetWindowTextA(hWnd: HWND, lpString: LPSTR, nMaxCount: core::ffi::c_int) -> core::ffi::c_int;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: core::ffi::c_int) -> core::ffi::c_int;
    fn SetWindowTextA(hWnd: HWND, lpString: LPCSTR) -> BOOL;
    fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    fn IsIconic(hwnd: HWND) -> BOOL;
    fn IsWindow(hwnd: HWND) -> BOOL;
    fn IsChild(hWndParent: HWND, hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn GetFocus() -> HWND;
    fn GetParent(hWnd: HWND) -> HWND;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn GetAncestor(hwnd: HWND, gaFlags: UINT) -> HWND;
    fn GetDesktopWindow() -> HWND;
    fn GetActiveWindow() -> HWND;
    fn GetCapture() -> HWND;
    fn GetKeyState(nVirtKey: INT) -> SHORT;
    fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    fn EnumChildWindows(hWndParent: HWND, lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
    fn ClientToScreen(hWnd: HWND, lpPoint: LPPOINT) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn SetWindowContextHelpId(hwnd: HWND, ctx: DWORD) -> BOOL;
    fn AdjustWindowRectEx(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
    fn SetRect(lprc: LPRECT, xLeft: INT, yTop: INT, xRight: INT, yBottom: INT) -> BOOL;
    fn GetCursorPos(lpPoint: LPPOINT) -> BOOL;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    fn MonitorFromWindow(hwnd: HWND, dwFlags: DWORD) -> HMONITOR;
    fn MonitorFromPoint(pt: POINT, dwFlags: DWORD) -> HMONITOR;
    fn GetMonitorInfoW(hMonitor: HMONITOR, lpmi: LPMONITORINFO) -> BOOL;
    fn LoadMenuW(hInstance: HINSTANCE, lpMenuName: LPCWSTR) -> HMENU;
    fn UpdateWindow(hWnd: HWND) -> BOOL;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn CharUpperW(lpsz: LPWSTR) -> LPWSTR;
    fn CharLowerW(lpsz: LPWSTR) -> LPWSTR;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn MulDiv(nNumber: INT, nNumerator: INT, nDenominator: INT) -> INT;
}

// wsprintfW is WINAPIV (__cdecl variadic) in winuser.h — C-variadics are illegal
// with the "system"/stdcall ABI, so it must live in an extern "C" block.
extern "C" {
    fn wsprintfW(buf: LPWSTR, fmt: LPCWSTR, ...) -> INT;
}

// windows/hook.c: `VOID FASTCALL IntNotifyWinEvent(...)`. i386 = @Name@N decoration.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}

// winuser.h: DWLP_DLGPROC = DWLP_MSGRESULT + sizeof(LRESULT); DWLP_MSGRESULT = 0.
// LRESULT is pointer-sized in the real ABI (4 on i386, 8 on amd64/arm64); use the
// pointer size directly since win32k-sys aliases LONG_PTR/LRESULT to 32-bit c_long.
const DWLP_DLGPROC: INT = core::mem::size_of::<*const core::ffi::c_void>() as INT;

// --- consts not present in win32k-sys (faithful to the SDK headers) ---
const CP_ACP: u32 = 0; // winnls.h
const WM_CTLCOLOR: UINT = 0x0019; // windowsx.h
const WC_DIALOG: usize = 0x8002; // winuser.h MAKEINTATOM(0x8002)
const RT_DIALOG: usize = 5; // winuser.h MAKEINTRESOURCE(5)
const CW_USEDEFAULT: INT = 0x8000_0000u32 as i32; // winuser.h ((int)0x80000000)
const CW_USEDEFAULT16: i32 = 0x8000; // wine ((short)0x8000)
const HWND_TOPMOST: HWND = -1isize as HWND; // winuser.h ((HWND)-1)
const HWND_BROADCAST: HWND = 0xffff as HWND; // winuser.h ((HWND)0xffff)

/// `GETDLGINFO(hwnd)` — user32 dialog.c macro: `DIALOG_get_info(hwnd, FALSE)`.
#[inline]
unsafe fn GETDLGINFO(hwnd: HWND) -> *mut DIALOGINFO {
    DIALOG_get_info(hwnd, 0 as BOOL)
}

// strlenW(s) == wcslen(s) (wine/unicode.h).
#[inline]
unsafe fn strlenW(s: *const u16) -> usize {
    wcslen(s)
}

// IS_INTRESOURCE — wine/winuser macro: a pointer whose high bits are clear
// (value < 0x10000) is an int-resource.
#[inline]
unsafe fn IS_INTRESOURCE(p: LPCVOID) -> bool {
    (p as usize) >> 16 == 0
}

// strchrW / strrchrW — wine/unicode.h UTF-16 char search.
#[inline]
unsafe fn strchrW(mut str: *const u16, ch: core::ffi::c_int) -> *mut u16 {
    loop {
        if *str as core::ffi::c_int == ch {
            return str as *mut u16;
        }
        if *str == 0 {
            return core::ptr::null_mut();
        }
        str = str.add(1);
    }
}
#[inline]
unsafe fn strrchrW(str: *const u16, ch: core::ffi::c_int) -> *mut u16 {
    let mut ret: *mut u16 = core::ptr::null_mut();
    let mut p = str;
    loop {
        if *p as core::ffi::c_int == ch {
            ret = p as *mut u16;
        }
        if *p == 0 {
            return ret;
        }
        p = p.add(1);
    }
}

// toupperW — wine/unicode.h. ASCII-fold uppercase (faithful to the C helper's
// ASCII fast-path used by these dialog accelerator comparisons).
#[inline]
unsafe fn toupperW(ch: core::ffi::c_int) -> core::ffi::c_int {
    if (b'a' as core::ffi::c_int..=b'z' as core::ffi::c_int).contains(&ch) {
        ch - 0x20
    } else {
        ch
    }
}

// WINE_DEFAULT_DEBUG_CHANNEL — ERR/TRACE compile to no-ops (discard args, like
// window.rs / message.rs). debugstr_w args are therefore never evaluated.

// wine debug helper — formats a UNICODE_STRING for the trace channels. Args are
// discarded by the no-op TRACE macro above, so this never actually runs.
#[inline]
unsafe fn debugstr_w(_s: LPCWSTR) -> *const i8 {
    b"\0".as_ptr() as *const i8
}


// ---- file-scope preamble (translated) ----
pub const DF_END: u32 = 0x0001;

pub const DF_DIALOGACTIVE: u32 = 0x4000;

pub const DLG_ISANSI: u32 = 2;

#[repr(C)]
pub struct DIALOGINFO {
    pub hwndFocus: HWND,
    pub hUserFont: HFONT,
    pub hMenu: HMENU,
    pub xBaseUnit: UINT,
    pub yBaseUnit: UINT,
    pub idResult: INT,
    pub flags: UINT,
}
pub type PDIALOGINFO = *mut DIALOGINFO;

#[repr(C)]
pub struct DLG_CONTROL_INFO {
    pub style: DWORD,
    pub exStyle: DWORD,
    pub helpId: DWORD,
    pub x: core::ffi::c_short,
    pub y: core::ffi::c_short,
    pub cx: core::ffi::c_short,
    pub cy: core::ffi::c_short,
    pub id: UINT,
    pub className: LPCWSTR,
    pub windowName: LPCWSTR,
    pub windowNameFree: BOOL,
    pub data: LPCVOID,
}
pub type PDLG_CONTROL_INFO = *mut DLG_CONTROL_INFO;

#[repr(C)]
pub struct DLG_TEMPLATE {
    pub style: DWORD,
    pub exStyle: DWORD,
    pub helpId: DWORD,
    pub nbItems: WORD,
    pub x: core::ffi::c_short,
    pub y: core::ffi::c_short,
    pub cx: core::ffi::c_short,
    pub cy: core::ffi::c_short,
    pub menuName: LPCWSTR,
    pub className: LPCWSTR,
    pub caption: LPCWSTR,
    pub pointSize: WORD,
    pub weight: WORD,
    pub italic: BOOL,
    pub faceName: LPCWSTR,
    pub dialogEx: BOOL,
}
pub type PDLG_TEMPLATE = *mut DLG_TEMPLATE;

#[repr(C)]
pub struct RADIOGROUP {
    pub firstID: UINT,
    pub lastID: UINT,
    pub checkID: UINT,
}
pub type PRADIOGROUP = *mut RADIOGROUP;

unsafe extern "system" fn DIALOG_get_info(hWnd: HWND, create: BOOL) -> *mut DIALOGINFO {
    let pWindow: PWND = ValidateHwnd(hWnd);
    if pWindow.is_null() {
        return core::ptr::null_mut();
    }

    let mut dlgInfo: *mut DIALOGINFO = (*pWindow).DialogPointer as *mut DIALOGINFO;

    if dlgInfo.is_null() && create != 0 {
        if !pWindow.is_null() && (*pWindow).cbwndExtra >= DLGWINDOWEXTRA {
            dlgInfo = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<DIALOGINFO>()) as SIZE_T,
            ) as *mut DIALOGINFO;
            if dlgInfo.is_null() {
                return core::ptr::null_mut();
            }

            (*dlgInfo).idResult = IDOK as INT;
            NtUserxSetDialogPointer(hWnd, dlgInfo as LPVOID);
        } else {
            return core::ptr::null_mut();
        }
    }

    dlgInfo
}

unsafe fn DIALOG_GetControl32(mut p: *const u16, info: *mut DLG_CONTROL_INFO, dialogEx: BOOL) -> *const u16 {
    if dialogEx != 0 {
        (*info).helpId = *(p as *const u32); p = p.add(2);
        (*info).exStyle = *(p as *const u32); p = p.add(2);
        (*info).style = *(p as *const u32); p = p.add(2);
    } else {
        (*info).helpId = 0;
        (*info).style = *(p as *const u32); p = p.add(2);
        (*info).exStyle = *(p as *const u32); p = p.add(2);
    }
    (*info).x = *p as i16; p = p.add(1);
    (*info).y = *p as i16; p = p.add(1);
    (*info).cx = *p as i16; p = p.add(1);
    (*info).cy = *p as i16; p = p.add(1);

    if dialogEx != 0 {
        (*info).id = *(p as *const i32) as UINT;
        p = p.add(2);
    } else {
        (*info).id = *p as UINT;
        p = p.add(1);
    }

    if *p == 0xffff {
        static CLASS_NAMES: [[u16; 10]; 6] = [
            [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, 0, 0, 0, 0],
            [b'E' as u16, b'd' as u16, b'i' as u16, b't' as u16, 0, 0, 0, 0, 0, 0],
            [b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'c' as u16, 0, 0, 0, 0],
            [b'L' as u16, b'i' as u16, b's' as u16, b't' as u16, b'B' as u16, b'o' as u16, b'x' as u16, 0, 0, 0],
            [b'S' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'l' as u16, b'B' as u16, b'a' as u16, b'r' as u16, 0],
            [b'C' as u16, b'o' as u16, b'm' as u16, b'b' as u16, b'o' as u16, b'B' as u16, b'o' as u16, b'x' as u16, 0, 0],
        ];
        let mut id = *p.add(1);
        if id >= 0x80 && id <= 0x85 { id -= 0x80; }
        if id <= 5 {
            (*info).className = CLASS_NAMES[id as usize].as_ptr();
        } else {
            (*info).className = core::ptr::null_mut();
            ERR!("Unknown built-in class id %04x\n", id as u32);
        }
        p = p.add(2);
    } else {
        (*info).className = p;
        p = p.add(strlenW(p) as usize + 1);
    }

    if *p == 0xffff {
        (*info).windowName = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<[u16; 7]>()) as SIZE_T) as LPCWSTR;
        if !(*info).windowName.is_null() {
            wsprintfW((*info).windowName as LPWSTR, {
                static FMT: [u16; 4] = [b'#' as u16, b'%' as u16, b'u' as u16, 0];
                FMT.as_ptr()
            }, *p.add(1) as u32);
            (*info).windowNameFree = 1;
        } else {
            (*info).windowNameFree = 0;
        }
        p = p.add(2);
    } else {
        (*info).windowName = p;
        (*info).windowNameFree = 0;
        p = p.add(strlenW(p) as usize + 1);
    }

    TRACE!(b"    %s %s %ld, %d, %d, %d, %d, %08x, %08x, %08x\n\0".as_ptr() as LPCSTR,
          debugstr_w((*info).className), debugstr_w((*info).windowName),
          (*info).id, (*info).x as i32, (*info).y as i32, (*info).cx as i32, (*info).cy as i32,
          (*info).style, (*info).exStyle, (*info).helpId);

    if *p != 0 {
        (*info).data = p as LPCVOID;
        p = p.add((*p as usize) / core::mem::size_of::<u16>());
    } else {
        (*info).data = core::ptr::null();
    }
    p = p.add(1);

    (((p as UINT_PTR) + 3) & !3) as *const u16
}

unsafe fn DIALOG_CreateControls32(hwnd: HWND, mut template: LPCSTR, dlgTemplate: *const DLG_TEMPLATE,
                                  hInst: HINSTANCE, unicode: BOOL) -> BOOL {
    let dlgInfo: *mut DIALOGINFO;
    let mut info: DLG_CONTROL_INFO = core::mem::zeroed();
    let mut hwndCtrl: HWND;
    let mut hwndDefButton: HWND = core::ptr::null_mut();
    let mut items: INT = (*dlgTemplate).nbItems as INT;

    dlgInfo = GETDLGINFO(hwnd);
    if dlgInfo.is_null() {
        return 0;
    }

    while {
        let cont = items != 0;
        items -= 1;
        cont
    } {
        template = DIALOG_GetControl32(template as *const u16, &mut info, (*dlgTemplate).dialogEx) as LPCSTR;
        info.style &= !WS_POPUP;
        info.style |= WS_CHILD;

        if info.style & WS_BORDER != 0 {
            info.style &= !WS_BORDER;
            info.exStyle |= WS_EX_CLIENTEDGE;
        }

        if unicode != 0 {
            hwndCtrl = CreateWindowExW(
                info.exStyle | WS_EX_NOPARENTNOTIFY,
                info.className,
                info.windowName,
                info.style | WS_CHILD,
                MulDiv(info.x as INT, (*dlgInfo).xBaseUnit as INT, 4),
                MulDiv(info.y as INT, (*dlgInfo).yBaseUnit as INT, 8),
                MulDiv(info.cx as INT, (*dlgInfo).xBaseUnit as INT, 4),
                MulDiv(info.cy as INT, (*dlgInfo).yBaseUnit as INT, 8),
                hwnd,
                info.id as ULONG_PTR as HMENU,
                hInst,
                info.data as LPVOID,
            );
        } else {
            let mut class = info.className as LPSTR;
            let mut caption = info.windowName as LPSTR;

            if !IS_INTRESOURCE(class as LPCVOID) {
                let len: DWORD = WideCharToMultiByte(CP_ACP, 0, info.className, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
                class = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as LPSTR;
                if !class.is_null() {
                    WideCharToMultiByte(CP_ACP, 0, info.className, -1, class, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
                }
            }
            if !IS_INTRESOURCE(caption as LPCVOID) {
                let len: DWORD = WideCharToMultiByte(CP_ACP, 0, info.windowName, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
                caption = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as LPSTR;
                if !caption.is_null() {
                    WideCharToMultiByte(CP_ACP, 0, info.windowName, -1, caption, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
                }
            }

            if !class.is_null() && !caption.is_null() {
                hwndCtrl = CreateWindowExA(
                    info.exStyle | WS_EX_NOPARENTNOTIFY,
                    class,
                    caption,
                    info.style | WS_CHILD,
                    MulDiv(info.x as INT, (*dlgInfo).xBaseUnit as INT, 4),
                    MulDiv(info.y as INT, (*dlgInfo).yBaseUnit as INT, 8),
                    MulDiv(info.cx as INT, (*dlgInfo).xBaseUnit as INT, 4),
                    MulDiv(info.cy as INT, (*dlgInfo).yBaseUnit as INT, 8),
                    hwnd,
                    info.id as ULONG_PTR as HMENU,
                    hInst,
                    info.data as LPVOID,
                );
            } else {
                hwndCtrl = core::ptr::null_mut();
            }
            if !IS_INTRESOURCE(class as LPCVOID) {
                HeapFree(GetProcessHeap(), 0, class as LPVOID);
            }
            if !IS_INTRESOURCE(caption as LPCVOID) {
                HeapFree(GetProcessHeap(), 0, caption as LPVOID);
            }
        }

        if info.windowNameFree != 0 {
            HeapFree(GetProcessHeap(), 0, info.windowName as LPVOID);
        }

        if hwndCtrl.is_null() {
            if (*dlgTemplate).style & DS_NOFAILCREATE != 0 {
                continue;
            }
            return 0;
        }

        if !(*dlgInfo).hUserFont.is_null() {
            SendMessageW(hwndCtrl, WM_SETFONT, (*dlgInfo).hUserFont as WPARAM, 0);
        }
        if SendMessageW(hwndCtrl, WM_GETDLGCODE, 0, 0) & DLGC_DEFPUSHBUTTON as LRESULT != 0 {
            if !hwndDefButton.is_null() {
                SendMessageW(hwndDefButton, BM_SETSTYLE, BS_PUSHBUTTON as WPARAM, 0);
            }
            hwndDefButton = hwndCtrl;
            (*dlgInfo).idResult = GetWindowLongPtrA(hwndCtrl, GWLP_ID) as INT;
        }
    }
    1
}

unsafe fn DIALOG_IsAccelerator(hwnd: HWND, hwndDlg: HWND, wParam: WPARAM) -> BOOL {
    let mut hwndControl: HWND = hwnd;
    let mut hwndNext: HWND;
    let mut dlgCode: INT;
    let mut buffer: [u16; 128] = [0; 128];

    loop {
        let style = GetWindowLongPtrW(hwndControl, GWL_STYLE as i32) as u32;
        if (style & (WS_VISIBLE | WS_DISABLED)) == WS_VISIBLE {
            dlgCode = SendMessageW(hwndControl, WM_GETDLGCODE, 0, 0) as INT;
            if (dlgCode & (DLGC_BUTTON as INT | DLGC_STATIC as INT)) != 0
                && GetWindowTextW(
                    hwndControl,
                    buffer.as_mut_ptr(),
                    (core::mem::size_of_val(&buffer) / core::mem::size_of::<u16>()) as i32,
                ) != 0
            {
                let mut p: LPWSTR = buffer.as_mut_ptr().offset(-2);

                loop {
                    p = strchrW(p.offset(2), b'&' as i32);
                    if !(p != core::ptr::null_mut() && *p.offset(1) == b'&' as u16) {
                        break;
                    }
                }

                if p != core::ptr::null_mut()
                    && toupperW(*p.offset(1) as i32) == toupperW(wParam as i32)
                {
                    if (dlgCode & DLGC_STATIC as INT) != 0
                        || (style & 0x0f) == BS_GROUPBOX
                    {
                        SendMessageW(hwndDlg, WM_NEXTDLGCTL, hwndControl as WPARAM, 1);
                        SendMessageW(hwndDlg, WM_NEXTDLGCTL, 0, 0);
                    } else if (dlgCode & DLGC_BUTTON as INT) != 0 {
                        SendMessageW(hwndControl, BM_CLICK, 0, 0);
                    }
                    return 1;
                }
            }
            hwndNext = GetWindow(hwndControl, GW_CHILD);
        } else {
            hwndNext = core::ptr::null_mut();
        }

        if hwndNext.is_null() {
            hwndNext = GetWindow(hwndControl, GW_HWNDNEXT);
        }

        while hwndNext.is_null() && !hwndControl.is_null() {
            hwndControl = GetParent(hwndControl);
            if hwndControl == hwndDlg {
                if hwnd == hwndDlg {
                    hwndNext = hwnd;
                    break;
                }
                hwndNext = GetWindow(hwndDlg, GW_CHILD);
            } else {
                hwndNext = GetWindow(hwndControl, GW_HWNDNEXT);
            }
        }
        hwndControl = hwndNext;

        if !(!hwndControl.is_null() && hwndControl != hwnd) {
            break;
        }
    }

    0
}

unsafe fn DIALOG_FindMsgDestination(mut hwndDlg: HWND) -> HWND {
    while GetWindowLongA(hwndDlg, GWL_STYLE) & DS_CONTROL as i32 != 0 {
        let pWnd: PWND;
        let hParent = GetParent(hwndDlg);
        if hParent.is_null() {
            break;
        }
        if IsWindow(hParent) == 0 {
            break;
        }

        pWnd = ValidateHwnd(hParent);
        if pWnd.is_null() || TestWindowProcess(pWnd) == 0 || hParent == GetDesktopWindow() {
            break;
        }

        if (*pWnd).state & WNDS_DIALOGWINDOW == 0 {
            break;
        }

        hwndDlg = hParent;
    }

    hwndDlg
}

unsafe extern "system" fn DIALOG_DoDialogBox(hwnd: HWND, owner: HWND) -> INT {
    let dlgInfo: *mut DIALOGINFO;
    let mut msg: MSG = core::mem::zeroed();
    let retval: INT;
    let mut bFirstEmpty: BOOL;
    let pWnd: PWND;

    pWnd = ValidateHwnd(hwnd);
    if pWnd.is_null() {
        return -1;
    }

    dlgInfo = GETDLGINFO(hwnd);
    if dlgInfo.is_null() {
        return -1;
    }

    bFirstEmpty = 1;
    if (*dlgInfo).flags & DF_END == 0 {
        loop {
            if PeekMessageW(&mut msg, core::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                if bFirstEmpty != 0 {
                    ShowWindow(hwnd, SW_SHOWNORMAL as i32);
                    bFirstEmpty = 0;
                }
                if GetWindowLongPtrW(hwnd, GWL_STYLE) & (DS_NOIDLEMSG as LONG_PTR) == 0 {
                    SendMessageW(owner, WM_ENTERIDLE, MSGF_DIALOGBOX as WPARAM, hwnd as LPARAM);
                }
                GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0);
            }

            if msg.message == WM_QUIT {
                PostQuitMessage(msg.wParam as INT);
                if IsWindow(hwnd) == 0 {
                    return 0;
                }
                break;
            }

            if msg.message == WM_KEYDOWN
                && (*pWnd).state & WNDS_MSGBOX != 0
                && GetForegroundWindow() == hwnd
            {
                if msg.wParam == ('C' as WPARAM) && GetKeyState(VK_CONTROL as INT) < 0 {
                    SendMessageW(hwnd, WM_COPY, 0, 0);
                }
            }

            if IsWindow(hwnd) == 0 {
                return 0;
            }
            if (*dlgInfo).flags & DF_END == 0 && IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
            if IsWindow(hwnd) == 0 {
                return 0;
            }
            if (*dlgInfo).flags & DF_END != 0 {
                break;
            }

            if bFirstEmpty != 0 && msg.message == WM_TIMER {
                ShowWindow(hwnd, SW_SHOWNORMAL as i32);
                bFirstEmpty = 0;
            }
        }
    }
    retval = (*dlgInfo).idResult;
    DestroyWindow(hwnd);
    retval
}

unsafe fn DIALOG_ParseTemplate32(template: LPCSTR, result: *mut DLG_TEMPLATE) -> LPCSTR {
    let mut p = template as *const u16;
    let signature: u16;
    let dlgver: u16;

    dlgver = *p; p = p.add(1);
    signature = *p; p = p.add(1);

    if dlgver == 1 && signature == 0xffff {
        (*result).dialogEx = 1;
        (*result).helpId = *(p as *const u32); p = p.add(2);
        (*result).exStyle = *(p as *const u32); p = p.add(2);
        (*result).style = *(p as *const u32); p = p.add(2);
    } else {
        (*result).style = *((p.sub(2)) as *const u32);
        (*result).dialogEx = 0;
        (*result).helpId = 0;
        (*result).exStyle = *(p as *const u32); p = p.add(2);
    }
    (*result).nbItems = *p; p = p.add(1);
    (*result).x = *p as i16; p = p.add(1);
    (*result).y = *p as i16; p = p.add(1);
    (*result).cx = *p as i16; p = p.add(1);
    (*result).cy = *p as i16; p = p.add(1);

    match *p {
        0x0000 => {
            (*result).menuName = core::ptr::null();
            p = p.add(1);
        }
        0xffff => {
            (*result).menuName = (*(p.add(1)) as UINT_PTR) as LPCWSTR;
            p = p.add(2);
        }
        _ => {
            (*result).menuName = p as LPCWSTR;
            p = p.add(strlenW((*result).menuName) as usize + 1);
        }
    }

    match *p {
        0x0000 => {
            (*result).className = WC_DIALOG as LPCWSTR;
            p = p.add(1);
        }
        0xffff => {
            (*result).className = (*(p.add(1)) as UINT_PTR) as LPCWSTR;
            p = p.add(2);
        }
        _ => {
            (*result).className = p as LPCWSTR;
            p = p.add(strlenW((*result).className) as usize + 1);
        }
    }

    (*result).caption = p as LPCWSTR;
    p = p.add(strlenW((*result).caption) as usize + 1);

    (*result).pointSize = 0;
    (*result).faceName = core::ptr::null();
    (*result).weight = FW_DONTCARE as WORD;
    (*result).italic = 0;

    if (*result).style & DS_SETFONT != 0 {
        (*result).pointSize = *p;
        p = p.add(1);

        if (*result).pointSize == 0x7fff {
        } else {
            if (*result).dialogEx != 0 {
                (*result).weight = *p; p = p.add(1);
                (*result).italic = (*p & 0xff) as BOOL; p = p.add(1);
            }
            (*result).faceName = p as LPCWSTR;
            p = p.add(strlenW((*result).faceName) as usize + 1);
        }
    }

    (((p as UINT_PTR) + 3) & !3) as LPCSTR
}

unsafe fn DEFDLG_SetFocus(hwndCtrl: HWND) {
    if (SendMessageW(hwndCtrl, WM_GETDLGCODE, 0, 0) & DLGC_HASSETSEL as LRESULT) != 0 {
        SendMessageW(hwndCtrl, EM_SETSEL, 0, -1i32 as LPARAM);
    }
    SetFocus(hwndCtrl);
}

unsafe fn DEFDLG_SaveFocus(hwnd: HWND) {
    let infoPtr: *mut DIALOGINFO;
    let hwndFocus: HWND = GetFocus();

    if hwndFocus.is_null() || IsChild(hwnd, hwndFocus) == 0 {
        return;
    }
    infoPtr = GETDLGINFO(hwnd);
    if infoPtr.is_null() {
        return;
    }
    (*infoPtr).hwndFocus = hwndFocus;
    /* Remove default button */
}

unsafe fn DEFDLG_RestoreFocus(hwnd: HWND, justActivate: BOOL) {
    let infoPtr: *mut DIALOGINFO;

    if IsIconic(hwnd) != 0 {
        return;
    }
    infoPtr = GETDLGINFO(hwnd);
    if infoPtr.is_null() {
        return;
    }
    if (*infoPtr).flags & DF_END != 0 {
        return;
    }
    if IsWindow((*infoPtr).hwndFocus) == 0 || (*infoPtr).hwndFocus == hwnd {
        if justActivate != 0 {
            return;
        }
        (*infoPtr).hwndFocus = GetNextDlgTabItem(hwnd, core::ptr::null_mut(), 0);
        if (*infoPtr).hwndFocus.is_null() {
            (*infoPtr).hwndFocus = GetNextDlgGroupItem(hwnd, core::ptr::null_mut(), 0);
        }
        if IsWindow((*infoPtr).hwndFocus) == 0 {
            return;
        }
    }
    if justActivate != 0 {
        SetFocus((*infoPtr).hwndFocus);
    } else {
        DEFDLG_SetFocus((*infoPtr).hwndFocus);
    }

    (*infoPtr).hwndFocus = core::ptr::null_mut();
}

unsafe fn DIALOG_CreateIndirect(
    hInst: HINSTANCE,
    mut dlgTemplate: LPCVOID,
    mut owner: HWND,
    dlgProc: DLGPROC,
    param: LPARAM,
    unicode: BOOL,
    modal_owner: *mut HWND,
) -> HWND {
    let mut hwnd: HWND;
    let mut rect: RECT = core::mem::zeroed();
    let mut pos: POINT = core::mem::zeroed();
    let mut size: SIZE = core::mem::zeroed();
    let mut template: DLG_TEMPLATE = core::mem::zeroed();
    let mut dlgInfo: *mut DIALOGINFO;
    let units: DWORD = GetDialogBaseUnits() as DWORD;
    let mut disabled_owner: HWND = core::ptr::null_mut();
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hUserFont: HFONT = core::ptr::null_mut();
    let flags: UINT = 0;
    let mut xBaseUnit: UINT = LOWORD(units) as UINT;
    let mut yBaseUnit: UINT = HIWORD(units) as UINT;

    if dlgTemplate.is_null() {
        return core::ptr::null_mut();
    }
    dlgTemplate = DIALOG_ParseTemplate32(dlgTemplate as LPCSTR, &mut template) as LPCVOID;

    if !template.menuName.is_null() {
        hMenu = LoadMenuW(hInst, template.menuName);
    }

    if template.style & DS_SETFONT != 0 {
        let dc: HDC = GetDC(core::ptr::null_mut());

        if template.pointSize == 0x7fff {
            let mut ncMetrics: NONCLIENTMETRICSW = core::mem::zeroed();
            ncMetrics.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
            if SystemParametersInfoW(
                SPI_GETNONCLIENTMETRICS,
                core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
                core::ptr::addr_of_mut!(ncMetrics) as LPVOID,
                0,
            ) != 0
            {
                hUserFont = CreateFontIndirectW(core::ptr::addr_of_mut!(ncMetrics.lfMessageFont));
            }
        } else {
            let pixels = MulDiv(template.pointSize as INT, GetDeviceCaps(dc, LOGPIXELSY as INT), 72);
            hUserFont = CreateFontW(
                -pixels,
                0,
                0,
                0,
                template.weight as INT,
                template.italic as DWORD,
                FALSE as DWORD,
                FALSE as DWORD,
                DEFAULT_CHARSET as DWORD,
                0,
                0,
                PROOF_QUALITY as DWORD,
                FF_DONTCARE as DWORD,
                template.faceName,
            );
        }

        if !hUserFont.is_null() {
            let mut charSize: SIZE = core::mem::zeroed();
            let hOldFont = SelectObject(dc, hUserFont as LPVOID);
            charSize.cx = GdiGetCharDimensions(dc, core::ptr::null_mut(), &mut charSize.cy);
            if charSize.cx != 0 {
                xBaseUnit = charSize.cx as UINT;
                yBaseUnit = charSize.cy as UINT;
            }
            SelectObject(dc, hOldFont);
        }
        ReleaseDC(core::ptr::null_mut(), dc);
    }

    SetRect(
        &mut rect,
        0,
        0,
        MulDiv(template.cx as INT, xBaseUnit as INT, 4),
        MulDiv(template.cy as INT, yBaseUnit as INT, 8),
    );
    if template.style & DS_CONTROL != 0 {
        template.style &= !(WS_CAPTION | WS_SYSMENU);
    }
    template.style |= DS_3DLOOK;
    if template.style & DS_MODALFRAME != 0 {
        template.exStyle |= WS_EX_DLGMODALFRAME;
    }
    if (template.style & DS_CONTROL != 0) || (template.style & WS_CHILD == 0) {
        template.exStyle |= WS_EX_CONTROLPARENT;
    }
    AdjustWindowRectEx(
        &mut rect,
        template.style,
        (!hMenu.is_null()) as BOOL,
        template.exStyle,
    );
    pos.x = rect.left;
    pos.y = rect.top;
    size.cx = rect.right - rect.left;
    size.cy = rect.bottom - rect.top;

    if template.x == CW_USEDEFAULT16 as i16 {
        pos.x = CW_USEDEFAULT;
        pos.y = CW_USEDEFAULT;
    } else {
        let mut monitor: HMONITOR = core::ptr::null_mut();
        let mut mon_info: MONITORINFO = core::mem::zeroed();

        mon_info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
        if template.style & DS_CENTER != 0 {
            monitor = MonitorFromWindow(
                if !owner.is_null() { owner } else { GetActiveWindow() },
                MONITOR_DEFAULTTOPRIMARY,
            );
            GetMonitorInfoW(monitor, &mut mon_info);
            pos.x = (mon_info.rcWork.left + mon_info.rcWork.right - size.cx) / 2;
            pos.y = (mon_info.rcWork.top + mon_info.rcWork.bottom - size.cy) / 2;
        } else if template.style & DS_CENTERMOUSE != 0 {
            GetCursorPos(&mut pos);
            monitor = MonitorFromPoint(pos, MONITOR_DEFAULTTOPRIMARY);
            GetMonitorInfoW(monitor, &mut mon_info);
        } else {
            pos.x += MulDiv(template.x as INT, xBaseUnit as INT, 4);
            pos.y += MulDiv(template.y as INT, yBaseUnit as INT, 8);
            if (template.style & (WS_CHILD | DS_ABSALIGN) == 0) && !owner.is_null() {
                ClientToScreen(owner, &mut pos);
            }
        }
        if template.style & WS_CHILD == 0 {
            let dX: INT;
            let dY: INT;

            if monitor.is_null() {
                SetRect(&mut rect, pos.x, pos.y, pos.x + size.cx, pos.y + size.cy);
                monitor = MonitorFromRect(&mut rect, MONITOR_DEFAULTTOPRIMARY);
                GetMonitorInfoW(monitor, &mut mon_info);
            }
            dX = pos.x + size.cx + GetSystemMetrics(SM_CXDLGFRAME as INT) - mon_info.rcWork.right;
            if dX > 0 {
                pos.x -= dX;
            }
            dY = pos.y + size.cy + GetSystemMetrics(SM_CYDLGFRAME as INT) - mon_info.rcWork.bottom;
            if dY > 0 {
                pos.y -= dY;
            }
            if pos.x < mon_info.rcWork.left {
                pos.x = mon_info.rcWork.left;
            }
            if pos.y < mon_info.rcWork.top {
                pos.y = mon_info.rcWork.top;
            }
        }
    }

    if !modal_owner.is_null() && !owner.is_null() {
        let mut parent: HWND = core::ptr::null_mut();
        while (GetWindowLongW(owner, GWL_STYLE) & (WS_POPUP | WS_CHILD) as LONG) == WS_CHILD as LONG {
            parent = GetParent(owner);
            if parent.is_null() || parent == GetDesktopWindow() {
                break;
            }
            owner = parent;
        }
        if parent.is_null() {
            parent = GetAncestor(owner, GA_ROOT);
        }

        if !parent.is_null() {
            owner = parent;

            if IsWindowEnabled(owner) != 0 {
                disabled_owner = owner;
                EnableWindow(disabled_owner, FALSE as BOOL);

                let captured = GetCapture();

                if !captured.is_null() {
                    SendMessageW(captured, WM_CANCELMODE, 0, 0);
                }
            }
        }
        *modal_owner = owner;
    }

    if unicode != 0 {
        hwnd = CreateWindowExW(
            template.exStyle,
            template.className,
            template.caption,
            template.style & !WS_VISIBLE,
            pos.x,
            pos.y,
            size.cx,
            size.cy,
            owner,
            hMenu,
            hInst,
            core::ptr::null_mut(),
        );
    } else {
        let mut class: LPCSTR = template.className as LPCSTR;
        let mut caption: LPCSTR = template.caption as LPCSTR;
        let mut class_tmp: LPSTR = core::ptr::null_mut();
        let mut caption_tmp: LPSTR = core::ptr::null_mut();

        if !IS_INTRESOURCE(class as LPCVOID) {
            let len = WideCharToMultiByte(
                CP_ACP,
                0,
                template.className,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            class_tmp = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as LPSTR;
            WideCharToMultiByte(
                CP_ACP,
                0,
                template.className,
                -1,
                class_tmp,
                len as INT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            class = class_tmp;
        }
        if !IS_INTRESOURCE(caption as LPCVOID) {
            let len = WideCharToMultiByte(
                CP_ACP,
                0,
                template.caption,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            caption_tmp = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as LPSTR;
            WideCharToMultiByte(
                CP_ACP,
                0,
                template.caption,
                -1,
                caption_tmp,
                len as INT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            caption = caption_tmp;
        }
        hwnd = CreateWindowExA(
            template.exStyle,
            class,
            caption,
            template.style & !WS_VISIBLE,
            pos.x,
            pos.y,
            size.cx,
            size.cy,
            owner,
            hMenu,
            hInst,
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, class_tmp as LPVOID);
        HeapFree(GetProcessHeap(), 0, caption_tmp as LPVOID);
    }

    if hwnd.is_null() {
        if !hUserFont.is_null() {
            DeleteObject(hUserFont as LPVOID);
        }
        if !hMenu.is_null() {
            DestroyMenu(hMenu);
        }
        if !disabled_owner.is_null() {
            EnableWindow(disabled_owner, TRUE as BOOL);
        }
        return core::ptr::null_mut();
    }

    dlgInfo = DIALOG_get_info(hwnd, TRUE as BOOL);
    if dlgInfo.is_null() {
        if !hUserFont.is_null() {
            DeleteObject(hUserFont as LPVOID);
        }
        if !hMenu.is_null() {
            DestroyMenu(hMenu);
        }
        if !disabled_owner.is_null() {
            EnableWindow(disabled_owner, TRUE as BOOL);
        }
        return core::ptr::null_mut();
    }
    (*dlgInfo).hwndFocus = core::ptr::null_mut();
    (*dlgInfo).hUserFont = hUserFont;
    (*dlgInfo).hMenu = hMenu;
    (*dlgInfo).xBaseUnit = xBaseUnit;
    (*dlgInfo).yBaseUnit = yBaseUnit;
    (*dlgInfo).flags = flags;

    if template.helpId != 0 {
        SetWindowContextHelpId(hwnd, template.helpId);
    }

    if unicode != 0 {
        SetWindowLongPtrW(hwnd, DWLP_DLGPROC, core::mem::transmute::<DLGPROC, isize>(dlgProc));
    } else {
        SetWindowLongPtrA(hwnd, DWLP_DLGPROC, core::mem::transmute::<DLGPROC, isize>(dlgProc));
    }

    if dlgProc.is_some() && !(*dlgInfo).hUserFont.is_null() {
        SendMessageW(hwnd, WM_SETFONT, (*dlgInfo).hUserFont as WPARAM, 0);
    }

    if DIALOG_CreateControls32(hwnd, dlgTemplate as LPCSTR, &template, hInst, unicode) != 0 {
        if dlgProc.is_some() {
            let mut focus = GetNextDlgTabItem(hwnd, core::ptr::null_mut(), FALSE as BOOL);
            if focus.is_null() {
                focus = GetNextDlgGroupItem(hwnd, core::ptr::null_mut(), FALSE as BOOL);
            }
            if SendMessageW(hwnd, WM_INITDIALOG, focus as WPARAM, param) != 0
                && IsWindow(hwnd) != 0
                && ((!template.style & DS_CONTROL != 0) || (template.style & WS_VISIBLE != 0))
            {
                focus = GetNextDlgTabItem(hwnd, core::ptr::null_mut(), FALSE as BOOL);
                if focus.is_null() {
                    focus = GetNextDlgGroupItem(hwnd, core::ptr::null_mut(), FALSE as BOOL);
                }
                if !focus.is_null() {
                    if SendMessageW(focus, WM_GETDLGCODE, 0, 0) & DLGC_HASSETSEL as LRESULT != 0 {
                        SendMessageW(focus, EM_SETSEL, 0, MAXLONG as LPARAM);
                    }
                    SetFocus(focus);
                } else {
                    if template.style & WS_CHILD == 0 {
                        SetFocus(hwnd);
                    }
                }
            }
        }
        if GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as LONG_PTR == 0 {
            SendMessageW(hwnd, WM_CHANGEUISTATE, MAKEWPARAM(UIS_INITIALIZE as WORD, 0), 0);
        }
        if template.style & WS_VISIBLE != 0
            && GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_VISIBLE as LONG_PTR == 0
        {
            ShowWindow(hwnd, SW_SHOWNORMAL as i32);
            UpdateWindow(hwnd);
            IntNotifyWinEvent(
                EVENT_SYSTEM_DIALOGSTART,
                hwnd,
                OBJID_WINDOW as LONG,
                CHILDID_SELF as LONG,
                0,
            );
        }
        return hwnd;
    }
    if !disabled_owner.is_null() {
        EnableWindow(disabled_owner, TRUE as BOOL);
    }
    IntNotifyWinEvent(
        EVENT_SYSTEM_DIALOGEND,
        hwnd,
        OBJID_WINDOW as LONG,
        CHILDID_SELF as LONG,
        0,
    );
    if IsWindow(hwnd) != 0 {
        DestroyWindow(hwnd);
        if !owner.is_null() {
            if NtUserGetThreadState(THREADSTATE_FOREGROUNDTHREAD as DWORD) != 0
                && NtUserQueryWindow(owner, QUERY_WINDOW_FOREGROUND as DWORD) == 0
            {
                SetForegroundWindow(owner);
            }
        }
    }
    core::ptr::null_mut()
}

unsafe fn DEFDLG_FindDefButton(hwndDlg: HWND) -> HWND {
    let mut hwndChild: HWND;
    let mut hwndTmp: HWND;

    hwndChild = GetWindow(hwndDlg, GW_CHILD as _);
    while !hwndChild.is_null() {
        if SendMessageW(hwndChild, WM_GETDLGCODE, 0, 0) & DLGC_DEFPUSHBUTTON as LRESULT != 0 {
            break;
        }

        if GetWindowLongPtrW(hwndChild, GWL_EXSTYLE) & WS_EX_CONTROLPARENT as LONG_PTR != 0 {
            let dsStyle: LONG = GetWindowLongPtrW(hwndChild, GWL_STYLE) as LONG;
            if (dsStyle & WS_VISIBLE as LONG) != 0
                && (dsStyle & WS_DISABLED as LONG) == 0
                && {
                    hwndTmp = DEFDLG_FindDefButton(hwndChild);
                    !hwndTmp.is_null()
                }
            {
                return hwndTmp;
            }
        }
        hwndChild = GetWindow(hwndChild, GW_HWNDNEXT as _);
    }
    hwndChild
}

unsafe fn DEFDLG_SetDefId(hwndDlg: HWND, dlgInfo: *mut DIALOGINFO, wParam: WPARAM) -> BOOL {
    let mut dlgcode: DWORD = 0;
    let mut hwndOld: HWND;
    let hwndNew: HWND = GetDlgItem(hwndDlg, wParam as core::ffi::c_int);
    let old_id: INT = (*dlgInfo).idResult;

    (*dlgInfo).idResult = wParam as INT;
    if !hwndNew.is_null() {
        dlgcode = SendMessageW(hwndNew, WM_GETDLGCODE, 0, 0) as DWORD;
        if (dlgcode & (DLGC_UNDEFPUSHBUTTON | DLGC_BUTTON)) == 0 {
            return 0u8 as BOOL;
        }
    }

    hwndOld = GetDlgItem(hwndDlg, old_id as core::ffi::c_int);
    if hwndOld.is_null()
        || (SendMessageW(hwndOld, WM_GETDLGCODE, 0, 0) as DWORD & DLGC_DEFPUSHBUTTON) == 0
    {
        hwndOld = DEFDLG_FindDefButton(hwndDlg);
    }
    if !hwndOld.is_null() && hwndOld != hwndNew {
        SendMessageW(hwndOld, BM_SETSTYLE, BS_PUSHBUTTON as WPARAM, 1i32 as LPARAM);
    }

    if !hwndNew.is_null() {
        if (dlgcode & DLGC_UNDEFPUSHBUTTON) != 0 {
            SendMessageW(hwndNew, BM_SETSTYLE, BS_DEFPUSHBUTTON as WPARAM, 1i32 as LPARAM);
        }
    }
    1u8 as BOOL
}

unsafe fn DEFDLG_SetDefButton(hwndDlg: HWND, dlgInfo: *mut DIALOGINFO, mut hwndNew: HWND) -> BOOL {
    let mut dlgcode: DWORD = 0;
    let mut hwndOld = GetDlgItem(hwndDlg, (*dlgInfo).idResult);

    if !hwndNew.is_null() {
        dlgcode = SendMessageW(hwndNew, WM_GETDLGCODE, 0, 0) as DWORD;
        if dlgcode & (DLGC_UNDEFPUSHBUTTON | DLGC_DEFPUSHBUTTON) == 0 {
            hwndNew = hwndOld;
            dlgcode = SendMessageW(hwndNew, WM_GETDLGCODE, 0, 0) as DWORD;
        }
    }

    if hwndOld.is_null()
        || (SendMessageW(hwndOld, WM_GETDLGCODE, 0, 0) as DWORD & DLGC_DEFPUSHBUTTON) == 0
    {
        hwndOld = DEFDLG_FindDefButton(hwndDlg);
    }
    if !hwndOld.is_null() && hwndOld != hwndNew {
        SendMessageW(hwndOld, BM_SETSTYLE, BS_PUSHBUTTON as WPARAM, TRUE as LPARAM);
    }

    if !hwndNew.is_null() {
        if dlgcode & DLGC_UNDEFPUSHBUTTON != 0 {
            SendMessageW(hwndNew, BM_SETSTYLE, BS_DEFPUSHBUTTON as WPARAM, TRUE as LPARAM);
        }
    }
    TRUE as BOOL
}

unsafe fn DEFDLG_Reposition(hwnd: HWND) {
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
    let mut rc: RECT = core::mem::zeroed();

    if GetWindowLongW(hwnd, GWL_STYLE as i32) & (WS_CHILD as i32) != 0 {
        return;
    }

    if IsIconic(hwnd) != 0 {
        return;
    }

    let hMon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

    if GetMonitorInfoW(hMon, &mut mi) == 0 || GetWindowRect(hwnd, &mut rc) == 0 {
        return;
    }

    let cx = rc.right - rc.left;
    let cy = rc.bottom - rc.top;

    if rc.right > mi.rcWork.right {
        rc.right = mi.rcWork.right;
        rc.left = rc.right - cx;
    }
    if rc.bottom > mi.rcWork.bottom - 4 {
        rc.bottom = mi.rcWork.bottom - 4;
        rc.top = rc.bottom - cy;
    }

    if rc.left < mi.rcWork.left {
        rc.left = mi.rcWork.left;
    }
    if rc.top < mi.rcWork.top {
        rc.top = mi.rcWork.top;
    }

    SetWindowPos(
        hwnd,
        core::ptr::null_mut(),
        rc.left,
        rc.top,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER,
    );
}

unsafe fn DEFDLG_Proc(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    mut dlgInfo: *mut DIALOGINFO,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => {
            let brush: HBRUSH = GetControlColor(hwnd, hwnd, wParam as HDC, WM_CTLCOLORDLG as UINT);
            if !brush.is_null() {
                let mut rect: RECT = core::mem::zeroed();
                let hdc: HDC = wParam as HDC;
                GetClientRect(hwnd, &mut rect);
                DPtoLP(hdc, core::ptr::addr_of_mut!(rect) as LPPOINT, 2);
                FillRect(hdc, &rect, brush);
            }
            return 1;
        }
        WM_NCDESTROY => {
            dlgInfo = DIALOG_get_info(hwnd, 0);
            if !dlgInfo.is_null() {
                if !(*dlgInfo).hUserFont.is_null() {
                    DeleteObject((*dlgInfo).hUserFont as _);
                }
                if !(*dlgInfo).hMenu.is_null() {
                    DestroyMenu((*dlgInfo).hMenu);
                }
                HeapFree(GetProcessHeap(), 0, dlgInfo as LPVOID);
                NtUserSetThreadState(0, DF_DIALOGACTIVE);
                NtUserxSetDialogPointer(hwnd, core::ptr::null_mut());
            }
            return DefWindowProcA(hwnd, msg, wParam, lParam);
        }
        WM_SHOWWINDOW => {
            if wParam == 0 {
                DEFDLG_SaveFocus(hwnd);
            }
            return DefWindowProcA(hwnd, msg, wParam, lParam);
        }
        WM_ACTIVATE => {
            {
                let dwSetFlag: DWORD;
                let hwndparent: HWND = DIALOG_FindMsgDestination(hwnd);
                dwSetFlag = if wParam != 0 { DF_DIALOGACTIVE } else { 0 };
                if hwndparent != hwnd {
                    NtUserSetThreadState(dwSetFlag, DF_DIALOGACTIVE);
                }
            }
            if wParam != 0 {
                DEFDLG_RestoreFocus(hwnd, 1);
            } else {
                DEFDLG_SaveFocus(hwnd);
            }
            return 0;
        }
        WM_SETFOCUS => {
            DEFDLG_RestoreFocus(hwnd, 0);
            return 0;
        }
        DM_SETDEFID => {
            if !dlgInfo.is_null() && ((*dlgInfo).flags & DF_END) == 0 {
                DEFDLG_SetDefId(hwnd, dlgInfo, wParam);
            }
            return 1;
        }
        DM_GETDEFID => {
            if !dlgInfo.is_null() && ((*dlgInfo).flags & DF_END) == 0 {
                let hwndDefId: HWND;
                if (*dlgInfo).idResult != 0 {
                    return MAKELONG((*dlgInfo).idResult as _, DC_HASDEFID as _) as LRESULT;
                }
                hwndDefId = DEFDLG_FindDefButton(hwnd);
                if !hwndDefId.is_null() {
                    return MAKELONG(GetDlgCtrlID(hwndDefId) as _, DC_HASDEFID as _) as LRESULT;
                }
            }
            return 0;
        }
        DM_REPOSITION => {
            DEFDLG_Reposition(hwnd);
            return 0;
        }
        WM_NEXTDLGCTL => {
            if !dlgInfo.is_null() {
                let mut hwndDest: HWND = wParam as HWND;
                if lParam == 0 {
                    hwndDest = GetNextDlgTabItem(hwnd, GetFocus(), wParam as BOOL);
                }
                if !hwndDest.is_null() {
                    DEFDLG_SetFocus(hwndDest);
                }
                DEFDLG_SetDefButton(hwnd, dlgInfo, hwndDest);
            }
            return 0;
        }
        WM_ENTERMENULOOP | WM_LBUTTONDOWN | WM_NCLBUTTONDOWN => {
            {
                let hwndFocus: HWND = GetFocus();
                if !hwndFocus.is_null() {
                    if SendMessageW(hwndFocus, CB_SHOWDROPDOWN as UINT, 0, 0) == 0 {
                        SendMessageW(GetParent(hwndFocus), CB_SHOWDROPDOWN as UINT, 0, 0);
                    }
                }
            }
            return DefWindowProcA(hwnd, msg, wParam, lParam);
        }
        WM_GETFONT => {
            return if !dlgInfo.is_null() {
                (*dlgInfo).hUserFont as LRESULT
            } else {
                0
            };
        }
        WM_CLOSE => {
            PostMessageA(
                hwnd,
                WM_COMMAND as UINT,
                MAKEWPARAM(IDCANCEL as _, BN_CLICKED as _) as WPARAM,
                GetDlgItem(hwnd, IDCANCEL as core::ffi::c_int) as LPARAM,
            );
            return 0;
        }
        _ => {}
    }
    0
}

unsafe fn DEFDLG_Epilog(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, fResult: BOOL, fAnsi: BOOL) -> LRESULT {
    if (msg >= WM_CTLCOLORMSGBOX && msg <= WM_CTLCOLORSTATIC) || msg == WM_CTLCOLOR {
        if fResult != 0 {
            return fResult as LRESULT;
        }

        return if fAnsi != 0 {
            DefWindowProcA(hwnd, msg, wParam, lParam)
        } else {
            DefWindowProcW(hwnd, msg, wParam, lParam)
        };
    }
    if msg == WM_COMPAREITEM ||
        msg == WM_VKEYTOITEM || msg == WM_CHARTOITEM ||
        msg == WM_QUERYDRAGICON || msg == WM_INITDIALOG {
        return fResult as LRESULT;
    }

    GetWindowLongPtrW(hwnd, DWLP_MSGRESULT as i32) as LRESULT
}

unsafe fn DIALOG_GetNextTabItem(hwndMain: HWND, hwndDlg: HWND, hwndCtrl: HWND, fPrevious: BOOL) -> HWND {
    let mut dsStyle: LONG;
    let mut exStyle: LONG;
    let wndSearch: UINT = if fPrevious != 0 { GW_HWNDPREV } else { GW_HWNDNEXT };
    let mut retWnd: HWND = core::ptr::null_mut();
    let mut hChildFirst: HWND = core::ptr::null_mut();

    if hwndCtrl.is_null() {
        hChildFirst = GetWindow(hwndDlg, GW_CHILD);
        if fPrevious != 0 {
            hChildFirst = GetWindow(hChildFirst, GW_HWNDLAST);
        }
    } else if IsChild(hwndMain, hwndCtrl) != 0 {
        hChildFirst = GetWindow(hwndCtrl, wndSearch);
        if hChildFirst.is_null() {
            if GetParent(hwndCtrl) != hwndMain {
                hChildFirst = GetWindow(GetParent(hwndCtrl), wndSearch);
            } else {
                hChildFirst = GetWindow(hwndCtrl, if fPrevious != 0 { GW_HWNDLAST } else { GW_HWNDFIRST });
            }
        }
    }

    while !hChildFirst.is_null() {
        dsStyle = GetWindowLongPtrA(hChildFirst, GWL_STYLE) as LONG;
        exStyle = GetWindowLongPtrA(hChildFirst, GWL_EXSTYLE) as LONG;
        if (exStyle & WS_EX_CONTROLPARENT as LONG) != 0
            && (dsStyle & WS_VISIBLE as LONG) != 0
            && (dsStyle & WS_DISABLED as LONG) == 0
        {
            let retWnd2 = DIALOG_GetNextTabItem(hwndMain, hChildFirst, core::ptr::null_mut(), fPrevious);
            if !retWnd2.is_null() {
                return retWnd2;
            }
        } else if (dsStyle & WS_TABSTOP as LONG) != 0
            && (dsStyle & WS_VISIBLE as LONG) != 0
            && (dsStyle & WS_DISABLED as LONG) == 0
        {
            return hChildFirst;
        }
        hChildFirst = GetWindow(hChildFirst, wndSearch);
    }

    if !hwndCtrl.is_null() {
        let mut hParent = GetParent(hwndCtrl);
        while !hParent.is_null() {
            if hParent == hwndMain {
                break;
            }
            retWnd = DIALOG_GetNextTabItem(hwndMain, GetParent(hParent), hParent, fPrevious);
            if !retWnd.is_null() {
                break;
            }
            hParent = GetParent(hParent);
        }
        if retWnd.is_null() {
            retWnd = DIALOG_GetNextTabItem(hwndMain, hwndMain, core::ptr::null_mut(), fPrevious);
        }
    }

    if !retWnd.is_null() {
        retWnd
    } else {
        hwndCtrl
    }
}

unsafe fn DIALOG_DlgDirListW(
    hDlg: HWND,
    spec: LPWSTR,
    idLBox: INT,
    idStatic: INT,
    mut attrib: UINT,
    combo: BOOL,
) -> INT {
    let mut hwnd: HWND;
    let orig_spec: LPWSTR = spec;
    let mut spec = spec;
    let mut any: [WCHAR; 4] = ['*' as WCHAR, '.' as WCHAR, '*' as WCHAR, 0];
    let mut star: [WCHAR; 2] = ['*' as WCHAR, 0];

    macro_rules! SENDMSG {
        ($msg:expr, $wparam:expr, $lparam:expr) => {
            if (attrib & DDL_POSTMSGS) != 0 {
                PostMessageW(hwnd, $msg, $wparam, $lparam);
            } else {
                SendMessageW(hwnd, $msg, $wparam, $lparam);
            }
        };
    }

    /* If the path exists and is a directory, chdir to it */
    if spec.is_null() || *spec == 0 || SetCurrentDirectoryW(spec) != 0 {
        spec = star.as_mut_ptr();
    } else {
        let mut p: *mut WCHAR;
        let mut p2: *mut WCHAR;

        if strchrW(spec, '*' as core::ffi::c_int).is_null()
            && strchrW(spec, '?' as core::ffi::c_int).is_null()
        {
            SetLastError(ERROR_NO_WILDCARD_CHARACTERS);
            return 0;
        }
        p = spec;
        p2 = strchrW(p, ':' as core::ffi::c_int);
        if !p2.is_null() {
            p = p2.add(1);
        }
        p2 = strrchrW(p, '\\' as core::ffi::c_int);
        if !p2.is_null() {
            p = p2;
        }
        p2 = strrchrW(p, '/' as core::ffi::c_int);
        if !p2.is_null() {
            p = p2;
        }
        if p != spec {
            let sep = *p;
            *p = 0;
            if SetCurrentDirectoryW(spec) == 0 {
                *p = sep;
                return 0;
            }
            spec = p.add(1);
        }
    }

    if idLBox != 0 && {
        hwnd = GetDlgItem(hDlg, idLBox);
        hwnd != core::ptr::null_mut()
    } {
        if attrib == DDL_DRIVES {
            attrib |= DDL_EXCLUSIVE;
        }

        SENDMSG!(
            if combo != 0 { CB_RESETCONTENT } else { LB_RESETCONTENT },
            0,
            0
        );
        if (attrib & DDL_DIRECTORY) != 0 {
            if (attrib & DDL_EXCLUSIVE) == 0 {
                SENDMSG!(
                    if combo != 0 { CB_DIR } else { LB_DIR },
                    (attrib & !(DDL_DIRECTORY | DDL_DRIVES)) as WPARAM,
                    spec as LPARAM
                );
            }
            SENDMSG!(
                if combo != 0 { CB_DIR } else { LB_DIR },
                ((attrib & (DDL_DIRECTORY | DDL_DRIVES)) | DDL_EXCLUSIVE) as WPARAM,
                any.as_mut_ptr() as LPARAM
            );
        } else {
            SENDMSG!(
                if combo != 0 { CB_DIR } else { LB_DIR },
                attrib as WPARAM,
                spec as LPARAM
            );
        }
    }

    /* Convert path specification to uppercase */
    if !spec.is_null() {
        CharUpperW(spec);
    }

    if idStatic != 0 && {
        hwnd = GetDlgItem(hDlg, idStatic);
        hwnd != core::ptr::null_mut()
    } {
        let mut temp: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        GetCurrentDirectoryW(
            (core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() / core::mem::size_of::<WCHAR>())
                as u32,
            temp.as_mut_ptr(),
        );
        CharLowerW(temp.as_mut_ptr());
        SetDlgItemTextW(hDlg, idStatic, temp.as_ptr());
    }

    if !orig_spec.is_null() && spec != orig_spec {
        let mut p = spec;
        let mut o = orig_spec;
        loop {
            let v = *p;
            *o = v;
            o = o.add(1);
            p = p.add(1);
            if v == 0 {
                break;
            }
        }
    }

    1
}

unsafe fn DIALOG_DlgDirListA(
    hDlg: HWND,
    spec: LPSTR,
    idLBox: INT,
    idStatic: INT,
    attrib: UINT,
    combo: BOOL,
) -> INT {

    if !spec.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, spec, -1, core::ptr::null_mut(), 0);
        let specW = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if specW.is_null() {
            return 0;
        }
        MultiByteToWideChar(CP_ACP, 0, spec, -1, specW, len);
        let ret = DIALOG_DlgDirListW(hDlg, specW, idLBox, idStatic, attrib, combo);
        WideCharToMultiByte(
            CP_ACP,
            0,
            specW,
            -1,
            spec,
            0x7fffffff,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, specW as *mut core::ffi::c_void);
        return ret;
    }
    DIALOG_DlgDirListW(
        hDlg,
        core::ptr::null_mut(),
        idLBox,
        idStatic,
        attrib,
        combo,
    )
}

unsafe fn DIALOG_DlgDirSelect(hwnd: HWND, str: LPWSTR, len: INT, id: INT, unicode: BOOL, combo: BOOL) -> BOOL {
    let buffer: *mut u16;
    let ptr: *mut u16;
    let item: INT;
    let size: INT;
    let ret: BOOL;
    let listbox: HWND = GetDlgItem(hwnd, id);

    if listbox.is_null() {
        return 0;
    }

    item = SendMessageW(listbox, if combo != 0 { CB_GETCURSEL } else { LB_GETCURSEL }, 0, 0) as INT;
    if item == LB_ERR {
        return 0;
    }

    size = SendMessageW(listbox, if combo != 0 { CB_GETLBTEXTLEN } else { LB_GETTEXTLEN }, item as WPARAM, 0) as INT;
    if size == LB_ERR {
        return 0;
    }

    buffer = HeapAlloc(GetProcessHeap(), 0, ((size + 2) as usize * core::mem::size_of::<u16>()) as _) as *mut u16;
    if buffer.is_null() {
        return 0;
    }

    SendMessageW(listbox, if combo != 0 { CB_GETLBTEXT } else { LB_GETTEXT }, item as WPARAM, buffer as LPARAM);

    ret = (*buffer == b'[' as u16) as BOOL;
    if ret != 0 {
        if *buffer.add(1) == b'-' as u16 {
            *buffer.add(3) = b':' as u16;
            *buffer.add(4) = 0;
            ptr = buffer.add(2);
        } else {
            *buffer.add(strlenW(buffer) as usize - 1) = b'\\' as u16;
            ptr = buffer.add(1);
        }
    } else {
        if strchrW(buffer, b'.' as i32).is_null() {
            *buffer.add(strlenW(buffer) as usize + 1) = 0;
            *buffer.add(strlenW(buffer) as usize) = b'.' as u16;
        }
        ptr = buffer;
    }

    if unicode == 0 {
        if len > 0 && WideCharToMultiByte(CP_ACP, 0, ptr, -1, str as LPSTR, len, core::ptr::null(), core::ptr::null_mut()) == 0 {
            *(str as LPSTR).add((len - 1) as usize) = 0;
        }
    } else {
        lstrcpynW(str, ptr, len);
    }
    HeapFree(GetProcessHeap(), 0, buffer as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CreateDialogIndirectParamAorW(
    hInstance: HINSTANCE,
    lpTemplate: LPCDLGTEMPLATE,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    lParamInit: LPARAM,
    Flags: DWORD,
) -> HWND {
    DIALOG_CreateIndirect(
        hInstance,
        lpTemplate as LPCVOID,
        hWndParent,
        lpDialogFunc,
        lParamInit,
        if Flags == DLG_ISANSI as DWORD { 0 } else { 1 },
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateDialogIndirectParamA(
    hInstance: HINSTANCE,
    lpTemplate: LPCDLGTEMPLATE,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    lParamInit: LPARAM,
) -> HWND {
    CreateDialogIndirectParamAorW(
        hInstance,
        lpTemplate,
        hWndParent,
        lpDialogFunc,
        lParamInit,
        DLG_ISANSI as DWORD,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateDialogIndirectParamW(
    hInstance: HINSTANCE,
    lpTemplate: LPCDLGTEMPLATE,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    lParamInit: LPARAM,
) -> HWND {
    CreateDialogIndirectParamAorW(hInstance, lpTemplate, hWndParent, lpDialogFunc, lParamInit, 0)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDialogParamA(
    hInstance: HINSTANCE,
    lpTemplateName: LPCSTR,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> HWND {

    let hrsrc = FindResourceA(hInstance, lpTemplateName, RT_DIALOG as LPCSTR);
    if hrsrc.is_null() {
        return core::ptr::null_mut();
    }

    let ptr = LoadResource(hInstance, hrsrc) as LPCDLGTEMPLATE;
    if ptr.is_null() {
        return core::ptr::null_mut();
    }

    CreateDialogIndirectParamA(hInstance, ptr, hWndParent, lpDialogFunc, dwInitParam)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDialogParamW(
    hInstance: HINSTANCE,
    lpTemplateName: LPCWSTR,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> HWND {

    let hrsrc = FindResourceW(hInstance, lpTemplateName, RT_DIALOG as LPCWSTR);
    if hrsrc.is_null() {
        return core::ptr::null_mut();
    }
    let ptr = LoadResource(hInstance, hrsrc) as LPCDLGTEMPLATE;
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    CreateDialogIndirectParamW(hInstance, ptr, hWndParent, lpDialogFunc, dwInitParam)
}

#[no_mangle]
pub unsafe extern "system" fn DefDlgProcA(
    hDlg: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let dlgInfo: *mut DIALOGINFO;
    let dlgproc: WNDPROC;
    let mut result: BOOL = 0;

    dlgInfo = DIALOG_get_info(hDlg, 1);
    if dlgInfo.is_null() {
        return 0;
    }

    SetWindowLongPtrW(hDlg, DWLP_MSGRESULT as i32, 0);

    dlgproc = core::mem::transmute::<usize, WNDPROC>(GetWindowLongPtrW(hDlg, DWLP_DLGPROC) as usize);
    if dlgproc.is_some() {
        result = CallWindowProcA(dlgproc, hDlg, Msg, wParam, lParam) as BOOL;
    }

    if result == 0 && IsWindow(hDlg) != 0 {
        match Msg {
            WM_ERASEBKGND
            | WM_SHOWWINDOW
            | WM_ACTIVATE
            | WM_SETFOCUS
            | DM_SETDEFID
            | DM_GETDEFID
            | DM_REPOSITION
            | WM_NEXTDLGCTL
            | WM_GETFONT
            | WM_CLOSE
            | WM_NCDESTROY
            | WM_ENTERMENULOOP
            | WM_LBUTTONDOWN
            | WM_NCLBUTTONDOWN => {
                return DEFDLG_Proc(hDlg, Msg, wParam, lParam, dlgInfo);
            }
            WM_INITDIALOG | WM_VKEYTOITEM | WM_COMPAREITEM | WM_CHARTOITEM => {}
            _ => {
                return DefWindowProcA(hDlg, Msg, wParam, lParam);
            }
        }
    }
    DEFDLG_Epilog(hDlg, Msg, wParam, lParam, result, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DefDlgProcW(
    hDlg: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let dlgInfo: *mut DIALOGINFO;
    let dlgproc: WNDPROC;
    let mut result: BOOL = 0;

    dlgInfo = DIALOG_get_info(hDlg, 1);
    if dlgInfo.is_null() {
        return 0;
    }

    SetWindowLongPtrW(hDlg, DWLP_MSGRESULT as i32, 0);

    dlgproc = core::mem::transmute::<usize, WNDPROC>(GetWindowLongPtrW(hDlg, DWLP_DLGPROC) as usize);
    if dlgproc.is_some() {
        result = CallWindowProcW(dlgproc, hDlg, Msg, wParam, lParam) as BOOL;
    }

    if result == 0 && IsWindow(hDlg) != 0 {
        match Msg {
            WM_ERASEBKGND
            | WM_SHOWWINDOW
            | WM_ACTIVATE
            | WM_SETFOCUS
            | DM_SETDEFID
            | DM_GETDEFID
            | DM_REPOSITION
            | WM_NEXTDLGCTL
            | WM_GETFONT
            | WM_CLOSE
            | WM_NCDESTROY
            | WM_ENTERMENULOOP
            | WM_LBUTTONDOWN
            | WM_NCLBUTTONDOWN => {
                return DEFDLG_Proc(hDlg, Msg, wParam, lParam, dlgInfo);
            }
            WM_INITDIALOG | WM_VKEYTOITEM | WM_COMPAREITEM | WM_CHARTOITEM => {}
            _ => {
                return DefWindowProcW(hDlg, Msg, wParam, lParam);
            }
        }
    }
    DEFDLG_Epilog(hDlg, Msg, wParam, lParam, result, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DialogBoxIndirectParamAorW(
    hInstance: HINSTANCE,
    hDialogTemplate: LPCDLGTEMPLATE,
    mut hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
    Flags: u32,
) -> INT_PTR {
    let hWnd = DIALOG_CreateIndirect(
        hInstance,
        hDialogTemplate as LPCVOID,
        hWndParent,
        lpDialogFunc,
        dwInitParam,
        if Flags == DLG_ISANSI as u32 { 0 } else { 1 },
        core::ptr::addr_of_mut!(hWndParent),
    );
    if !hWnd.is_null() {
        return DIALOG_DoDialogBox(hWnd, hWndParent) as INT_PTR;
    }
    -1
}

#[no_mangle]
pub unsafe extern "system" fn DialogBoxIndirectParamA(
    hInstance: HINSTANCE,
    hDialogTemplate: LPCDLGTEMPLATE,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> INT_PTR {
    DialogBoxIndirectParamAorW(
        hInstance,
        hDialogTemplate,
        hWndParent,
        lpDialogFunc,
        dwInitParam,
        DLG_ISANSI as u32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DialogBoxIndirectParamW(
    hInstance: HINSTANCE,
    hDialogTemplate: LPCDLGTEMPLATE,
    hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> INT_PTR {
    DialogBoxIndirectParamAorW(hInstance, hDialogTemplate, hWndParent, lpDialogFunc, dwInitParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DialogBoxParamA(
    hInstance: HINSTANCE,
    lpTemplateName: LPCSTR,
    mut hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> INT_PTR {
    let hwnd: HWND;
    let hrsrc: HRSRC;
    let ptr: LPCDLGTEMPLATE;

    hrsrc = FindResourceA(hInstance, lpTemplateName, RT_DIALOG as LPCSTR);
    if hrsrc.is_null() {
        SetLastError(ERROR_RESOURCE_NAME_NOT_FOUND);
        return -1;
    }
    ptr = LoadResource(hInstance, hrsrc) as LPCDLGTEMPLATE;
    if ptr.is_null() {
        SetLastError(ERROR_RESOURCE_NAME_NOT_FOUND);
        return -1;
    }
    if !hWndParent.is_null() && IsWindow(hWndParent) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    hwnd = DIALOG_CreateIndirect(
        hInstance,
        ptr as LPCVOID,
        hWndParent,
        lpDialogFunc,
        dwInitParam,
        0,
        core::ptr::addr_of_mut!(hWndParent),
    );
    if !hwnd.is_null() {
        return DIALOG_DoDialogBox(hwnd, hWndParent) as INT_PTR;
    }
    -1
}

#[no_mangle]
pub unsafe extern "system" fn DialogBoxParamW(
    hInstance: HINSTANCE,
    lpTemplateName: LPCWSTR,
    mut hWndParent: HWND,
    lpDialogFunc: DLGPROC,
    dwInitParam: LPARAM,
) -> INT_PTR {
    let hwnd: HWND;
    let hrsrc: HRSRC;
    let ptr: LPCDLGTEMPLATE;

    hrsrc = FindResourceW(hInstance, lpTemplateName, RT_DIALOG as LPCWSTR);
    if hrsrc.is_null() {
        SetLastError(ERROR_RESOURCE_NAME_NOT_FOUND);
        return -1;
    }
    ptr = LoadResource(hInstance, hrsrc) as LPCDLGTEMPLATE;
    if ptr.is_null() {
        SetLastError(ERROR_RESOURCE_NAME_NOT_FOUND);
        return -1;
    }
    if !hWndParent.is_null() && IsWindow(hWndParent) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    hwnd = DIALOG_CreateIndirect(
        hInstance,
        ptr as LPCVOID,
        hWndParent,
        lpDialogFunc,
        dwInitParam,
        1u8 as BOOL,
        &mut hWndParent,
    );
    if !hwnd.is_null() {
        return DIALOG_DoDialogBox(hwnd, hWndParent) as INT_PTR;
    }
    -1
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirListA(
    hDlg: HWND,
    lpPathSpec: LPSTR,
    nIDListBox: INT,
    nIDStaticPath: INT,
    uFileType: UINT,
) -> INT {
    DIALOG_DlgDirListA(hDlg, lpPathSpec, nIDListBox, nIDStaticPath, uFileType, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirListComboBoxA(
    hDlg: HWND,
    lpPathSpec: LPSTR,
    nIDComboBox: INT,
    nIDStaticPath: INT,
    uFiletype: UINT,
) -> INT {
    DIALOG_DlgDirListA(hDlg, lpPathSpec, nIDComboBox, nIDStaticPath, uFiletype, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirListComboBoxW(
    hDlg: HWND,
    lpPathSpec: LPWSTR,
    nIDComboBox: core::ffi::c_int,
    nIDStaticPath: core::ffi::c_int,
    uFiletype: UINT,
) -> core::ffi::c_int {
    DIALOG_DlgDirListW(hDlg, lpPathSpec, nIDComboBox, nIDStaticPath, uFiletype, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirListW(
    hDlg: HWND,
    lpPathSpec: LPWSTR,
    nIDListBox: INT,
    nIDStaticPath: INT,
    uFileType: UINT,
) -> INT {
    DIALOG_DlgDirListW(hDlg, lpPathSpec, nIDListBox, nIDStaticPath, uFileType, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirSelectComboBoxExA(
    hDlg: HWND,
    lpString: LPSTR,
    nCount: core::ffi::c_int,
    nIDComboBox: core::ffi::c_int,
) -> BOOL {
    DIALOG_DlgDirSelect(hDlg, lpString as LPWSTR, nCount, nIDComboBox, 0, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirSelectComboBoxExW(
    hDlg: HWND,
    lpString: LPWSTR,
    nCount: INT,
    nIDComboBox: INT,
) -> BOOL {
    DIALOG_DlgDirSelect(hDlg, lpString as LPWSTR, nCount, nIDComboBox, 1, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirSelectExA(
    hDlg: HWND,
    lpString: LPSTR,
    nCount: INT,
    nIDListBox: INT,
) -> BOOL {
    DIALOG_DlgDirSelect(hDlg, lpString as LPWSTR, nCount, nIDListBox, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DlgDirSelectExW(
    hDlg: HWND,
    lpString: LPWSTR,
    nCount: core::ffi::c_int,
    nIDListBox: core::ffi::c_int,
) -> BOOL {
    DIALOG_DlgDirSelect(hDlg, lpString, nCount, nIDListBox, 1, 0)
}

#[no_mangle]
pub unsafe extern "system" fn EndDialog(hwnd: HWND, retval: INT_PTR) -> BOOL {
    let dlgInfo: *mut DIALOGINFO;
    let owner: HWND;
    let wasActive: BOOL;

    dlgInfo = GETDLGINFO(hwnd);
    if dlgInfo.is_null() {
        return 0u8 as BOOL;
    }
    wasActive = (hwnd == GetActiveWindow()) as BOOL;
    (*dlgInfo).idResult = retval as INT;
    (*dlgInfo).flags |= DF_END;

    if (GetWindowLongW(hwnd, GWL_STYLE) & ((WS_POPUP | WS_CHILD) as LONG)) == WS_CHILD as LONG {
        owner = GetAncestor(hwnd, GA_PARENT as UINT);
    } else {
        owner = GetWindow(hwnd, GW_OWNER as UINT);
    }

    if !owner.is_null() {
        EnableWindow(owner, 1u8 as BOOL);
    }

    if wasActive != 0 && IsChild(hwnd, GetFocus()) != 0 {
        SetFocus(hwnd);
    }

    SetWindowPos(
        hwnd,
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW) as UINT,
    );

    if wasActive != 0 && !owner.is_null() {
        SetActiveWindow(owner);
    } else if hwnd == GetActiveWindow() {
        NtUserCallNoParam(NOPARAM_ROUTINE_ZAPACTIVEANDFOUS as DWORD);
    }

    PostMessageA(hwnd, WM_NULL as UINT, 0, 0);
    1u8 as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetDialogBaseUnits() -> LONG {
    static mut units: DWORD = 0;

    if units == 0 {
        let hdc: HDC;
        let mut size: SIZE = core::mem::zeroed();

        hdc = GetDC(core::ptr::null_mut());
        if !hdc.is_null() {
            size.cx = GdiGetCharDimensions(hdc, core::ptr::null_mut(), &mut size.cy);
            if size.cx != 0 {
                units = MAKELONG(size.cx as u16, size.cy as u16) as DWORD;
            }
            ReleaseDC(core::ptr::null_mut(), hdc);
        }
    }
    units as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetDlgCtrlID(hwndCtl: HWND) -> core::ffi::c_int {
    GetWindowLongPtrW(hwndCtl, GWLP_ID) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "system" fn GetDlgItem(hDlg: HWND, nIDDlgItem: core::ffi::c_int) -> HWND {
    let mut i: core::ffi::c_int;
    let list: *mut HWND;
    let ret: HWND;

    if hDlg.is_null() {
        return core::ptr::null_mut();
    }

    list = WIN_ListChildren(hDlg);
    if list.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while !(*list.offset(i as isize)).is_null() {
        if GetWindowLongPtrW(*list.offset(i as isize), GWLP_ID) == nIDDlgItem as LONG_PTR {
            break;
        }
        i += 1;
    }
    ret = *list.offset(i as isize);
    HeapFree(GetProcessHeap(), 0, list as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetDlgItemInt(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    lpTranslated: *mut BOOL,
    bSigned: BOOL,
) -> UINT {

    let mut str: [core::ffi::c_char; 30] = [0; 30];
    let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut();
    let result: LONG_PTR;

    if !lpTranslated.is_null() {
        *lpTranslated = 0;
    }
    if SendDlgItemMessageA(
        hDlg,
        nIDDlgItem,
        WM_GETTEXT,
        core::mem::size_of::<[core::ffi::c_char; 30]>() as WPARAM,
        str.as_mut_ptr() as LPARAM,
    ) == 0
    {
        return 0;
    }
    if bSigned != 0 {
        let res = strtol(str.as_ptr(), &mut endptr, 10);
        result = res as LONG_PTR;
        if endptr.is_null() || (endptr == str.as_mut_ptr()) {
            return 0;
        }
        if (result as core::ffi::c_long == LONG_MIN as core::ffi::c_long)
            || (result as core::ffi::c_long == LONG_MAX as core::ffi::c_long)
        {
            return 0;
        }
    } else {
        let res = strtoul(str.as_ptr(), &mut endptr, 10);
        result = res as LONG_PTR;
        if endptr.is_null() || (endptr == str.as_mut_ptr()) {
            return 0;
        }
        if result as core::ffi::c_ulong == ULONG_MAX as core::ffi::c_ulong {
            return 0;
        }
    }
    if !lpTranslated.is_null() {
        *lpTranslated = 1;
    }
    result as UINT
}

#[no_mangle]
pub unsafe extern "system" fn GetDlgItemTextA(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    lpString: LPSTR,
    nMaxCount: core::ffi::c_int,
) -> UINT {

    let hWnd = GetDlgItem(hDlg, nIDDlgItem);
    if !hWnd.is_null() {
        return GetWindowTextA(hWnd, lpString, nMaxCount) as UINT;
    }
    if nMaxCount != 0 {
        *lpString = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetDlgItemTextW(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    lpString: LPWSTR,
    nMaxCount: core::ffi::c_int,
) -> UINT {

    let hWnd = GetDlgItem(hDlg, nIDDlgItem);
    if !hWnd.is_null() {
        return GetWindowTextW(hWnd, lpString, nMaxCount) as UINT;
    }
    if nMaxCount != 0 {
        *lpString = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetNextDlgGroupItem(
    hDlg: HWND,
    mut hCtl: HWND,
    bPrevious: BOOL,
) -> HWND {
    let mut hwnd: HWND;
    let mut hwndNext: HWND;
    let mut retvalue: HWND;
    let mut hwndLastGroup: HWND = core::ptr::null_mut();
    let mut fLooped: BOOL = 0;
    let mut fSkipping: BOOL = 0;

    if hDlg == hCtl {
        hCtl = core::ptr::null_mut();
    }
    if hCtl.is_null() && bPrevious != 0 {
        return core::ptr::null_mut();
    }

    if !hCtl.is_null() {
        if IsChild(hDlg, hCtl) == 0 {
            return core::ptr::null_mut();
        }
    } else {
        hCtl = GetWindow(hDlg, GW_CHILD as u32);
        if hCtl.is_null() {
            return core::ptr::null_mut();
        }

        if (GetWindowLongPtrW(hCtl, GWL_STYLE) as u32 & (WS_VISIBLE | WS_DISABLED))
            == WS_VISIBLE
        {
            return hCtl;
        }
    }

    retvalue = hCtl;
    hwnd = hCtl;

    'end: loop {
        loop {
            hwndNext = GetWindow(hwnd, GW_HWNDNEXT as u32);
            while hwndNext.is_null() {
                if hDlg == GetParent(hwnd) {
                    if fLooped != 0 {
                        break 'end;
                    }
                    fLooped = 1;
                    hwndNext = GetWindow(hDlg, GW_CHILD as u32);
                } else {
                    hwnd = GetParent(hwnd);
                    hwndNext = GetWindow(hwnd, GW_HWNDNEXT as u32);
                }
            }
            hwnd = hwndNext;

            while (GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_CONTROLPARENT) != 0
                && (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & (WS_VISIBLE | WS_DISABLED))
                    == WS_VISIBLE
            {
                hwndNext = GetWindow(hwnd, GW_CHILD as u32);
                if hwndNext.is_null() {
                    break;
                }
                hwnd = hwndNext;
            }

            if (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_GROUP) != 0 {
                hwndLastGroup = hwnd;
                if fSkipping == 0 {
                    fSkipping = 1;
                }
            }

            if hwnd == hCtl {
                if fSkipping == 0 {
                    break 'end;
                }
                if hwndLastGroup == hwnd {
                    break 'end;
                }
                hwnd = hwndLastGroup;
                fSkipping = 0;
                fLooped = 0;
            }

            if fSkipping == 0
                && (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & (WS_VISIBLE | WS_DISABLED))
                    == WS_VISIBLE
            {
                retvalue = hwnd;
                if bPrevious == 0 {
                    break 'end;
                }
            }
        }
    }

    retvalue
}

#[no_mangle]
pub unsafe extern "system" fn GetNextDlgTabItem(
    hDlg: HWND,
    mut hCtl: HWND,
    bPrevious: BOOL,
) -> HWND {
    let mut pWindow: PWND;

    pWindow = ValidateHwnd(hDlg);
    if pWindow.is_null() {
        return core::ptr::null_mut();
    }
    if !hCtl.is_null() {
        pWindow = ValidateHwnd(hCtl);
        if pWindow.is_null() {
            return core::ptr::null_mut();
        }
    }

    if hDlg == hCtl {
        hCtl = core::ptr::null_mut();
    }

    if hCtl.is_null() && bPrevious != 0 {
        return core::ptr::null_mut();
    }

    DIALOG_GetNextTabItem(hDlg, hDlg, hCtl, bPrevious)
}

// EnumChildWindows callback — ABI must match WNDENUMPROC: stdcall on i386, cdecl
// on amd64/arm64.
unsafe extern "system" fn DIALOG_FixOneChildOnChangeFocus(hwndChild: HWND, lParam: LPARAM) -> BOOL {
    DIALOG_FixOneChildOnChangeFocus_impl(hwndChild, lParam)
}
#[inline]
unsafe fn DIALOG_FixOneChildOnChangeFocus_impl(hwndChild: HWND, _lParam: LPARAM) -> BOOL {
    if (DLGC_DEFPUSHBUTTON as LRESULT) & SendMessageW(hwndChild, WM_GETDLGCODE, 0, 0) != 0 {
        SendMessageW(hwndChild, BM_SETSTYLE, BS_PUSHBUTTON as WPARAM, TRUE as LPARAM);
    }
    TRUE as BOOL
}

unsafe fn DIALOG_FixChildrenOnChangeFocus(hwndDlg: HWND, hwndNext: HWND) {
    let dlgcode_next: INT = SendMessageW(hwndNext, WM_GETDLGCODE, 0, 0) as INT;

    EnumChildWindows(hwndDlg, Some(DIALOG_FixOneChildOnChangeFocus), 0);

    if (dlgcode_next & DLGC_DEFPUSHBUTTON as INT) != 0 {
        let mut def_id: DWORD = SendMessageW(hwndDlg, DM_GETDEFID, 0, 0) as DWORD;
        if HIWORD(def_id) == DC_HASDEFID as _ {
            def_id = LOWORD(def_id) as DWORD;
            let hwndDef: HWND = GetDlgItem(hwndDlg, def_id as core::ffi::c_int);
            if !hwndDef.is_null() {
                let dlgcode_def: INT = SendMessageW(hwndDef, WM_GETDLGCODE, 0, 0) as INT;
                if (dlgcode_def & DLGC_BUTTON as INT) != 0
                    && (dlgcode_def & DLGC_UNDEFPUSHBUTTON as INT) != 0
                {
                    SendMessageW(hwndDef, BM_SETSTYLE, BS_DEFPUSHBUTTON as WPARAM, TRUE as LPARAM);
                }
            }
        }
    } else if (dlgcode_next & DLGC_BUTTON as INT) != 0
        && (dlgcode_next & DLGC_UNDEFPUSHBUTTON as INT) != 0
    {
        SendMessageW(hwndNext, BM_SETSTYLE, BS_DEFPUSHBUTTON as WPARAM, TRUE as LPARAM);
    }
}

unsafe fn DIALOG_IdToHwnd(hwndDlg: HWND, id: INT) -> HWND {
    let mut i: i32;
    let list: *mut HWND = WIN_ListChildren(hwndDlg);
    let mut ret: HWND = core::ptr::null_mut();

    if list.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while !(*list.offset(i as isize)).is_null() {
        if GetWindowLongPtrW(*list.offset(i as isize), GWLP_ID as i32) == id as LONG_PTR {
            ret = *list.offset(i as isize);
            break;
        }

        ret = DIALOG_IdToHwnd(*list.offset(i as isize), id);
        if !ret.is_null() {
            break;
        }

        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, list as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn IsDialogMessageW(mut hDlg: HWND, lpMsg: LPMSG) -> BOOL {
    let mut dlgCode: INT = 0;

    if IsWindow(hDlg) == 0 {
        return 0;
    }

    if CallMsgFilterW(lpMsg, MSGF_DIALOGBOX as INT) != 0 {
        return 1;
    }

    if hDlg == GetDesktopWindow() {
        return 0;
    }
    if hDlg != (*lpMsg).hwnd && IsChild(hDlg, (*lpMsg).hwnd) == 0 {
        return 0;
    }

    hDlg = DIALOG_FindMsgDestination(hDlg);

    match (*lpMsg).message {
        m if m == WM_KEYDOWN => {
            dlgCode = SendMessageW((*lpMsg).hwnd, WM_GETDLGCODE, (*lpMsg).wParam, lpMsg as LPARAM) as INT;
            if dlgCode & DLGC_WANTMESSAGE as INT != 0 {
                // break out of outer switch
            } else {
                match (*lpMsg).wParam as core::ffi::c_int {
                    x if x == VK_TAB as core::ffi::c_int => {
                        if dlgCode & DLGC_WANTTAB as INT == 0 {
                            let mut fIsDialog: BOOL = 1;
                            let pWnd = ValidateHwnd(hDlg);

                            if !pWnd.is_null() && TestWindowProcess(pWnd) != 0 {
                                fIsDialog = if GETDLGINFO(hDlg) != core::ptr::null_mut() { 1 } else { 0 };
                            }

                            SendMessageW(hDlg, WM_CHANGEUISTATE,
                                MAKEWPARAM(UIS_CLEAR as u16, UISF_HIDEFOCUS as u16) as WPARAM, 0);

                            if fIsDialog != 0 {
                                SendMessageW(hDlg, WM_NEXTDLGCTL,
                                    (GetKeyState(VK_SHIFT as INT) as INT & 0x8000) as WPARAM, 0);
                            } else {
                                let hwndFocus = GetFocus();
                                let hwndNext = GetNextDlgTabItem(hDlg,
                                    if hwndFocus == hDlg { core::ptr::null_mut() } else { hwndFocus },
                                    (GetKeyState(VK_SHIFT as INT) as INT & 0x8000) as BOOL);
                                if !hwndNext.is_null() {
                                    dlgCode = SendMessageW(hwndNext, WM_GETDLGCODE,
                                        (*lpMsg).wParam, lpMsg as LPARAM) as INT;
                                    if dlgCode & DLGC_HASSETSEL as INT != 0 {
                                        let maxlen: INT = 1 + SendMessageW(hwndNext, WM_GETTEXTLENGTH, 0, 0) as INT;
                                        let buffer = HeapAlloc(GetProcessHeap(), 0,
                                            ((maxlen as usize) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
                                        if !buffer.is_null() {
                                            SendMessageW(hwndNext, WM_GETTEXT, maxlen as WPARAM, buffer as LPARAM);
                                            let length = strlenW(buffer);
                                            HeapFree(GetProcessHeap(), 0, buffer as LPVOID);
                                            SendMessageW(hwndNext, EM_SETSEL, 0, length as LPARAM);
                                        }
                                    }
                                    SetFocus(hwndNext);
                                    DIALOG_FixChildrenOnChangeFocus(hDlg, hwndNext);
                                } else {
                                    return 0;
                                }
                            }
                            return 1;
                        }
                    }
                    x if x == VK_RIGHT as core::ffi::c_int
                        || x == VK_DOWN as core::ffi::c_int
                        || x == VK_LEFT as core::ffi::c_int
                        || x == VK_UP as core::ffi::c_int =>
                    {
                        if dlgCode & DLGC_WANTARROWS as INT == 0 {
                            let fPrevious: BOOL = if (*lpMsg).wParam as core::ffi::c_int == VK_LEFT as core::ffi::c_int
                                || (*lpMsg).wParam as core::ffi::c_int == VK_UP as core::ffi::c_int { 1 } else { 0 };

                            let hwndFirst = (*lpMsg).hwnd;
                            let mut hwndNext = GetNextDlgGroupItem(hDlg, hwndFirst, fPrevious);
                            while !hwndNext.is_null() && hwndFirst != hwndNext {
                                if SendMessageW(hwndNext, WM_GETDLGCODE, 0, 0) & DLGC_STATIC as LRESULT == 0 {
                                    break;
                                }
                                hwndNext = GetNextDlgGroupItem(hDlg, hwndNext, fPrevious);
                            }

                            if !hwndNext.is_null()
                                && (SendMessageW(hwndNext, WM_GETDLGCODE, (*lpMsg).wParam, lpMsg as LPARAM)
                                    & (DLGC_BUTTON as LRESULT | DLGC_RADIOBUTTON as LRESULT))
                                    == (DLGC_BUTTON as LRESULT | DLGC_RADIOBUTTON as LRESULT)
                            {
                                SetFocus(hwndNext);
                                if (GetWindowLongW(hwndNext, GWL_STYLE) as u32 & BS_TYPEMASK) == BS_AUTORADIOBUTTON
                                    && SendMessageW(hwndNext, BM_GETCHECK, 0, 0) != BST_CHECKED as LRESULT
                                {
                                    SendMessageW(hwndNext, BM_CLICK, 0, 0);
                                }
                            } else {
                                SendMessageW(hDlg, WM_NEXTDLGCTL, hwndNext as WPARAM, 1);
                            }
                            return 1;
                        }
                    }
                    x if x == VK_CANCEL as core::ffi::c_int
                        || x == VK_ESCAPE as core::ffi::c_int =>
                    {
                        SendMessageW(hDlg, WM_COMMAND, IDCANCEL as WPARAM,
                            GetDlgItem(hDlg, IDCANCEL as INT) as LPARAM);
                        return 1;
                    }
                    x if x == VK_EXECUTE as core::ffi::c_int
                        || x == VK_RETURN as core::ffi::c_int =>
                    {
                        let hwndFocus = GetFocus();
                        if IsChild(hDlg, hwndFocus) != 0
                            && SendMessageW(hwndFocus, WM_GETDLGCODE, 0, 0) & DLGC_DEFPUSHBUTTON as LRESULT != 0
                        {
                            SendMessageW(hDlg, WM_COMMAND,
                                MAKEWPARAM(GetDlgCtrlID(hwndFocus) as u16, BN_CLICKED as u16) as WPARAM,
                                hwndFocus as LPARAM);
                        } else {
                            let dw = SendMessageW(hDlg, DM_GETDEFID, 0, 0) as DWORD;
                            if DC_HASDEFID == HIWORD(dw) as u32 {
                                let hwndDef = DIALOG_IdToHwnd(hDlg, LOWORD(dw) as INT);
                                if hwndDef.is_null() || IsWindowEnabled(hwndDef) != 0 {
                                    SendMessageW(hDlg, WM_COMMAND,
                                        MAKEWPARAM(LOWORD(dw), BN_CLICKED as u16) as WPARAM,
                                        hwndDef as LPARAM);
                                }
                            } else {
                                SendMessageW(hDlg, WM_COMMAND, IDOK as WPARAM,
                                    GetDlgItem(hDlg, IDOK as INT) as LPARAM);
                            }
                        }
                        return 1;
                    }
                    _ => {}
                }
            }
        }
        m if m == WM_CHAR => {
            dlgCode = SendMessageW((*lpMsg).hwnd, WM_GETDLGCODE, (*lpMsg).wParam, lpMsg as LPARAM) as INT;
            if dlgCode & (DLGC_WANTCHARS as INT | DLGC_WANTMESSAGE as INT) != 0 {
                // break
            } else if (*lpMsg).wParam == b'\t' as WPARAM && dlgCode & DLGC_WANTTAB as INT != 0 {
                // break
            } else {
                if DIALOG_IsAccelerator((*lpMsg).hwnd, hDlg, (*lpMsg).wParam) != 0 {
                    return 1;
                }
            }
        }
        m if m == WM_SYSCHAR => {
            if DIALOG_IsAccelerator((*lpMsg).hwnd, hDlg, (*lpMsg).wParam) != 0 {
                return 1;
            }
        }
        m if m == WM_SYSKEYDOWN => {
            if HIWORD((*lpMsg).lParam as DWORD) as u32 & KF_ALTDOWN != 0
                && !gpsi.is_null()
                && ((*gpsi).dwSRVIFlags & SRVINFO_KBDPREF) == 0
                && (crate::wow64::psi::pusi_flags() & PUSIF_KEYBOARDCUES) == 0
            {
                SendMessageW(hDlg, WM_CHANGEUISTATE,
                    MAKEWPARAM(UIS_CLEAR as u16, (UISF_HIDEACCEL | UISF_HIDEFOCUS) as u16) as WPARAM, 0);
            }
        }
        m if m == WM_SYSCOMMAND => {
            if (*lpMsg).wParam == SC_KEYMENU as WPARAM
                && !gpsi.is_null()
                && ((*gpsi).dwSRVIFlags & SRVINFO_KBDPREF) == 0
                && (crate::wow64::psi::pusi_flags() & PUSIF_KEYBOARDCUES) == 0
            {
                SendMessageW(hDlg, WM_CHANGEUISTATE,
                    MAKEWPARAM(UIS_CLEAR as u16, (UISF_HIDEACCEL | UISF_HIDEFOCUS) as u16) as WPARAM, 0);
            }
        }
        _ => {}
    }

    TranslateMessage(lpMsg);
    DispatchMessageW(lpMsg);
    1
}

#[no_mangle]
pub unsafe extern "system" fn IsDlgButtonChecked(hDlg: HWND, nIDButton: core::ffi::c_int) -> UINT {
    SendDlgItemMessageW(hDlg, nIDButton, BM_GETCHECK, 0, 0) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn MapDialogRect(hDlg: HWND, lpRect: LPRECT) -> BOOL {
    let dlgInfo = GETDLGINFO(hDlg);
    if dlgInfo.is_null() {
        return 0;
    }
    (*lpRect).left = MulDiv((*lpRect).left, (*dlgInfo).xBaseUnit as i32, 4);
    (*lpRect).right = MulDiv((*lpRect).right, (*dlgInfo).xBaseUnit as i32, 4);
    (*lpRect).top = MulDiv((*lpRect).top, (*dlgInfo).yBaseUnit as i32, 8);
    (*lpRect).bottom = MulDiv((*lpRect).bottom, (*dlgInfo).yBaseUnit as i32, 8);
    1
}

#[no_mangle]
pub unsafe extern "system" fn SendDlgItemMessageA(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let hwndCtrl: HWND;
    if hDlg == HWND_TOPMOST || hDlg == HWND_BROADCAST {
        return 0;
    }
    hwndCtrl = GetDlgItem(hDlg, nIDDlgItem);
    if !hwndCtrl.is_null() {
        SendMessageA(hwndCtrl, Msg, wParam, lParam)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SendDlgItemMessageW(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {

    let hwndCtrl: HWND;
    if hDlg == (-1isize as HWND) || hDlg == (0xffffusize as HWND) {
        return 0;
    }
    hwndCtrl = GetDlgItem(hDlg, nIDDlgItem);
    if !hwndCtrl.is_null() {
        SendMessageW(hwndCtrl, Msg, wParam, lParam)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetDlgItemInt(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    uValue: UINT,
    bSigned: BOOL,
) -> BOOL {

    let mut str: [core::ffi::c_char; 20] = [0; 20];

    if bSigned != 0 {
        sprintf(str.as_mut_ptr(), b"%d\0".as_ptr() as *const core::ffi::c_char, uValue as core::ffi::c_int);
    } else {
        sprintf(str.as_mut_ptr(), b"%u\0".as_ptr() as *const core::ffi::c_char, uValue);
    }
    SendDlgItemMessageA(hDlg, nIDDlgItem, WM_SETTEXT, 0, str.as_mut_ptr() as LPARAM);
    1
}

#[no_mangle]
pub unsafe extern "system" fn SetDlgItemTextA(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    lpString: LPCSTR,
) -> BOOL {

    let hwndCtrl = GetDlgItem(hDlg, nIDDlgItem);
    if !hwndCtrl.is_null() {
        return SetWindowTextA(hwndCtrl, lpString);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetDlgItemTextW(
    hDlg: HWND,
    nIDDlgItem: core::ffi::c_int,
    lpString: LPCWSTR,
) -> BOOL {

    let hwndCtrl = GetDlgItem(hDlg, nIDDlgItem);
    if !hwndCtrl.is_null() {
        return SetWindowTextW(hwndCtrl, lpString);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CheckDlgButton(
    hDlg: HWND,
    nIDButton: core::ffi::c_int,
    uCheck: UINT,
) -> BOOL {
    SendDlgItemMessageW(hDlg, nIDButton, BM_SETCHECK, uCheck as WPARAM, 0);
    1
}

// EnumChildWindows callback — ABI must match WNDENUMPROC: stdcall on i386, cdecl
// on amd64/arm64.
unsafe extern "system" fn CheckRB(hwnd: HWND, lParam: LPARAM) -> BOOL {
    CheckRB_impl(hwnd, lParam)
}
#[inline]
unsafe fn CheckRB_impl(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let lChildID = GetWindowLongPtrW(hwnd, GWLP_ID as i32) as LONG;
    let lpRadioGroup = lParam as *mut RADIOGROUP;

    if (lChildID >= (*lpRadioGroup).firstID as LONG)
        && (lChildID <= (*lpRadioGroup).lastID as LONG)
    {
        if lChildID == (*lpRadioGroup).checkID as LONG {
            SendMessageW(hwnd, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        } else {
            SendMessageW(hwnd, BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CheckRadioButton(
    hDlg: HWND,
    nIDFirstButton: core::ffi::c_int,
    nIDLastButton: core::ffi::c_int,
    nIDCheckButton: core::ffi::c_int,
) -> BOOL {
    let mut radioGroup: RADIOGROUP = core::mem::zeroed();

    radioGroup.firstID = nIDFirstButton as UINT;
    radioGroup.lastID = nIDLastButton as UINT;
    radioGroup.checkID = nIDCheckButton as UINT;

    EnumChildWindows(
        hDlg,
        Some(CheckRB),
        core::ptr::addr_of_mut!(radioGroup) as LPARAM,
    )
}

// ─── DIALOG_builtin_class — cross-TU data symbol read by controls/regcontrol.c ───
// Faithful to windows/dialog.c:109. Must export the exact (unmangled) symbol name.
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
pub static DIALOG_builtin_class: builtin_class_descr = builtin_class_descr {
    name: WC_DIALOG as LPCWSTR,
    style: CS_SAVEBITS | CS_DBLCLKS,
    procA: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(DefDlgProcA)
    },
    procW: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(DefDlgProcW)
    },
    extra: DLGWINDOWEXTRA as INT,
    cursor: 32512 as LPCWSTR, // IDC_ARROW
    brush: 0 as HBRUSH,
};
