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
    LSAPR_ACL, PLSAPR_ACL,
    LSAPR_HANDLE, LSA_HANDLE,
    LSAPR_OBJECT_ATTRIBUTES, PLSAPR_OBJECT_ATTRIBUTES,
    LSAPR_SECURITY_DESCRIPTOR, PLSAPR_SECURITY_DESCRIPTOR,
    LSAPR_SR_SECURITY_DESCRIPTOR, PLSAPR_SR_SECURITY_DESCRIPTOR,
    LSAPR_POLICY_INFORMATION, PLSAPR_POLICY_INFORMATION,
    LSAPR_POLICY_PRIVILEGE_DEF, PLSAPR_POLICY_PRIVILEGE_DEF,
    LSAPR_PRIVILEGE_ENUM_BUFFER, PLSAPR_PRIVILEGE_ENUM_BUFFER,
    LSAPR_POLICY_AUDIT_EVENTS_INFO, PLSAPR_POLICY_AUDIT_EVENTS_INFO,
    LSAPR_POLICY_PRIMARY_DOM_INFO, LSAPR_POLICY_ACCOUNT_DOM_INFO,
    LSAPR_SID_INFORMATION, LSAPR_SID_ENUM_BUFFER,
    LSAPR_REFERENCED_DOMAIN_LIST, PLSAPR_REFERENCED_DOMAIN_LIST,
    LSAPR_TRUST_INFORMATION,
    POLICY_INFORMATION_CLASS,
    DSROLE_PRIMARY_DOMAIN_INFO_LEVEL,
    PDSROLER_PRIMARY_DOMAIN_INFORMATION,
    // database.c LSA DB object model + policy-info structs
    LSA_DB_OBJECT, _LSA_DB_OBJECT, PLSA_DB_OBJECT,
    LSA_DB_OBJECT_TYPE, _LSA_DB_OBJECT_TYPE, PLSA_DB_OBJECT_TYPE,
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
    POLICY_AUDIT_LOG_INFO, POLICY_AUDIT_FULL_QUERY_INFO,
    POLICY_LSA_REPLICA_SRCE_INFO, _POLICY_LSA_REPLICA_SRCE_INFO,
    PRPC_SID,
    // session.c logon-session types
    _SECURITY_LOGON_SESSION_DATA, SECURITY_LOGON_SESSION_DATA, PSECURITY_LOGON_SESSION_DATA,
    PLSA_STRING, PLSA_API_MSG,
    // notify.c
    POLICY_NOTIFICATION_INFORMATION_CLASS,
    // srm.c SRM port messages
    _SEP_RM_API_MESSAGE, SEP_RM_API_MESSAGE, PSEP_RM_API_MESSAGE,
    _LSAP_RM_API_MESSAGE, LSAP_RM_API_MESSAGE, PLSAP_RM_API_MESSAGE,
    _RM_API_NUMBER, _LSAP_API_NUMBER,
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
pub use core::ffi::{c_char, c_int, c_long, c_void};

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

// ---- CRT functions (cdecl → extern "C"); resolve against ntdll/CRT at link ----
extern "C" {
    pub fn memcpy(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memmove(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memset(d: *mut VOID, c: c_int, n: usize) -> *mut VOID;
    pub fn memcmp(a: *const VOID, b: *const VOID, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
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
