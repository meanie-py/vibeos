//! more-rust prelude — the single import surface the translated modules use.
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

// ================= console / pager (hand port, $0) ==========================================
// The bindgen islands do not agree on these: types_i386 carries them, types_amd64/arm64 do not
// (gen_driver_types.sh died mid-run on i386), and PCTCH resolves to a narrow LPCCH on i386. The
// UNICODE tree defines PCTCH = LPCWCH (sdk/include/xdk/ntbasedef.h). Layouts mirror wincon.h and
// sdk/lib/conutils/pager.h; every field is fixed-width or pointer-width, so this is per-arch exact.
pub type PCTCH = *const WCHAR;
pub const KEY_EVENT: WORD = 1;
pub const ENABLE_PROCESSED_INPUT: u32 = 1;
pub const LEFT_CTRL_PRESSED: u32 = 0x0008;
pub const RIGHT_CTRL_PRESSED: u32 = 0x0004;
pub const CON_PAGER_EXPAND_TABS: u32 = 1 << 0;
pub const CON_PAGER_EXPAND_FF: u32 = 1 << 1;
pub const CON_PAGER_CACHE_INCOMPLETE_LINE: u32 = 1 << 2;
// The i386 callback ABI is stdcall; every other arch is "system" (the same ABI, spelled portably).
#[cfg(target_arch = "x86")]
pub type PHANDLER_ROUTINE = ::core::option::Option<unsafe extern "stdcall" fn(CtrlType: DWORD) -> BOOL>;
#[cfg(not(target_arch = "x86"))]
pub type PHANDLER_ROUTINE = ::core::option::Option<unsafe extern "system" fn(CtrlType: DWORD) -> BOOL>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _KEY_EVENT_RECORD {
    pub bKeyDown: BOOL,
    pub wRepeatCount: WORD,
    pub wVirtualKeyCode: WORD,
    pub wVirtualScanCode: WORD,
    pub uChar: _KEY_EVENT_RECORD__bindgen_ty_1,
    pub dwControlKeyState: DWORD,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union _KEY_EVENT_RECORD__bindgen_ty_1 {
    pub UnicodeChar: WCHAR,
    pub AsciiChar: CHAR,
}
pub type KEY_EVENT_RECORD = _KEY_EVENT_RECORD;
pub type PKEY_EVENT_RECORD = *mut _KEY_EVENT_RECORD;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MOUSE_EVENT_RECORD {
    pub dwMousePosition: COORD,
    pub dwButtonState: DWORD,
    pub dwControlKeyState: DWORD,
    pub dwEventFlags: DWORD,
}
pub type MOUSE_EVENT_RECORD = _MOUSE_EVENT_RECORD;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _WINDOW_BUFFER_SIZE_RECORD {
    pub dwSize: COORD,
}
pub type WINDOW_BUFFER_SIZE_RECORD = _WINDOW_BUFFER_SIZE_RECORD;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MENU_EVENT_RECORD {
    pub dwCommandId: UINT,
}
pub type MENU_EVENT_RECORD = _MENU_EVENT_RECORD;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _FOCUS_EVENT_RECORD {
    pub bSetFocus: BOOL,
}
pub type FOCUS_EVENT_RECORD = _FOCUS_EVENT_RECORD;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _INPUT_RECORD {
    pub EventType: WORD,
    pub Event: _INPUT_RECORD__bindgen_ty_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union _INPUT_RECORD__bindgen_ty_1 {
    pub KeyEvent: KEY_EVENT_RECORD,
    pub MouseEvent: MOUSE_EVENT_RECORD,
    pub WindowBufferSizeEvent: WINDOW_BUFFER_SIZE_RECORD,
    pub MenuEvent: MENU_EVENT_RECORD,
    pub FocusEvent: FOCUS_EVENT_RECORD,
}
pub type INPUT_RECORD = _INPUT_RECORD;
pub type PINPUT_RECORD = *mut _INPUT_RECORD;
// _CON_SCREEN.Stream must be conutils-h's PCON_STREAM (the one ConStreamWrite & co. take); the
// island's own PCON_STREAM is a distinct struct, so the island's _CON_SCREEN cannot be used.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CON_SCREEN {
    pub Stream: PCON_STREAM,
    pub csbi: CONSOLE_SCREEN_BUFFER_INFO,
    pub cci: CONSOLE_CURSOR_INFO,
}
pub type CON_SCREEN = _CON_SCREEN;
pub type PCON_SCREEN = *mut _CON_SCREEN;
#[cfg(target_arch = "x86")]
pub type CON_PAGER_LINE_FN = ::core::option::Option<
    unsafe extern "stdcall" fn(Pager: *mut _CON_PAGER, line: PCTCH, cch: SIZE_T) -> BOOL,
>;
#[cfg(not(target_arch = "x86"))]
pub type CON_PAGER_LINE_FN = ::core::option::Option<
    unsafe extern "system" fn(Pager: *mut _CON_PAGER, line: PCTCH, cch: SIZE_T) -> BOOL,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CON_PAGER {
    pub Screen: PCON_SCREEN,
    pub PageColumns: DWORD,
    pub PageRows: DWORD,
    pub PagerLine: CON_PAGER_LINE_FN,
    pub dwFlags: DWORD,
    pub nTabWidth: LONG,
    pub ScrollRows: DWORD,
    pub CachedLine: PCTCH,
    pub cchCachedLine: SIZE_T,
    pub ich: SIZE_T,
    pub CurrentLine: PCTCH,
    pub ichCurr: SIZE_T,
    pub iEndLine: SIZE_T,
    pub nSpacePending: DWORD,
    pub iColumn: DWORD,
    pub iLine: DWORD,
    pub lineno: DWORD,
}
pub type CON_PAGER = _CON_PAGER;
pub type PCON_PAGER = *mut _CON_PAGER;
#[cfg(target_arch = "x86")]
pub type PAGE_PROMPT = ::core::option::Option<
    unsafe extern "stdcall" fn(Pager: PCON_PAGER, Done: DWORD, Total: DWORD) -> BOOL,
>;
#[cfg(not(target_arch = "x86"))]
pub type PAGE_PROMPT = ::core::option::Option<
    unsafe extern "system" fn(Pager: PCON_PAGER, Done: DWORD, Total: DWORD) -> BOOL,
>;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const STD_ERROR_HANDLE: u32 = 4294967284; // cmd-rust/src/prelude.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // cmd-rust/src/prelude.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // cmd-rust/src/prelude.rs
extern "C" {
    pub fn ConClearScreen(Screen: *const CON_SCREEN); // cmd-rust/src/prelude.rs
    pub fn ConGetScreenInfo(screen: *mut CON_SCREEN, csbi: *mut CONSOLE_SCREEN_BUFFER_INFO) -> i32; // conutils-rust/src/conutils/pager.rs
    pub fn ConWritePaging(Pager: *mut CON_PAGER, PagePrompt: PAGE_PROMPT, StartPaging: i32, szStr: *const u16, len: u32,) -> i32; // cmd-rust/src/prelude.rs
    pub fn _wtol(str: *const u16) -> i32; // cnt-rust/src/cabi/generated.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn towupper(c: c_int) -> c_int; // comctl32-rust/src/prelude.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16; // desk-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // comctl32-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn GetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, lpMode: LPDWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE, Add: ::core::ffi::c_int) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, dwMode: u32) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FlushConsoleInputBuffer(hConsoleInput: HANDLE) -> BOOL; // cmd-rust/src/prelude.rs
    pub fn GetConsoleCP() -> UINT; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStringTypeW( dwInfoType: DWORD, lpSrcStr: LPCWSTR, cchSrc: ::core::ffi::c_int, lpCharType: LPWORD, ) -> BOOL; // desk-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn IsTextUnicode( lpv: *const ::core::ffi::c_void, iSize: ::core::ffi::c_int, lpiResult: LPINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SetConsoleCursorPosition(hConsoleOutput: *mut ::core::ffi::c_void, dwCursorPosition: COORD) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn ConClearLine(Stream: PCON_STREAM); // more.c
}
extern "system" {
    pub fn ReadConsoleInputW(
        hConsoleInput: HANDLE,
        lpBuffer: PINPUT_RECORD,
        nLength: DWORD,
        lpNumberOfEventsRead: LPDWORD,
    ) -> BOOL; // more.c
}
pub use conutils_h::K32LoadStringW as LoadStringW; // sdk alias
pub use crate::prelude::ReadConsoleInputW as ReadConsoleInput; // sdk alias
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
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
// ==== end SDK header macros ====
