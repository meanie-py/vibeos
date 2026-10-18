//! taskmgr-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFFFFFF; // desk-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // console-rust/src/prelude.rs
pub const HWND_TOP: HWND = 0 as HWND; // comctl32-rust/src/prelude.rs
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // comctl32-rust/src/prelude.rs
pub const PROCESS_ALL_ACCESS: u32 = 2035711; // win32k-sys/src/bindgen_amd64.rs
pub const SB_SETTEXT: i32 = 0x40B; // charmap-rust/src/prelude.rs
pub const SECURITY_NT_AUTHORITY: i32 = 5; // imm32-rust/src/prelude.rs
pub const SE_KERNEL_OBJECT: _SE_OBJECT_TYPE = _SE_OBJECT_TYPE_SE_KERNEL_OBJECT; // island enum
pub const STILL_ACTIVE: u32 = 259; // msvcrt-sys/src/bindgen_amd64.rs
extern "C" {
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int; // browser-rust/src/prelude.rs
    pub fn _ultow(value: u32, string: *mut u16, radix: i32) -> *mut u16; // cnt-rust/src/cabi/generated.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // control-rust/src/prelude.rs
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn labs(n: c_long) -> c_long; // desk-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
extern "system" {
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn AllocateAndInitializeSid( arg1: PSID_IDENTIFIER_AUTHORITY, arg2: BYTE, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: DWORD, arg9: DWORD, arg10: DWORD, arg11: *mut PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn AppendMenuW(arg1: HMENU, arg2: UINT, arg3: UINT_PTR, arg4: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BringWindowToTop(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn CascadeWindows( hwndParent: HWND, wHow: UINT, lpRect: *const RECT, cKids: UINT, lpKids: *const HWND, ) -> WORD; // aclui-rust/src/types_i386.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuRadioItem(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CopySid(arg1: DWORD, arg2: PSID, arg3: PSID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreatePen(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: COLORREF) -> HPEN; // aclui-rust/src/types_i386.rs
    pub fn CreatePopupMenu() -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD,) -> HANDLE; // kernel32-rust/src/client/toolhelp.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DrawMenuBar(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndTask(arg1: HWND, arg2: BOOL, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EqualSid(arg1: PSID, arg2: PSID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExcludeClipRect( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ExitWindowsEx(arg1: UINT, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExtTextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, options: UINT, lprect: *const RECT, lpString: LPCWSTR, c: UINT, lpDx: *const INT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FreeSid(arg1: PSID) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetClassLongW")]
    pub fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR; // user32-rust/src/controls/appswitch.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetExitCodeProcess(arg1: HANDLE, arg2: PDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetExitCodeThread(arg1: HANDLE, arg2: PDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetGuiResources(arg1: HANDLE, arg2: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLengthSid(arg1: PSID) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemCount(arg1: HMENU) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetMenuState(arg1: HMENU, arg2: UINT, arg3: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetMenuStringW( hMenu: HMENU, uIDItem: UINT, lpString: LPWSTR, cchMax: ::core::ffi::c_int, flags: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetNumberFormatW( Locale: LCID, dwFlags: DWORD, lpValue: LPCWSTR, lpFormat: *const NUMBERFMTW, lpNumberStr: LPWSTR, cchNumber: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetPriorityClass(arg1: HANDLE) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessAffinityMask(arg1: HANDLE, arg2: PDWORD_PTR, arg3: PDWORD_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetProcessIoCounters(arg1: HANDLE, arg2: PIO_COUNTERS) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetSecurityInfo(handle: HANDLE, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, ppsidOwner: *mut PSID, ppsidGroup: *mut PSID, ppDacl: *mut PACL, ppSacl: *mut PACL, ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,) -> DWORD; // advapi32-rust/src/sec/misc.rs
    pub fn GetShellWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetTokenInformation( arg1: HANDLE, arg2: TOKEN_INFORMATION_CLASS, arg3: PVOID, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetUserNameW(lpBuffer: LPWSTR, pcbBuffer: LPDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Remove(himl: HIMAGELIST, i: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn InsertMenuW(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT_PTR, arg5: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWow64Process(arg1: HANDLE, arg2: PBOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsZoomed(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LineTo(arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadBitmapW(arg1: HINSTANCE, arg2: LPCWSTR) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LockWorkStation() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LookupAccountSidW( lpSystemName: LPCWSTR, Sid: PSID, Name: LPWSTR, cchName: LPDWORD, ReferencedDomainName: LPWSTR, cchReferencedDomainName: LPDWORD, peUse: PSID_NAME_USE, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ModifyMenuW(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT_PTR, arg5: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveToEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MsgWaitForMultipleObjects( nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn OpenIcon(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenProcess(arg1: DWORD, arg2: BOOL, arg3: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Process32FirstW(hSnapshot: HANDLE, lppe: LPPROCESSENTRY32W,) -> BOOL; // kernel32-rust/src/client/toolhelp.rs
    pub fn Process32NextW(hSnapshot: HANDLE, lppe: LPPROCESSENTRY32W,) -> BOOL; // kernel32-rust/src/client/toolhelp.rs
    pub fn QueryDosDeviceW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn ReadProcessMemory( arg1: HANDLE, arg2: LPCVOID, arg3: LPVOID, arg4: SIZE_T, arg5: PSIZE_T, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RestoreDC(arg1: HDC, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SaveDC(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetBkColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetMenuDefaultItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetPriorityClass(arg1: HANDLE, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessAffinityMask(arg1: HANDLE, arg2: DWORD_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessShutdownParameters(arg1: DWORD, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShellAboutW( hWnd: HWND, szApp: LPCWSTR, szOtherStuff: LPCWSTR, hIcon: HICON, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ShellExecuteExW(arg1: LPSHELLEXECUTEINFOW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn ShellExecuteW( hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: INT, ) -> HINSTANCE; // desk-rust/src/types_i386.rs
    pub fn Shell_NotifyIconW(arg1: DWORD, arg2: PNOTIFYICONDATAW) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindowAsync(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn StrFormatByteSizeW(arg1: LONGLONG, arg2: LPWSTR, cchBuf: UINT) -> LPWSTR; // desk-rust/src/types_i386.rs
    pub fn SwitchToThisWindow(arg1: HWND, arg2: BOOL); // aclui-rust/src/types_i386.rs
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TileWindows( hwndParent: HWND, wHow: UINT, lpRect: *const RECT, cKids: UINT, lpKids: *const HWND, ) -> WORD; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenuEx( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: HWND, arg6: LPTPMPARAMS, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const BRIGHT_GREEN: COLORREF = 0xFF00; // clang, graph.c
pub const DARK_GREEN: COLORREF = 0x8200; // clang, graph.c
pub const HDM_GETITEM: i32 = 0x120B; // clang, column.c
pub const HDN_ITEMCHANGED: u32 = 0xFFFFFEBF; // clang, procpage.c
pub const HDN_ITEMCLICK: u32 = 0xFFFFFEBE; // clang, applpage.c
pub const IDS_TAB_BASEPRIORITY: i32 = 0x149; // clang, about.c
pub const IDS_TAB_CPUUSAGE: i32 = 0x13F; // clang, about.c
pub const IDS_TAB_HANDLECOUNT: i32 = 0x14A; // clang, about.c
pub const IDS_TAB_IOWRITEBYTES: i32 = 0x152; // clang, about.c
pub const IDS_TAB_MEMORYUSAGE: i32 = 0x141; // clang, about.c
pub const IDS_TAB_MEMORYUSAGEDELTA: i32 = 0x143; // clang, about.c
pub const IDS_TAB_NONPAGEDPOOL: i32 = 0x148; // clang, about.c
pub const IDS_TAB_PAGEFAULTS: i32 = 0x144; // clang, about.c
pub const IDS_TAB_PAGEFAULTSDELTA: i32 = 0x145; // clang, about.c
pub const IDS_TAB_PEAKMEMORYUSAGE: i32 = 0x142; // clang, about.c
pub const IDS_TAB_THREADCOUNT: i32 = 0x14B; // clang, about.c
pub const IDS_TAB_USEROBJECTS: i32 = 0x14C; // clang, about.c
pub const IDS_TAB_VIRTUALMEMORYSIZE: i32 = 0x146; // clang, about.c
pub const LVN_GETDISPINFO: u32 = 0xFFFFFF4F; // clang, applpage.c
pub const MEDIUM_GREEN: COLORREF = 0xBE00; // clang, graph.c
pub const NM_CLICK: u32 = 0xFFFFFFFE; // clang, taskmgr.c
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // clang, applpage.c
pub const NM_RCLICK: u32 = 0xFFFFFFFB; // clang, applpage.c
pub const PAGE_OFFSET_HEIGHT: i32 = 0x68; // clang, taskmgr.c
pub const PAGE_OFFSET_LEFT: i32 = 0x11; // clang, taskmgr.c
pub const PAGE_OFFSET_TOP: i32 = 0x48; // clang, taskmgr.c
pub const PAGE_OFFSET_WIDTH: i32 = 0x22; // clang, taskmgr.c
pub const RED: COLORREF = 0xFF; // clang, graph.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, applpage.c
pub type HIMAGELIST = *mut _IMAGELIST;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CharUpperW(_: LPWSTR) -> LPWSTR; // procpage.c
    pub fn CreateStatusWindowW(style: LONG, lpszText: LPCWSTR, hwndParent: HWND, wID: UINT) -> HWND; // taskmgr.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // taskmgr.c
    pub fn EnumDisplaySettingsW(_: LPCWSTR, _: DWORD, _: PDEVMODEW) -> BOOL; // applpage.c
    pub fn FindWindowW(_: LPCWSTR, _: LPCWSTR) -> HWND; // taskmgr.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // taskmgr.c
    pub fn PeekMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT, _: UINT) -> BOOL; // taskmgr.c
    pub fn PostThreadMessageW(_: DWORD, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // applpage.c
}
pub use crate::prelude::CharUpperW as CharUpper; // sdk alias
pub use crate::prelude::CreateStatusWindowW as CreateStatusWindow; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::EnumDisplaySettingsW as EnumDisplaySettings; // sdk alias
pub use crate::prelude::FindWindowW as FindWindow; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::PeekMessageW as PeekMessage; // sdk alias
pub use crate::prelude::PostThreadMessageW as PostThreadMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
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

// ---- hand ports (outside the generated blocks, so a replay keeps them) ----
// perfdata.h:11 `Li2Double(x)`: C reads LARGE_INTEGER's ANONYMOUS struct; bindgen names it
// `u` (and `__bindgen_anon_1`, the same bytes), so `(x).HighPart` is `.u.HighPart` here.
#[allow(unused_macros)]
macro_rules! Li2Double {
    ($x:expr $(,)?) => {
        ((($x).u.HighPart as f64) * 4.294967296E9 + (($x).u.LowPart as f64))
    };
}
#[allow(unused_imports)]
pub(crate) use Li2Double;

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateDialogW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! Header_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 0)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! ListView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 9)) as _), ((0) as _), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_DeleteColumn { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 28)) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_DeleteItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 8)) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_EnsureVisible { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 19)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetExtendedListViewStyle { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 55)) as _), ((0) as _), ((0) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_GetHeader { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 31)) as _), ((0) as _), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! ListView_GetImageList { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 2)) as _), (((($p1) as INT)) as WPARAM), ((0) as _))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 75)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetNextItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 12)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetSelectedCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 50)) as _), ((0) as _), ((0) as _))) as UINT) }; }
#[macro_export] macro_rules! ListView_GetSelectionMark { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 66)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_RedrawItems { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 21)) as _), (((($p1) as c_int)) as WPARAM), (((($p2) as c_int)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyle { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), ((0) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 3)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 76)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = (($p3) as _); }; { _ms_lvi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! ListView_SortItems { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 48)) as _), (((($p2) as LPARAM)) as WPARAM), ((core::mem::transmute::<PFNLVCOMPARE, usize>(({ let __cb: PFNLVCOMPARE = $p1; __cb }))) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! TabCtrl_GetCurSel { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 11)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! TabCtrl_InsertItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 62)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! TabCtrl_SetCurFocus { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((0x1300 + 48)) as _), (($p1) as _), ((0) as _)) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub type HDITEM = HDITEMW;
pub const HWND_NOTOPMOST: HWND = (((-(2 as c_int))) as HWND);
pub const HWND_TOPMOST: HWND = (((-(1 as c_int))) as HWND);
pub type LPNMHEADER = LPNMHEADERW;
pub const LPSTR_TEXTCALLBACK: LPWSTR = (((-(1 as c_int))) as LPWSTR);
pub type LVCOLUMN = LVCOLUMNW;
pub type LVITEM = LVITEMW;
pub type LV_COLUMN = LVCOLUMNW;
pub type LV_DISPINFO = NMLVDISPINFOW;
pub type LV_ITEM = LVITEMW;
pub type NMLVDISPINFO = NMLVDISPINFOW;
pub const SE_DEBUG_NAME: *const u16 = (&[0x0053, 0x0065, 0x0044, 0x0065, 0x0062, 0x0075, 0x0067, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeDebugPrivilege"
pub const SE_SHUTDOWN_NAME: *const u16 = (&[0x0053, 0x0065, 0x0053, 0x0068, 0x0075, 0x0074, 0x0064, 0x006f, 0x0077, 0x006e, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeShutdownPrivilege"
pub type TCITEM = TCITEMW;
pub use crate::{CreateDialogW, DialogBoxW, Header_GetItemCount, ImageList_AddIcon, ListView_DeleteAllItems, ListView_DeleteColumn, ListView_DeleteItem, ListView_EnsureVisible, ListView_GetExtendedListViewStyle, ListView_GetHeader, ListView_GetImageList, ListView_GetItem, ListView_GetItemCount, ListView_GetNextItem, ListView_GetSelectedCount, ListView_GetSelectionMark, ListView_InsertColumn, ListView_InsertItem, ListView_RedrawItems, ListView_SetExtendedListViewStyle, ListView_SetImageList, ListView_SetItem, ListView_SetItemState, ListView_SortItems, MAKELPARAM, MAKEWPARAM, TabCtrl_GetCurSel, TabCtrl_InsertItem, TabCtrl_SetCurFocus, max, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/taskmgr/perfdata.c:38: #define CMD_LINE_MIN(a, b) (a < b ? a - sizeof(WCHAR) : b)
#[allow(unused_macros)]
macro_rules! CMD_LINE_MIN {
    ($a:expr, $b:expr $(,)?) => {
        (if $a < $b {
            $a - core::mem::size_of::<WCHAR>()
        } else {
            $b
        })
    };
}
#[allow(unused_imports)]
pub(crate) use CMD_LINE_MIN;
// base/applications/taskmgr/procpage.c:16: #define CMP(x1, x2) (x1 < x2 ? -1 : (x1 > x2 ? 1 : 0))
#[allow(unused_macros)]
macro_rules! CMP {
    ($x1:expr, $x2:expr $(,)?) => {
        (if $x1 < $x2 {
            -1
        } else if $x1 > $x2 {
            1
        } else {
            0
        })
    };
}
#[allow(unused_imports)]
pub(crate) use CMP;
// base/applications/taskmgr/procpage.c:19: #define CONST_STR_LEN(str) (_countof(str) - 1)
#[allow(unused_macros)]
macro_rules! CONST_STR_LEN {
    ($str:expr $(,)?) => {
        (_countof!($str) - 1)
    };
}
#[allow(unused_imports)]
pub(crate) use CONST_STR_LEN;
// ==== end port macros ====
