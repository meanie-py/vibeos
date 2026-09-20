//! The Win32 surface this library uses, declared by hand.
//!
//! Hand-written rather than generated: the list is ~50 entries of the most
//! stable API in Windows, and a bindgen island would tie the crate to one
//! tree's headers — this crate also builds against the Windows SDK for the
//! native validation harness.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

pub type HANDLE = *mut c_void;
pub type HWND = HANDLE;
pub type HDC = HANDLE;
pub type HBITMAP = HANDLE;
pub type HBRUSH = HANDLE;
pub type HPEN = HANDLE;
pub type HGDIOBJ = HANDLE;
pub type HINSTANCE = HANDLE;
pub type HMODULE = HANDLE;
pub type HCURSOR = HANDLE;
pub type HICON = HANDLE;
pub type HMENU = HANDLE;
pub type COLORREF = u32;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type UINT_PTR = usize;

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>;
pub type TIMERPROC = Option<unsafe extern "system" fn(HWND, u32, UINT_PTR, u32)>;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: c_int,
    pub rcPaint: RECT,
    pub fRestore: c_int,
    pub fIncUpdate: c_int,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BITMAP {
    pub bmType: c_int,
    pub bmWidth: c_int,
    pub bmHeight: c_int,
    pub bmWidthBytes: c_int,
    pub bmPlanes: u16,
    pub bmBitsPixel: u16,
    pub bmBits: *mut c_void,
}

impl Default for BITMAP {
    fn default() -> Self {
        // SAFETY: BITMAP is plain data; an all-zero one is a valid "unset".
        unsafe { core::mem::zeroed() }
    }
}

// ---- window styles / messages -------------------------------------------

pub const CS_DBLCLKS: u32 = 0x0008;
pub const CS_OWNDC: u32 = 0x0020;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_SETCURSOR: u32 = 0x0020;

pub const GWLP_USERDATA: c_int = -21;

pub const IDC_ARROW: usize = 32512;

pub const SRCCOPY: u32 = 0x00CC_0020;
pub const SRCAND: u32 = 0x0088_00C6;
pub const SRCPAINT: u32 = 0x00EE_0086;
pub const PATCOPY: u32 = 0x00F0_0021;
pub const BLACKNESS: u32 = 0x0000_0042;
pub const WHITENESS: u32 = 0x00FF_0062;

pub const NULL_BRUSH: c_int = 5;

pub const PS_SOLID: c_int = 0;

pub const BITSPIXEL: c_int = 12;

#[inline]
pub const fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

#[inline]
pub const fn GetRValue(c: COLORREF) -> u8 {
    (c & 0xFF) as u8
}
#[inline]
pub const fn GetGValue(c: COLORREF) -> u8 {
    ((c >> 8) & 0xFF) as u8
}
#[inline]
pub const fn GetBValue(c: COLORREF) -> u8 {
    ((c >> 16) & 0xFF) as u8
}

#[inline]
pub fn LOWORD_SIGNED(l: LPARAM) -> c_int {
    (l & 0xFFFF) as u16 as i16 as c_int
}
#[inline]
pub fn HIWORD_SIGNED(l: LPARAM) -> c_int {
    ((l >> 16) & 0xFFFF) as u16 as i16 as c_int
}

extern "system" {
    // kernel32
    pub fn GetTickCount() -> u32;
    pub fn ExitProcess(code: u32) -> !;
    pub fn DebugBreak();
    pub fn LoadLibraryW(name: *const u16) -> HMODULE;
    pub fn FreeLibrary(module: HMODULE) -> c_int;
    pub fn GetModuleHandleW(name: *const u16) -> HMODULE;

    // user32
    pub fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        parent: HWND,
        menu: HMENU,
        instance: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DestroyWindow(hwnd: HWND) -> c_int;
    pub fn DefWindowProcW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn SetWindowLongPtrW(hwnd: HWND, index: c_int, new: isize) -> isize;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: c_int) -> isize;
    pub fn BeginPaint(hwnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, ps: *const PAINTSTRUCT) -> c_int;
    pub fn InvalidateRect(hwnd: HWND, rect: *const RECT, erase: c_int) -> c_int;
    pub fn UpdateWindow(hwnd: HWND) -> c_int;
    pub fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> c_int;
    pub fn GetDC(hwnd: HWND) -> HDC;
    pub fn ReleaseDC(hwnd: HWND, hdc: HDC) -> c_int;
    pub fn SetCapture(hwnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> c_int;
    pub fn LoadCursorW(instance: HINSTANCE, name: usize) -> HCURSOR;
    pub fn LoadBitmapW(instance: HINSTANCE, name: usize) -> HBITMAP;
    pub fn SetTimer(hwnd: HWND, id: UINT_PTR, elapse: u32, proc_: TIMERPROC) -> UINT_PTR;
    pub fn KillTimer(hwnd: HWND, id: UINT_PTR) -> c_int;
    pub fn FillRect(hdc: HDC, rect: *const RECT, brush: HBRUSH) -> c_int;
    pub fn Sleep(ms: u32);

    // gdi32
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, w: c_int, h: c_int) -> HBITMAP;
    pub fn DeleteDC(hdc: HDC) -> c_int;
    pub fn DeleteObject(obj: HGDIOBJ) -> c_int;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn GetStockObject(index: c_int) -> HGDIOBJ;
    pub fn BitBlt(
        dst: HDC,
        x: c_int,
        y: c_int,
        w: c_int,
        h: c_int,
        src: HDC,
        sx: c_int,
        sy: c_int,
        rop: u32,
    ) -> c_int;
    pub fn PatBlt(hdc: HDC, x: c_int, y: c_int, w: c_int, h: c_int, rop: u32) -> c_int;
    pub fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    pub fn CreatePen(style: c_int, width: c_int, color: COLORREF) -> HPEN;
    pub fn MoveToEx(hdc: HDC, x: c_int, y: c_int, pt: *mut POINT) -> c_int;
    pub fn LineTo(hdc: HDC, x: c_int, y: c_int) -> c_int;
    pub fn GetPixel(hdc: HDC, x: c_int, y: c_int) -> COLORREF;
    pub fn SetPixel(hdc: HDC, x: c_int, y: c_int, color: COLORREF) -> COLORREF;
    pub fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;
    pub fn GetObjectW(obj: HGDIOBJ, size: c_int, out: *mut c_void) -> c_int;
    pub fn IntersectClipRect(hdc: HDC, l: c_int, t: c_int, r: c_int, b: c_int) -> c_int;
    pub fn SelectClipRgn(hdc: HDC, rgn: HGDIOBJ) -> c_int;
    pub fn ExcludeClipRect(hdc: HDC, l: c_int, t: c_int, r: c_int, b: c_int) -> c_int;
}
