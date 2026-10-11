//! xcopy-rust prelude — the single import surface the translated modules use.
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
pub const CSTR_EQUAL: u32 = 2; // appwiz-rust/src/types_amd64.rs
pub const DATE_SHORTDATE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const NORM_IGNORECASE: u32 = 1; // appwiz-rust/src/types_amd64.rs
pub const SORT_STRINGSORT: u32 = 4096; // appwiz-rust/src/types_amd64.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // at-rust/src/prelude.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // cmd-rust/src/prelude.rs
pub const TIME_NOSECONDS: u32 = 2; // appwiz-rust/src/types_amd64.rs
extern "C" {
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wfopen(file: *const u16, mode: *const u16) -> *mut c_void; // comp-rust/src/prelude.rs
    pub fn _wsplitpath(path: *const u16, drive: *mut u16, dir: *mut u16, fname: *mut u16, ext: *mut u16) -> (); // cnt-rust/src/cabi/generated.rs
    pub fn _wtol(str: *const u16) -> i32; // cnt-rust/src/cabi/generated.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // cmd-rust/src/prelude.rs
    pub fn feof(file: *mut FILE) -> i32; // msconfig-rust/src/prelude.rs
    pub fn fgetws(s: *mut wchar_t, size: i32, file: *mut FILE) -> *mut wchar_t; // doskey-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn toupper(c: i32) -> i32; // apphelp-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // appwiz-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn WriteConsoleW( hConsoleOutput: *mut ::core::ffi::c_void, lpBuffer: *const ::core::ffi::c_void, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: LPDWORD, lpReserved: *mut ::core::ffi::c_void, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CharUpperBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn CompareStringW( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCWSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn GetConsoleOutputCP() -> UINT; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetCurrentDirectoryW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MAX_WRITECONSOLE_SIZE: i32 = 0xFFFF; // clang, xcopy.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_xcopy: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x78\x63\x6f\x70\x79\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_xcopy };
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
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn XCOPY_wprintf(format: *const WCHAR, ...) -> i32;
}
// ==== end port variadic ====
