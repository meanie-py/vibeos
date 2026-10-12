//! replace-rust prelude — the single import surface the translated modules use.
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

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const EXIT_SUCCESS: u32 = 0; // shutdown-rust/src/prelude.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // cacls-rust/src/prelude.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // cmd-rust/src/prelude.rs
extern "C" {
    pub fn ConMsgPrintfV(Stream: *mut CON_STREAM, dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, Arguments: *mut va_list,) -> i32; // cmd-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn iswctype(c: u16, mask: u16) -> c_int; // cmd-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
}
extern "stdcall" {
    pub fn GetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, lpMode: LPDWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE, Add: ::core::ffi::c_int) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, dwMode: u32) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetConsoleCP() -> UINT; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFileTime( arg1: HANDLE, arg2: *const FILETIME, arg3: *const FILETIME, arg4: *const FILETIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn VirtualAlloc(arg1: PVOID, arg2: SIZE_T, arg3: DWORD, arg4: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn VirtualFree(arg1: PVOID, arg2: SIZE_T, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsupr(_String: *mut wchar_t) -> *mut wchar_t; // util.c
    pub fn _wgetdcwd(
        _Drive: ::core::ffi::c_int,
        _DstBuf: *mut wchar_t,
        _SizeInWords: ::core::ffi::c_int,
    ) -> *mut wchar_t; // util.c
    pub fn towupper(_C: wint_t) -> wint_t; // replace.c
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // replace.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // replace.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // replace.c
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // replace.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncat(_Dest: *mut wchar_t, _Source: *const wchar_t, _Count: size_t) -> *mut wchar_t; // util.c
    pub fn wcsncmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // util.c
}
extern "system" {
    pub fn CreateFileW(
        _: LPCWSTR,
        _: DWORD,
        _: DWORD,
        _: LPSECURITY_ATTRIBUTES,
        _: DWORD,
        _: DWORD,
        _: HANDLE,
    ) -> HANDLE; // replace.c
    pub fn DeleteFileW(_: LPCWSTR) -> BOOL; // replace.c
    pub fn FindFirstFileW(_: LPCWSTR, _: LPWIN32_FIND_DATAW) -> HANDLE; // replace.c
    pub fn FindNextFileW(_: HANDLE, _: LPWIN32_FIND_DATAW) -> BOOL; // replace.c
    pub fn GetCurrentDirectoryW(_: DWORD, _: LPWSTR) -> DWORD; // replace.c
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // replace.c
    pub fn GetFullPathNameW(_: LPCWSTR, _: DWORD, _: LPWSTR, _: *mut LPWSTR) -> DWORD; // replace.c
    pub fn ReadConsoleInputW(
        hConsoleInput: HANDLE,
        lpBuffer: PINPUT_RECORD,
        nLength: DWORD,
        lpNumberOfEventsRead: LPDWORD,
    ) -> BOOL; // util.c
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // replace.c
}
pub use crate::prelude::CreateFileW as CreateFile; // sdk alias
pub use crate::prelude::DeleteFileW as DeleteFile; // sdk alias
pub use crate::prelude::FindFirstFileW as FindFirstFile; // sdk alias
pub use crate::prelude::FindNextFileW as FindNextFile; // sdk alias
pub use crate::prelude::GetCurrentDirectoryW as GetCurrentDirectory; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetFullPathNameW as GetFullPathName; // sdk alias
pub use conutils_h::K32LoadStringW as LoadStringW; // sdk alias
pub use crate::prelude::ReadConsoleInputW as ReadConsoleInput; // sdk alias
pub use crate::prelude::SetFileAttributesW as SetFileAttributes; // sdk alias
pub use crate::prelude::wcscat as _tcscat; // sdk alias
pub use crate::prelude::wcschr as _tcschr; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::wcsncat as _tcsncat; // sdk alias
pub use crate::prelude::wcsncmp as _tcsncmp; // sdk alias
pub use crate::prelude::_wcsupr as _tcsupr; // sdk alias
pub use crate::prelude::_wgetdcwd as _tgetdcwd; // sdk alias
pub use crate::prelude::towupper as _totupper; // sdk alias
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
pub type WIN32_FIND_DATA = WIN32_FIND_DATAW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! _istspace { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x8) as _)) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub use crate::{_istspace, va_end, va_start};
// ==== end SDK header macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::replace::util::bCtrlBreak;
// ==== end cross-TU definitions ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/cmdutils/replace/replace.h:34: #define ConOutFormatMessage(MessageId, ...) ConFormatMessage(StdOut, (MessageId), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConOutFormatMessage {
    ($messageid:expr $(, $arg:expr)* $(,)?) => {
        ConFormatMessage(StdOut, ($messageid) $(, $arg)*)
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutFormatMessage;
// base/applications/cmdutils/replace/replace.h:31: #define ConOutResPrintf(uID, ...) ConResPrintf(StdOut, (uID), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConOutResPrintf {
    ($uid:expr $(, $arg:expr)* $(,)?) => {
        ConResPrintf(StdOut, ($uid) $(, $arg)*)
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutResPrintf;
// base/applications/cmdutils/replace/replace.h:28: #define ConOutResPuts(uID) ConResPuts(StdOut, (uID))
#[allow(unused_macros)]
macro_rules! ConOutResPuts {
    ($uid:expr $(,)?) => {
        ConResPuts(StdOut, ($uid))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutResPuts;
// ==== end port macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn ConFormatMessage(Stream: PCON_STREAM, MessageId: DWORD, ...);
}
// ==== end port variadic ====
