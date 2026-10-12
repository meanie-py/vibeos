//! kbswitch-rust prelude — the single import surface the translated modules use.
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
pub const HWND_DESKTOP: HWND = 0x0usize as HWND; // eventvwr-rust/src/prelude.rs
pub const LANGID_CHINESE_SIMPLIFIED: i32 = 0x804; // imm32-rust/src/prelude.rs
pub const LANGID_CHINESE_TRADITIONAL: i32 = 0x404; // imm32-rust/src/prelude.rs
pub const LANGID_JAPANESE: i32 = 0x411; // imm32-rust/src/prelude.rs
pub const LANGID_KOREAN: i32 = 0x412; // msctfime-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn _wtoi(s: *const u16) -> i32; // chcp-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
}
extern "system" {
    pub fn ActivateKeyboardLayout(arg1: HKL, arg2: UINT) -> HKL; // aclui-rust/src/types_i386.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn CreatePopupMenu() -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DrawIconEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, arg8: HBRUSH, arg9: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetACP() -> UINT; // desk-rust/src/types_i386.rs
    pub fn GetAncestor(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardLayoutList(nBuff: ::core::ffi::c_int, lpList: *mut HKL) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemCount(arg1: HMENU) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSysColorBrush(arg1: ::core::ffi::c_int) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn ImmGetDefaultIMEWnd(hWnd: HWND) -> HWND; // cicero-rust/src/types_i386.rs
    pub fn ImmGetProperty(hKL: HKL, fdwIndex: DWORD) -> DWORD; // cicero-rust/src/types_i386.rs
    pub fn InSendMessageEx(arg1: LPVOID) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterShellHookWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetDCBrushColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenuEx( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: HWND, arg6: LPTPMPARAMS, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstoul(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong; // kbswitch.c
}
extern "system" {
    pub fn AppendMenuW(_: HMENU, _: UINT, _: UINT_PTR, _: LPCWSTR) -> BOOL; // kbswitch.c
    pub fn CreateFontIndirectW(_: *const LOGFONTW) -> HFONT; // kbswitch.c
    pub fn CreateMutexW(_: LPSECURITY_ATTRIBUTES, _: BOOL, _: LPCWSTR) -> HANDLE; // kbswitch.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // kbswitch.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // kbswitch.c
    pub fn DrawTextW(
        hdc: HDC,
        lpchText: LPCWSTR,
        cchText: ::core::ffi::c_int,
        lprc: LPRECT,
        format: UINT,
    ) -> ::core::ffi::c_int; // kbswitch.c
    pub fn EnumResourceNamesW(_: HMODULE, _: LPCWSTR, _: ENUMRESNAMEPROCW, _: LONG_PTR) -> BOOL; // kbswitch.c
    pub fn FindWindowExW(_: HWND, _: HWND, _: LPCWSTR, _: LPCWSTR) -> HWND; // kbswitch.c
    pub fn FindWindowW(_: LPCWSTR, _: LPCWSTR) -> HWND; // kbswitch.c
    pub fn GetClassNameW(
        hWnd: HWND,
        lpClassName: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // kbswitch.c
    pub fn GetLocaleInfoW(
        Locale: LCID,
        LCType: LCTYPE,
        lpLCData: LPWSTR,
        cchData: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // kbswitch.c
    pub fn GetMenuItemInfoW(_: HMENU, _: UINT, _: BOOL, _: LPMENUITEMINFOW) -> BOOL; // kbswitch.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // kbswitch.c
    pub fn GetSystemDirectoryW(_: LPWSTR, _: UINT) -> UINT; // kbswitch.c
    pub fn GlobalGetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: ::core::ffi::c_int) -> UINT; // kbswitch.c
    pub fn ImmGetDescriptionW(hKL: HKL, lpszDescription: LPWSTR, uBufLen: UINT) -> UINT; // kbswitch.c
    pub fn ImmGetImeMenuItemsW(
        hIMC: HIMC,
        dwFlags: DWORD,
        dwType: DWORD,
        lpImeParentMenu: LPIMEMENUITEMINFOW,
        lpImeMenu: LPIMEMENUITEMINFOW,
        dwSize: DWORD,
    ) -> DWORD; // imemenu.c
    pub fn InsertMenuItemW(_: HMENU, _: UINT, _: BOOL, _: LPCMENUITEMINFOW) -> BOOL; // kbswitch.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // kbswitch.c
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // kbswitch.c
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // kbswitch.c
    pub fn LoadKeyboardLayoutW(_: LPCWSTR, _: UINT) -> HKL; // kbswitch.c
    pub fn LoadLibraryExW(_: LPCWSTR, _: HANDLE, _: DWORD) -> HINSTANCE; // kbswitch.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // kbswitch.c
    pub fn LoadMenuW(_: HINSTANCE, _: LPCWSTR) -> HMENU; // kbswitch.c
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // kbswitch.c
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // kbswitch.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // kbswitch.c
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // kbswitch.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // kbswitch.c
    pub fn RegOpenKeyW(_: HKEY, _: LPCWSTR, _: PHKEY) -> LSTATUS; // kbswitch.c
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // kbswitch.c
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS; // kbswitch.c
    pub fn RegisterClassW(_: *const WNDCLASSW) -> ATOM; // kbswitch.c
    pub fn RegisterWindowMessageW(_: LPCWSTR) -> UINT; // kbswitch.c
    pub fn SHLoadRegUIStringW(hkey: HKEY, value: LPCWSTR, buf: LPWSTR, size: DWORD) -> HRESULT; // kbswitch.c
    pub fn SendMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // kbswitch.c
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE; // kbswitch.c
    pub fn Shell_NotifyIconW(_: DWORD, _: PNOTIFYICONDATAW) -> BOOL; // kbswitch.c
    pub fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL; // kbswitch.c
    pub fn lstrcmpiW(_: LPCWSTR, _: LPCWSTR) -> ::core::ffi::c_int; // kbswitch.c
    pub fn lstrlenW(_: LPCWSTR) -> ::core::ffi::c_int; // kbswitch.c
}
pub use crate::prelude::AppendMenuW as AppendMenu; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DrawTextW as DrawText; // sdk alias
pub use crate::prelude::EnumResourceNamesW as EnumResourceNames; // sdk alias
pub use crate::prelude::FindWindowW as FindWindow; // sdk alias
pub use crate::prelude::FindWindowExW as FindWindowEx; // sdk alias
pub use crate::prelude::GetClassNameW as GetClassName; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::GetMenuItemInfoW as GetMenuItemInfo; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetSystemDirectoryW as GetSystemDirectory; // sdk alias
pub use crate::prelude::GlobalGetAtomNameW as GlobalGetAtomName; // sdk alias
pub use crate::prelude::ImmGetDescriptionW as ImmGetDescription; // sdk alias
pub use crate::prelude::ImmGetImeMenuItemsW as ImmGetImeMenuItems; // sdk alias
pub use crate::prelude::InsertMenuItemW as InsertMenuItem; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadKeyboardLayoutW as LoadKeyboardLayout; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadLibraryExW as LoadLibraryEx; // sdk alias
pub use crate::prelude::LoadMenuW as LoadMenu; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyW as RegOpenKey; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::RegisterClassW as RegisterClass; // sdk alias
pub use crate::prelude::RegisterWindowMessageW as RegisterWindowMessage; // sdk alias
pub use crate::prelude::SHLoadRegUIStringW as SHLoadRegUIString; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::ShellExecuteW as ShellExecute; // sdk alias
pub use crate::prelude::Shell_NotifyIconW as Shell_NotifyIcon; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use crate::prelude::SystemParametersInfoW as SystemParametersInfo; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::wcstoul as _tcstoul; // sdk alias
pub use crate::prelude::lstrcmpiW as lstrcmpi; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_internat: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x69\x6e\x74\x65\x72\x6e\x61\x74\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_internat };
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
pub type MENUITEMINFO = MENUITEMINFOW;
pub type WNDCLASS = WNDCLASSW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindow { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! FillMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { memset((($p0) as _), (($p2) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int) }; }
#[macro_export] macro_rules! IS_SPECIAL_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xF0000000) as c_int) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const INDICATOR_CLASSW: *const u16 = (&[0x0049, 0x006e, 0x0064, 0x0069, 0x0063, 0x0061, 0x0074, 0x006f, 0x0072, 0x0000u16]).as_ptr(); // L"Indicator"
pub const RT_GROUP_ICON: LPWSTR = 0xeusize as LPWSTR;
pub use crate::{CreateWindow, FillMemory, IS_IME_HKL, IS_SPECIAL_HKL, MAKEINTRESOURCE};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/kbswitch/kbswitch.c:47: #define IS_KOREAN_IME_HKL(hKL) ((HandleToUlong(hKL) & IME_HKL_MASK) == 0xE0000412)
#[allow(unused_macros)]
macro_rules! IS_KOREAN_IME_HKL {
    ($hKL:expr $(,)?) => {
        ((HandleToUlong!($hKL) & IME_HKL_MASK) == 0xE0000412)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_KOREAN_IME_HKL;
// base/applications/kbswitch/kbswitch.h:31: header-inline `IsWndClassName`
#[allow(dead_code)]
#[inline]
pub unsafe fn IsWndClassName(hwndTarget: HWND, pszName: PCTSTR) -> BOOL {
    let mut szClass: [TCHAR; 32] = [0; 32];
    GetClassName(hwndTarget, szClass.as_mut_ptr(), _countof!(szClass) as _);
    (lstrcmpi(szClass.as_ptr(), pszName) == 0) as BOOL
}
// ==== end port macros ====
