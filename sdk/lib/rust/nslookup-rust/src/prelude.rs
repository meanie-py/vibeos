//! nslookup-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AF_INET: u32 = 2; // finger-rust/src/prelude.rs
pub const INADDR_ANY: u_long = 0; // tcpsvcs-rust/src/prelude.rs
pub const IPPROTO_UDP: i32 = 17; // msafd-rust/src/glue.rs
pub const SOCK_DGRAM: u32 = 2; // iphlpapi-rust/src/types_amd64.rs
extern "C" {
    pub fn __iob_func() -> *mut FILE; // crt-startup-rust/src/misc.rs
    pub fn _stricmp(s1: *const i8, s2: *const i8) -> i32; // kernel32-rust/src/string/nls_subst.rs
    pub fn _strnicmp(a: *const c_char, b: *const c_char, n: usize) -> c_int; // crt-shared-rust/src/prelude.rs
    pub fn fgets(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char; // dbgprint_app-rust/src/prelude.rs
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32; // appwiz-rust/src/prelude.rs
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char; // comctl32-rust/src/prelude.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8; // appwiz-rust/src/prelude.rs
    pub fn strlen(s: *const c_char) -> usize; // arp-rust/src/prelude.rs
    pub fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int; // arp-rust/src/prelude.rs
    pub fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8; // cnt-rust/src/cabi/generated.rs
    pub fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> u32; // setuplib-rust/src/prelude.rs
}
extern "system" {
    pub fn GetNetworkParams(arg1: PFIXED_INFO, arg2: PULONG) -> DWORD; // tracert-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WSACleanup() -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn WSAGetLastError() -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn WSAStartup(wVersionRequired: WORD, lpWSAData: LPWSADATA) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn bind( s: SOCKET, addr: *const sockaddr, namelen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn closesocket(s: SOCKET) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn htonl(hostlong: ::core::ffi::c_ulong) -> u_long; // tracert-rust/src/types_i386.rs
    pub fn htons(hostshort: ::core::ffi::c_ushort) -> u_short; // tracert-rust/src/types_i386.rs
    pub fn inet_addr(cp: *const ::core::ffi::c_char) -> ::core::ffi::c_ulong; // tracert-rust/src/types_i386.rs
    pub fn ntohl(netlong: ::core::ffi::c_ulong) -> u_long; // tracert-rust/src/types_i386.rs
    pub fn ntohs(netshort: ::core::ffi::c_ushort) -> u_short; // tracert-rust/src/types_i386.rs
    pub fn recvfrom( s: SOCKET, buf: *mut ::core::ffi::c_char, len: ::core::ffi::c_int, flags: ::core::ffi::c_int, from: *mut sockaddr, fromlen: *mut ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn sendto( s: SOCKET, buf: *const ::core::ffi::c_char, len: ::core::ffi::c_int, flags: ::core::ffi::c_int, to: *const sockaddr, tolen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn socket( af: ::core::ffi::c_int, type_: ::core::ffi::c_int, protocol: ::core::ffi::c_int, ) -> SOCKET; // tracert-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const WSAEACCES: i32 = 0x271D; // clang, utility.c
pub const WSAEADDRNOTAVAIL: i32 = 0x2741; // clang, utility.c
pub const WSAEAFNOSUPPORT: i32 = 0x273F; // clang, utility.c
pub const WSAECONNABORTED: i32 = 0x2745; // clang, utility.c
pub const WSAECONNRESET: i32 = 0x2746; // clang, utility.c
pub const WSAEDESTADDRREQ: i32 = 0x2737; // clang, utility.c
pub const WSAEFAULT: i32 = 0x271E; // clang, utility.c
pub const WSAEHOSTUNREACH: i32 = 0x2751; // clang, utility.c
pub const WSAEINPROGRESS: i32 = 0x2734; // clang, utility.c
pub const WSAEINTR: i32 = 0x2714; // clang, utility.c
pub const WSAEINVAL: i32 = 0x2726; // clang, utility.c
pub const WSAEMSGSIZE: i32 = 0x2738; // clang, utility.c
pub const WSAENETDOWN: i32 = 0x2742; // clang, utility.c
pub const WSAENETRESET: i32 = 0x2744; // clang, utility.c
pub const WSAENETUNREACH: i32 = 0x2743; // clang, utility.c
pub const WSAENOBUFS: i32 = 0x2747; // clang, utility.c
pub const WSAENOTCONN: i32 = 0x2749; // clang, utility.c
pub const WSAENOTSOCK: i32 = 0x2736; // clang, utility.c
pub const WSAEOPNOTSUPP: i32 = 0x273D; // clang, utility.c
pub const WSAESHUTDOWN: i32 = 0x274A; // clang, utility.c
pub const WSAETIMEDOUT: i32 = 0x274C; // clang, utility.c
pub const WSAEWOULDBLOCK: i32 = 0x2733; // clang, utility.c
pub const WSANOTINITIALISED: i32 = 0x276D; // clang, utility.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// SOCKADDR: c2rust, utility.c
pub type SOCKADDR = sockaddr;
// SOCKADDR_IN: c2rust, utility.c
pub type SOCKADDR_IN = sockaddr_in;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn isspace(_C: ::core::ffi::c_int) -> ::core::ffi::c_int; // nslookup.c
    pub fn printf(_Format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int; // nslookup.c
}
pub use crate::prelude::isspace as _istspace; // sdk alias
pub use crate::prelude::printf as _tprintf; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! _T { ($p0:expr $(,)?) => { $p0 }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const INVALID_SOCKET: SOCKET = (((!0)) as SOCKET);
pub use crate::{_T, min};
// ==== end SDK header macros ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::nslookup::nslookup::State;
// ==== end cross-TU definitions ====
