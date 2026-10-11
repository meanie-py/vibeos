//! svchost-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // console-rust/src/prelude.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // diskpart-rust/src/prelude.rs
pub const LCMAP_LOWERCASE: u32 = 256; // appwiz-rust/src/types_amd64.rs
pub const RPC_S_OK: RPC_STATUS = 0; // advapi32-rust/src/prelude.rs
extern "C" {
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn ActivateActCtx(arg1: HANDLE, arg2: *mut ULONG_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateActCtxW(arg1: PCACTCTXW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn DeactivateActCtx(arg1: DWORD, arg2: ULONG_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DebugBreak(); // aclui-rust/src/types_amd64.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn ExitProcess(arg1: UINT) -> !; // aclui-rust/src/types_amd64.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn I_RpcMapWin32Status(Status: i32) -> NTSTATUS; // lsasrv-rust/src/lsasrv/lsarpc.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn LCMapStringW( Locale: LCID, dwMapFlags: DWORD, lpSrcStr: LPCWSTR, cchSrc: ::core::ffi::c_int, lpDestStr: LPWSTR, cchDest: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn Netbios(ncb: PNCB) -> UCHAR; // netapi32-rust/src/netapi32/netbios.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegisterServiceCtrlHandlerW( arg1: LPCWSTR, arg2: LPHANDLER_FUNCTION, ) -> SERVICE_STATUS_HANDLE; // syssetup-rust/src/types_amd64.rs
    pub fn RegisterWaitForSingleObject( arg1: PHANDLE, arg2: HANDLE, arg3: WAITORTIMERCALLBACK, arg4: PVOID, arg5: ULONG, arg6: ULONG, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseActCtx(arg1: HANDLE); // aclui-rust/src/types_amd64.rs
    pub fn RpcMgmtSetServerStackSize(ThreadStackSize: ULONG) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS; // browser-rust/src/prelude.rs
    pub fn RpcMgmtWaitServerListen() -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerListen( MinimumCallThreads: ::core::ffi::c_uint, MaxCalls: ::core::ffi::c_uint, DontWait: ::core::ffi::c_uint, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerRegisterIf( IfSpec: RPC_IF_HANDLE, MgrTypeUuid: *mut UUID, MgrEpv: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerUnregisterIf( IfSpec: RPC_IF_HANDLE, MgrTypeUuid: *mut UUID, WaitForCallsToComplete: ::core::ffi::c_uint, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerUnregisterIfEx( IfSpec: RPC_IF_HANDLE, MgrTypeUuid: *mut UUID, RundownContextHandles: ::core::ffi::c_int, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn RpcServerUseProtseqEpW( Protseq: RPC_WSTR, MaxCalls: ::core::ffi::c_uint, Endpoint: RPC_WSTR, SecurityDescriptor: *mut ::core::ffi::c_void, ) -> RPC_STATUS; // syssetup-rust/src/types_amd64.rs
    pub fn SetErrorMode(arg1: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn SetServiceStatus(arg1: SERVICE_STATUS_HANDLE, arg2: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn SetUnhandledExceptionFilter( arg1: LPTOP_LEVEL_EXCEPTION_FILTER, ) -> LPTOP_LEVEL_EXCEPTION_FILTER; // aclui-rust/src/types_amd64.rs
    pub fn StartServiceCtrlDispatcherW(arg1: *const SERVICE_TABLE_ENTRYW) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const EOAC_DISABLE_AAA: i32 = 0x1000; // clang, svchost.c
pub const EOAC_NO_CUSTOM_MARSHAL: i32 = 0x2000; // clang, svchost.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
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

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::svchost::rpcsrv::RpcpInitRpcServer;
pub use crate::svchost::rpcsrv::RpcpStartRpcServer;
pub use crate::svchost::rpcsrv::RpcpStopRpcServer;
pub use crate::svchost::rpcsrv::RpcpStopRpcServerEx;
pub use crate::svchost::netbios::SvcNetBiosClose;
pub use crate::svchost::netbios::SvcNetBiosInit;
pub use crate::svchost::netbios::SvcNetBiosOpen;
pub use crate::svchost::netbios::SvcNetBiosReset;
pub use crate::svchost::svchost::SvcRegisterStopCallback;
// ==== end cross-TU definitions ====

// ================= hand: what the generated blocks above cannot supply ==========================
// svchost.h's debug macros: `SvchostDbgPrint(lev, fmt, ...)` is DbgPrintEx(DPFLTR_SVCHOST_ID,
// DPFLTR_MASK | lev, "[SVCHOST] %lx.%lx: " fmt, pid, tid, ...) -- the prefix PASTED onto the format
// literal, so the call sites pass the format as a literal and concat! joins them at compile time.
macro_rules! SvchostDbgPrint {
    ($lev:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        DbgPrintEx(
            crate::types::_DPFLTR_TYPE_DPFLTR_SVCHOST_ID as ULONG,
            DPFLTR_MASK | $lev,
            concat!("[SVCHOST] %lx.%lx: ", $fmt, "\0").as_ptr() as PCSTR,
            GetCurrentProcessId(),
            GetCurrentThreadId()
            $(, $arg)*
        )
    };
}
macro_rules! DBG_ERR {
    ($fmt:literal $(, $arg:expr)* $(,)?) => { SvchostDbgPrint!(1, $fmt $(, $arg)*) };
}
macro_rules! DBG_TRACE {
    ($fmt:literal $(, $arg:expr)* $(,)?) => { SvchostDbgPrint!(4, $fmt $(, $arg)*) };
}
#[allow(unused_imports)]
pub(crate) use {DBG_ERR, DBG_TRACE, SvchostDbgPrint};

// security.cxx is C++ and stays C (linked alongside): svchost.h's prototype.
extern "system" {
    pub fn InitializeSecurity(dwParam: DWORD, dwAuthnLevel: DWORD, dwImpLevel: DWORD, dwCapabilities: DWORD) -> BOOL;
}
