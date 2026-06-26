//! advapi32's own exported APIs that translated files call cross-file but whose
//! owning .c is not yet flipped to Rust. Declared `extern "system"` here →
//! resolve against the still-C cohab TUs at link.
//!
//! INVARIANT: when a .c flips to Rust, REMOVE its functions from this list — the
//! crate's `#[no_mangle]` definition then provides the symbol, and a lingering
//! extern decl in scope would conflict with the definition (Rust name clash).
#![allow(non_snake_case, dead_code)]

use crate::prelude::{PREGISTRY_SERVER_NAME, PRPC_UNICODE_STRING, RPC_STATUS};
use ntdll_sys::nt::*;

// advapi32's own exports defined in not-yet-flipped C TUs (WINAPI → stdcall).
extern "system" {
    // security.c (Wave 4)
    pub fn DuplicateTokenEx(
        ExistingTokenHandle: HANDLE,
        dwDesiredAccess: ULONG,
        lpTokenAttributes: PVOID,
        ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL,
        TokenType: TOKEN_TYPE,
        DuplicateTokenHandle: PHANDLE,
    ) -> BOOL;
}

// Imported from sibling DLLs (kernel32 / rpcrt4) — WINAPI/RPC_ENTRY = stdcall.
extern "system" {
    pub fn SetLastError(dwErrCode: ULONG);
    pub fn GetLastError() -> ULONG;
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> RPC_STATUS;
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
        Binding: *mut *mut core::ffi::c_void,
    ) -> RPC_STATUS;
    pub fn RpcBindingFree(Binding: *mut *mut core::ffi::c_void) -> RPC_STATUS;
    pub fn UuidCreate(Uuid: *mut crate::prelude::UUID) -> LONG;
    // advapi32's own reg exports (reg/reg.c, Wave 4) — WINAPI = stdcall.
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        ulOptions: DWORD,
        samDesired: DWORD,
        phkResult: *mut HKEY,
    ) -> LONG;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: LPBYTE,
        lpcbData: *mut DWORD,
    ) -> LONG;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: LPBYTE,
        cbData: DWORD,
    ) -> LONG;
    // winreg_c.c RPC client stubs (generated C, never flipped) — __stdcall.
    pub fn BaseInitiateSystemShutdown(
        ServerName: PREGISTRY_SERVER_NAME,
        lpMessage: PRPC_UNICODE_STRING,
        dwTimeout: ULONG,
        bForceAppsClosed: BOOLEAN,
        bRebootAfterShutdown: BOOLEAN,
    ) -> ULONG;
    pub fn BaseInitiateSystemShutdownEx(
        ServerName: PREGISTRY_SERVER_NAME,
        lpMessage: PRPC_UNICODE_STRING,
        dwTimeout: ULONG,
        bForceAppsClosed: BOOLEAN,
        bRebootAfterShutdown: BOOLEAN,
        dwReason: ULONG,
    ) -> ULONG;
    pub fn BaseAbortSystemShutdown(ServerName: PREGISTRY_SERVER_NAME) -> ULONG;
}

// advapi32-internal cdecl helpers in not-yet-flipped C TUs (plain C = cdecl).
extern "C" {
    // wine/crypt.c (Wave 3)
    pub fn CRYPT_DEShash(dst: *mut u8, key: *const u8, src: *const u8) -> *mut u8;
    pub fn CRYPT_DESunhash(dst: *mut u8, key: *const u8, src: *const u8) -> *mut u8;
    // sec/misc.c (Wave 3)
    pub fn UnloadNtMarta();
    // reg/reg.c (Wave 4)
    pub fn RegInitialize() -> BOOL;
    pub fn RegCleanup() -> BOOL;
    // misc/sysfunc.c (Wave 2)
    pub fn CloseKsecDdHandle();
}
