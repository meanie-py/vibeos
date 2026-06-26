//! advapi32's own exported APIs that translated files call cross-file but whose
//! owning .c is not yet flipped to Rust. Declared `extern "system"` here →
//! resolve against the still-C cohab TUs at link.
//!
//! INVARIANT: when a .c flips to Rust, REMOVE its functions from this list — the
//! crate's `#[no_mangle]` definition then provides the symbol, and a lingering
//! extern decl in scope would conflict with the definition (Rust name clash).
#![allow(non_snake_case, dead_code)]

use core::ffi::{c_int, c_uint};
use crate::prelude::{
    PREGISTRY_SERVER_NAME, PRPC_UNICODE_STRING, PTRUSTEE_A, PTRUSTEE_W, RPC_STATUS, REGSAM,
    TRUSTEE_FORM, TRUSTEE_TYPE, MD4_CTX, MD5_CTX,
    LPSERVICE_STATUS, RPC_SERVICE_STATUS_HANDLE, SE_OBJECT_TYPE, PFN_OBJECT_MGR_FUNCTS,
    PINHERITED_FROMW, PEXPLICIT_ACCESS_W,
};
use ntdll_sys::nt::*;

// advapi32's own exports defined in not-yet-flipped C TUs (WINAPI → stdcall).
extern "system" {
    // security.c (Wave 4)
    pub fn DuplicateTokenEx( ExistingTokenHandle: HANDLE, dwDesiredAccess: ULONG, lpTokenAttributes: PVOID, ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL, TokenType: TOKEN_TYPE, DuplicateTokenHandle: PHANDLE, ) -> BOOL;
}

// Imported from sibling DLLs (kernel32 / rpcrt4) — WINAPI/RPC_ENTRY = stdcall.
extern "system" {
    pub fn SetLastError(dwErrCode: ULONG);
    pub fn GetLastError() -> ULONG;
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> RPC_STATUS;
    pub fn RpcStringBindingComposeW( ObjUuid: *mut u16, ProtSeq: *mut u16, NetworkAddr: *mut u16, Endpoint: *mut u16, Options: *mut u16, StringBinding: *mut *mut u16, ) -> RPC_STATUS;
    pub fn RpcBindingFromStringBindingW( StringBinding: *mut u16, Binding: *mut *mut core::ffi::c_void, ) -> RPC_STATUS;
    pub fn RpcBindingFree(Binding: *mut *mut core::ffi::c_void) -> RPC_STATUS;
    pub fn UuidCreate(Uuid: *mut crate::prelude::UUID) -> LONG;
    // advapi32's own reg exports (reg/reg.c, Wave 4) — WINAPI = stdcall.
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegOpenKeyExW( hKey: HKEY, lpSubKey: *const u16, ulOptions: DWORD, samDesired: DWORD, phkResult: *mut HKEY, ) -> LONG;
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: *const u16, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD, ) -> LONG;
    // RegSetValueExW takes `const BYTE* lpData` (winreg.h) — input, so *const u8.
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: *const u16, Reserved: DWORD, dwType: DWORD, lpData: *const u8, cbData: DWORD, ) -> LONG;
    // winreg_c.c RPC client stubs (generated C, never flipped) — __stdcall.
    pub fn BaseInitiateSystemShutdown( ServerName: PREGISTRY_SERVER_NAME, lpMessage: PRPC_UNICODE_STRING, dwTimeout: ULONG, bForceAppsClosed: BOOLEAN, bRebootAfterShutdown: BOOLEAN, ) -> ULONG;
    pub fn BaseInitiateSystemShutdownEx( ServerName: PREGISTRY_SERVER_NAME, lpMessage: PRPC_UNICODE_STRING, dwTimeout: ULONG, bForceAppsClosed: BOOLEAN, bRebootAfterShutdown: BOOLEAN, dwReason: ULONG, ) -> ULONG;
    pub fn BaseAbortSystemShutdown(ServerName: PREGISTRY_SERVER_NAME) -> ULONG;
    // kernel32 base APIs (WINAPI = stdcall).
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID, dwBytes: SIZE_T) -> PVOID;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn WriteFile(
        hFile: HANDLE,
        lpBuffer: LPCVOID,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: LPDWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
    // advapi32 reg exports (reg/reg.c, Wave 4) — WINAPI = stdcall.
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        Reserved: DWORD,
        lpClass: *mut u16,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: *mut _SECURITY_ATTRIBUTES,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut DWORD,
    ) -> LONG;
    pub fn RegDeleteKeyExW(hKey: HKEY, lpSubKey: *const u16, samDesired: REGSAM, Reserved: DWORD) -> LONG;
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: *const u16) -> LONG;
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: *mut u16,
        lpcchName: *mut DWORD,
        lpReserved: *mut DWORD,
        lpClass: *mut u16,
        lpcchClass: *mut DWORD,
        lpftLastWriteTime: *mut FILETIME,
    ) -> LONG;
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: *mut u16,
        lpcchValueName: *mut DWORD,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: LPBYTE,
        lpcbData: *mut DWORD,
    ) -> LONG;
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: *mut u16,
        lpcchClass: *mut DWORD,
        lpReserved: *mut DWORD,
        lpcSubKeys: *mut DWORD,
        lpcbMaxSubKeyLen: *mut DWORD,
        lpcbMaxClassLen: *mut DWORD,
        lpcValues: *mut DWORD,
        lpcbMaxValueNameLen: *mut DWORD,
        lpcbMaxValueLen: *mut DWORD,
        lpcbSecurityDescriptor: *mut DWORD,
        lpftLastWriteTime: *mut FILETIME,
    ) -> LONG;
    // kernel32 base APIs — service/sctrl.c (Wave 2). WINAPI = stdcall.
    pub fn WaitNamedPipeW(lpNamedPipeName: LPCWSTR, nTimeOut: DWORD) -> BOOL;
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn SetNamedPipeHandleState(
        hNamedPipe: HANDLE,
        lpMode: LPDWORD,
        lpMaxCollectionCount: LPDWORD,
        lpCollectDataTimeout: LPDWORD,
    ) -> BOOL;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: LPDWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: LPVOID,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    pub fn lstrlenW(lpString: LPCWSTR) -> c_int;
    // wine/security.c trustee accessors (Wave 4) — @ stdcall.
    pub fn GetTrusteeNameA(pTrustee: PTRUSTEE_A) -> LPSTR;
    pub fn GetTrusteeNameW(pTrustee: PTRUSTEE_W) -> LPWSTR;
    pub fn GetTrusteeFormA(pTrustee: PTRUSTEE_A) -> TRUSTEE_FORM;
    pub fn GetTrusteeFormW(pTrustee: PTRUSTEE_W) -> TRUSTEE_FORM;
    pub fn GetTrusteeTypeA(pTrustee: PTRUSTEE_A) -> TRUSTEE_TYPE;
    pub fn GetTrusteeTypeW(pTrustee: PTRUSTEE_W) -> TRUSTEE_TYPE;
}

// svcctl RPC client stubs (generated svcctl_c.c, never flipped) — __stdcall.
extern "system" {
    pub fn RSetServiceStatus(
        hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
        lpServiceStatus: LPSERVICE_STATUS,
    ) -> DWORD;
    pub fn RI_ScSetServiceBitsW(
        hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
        dwServiceBits: DWORD,
        bSetBitsOn: BOOL,
        bUpdateImmediately: BOOL,
        lpString: LPWSTR,
    ) -> DWORD;
    pub fn RI_ScSetServiceBitsA(
        hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
        dwServiceBits: DWORD,
        bSetBitsOn: BOOL,
        bUpdateImmediately: BOOL,
        lpString: LPSTR,
    ) -> DWORD;
}

// ntmarta.dll exports — sec/ac.c calls AccGetInheritanceSource /
// AccRewriteSetEntriesInAcl, which advapi32.h #defines to NtMartaStatic.* function
// pointers loaded via GetProcAddress (owning sec/misc.c is Wave 3). Declared here
// as direct imports; @ stdcall (ntmarta.spec) → extern "system".
// NOTE: link will need ntmarta resolution (import lib or the misc.c GetProcAddress
// table) — flag for the link gate.
extern "system" {
    pub fn AccGetInheritanceSource(
        pObjectName: LPWSTR,
        ObjectType: SE_OBJECT_TYPE,
        SecurityInfo: SECURITY_INFORMATION,
        Container: BOOL,
        pObjectClassGuids: *mut *mut GUID,
        GuidCount: DWORD,
        pAcl: PACL,
        pfnArray: PFN_OBJECT_MGR_FUNCTS,
        pGenericMapping: PGENERIC_MAPPING,
        pInheritArray: PINHERITED_FROMW,
    ) -> DWORD;
    pub fn AccRewriteSetEntriesInAcl(
        cCountOfExplicitEntries: ULONG,
        pListOfExplicitEntries: PEXPLICIT_ACCESS_W,
        OldAcl: PACL,
        NewAcl: *mut PACL,
    ) -> DWORD;
}

// cryptlib MD4/MD5 (NTAPI: i386 stdcall, amd64/arm64 cdecl → extern "system").
extern "system" {
    pub fn MD4Init(ctx: *mut MD4_CTX);
    pub fn MD4Update(ctx: *mut MD4_CTX, buf: *const u8, len: c_uint);
    pub fn MD4Final(ctx: *mut MD4_CTX);
    pub fn MD5Init(ctx: *mut MD5_CTX);
    pub fn MD5Update(ctx: *mut MD5_CTX, buf: *const u8, len: c_uint);
    pub fn MD5Final(ctx: *mut MD5_CTX);
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
