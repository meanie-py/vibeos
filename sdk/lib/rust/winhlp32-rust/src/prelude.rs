//! winhlp32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BOOL_FUNCTION: token_types = token_types_BOOL_FUNCTION; // island enum
pub const EMPTY: token_types = token_types_EMPTY; // island enum
pub const IDENTIFIER: token_types = token_types_IDENTIFIER; // island enum
pub const INTEGER: token_types = token_types_INTEGER; // island enum
pub const VOID_FUNCTION: token_types = token_types_VOID_FUNCTION; // island enum
pub const hlp_link_link: tagHlpFileLink__bindgen_ty_1 = tagHlpFileLink_hlp_link_link; // island enum
pub const hlp_link_macro: tagHlpFileLink__bindgen_ty_1 = tagHlpFileLink_hlp_link_macro; // island enum
pub const hlp_link_popup: tagHlpFileLink__bindgen_ty_1 = tagHlpFileLink_hlp_link_popup; // island enum
extern "C" {
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn sprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int; // crt-shared-rust/src/prelude.rs
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char; // crt-shared-rust/src/prelude.rs
    pub fn strchr(s: *const i8, c: i32) -> *mut i8; // acgenral-rust/src/prelude.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char; // crt-shared-rust/src/prelude.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn strrchr(s: *const u8, c: c_int) -> *mut u8; // ucrt-rust/src/cvt/format.rs
    pub fn wsprintfA(arg1: LPSTR, arg2: LPCSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
extern "system" {
    pub fn AdjustWindowRect(arg1: LPRECT, arg2: DWORD, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CallWindowProcA( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseEnhMetaFile(hdc: HDC) -> HENHMETAFILE; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateDIBitmap( hdc: HDC, pbmih: *const BITMAPINFOHEADER, fdwInit: DWORD, pvInit: *const ::core::ffi::c_void, pbmi: *const BITMAPINFO, uUsage: UINT, ) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateEnhMetaFileW(arg1: HDC, arg2: LPCWSTR, arg3: LPCRECT, arg4: LPCWSTR) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_i386.rs
    pub fn CreateIconFromResourceEx( presbits: PBYTE, dwResSize: DWORD, fIcon: BOOL, dwVer: DWORD, cxDesired: ::core::ffi::c_int, cyDesired: ::core::ffi::c_int, Flags: UINT, ) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn CreatePropertySheetPageA(arg1: LPCPROPSHEETPAGEA) -> HPROPSHEETPAGE; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExA( dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteEnhMetaFile(arg1: HENHMETAFILE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentDirectoryA(arg1: DWORD, arg2: LPSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetEnhMetaFileBits(hEMF: HENHMETAFILE, nSize: UINT, lpData: LPBYTE) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMessagePos() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetOpenFileNameA(arg1: LPOPENFILENAMEA) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetTextExtentPointA( hdc: HDC, lpString: LPCSTR, c: ::core::ffi::c_int, lpsz: LPSIZE, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetTextMetricsW(arg1: HDC, arg2: LPTEXTMETRICW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongA")]
    pub fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR; // shell32-rust/src/wine/shellole.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn LoadStringA( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxA( hWnd: HWND, lpText: LPCSTR, lpCaption: LPCSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn OpenFile(arg1: LPCSTR, arg2: LPOFSTRUCT, arg3: UINT) -> HFILE; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn PrintDlgW(arg1: LPPRINTDLGW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn PropertySheetA(arg1: LPCPROPSHEETHEADERA) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn RedrawWindow(arg1: HWND, arg2: LPCRECT, arg3: HRGN, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegisterClassExA(arg1: *const WNDCLASSEXA) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SearchPathA( lpPath: LPCSTR, lpFileName: LPCSTR, lpExtension: LPCSTR, nBufferLength: DWORD, lpBuffer: LPSTR, lpFilePart: *mut LPSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetActiveWindow(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetBkColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongA")]
    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> LONG_PTR; // shell32-rust/src/wine/shellole.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextA(arg1: HWND, arg2: LPCSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShellAboutW( hWnd: HWND, szApp: LPCWSTR, szOtherStuff: LPCWSTR, hIcon: HICON, ) -> ::core::ffi::c_int; // ctfmon-rust/src/types_i386.rs
    pub fn ShellExecuteA( hwnd: HWND, lpOperation: LPCSTR, lpFile: LPCSTR, lpParameters: LPCSTR, lpDirectory: LPCSTR, nShowCmd: INT, ) -> HINSTANCE; // ctfmon-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TextOutA( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, lpString: LPCSTR, c: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn _hread( hFile: HFILE, lpBuffer: LPVOID, lBytes: ::core::ffi::c_long, ) -> ::core::ffi::c_long; // aclui-rust/src/types_i386.rs
    pub fn _lclose(arg1: HFILE) -> HFILE; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::winhlp32::string::BUTTON_BOX_WIN_CLASS_NAME;
pub use crate::winhlp32::callback::Callbacks;
pub use crate::winhlp32::winhelp::Globals;
pub use crate::winhlp32::string::HISTORY_WIN_CLASS_NAME;
pub use crate::winhlp32::hlpfile::HLPFILE_BPTreeEnum;
pub use crate::winhlp32::hlpfile::HLPFILE_BrowsePage;
pub use crate::winhlp32::hlpfile::HLPFILE_FreeHlpFile;
pub use crate::winhlp32::hlpfile::HLPFILE_Hash;
pub use crate::winhlp32::hlpfile::HLPFILE_PageByHash;
pub use crate::winhlp32::hlpfile::HLPFILE_PageByMap;
pub use crate::winhlp32::hlpfile::HLPFILE_PageByOffset;
pub use crate::winhlp32::hlpfile::HLPFILE_ReadHlpFile;
pub use crate::winhlp32::string::MAIN_WIN_CLASS_NAME;
pub use crate::winhlp32::string::STRING_BUTTON;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindowA { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExA(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const HFILE_ERROR: HFILE = (((-(1 as c_int))) as HFILE);
pub const HWND_NOTOPMOST: HWND = (((-(2 as c_int))) as HWND);
pub const HWND_TOP: HWND = 0x0usize as HWND;
pub const HWND_TOPMOST: HWND = (((-(1 as c_int))) as HWND);
pub const IDC_ARROW: ULONG_PTR = ((((32512) as WORD)) as ULONG_PTR);
pub const IDC_HAND: ULONG_PTR = ((((32649) as WORD)) as ULONG_PTR);
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub use crate::{CreateWindowA, MAKELPARAM, max, min};
// ==== end SDK header macros ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_winhelp: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x69\x6e\x68\x65\x6c\x70\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_winhelp };
// ==== end wine debug channels ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn MACRO_CurrentWindow() -> *mut tagWinHelp; // macro.c
    pub fn MACRO_ExecuteMacro(_: *mut tagWinHelp, _: LPCSTR) -> BOOL; // macro.c
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // winhelp.c
    pub fn _stricmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // macro.c
}
pub use crate::prelude::_snprintf as snprintf; // sdk alias
pub use crate::prelude::_stricmp as strcasecmp; // sdk alias
// ==== end port externs ====
