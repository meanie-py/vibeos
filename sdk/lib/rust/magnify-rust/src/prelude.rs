//! magnify-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "system" {
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateDialogParamA( arg1: HINSTANCE, arg2: LPCSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamA( arg1: HINSTANCE, arg2: LPCSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DrawIcon( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCursorInfo(arg1: PCURSORINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetGUIThreadInfo(arg1: DWORD, arg2: LPGUITHREADINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetIconInfo(arg1: HICON, arg2: PICONINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IntersectRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRgn(arg1: HWND, arg2: HRGN, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn IsIconic(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromRect(arg1: LPCRECT, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SHAppBarMessage(dwMessage: DWORD, pData: PAPPBARDATA) -> UINT_PTR; // desk-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StretchBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HDC, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: ::core::ffi::c_int, arg11: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::magnify::settings::AppBarConfig;
pub use crate::magnify::settings::bFollowCaret;
pub use crate::magnify::settings::bFollowFocus;
pub use crate::magnify::settings::bFollowMouse;
pub use crate::magnify::settings::bInvertColors;
pub use crate::magnify::settings::bShowMagnifier;
pub use crate::magnify::settings::bShowWarning;
pub use crate::magnify::settings::bStartMinimized;
pub use crate::magnify::settings::uiZoom;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn atoi(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // magnifier.c
}
extern "system" {
    pub fn CreateWindowExA(
        dwExStyle: DWORD,
        lpClassName: LPCSTR,
        lpWindowName: LPCSTR,
        dwStyle: DWORD,
        X: ::core::ffi::c_int,
        Y: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND; // magnifier.c
    pub fn DefWindowProcA(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // magnifier.c
    pub fn DispatchMessageA(_: *const MSG) -> LRESULT; // magnifier.c
    pub fn GetMessageA(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // magnifier.c
    pub fn GetMonitorInfoA(_: HMONITOR, _: LPMONITORINFO) -> BOOL; // magnifier.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongA")]
    pub fn GetWindowLongPtrA(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // magnifier.c
    pub fn LoadAcceleratorsA(_: HINSTANCE, _: LPCSTR) -> HACCEL; // magnifier.c
    pub fn LoadCursorA(_: HINSTANCE, _: LPCSTR) -> HCURSOR; // magnifier.c
    pub fn LoadIconA(hInstance: HINSTANCE, lpIconName: LPCSTR) -> HICON; // magnifier.c
    pub fn LoadImageA(
        hInst_0: HINSTANCE,
        name: LPCSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // magnifier.c
    pub fn LoadMenuA(_: HINSTANCE, _: LPCSTR) -> HMENU; // magnifier.c
    pub fn LoadStringA(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // magnifier.c
    pub fn MessageBoxA(
        hWnd: HWND,
        lpText: LPCSTR,
        lpCaption: LPCSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // magnifier.c
    pub fn PostMessageA(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // magnifier.c
    pub fn RegCreateKeyExA(
        hKey: HKEY,
        lpSubKey: LPCSTR,
        Reserved: DWORD,
        lpClass: LPSTR,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        phkResult: PHKEY,
        lpdwDisposition: PDWORD,
    ) -> LSTATUS; // settings.c
    pub fn RegOpenKeyExA(_: HKEY, _: LPCSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // settings.c
    pub fn RegQueryValueExA(
        hKey: HKEY,
        lpValueName: LPCSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // settings.c
    pub fn RegSetValueExA(
        hKey: HKEY,
        lpValueName: LPCSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS; // settings.c
    pub fn RegisterClassA(_: *const WNDCLASSA) -> ATOM; // magnifier.c
    pub fn SendDlgItemMessageA(
        _: HWND,
        _: ::core::ffi::c_int,
        _: UINT,
        _: WPARAM,
        _: LPARAM,
    ) -> LRESULT; // magnifier.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongA")]
    pub fn SetWindowLongPtrA(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // magnifier.c
    pub fn Shell_NotifyIconA(_: DWORD, _: PNOTIFYICONDATAA) -> BOOL; // magnifier.c
    pub fn SystemParametersInfoA(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL; // magnifier.c
    pub fn TranslateAcceleratorA(_: HWND, _: HACCEL, _: LPMSG) -> ::core::ffi::c_int; // magnifier.c
}
pub use crate::prelude::CreateWindowExA as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcA as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageA as DispatchMessage; // sdk alias
pub use crate::prelude::GetMessageA as GetMessage; // sdk alias
pub use crate::prelude::GetMonitorInfoA as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetWindowLongPtrA as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadAcceleratorsA as LoadAccelerators; // sdk alias
pub use crate::prelude::LoadCursorA as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconA as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageA as LoadImage; // sdk alias
pub use crate::prelude::LoadMenuA as LoadMenu; // sdk alias
pub use crate::prelude::LoadStringA as LoadString; // sdk alias
pub use crate::prelude::MessageBoxA as MessageBox; // sdk alias
pub use crate::prelude::PostMessageA as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExA as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyExA as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExA as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExA as RegSetValueEx; // sdk alias
pub use crate::prelude::RegisterClassA as RegisterClass; // sdk alias
pub use crate::prelude::SendDlgItemMessageA as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrA as SetWindowLongPtr; // sdk alias
pub use crate::prelude::Shell_NotifyIconA as Shell_NotifyIcon; // sdk alias
pub use crate::prelude::SystemParametersInfoA as SystemParametersInfo; // sdk alias
pub use crate::prelude::TranslateAcceleratorA as TranslateAccelerator; // sdk alias
pub use crate::prelude::atoi as _ttoi; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageA((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetLBText { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageA((($p0) as _), ((328) as _), (((($p1) as c_int)) as WPARAM), (((($p2) as LPCTSTR)) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! CreateDialog { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamA((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! DialogBox { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamA((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPSTR) }; }
#[macro_export] macro_rules! TEXT { ($p0:expr $(,)?) => { $p0 }; }
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HWND_BOTTOM: HWND = 0x1usize as HWND;
pub const HWND_TOP: HWND = 0x0usize as HWND;
pub const HWND_TOPMOST: HWND = (((-(1 as c_int))) as HWND);
pub const IDC_ARROW: LPSTR = 0x7f00usize as LPSTR;
pub use crate::{ComboBox_GetCurSel, ComboBox_GetLBText, CreateDialog, DialogBox, MAKEINTRESOURCE, TEXT};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/magnify/magnifier.c:506: #define PointsAreEqual(pt1, pt2) (((pt1).x == (pt2).x) && ((pt1).y == (pt2).y))
#[allow(unused_macros)]
macro_rules! PointsAreEqual {
    ($pt1:expr, $pt2:expr $(,)?) => {
        ((($pt1).x == ($pt2).x) && (($pt1).y == ($pt2).y))
    };
}
#[allow(unused_imports)]
pub(crate) use PointsAreEqual;
// ==== end port macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
