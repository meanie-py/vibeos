//! setupapi-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// wine/unicode.h, as THIS tree has it (sdk/include/wine/unicode.h, the one setupapi_private.h
// includes): not Wine's inline loops but `#define strlenW(s) wcslen((s))` and friends, so the
// C calls the CRT. Aliases, then, exactly as the #defines are -- no bodies of our own.
extern "C" {
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wcstol(s: *const u16, end: *mut *mut u16, base: c_int) -> c_long;
}
pub use self::{
    wcscat as strcatW, wcscmp as strcmpW, wcscpy as strcpyW, wcslen as strlenW,
    wcsrchr as strrchrW, wcstol as strtolW,
};
// stdlib.h `max(a,b)`: the shared transcription.
pub use win32macros_rust::{max, InterlockedExchangePointer};
// winnt.h (xdk/winnt_old.h:225) `INVALID_FILE_ATTRIBUTES ((DWORD)-1)`, winbase.h:118
// `INVALID_SET_FILE_POINTER ((DWORD)-1)`;
// bindgen does not fold the casts.
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFF_FFFF;
pub const INVALID_SET_FILE_POINTER: DWORD = 0xFFFF_FFFF;
// regstr.h:483 `REGSTR_PATH_SYSTEMENUM TEXT("System\\CurrentControlSet\\Enum")`, UNICODE.
pub const REGSTR_PATH_SYSTEMENUM: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0043, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x0053, 0x0065, 0x0074, 0x005c, 0x0045, 0x006e, 0x0075, 0x006d, 0x0000u16]).as_ptr(); // L"System\\CurrentControlSet\\Enum"

// wine/debug.h:156 `int ros_dbg_log(enum __wine_debug_class cls, ...)`. Hand-kept: resolve's
// tree consensus picked winegdi's `cls: c_uint`, which no longer accepts the `c_int`
// __wine_debug_class every TRACE/ERR expansion here passes.
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        format: *const c_char,
        ...
    ) -> c_int;
}
// setupapi_private.h's static inline, translated once (port headers).
pub use crate::setupapi::setupapi_private_h::strdupAtoW;
// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const _O_RDONLY: i32 = 0; // clang, setupcab.c
pub const _O_RDWR: i32 = 2; // clang, setupcab.c
pub const _O_WRONLY: i32 = 1; // clang, setupcab.c
pub const _SH_COMPAT: i32 = 0; // clang, setupcab.c
pub const _SH_DENYNO: i32 = 0x40; // clang, setupcab.c
pub const _SH_DENYRD: i32 = 0x30; // clang, setupcab.c
pub const _SH_DENYRW: i32 = 0x10; // clang, setupcab.c
pub const _SH_DENYWR: i32 = 0x20; // clang, setupcab.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const _O_BINARY: u32 = 32768; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_CREAT: u32 = 256; // cabview-rust/src/prelude.rs
pub const _O_EXCL: u32 = 1024; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_NOINHERIT: u32 = 128; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_TRUNC: u32 = 512; // msvcrt-sys/src/bindgen_amd64.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const fdintCABINET_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCABINET_INFO; // island enum
pub const fdintCLOSE_FILE_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCLOSE_FILE_INFO; // island enum
pub const fdintCOPY_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCOPY_FILE; // island enum
pub const fdintNEXT_CABINET: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintNEXT_CABINET; // island enum
pub const fdintPARTIAL_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintPARTIAL_FILE; // island enum
extern "C" {
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub static GUID_NULL: IID; // evr-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDiskFreeSpaceW( arg1: LPCWSTR, arg2: PDWORD, arg3: PDWORD, arg4: PDWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDriveTypeW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameA(arg1: LPCSTR, arg2: DWORD, arg3: LPSTR, arg4: *mut LPSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLogicalDriveStringsW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn OpenEventW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn UuidFromStringW(StringUuid: *mut ::core::ffi::c_ushort, Uuid: *mut UUID) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, devclass.c
pub type HIMAGELIST = *mut _IMAGELIST;
// RPC_WSTR: c2rust, misc.c
pub type RPC_WSTR = *mut ::core::ffi::c_ushort;
// field: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct field {
    pub text: *const WCHAR,
}
// inf_file: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct inf_file {
    pub next: *mut inf_file,
    pub strings: *mut WCHAR,
    pub string_pos: *mut WCHAR,
    pub nb_sections: ::core::ffi::c_uint,
    pub alloc_sections: ::core::ffi::c_uint,
    pub sections: *mut *mut section,
    pub nb_fields: ::core::ffi::c_uint,
    pub alloc_fields: ::core::ffi::c_uint,
    pub fields: *mut field,
    pub strings_section: ::core::ffi::c_int,
    pub filename: *mut WCHAR,
}
// line: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct line {
    pub first_field: ::core::ffi::c_int,
    pub nb_fields: ::core::ffi::c_int,
    pub key_field: ::core::ffi::c_int,
}
// section: c2rust, parser.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct section {
    pub name: *const WCHAR,
    pub nb_lines: ::core::ffi::c_uint,
    pub alloc_lines: ::core::ffi::c_uint,
    pub lines: [line; 16],
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // devinst.c
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // devprop.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // setupcab.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // setupcab.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // cfgmgr.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // devinst.c
}
extern "system" {
    pub fn CreateMutexW(_: LPSECURITY_ATTRIBUTES, _: BOOL, _: LPCWSTR) -> HANDLE; // cfgmgr.c
    pub fn UuidToStringW(Uuid: *mut UUID, StringUuid: *mut RPC_WSTR) -> RPC_STATUS; // cfgmgr.c
}
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use win32macros_rust::WINE_ERR as ERR; // sdk alias
pub use win32macros_rust::WINE_FIXME as FIXME; // sdk alias
pub use win32macros_rust::WINE_TRACE as TRACE; // sdk alias
pub use win32macros_rust::WINE_WARN as WARN; // sdk alias
pub use crate::prelude::_snwprintf as snprintfW; // sdk alias
pub use crate::prelude::_swprintf as sprintfW; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
pub static mut __wine_dbch_setupapi: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x65\x74\x75\x70\x61\x70\x69\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_setupapi };
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

// winreg.h: `((HKEY)(ULONG_PTR)((LONG)0x80000002))` -- the LONG cast sign-extends, so on
// 64-bit the handle is 0xFFFFFFFF80000002, not 0x80000002.
pub const HKEY_LOCAL_MACHINE: HKEY = (0x80000002u32 as LONG) as LONG_PTR as HKEY;
// winbase.h: `((DWORD)0xFFFFFFFF)`.
pub const WAIT_FAILED: DWORD = 0xFFFFFFFF;

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::setupapi::cfgmgr::__WINE_DBCL_TRACE;
// ==== end cross-TU definitions ====
