//! mspaint-rust prelude — the single import surface the translated modules use.
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

// ================= ATL ========================================================================
// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.
pub use atl_rust::{atl_format, atl_format_id, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION};
pub use atl_rust::CPath;
pub use atl_rust::CStringW as CSimpleString;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BrushStyleBackSlash: BrushStyle = BrushStyle_BrushStyleBackSlash; // island enum
pub const BrushStyleForeSlash: BrushStyle = BrushStyle_BrushStyleForeSlash; // island enum
pub const BrushStyleRound: BrushStyle = BrushStyle_BrushStyleRound; // island enum
pub const BrushStyleSquare: BrushStyle = BrushStyle_BrushStyleSquare; // island enum
pub const CHAR_BIT: u32 = 8; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT; // comctl32-rust/src/prelude.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const HIT_BORDER: HITTEST = HITTEST_HIT_BORDER; // island enum
pub const HIT_INNER: HITTEST = HITTEST_HIT_INNER; // island enum
pub const HIT_LOWER_CENTER: HITTEST = HITTEST_HIT_LOWER_CENTER; // island enum
pub const HIT_LOWER_LEFT: HITTEST = HITTEST_HIT_LOWER_LEFT; // island enum
pub const HIT_LOWER_RIGHT: HITTEST = HITTEST_HIT_LOWER_RIGHT; // island enum
pub const HIT_MIDDLE_LEFT: HITTEST = HITTEST_HIT_MIDDLE_LEFT; // island enum
pub const HIT_MIDDLE_RIGHT: HITTEST = HITTEST_HIT_MIDDLE_RIGHT; // island enum
pub const HIT_NONE: HITTEST = HITTEST_HIT_NONE; // island enum
pub const HIT_UPPER_CENTER: HITTEST = HITTEST_HIT_UPPER_CENTER; // island enum
pub const HIT_UPPER_LEFT: HITTEST = HITTEST_HIT_UPPER_LEFT; // island enum
pub const HIT_UPPER_RIGHT: HITTEST = HITTEST_HIT_UPPER_RIGHT; // island enum
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // access-rust/src/prelude.rs
pub const IDC_WAIT: LPWSTR = 0x7f02usize as LPWSTR; // notepad-rust/src/prelude.rs
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // comctl32-rust/src/prelude.rs
pub const NM_CUSTOMDRAW: u32 = 0xFFFFFFF4; // comctl32-rust/src/prelude.rs
pub const PAL_GRAYSCALE: PAL_TYPE = PAL_TYPE_PAL_GRAYSCALE; // island enum
pub const PAL_MODERN: PAL_TYPE = PAL_TYPE_PAL_MODERN; // island enum
pub const PAL_MONOCHROME: PAL_TYPE = PAL_TYPE_PAL_MONOCHROME; // island enum
pub const PAL_OLDTYPE: PAL_TYPE = PAL_TYPE_PAL_OLDTYPE; // island enum
pub const SB_SETTEXT: i32 = 0x40B; // notepad-rust/src/prelude.rs
pub const SPI_GETWHEELSCROLLCHARS: u32 = 108; // shell32-rust/src/island_amd64.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const TOOL_AIRBRUSH: TOOLTYPE = TOOLTYPE_TOOL_AIRBRUSH; // island enum
pub const TOOL_BEZIER: TOOLTYPE = TOOLTYPE_TOOL_BEZIER; // island enum
pub const TOOL_BRUSH: TOOLTYPE = TOOLTYPE_TOOL_BRUSH; // island enum
pub const TOOL_COLOR: TOOLTYPE = TOOLTYPE_TOOL_COLOR; // island enum
pub const TOOL_ELLIPSE: TOOLTYPE = TOOLTYPE_TOOL_ELLIPSE; // island enum
pub const TOOL_FILL: TOOLTYPE = TOOLTYPE_TOOL_FILL; // island enum
pub const TOOL_FREESEL: TOOLTYPE = TOOLTYPE_TOOL_FREESEL; // island enum
pub const TOOL_LINE: TOOLTYPE = TOOLTYPE_TOOL_LINE; // island enum
pub const TOOL_PEN: TOOLTYPE = TOOLTYPE_TOOL_PEN; // island enum
pub const TOOL_RECT: TOOLTYPE = TOOLTYPE_TOOL_RECT; // island enum
pub const TOOL_RECTSEL: TOOLTYPE = TOOLTYPE_TOOL_RECTSEL; // island enum
pub const TOOL_RRECT: TOOLTYPE = TOOLTYPE_TOOL_RRECT; // island enum
pub const TOOL_RUBBER: TOOLTYPE = TOOLTYPE_TOOL_RUBBER; // island enum
pub const TOOL_SHAPE: TOOLTYPE = TOOLTYPE_TOOL_SHAPE; // island enum
pub const TOOL_TEXT: TOOLTYPE = TOOLTYPE_TOOL_TEXT; // island enum
pub const TOOL_ZOOM: TOOLTYPE = TOOLTYPE_TOOL_ZOOM; // island enum
pub const ULONG_MAX: u32 = 4294967295; // msv1_0-rust/src/types_amd64.rs
extern "C" {
    pub fn _wtoi(s: *const u16) -> i32; // userinit-rust/src/preamble.rs
    pub fn abs(x: i32) -> i32; // cmlib-rust/src/cmcheck.rs
    pub fn atan2(y: f64, x: f64) -> f64; // atan2-rust/src/lib.rs
    pub fn labs(n: c_long) -> c_long; // desk-rust/src/prelude.rs
    pub fn rand() -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn srand(seed: c_uint); // dhcpcsvc-rust/src/prelude.rs
    pub fn tan(x: f64) -> f64; // msvcrt-crt-rust/src/msvcrt/math.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstod(str_: *const u16, end: *mut *mut u16) -> f64; // msvcrt-crt-rust/src/fpnum.rs
    pub static CLSID_NULL: GUID; // shell32-rust/src/cfs_folder.rs
    pub static GUID_NULL: GUID; // audiosrv-rust/src/types_amd64.rs
    pub static mut __argc: ::core::ffi::c_int; // ntkrnl-sys/src/hal_bindgen_amd64.rs
}
extern "system" {
    pub fn BeginDeferWindowPos(arg1: ::core::ffi::c_int) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CheckRadioButton( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CopyIcon(arg1: HICON) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn CopyImage( arg1: HANDLE, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DeferWindowPos( arg1: HDWP, arg2: HWND, arg3: HWND, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, ) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyAcceleratorTable(arg1: HACCEL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyCursor(arg1: HCURSOR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawEdge(arg1: HDC, arg2: LPRECT, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawFocusRect(arg1: HDC, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawIconEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, arg8: HBRUSH, arg9: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn EmptyClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDeferWindowPos(arg1: HDWP) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EqualRect(arg1: LPCRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetClipboardData(arg1: UINT) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDCEx(arg1: HWND, arg2: HRGN, arg3: DWORD) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: PBOOL, arg4: BOOL) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemCount(arg1: HMENU) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetScrollInfo(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPSCROLLINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetScrollPos(arg1: HWND, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSysColorBrush(arg1: ::core::ffi::c_int) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetTempFileNameW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: UINT, arg4: LPWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86_64", link_name = "GetWindowLongPtrW")]
    #[cfg_attr(target_arch = "aarch64", link_name = "GetWindowLongPtrW")]
    pub fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR; // shell32-rust/src/dialogs/drvdefext.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn IntersectRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InvertRect(arg1: HDC, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsClipboardFormatAvailable(arg1: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn IsIconic(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsRectEmpty(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsZoomed(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PathFileExistsW(pszPath: LPCWSTR) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // desk-rust/src/types_i386.rs
    pub fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR; // desk-rust/src/types_i386.rs
    pub fn PathRemoveExtensionW(arg1: LPWSTR); // desk-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn PtInRect(arg1: LPCRECT, arg2: POINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetMenu(arg1: HWND, arg2: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetRectEmpty(arg1: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetScrollInfo( arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCSCROLLINFO, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetScrollPos( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86_64", link_name = "SetWindowLongPtrW")]
    #[cfg_attr(target_arch = "aarch64", link_name = "SetWindowLongPtrW")]
    pub fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR; // shell32-rust/src/dialogs/drvdefext.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowScrollBar(arg1: HWND, arg2: ::core::ffi::c_int, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackMouseEvent(arg1: LPTRACKMOUSEEVENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnionRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mspaint::dib_cpp_h::BitmapFromClipboardDIB;
pub use crate::mspaint::dib_cpp_h::BitmapFromHEMF;
pub use crate::mspaint::dib_cpp_h::BitmapToClipboardDIB;
pub use crate::mspaint::drawing_cpp_h::Brush__0;
pub use crate::mspaint::dib_cpp_h::CachedBufferDIB;
pub use crate::mspaint::drawing_cpp_h::ColorKeyedMaskBlt;
pub use crate::mspaint::dib_cpp_h::ConvertToBlackAndWhite;
pub use crate::mspaint::dib_cpp_h::CopyDIBImage;
pub use crate::mspaint::dib_cpp_h::CopyMonoImage;
pub use crate::mspaint::dib_cpp_h::CreateColorDIB;
pub use crate::mspaint::dib_cpp_h::CreateDIBWithProperties;
pub use crate::mspaint::drawing_cpp_h::CreateDitherBrush;
pub use crate::mspaint::dib_cpp_h::CreateMonoBitmap;
pub use crate::mspaint::dib_cpp_h::DoLoadImageFile;
pub use crate::mspaint::drawing_cpp_h::DrawXorRect;
pub use crate::mspaint::drawing_cpp_h::Erase__0;
pub use crate::mspaint::dib_cpp_h::GetDIBHeight;
pub use crate::mspaint::dib_cpp_h::GetDIBWidth;
pub use crate::mspaint::dib_cpp_h::InitializeImage;
pub use crate::mspaint::dib_cpp_h::IsBitmapBlackAndWhite;
pub use crate::mspaint::dib_cpp_h::PpcmFromDpi;
pub use crate::mspaint::drawing_cpp_h::RectSel;
pub use crate::mspaint::drawing_cpp_h::Rect__0;
pub use crate::mspaint::dib_cpp_h::Rotate90DegreeBlt;
pub use crate::mspaint::dib_cpp_h::SaveDIBToFile;
pub use crate::mspaint::main_cpp_h::ShowOutOfMemory;
pub use crate::mspaint::dib_cpp_h::SkewDIB;
pub use crate::mspaint::toolsmodel_h_h::UnZoomed__0;
pub use crate::mspaint::toolsmodel_h_h::UnZoomed__1;
pub use crate::mspaint::toolsmodel_h_h::UnZoomed__2;
pub use crate::mspaint::toolsmodel_h_h::Zoomed__0;
pub use crate::mspaint::toolsmodel_h_h::Zoomed__1;
pub use crate::mspaint::toolsmodel_h_h::Zoomed__2;
pub use crate::mspaint::sizebox_cpp_h::drawSizeBoxes;
pub use crate::mspaint::sizebox_cpp_h::getSizeBoxHitTest;
pub use crate::mspaint::dib_cpp_h::getSubImage;
pub use crate::mspaint::dib_cpp_h::putSubImage;
pub use crate::mspaint::sizebox_cpp_h::setCursorOnSizeBox;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CDN_TYPECHANGE: u32 = 0xFFFFFDA1; // clang, main.cpp
pub const COLOR_COUNT: i32 = 0x1C; // clang, palette.cpp
pub const CXY_SELECTEDBOX: i32 = 0xF; // clang, palette.cpp
pub const CX_TOOLSETTINGS: i32 = 0x32; // clang, toolsettings.cpp
pub const CX_TRANS_ICON: i32 = 0x28; // clang, toolsettings.cpp
pub const CY_TOOLSETTINGS: i32 = 0x8C; // clang, toolsettings.cpp
pub const CY_TRANS_ICON: i32 = 0x1E; // clang, toolsettings.cpp
pub const DEFAULT_ZOOM_TRACK: i32 = 3; // clang, toolsettings.cpp
pub const HALF_COLOR_COUNT: i32 = 0xE; // clang, palette.cpp
pub const MARGIN1: i32 = 3; // clang, toolsettings.cpp
pub const MARGIN2: i32 = 2; // clang, toolsettings.cpp
pub const MAX_RECENT_PATHNAME_DISPLAY: i32 = 0x1E; // clang, main.cpp
pub const MAX_ZOOM_TRACK: i32 = 6; // clang, toolsettings.cpp
pub const MIN_ZOOM_TRACK: i32 = 0; // clang, toolsettings.cpp
pub const M_PI: f64 = 3.14159265358979323846; // clang, dib.cpp
pub const TB_ADDBUTTONS: i32 = 0x444; // clang, dialogs.cpp
pub const THRESHOULD_DEG: i32 = 0xF; // clang, mouse.cpp
pub const TTN_NEEDTEXT: u32 = 0xFFFFFDEE; // clang, dialogs.cpp
pub const X_COLORBOX_OFFSET: i32 = 0x28; // clang, palette.cpp
pub const X_MARGIN: i32 = 4; // clang, palette.cpp
pub const X_TOOLSETTINGS: i32 = 0; // clang, toolsettings.cpp
pub const Y_TOOLSETTINGS: i32 = 0xCB; // clang, toolsettings.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn CharLowerW(arg1: LPWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn ChooseColorW(arg1: LPCHOOSECOLORW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DragQueryFileW(hDrop: HDROP, iFile: UINT, lpszFile: LPWSTR, cch: UINT) -> UINT; // desk-rust/src/types_i386.rs
    pub fn FindWindowExW(arg1: HWND, arg2: HWND, arg3: LPCWSTR, arg4: LPCWSTR) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetTextMetricsW(arg1: HDC, arg2: LPTEXTMETRICW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ImageList_LoadImageW( hi: HINSTANCE, lpbmp: LPCWSTR, cx: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, crMask: COLORREF, uType: UINT, uFlags: UINT, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn InsertMenuW(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT_PTR, arg5: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDialogMessageW(arg1: HWND, arg2: LPMSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn PageSetupDlgW(arg1: LPPAGESETUPDLGW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn PrintDlgW(arg1: LPPRINTDLGW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::CallWindowProcW as CallWindowProc; // sdk alias
pub use crate::prelude::CharLowerW as CharLower; // sdk alias
pub use crate::prelude::ChooseColorW as ChooseColor; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DragQueryFileW as DragQueryFile; // sdk alias
pub use crate::prelude::FindWindowExW as FindWindowEx; // sdk alias
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetTextMetricsW as GetTextMetrics; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::ImageList_LoadImageW as ImageList_LoadImage; // sdk alias
pub use crate::prelude::InsertMenuW as InsertMenu; // sdk alias
pub use crate::prelude::IsDialogMessageW as IsDialogMessage; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PageSetupDlgW as PageSetupDlg; // sdk alias
pub use crate::prelude::PathFindFileNameW as PathFindFileName; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::PrintDlgW as PrintDlg; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
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

// HAND-WRITTEN: msvcrt's `__argc` / `__wargv` (`__targv` in a UNICODE build) are DATA
// exports (`@ extern __argc`), so there is no `__argc` symbol to link -- only its import
// slot, which holds the variable's address. i386 decorates the slot `__imp____argc`
// (rustc prepends one `_` to a link_name there); amd64/arm64 spell it `__imp___argc`.
extern "C" {
    #[cfg_attr(target_arch = "x86", link_name = "_imp____argc")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp___argc")]
    pub static __imp___argc: *mut ::core::ffi::c_int;
    #[cfg_attr(target_arch = "x86", link_name = "_imp____wargv")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp___wargv")]
    pub static __imp___wargv: *mut *mut *mut u16;
}

// HAND-WRITTEN: dialogs.cpp's `void ShowError(INT stringID, ...)` is DEFINED by the C
// trampoline base/applications/mspaint/rust_varargs_shim.c (stable Rust cannot define a
// C-variadic function); its body is dialogs_cpp_h::ShowError_v. Callers use this.
extern "C" {
    pub fn ShowError(stringID: INT, ...);
}

// HAND-WRITTEN (outside the generated blocks, so `port macros` leaves it alone):
// precomp.h:78 `#define DEG2RAD(degree) (((degree) * M_PI) / 180)`. In C `degree` (an int at
// every call site) is promoted to double by `* M_PI`; the generated lowering multiplied i32
// by f64.
#[allow(unused_macros)]
macro_rules! DEG2RAD {
    ($degree:expr $(,)?) => {
        ((($degree) as f64 * M_PI) / 180.0)
    };
}
#[allow(unused_imports)]
pub(crate) use DEG2RAD;

// HAND-WRITTEN: precomp.h:89 `template <typename T> inline void Swap(T& a, T& b)
// { T tmp = a; a = b; b = tmp; }` -- cppapp does not generate templates. Both call sites swap
// two places of one type (HBITMAPs in history.cpp, CStringW in registry.cpp), and the copy
// dance leaves exactly the two values exchanged, which is `mem::swap`.
#[allow(dead_code)]
pub unsafe fn Swap<T>(a: &mut T, b: &mut T) {
    core::mem::swap(a, b);
}

// HAND-WRITTEN: gdiplusimaging.h's `DEFINE_GUID(ImageFormatXXX, ..)` under the module's
// -DINITGUID DEFINES each GUID in every TU that includes it, so nothing exports them to link
// against; the island's `extern { static Gdiplus_ImageFormatXXX }` would never resolve. The
// values are the header's (gdiplusimaging.h:24-32). These shadow the island's declarations.
pub static Gdiplus_ImageFormatBMP: GUID = GUID { Data1: 0xb96b3cab, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatEMF: GUID = GUID { Data1: 0xb96b3cac, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatWMF: GUID = GUID { Data1: 0xb96b3cad, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatJPEG: GUID = GUID { Data1: 0xb96b3cae, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatPNG: GUID = GUID { Data1: 0xb96b3caf, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatGIF: GUID = GUID { Data1: 0xb96b3cb0, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatTIFF: GUID = GUID { Data1: 0xb96b3cb1, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
pub static Gdiplus_ImageFormatIcon: GUID = GUID { Data1: 0xb96b3cb5, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ComboBox_AddString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((323) as _), ((0) as _), (($p1) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_FindStringExact { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((344) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetLBText { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((328) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetLBTextLen { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((329) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_ResetContent { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((331) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_SetCurSel { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((334) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! CopyCursor { ($p0:expr $(,)?) => { ((CopyIcon((($p0) as HICON))) as HCURSOR) }; }
#[macro_export] macro_rules! Edit_GetModify { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((184) as _), ((0) as _), ((0) as _))) as DWORD)) as BOOL) }; }
#[macro_export] macro_rules! Edit_SetModify { ($p0:expr, $p1:expr $(,)?) => { { let _ = SendMessageW((($p0) as _), ((185) as _), (((($p1) as UINT)) as WPARAM), ((0) as _)); } }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEROP4 { ($p0:expr, $p1:expr $(,)?) => { ((((($p1 << 8) & 0xFF000000) | $p0)) as DWORD) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! SetWindowFont { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { { let _ = SendMessageW((($p0) as _), ((48) as _), (($p1) as WPARAM), (((($p2) as BOOL)) as LPARAM)); } }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const HWND_DESKTOP: HWND = 0x0usize as HWND;
pub const IDC_CROSS: LPWSTR = 0x7f03usize as LPWSTR;
pub const IDC_SIZEALL: LPWSTR = 0x7f86usize as LPWSTR;
pub type LPTOOLTIPTEXTW = LPNMTTDISPINFOW;
pub const STATUSCLASSNAME: *const u16 = (&[0x006d, 0x0073, 0x0063, 0x0074, 0x006c, 0x0073, 0x005f, 0x0073, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0062, 0x0061, 0x0072, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"msctls_statusbar32"
pub const TOOLBARCLASSNAME: *const u16 = (&[0x0054, 0x006f, 0x006f, 0x006c, 0x0062, 0x0061, 0x0072, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"ToolbarWindow32"
pub const TRACKBAR_CLASS: *const u16 = (&[0x006d, 0x0073, 0x0063, 0x0074, 0x006c, 0x0073, 0x005f, 0x0074, 0x0072, 0x0061, 0x0063, 0x006b, 0x0062, 0x0061, 0x0072, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"msctls_trackbar32"
pub const WC_EDIT: *const u16 = (&[0x0045, 0x0064, 0x0069, 0x0074, 0x0000u16]).as_ptr(); // L"Edit"
pub use crate::{ComboBox_AddString, ComboBox_FindStringExact, ComboBox_GetCurSel, ComboBox_GetLBText, ComboBox_GetLBTextLen, ComboBox_ResetContent, ComboBox_SetCurSel, CopyCursor, Edit_GetModify, Edit_SetModify, MAKELPARAM, MAKEROP4, MAKEWPARAM, SetWindowFont, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/mspaint/precomp.h:82: #define CHECKED_IF(bChecked) ((bChecked) ? (MF_CHECKED | MF_BYCOMMAND) : (MF_UNCHECKED | MF_BYCOMMAND))
#[allow(unused_macros)]
macro_rules! CHECKED_IF {
    ($bChecked:expr $(,)?) => {
        (if ($bChecked) {
            MF_CHECKED | MF_BYCOMMAND
        } else {
            MF_UNCHECKED | MF_BYCOMMAND
        })
    };
}
#[allow(unused_imports)]
pub(crate) use CHECKED_IF;
// base/applications/mspaint/precomp.h:86: #define ENABLED_IF(bEnabled) ((bEnabled) ? (MF_ENABLED | MF_BYCOMMAND) : (MF_GRAYED | MF_BYCOMMAND))
#[allow(unused_macros)]
macro_rules! ENABLED_IF {
    ($bEnabled:expr $(,)?) => {
        (if ($bEnabled) {
            MF_ENABLED | MF_BYCOMMAND
        } else {
            MF_GRAYED | MF_BYCOMMAND
        })
    };
}
#[allow(unused_imports)]
pub(crate) use ENABLED_IF;
// base/applications/mspaint/precomp.h:79: #define RAD2DEG(radian) ((LONG)(((radian) * 180) / M_PI))
#[allow(unused_macros)]
macro_rules! RAD2DEG {
    ($radian:expr $(,)?) => {
        ((((($radian) * 180.0) / M_PI)) as LONG)
    };
}
#[allow(unused_imports)]
pub(crate) use RAD2DEG;
// base/applications/mspaint/dib.h:38: #define ROUND(x) (INT)((x) + 0.5)
#[allow(unused_macros)]
macro_rules! ROUND {
    ($x:expr $(,)?) => {
        ((($x) + 0.5) as INT)
    };
}
#[allow(unused_imports)]
pub(crate) use ROUND;
// base/applications/mspaint/dib.cpp:15: #define WIDTHBYTES(i) (((i) + 31) / 32 * 4)
#[allow(unused_macros)]
macro_rules! WIDTHBYTES {
    ($i:expr $(,)?) => {
        ((($i) + 31) / 32 * 4)
    };
}
#[allow(unused_imports)]
pub(crate) use WIDTHBYTES;
// ==== end port macros ====

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::mspaint::cpp_classes::*;
pub use crate::mspaint::cpp_classes::{AirBrushTool, AirBrushTool__VTBL_b0b0, AirBrushTool__delete, AirBrushTool__make_0, AirBrushTool__new_0, AirBrushTool__new_in_0, AirBrushTool__zeroed, BITMAPINFODX, BITMAPINFODX__delete, BITMAPINFODX__make_0, BITMAPINFODX__new_0, BITMAPINFODX__new_in_0, BITMAPINFODX__zeroed, BITMAPINFOEX, BITMAPINFOEX__delete, BITMAPINFOEX__make_0, BITMAPINFOEX__new_0, BITMAPINFOEX__new_in_0, BITMAPINFOEX__zeroed, BezierTool, BezierTool__VTBL_b0, BezierTool__delete, BezierTool__make_0, BezierTool__new_0, BezierTool__new_in_0, BezierTool__zeroed, BrushStyleAndWidth, BrushStyleAndWidth__delete, BrushStyleAndWidth__make_0, BrushStyleAndWidth__new_0, BrushStyleAndWidth__new_in_0, BrushStyleAndWidth__zeroed, BrushTool, BrushTool__VTBL_b0b0, BrushTool__delete, BrushTool__make_0, BrushTool__new_0, BrushTool__new_in_0, BrushTool__zeroed, CAttributesDialog, CAttributesDialog__VTBL_own, CAttributesDialog__delete, CAttributesDialog__make_0, CAttributesDialog__new_0, CAttributesDialog__new_in_0, CAttributesDialog__vtbl, CAttributesDialog__zeroed, CCanvasWindow, CCanvasWindow__CLASS_NAME, CCanvasWindow__VTBL_own, CCanvasWindow__WND_CLASS, CCanvasWindow__delete, CCanvasWindow__make_0, CCanvasWindow__new_0, CCanvasWindow__new_in_0, CCanvasWindow__vtbl, CCanvasWindow__zeroed, CFontsDialog, CFontsDialog__VTBL_own, CFontsDialog__delete, CFontsDialog__make_0, CFontsDialog__new_0, CFontsDialog__new_in_0, CFontsDialog__vtbl, CFontsDialog__zeroed, CFullscreenWindow, CFullscreenWindow__CLASS_NAME, CFullscreenWindow__VTBL_own, CFullscreenWindow__WND_CLASS, CFullscreenWindow__delete, CFullscreenWindow__make_0, CFullscreenWindow__new_0, CFullscreenWindow__new_in_0, CFullscreenWindow__vtbl, CFullscreenWindow__zeroed, CImage, CImageDx, CImageDx__delete, CImageDx__make_0, CImageDx__new_0, CImageDx__new_in_0, CImageDx__zeroed, CImage__VTBL_own, CImage__delete, CImage__make_0, CImage__new_0, CImage__new_in_0, CImage__s_gdiplus, CImage__vtbl, CImage__zeroed, CInitGDIPlus, CInitGDIPlus__delete, CInitGDIPlus__make_0, CInitGDIPlus__new_0, CInitGDIPlus__new_in_0, CInitGDIPlus__zeroed, CMainWindow, CMainWindow__CLASS_NAME, CMainWindow__VTBL_own, CMainWindow__WND_CLASS, CMainWindow__delete, CMainWindow__make_0, CMainWindow__new_0, CMainWindow__new_in_0, CMainWindow__vtbl, CMainWindow__zeroed, CMiniatureWindow, CMiniatureWindow__CLASS_NAME, CMiniatureWindow__VTBL_own, CMiniatureWindow__WND_CLASS, CMiniatureWindow__delete, CMiniatureWindow__make_0, CMiniatureWindow__new_0, CMiniatureWindow__new_in_0, CMiniatureWindow__vtbl, CMiniatureWindow__zeroed, CMirrorRotateDialog, CMirrorRotateDialog__VTBL_own, CMirrorRotateDialog__delete, CMirrorRotateDialog__make_0, CMirrorRotateDialog__new_0, CMirrorRotateDialog__new_in_0, CMirrorRotateDialog__vtbl, CMirrorRotateDialog__zeroed, COMMAND_TO_TOOL, COMMAND_TO_TOOL__delete, COMMAND_TO_TOOL__make_0, COMMAND_TO_TOOL__new_0, COMMAND_TO_TOOL__new_in_0, COMMAND_TO_TOOL__zeroed, CPaintToolBar, CPaintToolBar__delete, CPaintToolBar__make_0, CPaintToolBar__new_0, CPaintToolBar__new_in_0, CPaintToolBar__zeroed, CPaletteWindow, CPaletteWindow__CLASS_NAME, CPaletteWindow__VTBL_own, CPaletteWindow__WND_CLASS, CPaletteWindow__delete, CPaletteWindow__make_0, CPaletteWindow__new_0, CPaletteWindow__new_in_0, CPaletteWindow__vtbl, CPaletteWindow__zeroed, CPoint, CPoint__delete, CPoint__make_0, CPoint__make_1, CPoint__make_2, CPoint__make_3, CPoint__make_4, CPoint__new_0, CPoint__new_1, CPoint__new_2, CPoint__new_3, CPoint__new_4, CPoint__new_in_0, CPoint__new_in_1, CPoint__new_in_2, CPoint__new_in_3, CPoint__new_in_4, CPoint__zeroed, CRect, CRect__delete, CRect__make_0, CRect__make_1, CRect__make_2, CRect__make_3, CRect__make_4, CRect__make_5, CRect__new_0, CRect__new_1, CRect__new_2, CRect__new_3, CRect__new_4, CRect__new_5, CRect__new_in_0, CRect__new_in_1, CRect__new_in_2, CRect__new_in_3, CRect__new_in_4, CRect__new_in_5, CRect__zeroed, CRegKey, CRegKey__delete, CRegKey__make_0, CRegKey__make_1, CRegKey__make_2, CRegKey__new_0, CRegKey__new_1, CRegKey__new_2, CRegKey__new_in_0, CRegKey__new_in_1, CRegKey__new_in_2, CRegKey__zeroed, CSize, CSize__delete, CSize__make_0, CSize__make_1, CSize__make_2, CSize__make_3, CSize__make_4, CSize__new_0, CSize__new_1, CSize__new_2, CSize__new_3, CSize__new_4, CSize__new_in_0, CSize__new_in_1, CSize__new_in_2, CSize__new_in_3, CSize__new_in_4, CSize__zeroed, CStretchSkewDialog, CStretchSkewDialog__VTBL_own, CStretchSkewDialog__delete, CStretchSkewDialog__make_0, CStretchSkewDialog__new_0, CStretchSkewDialog__new_in_0, CStretchSkewDialog__vtbl, CStretchSkewDialog__zeroed, CStyledCursor, CStyledCursor__delete, CStyledCursor__make_0, CStyledCursor__new_0, CStyledCursor__new_in_0, CStyledCursor__zeroed, CTextEditWindow, CTextEditWindow__CLASS_NAME, CTextEditWindow__VTBL_own, CTextEditWindow__WND_CLASS, CTextEditWindow__delete, CTextEditWindow__make_0, CTextEditWindow__new_0, CTextEditWindow__new_in_0, CTextEditWindow__vtbl, CTextEditWindow__zeroed, CToolBox, CToolBox__CLASS_NAME, CToolBox__VTBL_own, CToolBox__WND_CLASS, CToolBox__delete, CToolBox__make_0, CToolBox__new_0, CToolBox__new_in_0, CToolBox__vtbl, CToolBox__zeroed, CToolSettingsWindow, CToolSettingsWindow__CLASS_NAME, CToolSettingsWindow__VTBL_own, CToolSettingsWindow__WND_CLASS, CToolSettingsWindow__delete, CToolSettingsWindow__make_0, CToolSettingsWindow__new_0, CToolSettingsWindow__new_in_0, CToolSettingsWindow__vtbl, CToolSettingsWindow__zeroed, CWaitCursor, CWaitCursor__delete, CWaitCursor__make_0, CWaitCursor__new_0, CWaitCursor__new_in_0, CWaitCursor__s_hOldCursor, CWaitCursor__s_hWaitCursor, CWaitCursor__s_nLock, CWaitCursor__zeroed, CWindow, CWindow__delete, CWindow__make_0, CWindow__new_0, CWindow__new_in_0, CWindow__rcDefault, CWindow__zeroed, ColorTool, ColorTool__VTBL_b0, ColorTool__delete, ColorTool__make_0, ColorTool__new_0, ColorTool__new_in_0, ColorTool__zeroed, CommandToToolMapping, DIRECTION, DIRECTION_DIAGONAL_RIGHT_DOWN, DIRECTION_DIAGONAL_RIGHT_UP, DIRECTION_HORIZONTAL, DIRECTION_VERTICAL, EllipseTool, EllipseTool__VTBL_b0b0, EllipseTool__delete, EllipseTool__make_0, EllipseTool__new_0, EllipseTool__new_in_0, EllipseTool__zeroed, FillTool, FillTool__VTBL_b0, FillTool__delete, FillTool__make_0, FillTool__new_0, FillTool__new_in_0, FillTool__zeroed, FreeSelTool, FreeSelTool__VTBL_b0b0, FreeSelTool__delete, FreeSelTool__make_0, FreeSelTool__new_0, FreeSelTool__new_in_0, FreeSelTool__zeroed, IMAGE_PART, IMAGE_PART__delete, IMAGE_PART__make_0, IMAGE_PART__new_0, IMAGE_PART__new_in_0, IMAGE_PART__zeroed, ImageModel, ImageModel__VTBL_own, ImageModel__delete, ImageModel__make_0, ImageModel__new_0, ImageModel__new_in_0, ImageModel__vtbl, ImageModel__zeroed, LineTool, LineTool__VTBL_b0b0, LineTool__delete, LineTool__make_0, LineTool__new_0, LineTool__new_in_0, LineTool__zeroed, MYBITMAPINFOEX, MYBITMAPINFOEX__delete, MYBITMAPINFOEX__make_0, MYBITMAPINFOEX__new_0, MYBITMAPINFOEX__new_in_0, MYBITMAPINFOEX__zeroed, NO_DIRECTION, PaletteModel, PaletteModel__delete, PaletteModel__make_0, PaletteModel__new_0, PaletteModel__new_in_0, PaletteModel__zeroed, PenTool, PenTool__VTBL_b0b0, PenTool__delete, PenTool__make_0, PenTool__new_0, PenTool__new_in_0, PenTool__zeroed, RRectTool, RRectTool__VTBL_b0b0, RRectTool__delete, RRectTool__make_0, RRectTool__new_0, RRectTool__new_in_0, RRectTool__zeroed, RectSelTool, RectSelTool__VTBL_b0b0, RectSelTool__delete, RectSelTool__make_0, RectSelTool__new_0, RectSelTool__new_in_0, RectSelTool__zeroed, RectTool, RectTool__VTBL_b0b0, RectTool__delete, RectTool__make_0, RectTool__new_0, RectTool__new_in_0, RectTool__zeroed, RegistrySettings, RegistrySettings__delete, RegistrySettings__make_0, RegistrySettings__new_0, RegistrySettings__new_in_0, RegistrySettings__zeroed, RubberTool, RubberTool__VTBL_b0b0, RubberTool__delete, RubberTool__make_0, RubberTool__new_0, RubberTool__new_in_0, RubberTool__zeroed, SHARED, SHARED__delete, SHARED__make_0, SHARED__new_0, SHARED__new_in_0, SHARED__zeroed, SelectionBaseTool, SelectionBaseTool__VTBL_b0, SelectionBaseTool__delete, SelectionBaseTool__make_0, SelectionBaseTool__new_0, SelectionBaseTool__new_in_0, SelectionBaseTool__zeroed, SelectionModel, SelectionModel__delete, SelectionModel__make_0, SelectionModel__new_0, SelectionModel__new_in_0, SelectionModel__zeroed, ShapeTool, ShapeTool__VTBL_b0, ShapeTool__delete, ShapeTool__make_0, ShapeTool__new_0, ShapeTool__new_in_0, ShapeTool__zeroed, SmoothDrawTool, SmoothDrawTool__VTBL_b0, SmoothDrawTool__delete, SmoothDrawTool__make_0, SmoothDrawTool__new_0, SmoothDrawTool__new_in_0, SmoothDrawTool__vtbl, SmoothDrawTool__zeroed, TextTool, TextTool__VTBL_b0, TextTool__delete, TextTool__make_0, TextTool__new_0, TextTool__new_in_0, TextTool__zeroed, ToolBase, ToolBase__VTBL_own, ToolBase__delete, ToolBase__make_0, ToolBase__new_0, ToolBase__new_in_0, ToolBase__vtbl, ToolBase__zeroed, ToolsModel, ToolsModel__delete, ToolsModel__make_0, ToolsModel__new_0, ToolsModel__new_in_0, ToolsModel__zeroed, TwoPointDrawTool, TwoPointDrawTool__VTBL_b0, TwoPointDrawTool__delete, TwoPointDrawTool__make_0, TwoPointDrawTool__new_0, TwoPointDrawTool__new_in_0, TwoPointDrawTool__zeroed, VirtualBrush, VirtualBrush__delete, VirtualBrush__make_0, VirtualBrush__new_0, VirtualBrush__new_in_0, VirtualBrush__zeroed, ZoomTool, ZoomTool__VTBL_b0, ZoomTool__delete, ZoomTool__make_0, ZoomTool__new_0, ZoomTool__new_in_0, ZoomTool__zeroed, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, attributesDialog, c_BrushPresets, canvasWindow, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, fontsDialog, fullscreenWindow, imageModel, mainWindow, miniature, mirrorRotateDialog, paletteModel, paletteWindow, registrySettings, selectionModel, stretchSkewDialog, textEditWindow, toolBoxContainer, toolSettingsWindow, toolsModel};
// ==== end port cppapp ====
