//! rpcss-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// Hand port, OUTSIDE the generated blocks: ntsecapi.h's `PLSA_UNICODE_STRING`, which the
// tree-consensus LsaAddAccountRights declaration names and no island here carries.
pub type PLSA_UNICODE_STRING = *mut LSA_UNICODE_STRING;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const POLICY_CREATE_ACCOUNT: u32 = 16; // advapi32-sys/src/bindgen_amd64.rs
pub const POLICY_LOOKUP_NAMES: u32 = 2048; // advapi32-sys/src/bindgen_amd64.rs
pub const REGCLS_MULTIPLEUSE: tagREGCLS = tagREGCLS_REGCLS_MULTIPLEUSE; // island enum
pub const REGCLS_MULTI_SEPARATE: tagREGCLS = tagREGCLS_REGCLS_MULTI_SEPARATE; // island enum
pub const ROTFLAGS_ALLOWANYCLIENT: u32 = 2; // cicero-rust/src/types_amd64.rs
pub const ROTFLAGS_REGISTRATIONKEEPSALIVE: u32 = 1; // cicero-rust/src/types_amd64.rs
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: u32 = 1234; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_PROTSEQ_MAX_REQS_DEFAULT: u32 = 10; // eventlog-rust/src/types_amd64.rs
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
pub const SECURITY_NT_AUTHORITY: i32 = 5; // imm32-rust/src/prelude.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
}
extern "system" {
    pub fn ChangeServiceConfigW( arg1: SC_HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: LPCWSTR, arg6: LPCWSTR, arg7: LPDWORD, arg8: LPCWSTR, arg9: LPCWSTR, arg10: LPCWSTR, arg11: LPCWSTR, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EqualSid(arg1: PSID, arg2: PSID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetTokenInformation( arg1: HANDLE, arg2: TOKEN_INFORMATION_CLASS, arg3: PVOID, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LsaAddAccountRights( arg1: LSA_HANDLE, arg2: PSID, arg3: PLSA_UNICODE_STRING, arg4: ULONG, ) -> NTSTATUS; // aclui-rust/src/types_i386.rs
    pub fn LsaClose(arg1: LSA_HANDLE) -> NTSTATUS; // aclui-rust/src/types_i386.rs
    pub fn LsaOpenPolicy( SystemName: PLSA_UNICODE_STRING, ObjectAttributes: PVOID, DesiredAccess: ACCESS_MASK, PolicyHandle: PHANDLE, ) -> NTSTATUS; // msgina-rust/src/msgina_externs.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterServiceCtrlHandlerExW( arg1: LPCWSTR, arg2: LPHANDLER_FUNCTION_EX, arg3: LPVOID, ) -> SERVICE_STATUS_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcServerListen( MinimumCallThreads: ::core::ffi::c_uint, MaxCalls: ::core::ffi::c_uint, DontWait: ::core::ffi::c_uint, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcServerRegisterIf( IfSpec: *mut ::core::ffi::c_void, MgrTypeUuid: *mut UUID, MgrEpv: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcServerUnregisterIf( IfSpec: *mut ::core::ffi::c_void, MgrTypeUuid: *mut UUID, WaitForCallsToComplete: ::core::ffi::c_uint, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn RpcServerUseProtseqEpW( Protseq: *mut ::core::ffi::c_ushort, MaxCalls: ::core::ffi::c_uint, Endpoint: *mut ::core::ffi::c_ushort, SecurityDescriptor: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetServiceStatus(arg1: SERVICE_STATUS_HANDLE, arg2: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn StartServiceCtrlDispatcherW(arg1: *const SERVICE_TABLE_ENTRYW) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// GUID: c2rust, epmp.c
pub type GUID = _GUID;
// LSA_HANDLE: c2rust, setup.c
pub type LSA_HANDLE = PVOID;
// LSA_OBJECT_ATTRIBUTES: c2rust, setup.c
pub type LSA_OBJECT_ATTRIBUTES = OBJECT_ATTRIBUTES;
// LSA_UNICODE_STRING: c2rust, setup.c
pub type LSA_UNICODE_STRING = UNICODE_STRING;
// UUID: c2rust, epmp.c
pub type UUID = GUID;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn I_RpcAllocate(Size: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // irotp.c
    pub fn I_RpcFree(Object: *mut ::core::ffi::c_void); // epmp.c
    pub fn TowerConstruct(
        object: *const RPC_SYNTAX_IDENTIFIER,
        syntax: *const RPC_SYNTAX_IDENTIFIER,
        protseq: *const ::core::ffi::c_char,
        endpoint: *const ::core::ffi::c_char,
        address: *const ::core::ffi::c_char,
        tower: *mut *mut twr_t,
    ) -> RPC_STATUS; // epmp.c
    pub fn TowerExplode(
        tower: *const twr_t,
        object: PRPC_SYNTAX_IDENTIFIER,
        syntax: PRPC_SYNTAX_IDENTIFIER,
        protseq: *mut *mut ::core::ffi::c_char,
        endpoint: *mut *mut ::core::ffi::c_char,
        address: *mut *mut ::core::ffi::c_char,
    ) -> RPC_STATUS; // epmp.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_ole: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6f\x6c\x65\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_ole };
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
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const MK_E_UNAVAILABLE: HRESULT = ((0x800401E3u32) as HRESULT);
pub const MK_S_MONIKERALREADYREGISTERED: HRESULT = ((0x000401E7) as HRESULT);
pub const SERVICES_ACTIVE_DATABASEW: *const u16 = (&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0000u16]).as_ptr(); // L"ServicesActive"
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/services/rpcss/rpcss_main.c:34: #define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#[allow(unused_macros)]
macro_rules! ARRAY_SIZE {
    ($x:expr $(,)?) => {
        (core::mem::size_of_val(&$x) / core::mem::size_of_val(&($x)[0]))
    };
}
#[allow(unused_imports)]
pub(crate) use ARRAY_SIZE;
// ==== end port macros ====
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};
