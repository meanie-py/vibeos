//! conime-rust prelude — the single import surface the translated modules use.
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
pub const BACKGROUND_BLUE: u32 = 16; // attrib-rust/src/types_amd64.rs
pub const BACKGROUND_GREEN: u32 = 32; // attrib-rust/src/types_amd64.rs
pub const BLACK_BRUSH: u32 = 4; // access-rust/src/types_amd64.rs
pub const COMMON_LVB_UNDERSCORE: u32 = 32768; // attrib-rust/src/types_amd64.rs
pub const FOREGROUND_BLUE: u32 = 1; // attrib-rust/src/types_amd64.rs
pub const FOREGROUND_RED: u32 = 4; // attrib-rust/src/types_amd64.rs
pub const LANGID_CHINESE_SIMPLIFIED: i32 = 0x804; // imm32-rust/src/prelude.rs
pub const LANGID_CHINESE_TRADITIONAL: i32 = 0x404; // imm32-rust/src/prelude.rs
pub const LANGID_JAPANESE: i32 = 0x411; // imm32-rust/src/prelude.rs
pub const LANGID_KOREAN: i32 = 0x412; // msctfime-rust/src/prelude.rs
extern "C" {
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn ActivateKeyboardLayout(arg1: HKL, arg2: UINT) -> HKL; // aclui-rust/src/types_i386.rs
    pub fn AttachThreadInput(arg1: DWORD, arg2: DWORD, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardLayoutList(nBuff: ::core::ffi::c_int, lpList: *mut HKL) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetKeyboardLayoutNameW(arg1: LPWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ImmAssociateContext(hWnd: HWND, hIMC: HIMC) -> HIMC; // cicero-rust/src/types_i386.rs
    pub fn ImmCallImeConsoleIME( hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, puVK: LPUINT, ) -> DWORD; // cicero-rust/src/types_i386.rs
    pub fn ImmCreateContext() -> HIMC; // cicero-rust/src/types_i386.rs
    pub fn ImmDestroyContext(hIMC: HIMC) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmDisableTextFrameService(dwThreadId: DWORD) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmEscapeW(hKL: HKL, hIMC: HIMC, uSubFunc: UINT, lpData: LPVOID) -> LRESULT; // cicero-rust/src/types_i386.rs
    pub fn ImmGetCandidateListW( hIMC: HIMC, deIndex: DWORD, lpCandList: LPCANDIDATELIST, dwBufLen: DWORD, ) -> DWORD; // cicero-rust/src/types_i386.rs
    pub fn ImmGetCompositionStringW( hIMC: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD, ) -> LONG; // cicero-rust/src/types_i386.rs
    pub fn ImmGetContext(hWnd: HWND) -> HIMC; // cicero-rust/src/types_i386.rs
    pub fn ImmGetConversionStatus( hIMC: HIMC, lpfdwConversion: LPDWORD, lpfdwSentence: LPDWORD, ) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmGetGuideLineW(hIMC: HIMC, dwIndex: DWORD, lpBuf: LPWSTR, dwBufLen: DWORD) -> DWORD; // cicero-rust/src/types_i386.rs
    pub fn ImmGetIMEFileNameW(hKL: HKL, lpszFileName: LPWSTR, uBufLen: UINT) -> UINT; // cicero-rust/src/types_i386.rs
    pub fn ImmGetOpenStatus(hIMC: HIMC) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmGetProperty(hKL: HKL, fdwIndex: DWORD) -> DWORD; // cicero-rust/src/types_i386.rs
    pub fn ImmIsIME(hKL: HKL) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmNotifyIME(hIMC: HIMC, dwAction: DWORD, dwIndex: DWORD, dwValue: DWORD_PTR) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmReleaseContext(hWnd: HWND, hIMC: HIMC) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmSetActiveContextConsoleIME(hwnd: HWND, fFlag: BOOL) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmSetConversionStatus(hIMC: HIMC, fdwConversion: DWORD, fdwSentence: DWORD) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmSetOpenStatus(hIMC: HIMC, fOpen: BOOL) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmSimulateHotKey(hWnd: HWND, dwHotKeyID: DWORD) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn ImmTranslateMessage(hwnd: HWND, msg: UINT, wParam: WPARAM, lKeyData: LPARAM) -> BOOL; // cicero-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn OpenEventW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_i386.rs
    pub fn RegisterConsoleIME(hWndConsoleIME: HWND, lpdwConsoleThreadId: *mut DWORD) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn SendMessageTimeoutW( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: UINT, arg6: UINT, arg7: PDWORD_PTR, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetCurrentDirectoryW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnregisterClassW(arg1: LPCWSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnregisterConsoleIME() -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// PCHAR_INFO: c2rust, conime.c
pub type PCHAR_INFO = *mut _CHAR_INFO;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_conime: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x63\x6f\x6e\x69\x6d\x65\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_conime };
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
#[macro_export] macro_rules! CreateWindowW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! FillMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { memset((($p0) as _), (($p2) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int) }; }
#[macro_export] macro_rules! RtlEqualMemory { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), (($p2) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub use crate::{CreateWindowW, FillMemory, IS_IME_HKL, RtlEqualMemory, max, min};
// ==== end SDK header macros ====
