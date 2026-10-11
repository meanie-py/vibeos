//! arp-rust prelude — the single import surface the translated modules use.
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
pub const EXIT_FAILURE: u32 = 1; // shutdown-rust/src/prelude.rs
pub const EXIT_SUCCESS: u32 = 0; // shutdown-rust/src/prelude.rs
pub const INADDR_NONE: u32 = 4294967295; // iphlpapi-rust/src/types_amd64.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // cacls-rust/src/prelude.rs
pub const MIB_IPNET_TYPE_DYNAMIC: MIB_IPNET_TYPE = MIB_IPNET_TYPE_MIB_IPNET_TYPE_DYNAMIC; // island enum
pub const MIB_IPNET_TYPE_INVALID: MIB_IPNET_TYPE = MIB_IPNET_TYPE_MIB_IPNET_TYPE_INVALID; // island enum
pub const MIB_IPNET_TYPE_OTHER: MIB_IPNET_TYPE = MIB_IPNET_TYPE_MIB_IPNET_TYPE_OTHER; // island enum
pub const MIB_IPNET_TYPE_STATIC: MIB_IPNET_TYPE = MIB_IPNET_TYPE_MIB_IPNET_TYPE_STATIC; // island enum
extern "C" {
    pub fn isdigit(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isxdigit(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn sscanf(s: *const i8, fmt: *const i8, ...) -> i32; // secur32-rust/src/externs.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8; // appwiz-rust/src/prelude.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int; // crt-shared-rust/src/prelude.rs
    pub fn toupper(c: i32) -> i32; // apphelp-rust/src/prelude.rs
}
extern "system" {
    pub fn DeleteIpNetEntry(arg1: PMIB_IPNETROW) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn FlushIpNetTable(arg1: DWORD) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn GetIpAddrTable(arg1: PMIB_IPADDRTABLE, arg2: PULONG, arg3: BOOL) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn GetIpNetTable(arg1: PMIB_IPNETTABLE, arg2: PULONG, arg3: BOOL) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn SetIpNetEntry(arg1: PMIB_IPNETROW) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn inet_addr(cp: *const ::core::ffi::c_char) -> ::core::ffi::c_ulong; // tracert-rust/src/types_i386.rs
    pub fn inet_ntoa(in_: in_addr) -> *mut ::core::ffi::c_char; // tracert-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MSG_ARP_BAD_ARGUMENT: DWORD = 0x2712; // clang, arp.c
pub const MSG_ARP_BAD_IP_ADDRESS: DWORD = 0x2711; // clang, arp.c
pub const MSG_ARP_DYNAMIC: DWORD = 0x2717; // clang, arp.c
pub const MSG_ARP_INTERFACE: DWORD = 0x2713; // clang, arp.c
pub const MSG_ARP_INVALID: DWORD = 0x2716; // clang, arp.c
pub const MSG_ARP_NO_ENTRIES: DWORD = 0x2722; // clang, arp.c
pub const MSG_ARP_NO_MEMORY: DWORD = 0x2714; // clang, arp.c
pub const MSG_ARP_OTHER: DWORD = 0x2715; // clang, arp.c
pub const MSG_ARP_STATIC: DWORD = 0x2718; // clang, arp.c
pub const MSG_ARP_SYNTAX: DWORD = 0x2710; // clang, arp.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn printf(_Format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int; // arp.c
    pub fn puts(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // arp.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // arp.c
}
extern "system" {
    pub fn FormatMessageA( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::FormatMessageA as FormatMessage; // sdk alias
pub use crate::prelude::puts as _putts; // sdk alias
pub use crate::prelude::sprintf as _stprintf; // sdk alias
pub use crate::prelude::printf as _tprintf; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! _T { ($p0:expr $(,)?) => { $p0 }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub use crate::{_T, va_end, va_start};
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn PrintMessageV(dwMessage: DWORD, ...);
}
// ==== end port variadic ====
