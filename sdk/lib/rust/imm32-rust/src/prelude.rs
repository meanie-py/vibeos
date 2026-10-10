//! imm32-rust prelude — the single import surface the translated modules use.
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
pub const ANSI_NULL: CHAR = 0; // atapi-rust/src/prelude.rs
pub const COINIT_APARTMENTTHREADED: DWORD = 0x2; // ole32-rust/src/compobj/com.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // console-rust/src/prelude.rs
pub const IDC_HAND: LPCWSTR = 32649 as LPCWSTR; // comctl32-rust/src/prelude.rs
pub const ImeInfoExImeFileName: IMEINFOEXCLASS = IMEINFOEXCLASS_ImeInfoExImeFileName; // island enum
pub const ImeInfoExKeyboardLayout: IMEINFOEXCLASS = IMEINFOEXCLASS_ImeInfoExKeyboardLayout; // island enum
pub const ImeInfoExKeyboardLayoutTFS: IMEINFOEXCLASS = IMEINFOEXCLASS_ImeInfoExKeyboardLayoutTFS; // island enum
pub const QIC_DEFAULTWINDOWIME: _QUERY_INPUT_CONTEXT = _QUERY_INPUT_CONTEXT_QIC_DEFAULTWINDOWIME; // island enum
pub const QIC_INPUTPROCESSID: _QUERY_INPUT_CONTEXT = _QUERY_INPUT_CONTEXT_QIC_INPUTPROCESSID; // island enum
pub const QIC_INPUTTHREADID: _QUERY_INPUT_CONTEXT = _QUERY_INPUT_CONTEXT_QIC_INPUTTHREADID; // island enum
pub const S_FALSE: HRESULT = 1; // comctl32-rust/src/prelude.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const THREADSTATE_DEFAULTIMEWINDOW: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_DEFAULTIMEWINDOW; // island enum
pub const THREADSTATE_DEFAULTINPUTCONTEXT: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_DEFAULTINPUTCONTEXT; // island enum
pub const THREADSTATE_GETTHREADINFO: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_GETTHREADINFO; // island enum
pub const THREADSTATE_IMECOMPATFLAGS: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_IMECOMPATFLAGS; // island enum
pub const THREADSTATE_ISWINLOGON: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_ISWINLOGON; // island enum
pub const THREADSTATE_OLDKEYBOARDLAYOUT: ThreadStateRoutines = ThreadStateRoutines_THREADSTATE_OLDKEYBOARDLAYOUT; // island enum
pub const TYPE_INPUTCONTEXT: _HANDLE_TYPE = _HANDLE_TYPE_TYPE_INPUTCONTEXT; // island enum
pub const TYPE_WINDOW: _HANDLE_TYPE = _HANDLE_TYPE_TYPE_WINDOW; // island enum
pub const UIC_CLIENTIMCDATA: _UPDATE_INPUT_CONTEXT = _UPDATE_INPUT_CONTEXT_UIC_CLIENTIMCDATA; // island enum
pub const VFT2_DRV_INPUTMETHOD: DWORD = 11; // version-rust/src/lib.rs
pub const VFT_DRV: DWORD = 3; // version-rust/src/lib.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> core::ffi::c_uchar; // rtl-diag/src/shim.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn labs(n: c_long) -> c_long; // desk-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
    pub fn wcscspn(s: *const u16, set: *const u16) -> usize; // kernel32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // inflib-rust/src/infget.rs
    pub fn wine_dbg_printf(format: *const c_char, ...) -> c_int; // oleaut32-rust/src/debug_shims.rs
}
extern "system" {
    pub fn AllocateAndInitializeSid( arg1: PSID_IDENTIFIER_AUTHORITY, arg2: BYTE, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: DWORD, arg9: DWORD, arg10: DWORD, arg11: *mut PSID, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn BaseCheckAppcompatCache(ApplicationName: PCWSTR, FileHandle: HANDLE, Environment: PCWSTR, pdwReason: PULONG,) -> BOOL; // kernel32-rust/src/client/appcache.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CharNextA(arg1: LPCSTR) -> LPSTR; // aclui-rust/src/types_amd64.rs
    pub fn CharNextW(arg1: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn CheckTokenMembership(arg1: HANDLE, arg2: PSID, arg3: PBOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateDCW( pszDriver: LPCWSTR, pszDevice: LPCWSTR, psz: LPCWSTR, pdmInit: *const DEVMODEW, ) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateDIBitmap( hdc: HDC, pbmih: *const BITMAPINFOHEADER, fdwInit: DWORD, pvInit: *const ::core::ffi::c_void, pbmi: *const BITMAPINFO, uUsage: UINT, ) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawEdge(arg1: HDC, arg2: LPRECT, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ExtTextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, options: UINT, lprect: *const RECT, lpString: LPCWSTR, c: UINT, lpDx: *const INT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FreeSid(arg1: PSID) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn GetACP() -> UINT; // crt-shared-rust/src/prelude.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetClassInfoExW(arg1: HINSTANCE, arg2: LPCWSTR, arg3: LPWNDCLASSEXW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClassInfoW(arg1: HINSTANCE, arg2: LPCWSTR, arg3: LPWNDCLASSW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDIBits( hdc: HDC, hbm: HBITMAP, start: UINT, cLines: UINT, lpvBits: LPVOID, lpbmi: LPBITMAPINFO, usage: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayoutList(nBuff: ::core::ffi::c_int, lpList: *mut HKL) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardState(lpKeyState: PBYTE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_amd64.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProfileIntW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: INT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDefaultLCID() -> LCID; // desk-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentPoint32W( hdc: HDC, lpString: LPCWSTR, c: ::core::ffi::c_int, psizl: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextFaceW(hdc: HDC, c: ::core::ffi::c_int, lpName: LPWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTextMetricsW(arg1: HDC, arg2: LPTEXTMETRICW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetThreadLocale() -> LCID; // desk-rust/src/types_amd64.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalHandle(arg1: LPCVOID) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_amd64.rs
    pub fn GlobalSize(arg1: HGLOBAL) -> SIZE_T; // aclui-rust/src/types_amd64.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsBadWritePtr(arg1: PVOID, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsDBCSLeadByte(arg1: BYTE) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn IsDBCSLeadByteEx(arg1: UINT, arg2: BYTE) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsWindowUnicode(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadBitmapW(arg1: HINSTANCE, arg2: LPCWSTR) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadKeyboardLayoutW(arg1: LPCWSTR, arg2: UINT) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFlags(arg1: HLOCAL) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalLock(arg1: HLOCAL) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalSize(arg1: HLOCAL) -> SIZE_T; // aclui-rust/src/types_amd64.rs
    pub fn LocalUnlock(arg1: HLOCAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn MapVirtualKeyW(arg1: UINT, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MessageBeep(uType: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn NtUserAssociateInputContext(hWnd: HWND, hIMC: HIMC, dwFlags: DWORD) -> DWORD; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserBuildHimcList( dwThreadId: DWORD, dwCount: DWORD, phList: *mut HIMC, pdwCount: PDWORD, ) -> NTSTATUS; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserCallOneParam(Param: DWORD_PTR, Routine: DWORD) -> DWORD_PTR; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserCreateInputContext(dwClientImcData: ULONG_PTR) -> HIMC; // win32k-rust/src/ime.rs
    pub fn NtUserDestroyInputContext(hIMC: HIMC) -> BOOL; // win32k-rust/src/ime.rs
    pub fn NtUserDisableThreadIme(dwThreadID: DWORD) -> BOOL; // win32k-rust/src/ime.rs
    pub fn NtUserGetAppImeLevel(hWnd: HWND) -> DWORD; // win32k-rust/src/ime.rs
    pub fn NtUserGetImeHotKey( dwHotKeyId: DWORD, lpuModifiers: PUINT, lpuVirtualKey: PUINT, lphKL: LPHKL, ) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserGetImeInfoEx(pImeInfoEx: PIMEINFOEX, SearchType: IMEINFOEXCLASS) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserGetThreadState(Routine: DWORD) -> DWORD_PTR; // win32k-rust/src/misc.rs
    pub fn NtUserNotifyIMEStatus(hwnd: HWND, fOpen: BOOL, dwConversion: DWORD) -> DWORD; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserQueryInputContext(hIMC: HIMC, dwType: DWORD) -> DWORD_PTR; // win32k-rust/src/ime.rs
    pub fn NtUserQueryWindow(hWnd: HWND, Index: DWORD) -> DWORD_PTR; // win32k-rust/src/window.rs
    pub fn NtUserSetAppImeLevel(hWnd: HWND, dwLevel: DWORD) -> BOOL; // win32k-rust/src/ime.rs
    pub fn NtUserSetImeInfoEx(pImeInfoEx: *const IMEINFOEX) -> BOOL; // win32k-rust/src/ime.rs
    pub fn NtUserSetThreadLayoutHandles(hNewKL: HKL, hOldKL: HKL) -> DWORD; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserUpdateInputContext(hIMC: HIMC, dwType: DWORD, dwValue: DWORD_PTR) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtUserValidateHandleSecure(hHdl: HANDLE) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn OpenFile(arg1: LPCSTR, arg2: LPOFSTRUCT, arg3: UINT) -> HFILE; // aclui-rust/src/types_amd64.rs
    pub fn OpenFileMappingW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostMessageA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn Rectangle( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageTimeoutW( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: UINT, arg6: UINT, arg7: PDWORD_PTR, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetBkColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_amd64.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ToAsciiEx( arg1: UINT, arg2: UINT, arg3: *const BYTE, arg4: LPWORD, arg5: UINT, arg6: HKL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ToUnicode( wVirtKey: UINT, wScanCode: UINT, lpKeyState: *const BYTE, pwszBuff: LPWSTR, cchBuff: ::core::ffi::c_int, wFlags: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn TranslateCharsetInfo(arg1: PDWORD, arg2: LPCHARSETINFO, arg3: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnloadKeyboardLayout(arg1: HKL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn User32InitializeImmEntryTable(arg1: DWORD) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn _lclose(arg1: HFILE) -> HFILE; // aclui-rust/src/types_amd64.rs
    pub fn keybd_event(arg1: BYTE, arg2: BYTE, arg3: DWORD, arg4: ULONG_PTR); // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::imm32::candidate::CandidateListAnsiToWide;
pub use crate::imm32::candidate::CandidateListWideToAnsi;
pub use crate::imm32::ctf::CtfImmIsGuidMapEnable;
pub use crate::imm32::utils::IchAnsiFromWide;
pub use crate::imm32::utils::IchWideFromAnsi;
pub use crate::imm32::utils::Imm32AnsiFromWide;
pub use crate::imm32::utils::Imm32BuildHimcList;
pub use crate::imm32::utils::Imm32CheckImcProcess;
pub use crate::imm32::utils::Imm32CurrentPti;
pub use crate::imm32::utils::Imm32FetchImeState;
pub use crate::imm32::ime::Imm32FindOrLoadImeDpi;
pub use crate::imm32::utils::Imm32GetSystemLibraryPath;
pub use crate::imm32::utils::Imm32IsCrossThreadAccess;
pub use crate::imm32::utils::Imm32IsImcAnsi;
pub use crate::imm32::utils::Imm32IsSystemJapaneseOrKorean;
pub use crate::imm32::utils::Imm32LoadImeStateSentence;
pub use crate::imm32::install::Imm32LoadImeVerInfo;
pub use crate::imm32::utils::Imm32MakeIMENotify;
pub use crate::imm32::utils::Imm32ReconvertAnsiFromWide;
pub use crate::imm32::utils::Imm32ReconvertWideFromAnsi;
pub use crate::imm32::utils::Imm32SaveImeStateSentence;
pub use crate::imm32::utils::Imm32StrToUInt;
pub use crate::imm32::utils::Imm32UIntToStr;
pub use crate::imm32::utils::Imm32WideFromAnsi;
pub use crate::imm32::utils::ImmCreateIMCC;
pub use crate::imm32::utils::ImmDestroyIMCC;
pub use crate::imm32::imm::ImmEnumInputContext;
pub use crate::imm32::ime::ImmGetDefaultIMEWnd;
pub use crate::imm32::utils::ImmGetIMCCLockCount;
pub use crate::imm32::utils::ImmGetIMCCSize;
pub use crate::imm32::ime::ImmIsIME;
pub use crate::imm32::ime::ImmLoadIME;
pub use crate::imm32::imm::ImmLoadLayout;
pub use crate::imm32::utils::ImmLocalAlloc;
pub use crate::imm32::imm::ImmLockClientImc;
pub use crate::imm32::imm::ImmLockIMC;
pub use crate::imm32::utils::ImmLockIMCC;
pub use crate::imm32::ime::ImmLockImeDpi;
pub use crate::imm32::utils::ImmReSizeIMCC;
pub use crate::imm32::imm::ImmSetActiveContext;
pub use crate::imm32::candidate::ImmSetCandidateWindow;
pub use crate::imm32::imm::ImmUnlockClientImc;
pub use crate::imm32::imm::ImmUnlockIMC;
pub use crate::imm32::utils::ImmUnlockIMCC;
pub use crate::imm32::ime::ImmUnlockImeDpi;
pub use crate::imm32::utils::LogFontAnsiToWide;
pub use crate::imm32::utils::LogFontWideToAnsi;
pub use crate::imm32::utils::ValidateHandle;
pub use crate::imm32::utils::ValidateHandleNoErr;
pub use crate::imm32::win3::WINNLSTranslateMessage;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const C1K_A: i32 = 0x1A; // clang, softkbd.c
pub const C1K_BACKSPACE: i32 = 0x2F; // clang, softkbd.c
pub const C1K_CAPS: i32 = 0x31; // clang, softkbd.c
pub const C1K_DELETE: i32 = 0x35; // clang, softkbd.c
pub const C1K_ENTER: i32 = 0x32; // clang, softkbd.c
pub const C1K_ESCAPE: i32 = 0x37; // clang, softkbd.c
pub const C1K_INSERT: i32 = 0x34; // clang, softkbd.c
pub const C1K_OEM_3: i32 = 0; // clang, softkbd.c
pub const C1K_Q: i32 = 0xD; // clang, softkbd.c
pub const C1K_SHIFT: i32 = 0x33; // clang, softkbd.c
pub const C1K_SPACE: i32 = 0x36; // clang, softkbd.c
pub const C1K_TAB: i32 = 0x30; // clang, softkbd.c
pub const C1K_Z: i32 = 0x25; // clang, softkbd.c
pub const HFILE_ERROR: HFILE = -1; // clang, install.c
pub const HKL_RELEASE_IME: HKL = 0x2usize as HKL; // clang, imm.c
pub const HKL_SWITCH_TO_NON_IME: HKL = 0x1usize as HKL; // clang, imm.c
pub const IDC_SIZEALL: LPWSTR = 0x7F86usize as LPWSTR; // clang, softkbd.c
pub const IDI_APPLICATION: LPWSTR = 0x7F00usize as LPWSTR; // clang, softkbd.c
pub const IGP_GETIMEVERSION: DWORD = 0xFFFFFFFC; // clang, ime.c
pub const LANGID_CHINESE_SIMPLIFIED: i32 = 0x804; // clang, keymsg.c
pub const LANGID_CHINESE_TRADITIONAL: i32 = 0x404; // clang, keymsg.c
pub const LANGID_JAPANESE: i32 = 0x411; // clang, keymsg.c
pub const MAX_SYSTEM_IMM_IME_ID: i32 = 0xE0FF; // clang, install.c
pub const MAX_USER_IMM_IME_ID: i32 = 0xE0FF; // clang, install.c
pub const MIN_SYSTEM_IMM_IME_ID: i32 = 0xE000; // clang, install.c
pub const MIN_USER_IMM_IME_ID: i32 = 0xE020; // clang, install.c
pub const SECURITY_NT_AUTHORITY: i32 = 5; // clang, ctf.c
pub const T1K_A: i32 = 0x1A; // clang, softkbd.c
pub const T1K_BACKSPACE: i32 = 0x2F; // clang, softkbd.c
pub const T1K_CAPS: i32 = 0x31; // clang, softkbd.c
pub const T1K_ENTER: i32 = 0x32; // clang, softkbd.c
pub const T1K_ESCAPE: i32 = 0x39; // clang, softkbd.c
pub const T1K_L_ALT: i32 = 0x37; // clang, softkbd.c
pub const T1K_L_CTRL: i32 = 0x35; // clang, softkbd.c
pub const T1K_L_SHIFT: i32 = 0x33; // clang, softkbd.c
pub const T1K_NONE: i32 = 0x3B; // clang, softkbd.c
pub const T1K_Q: i32 = 0xE; // clang, softkbd.c
pub const T1K_R_ALT: i32 = 0x38; // clang, softkbd.c
pub const T1K_R_CTRL: i32 = 0x36; // clang, softkbd.c
pub const T1K_R_SHIFT: i32 = 0x34; // clang, softkbd.c
pub const T1K_SPACE: i32 = 0x3A; // clang, softkbd.c
pub const T1K_TAB: i32 = 0x30; // clang, softkbd.c
pub const T1K_Z: i32 = 0x25; // clang, softkbd.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// PTHREADINFO: c2rust, utils.c
pub type PTHREADINFO = *mut _THREADINFO;
// VS_FIXEDFILEINFO: c2rust, install.c
pub type VS_FIXEDFILEINFO = tagVS_FIXEDFILEINFO;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_imm: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x69\x6d\x6d\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_imm };
// ==== end wine debug channels ====

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
#[macro_export] macro_rules! IS_CICERO_MODE { () => { (((((((gpsi) as isize) != 0) && (((((*gpsi).dwSRVIFlags & 0x0020)) as isize) != 0)) as c_int)) != 0) }; }
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int)) != 0) }; }
#[macro_export] macro_rules! IS_IMM_MODE { () => { (((((gpsi) as isize) != 0) && (((((*gpsi).dwSRVIFlags & 0x0004)) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKELRESULT { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LRESULT) }; }
pub type REFIID = *const IID;
pub use crate::{IS_CICERO_MODE, IS_IME_HKL, IS_IMM_MODE, IsEqualIID, MAKELPARAM, MAKELRESULT};
// ==== end SDK header macros ====

// ==== hand: ctf.c's two `#include <CtfImeTable.h>` X-macro passes ====
// DEFINE_CTF_IME_FN(name, ret, params) is expanded once into `typedef ret (WINAPI *FN_name)params;`
// and once into `FN_name g_pfnname = NULL;`. The preamble pass never saw the table (it is an
// #include inside the expansion), so the eight pairs are written out here from that header.
pub type FN_CtfImeCreateThreadMgr = Option<unsafe extern "system" fn() -> HRESULT>;
pub static mut g_pfnCtfImeCreateThreadMgr: FN_CtfImeCreateThreadMgr = None;
pub type FN_CtfImeDestroyThreadMgr = Option<unsafe extern "system" fn() -> HRESULT>;
pub static mut g_pfnCtfImeDestroyThreadMgr: FN_CtfImeDestroyThreadMgr = None;
pub type FN_CtfImeCreateInputContext = Option<unsafe extern "system" fn(hIMC: HIMC) -> HRESULT>;
pub static mut g_pfnCtfImeCreateInputContext: FN_CtfImeCreateInputContext = None;
pub type FN_CtfImeDestroyInputContext = Option<unsafe extern "system" fn(hIMC: HIMC) -> HRESULT>;
pub static mut g_pfnCtfImeDestroyInputContext: FN_CtfImeDestroyInputContext = None;
pub type FN_CtfImeSetActiveContextAlways = Option<unsafe extern "system" fn(hIMC: HIMC, fActive: BOOL, hWnd: HWND, hKL: HKL) -> HRESULT>;
pub static mut g_pfnCtfImeSetActiveContextAlways: FN_CtfImeSetActiveContextAlways = None;
pub type FN_CtfImeProcessCicHotkey = Option<unsafe extern "system" fn(hIMC: HIMC, vKey: UINT, lParam: LPARAM) -> HRESULT>;
pub static mut g_pfnCtfImeProcessCicHotkey: FN_CtfImeProcessCicHotkey = None;
pub type FN_CtfImeDispatchDefImeMessage = Option<unsafe extern "system" fn(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT>;
pub static mut g_pfnCtfImeDispatchDefImeMessage: FN_CtfImeDispatchDefImeMessage = None;
pub type FN_CtfImeIsIME = Option<unsafe extern "system" fn(hKL: HKL) -> BOOL>;
pub static mut g_pfnCtfImeIsIME: FN_CtfImeIsIME = None;

// ==== hand: compstr.c's CS_* accessors ====
// `CS_StrA(pCS, Name)` is `(LPCSTR)(pCS) + (pCS)->dw##Name##Offset`: a token paste, which
// macro_rules! cannot do on stable, so each COMPOSITIONSTRING member gets its own arm.
macro_rules! __cs_offset {
    ($p:expr, CompAttr) => { (*$p).dwCompAttrOffset };
    ($p:expr, CompClause) => { (*$p).dwCompClauseOffset };
    ($p:expr, CompReadAttr) => { (*$p).dwCompReadAttrOffset };
    ($p:expr, CompReadClause) => { (*$p).dwCompReadClauseOffset };
    ($p:expr, CompReadStr) => { (*$p).dwCompReadStrOffset };
    ($p:expr, CompStr) => { (*$p).dwCompStrOffset };
    ($p:expr, ResultClause) => { (*$p).dwResultClauseOffset };
    ($p:expr, ResultReadClause) => { (*$p).dwResultReadClauseOffset };
    ($p:expr, ResultReadStr) => { (*$p).dwResultReadStrOffset };
    ($p:expr, ResultStr) => { (*$p).dwResultStrOffset };
}
pub(crate) use __cs_offset;
macro_rules! __cs_len {
    ($p:expr, CompAttr) => { (*$p).dwCompAttrLen };
    ($p:expr, CompClause) => { (*$p).dwCompClauseLen };
    ($p:expr, CompReadAttr) => { (*$p).dwCompReadAttrLen };
    ($p:expr, CompReadClause) => { (*$p).dwCompReadClauseLen };
    ($p:expr, CompReadStr) => { (*$p).dwCompReadStrLen };
    ($p:expr, CompStr) => { (*$p).dwCompStrLen };
    ($p:expr, ResultClause) => { (*$p).dwResultClauseLen };
    ($p:expr, ResultReadClause) => { (*$p).dwResultReadClauseLen };
    ($p:expr, ResultReadStr) => { (*$p).dwResultReadStrLen };
    ($p:expr, ResultStr) => { (*$p).dwResultStrLen };
}
pub(crate) use __cs_len;
macro_rules! CS_StrA {
    ($pCS:expr, $name:ident $(,)?) => {
        (($pCS) as LPCSTR).wrapping_add(__cs_offset!($pCS, $name) as usize)
    };
}
pub(crate) use CS_StrA;
macro_rules! CS_StrW {
    ($pCS:expr, $name:ident $(,)?) => {
        (CS_StrA!($pCS, $name) as LPCWSTR)
    };
}
pub(crate) use CS_StrW;
macro_rules! CS_Size {
    ($pCS:expr, $name:ident $(,)?) => {
        __cs_len!($pCS, $name)
    };
}
pub(crate) use CS_Size;
macro_rules! CS_SizeA {
    ($pCS:expr, $name:ident $(,)?) => {
        (CS_Size!($pCS, $name) as usize * core::mem::size_of::<CHAR>())
    };
}
pub(crate) use CS_SizeA;
macro_rules! CS_SizeW {
    ($pCS:expr, $name:ident $(,)?) => {
        (CS_Size!($pCS, $name) as usize * core::mem::size_of::<WCHAR>())
    };
}
pub(crate) use CS_SizeW;

// ==== hand: ctf.c's dynamically-bound msctf entries ====
// `MSCTF_FN(name)` is `g_pfnMSCTF_##name` (a paste: one arm per pointer ctf.c declares) and
// `IMM32_GET_FN(..., func_name)` passes `#func_name`, a NUL-terminated C string.
macro_rules! MSCTF_FN {
    (TF_CreateLangBarMgr) => { g_pfnMSCTF_TF_CreateLangBarMgr };
    (TF_InvalidAssemblyListCacheIfExist) => { g_pfnMSCTF_TF_InvalidAssemblyListCacheIfExist };
}
pub(crate) use MSCTF_FN;
macro_rules! IMM32_GET_FN {
    ($ppfn:expr, $phinstDLL:expr, $dll_name:expr, $func_name:ident $(,)?) => {
        Imm32GetFn(
            ($ppfn) as *mut FARPROC,
            $phinstDLL,
            $dll_name,
            concat!(stringify!($func_name), "\0").as_ptr() as LPCSTR,
        )
    };
}
pub(crate) use IMM32_GET_FN;
macro_rules! Imm32GetMsctfFn {
    ($func_name:ident $(,)?) => {
        IMM32_GET_FN!(
            core::ptr::addr_of_mut!(MSCTF_FN!($func_name)),
            core::ptr::addr_of_mut!(g_hMsctf),
            wide_bytes!(b"msctf.dll") as LPCWSTR,
            $func_name
        )
    };
}
pub(crate) use Imm32GetMsctfFn;

// ==== hand: user32's TEB client-info accessors ====
// `NtCurrentTeb()->Win32ClientInfo` is a ULONG_PTR ARRAY: C's cast decays it to a pointer,
// so the Rust takes its address (the lowered `array as PCLIENTINFO` is E0605).
#[macro_export]
macro_rules! GetWin32ClientInfo {
    () => {
        (core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO)
    };
}
#[macro_export]
macro_rules! IS_16BIT_MODE {
    () => {
        ((((*GetWin32ClientInfo!()).dwTIFlags & TIF_16BIT as DWORD)) != 0)
    };
}
#[macro_export]
macro_rules! IS_CICERO_COMPAT_DISABLED {
    () => {
        ((((*GetWin32ClientInfo!()).dwCompatFlags2 & COMPAT_FLAG_2_CICERO_DISABLED as DWORD)) != 0)
    };
}
pub use crate::{GetWin32ClientInfo, IS_16BIT_MODE, IS_CICERO_COMPAT_DISABLED};

// ==== hand: function-scope #defines (ime.c, imm.c, install.c) ====
// Defined inside a function body in the C, so neither the preamble nor resolve saw them.
pub const VALID_IME_PROP: DWORD = IME_PROP_AT_CARET as DWORD | IME_PROP_SPECIAL_UI as DWORD | IME_PROP_CANDLIST_START_FROM_1 as DWORD | IME_PROP_UNICODE as DWORD | IME_PROP_COMPLETE_ON_UNSELECT as DWORD | IME_PROP_END_UNLOAD as DWORD | IME_PROP_KBD_CHAR_FIRST as DWORD | IME_PROP_IGNORE_UPKEYS as DWORD | IME_PROP_NEED_ALTKEY as DWORD | IME_PROP_NO_KEYS_ON_CLOSE as DWORD | IME_PROP_ACCEPT_WIDE_VKEY as DWORD;
pub const VALID_CMODE_CAPS: DWORD = IME_CMODE_ALPHANUMERIC as DWORD | IME_CMODE_NATIVE as DWORD | IME_CMODE_KATAKANA as DWORD | IME_CMODE_LANGUAGE as DWORD | IME_CMODE_FULLSHAPE as DWORD | IME_CMODE_ROMAN as DWORD | IME_CMODE_CHARCODE as DWORD | IME_CMODE_HANJACONVERT as DWORD | IME_CMODE_SOFTKBD as DWORD | IME_CMODE_NOCONVERSION as DWORD | IME_CMODE_EUDC as DWORD | IME_CMODE_SYMBOL as DWORD | IME_CMODE_FIXED as DWORD;
pub const VALID_SMODE_CAPS: DWORD = IME_SMODE_NONE as DWORD | IME_SMODE_PLAURALCLAUSE as DWORD | IME_SMODE_SINGLECONVERT as DWORD | IME_SMODE_AUTOMATIC as DWORD | IME_SMODE_PHRASEPREDICT as DWORD | IME_SMODE_CONVERSATION as DWORD;
pub const VALID_UI_CAPS: DWORD = UI_CAP_2700 as DWORD | UI_CAP_ROT90 as DWORD | UI_CAP_ROTANY as DWORD | UI_CAP_SOFTKBD as DWORD;
pub const VALID_SCS_CAPS: DWORD = SCS_CAP_COMPSTR as DWORD | SCS_CAP_MAKEREAD as DWORD | SCS_CAP_SETRECONVERTSTRING as DWORD;
pub const VALID_SELECT_CAPS: DWORD = SELECT_CAP_CONVERSION as DWORD | SELECT_CAP_SENTENCE as DWORD;
pub const MAX_IMCC_SIZE: DWORD = 0x1000; // imm.c
pub const MAX_PRELOAD: UINT = 0x400; // install.c

// ==== hand: ime.c's `Stub##name` X-macro over <imetable.h> ====
// `type APIENTRY Stub##name params { FIXME("%s: Why stub called?\n", #name); return (type)0; }`
#[no_mangle]
pub unsafe extern "system" fn StubImeInquire(_lpIMEInfo: LPIMEINFO, _lpszWndClass: LPVOID, _dwSystemInfoFlags: DWORD) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeInquire\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeConversionList(_hIMC: HIMC, _lpSrc: LPCVOID, _lpDst: LPCANDIDATELIST, _dwBufLen: DWORD, _uFlag: UINT) -> DWORD {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeConversionList\0".as_ptr());
    0 as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn StubImeRegisterWord(_lpszReading: LPCVOID, _dwStyle: DWORD, _lpszString: LPCVOID) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeRegisterWord\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeUnregisterWord(_lpszReading: LPCVOID, _dwStyle: DWORD, _lpszString: LPCVOID) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeUnregisterWord\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeGetRegisterWordStyle(_nItem: UINT, _lpStyleBuf: LPVOID) -> UINT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeGetRegisterWordStyle\0".as_ptr());
    0 as UINT
}
#[no_mangle]
pub unsafe extern "system" fn StubImeEnumRegisterWord(_lpfnEnumProc: LPVOID, _lpszReading: LPCVOID, _dwStyle: DWORD, _lpszString: LPCVOID, _lpData: LPVOID) -> UINT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeEnumRegisterWord\0".as_ptr());
    0 as UINT
}
#[no_mangle]
pub unsafe extern "system" fn StubImeConfigure(_hKL: HKL, _hWnd: HWND, _dwMode: DWORD, _lpData: LPVOID) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeConfigure\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeDestroy(_uReserved: UINT) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeDestroy\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeEscape(_hIMC: HIMC, _uEscape: UINT, _lpData: LPVOID) -> LRESULT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeEscape\0".as_ptr());
    0 as LRESULT
}
#[no_mangle]
pub unsafe extern "system" fn StubImeProcessKey(_hIMC: HIMC, _uVirKey: UINT, _lParam: LPARAM, _lpbKeyState: LPBYTE) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeProcessKey\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeSelect(_hIMC: HIMC, _fSelect: BOOL) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeSelect\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeSetActiveContext(_hIMC: HIMC, _fFlag: BOOL) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeSetActiveContext\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeToAsciiEx(_uVirKey: UINT, _uScanCode: UINT, _lpbKeyState: LPBYTE, _lpTransMsgList: LPTRANSMSGLIST, _fuState: UINT, _hIMC: HIMC) -> UINT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeToAsciiEx\0".as_ptr());
    0 as UINT
}
#[no_mangle]
pub unsafe extern "system" fn StubNotifyIME(_hIMC: HIMC, _dwAction: DWORD, _dwIndex: DWORD, _dwValue: DWORD_PTR) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"NotifyIME\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeSetCompositionString(_hIMC: HIMC, _dwIndex: DWORD, _lpComp: LPCVOID, _dwCompLen: DWORD, _lpRead: LPCVOID, _dwReadLen: DWORD) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeSetCompositionString\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubImeGetImeMenuItems(_hIMC: HIMC, _dwFlags: DWORD, _dwType: DWORD, _lpImeParentMenu: LPIMEMENUITEMINFOW, _lpImeMenu: LPIMEMENUITEMINFOW, _dwSize: DWORD) -> DWORD {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"ImeGetImeMenuItems\0".as_ptr());
    0 as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn StubCtfImeInquireExW(_lpIMEInfo: LPIMEINFO, _lpszWndClass: LPVOID, _dwSystemInfoFlags: DWORD, _hKL: HKL) -> HRESULT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"CtfImeInquireExW\0".as_ptr());
    0 as HRESULT
}
#[no_mangle]
pub unsafe extern "system" fn StubCtfImeSelectEx(_hIMC: HIMC, _fSelect: BOOL, _hKL: HKL) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"CtfImeSelectEx\0".as_ptr());
    0 as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn StubCtfImeEscapeEx(_hIMC: HIMC, _uSubFunc: UINT, _lpData: LPVOID, _hKL: HKL) -> LRESULT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"CtfImeEscapeEx\0".as_ptr());
    0 as LRESULT
}
#[no_mangle]
pub unsafe extern "system" fn StubCtfImeGetGuidAtom(_hIMC: HIMC, _dwUnknown: DWORD, _pdwGuidAtom: LPDWORD) -> HRESULT {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"CtfImeGetGuidAtom\0".as_ptr());
    0 as HRESULT
}
#[no_mangle]
pub unsafe extern "system" fn StubCtfImeIsGuidMapEnable(_hIMC: HIMC) -> BOOL {
    ::win32macros_rust::DPRINT1!("fixme: %s: Why stub called?\n\0", b"CtfImeIsGuidMapEnable\0".as_ptr());
    0 as BOOL
}

// ==== hand: more imm32 locals ====
// utils.c's function-scope #defines.
pub const INITIAL_COUNT: DWORD = 0x40; // nolint: const-contradiction (utils.c-local #define)
pub const MAX_RETRY: DWORD = 10;
// precomp.h prototypes these for every TU; the IS_CROSS_* macros expand at their call sites.
pub use crate::imm32::utils::Imm32IsCrossProcessAccess;
extern "system" {
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
}
extern "C" {
    pub static IID_IUnknown: IID;
}
// softkbd.c's `#define T1_KEYPOS(iKey) pT1->KeyPos[iKey]` reads the caller's `pT1`, which a
// macro_rules! cannot see (hygiene); the call sites pass it explicitly.
macro_rules! T1_KEYPOS {
    ($pT1:expr, $iKey:expr $(,)?) => {
        ((*$pT1).KeyPos[($iKey) as usize])
    };
}
pub(crate) use T1_KEYPOS;

// ==== hand: precomp.h's ValidateHwnd and win3.c's ALIGN_* ====
// ValidateHandle returns PVOID, which C converts implicitly; the PWND sites cast.
macro_rules! ValidateHwnd {
    ($hwnd:expr $(,)?) => {
        (ValidateHandle(($hwnd), TYPE_WINDOW))
    };
}
pub(crate) use ValidateHwnd;
macro_rules! ValidateHwndNoErr {
    ($hwnd:expr $(,)?) => {
        (ValidateHandleNoErr(($hwnd), TYPE_WINDOW))
    };
}
pub(crate) use ValidateHwndNoErr;
// Every use adds the size to a DWORD offset, so the size_t arithmetic is narrowed to DWORD
// as the C's assignment narrows it.
macro_rules! ALIGN_DWORD {
    ($len:expr $(,)?) => {
        ((($len) + 3) & !3)
    };
}
pub(crate) use ALIGN_DWORD;
macro_rules! ALIGN_ASTR_SIZE {
    ($len:expr $(,)?) => {
        (ALIGN_DWORD!(2 * ($len) as usize + core::mem::size_of::<CHAR>()) as DWORD)
    };
}
pub(crate) use ALIGN_ASTR_SIZE;
macro_rules! ALIGN_WSTR_SIZE {
    ($len:expr $(,)?) => {
        (ALIGN_DWORD!(($len) as usize * core::mem::size_of::<WCHAR>() + core::mem::size_of::<WCHAR>())
            as DWORD)
    };
}
pub(crate) use ALIGN_WSTR_SIZE;

// ==== hand: wine/debug.h's WARN ====
// resolve borrowed freeldr-rust's variadic `fn WARN` (a function there, linked into nothing
// here: i386 LINK_FAIL `_WARN`). In imm32 it is the wine macro, expanded the way every other
// site in this crate spells TRACE/FIXME/ERR: a channel-flag test, then ros_dbg_log.
macro_rules! WARN {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        if (*__wine_dbch___default).flags as c_int & (1 << __WINE_DBCL_WARN as c_int) != 0 {
            ros_dbg_log(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                concat!(file!(), "\0").as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
                line!() as c_int,
                $fmt $(, $arg)*
            );
        }
    };
}
pub(crate) use WARN;

// ==== hand macros: precomp.h's "Unexpected Condition Checkers" ====
// `port macros` refuses these (DBG and non-DBG define them differently). This is the DBG=1
// arm: log through ERR_PRINTF, and evaluate to the C's BOOL -- ERR_PRINTF itself yields TRUE,
// so the value is just the condition. `!(p)` is "is zero" for a pointer, handle or integer.
macro_rules! __imm32_unexpected {
    ($cond:expr, $what:literal, $expr:expr) => {{
        let __c: bool = $cond;
        if __c {
            ::win32macros_rust::DPRINT1!(
                concat!("err: %s ", $what, "\n\0"),
                concat!(stringify!($expr), "\0").as_ptr()
            );
        }
        ((__c as BOOL) != 0)
    }};
}
pub(crate) use __imm32_unexpected;
macro_rules! FAILED_UNEXPECTEDLY {
    ($hr:expr $(,)?) => {
        __imm32_unexpected!((($hr) as HRESULT) < 0, "FAILED", $hr)
    };
}
pub(crate) use FAILED_UNEXPECTEDLY;
macro_rules! IS_NULL_UNEXPECTEDLY {
    ($p:expr $(,)?) => {
        __imm32_unexpected!((($p) as usize) == 0, "was NULL", $p)
    };
}
pub(crate) use IS_NULL_UNEXPECTEDLY;
macro_rules! IS_ZERO_UNEXPECTEDLY {
    ($p:expr $(,)?) => {
        __imm32_unexpected!((($p) as usize) == 0, "was zero", $p)
    };
}
pub(crate) use IS_ZERO_UNEXPECTEDLY;
macro_rules! IS_TRUE_UNEXPECTEDLY {
    ($x:expr $(,)?) => {
        __imm32_unexpected!((($x) as usize) != 0, "was non-zero", $x)
    };
}
pub(crate) use IS_TRUE_UNEXPECTEDLY;
macro_rules! IS_ERROR_UNEXPECTEDLY {
    ($x:expr $(,)?) => {
        __imm32_unexpected!((($x) as u32) != ERROR_SUCCESS as u32, "was not ERROR_SUCCESS", $x)
    };
}
pub(crate) use IS_ERROR_UNEXPECTEDLY;

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// win32ss/user/imm32/precomp.h:74: #define IS_CROSS_PROCESS_HWND(hWnd) IS_TRUE_UNEXPECTEDLY(Imm32IsCrossProcessAccess(hWnd))
#[allow(unused_macros)]
macro_rules! IS_CROSS_PROCESS_HWND {
    ($hWnd:expr $(,)?) => {
        (IS_TRUE_UNEXPECTEDLY!(Imm32IsCrossProcessAccess($hWnd)))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_CROSS_PROCESS_HWND;
// win32ss/user/imm32/precomp.h:73: #define IS_CROSS_THREAD_HIMC(hIMC) IS_TRUE_UNEXPECTEDLY(Imm32IsCrossThreadAccess(hIMC))
#[allow(unused_macros)]
macro_rules! IS_CROSS_THREAD_HIMC {
    ($hIMC:expr $(,)?) => {
        (IS_TRUE_UNEXPECTEDLY!(Imm32IsCrossThreadAccess($hIMC)))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_CROSS_THREAD_HIMC;
// win32ss/user/imm32/precomp.h:117: #define ImeDpi_IsUnicode(pImeDpi) ((pImeDpi)->ImeInfo.fdwProperty & IME_PROP_UNICODE)
#[allow(unused_macros)]
macro_rules! ImeDpi_IsUnicode {
    ($pImeDpi:expr $(,)?) => {
        ((((*$pImeDpi).ImeInfo.fdwProperty & IME_PROP_UNICODE)) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use ImeDpi_IsUnicode;
// win32ss/user/imm32/ctf.c:568: #define Imm32DeleteIMMISPY(pSpy) ImmLocalFree(pSpy)
#[allow(unused_macros)]
macro_rules! Imm32DeleteIMMISPY {
    ($pSpy:expr $(,)?) => {
        (ImmLocalFree!($pSpy))
    };
}
#[allow(unused_imports)]
pub(crate) use Imm32DeleteIMMISPY;
// win32ss/user/imm32/precomp.h:102: #define ImmLocalFree(lpData) HeapFree(ghImmHeap, 0, (lpData))
#[allow(unused_macros)]
macro_rules! ImmLocalFree {
    ($lpData:expr $(,)?) => {
        (HeapFree(ghImmHeap, 0, ($lpData)))
    };
}
#[allow(unused_imports)]
pub(crate) use ImmLocalFree;
// win32ss/user/imm32/win3.c:22: #define KOR_IS_LEAD_BYTE(ch) (KOR_LEAD_BYTE_FIRST <= (BYTE)(ch) && (BYTE)(ch) <= KOR_LEAD_BYTE_LAST)
#[allow(unused_macros)]
macro_rules! KOR_IS_LEAD_BYTE {
    ($ch:expr $(,)?) => {
        (KOR_LEAD_BYTE_FIRST <= (($ch) as BYTE) && (($ch) as BYTE) <= KOR_LEAD_BYTE_LAST)
    };
}
#[allow(unused_imports)]
pub(crate) use KOR_IS_LEAD_BYTE;
// win32ss/user/imm32/candidate.c:12: #define ROUNDUP4(n) (((n) + 3) & ~3)
#[allow(unused_macros)]
macro_rules! ROUNDUP4 {
    ($n:expr $(,)?) => {
        ((($n) + 3) & !3)
    };
}
#[allow(unused_imports)]
pub(crate) use ROUNDUP4;
// ==== end port macros ====
