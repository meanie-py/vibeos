//! wlansvc-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Grown during the cargo-check
//! grind: anything bindgen left out (CRT, inlines, sibling-C externs) is declared here once.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ========================================================================
// The ROS DPRINT channel. NDEBUG is #defined in the C build, so these compile out. They MUST stay
// MACROS: an out-of-line no-op fn is an optimization barrier that uncloaks latent ABI mismatches
// into ud2 (field manual §9.65) — and the dhcpcsvc bug where one became `loop {}` hung the caller.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}
pub use crate::{DPRINT, DPRINT1};

extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// ================= CRT (cdecl) — msvcrt/ntdll exports ==========================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    // C-variadic: DECLARING and CALLING one is stable Rust; only *defining* one needs nightly.
    pub fn _snprintf(buf: *mut c_char, count: usize, format: *const c_char, ...) -> c_int;
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// UNREFERENCED_PARAMETER(x) is windef.h's `(void)(x)`. It MUST stay a macro: an out-of-line
// no-op fn is an optimization barrier that uncloaks latent ABI mismatches into ud2 (§9.65).
#[macro_export] macro_rules! UNREFERENCED_PARAMETER { ($p:expr) => { let _ = &$p; }; }
pub use crate::UNREFERENCED_PARAMETER;

// ================= sibling-module forward declarations ==========================================
// wlansvc.c forward-declares `DWORD WINAPI RpcThreadRoutine(LPVOID)`; it is defined in
// rpcserver.c, i.e. the sibling module here.
pub use crate::wlansvc::rpcserver::RpcThreadRoutine;

// ================= rpcdce.h / rpcdcep.h scalars =================================================
// The IDL island (types_*.rs) is generated from wlansvc.idl only, so it carries the *interface*
// types (WLANSVC_RPC_HANDLE, wlansvc_interface_v1_0_s_ifspec, RPC_IF_HANDLE) but none of the
// runtime's own scalars. Same declarations as wkssvc-rust/src/prelude.rs.
//   rpcdce.h:  typedef long RPC_STATUS;   typedef void* RPC_MGR_EPV;   typedef GUID UUID;
pub type RPC_STATUS = LONG;
pub type RPC_MGR_EPV = *mut c_void;
pub type UUID = GUID;
pub const RPC_S_OK: RPC_STATUS = 0;
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: c_uint = 1234;

// ================= winsvc.h =====================================================================
// precomp.h includes <winsvc.h>, but this crate's bindgen island is IDL-scoped and does not
// cover it (wkssvc-rust's island DOES — these declarations are copied verbatim from
// wkssvc-rust/src/types_{i386,amd64,arm64}.rs, which are byte-identical across all three arches
// for these types: SERVICE_STATUS is 7x DWORD, the rest are pointer-sized).
#[repr(C)]
pub struct _SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}
pub type SERVICE_STATUS = _SERVICE_STATUS;
pub type LPSERVICE_STATUS = *mut _SERVICE_STATUS;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SERVICE_STATUS_HANDLE__ {
    pub unused: c_int,
}
pub type SERVICE_STATUS_HANDLE = *mut SERVICE_STATUS_HANDLE__;

// WINAPI == stdcall on i386, cdecl elsewhere; `extern "system"` resolves to the right one per
// arch, so the whole crate spells service/thread callbacks that one way (no cfg split needed).
pub type LPSERVICE_MAIN_FUNCTIONW =
    Option<unsafe extern "system" fn(dwNumServicesArgs: DWORD, lpServiceArgVectors: *mut LPWSTR)>;

#[repr(C)]
pub struct _SERVICE_TABLE_ENTRYW {
    pub lpServiceName: LPWSTR,
    pub lpServiceProc: LPSERVICE_MAIN_FUNCTIONW,
}
pub type SERVICE_TABLE_ENTRYW = _SERVICE_TABLE_ENTRYW;
pub type LPSERVICE_TABLE_ENTRYW = *mut _SERVICE_TABLE_ENTRYW;

// winsvc.h dwCurrentState values
pub const SERVICE_STOPPED: DWORD = 1;
pub const SERVICE_START_PENDING: DWORD = 2;
pub const SERVICE_STOP_PENDING: DWORD = 3;
pub const SERVICE_RUNNING: DWORD = 4;
// winsvc.h dwControlsAccepted bits
pub const SERVICE_ACCEPT_STOP: DWORD = 1;
pub const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4;
// winsvc.h control codes. NOTE: SERVICE_CONTROL_INTERROGATE was previously undeclared, which
// silently turned `SERVICE_CONTROL_INTERROGATE =>` in ServiceControlHandler's match into an
// irrefutable catch-all BINDING (every control code returned NO_ERROR). See the report.
pub const SERVICE_CONTROL_STOP: DWORD = 1;
pub const SERVICE_CONTROL_INTERROGATE: DWORD = 4;
pub const SERVICE_CONTROL_SHUTDOWN: DWORD = 5;

// ================= advapi32 / kernel32 / rpcrt4 =================================================
// The island is generated with prototypes blocklisted (it must not re-declare what this crate
// #[no_mangle]-defines), so the genuine imports are declared here once. All are
// `extern "system"` = stdcall-on-i386 / cdecl elsewhere, which is what the import libs export.
extern "system" {
    // advapi32 — service control
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: LPCWSTR,
        lpHandlerProc: Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD>,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn SetServiceStatus(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        lpServiceStatus: LPSERVICE_STATUS,
    ) -> BOOL;
    pub fn StartServiceCtrlDispatcherW(lpServiceStartTable: *const SERVICE_TABLE_ENTRYW) -> BOOL;

    // kernel32
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    // DECLSPEC_NORETURN in the header, but declared returning () here on purpose: a Rust `!`
    // return teaches LLVM the call cannot return and it emits ud2 after it (§9.65 cascade).
    pub fn ExitThread(dwExitCode: DWORD);
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;

    // rpcrt4
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS;
    pub fn RpcServerListen(
        MinimumCallThreads: c_uint,
        MaxCalls: c_uint,
        DontWait: c_uint,
    ) -> RPC_STATUS;
    pub fn RpcServerRegisterIf(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut UUID,
        MgrEpv: RPC_MGR_EPV,
    ) -> RPC_STATUS;
    pub fn RpcServerUseProtseqEpW(
        Protseq: *mut c_ushort,
        MaxCalls: c_uint,
        Endpoint: *mut c_ushort,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
}
