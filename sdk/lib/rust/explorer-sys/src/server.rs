//! explorer_sys::server — the ATL/WTL replacement substrate for the tier-C
//! explorer COM/window-server classes (trayclock, trayntfy, syspager,
//! taskswnd, traywnd, …). ATL's `CWindowImpl`/`CComObjectRootEx`/`CComCoClass`
//! templates compile down to a handful of ABI contracts; this module provides
//! the Rust equivalents so each translated class is a flat `#[repr(C)]`
//! `lpVtbl`-first object (ks-rust `filterfactory.rs` COM shape) plus a
//! match-based WndProc (the `BEGIN_MSG_MAP` replacement).
//!
//! What lives here (reusable across every tier-C class):
//!   1. **Window-class registration** — `register_class_once` (idempotent
//!      RegisterClassExW, the `DECLARE_WND_CLASS_EX` replacement).
//!   2. **`this`-recovery** — `set_window_this`/`get_window_this` over
//!      GWLP_USERDATA, ARCH-SPLIT (i386 `SetWindowLongW`, LLP64
//!      `SetWindowLongPtrW`; on i386 the *Ptr* names are C macros with no
//!      import). A class's static WndProc stashes `this` on WM_NCCREATE and
//!      recovers it on every later message — the CWindowImpl thunk equivalent.
//!   3. **uxtheme / tooltip / ShellExecute** surface winsrv-sys stops short of,
//!      + the theme-part/tooltip constants and `TOOLINFOW` layout.
//!   4. **word helpers** (LOWORD/HIWORD/MAKELPARAM) the msgmap handlers use.
//!
//! The per-class COM boilerplate (QueryInterface IID-walk, interlocked
//! AddRef/Release, the vtbl static, HeapAlloc/HeapFree new/delete, the
//! `*_CreateInstance` factory) stays in each translated module — it is short
//! and class-specific (mirrors ks-rust); only the genuinely shared machinery
//! is centralised here.

use core::ffi::c_void;
use winsrv_sys::winsrv::{
    BOOL, DWORD, HBRUSH, HINSTANCE, HWND, INT, LPARAM, LPCWSTR, LPWSTR, RECT, UINT, UINT_PTR,
    WNDCLASSEXW, WNDPROC,
};

/// `IDC_ARROW` (winuser.h) as MAKEINTRESOURCE — the default class cursor
/// DECLARE_WND_CLASS_EX loads.
const IDC_ARROW: LPCWSTR = 32512 as usize as LPCWSTR;

// ---------------------------------------------------------------------------
// Interlocked refcount — CComObjectRootEx AddRef/Release use
// InterlockedIncrement/Decrement, which are compiler intrinsics (no DLL
// export), so bindgen never surfaced them. Model them on a core AtomicI32 view
// of the LONG `ref_` field (LONG is 32-bit on every Windows target). Every
// tier-C class's AddRef/Release routes through these.
// ---------------------------------------------------------------------------

use core::sync::atomic::{AtomicI32, Ordering};

/// `InterlockedIncrement` — returns the NEW value.
#[inline]
pub unsafe fn interlocked_increment(p: *mut i32) -> i32 {
    (*(p as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
/// `InterlockedDecrement` — returns the NEW value.
#[inline]
pub unsafe fn interlocked_decrement(p: *mut i32) -> i32 {
    (*(p as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}

// ---------------------------------------------------------------------------
// Window-class registration — DECLARE_WND_CLASS_EX(name, style, bkgnd)
// ---------------------------------------------------------------------------

/// Register `class_name` once (idempotent). ATL registers a CWindowImpl's
/// class lazily on first `Create()`; we mirror that with a GetClassInfoExW
/// probe so repeated instantiations don't fail with ERROR_CLASS_ALREADY_EXISTS.
/// `bkgnd_color` is a `COLOR_*` index (as in DECLARE_WND_CLASS_EX); the brush
/// is `(HBRUSH)(color + 1)`, exactly what ATL stores. Pass `bkgnd_color < 0`
/// for no background brush.
#[inline]
pub unsafe fn register_class_once(
    hinst: HINSTANCE,
    class_name: LPCWSTR,
    wndproc: WNDPROC,
    style: UINT,
    bkgnd_color: i32,
) -> bool {
    let mut existing: WNDCLASSEXW = core::mem::zeroed();
    existing.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    if winsrv_sys::winsrv::GetClassInfoExW(hinst, class_name, &mut existing) != 0 {
        return true; // already registered — reuse
    }

    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.style = style;
    wc.lpfnWndProc = wndproc;
    wc.hInstance = hinst;
    wc.hbrBackground = if bkgnd_color >= 0 {
        (bkgnd_color as usize + 1) as HBRUSH
    } else {
        core::ptr::null_mut()
    };
    wc.hCursor = winsrv_sys::winsrv::LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
    wc.lpszClassName = class_name;
    winsrv_sys::winsrv::RegisterClassExW(&wc) != 0
}

// ---------------------------------------------------------------------------
// `this`-recovery over GWLP_USERDATA — the CWindowImpl thunk replacement.
// ARCH-SPLIT: on i386 SetWindowLongPtrW/GetWindowLongPtrW are C macros that
// resolve to SetWindowLongW/GetWindowLongW (LONG is pointer-wide on ILP32),
// so no *Ptr* import exists there; on amd64/arm64 the real *Ptr* fns take
// LONG_PTR. Same distinct-Rust-type-per-ABI trap as the arch-split callbacks.
// ---------------------------------------------------------------------------

/// GWLP_USERDATA (winuser.h) — the per-window slot we stash `this` in.
pub const GWLP_USERDATA: INT = -21;

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn set_window_this(hwnd: HWND, this: *mut c_void) {
    winsrv_sys::winsrv::SetWindowLongW(hwnd, GWLP_USERDATA, this as winsrv_sys::winsrv::LONG);
}
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn set_window_this(hwnd: HWND, this: *mut c_void) {
    winsrv_sys::winsrv::SetWindowLongPtrW(
        hwnd,
        GWLP_USERDATA,
        this as winsrv_sys::winsrv::LONG_PTR,
    );
}

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn get_window_this(hwnd: HWND) -> *mut c_void {
    winsrv_sys::winsrv::GetWindowLongW(hwnd, GWLP_USERDATA) as *mut c_void
}
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn get_window_this(hwnd: HWND) -> *mut c_void {
    winsrv_sys::winsrv::GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut c_void
}

/// On WM_NCCREATE, `lParam` is `CREATESTRUCTW*`; its first field
/// `lpCreateParams` is the `this` we passed as CreateWindowEx's last arg.
/// Stash it in GWLP_USERDATA and return it. Call once from a class WndProc's
/// WM_NCCREATE arm before dispatching.
#[inline]
pub unsafe fn wndproc_nccreate_this(hwnd: HWND, lparam: LPARAM) -> *mut c_void {
    let cs = lparam as *const winsrv_sys::winsrv::CREATESTRUCTW;
    let this = (*cs).lpCreateParams;
    set_window_this(hwnd, this);
    this
}

// ---------------------------------------------------------------------------
// word/param helpers (windowsx.h / windef.h macros the msgmap bodies use)
// ---------------------------------------------------------------------------

#[inline]
pub fn LOWORD(l: usize) -> u16 {
    (l & 0xFFFF) as u16
}
#[inline]
pub fn HIWORD(l: usize) -> u16 {
    ((l >> 16) & 0xFFFF) as u16
}
/// signed 16-bit extraction (GET_X_LPARAM / mouse coords)
#[inline]
pub fn GET_X_LPARAM(lp: LPARAM) -> i32 {
    (lp & 0xFFFF) as i16 as i32
}
#[inline]
pub fn GET_Y_LPARAM(lp: LPARAM) -> i32 {
    ((lp >> 16) & 0xFFFF) as i16 as i32
}

// ---------------------------------------------------------------------------
// uxtheme.dll (in explorer's add_importlibs; stdcall) + theme-part constants
// ---------------------------------------------------------------------------

/// `HTHEME` — opaque theme handle (uxtheme.h).
pub type HTHEME = *mut c_void;

/// vssym32.h `enum CLOCKPARTS`
pub const CLP_TIME: INT = 1;
/// tmschema.h theme properties used by the clock
pub const TMT_FONT: INT = 210;
pub const TMT_TEXTCOLOR: INT = 3803;

extern "system" {
    pub fn OpenThemeData(hwnd: HWND, pszClassList: LPCWSTR) -> HTHEME;
    pub fn CloseThemeData(hTheme: HTHEME) -> winsrv_sys::winsrv::HRESULT;
    pub fn IsAppThemed() -> BOOL;
    pub fn IsThemeActive() -> BOOL;
    pub fn GetThemeFont(
        hTheme: HTHEME,
        hdc: winsrv_sys::winsrv::HDC,
        iPartId: INT,
        iStateId: INT,
        iPropId: INT,
        pFont: *mut winsrv_sys::winsrv::LOGFONTW,
    ) -> winsrv_sys::winsrv::HRESULT;
    pub fn GetThemeColor(
        hTheme: HTHEME,
        iPartId: INT,
        iStateId: INT,
        iPropId: INT,
        pColor: *mut winsrv_sys::winsrv::COLORREF,
    ) -> winsrv_sys::winsrv::HRESULT;
    pub fn DrawThemeParentBackground(
        hwnd: HWND,
        hdc: winsrv_sys::winsrv::HDC,
        prc: *const RECT,
    ) -> winsrv_sys::winsrv::HRESULT;
    pub fn SetWindowTheme(
        hwnd: HWND,
        pszSubAppName: LPCWSTR,
        pszSubIdList: LPCWSTR,
    ) -> winsrv_sys::winsrv::HRESULT;
}

// ---------------------------------------------------------------------------
// shell32 ShellExecuteW (stdcall) — trayclock double-click → timedate.cpl
// ---------------------------------------------------------------------------

extern "system" {
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE;
}

// ---------------------------------------------------------------------------
// comctl32 tooltip surface — the CTooltips WTL wrapper is just SendMessage to
// a "tooltips_class32" control; we expose the layout + message constants and
// let translated classes drive it (thin, per-class use).
// ---------------------------------------------------------------------------

/// commctrl.h `TTTOOLINFOW` (V2 layout; V1_SIZE stops after `lpszText`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TOOLINFOW {
    pub cbSize: UINT,
    pub uFlags: UINT,
    pub hwnd: HWND,
    pub uId: UINT_PTR,
    pub rect: RECT,
    pub hinst: HINSTANCE,
    pub lpszText: LPWSTR,
    pub lParam: LPARAM,
    pub lpReserved: *mut c_void,
}

/// `TTTOOLINFOW_V1_SIZE = CCSIZEOF_STRUCT(TTTOOLINFOW, lpszText)` —
/// offset(lpszText)+sizeof(lpszText) == offset of the following field.
#[inline]
pub fn TTTOOLINFOW_V1_SIZE() -> UINT {
    core::mem::offset_of!(TOOLINFOW, lParam) as UINT
}

// tooltip window styles / flags / messages (commctrl.h)
pub const TTS_ALWAYSTIP: DWORD = 0x1;
pub const TTS_NOPREFIX: DWORD = 0x2;
pub const TTF_IDISHWND: UINT = 0x1;
pub const TTF_SUBCLASS: UINT = 0x10;
pub const WM_USER: UINT = 0x0400;
pub const TTM_ADDTOOLW: UINT = WM_USER + 50;
pub const TTM_UPDATETIPTEXTW: UINT = WM_USER + 57;

/// tooltip control class name — CreateWindowEx target for CTooltips::Create.
pub static TOOLTIPS_CLASSW: [u16; 16] = [
    b't' as u16, b'o' as u16, b'o' as u16, b'l' as u16, b't' as u16, b'i' as u16, b'p' as u16,
    b's' as u16, b'_' as u16, b'c' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16,
    b'3' as u16, b'2' as u16,
];

// ---------------------------------------------------------------------------
// winnls locale + misc constants the tier-C classes need
// ---------------------------------------------------------------------------

/// `LOCALE_USER_DEFAULT` (winnt.h) — MAKELCID(LANG_USER_DEFAULT, SORT_DEFAULT).
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400;

/// `CW_USEDEFAULT` (winuser.h). ATL's `CWindow::rcDefault` is
/// `{CW_USEDEFAULT, CW_USEDEFAULT, 0, 0}` and `Create` passes
/// left/top/(right-left)/(bottom-top); `0 - CW_USEDEFAULT` wraps back to
/// CW_USEDEFAULT in 32-bit int, so all four args are CW_USEDEFAULT (USER32
/// resolves it for the child). Pass it verbatim for the four geometry args.
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT;
