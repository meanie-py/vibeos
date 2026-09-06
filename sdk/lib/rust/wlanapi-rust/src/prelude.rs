//! wlanapi-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never
// out-of-line fns: an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).

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

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// wine/debug.h: `#define UNIMPLEMENTED WINE_FIXME("%s is UNIMPLEMENTED!\n", __FUNCTION__)`.
// A MACRO, not a fn — an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).
#[macro_export]
macro_rules! UNIMPLEMENTED {
    () => {{}};
}
pub use crate::UNIMPLEMENTED;

// rpcnterr.h: `#define RPC_S_OK ERROR_SUCCESS`. Deliberately NOT allowlisted into the island:
// that header defines the whole RPC_S_* range, which ntdll-sys already owns, and emitting a
// second copy would give those codes two identities. Aliasing the one name we use keeps a
// single identity and still avoids a magic number.
// Typed RPC_STATUS (i32), not DWORD: every use compares it against an RPC_STATUS return.
pub const RPC_S_OK: RPC_STATUS = ERROR_SUCCESS as RPC_STATUS;

// bindgen prefixes C enum members with their enum's tag, so the bare C spelling is absent.
// Alias rather than re-declare, so this cannot drift from the island's value.
pub const WLAN_SECURABLE_OBJECT_COUNT: _WLAN_SECURABLE_OBJECT =
    _WLAN_SECURABLE_OBJECT_WLAN_SECURABLE_OBJECT_COUNT;

// ================= Win32 imports (kernel32 / rpcrt4) ==========================================
// `extern "system"` — these are WINAPI/stdcall on i386; the island sets BLOCKLIST_FUNCS=1 so it
// carries types only. Matches wlanapi's add_importlibs(rpcrt4 msvcrt kernel32 ntdll).
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn RpcStringBindingComposeW(
        ObjUuid: *mut u16,
        ProtSeq: *mut u16,
        NetworkAddr: *mut u16,
        Endpoint: *mut u16,
        Options: *mut u16,
        StringBinding: *mut *mut u16,
    ) -> RPC_STATUS;
    pub fn RpcBindingFromStringBindingW(
        StringBinding: *mut u16,
        Binding: *mut RPC_BINDING_HANDLE,
    ) -> RPC_STATUS;
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> RPC_STATUS;
}

// ================= wlansvc MIDL client stubs (sibling C) ======================================
// Generated into wlansvc_c.c by add_rpc_files(client ... wlansvc.idl) and linked alongside this
// crate. The island sets BLOCKLIST_FUNCS=1 (it emits types, never prototypes), so the stubs are
// declared here once. `__cdecl` in the generated header == `extern "C"`.
extern "C" {
    pub fn _RpcCloseHandle(phClientHandle: *mut WLANSVC_RPC_HANDLE) -> DWORD;
    pub fn _RpcConnect(
        hClientHandle: WLANSVC_RPC_HANDLE,
        pInterfaceGuid: *const GUID,
        pConnectionParameters: *const PWLAN_CONNECTION_PARAMETERS,
    ) -> DWORD;
    pub fn _RpcDeleteProfile(
        hClientHandle: WLANSVC_RPC_HANDLE,
        pInterfaceGuid: *const GUID,
        strProfileName: *const u16,
    ) -> DWORD;
    pub fn _RpcDisconnect(
        hClientHandle: WLANSVC_RPC_HANDLE,
        pInterfaceGUID: *const GUID,
    ) -> DWORD;
    pub fn _RpcEnumInterfaces(
        hClientHandle: WLANSVC_RPC_HANDLE,
        ppInterfaceList: *mut PWLAN_INTERFACE_INFO_LIST,
    ) -> DWORD;
    pub fn _RpcOpenHandle(
        szMachineName: WLANSVC_HANDLE,
        dwClientVersion: DWORD,
        pdwNegotiatedVersion: *mut DWORD,
        phClientHandle: *mut WLANSVC_RPC_HANDLE,
    ) -> DWORD;
    pub fn _RpcRenameProfile(
        hClientHandle: WLANSVC_RPC_HANDLE,
        pInterfaceGuid: *const GUID,
        strOldProfileName: *const u16,
        strNewProfileName: *const u16,
    ) -> DWORD;
    pub fn _RpcScan(
        hClientHandle: WLANSVC_RPC_HANDLE,
        pInterfaceGuid: *const GUID,
        pDot11Ssid: PDOT11_SSID,
        pIeData: PWLAN_RAW_DATA,
    ) -> DWORD;
    pub fn _RpcSetSecuritySettings(
        hClientHandle: WLANSVC_RPC_HANDLE,
        SecurableObject: WLAN_SECURABLE_OBJECT,
        strModifiedSDDL: *const u16,
    ) -> DWORD;
}
