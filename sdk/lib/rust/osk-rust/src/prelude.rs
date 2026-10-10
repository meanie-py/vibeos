//! osk-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BLACK_BRUSH: u32 = 4; // access-rust/src/types_amd64.rs
pub const SETKEYS_INIT: SetKeys_Reason = SetKeys_Reason_SETKEYS_INIT; // island enum
pub const SETKEYS_LANG: SetKeys_Reason = SetKeys_Reason_SETKEYS_LANG; // island enum
pub const SETKEYS_LAYOUT: SetKeys_Reason = SetKeys_Reason_SETKEYS_LAYOUT; // island enum
pub const SND_ASYNC: u32 = 1; // comctl32-rust/src/types_amd64.rs
pub const SND_RESOURCE: u32 = 262148; // comctl32-rust/src/types_amd64.rs
extern "C" {
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
}
extern "system" {
    pub fn ActivateKeyboardLayout(arg1: HKL, arg2: UINT) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn ChooseFontW(arg1: LPCHOOSEFONTW) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CopyImage( arg1: HANDLE, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardState(lpKeyState: PBYTE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetMessageExtraInfo() -> LPARAM; // aclui-rust/src/types_amd64.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IntersectRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MapVirtualKeyW(arg1: UINT, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromPoint(arg1: POINT, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL; // shell32-rust/src/folders/CRecycleBinEnum.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendInput(cInputs: UINT, pInputs: LPINPUT, cbSize: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShellAboutW( hWnd: HWND, szApp: LPCWSTR, szOtherStuff: LPCWSTR, hIcon: HICON, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ToUnicode( wVirtKey: UINT, wScanCode: UINT, lpKeyState: *const BYTE, pwszBuff: LPWSTR, cchBuff: ::core::ffi::c_int, wFlags: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::osk::main::Globals;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// LOGFONTW: c2rust, main.c
pub type LOGFONTW = tagLOGFONTW;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // main.c
}
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
// ==== end port externs ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindowW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HWND_NOTOPMOST: HWND = (((-(2 as c_int))) as HWND);
pub const HWND_TOPMOST: HWND = (((-(1 as c_int))) as HWND);
pub const WC_BUTTONW: *const u16 = (&[0x0042, 0x0075, 0x0074, 0x0074, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"Button"
pub const WC_STATICW: *const u16 = (&[0x0053, 0x0074, 0x0061, 0x0074, 0x0069, 0x0063, 0x0000u16]).as_ptr(); // L"Static"
pub use crate::{CreateWindowW, DialogBoxW};
// ==== end SDK header macros ====
