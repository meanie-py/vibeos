//! wkssvc-rust prelude — the single import surface the translated modules use.
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

// ================= UNIMPLEMENTED ================================================================
// reactos/debug.h `UNIMPLEMENTED` is __NOTICE(WARNING, "is UNIMPLEMENTED!\n") — a DbgPrint that
// FALLS THROUGH (it neither blocks nor returns). A no-op MACRO, never an out-of-line no-op fn
// (field manual §9.65). The translation rendered these as `eprintln!("...stub")`, which is both
// std-only (this is no_std) and unfaithful — the C prints to the debug channel and continues.
#[macro_export]
macro_rules! UNIMPLEMENTED {
    ($($arg:tt)*) => {{}};
}
pub use crate::UNIMPLEMENTED;

// ================= wide string literals =========================================================
// `w!("x")` is the LPCWSTR literal the C wrote as L"x".
//
// The array is a `static`, NOT a `const`, and that distinction is load-bearing: a `const` is
// inlined at each use site, so `CONST.as_ptr()` would take the address of a temporary and hand
// the Win32 callee a dangling pointer. A `static` has one stable address for the program's life.
// ASCII-only by construction — a non-ASCII literal fails at COMPILE time rather than silently
// truncating, since every caller here passes registry paths and value names.
#[macro_export]
macro_rules! w {
    ($s:expr) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT.as_ptr()
    }};
}
pub use crate::w;

// ================= Win32 memory helpers =========================================================
// RtlZeroMemory/RtlCopyMemory. Real work (not no-ops), so #[inline(always)] fns are fine here —
// the §9.65 out-of-line-barrier hazard is specific to NO-OP debug shims. Both lower to the
// memset/memcpy INTRINSIC; never hand-roll a byte loop (LLVM folds one back into a call to
// itself → infinite self-recursion → silent hang, per AGENTS.md).
#[inline(always)]
pub unsafe fn ZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len)
}
#[inline(always)]
pub unsafe fn CopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len)
}

// ================= consts bindgen cannot emit ===================================================
// HKEY_LOCAL_MACHINE is a cast macro, not an enum/const, so bindgen skips it.
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const RPC_S_OK: RPC_STATUS = 0;
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: c_uint = 1234;
pub const MAX_PREFERRED_LENGTH: DWORD = 0xFFFF_FFFF;

// LSA_SUCCESS(x) is an ntsecapi.h inline predicate, not a symbol.
#[inline(always)]
pub fn LSA_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ================= advapi32 / kernel32 / rpcrt4 / secur32 =======================================
// BLOCKLIST_FUNCS=1 drops every prototype from the island on purpose (the island must not
// re-declare what the translation #[no_mangle]-defines), so the genuine imports are declared
// here once. All are `extern "system"` = stdcall-on-i386 / cdecl elsewhere, which is what the
// import libraries export.
extern "system" {
    // advapi32 — registry
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LSTATUS;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS;

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
    pub fn SetServiceBits(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        dwServiceBits: DWORD,
        bSetBitsOn: BOOL,
        bUpdateImmediately: BOOL,
    ) -> BOOL;

    // kernel32
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(*mut c_void) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn GetVersionExW(lpVersionInformation: *mut OSVERSIONINFOW) -> BOOL;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;

    // rpcrt4
    pub fn RpcImpersonateClient(BindingHandle: RPC_BINDING_HANDLE) -> RPC_STATUS;
    pub fn RpcRevertToSelf() -> RPC_STATUS;
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

    // secur32 / advapi32 — LSA
    pub fn LsaOpenPolicy(
        SystemName: PLSA_UNICODE_STRING,
        ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: PLSA_HANDLE,
    ) -> NTSTATUS;
    pub fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS;
    pub fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaNtStatusToWinError(Status: NTSTATUS) -> ULONG;
    pub fn LsaQueryInformationPolicy(
        PolicyHandle: LSA_HANDLE,
        InformationClass: POLICY_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn LsaSetInformationPolicy(
        PolicyHandle: LSA_HANDLE,
        InformationClass: POLICY_INFORMATION_CLASS,
        Buffer: PVOID,
    ) -> NTSTATUS;
    pub fn LsaRegisterLogonProcess(
        LogonProcessName: PLSA_STRING,
        LsaHandle: PHANDLE,
        SecurityMode: PLSA_OPERATIONAL_MODE,
    ) -> NTSTATUS;
    pub fn LsaDeregisterLogonProcess(LsaHandle: HANDLE) -> NTSTATUS;
    pub fn LsaLookupAuthenticationPackage(
        LsaHandle: HANDLE,
        PackageName: PLSA_STRING,
        AuthenticationPackage: PULONG,
    ) -> NTSTATUS;
    pub fn LsaCallAuthenticationPackage(
        LsaHandle: HANDLE,
        AuthenticationPackage: ULONG,
        ProtocolSubmitBuffer: PVOID,
        SubmitBufferLength: ULONG,
        ProtocolReturnBuffer: *mut PVOID,
        ReturnBufferLength: PULONG,
        ProtocolStatus: PNTSTATUS,
    ) -> NTSTATUS;
    pub fn LsaFreeReturnBuffer(Buffer: PVOID) -> NTSTATUS;
}

// RtlGetProcessHeap is a MACRO, not an export: ndk/rtlfuncs.h:1127
//     #define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)
// Declaring it extern manufactured an undefined `_RtlGetProcessHeap@0` that only appeared at
// LINK. kernel32's GetProcessHeap() returns that very same handle, so this is an exact
// equivalent — matching the netapi32-rust/prelude_common.rs precedent.
#[inline(always)]
pub unsafe fn RtlGetProcessHeap() -> HANDLE {
    GetProcessHeap()
}

// ================= intra-crate surface ==========================================================
// The four service globals: DEFINED by the modules below (mirroring which .c owned each), and
// blocklisted out of the island so there is exactly ONE definition. Re-exported here so every
// module reaches them through the single prelude import, as the C reached them via precomp.h.
pub use crate::wkssvc::info::WkstaInfo502;
pub use crate::wkssvc::wkssvc::{LsaAuthenticationPackage, LsaHandle, VersionInfo};
// Cross-module functions the C declared in precomp.h.
pub use crate::wkssvc::domain::{NetpGetJoinInformation, NetpJoinWorkgroup};
pub use crate::wkssvc::info::{InitWorkstationInfo, SaveWorkstationInfo};
pub use crate::wkssvc::rpcserver::RpcThreadRoutine;

// ================= enum bare-name aliases =======================================================
// bindgen prefixes every enum member with its type name (`_POLICY_INFORMATION_CLASS_Policy...`);
// the C names the member bare. Alias the ones the translated bodies actually use.
pub use crate::types::{
    // lmjoin.h — NETSETUP_JOIN_STATUS
    _NETSETUP_JOIN_STATUS_NetSetupDomainName as NetSetupDomainName,
    _NETSETUP_JOIN_STATUS_NetSetupUnjoined as NetSetupUnjoined,
    _NETSETUP_JOIN_STATUS_NetSetupUnknownStatus as NetSetupUnknownStatus,
    _NETSETUP_JOIN_STATUS_NetSetupWorkgroupName as NetSetupWorkgroupName,
    // ntmsv1_0.h — MSV1_0_PROTOCOL_MESSAGE_TYPE
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0EnumerateUsers as MsV1_0EnumerateUsers,
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0GetUserInfo as MsV1_0GetUserInfo,
    // ntsecapi.h — POLICY_INFORMATION_CLASS
    _POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation as PolicyPrimaryDomainInformation,
};

// ================= rpcdce.h scalars =============================================================
// rpcdce.h is deliberately NOT allowlisted (it would drag the whole RPC runtime surface into the
// island for the handful of scalars this module needs). RPC_IF_HANDLE / RPC_BINDING_HANDLE DO
// come from the island via wkssvc_s.h; only these three are missing.
//   rpcdce.h:  typedef long RPC_STATUS;   typedef void* RPC_MGR_EPV;   typedef GUID UUID;
pub type RPC_STATUS = LONG;
pub type RPC_MGR_EPV = *mut c_void;
pub type UUID = GUID;

// The MIDL allocator hooks are APPLICATION-supplied: the C defines them in this module's own
// rpcserver.c and the generated _s.c marshaller calls them. They are NOT rpcrt4 imports —
// declaring them as such (under the capitalised `MIDL_user_allocate` link name) manufactured
// an undefined symbol that only surfaced at LINK, never at cargo check. Re-export the crate's
// own definitions so every module sees the one real implementation.
pub use crate::wkssvc::rpcserver::{midl_user_allocate, midl_user_free};
