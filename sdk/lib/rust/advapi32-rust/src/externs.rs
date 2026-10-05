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
    // Wave 3
    LSA_STRING, LSA_OPERATIONAL_MODE, SECURITY_LOGON_TYPE, QUOTA_LIMITS,
    FN_PROGRESS, PROG_INVOKE_SETTING, RPC_CSTR, handle_t,
    // unified Win32 handle type (advapi32-sys; overrides nt::* glob below)
    HMODULE,
};
use ntdll_sys::nt::*;

// advapi32's own exports defined in not-yet-flipped C TUs (WINAPI → stdcall).
extern "system" {
    // security.c (Wave 4)
    pub fn DuplicateTokenEx( ExistingTokenHandle: HANDLE, dwDesiredAccess: ULONG, lpTokenAttributes: PVOID, ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL, TokenType: TOKEN_TYPE, DuplicateTokenHandle: PHANDLE, ) -> BOOL;
}

// Imported from sibling DLLs (kernel32 / rpcrt4) — WINAPI/RPC_ENTRY = stdcall.
extern "system" {
    /// rpcrt4: the C's `RpcExcept` handlers map a caught RPC exception through it.
    pub fn I_RpcMapWin32Status(Status: RPC_STATUS) -> NTSTATUS;
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
    pub fn WriteFile( hFile: HANDLE, lpBuffer: LPCVOID, nNumberOfBytesToWrite: DWORD, lpNumberOfBytesWritten: LPDWORD, lpOverlapped: LPVOID, ) -> BOOL;
    pub fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: c_int, lpWideCharStr: LPWSTR, cchWideChar: c_int, ) -> c_int;
    pub fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: c_int, lpMultiByteStr: LPSTR, cbMultiByte: c_int, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL, ) -> c_int;
    // advapi32 reg exports (reg/reg.c, Wave 4) — WINAPI = stdcall.
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: *const u16, Reserved: DWORD, lpClass: *mut u16, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: *mut _SECURITY_ATTRIBUTES, phkResult: *mut HKEY, lpdwDisposition: *mut DWORD, ) -> LONG;
    pub fn RegDeleteKeyExW(hKey: HKEY, lpSubKey: *const u16, samDesired: REGSAM, Reserved: DWORD) -> LONG;
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: *const u16) -> LONG;
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: *mut u16, lpcchName: *mut DWORD, lpReserved: *mut DWORD, lpClass: *mut u16, lpcchClass: *mut DWORD, lpftLastWriteTime: *mut FILETIME, ) -> LONG;
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: *mut u16, lpcchValueName: *mut DWORD, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD, ) -> LONG;
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: *mut u16, lpcchClass: *mut DWORD, lpReserved: *mut DWORD, lpcSubKeys: *mut DWORD, lpcbMaxSubKeyLen: *mut DWORD, lpcbMaxClassLen: *mut DWORD, lpcValues: *mut DWORD, lpcbMaxValueNameLen: *mut DWORD, lpcbMaxValueLen: *mut DWORD, lpcbSecurityDescriptor: *mut DWORD, lpftLastWriteTime: *mut FILETIME, ) -> LONG;
    // kernel32 base APIs — service/sctrl.c (Wave 2). WINAPI = stdcall.
    pub fn WaitNamedPipeW(lpNamedPipeName: LPCWSTR, nTimeOut: DWORD) -> BOOL;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn CreateFileW( lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: LPVOID, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE, ) -> HANDLE;
    pub fn SetNamedPipeHandleState( hNamedPipe: HANDLE, lpMode: LPDWORD, lpMaxCollectionCount: LPDWORD, lpCollectDataTimeout: LPDWORD, ) -> BOOL;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn ReadFile( hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: DWORD, lpNumberOfBytesRead: LPDWORD, lpOverlapped: LPVOID, ) -> BOOL;
    pub fn CreateThread( lpThreadAttributes: LPVOID, dwStackSize: SIZE_T, lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>, lpParameter: LPVOID, dwCreationFlags: DWORD, lpThreadId: LPDWORD, ) -> HANDLE;
    pub fn lstrlenW(lpString: LPCWSTR) -> c_int;
    // kernel32 LocalAlloc/LocalFree (wine/crypt.rs CRYPT_Alloc/CRYPT_Free) — WINAPI = stdcall.
    pub fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    // wine/security.c trustee accessors (Wave 4) — @ stdcall.
    pub fn GetTrusteeNameA(pTrustee: PTRUSTEE_A) -> LPSTR;
    pub fn GetTrusteeNameW(pTrustee: PTRUSTEE_W) -> LPWSTR;
    pub fn GetTrusteeFormA(pTrustee: PTRUSTEE_A) -> TRUSTEE_FORM;
    pub fn GetTrusteeFormW(pTrustee: PTRUSTEE_W) -> TRUSTEE_FORM;
    pub fn GetTrusteeTypeA(pTrustee: PTRUSTEE_A) -> TRUSTEE_TYPE;
    pub fn GetTrusteeTypeW(pTrustee: PTRUSTEE_W) -> TRUSTEE_TYPE;
}

// svcctl RPC client stubs (generated svcctl_c.c, never flipped) — __stdcall on
// i386, cdecl on win64. Declarations + correct per-arch ABI come from advapi32-sys
// bindgen (allowlist-function R[A-Z].*|Secl.*); re-export by name. These resolve
// against the generated C at link. Wave 2 (sctrl.rs) used RSetServiceStatus /
// RI_ScSetServiceBits*; Wave 4 (scm.rs) adds the full SCM client surface +
// seclogon SeclCreateProcessWithLogonW.
pub use advapi32_sys::{
    RSetServiceStatus, RI_ScSetServiceBitsW, RI_ScSetServiceBitsA,
    RChangeServiceConfig2A, RChangeServiceConfig2W, RChangeServiceConfigA, RChangeServiceConfigW,
    RCloseServiceHandle, RControlService, RCreateServiceA, RCreateServiceW, RDeleteService,
    REnumDependentServicesA, REnumDependentServicesW, REnumServiceGroupW,
    REnumServicesStatusA, REnumServicesStatusW, REnumServicesStatusExA, REnumServicesStatusExW,
    RGetServiceDisplayNameA, RGetServiceDisplayNameW, RGetServiceKeyNameA, RGetServiceKeyNameW,
    RI_ScGetCurrentGroupStateW, RI_ScQueryServiceTagInfo, RI_ScSendPnPMessage, RI_ScValidatePnPService,
    RLockServiceDatabase, RNotifyBootConfigStatus, ROpenSCManagerA, ROpenSCManagerW,
    ROpenServiceA, ROpenServiceW, RQueryServiceConfig2A, RQueryServiceConfig2W,
    RQueryServiceConfigA, RQueryServiceConfigW, RQueryServiceLockStatusA, RQueryServiceLockStatusW,
    RQueryServiceObjectSecurity, RQueryServiceStatus, RQueryServiceStatusEx,
    RSetServiceObjectSecurity, RStartServiceA, RStartServiceW, RUnlockServiceDatabase,
    SeclCreateProcessWithLogonW,
};

// ntmarta.dll exports — sec/ac.c calls AccGetInheritanceSource /
// AccRewriteSetEntriesInAcl, which advapi32.h #defines to NtMartaStatic.* function
// pointers loaded via GetProcAddress (owning sec/misc.c is Wave 3). Declared here
// as direct imports; @ stdcall (ntmarta.spec) → extern "system".
// NOTE: link will need ntmarta resolution (import lib or the misc.c GetProcAddress
// table) — flag for the link gate.
extern "system" {
    pub fn AccGetInheritanceSource( pObjectName: LPWSTR, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, Container: BOOL, pObjectClassGuids: *mut *mut GUID, GuidCount: DWORD, pAcl: PACL, pfnArray: PFN_OBJECT_MGR_FUNCTS, pGenericMapping: PGENERIC_MAPPING, pInheritArray: PINHERITED_FROMW, ) -> DWORD;
    pub fn AccRewriteSetEntriesInAcl( cCountOfExplicitEntries: ULONG, pListOfExplicitEntries: PEXPLICIT_ACCESS_W, OldAcl: PACL, NewAcl: *mut PACL, ) -> DWORD;
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

// MIDL RPC allocators — defined in advapi32 service/rpc.c (not flipped). rpcndr.h
// does `#define midl_user_allocate MIDL_user_allocate`, so the actual exported
// symbol is the UPPERCASE name (__RPC_USER = __stdcall → extern "system"). The
// translation calls both spellings; lowercase forwards to the real symbol.
extern "system" {
    pub fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void;
    pub fn MIDL_user_free(ptr: *mut core::ffi::c_void);
}
#[inline]
pub unsafe fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    MIDL_user_allocate(len)
}
#[inline]
pub unsafe fn midl_user_free(ptr: *mut core::ffi::c_void) {
    MIDL_user_free(ptr)
}

// advapi32-internal cdecl helpers in not-yet-flipped C TUs (plain C = cdecl).
extern "C" {
    // wine/crypt.c (Wave 3)
    pub fn CRYPT_DEShash(dst: *mut core::ffi::c_uchar, key: *const core::ffi::c_uchar, src: *const core::ffi::c_uchar) -> *mut core::ffi::c_uchar;
    pub fn CRYPT_DESunhash(dst: *mut core::ffi::c_uchar, key: *const core::ffi::c_uchar, src: *const core::ffi::c_uchar) -> *mut core::ffi::c_uchar;
    // sec/misc.c (Wave 3)
    pub fn UnloadNtMarta();
    // reg/reg.c (Wave 4)
    pub fn RegInitialize() -> BOOL;
    pub fn RegCleanup() -> BOOL;
    // misc/sysfunc.c (Wave 2)
    pub fn CloseKsecDdHandle();
}

// ---- Wave 3: lsa_c.c / eventlogrpc_c.c RPC client stubs ----
// Generated MIDL client stubs (__stdcall on i386, cdecl on win64). Declarations
// + correct per-arch ABI come from advapi32-sys bindgen (allowlist-function
// Lsar.*|Elfr.*) — re-export by name. These resolve against the generated C at
// link (never flipped).
pub use advapi32_sys::{
    ElfrBackupELFA, ElfrBackupELFW, ElfrChangeNotify, ElfrClearELFA, ElfrClearELFW,
    ElfrCloseEL, ElfrDeregisterEventSource, ElfrFlushEL, ElfrGetLogInformation,
    ElfrOldestRecord, ElfrOpenBELA, ElfrOpenBELW, ElfrOpenELA, ElfrOpenELW,
    ElfrReadELA, ElfrReadELW, ElfrRegisterEventSourceA, ElfrRegisterEventSourceW,
    ElfrReportEventA, ElfrReportEventAndSourceW, ElfrReportEventW,
    LsarAddAccountRights, LsarAddPrivilegesToAccount, LsarClearAuditLog, LsarClose,
    LsarCreateAccount, LsarCreateSecret, LsarCreateTrustedDomain,
    LsarCreateTrustedDomainEx2, LsarDelete, LsarDeleteTrustedDomain,
    LsarEnumerateAccountRights, LsarEnumerateAccounts,
    LsarEnumerateAccountsWithUserRight, LsarEnumeratePrivileges,
    LsarEnumeratePrivilegesAccount, LsarEnumerateTrustedDomains,
    LsarEnumerateTrustedDomainsEx, LsarGetQuotasForAccount, LsarGetUserName,
    LsarGetSystemAccessAccount, LsarLookupNames, LsarLookupNames3,
    LsarLookupPrivilegeDisplayName, LsarLookupPrivilegeName,
    LsarLookupPrivilegeValue, LsarLookupSids, LsarOpenAccount, LsarOpenPolicy,
    LsarOpenPolicySce, LsarOpenSecret, LsarOpenTrustedDomain,
    LsarOpenTrustedDomainByName, LsarQueryDomainInformationPolicy,
    LsarQueryForestTrustInformation, LsarQueryInfoTrustedDomain,
    LsarQueryInformationPolicy, LsarQuerySecret, LsarQuerySecurityObject,
    LsarQueryTrustedDomainInfo, LsarQueryTrustedDomainInfoByName,
    LsarRemoveAccountRights, LsarRemovePrivilegesFromAccount,
    LsarRetrievePrivateData, LsarSetDomainInformationPolicy,
    LsarSetForestTrustInformation, LsarSetInformationPolicy,
    LsarSetQuotasForAccount, LsarSetSecret, LsarSetSecurityObject,
    LsarSetSystemAccessAccount, LsarStorePrivateData,
};

// ---- Wave 3: kernel32 / advapi32-own / secur32 / rpcrt4 / ntmarta imports ----
// All WINAPI / NTAPI / RPC_ENTRY = __stdcall on i386 → extern "system". The
// advapi32-own ones (token/sid/lsa exports) resolve to the crate's #[no_mangle]
// defs or the not-yet-flipped C; sibling-DLL ones link from their import libs.
extern "system" {
    // kernel32
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;
    pub fn ResumeThread(hThread: HANDLE) -> DWORD;
    pub fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn GetComputerNameA(lpBuffer: LPSTR, nSize: LPDWORD) -> BOOL;
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL;
    pub fn lstrcmpW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> c_int;
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    pub fn GetThreadLocale() -> DWORD;
    pub fn GetSystemDefaultUILanguage() -> u16;
    pub fn GetUserDefaultUILanguage() -> u16;
    pub fn CompareStringW(Locale: DWORD, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: c_int, lpString2: LPCWSTR, cchCount2: c_int) -> c_int;
    pub fn CreateProcessW( lpApplicationName: LPCWSTR, lpCommandLine: LPWSTR, lpProcessAttributes: LPSECURITY_ATTRIBUTES, lpThreadAttributes: LPSECURITY_ATTRIBUTES, bInheritHandles: BOOL, dwCreationFlags: DWORD, lpEnvironment: LPVOID, lpCurrentDirectory: LPCWSTR, lpStartupInfo: LPSTARTUPINFOW, lpProcessInformation: LPPROCESS_INFORMATION, ) -> BOOL;
    pub fn CreateProcessA( lpApplicationName: LPCSTR, lpCommandLine: LPSTR, lpProcessAttributes: LPSECURITY_ATTRIBUTES, lpThreadAttributes: LPSECURITY_ATTRIBUTES, bInheritHandles: BOOL, dwCreationFlags: DWORD, lpEnvironment: LPVOID, lpCurrentDirectory: LPCSTR, lpStartupInfo: LPSTARTUPINFOA, lpProcessInformation: LPPROCESS_INFORMATION, ) -> BOOL;
    // advapi32-own reg exports (reg/reg.c, Wave 4)
    pub fn RegOpenKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: *mut HKEY) -> LONG;
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LONG;
    pub fn RegDeleteValueW(hKey: HKEY, lpValueName: LPCWSTR) -> LONG;
    // advapi32-own security exports (token/sid/sec — not flipped or crate-defined)
    pub fn CopySid(nDestinationSidLength: DWORD, pDestinationSid: PSID, pSourceSid: PSID) -> BOOL;
    pub fn GetSidLengthRequired(nSubAuthorityCount: u8) -> DWORD;
    pub fn GetSidSubAuthority(pSid: PSID, nSubAuthority: DWORD) -> *mut DWORD;
    pub fn GetSidSubAuthorityCount(pSid: PSID) -> *mut u8;
    pub fn GetTokenInformation(TokenHandle: HANDLE, TokenInformationClass: TOKEN_INFORMATION_CLASS, TokenInformation: LPVOID, TokenInformationLength: DWORD, ReturnLength: *mut DWORD) -> BOOL;
    pub fn OpenProcessToken(ProcessHandle: HANDLE, DesiredAccess: DWORD, TokenHandle: *mut HANDLE) -> BOOL;
    pub fn OpenThreadToken(ThreadHandle: HANDLE, DesiredAccess: DWORD, OpenAsSelf: BOOL, TokenHandle: *mut HANDLE) -> BOOL;
    pub fn RevertToSelf() -> BOOL;
    pub fn SetKernelObjectSecurity(Handle: HANDLE, SecurityInformation: SECURITY_INFORMATION, SecurityDescriptor: PSECURITY_DESCRIPTOR) -> BOOL;
    pub fn AllocateLocallyUniqueId(Luid: PLUID) -> BOOL;
    // Lsa* (advapi32/secur32 exports; some crate-defined — cohab decl)
    pub fn LsaRegisterLogonProcess(LogonProcessName: *mut LSA_STRING, LsaHandle: *mut HANDLE, SecurityMode: *mut LSA_OPERATIONAL_MODE) -> NTSTATUS;
    pub fn LsaDeregisterLogonProcess(LsaHandle: HANDLE) -> NTSTATUS;
    pub fn LsaLookupAuthenticationPackage(LsaHandle: HANDLE, PackageName: *mut LSA_STRING, AuthenticationPackage: *mut ULONG) -> NTSTATUS;
    pub fn LsaFreeReturnBuffer(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaLogonUser( LsaHandle: HANDLE, OriginName: *mut LSA_STRING, LogonType: SECURITY_LOGON_TYPE, AuthenticationPackage: ULONG, AuthenticationInformation: PVOID, AuthenticationInformationLength: ULONG, LocalGroups: PTOKEN_GROUPS, SourceContext: *mut TOKEN_SOURCE, ProfileBuffer: *mut PVOID, ProfileBufferLength: *mut ULONG, LogonId: *mut LUID, Token: *mut HANDLE, Quotas: *mut QUOTA_LIMITS, SubStatus: *mut NTSTATUS, ) -> NTSTATUS;
    pub fn LsaLookupNames(PolicyHandle: crate::prelude::LSA_HANDLE, Count: ULONG, Names: crate::prelude::PLSA_UNICODE_STRING, ReferencedDomains: *mut crate::prelude::PLSA_REFERENCED_DOMAIN_LIST, Sids: *mut crate::prelude::PLSA_TRANSLATED_SID) -> NTSTATUS;
    pub fn LsaLookupPrivilegeDisplayName(PolicyHandle: crate::prelude::LSA_HANDLE, Name: crate::prelude::PLSA_UNICODE_STRING, DisplayName: *mut crate::prelude::PLSA_UNICODE_STRING, LanguageReturned: PUSHORT) -> NTSTATUS;
    pub fn LsaLookupPrivilegeValue(PolicyHandle: crate::prelude::LSA_HANDLE, Name: crate::prelude::PLSA_UNICODE_STRING, Value: PLUID) -> NTSTATUS;
    pub fn LsaLookupSids(PolicyHandle: crate::prelude::LSA_HANDLE, Count: ULONG, Sids: *mut PSID, ReferencedDomains: *mut crate::prelude::PLSA_REFERENCED_DOMAIN_LIST, Names: *mut crate::prelude::PLSA_TRANSLATED_NAME) -> NTSTATUS;
    // rpcrt4 (RPC_ENTRY = stdcall) — ANSI binding helpers (service/eventlog.rs)
    pub fn RpcStringBindingComposeA(ObjUuid: RPC_CSTR, ProtSeq: RPC_CSTR, NetworkAddr: RPC_CSTR, Endpoint: RPC_CSTR, Options: RPC_CSTR, StringBinding: *mut RPC_CSTR) -> RPC_STATUS;
    pub fn RpcBindingFromStringBindingA(StringBinding: RPC_CSTR, Binding: *mut handle_t) -> RPC_STATUS;
    pub fn RpcStringFreeA(String: *mut RPC_CSTR) -> RPC_STATUS;
    // ntmarta Acc{Rewrite,Tree}* removed: these are advapi32.h macros for
    // NtMartaStatic.<field>(...) (no real symbol) — sec/misc.rs now dispatches
    // through the NTMARTA function-pointer table instead of a direct extern call.
}

// ---- Wave 4: advapi32-own / kernel32 / aclapi exports (WINAPI = stdcall) ----
extern "system" {
    // reg/hkcr.c HKCR predefined-key helpers (LONG WINAPI, internal, not flipped).
    pub fn CreateHKCRKey( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: LPDWORD, ) -> LONG;
    pub fn OpenHKCRKey(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: PHKEY) -> LONG;
    pub fn DeleteHKCRKey(hKey: HKEY, lpSubKey: LPCWSTR, RegSam: REGSAM, Reserved: DWORD) -> LONG;
    pub fn DeleteHKCRValue(hKey: HKEY, ValueName: PUNICODE_STRING) -> LONG;
    pub fn QueryHKCRValue(hKey: HKEY, Name: LPCWSTR, Reserved: LPDWORD, Type: LPDWORD, Data: LPBYTE, Count: LPDWORD) -> LONG;
    pub fn SetHKCRValue(hKey: HKEY, Name: LPCWSTR, Reserved: DWORD, Type: DWORD, Data: *const u8, DataSize: DWORD) -> LONG;
    pub fn EnumHKCRKey( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcbName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcbClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LONG;
    pub fn EnumHKCRValue( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcbName: *mut DWORD, lpReserved: *mut DWORD, lpdwType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD, ) -> LONG;
    pub fn QueryInfoHKCRKey( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LONG;
    // advapi32-own security exports (sec/*, not flipped or crate-defined elsewhere).
    pub fn GetSecurityDescriptorControl(pSecurityDescriptor: PSECURITY_DESCRIPTOR, pControl: *mut SECURITY_DESCRIPTOR_CONTROL, lpdwRevision: LPDWORD) -> BOOL;
    pub fn GetSecurityDescriptorDacl(pSecurityDescriptor: PSECURITY_DESCRIPTOR, lpbDaclPresent: LPBOOL, pDacl: *mut PACL, lpbDaclDefaulted: LPBOOL) -> BOOL;
    pub fn GetSecurityDescriptorSacl(pSecurityDescriptor: PSECURITY_DESCRIPTOR, lpbSaclPresent: LPBOOL, pSacl: *mut PACL, lpbSaclDefaulted: LPBOOL) -> BOOL;
    pub fn GetSecurityDescriptorOwner(pSecurityDescriptor: PSECURITY_DESCRIPTOR, pOwner: *mut PSID, lpbOwnerDefaulted: LPBOOL) -> BOOL;
    pub fn GetSecurityDescriptorGroup(pSecurityDescriptor: PSECURITY_DESCRIPTOR, pGroup: *mut PSID, lpbGroupDefaulted: LPBOOL) -> BOOL;
    pub fn GetNamedSecurityInfoW(pObjectName: LPWSTR, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, ppsidOwner: *mut PSID, ppsidGroup: *mut PSID, ppDacl: *mut PACL, ppSacl: *mut PACL, ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR) -> DWORD;
    pub fn SetNamedSecurityInfoW(pObjectName: LPWSTR, ObjectType: SE_OBJECT_TYPE, SecurityInfo: SECURITY_INFORMATION, psidOwner: PSID, psidGroup: PSID, pDacl: PACL, pSacl: PACL) -> DWORD;
    pub fn LookupAccountNameW(lpSystemName: LPCWSTR, lpAccountName: LPCWSTR, Sid: PSID, cbSid: LPDWORD, ReferencedDomainName: LPWSTR, cchReferencedDomainName: LPDWORD, peUse: *mut SID_NAME_USE) -> BOOL;
    pub fn LookupPrivilegeNameW(lpSystemName: LPCWSTR, lpLuid: PLUID, lpName: LPWSTR, cchName: LPDWORD) -> BOOL;
    pub fn LookupPrivilegeDisplayNameW(lpSystemName: LPCWSTR, lpName: LPCWSTR, lpDisplayName: LPWSTR, cchDisplayName: LPDWORD, lpLanguageId: LPDWORD) -> BOOL;
    // wine/security.c local exports QuerySecurityAccessMask/SetSecurityAccessMask (VOID WINAPI).
    pub fn QuerySecurityAccessMask(SecurityInformation: SECURITY_INFORMATION, DesiredAccess: LPDWORD);
    pub fn SetSecurityAccessMask(SecurityInformation: SECURITY_INFORMATION, DesiredAccess: LPDWORD);
    // cryptsp/SystemFunction* (advapi32.spec @ stdcall) — RC4 session-key crypto.
    pub fn SystemFunction004(input: *const crate::prelude::ustring, key: *const crate::prelude::ustring, output: *mut crate::prelude::ustring) -> NTSTATUS;
    pub fn SystemFunction028(ContextHandle: PVOID, SessionKey: *mut u8) -> NTSTATUS;
    // kernel32 base APIs (WINAPI = stdcall) — service/scm.rs + reg/reg.rs (Wave 4).
    pub fn CreateEventW(lpEventAttributes: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR) -> HANDLE;
    pub fn OpenEventW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> HANDLE;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HANDLE) -> HANDLE;
    pub fn LockResource(hResData: HANDLE) -> LPVOID;
    pub fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn ExpandEnvironmentStringsA(lpSrc: LPCSTR, lpDst: LPSTR, nSize: DWORD) -> DWORD;
    pub fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: c_int) -> LPWSTR;
    pub fn GetVersion() -> DWORD;
}
