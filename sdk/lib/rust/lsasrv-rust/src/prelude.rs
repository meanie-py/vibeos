//! Crate-wide prelude of the universal symbols the per-file DeepSeek
//! translations expect in scope: the NT base substrate (ntdll_sys::nt), the
//! LSAPR_*/RPC_* marshalled types (lsasrv-sys), CRT + Wine wide-string helpers,
//! the lsasrv consts (resources/STATUS/SECURITY_ACCESS), MIDL allocators, and
//! no-op Wine debug shims/macros. Each translated file adds
//! `use crate::prelude::*;`. Mirrors kernel32-rust/src/prelude.rs.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;
// lsasrv marshalled + policy info types absent from ntdll_sys::nt. Curated
// (not a glob) so the base types lsasrv-sys re-bindgen'd (UNICODE_STRING/GUID/
// LUID/…) don't collide with ntdll_sys::nt's.
pub use lsasrv_sys::{
    RPC_UNICODE_STRING, PRPC_UNICODE_STRING, _RPC_UNICODE_STRING,
    LSAPR_HANDLE, PLSAPR_OBJECT_ATTRIBUTES,
    LSAPR_SR_SECURITY_DESCRIPTOR, PLSAPR_SR_SECURITY_DESCRIPTOR,
    LSAPR_POLICY_INFORMATION, PLSAPR_POLICY_INFORMATION,
    LSAPR_POLICY_PRIVILEGE_DEF, PLSAPR_POLICY_PRIVILEGE_DEF, PLSAPR_PRIVILEGE_ENUM_BUFFER,
    LSAPR_POLICY_AUDIT_EVENTS_INFO, PLSAPR_POLICY_AUDIT_EVENTS_INFO,
    LSAPR_POLICY_PRIMARY_DOM_INFO,
    LSAPR_REFERENCED_DOMAIN_LIST, PLSAPR_REFERENCED_DOMAIN_LIST,
    POLICY_INFORMATION_CLASS,
    // database.c LSA DB object model + policy-info structs
    LSA_DB_OBJECT, _LSA_DB_OBJECT, PLSA_DB_OBJECT,
    LSA_DB_OBJECT_TYPE, _LSA_DB_OBJECT_TYPE,
    _LSA_DB_OBJECT_TYPE_LsaDbIgnoreObject,
    _LSA_DB_OBJECT_TYPE_LsaDbPolicyObject,
    _LSA_DB_OBJECT_TYPE_LsaDbAccountObject,
    _LSA_DB_OBJECT_TYPE_LsaDbDomainObject,
    _LSA_DB_OBJECT_TYPE_LsaDbSecretObject,
    _POLICY_AUDIT_LOG_INFO, _POLICY_DEFAULT_QUOTA_INFO,
    _POLICY_MODIFICATION_INFO, _POLICY_AUDIT_FULL_QUERY_INFO,
};

// Bare enum-variant aliases (bindgen names them `_LSA_DB_OBJECT_TYPE_<v>`; the
// translation uses both the bindgen form and the bare C name).
pub const LsaDbIgnoreObject: LSA_DB_OBJECT_TYPE = _LSA_DB_OBJECT_TYPE_LsaDbIgnoreObject;
pub const LsaDbPolicyObject: LSA_DB_OBJECT_TYPE = _LSA_DB_OBJECT_TYPE_LsaDbPolicyObject;
pub const LsaDbAccountObject: LSA_DB_OBJECT_TYPE = _LSA_DB_OBJECT_TYPE_LsaDbAccountObject;
pub const LsaDbDomainObject: LSA_DB_OBJECT_TYPE = _LSA_DB_OBJECT_TYPE_LsaDbDomainObject;
pub const LsaDbSecretObject: LSA_DB_OBJECT_TYPE = _LSA_DB_OBJECT_TYPE_LsaDbSecretObject;

// lsasrv-internal type (its `_LSAP_` prefix dodged the lsasrv-sys bindgen
// allowlist) — hand-declared to match lsasrv.h. AuditEvents length is
// POLICY_AUDIT_EVENT_TYPE_COUNT.
#[repr(C)]
pub struct _LSAP_POLICY_AUDIT_EVENTS_DATA {
    pub AuditingMode: BOOLEAN,
    pub AuditEvents: [DWORD; crate::consts::POLICY_AUDIT_EVENT_TYPE_COUNT as usize],
    pub MaximumAuditEventCount: DWORD,
}
pub type LSAP_POLICY_AUDIT_EVENTS_DATA = _LSAP_POLICY_AUDIT_EVENTS_DATA;
pub type PLSAP_POLICY_AUDIT_EVENTS_DATA = *mut _LSAP_POLICY_AUDIT_EVENTS_DATA;

// policy.c LSAPR_/POLICY_ marshalled info structs (all in lsasrv-sys bindgen).
pub use lsasrv_sys::{
    _LSAPR_POLICY_DNS_DOMAIN_INFO, PLSAPR_POLICY_DNS_DOMAIN_INFO,
    _LSAPR_POLICY_ACCOUNT_DOM_INFO, PLSAPR_POLICY_ACCOUNT_DOM_INFO,
    PLSAPR_POLICY_PRIMARY_DOM_INFO,
    LSAPR_POLICY_PD_ACCOUNT_INFO, PLSAPR_POLICY_PD_ACCOUNT_INFO,
    _POLICY_LSA_SERVER_ROLE_INFO, POLICY_LSA_SERVER_ROLE_INFO,
    _POLICY_LSA_SERVER_ROLE, _POLICY_LSA_SERVER_ROLE_PolicyServerRolePrimary,
    POLICY_MODIFICATION_INFO, POLICY_DEFAULT_QUOTA_INFO,
    POLICY_AUDIT_LOG_INFO, POLICY_AUDIT_FULL_QUERY_INFO, _POLICY_LSA_REPLICA_SRCE_INFO,
    PRPC_SID, SECURITY_LOGON_SESSION_DATA, PSECURITY_LOGON_SESSION_DATA,
    PLSA_STRING, PLSA_API_MSG,
    // notify.c
    POLICY_NOTIFICATION_INFORMATION_CLASS, SEP_RM_API_MESSAGE, LSAP_RM_API_MESSAGE, PLSAP_RM_API_MESSAGE,
    _RM_API_NUMBER, _LSAP_API_NUMBER,
    // lsasrv.c LsaIFree* marshalled-type helpers
    _LSAPR_SR_SECURITY_DESCRIPTOR, _LSAPR_PRIVILEGE_ENUM_BUFFER,
    PLSAPR_TRANSLATED_SIDS, PLSAPR_TRANSLATED_NAMES, PLSAPR_PRIVILEGE_SET,
    PLSAPR_CR_CIPHER_VALUE, PLSAPR_ACCOUNT_ENUM_BUFFER,
    // authport.c LSA API LPC port (PLSA_API_MSG already above for session.c)
    _LSA_API_MSG, LSA_API_MSG,
    LSA_CONNECTION_INFO,
    _LSAP_LOGON_CONTEXT, LSAP_LOGON_CONTEXT, PLSAP_LOGON_CONTEXT,
};

// Bare SRM/LSAP API-number enum-variant aliases (bindgen tags them).
pub const RmAuditSetCommand: _RM_API_NUMBER = lsasrv_sys::_RM_API_NUMBER_RmAuditSetCommand;
pub const RmCreateLogonSession: _RM_API_NUMBER = lsasrv_sys::_RM_API_NUMBER_RmCreateLogonSession;
pub const RmDeleteLogonSession: _RM_API_NUMBER = lsasrv_sys::_RM_API_NUMBER_RmDeleteLogonSession;
pub const LsapAdtWriteLogApi: _LSAP_API_NUMBER = lsasrv_sys::_LSAP_API_NUMBER_LsapAdtWriteLogApi;
pub const LsapComponentTestApi: _LSAP_API_NUMBER = lsasrv_sys::_LSAP_API_NUMBER_LsapComponentTestApi;
pub const LsapAsyncApi: _LSAP_API_NUMBER = lsasrv_sys::_LSAP_API_NUMBER_LsapAsyncApi;

// LSAPR_POLICY_INFORMATION is a union typedef'd directly (no `_`-tag form);
// alias the `_`-name the c2rust output uses.
pub type _LSAPR_POLICY_INFORMATION = LSAPR_POLICY_INFORMATION;
// bindgen names the enum variant with the full `_POLICY_LSA_SERVER_ROLE_` tag;
// the c2rust output drops `LSA_`.
pub const _POLICY_SERVER_ROLE_PolicyServerRolePrimary: _POLICY_LSA_SERVER_ROLE =
    _POLICY_LSA_SERVER_ROLE_PolicyServerRolePrimary;
// PPOLICY_* pointer aliases bindgen did not emit (only the struct + bare alias).
pub type PPOLICY_LSA_SERVER_ROLE_INFO = *mut _POLICY_LSA_SERVER_ROLE_INFO;
pub type PPOLICY_DEFAULT_QUOTA_INFO = *mut _POLICY_DEFAULT_QUOTA_INFO;
pub type PPOLICY_AUDIT_LOG_INFO = *mut _POLICY_AUDIT_LOG_INFO;
pub type PPOLICY_AUDIT_FULL_QUERY_INFO = *mut _POLICY_AUDIT_FULL_QUERY_INFO;
pub type PPOLICY_MODIFICATION_INFO = *mut _POLICY_MODIFICATION_INFO;
pub type PPOLICY_LSA_REPLICA_SRCE_INFO = *mut _POLICY_LSA_REPLICA_SRCE_INFO;

pub use crate::consts::*;

// ---- universal type aliases the c2rust output uses ----
pub type VOID = core::ffi::c_void;
pub type size_t = usize;
pub use core::ffi::{c_char, c_int, c_void};

// ---- wine debug internals (wine/debug.h) ----
// Some c2rust files expand the TRACE/WARN/ERR/FIXME macros to their raw form
// (`if dbch.flags & (1<<__WINE_DBCL_x) { ros_dbg_log(...) }`) rather than the
// crate's no-op TRACE! macros. ros_dbg_log is a real ReactOS export (variadic),
// so declare it and let it resolve at link — preserving the real debug path.
pub type __wine_debug_class = c_int;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        msg: *const c_char,
        ...
    ) -> c_int;
}

// ---- common NT/Win32 constants the translations bare-reference ----
pub const UNICODE_NULL: u16 = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const MAXULONG: ULONG = 0xFFFF_FFFF;

#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- Wine debug channel (wine/debug.h) — the c2rust file preambles construct a
// static __wine_debug_channel; flags are release-inert (TRACE/WARN are no-ops). ----
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub const WINE_DBG_DEFAULT_FLAGS: ::core::ffi::c_uchar = 3;

// ---- MIDL allocators (lsasrv.c; __RPC_USER = stdcall; uppercase real symbols) ----
extern "system" {
    pub fn MIDL_user_allocate(len: usize) -> *mut VOID;
    pub fn MIDL_user_free(ptr: *mut VOID);
    // kernel32 imports (WINAPI = stdcall).
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HMODULE;
    pub fn FindResourceW(hModule: HMODULE, lpName: *const u16, lpType: *const u16) -> HRSRC;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    pub fn LockResource(hResData: HGLOBAL) -> *mut VOID;
    pub fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;
    pub fn CreateThread(
        lpThreadAttributes: *mut VOID,
        dwStackSize: usize,
        lpStartAddress: Option<unsafe extern "system" fn(*mut VOID) -> DWORD>,
        lpParameter: *mut VOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> i32;
}

// Resource handle aliases (winbase) — kernel32 resource API handle types.
pub type HRSRC = HANDLE;
pub type HGLOBAL = HANDLE;

// ---- winsvc substrate (service.rs / service.c) ----
// SCM types + the advapi32/kernel32 service entry points. All WINAPI = stdcall
// on i386 → extern "system" (the service-main + control-handler fn-ptr types are
// WINAPI in winsvc.h; a wrong cdecl cast would drift ESP on i386 — see lane lore).
pub const SERVICE_START_PENDING: DWORD = 2;
pub const SERVICE_RUNNING: DWORD = 4;

pub type LPSERVICE_MAIN_FUNCTIONW = Option<unsafe extern "system" fn(DWORD, *mut LPWSTR)>;
pub type LPHANDLER_FUNCTION = Option<unsafe extern "system" fn(DWORD)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SERVICE_TABLE_ENTRYW {
    pub lpServiceName: LPWSTR,
    pub lpServiceProc: LPSERVICE_MAIN_FUNCTIONW,
}
pub type SERVICE_TABLE_ENTRYW = _SERVICE_TABLE_ENTRYW;
pub type LPSERVICE_TABLE_ENTRYW = *mut _SERVICE_TABLE_ENTRYW;

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
pub type LPSERVICE_STATUS = *mut SERVICE_STATUS;
pub type SERVICE_STATUS_HANDLE = HANDLE; // DECLARE_HANDLE → opaque pointer

extern "system" {
    pub fn StartServiceCtrlDispatcherW(lpServiceStartTable: *const SERVICE_TABLE_ENTRYW) -> BOOL;
    pub fn RegisterServiceCtrlHandlerW(
        lpServiceName: LPCWSTR,
        lpHandlerProc: LPHANDLER_FUNCTION,
    ) -> SERVICE_STATUS_HANDLE;
    pub fn SetServiceStatus(
        hServiceStatus: SERVICE_STATUS_HANDLE,
        lpServiceStatus: LPSERVICE_STATUS,
    ) -> BOOL;
    pub fn I_ScIsSecurityProcess();

    // kernel32 module/event/wait helpers used by the service dispatcher thread.
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> HINSTANCE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const c_char) -> FARPROC;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn CreateEventW(
        lpEventAttributes: *mut _SECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;
    pub fn OpenEventW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: *const u16) -> HANDLE;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn GetLastError() -> DWORD;
}

// ---- SAM substrate (authpackage.c SAM bridge → samsrv-sys) ----
// SAMPR_* marshalled types + Samr* __stdcall client stubs from samsrv-sys
// (bindgen over the widl sam_c.h). SamIFree_SAMPR_ULONG_ARRAY is a samsrv.dll
// export (not in sam_c.h) — hand-declared. SAM access masks: ntsam.h (ddk).
pub use samsrv_sys::{
    SamrCloseHandle, SamrGetAliasMembership, SamrOpenDomain,
    SAMPR_SID_INFORMATION, SAMPR_ULONG_ARRAY,
};
pub type PSAMPR_SID_INFORMATION = *mut SAMPR_SID_INFORMATION;
// authpackage.c LsarOpenPolicy bridge: server-name arg type (lsasrv-sys bindgen).
pub use lsasrv_sys::PLSAPR_SERVER_NAME;
pub const SAM_SERVER_CONNECT: u32 = 0x00000001;
pub const SAM_SERVER_LOOKUP_DOMAIN: u32 = 0x00000020;
pub const DOMAIN_GET_ALIAS_MEMBERSHIP: u32 = 0x00000080;
pub const ACCOUNT_VIEW: u32 = 1; // ntlsa.h
extern "system" {
    pub fn SamIFree_SAMPR_ULONG_ARRAY(ptr: *mut SAMPR_ULONG_ARRAY);
}

// ---- lookup.c marshalled types + SAM bridge ----
// LSAPR_* translated-name/SID marshalled structs + referenced-domain-list +
// SID-enum-buffer + trust-info (all in lsasrv-sys bindgen). _RPC_SID is the
// marshalled SID. Samr* client stubs + SAMPR arrays come from samsrv-sys.
pub use lsasrv_sys::{
    LSAPR_TRANSLATED_NAME_EX, PLSAPR_TRANSLATED_NAME_EX, PLSAPR_TRANSLATED_NAMES_EX, PLSAPR_TRANSLATED_SID_EX,
    _LSAPR_TRANSLATED_SID_EX2, LSAPR_TRANSLATED_SID_EX2, PLSAPR_TRANSLATED_SID_EX2,
    _LSAPR_TRANSLATED_SIDS_EX2, LSAPR_TRANSLATED_SIDS_EX2, PLSAPR_TRANSLATED_SIDS_EX2,
    _LSAPR_REFERENCED_DOMAIN_LIST, PLSAPR_SID_ENUM_BUFFER, PLSAPR_TRUST_INFORMATION,
    _LSA_TRUST_INFORMATION, LSA_TRUST_INFORMATION,
    _RPC_SID,
};

// LSAP_LOOKUP_LEVEL enum (lsa.idl _LSAP_LOOKUP_LEVEL) — its `_LSAP_` prefix
// dodged the lsasrv-sys bindgen allowlist; hand-declared to match the IDL.
pub type LSAP_LOOKUP_LEVEL = core::ffi::c_int;
pub type PLSAP_LOOKUP_LEVEL = *mut LSAP_LOOKUP_LEVEL;
pub const LsapLookupWksta: LSAP_LOOKUP_LEVEL = 1;
pub const LsapLookupPDC: LSAP_LOOKUP_LEVEL = 2;
pub const LsapLookupTDL: LSAP_LOOKUP_LEVEL = 3;
pub const LsapLookupGC: LSAP_LOOKUP_LEVEL = 4;
pub const LsapLookupXForestReferral: LSAP_LOOKUP_LEVEL = 5;
pub const LsapLookupXForestResolve: LSAP_LOOKUP_LEVEL = 6;
pub const LsapLookupRODCReferralToFullDC: LSAP_LOOKUP_LEVEL = 7;

// SAM bridge for lookup.c name/id lookups. Samr* are __stdcall client stubs.
pub use samsrv_sys::{
    SamrConnect, SamrLookupNamesInDomain, SamrLookupIdsInDomain,
    _SAMPR_ULONG_ARRAY,
    SAMPR_RETURNED_USTRING_ARRAY,
};
// SamIFree_SAMPR_RETURNED_USTRING_ARRAY is a samsrv.dll export (not in sam_c.h).
extern "system" {
    pub fn SamIFree_SAMPR_RETURNED_USTRING_ARRAY(
        ptr: *mut samsrv_sys::SAMPR_RETURNED_USTRING_ARRAY,
    );
}
// SAM domain-lookup access mask (ddk/ntsam.h).
pub const DOMAIN_LOOKUP: u32 = 0x00000200;

// ---- lsarpc.c marshalled types (the LSA RPC server methods) ----
// LSAPR_*/RPC_*/POLICY_*/TRUSTED_*/LSA_FOREST_* marshalled types from the
// lsasrv-sys bindgen (lsa_s.h widl output). The enum *-class types alias their
// bindgen `_`-tagged form via `pub use self::_X as X` inside lsasrv-sys.
pub use lsasrv_sys::{
    PLSAPR_HANDLE,
    LSAPR_ACCOUNT_INFORMATION, PLSAPR_ACCOUNT_INFORMATION, LSAPR_ACCOUNT_ENUM_BUFFER,
    PLSAPR_USER_RIGHT_SET,
    LSAPR_CR_CIPHER_VALUE, // PLSAPR_CR_CIPHER_VALUE already re-exported above
    LSAPR_TRANSLATED_NAME, LSAPR_TRANSLATED_NAMES_EX,
    PLSAPR_TRANSLATED_SIDS_EX,
    LSA_TRANSLATED_SID,
    PLSA_UNICODE_STRING,
    PLSAPR_TRUSTED_DOMAIN_INFO, PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX,
    PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION, PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL,
    PLSAPR_TRUSTED_ENUM_BUFFER, PLSAPR_TRUSTED_ENUM_BUFFER_EX,
    PLSAPR_POLICY_DOMAIN_INFORMATION,
    TRUSTED_INFORMATION_CLASS,
    PLSA_FOREST_TRUST_INFORMATION, PLSA_FOREST_TRUST_COLLISION_INFORMATION,
    LSA_FOREST_TRUST_RECORD_TYPE,
};
// POLICY_INFORMATION_CLASS already re-exported at top of file.

// POLICY_INFORMATION_CLASS / POLICY_NOTIFICATION_INFORMATION_CLASS enum *variants*:
// the c2rust match arms reference them by the bare C name (`POLICY_INFORMATION_CLASS_PolicyX`,
// without bindgen's leading `_`). Alias the bindgen consts so the patterns resolve.
pub use lsasrv_sys::{
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation as POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation,
    _POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation as POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation,
    _POLICY_INFORMATION_CLASS_PolicyAuditFullQueryInformation as POLICY_INFORMATION_CLASS_PolicyAuditFullQueryInformation,
    _POLICY_INFORMATION_CLASS_PolicyAuditFullSetInformation as POLICY_INFORMATION_CLASS_PolicyAuditFullSetInformation,
    _POLICY_INFORMATION_CLASS_PolicyAuditLogInformation as POLICY_INFORMATION_CLASS_PolicyAuditLogInformation,
    _POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation as POLICY_INFORMATION_CLASS_PolicyDefaultQuotaInformation,
    _POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation as POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation,
    _POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt as POLICY_INFORMATION_CLASS_PolicyDnsDomainInformationInt,
    _POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation as POLICY_INFORMATION_CLASS_PolicyLocalAccountDomainInformation,
    _POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation as POLICY_INFORMATION_CLASS_PolicyLsaServerRoleInformation,
    _POLICY_INFORMATION_CLASS_PolicyModificationInformation as POLICY_INFORMATION_CLASS_PolicyModificationInformation,
    _POLICY_INFORMATION_CLASS_PolicyPdAccountInformation as POLICY_INFORMATION_CLASS_PolicyPdAccountInformation,
    _POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation as POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation,
    _POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation as POLICY_INFORMATION_CLASS_PolicyReplicaSourceInformation,
    _POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAccountDomainInformation as POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAccountDomainInformation,
    _POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAuditEventsInformation as POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyAuditEventsInformation,
    _POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyDnsDomainInformation as POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyDnsDomainInformation,
    _POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyServerRoleInformation as POLICY_NOTIFICATION_INFORMATION_CLASS_PolicyNotifyServerRoleInformation,
};

// PLSAPR_SID = LSAPR_SID* (LSAPR_SID is the marshalled RPC_SID — lsa.idl); the
// `_RPC_SID` form is what lsasrv-sys emits, so its pointer is PRPC_SID.
pub type PLSAPR_SID = PRPC_SID;

// RPC binding handle (rpcdce.h: `typedef void* handle_t`).
pub type handle_t = *mut VOID;

// _CREDPR_TARGET_INFORMATION (lsa.idl /* Credentials */) — dodged the lsasrv-sys
// bindgen allowlist; hand-declared to match the IDL struct verbatim.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _CREDPR_TARGET_INFORMATION {
    pub TargetName: LPWSTR,
    pub NetbiosServerName: LPWSTR,
    pub DnsServerName: LPWSTR,
    pub NetbiosDomainName: LPWSTR,
    pub DnsDomainName: LPWSTR,
    pub DnsTreeName: LPWSTR,
    pub PackageName: LPWSTR,
    pub Flags: ULONG,
}
pub type CREDPR_TARGET_INFORMATION = _CREDPR_TARGET_INFORMATION;
pub type PCREDPR_TARGET_INFORMATION = *mut _CREDPR_TARGET_INFORMATION;

// Cross-file global: NT_PRODUCT_TYPE LsapProductType (lsasrv.rs #[no_mangle] static).
pub use crate::lsasrv::lsasrv::LsapProductType;

// LSA DB object-type: the c2rust output uses the `LsaDbObjectType_<v>` spelling
// at one site (alongside the bare LsaDb*Object names already in scope).
pub const LsaDbObjectType_LsaDbPolicyObject: LSA_DB_OBJECT_TYPE = LsaDbPolicyObject;

// MIDL allocators, lowercase form: in C these are `#define midl_user_allocate
// MIDL_user_allocate` (rpc runtime). Thin wrappers over the real RPC symbols.
#[inline]
pub unsafe fn midl_user_allocate(len: usize) -> *mut VOID {
    MIDL_user_allocate(len)
}
#[inline]
pub unsafe fn midl_user_free(ptr: *mut VOID) {
    MIDL_user_free(ptr)
}

// ---- LSA token-info + logon types (authpackage.c LsapLogonUser token build) ----
// ntsecpkg.h collides with the reduced NDK bindgen context (SSPI/SecBuffer), so
// these few types are hand-written #[repr(C)] over the prelude TOKEN_*/UNICODE
// types (which already carry the exact C layout). SECURITY_LOGON_TYPE + its
// variants come from lsasrv-sys bindgen; alias the bare c2rust names.
pub use lsasrv_sys::SECURITY_LOGON_TYPE;
pub const Interactive: SECURITY_LOGON_TYPE = 2;
pub const Network: SECURITY_LOGON_TYPE = 3;
pub const Batch: SECURITY_LOGON_TYPE = 4;
pub const Service: SECURITY_LOGON_TYPE = 5;

// LSAPR privilege/luid marshalled types (lsasrv-sys bindgen) used by the token build.
pub use lsasrv_sys::{
    LSAPR_LUID_AND_ATTRIBUTES, PLSAPR_LUID_AND_ATTRIBUTES,
    _LSAPR_PRIVILEGE_SET,
}; // PLSAPR_PRIVILEGE_SET already re-exported above

pub type LSA_TOKEN_INFORMATION_TYPE = core::ffi::c_uint;
pub const LsaTokenInformationNull: LSA_TOKEN_INFORMATION_TYPE = 0;
pub const LsaTokenInformationV1: LSA_TOKEN_INFORMATION_TYPE = 1;
pub const LsaTokenInformationV2: LSA_TOKEN_INFORMATION_TYPE = 2;
// c2rust also references the bindgen-style enum-const spelling.
pub const _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationNull: LSA_TOKEN_INFORMATION_TYPE = 0;
pub const _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationV1: LSA_TOKEN_INFORMATION_TYPE = 1;
pub const _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationV2: LSA_TOKEN_INFORMATION_TYPE = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _LSA_TOKEN_INFORMATION_V1 {
    pub ExpirationTime: LARGE_INTEGER,
    pub User: TOKEN_USER,
    pub Groups: PTOKEN_GROUPS,
    pub PrimaryGroup: TOKEN_PRIMARY_GROUP,
    pub Privileges: PTOKEN_PRIVILEGES,
    pub Owner: TOKEN_OWNER,
    pub DefaultDacl: TOKEN_DEFAULT_DACL,
}
pub type LSA_TOKEN_INFORMATION_V1 = _LSA_TOKEN_INFORMATION_V1;
pub type PLSA_TOKEN_INFORMATION_V1 = *mut _LSA_TOKEN_INFORMATION_V1;

pub type MSV1_0_LOGON_SUBMIT_TYPE = core::ffi::c_uint;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _MSV1_0_INTERACTIVE_LOGON {
    pub MessageType: MSV1_0_LOGON_SUBMIT_TYPE,
    pub LogonDomainName: _UNICODE_STRING,
    pub UserName: _UNICODE_STRING,
    pub Password: _UNICODE_STRING,
}
pub type MSV1_0_INTERACTIVE_LOGON = _MSV1_0_INTERACTIVE_LOGON;
pub type PMSV1_0_INTERACTIVE_LOGON = *mut _MSV1_0_INTERACTIVE_LOGON;

// ---- CRT functions (cdecl → extern "C"); resolve against ntdll/CRT at link ----
extern "C" {
    pub fn memcpy(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memmove(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memset(d: *mut VOID, c: c_int, n: usize) -> *mut VOID;
    pub fn memcmp(a: *const VOID, b: *const VOID, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn _strnicmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG;
}

// RtlGetProcessHeap() is a macro (NtCurrentPeb()->ProcessHeap) in rtlfuncs.h,
// not an export — implement in-crate so call sites bind directly (mirrors
// kernel32-rust/k32_externs).
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

// RtlCopyMemory is a `#define RtlCopyMemory memcpy` macro (ntrtl.h) — not an
// export. In-crate wrapper so call sites bind directly.
#[inline]
pub unsafe fn RtlCopyMemory(destination: *mut VOID, source: *const VOID, length: usize) {
    memcpy(destination, source, length);
}

// RtlInitEmptyUnicodeString is a FORCEINLINE in rtlfuncs.h (not an export).
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: *mut u16,
    BufferSize: u16,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// ---- STUB / UNIMPLEMENTED markers (ReactOS `STUB;` statements → no-op) ----
pub const STUB: () = ();
pub const UNIMPLEMENTED: () = ();

// DUMMY_PSEH: SEH filters/handlers are dead code.
#[inline]
pub fn _SEH2_GetExceptionCode() -> LONG {
    0
}

// ---- no-op Wine debug shims (NDEBUG: TRACE/WARN/ERR/FIXME compile out) ----
#[inline]
pub fn debugstr_w(_s: *const u16) -> *const i8 {
    core::ptr::null()
}
#[inline]
pub fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}
