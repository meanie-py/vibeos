//! dinput-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_TYPECHECK, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs // gen:b2344a3cbdb6
pub const DIPROP_VIDPID: u32 = 0x18; // dxdiag-rust/src/prelude.rs // gen:68cef64bd77b
pub const DIRECTINPUT_VERSION: u32 = 2048; // dxdiag-rust/src/types_amd64.rs // gen:1c6473864d9b
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs // gen:845fcd2ce5ae
extern "C" {
    pub fn abs(x: i32) -> i32; // charmap-rust/src/prelude.rs // gen:0d23679e353a
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs // gen:13391f57321c
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4bffac11aa02
    pub static GUID_NULL: IID; // evr-rust/src/types_amd64.rs // gen:baf72c332eee
}
extern "system" {
    pub fn CallNextHookEx( arg1: HHOOK, arg2: ::core::ffi::c_int, arg3: WPARAM, arg4: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:e2583d3437b5
    pub fn ClipCursor(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:01f42bbf5e44
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4a43278b42fd
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:f162982b6326
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:fd1fce13f31c
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:29214eec4ab3
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:7893eb6ab4e4
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs // gen:ff91da798971
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:a39094df6b0b
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:0772d2cbe8ed
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:b4d73724c7aa
    pub fn EqualRect(arg1: LPCRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6bbe709f7fac
    pub fn FreeLibraryAndExitThread(arg1: HMODULE, arg2: DWORD) -> !; // aclui-rust/src/types_i386.rs // gen:ee650b60881b
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ef10fc0cc6f5
    pub fn GetClipCursor(arg1: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c2243c222dbc
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs // gen:b82f9257e472
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4ebf9f96bef7
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_i386.rs // gen:dec8495c75ca
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs // gen:102d2f98c4f8
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_i386.rs // gen:e7b5ca97e8ff
    pub fn GetKeyNameTextA( lParam: LONG, lpString: LPSTR, cchSize: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:6464c7ea66cf
    pub fn GetKeyNameTextW( lParam: LONG, lpString: LPWSTR, cchSize: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:c45d6c73f43e
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs // gen:39ec0894e330
    pub fn GetKeyboardType(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:a2d29aed1b65
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs // gen:50e246193424
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:2b769d30b290
    pub fn GetModuleFileNameA(hModule: HINSTANCE, lpFilename: LPSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:51bffb082f29
    pub fn GetModuleHandleExW(arg1: DWORD, arg2: LPCWSTR, arg3: *mut HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e24f313d56d9
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:8b50fa1e44f3
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:33a0dab07dca
    pub fn GetUserNameW(lpBuffer: LPWSTR, pcbBuffer: LPDWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:1e2aae1e0aa2
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:79545aae4953
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs // gen:f8e45f883779
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:cd40b76d9b6b
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:13245079d396
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:4cc9a88dd118
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:40af86961eab
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:d32c25758174
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:cb1bdfb02278
    pub fn MapVirtualKeyW(arg1: UINT, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs // gen:4b9b1551ed06
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:9c84986486a6
    pub fn MessageBoxA( hWnd: HWND, lpText: LPCSTR, lpCaption: LPCSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:779e8d865e63
    pub fn MsgWaitForMultipleObjectsEx( nCount: DWORD, pHandles: *const HANDLE, dwMilliseconds: DWORD, dwWakeMask: DWORD, dwFlags: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs // gen:aff79228c265
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:eaeee7dbe282
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:364345b52ec2
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8662c61ca102
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs // gen:0a9a5cfcc4a2
    pub fn PostThreadMessageW(arg1: DWORD, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs // gen:715d31b9109d
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:450c63aa5336
    pub fn SetCursorPos(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:2421e291c6bb
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:198f85e2baee
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:07cfdf4cb040
    pub fn SetWindowsHookExW( arg1: ::core::ffi::c_int, arg2: HOOKPROC, arg3: HINSTANCE, arg4: DWORD, ) -> HHOOK; // aclui-rust/src/types_i386.rs // gen:ec97e57f1b44
    pub fn ShowCursor(arg1: BOOL) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:1f87b872acd3
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs // gen:66dd7ed322c3
    pub fn UnhookWindowsHookEx(arg1: HHOOK) -> BOOL; // aclui-rust/src/types_i386.rs // gen:d6dccb9b9b7f
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:0d4f240a125e
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0e129f7cfff2
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4862658b57c4
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:dcf787d63821
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:0abd4a9770ae
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:244f3df56f54
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::dinput::dinput_main::C2RustUnnamed_3;
pub use crate::dinput::dinput_main::C2RustUnnamed_4;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const DIJOFS_RX: LONG = 0xC; // clang, joystick.c
pub const DIJOFS_RY: LONG = 0x10; // clang, joystick.c
pub const DIJOFS_RZ: LONG = 0x14; // clang, joystick.c
pub const DIJOFS_X: LONG = 0; // clang, joystick.c
pub const DIJOFS_Y: LONG = 4; // clang, joystick.c
pub const DIJOFS_Z: LONG = 8; // clang, joystick.c
pub const DIMOFS_X: LONG = 0; // clang, mouse.c
pub const DIMOFS_Y: LONG = 4; // clang, mouse.c
pub const DIPROP_AXISMODE: u32 = 2; // clang, device.c
pub const DIPROP_BUFFERSIZE: u32 = 1; // clang, device.c
pub const DIPROP_CALIBRATIONMODE: u32 = 0xA; // clang, joystick.c
pub const DIPROP_DEADZONE: u32 = 5; // clang, joystick.c
pub const DIPROP_GRANULARITY: u32 = 3; // clang, mouse.c
pub const DIPROP_INSTANCENAME: u32 = 0xD; // clang, joystick.c
pub const DIPROP_JOYSTICKID: u32 = 0xF; // clang, joystick.c
pub const DIPROP_KEYNAME: u32 = 0x14; // clang, keyboard.c
pub const DIPROP_PRODUCTNAME: u32 = 0xE; // clang, joystick.c
pub const DIPROP_RANGE: u32 = 4; // clang, device.c
pub const DIPROP_SATURATION: u32 = 6; // clang, joystick.c
pub const DIPROP_USERNAME: u32 = 0x19; // clang, device.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // device.c
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // joystick.c
    pub fn _strnicmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // mouse.c
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // device.c
    pub fn atoi(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // joystick.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // device.c
    pub fn strcat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // device.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // joystick.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // keyboard.c
    pub fn strncmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // joystick.c
    pub fn strrchr(
        _Str: *const ::core::ffi::c_char,
        _Ch: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // device.c
    pub fn strtok(
        _Str: *mut ::core::ffi::c_char,
        _Delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // joystick.c
    pub fn strtol(
        _Str: *const ::core::ffi::c_char,
        _EndPtr: *mut *mut ::core::ffi::c_char,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // joystick.c
}
extern "system" {
    pub fn GetTickCount() -> DWORD; // keyboard.c
}
pub use crate::prelude::GetTickCount as GetCurrentTime; // sdk alias
pub use crate::prelude::_snprintf as snprintf; // sdk alias
pub use crate::prelude::_swprintf as sprintfW; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_dinput: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x64\x69\x6e\x70\x75\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_dinput };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! DIDFT_GETINSTANCE { ($p0:expr $(,)?) => { (((((($p0 >> 8)) as DWORD_PTR) & 0xffff)) as WORD) }; } // gen:140ac5476c6b
#[macro_export] macro_rules! DIDFT_GETTYPE { ($p0:expr $(,)?) => { ((((($p0) as DWORD_PTR) & 0xff)) as BYTE) }; } // gen:6675aae48618
#[macro_export] macro_rules! DIDFT_MAKEINSTANCE { ($p0:expr $(,)?) => { ((((($p0) as WORD)) as c_int) << 8) }; } // gen:d8688e8fed21
#[macro_export] macro_rules! GET_DIDEVICE_TYPE { ($p0:expr $(,)?) => { ((((($p0) as DWORD_PTR) & 0xff)) as BYTE) }; } // gen:72e750c46426
#[macro_export] macro_rules! IDirectInput8_EnumDevicesBySemantics { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr $(,)?) => { ((*(*$p0).lpVtbl).EnumDevicesBySemantics).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _)) }; } // gen:b170ae18e2fe
#[macro_export] macro_rules! IDirectInput8_Initialize { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((*(*$p0).lpVtbl).Initialize).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:5c6ba034f9b9
#[macro_export] macro_rules! IDirectInput8_Release { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Release).unwrap_unchecked()((($p0) as _)) }; } // gen:a3a28a96acf8
#[macro_export] macro_rules! IDirectInputDevice2_AddRef { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).AddRef).unwrap_unchecked()((($p0) as _)) }; } // gen:65d0b3efdea5
#[macro_export] macro_rules! IDirectInputDevice2_GetProperty { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((*(*$p0).lpVtbl).GetProperty).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:8d071cf2bc0d
#[macro_export] macro_rules! IDirectInputDevice2_Poll { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Poll).unwrap_unchecked()((($p0) as _)) }; } // gen:c696a9154da4
#[macro_export] macro_rules! IDirectInputDevice8_BuildActionMap { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((*(*$p0).lpVtbl).BuildActionMap).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:d608f604a4d4
#[macro_export] macro_rules! IDirectInputDevice8_GetDeviceInfo { ($p0:expr, $p1:expr $(,)?) => { ((*(*$p0).lpVtbl).GetDeviceInfo).unwrap_unchecked()((($p0) as _), (($p1) as _)) }; } // gen:e61b982e1ba4
#[macro_export] macro_rules! IDirectInputDevice8_Release { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Release).unwrap_unchecked()((($p0) as _)) }; } // gen:f31180d6606e
#[macro_export] macro_rules! IDirectInputDevice8_SendForceFeedbackCommand { ($p0:expr, $p1:expr $(,)?) => { ((*(*$p0).lpVtbl).SendForceFeedbackCommand).unwrap_unchecked()((($p0) as _), (($p1) as _)) }; } // gen:4a8367f99a13
#[macro_export] macro_rules! IDirectInputDevice8_SetDataFormat { ($p0:expr, $p1:expr $(,)?) => { ((*(*$p0).lpVtbl).SetDataFormat).unwrap_unchecked()((($p0) as _), (($p1) as _)) }; } // gen:2c6301a505e2
#[macro_export] macro_rules! IDirectInputDevice8_SetProperty { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((*(*$p0).lpVtbl).SetProperty).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:a8b7f31b4d75
#[macro_export] macro_rules! IDirectInputDevice_AddRef { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).AddRef).unwrap_unchecked()((($p0) as _)) }; } // gen:43412ca6d824
#[macro_export] macro_rules! IDirectInputDevice_EnumObjects { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((*(*$p0).lpVtbl).EnumObjects).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:27388aba7548
#[macro_export] macro_rules! IDirectInputDevice_GetDeviceInfo { ($p0:expr, $p1:expr $(,)?) => { ((*(*$p0).lpVtbl).GetDeviceInfo).unwrap_unchecked()((($p0) as _), (($p1) as _)) }; } // gen:152e337edca2
#[macro_export] macro_rules! IDirectInputDevice_GetObjectInfo { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((*(*$p0).lpVtbl).GetObjectInfo).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:2541c7900dc4
#[macro_export] macro_rules! IDirectInputDevice_Release { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Release).unwrap_unchecked()((($p0) as _)) }; } // gen:37f0fcd96dfc
#[macro_export] macro_rules! IDirectInputDevice_Unacquire { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Unacquire).unwrap_unchecked()((($p0) as _)) }; } // gen:02bbdd853426
#[macro_export] macro_rules! IDirectInput_AddRef { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).AddRef).unwrap_unchecked()((($p0) as _)) }; } // gen:6abc8de29c65
#[macro_export] macro_rules! IDirectInput_CreateDevice { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((*(*$p0).lpVtbl).CreateDevice).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _)) }; } // gen:95dd00fd3c6f
#[macro_export] macro_rules! IDirectInput_Initialize { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((*(*$p0).lpVtbl).Initialize).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:2fd7a2145c0e
#[macro_export] macro_rules! IDirectInput_QueryInterface { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((*(*$p0).lpVtbl).QueryInterface).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:1235dddd16f9
#[macro_export] macro_rules! IDirectInput_Release { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Release).unwrap_unchecked()((($p0) as _)) }; } // gen:e52aeff4f3c2
#[macro_export] macro_rules! IsEqualCLSID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:f2d273a54bca
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { (((($p0) as WORD)) as ULONG_PTR) }; } // gen:80fecfa6e087
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
#[macro_export] macro_rules! strcpyW { ($p0:expr, $p1:expr $(,)?) => { wcscpy((($p0) as _), (($p1) as _)) }; } // gen:e40c0222f10c
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const DIERR_ACQUIRED: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((170) as c_uint))) as HRESULT); // gen:92c4fd832dde
pub const DIERR_BETADIRECTINPUTVERSION: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((1153) as c_uint))) as HRESULT); // gen:37520f141de3
pub const DIERR_DEVICENOTREG: HRESULT = ((0x80040154u32) as HRESULT); // gen:b2a0562b2d2b
pub const DIERR_GENERIC: HRESULT = ((0x80004005u32) as HRESULT); // gen:71905918cd95
pub const DIERR_INVALIDPARAM: HRESULT = ((0x80070057u32) as HRESULT); // gen:e26e24d74a69
pub const DIERR_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:b27596f7137c
pub const DIERR_NOMOREITEMS: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((259) as c_uint))) as HRESULT); // gen:096dc3192011
pub const DIERR_NOTACQUIRED: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((12) as c_uint))) as HRESULT); // gen:f8aac20e3eec
pub const DIERR_NOTINITIALIZED: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((21) as c_uint))) as HRESULT); // gen:d213d5a2c0e9
pub const DIERR_OBJECTNOTFOUND: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((2) as c_uint))) as HRESULT); // gen:515ad1b295dd
pub const DIERR_OLDDIRECTINPUTVERSION: HRESULT = (((((((1) as c_uint) << 31) | (((7) as c_uint) << 16)) | ((1150) as c_uint))) as HRESULT); // gen:ae4c0921b632
pub const DIERR_OTHERAPPHASPRIO: HRESULT = ((0x80070005u32) as HRESULT); // gen:f750c749ea4c
pub const DIERR_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:9bda7443b774
pub const DIERR_UNSUPPORTED: HRESULT = ((0x80004001u32) as HRESULT); // gen:879805cdc494
pub const DIPROPRANGE_NOMAX: LONG = ((0x7FFFFFFF) as LONG); // gen:4db34fbc3728
pub const DIPROPRANGE_NOMIN: LONG = ((0x80000000u32) as LONG); // gen:9a0320b51494
pub const DI_BUFFEROVERFLOW: HRESULT = ((1) as HRESULT); // gen:30391f2f3fd4
pub const DI_NOEFFECT: HRESULT = ((1) as HRESULT); // gen:aa4a185917c2
pub const DI_NOTATTACHED: HRESULT = ((1) as HRESULT); // gen:6633f17f5c74
pub const DI_OK: HRESULT = ((0) as HRESULT); // gen:730f86acf849
pub const DI_SETTINGSNOTSAVED: HRESULT = ((0x0000000B) as HRESULT); // gen:de11ef646925
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_HANDLE: HRESULT = ((0x80070006u32) as HRESULT); // gen:66b5f70fc222
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
#[inline] // gen:dc9a16403a7a
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{DIDFT_GETINSTANCE, DIDFT_GETTYPE, DIDFT_MAKEINSTANCE, GET_DIDEVICE_TYPE, IDirectInput8_EnumDevicesBySemantics, IDirectInput8_Initialize, IDirectInput8_Release, IDirectInputDevice2_AddRef, IDirectInputDevice2_GetProperty, IDirectInputDevice2_Poll, IDirectInputDevice8_BuildActionMap, IDirectInputDevice8_GetDeviceInfo, IDirectInputDevice8_Release, IDirectInputDevice8_SendForceFeedbackCommand, IDirectInputDevice8_SetDataFormat, IDirectInputDevice8_SetProperty, IDirectInputDevice_AddRef, IDirectInputDevice_EnumObjects, IDirectInputDevice_GetDeviceInfo, IDirectInputDevice_GetObjectInfo, IDirectInputDevice_Release, IDirectInputDevice_Unacquire, IDirectInput_AddRef, IDirectInput_CreateDevice, IDirectInput_Initialize, IDirectInput_QueryInterface, IDirectInput_Release, IsEqualCLSID, MAKEINTRESOURCE, max, min, strcpyW};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/directx/wine/dinput/dinput_private.h:86: #define IS_DIPROP(x) (((ULONG_PTR)(x) >> 16) == 0) // gen:2341f90f1fed
#[allow(unused_macros)]
macro_rules! IS_DIPROP {
    ($x:expr $(,)?) => {
        ((((($x) as usize) >> 16)) == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_DIPROP;
// dll/directx/wine/dinput/dinput_main.c:455: #define enum_callback_wrapper(callback, instance, ref) (callback)((instance), (ref)) // gen:3bf1de63e801
#[allow(unused_macros)]
macro_rules! enum_callback_wrapper {
    ($callback:expr, $instance:expr, $reference:expr $(,)?) => {
        (($callback)(($instance), ($reference)))
    };
}
#[allow(unused_imports)]
pub(crate) use enum_callback_wrapper;
// ==== end port macros ====
