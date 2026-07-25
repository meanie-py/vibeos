//! browser-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Grown during the cargo-check
//! grind: anything bindgen left out (CRT, inlines, sibling-C externs) is declared here once.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ========================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never out-of-line fns:
// an out-of-line no-op is an optimization barrier that uncloaks latent ABI mismatches into ud2
// (field manual §9.65). The WINE_-prefixed spellings are the same macros under their long names.
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};
#[macro_export] macro_rules! WINE_TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WINE_FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN};

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

// UNREFERENCED_PARAMETER / UNIMPLEMENTED are winnt.h/debug.h MACROS. Same rule as the debug
// channel above: a no-op must stay a macro, never an out-of-line fn (field manual §9.65).
#[macro_export] macro_rules! UNREFERENCED_PARAMETER { ($e:expr) => { let _ = &$e; }; }
#[macro_export] macro_rules! UNIMPLEMENTED { () => {{}}; ($($arg:tt)*) => {{}}; }
pub use crate::{UNIMPLEMENTED, UNREFERENCED_PARAMETER};

// ================= service control manager (psdk/winsvc.h) ======================================
// The ntdll_sys::nt island stops at the NT layer, so the winsvc.h surface this service needs is
// declared here. Every value below is transcribed from sdk/include/psdk/winsvc.h.

/// DECLARE_HANDLE(SERVICE_STATUS_HANDLE) — winsvc.h:195. Opaque handle, pointer-width.
pub type SERVICE_STATUS_HANDLE = *mut c_void;

/// winsvc.h:104 — `struct _SERVICE_STATUS`, seven DWORDs, no padding on any of our three targets.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}
impl SERVICE_STATUS {
    /// const zero-init — mirrors the C file-scope `static SERVICE_STATUS ServiceStatus;`, which
    /// lands in .bss zeroed. `Default::default()` is not const, so a static needs this.
    pub const fn zeroed() -> Self {
        Self {
            dwServiceType: 0,
            dwCurrentState: 0,
            dwControlsAccepted: 0,
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        }
    }
}
pub type LPSERVICE_STATUS = *mut SERVICE_STATUS;

// Service states — winsvc.h:21-27.
pub const SERVICE_STOPPED: DWORD = 1;
pub const SERVICE_START_PENDING: DWORD = 2;
pub const SERVICE_STOP_PENDING: DWORD = 3;
pub const SERVICE_RUNNING: DWORD = 4;
pub const SERVICE_CONTINUE_PENDING: DWORD = 5;
pub const SERVICE_PAUSE_PENDING: DWORD = 6;
pub const SERVICE_PAUSED: DWORD = 7;

// Accepted-control bits — winsvc.h:28,30.
pub const SERVICE_ACCEPT_STOP: DWORD = 1;
pub const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4;

// Control codes — winsvc.h:42-46. These MUST exist as consts: an unresolved uppercase identifier
// in a `match` arm is a catch-all BINDING, not a comparison, and rustc accepts it silently under
// this crate's `allow(non_snake_case)`. See the note in browser/browser.rs.
pub const SERVICE_CONTROL_STOP: DWORD = 1;
pub const SERVICE_CONTROL_PAUSE: DWORD = 2;
pub const SERVICE_CONTROL_CONTINUE: DWORD = 3;
pub const SERVICE_CONTROL_INTERROGATE: DWORD = 4;
pub const SERVICE_CONTROL_SHUTDOWN: DWORD = 5;

extern "system" {
    /// winsvc.h:491 — `BOOL WINAPI SetServiceStatus(SERVICE_STATUS_HANDLE, LPSERVICE_STATUS)`
    pub fn SetServiceStatus(hServiceStatus: SERVICE_STATUS_HANDLE, lpServiceStatus: LPSERVICE_STATUS) -> BOOL;
    /// winsvc.h:489 — the handler is `LPHANDLER_FUNCTION_EX` (winsvc.h:197), a WINAPI fn ptr.
    /// Spelled out rather than aliased so a plain `Some(handler)` typechecks without a transmute.
    pub fn RegisterServiceCtrlHandlerExW(
        lpServiceName: LPCWSTR,
        lpHandlerProc: Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD>,
        lpContext: LPVOID,
    ) -> SERVICE_STATUS_HANDLE;
}

// ================= RPC runtime (psdk/rpc.h, psdk/rpcdce.h) ======================================
// RPC_ENTRY is __stdcall (rpc.h:63), i.e. exactly `extern "system"` on all three targets.

/// rpc.h:48 — `typedef long RPC_STATUS` (LONG is 32-bit on every Windows target).
pub type RPC_STATUS = c_long;
/// rpcdce.h:122
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: c_uint = 1234;

extern "system" {
    /// rpcdce.h:480 — RPC_WSTR is `unsigned short*` (rpcdce.h:46).
    pub fn RpcServerUseProtseqEpW(
        Protseq: *mut u16,
        MaxCalls: c_uint,
        Endpoint: *mut u16,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
    /// rpcdce.h:449 — MgrTypeUuid is `UUID*`, MgrEpv is `RPC_MGR_EPV*`; both are only ever
    /// passed NULL here, so they stay opaque rather than dragging in the MIDL EPV layout.
    pub fn RpcServerRegisterIf(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut c_void,
        MgrEpv: *mut c_void,
    ) -> RPC_STATUS;
    /// rpcdce.h:409
    pub fn RpcServerListen(MinimumCallThreads: c_uint, MaxCalls: c_uint, DontWait: c_uint) -> RPC_STATUS;
    /// rpcdce.h:427
    pub fn RpcMgmtStopServerListening(Binding: RPC_BINDING_HANDLE) -> RPC_STATUS;
}

// ================= kernel32 (winbase.h / heapapi.h / processthreadsapi.h) =======================
extern "system" {
    pub fn GetLastError() -> DWORD;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    /// LPTHREAD_START_ROUTINE spelled out (same reason as the SCM handler above): `Some(f)` on a
    /// plain `extern "system"` fn item then typechecks directly, with no transmute at the site.
    pub fn CreateThread(
        lpThreadAttributes: *mut c_void,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(*mut c_void) -> DWORD>,
        lpParameter: *mut c_void,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
}

// Defined in the sibling module; browser.rs hands it to CreateThread.
pub use crate::browser::rpcserver::RpcThreadRoutine;
