//! Crate-wide prelude of the universal symbols the per-file translations expect
//! in scope: the NT base substrate (ntdll_sys::nt), the SAMPR_*/RPC_* marshalled
//! types (samsrv-sys), shared LSA SD/SID/policy types (lsasrv-sys), the
//! samsrv-internal types (samsrv.h, hand-declared), CRT + Wine wide-string
//! helpers, MIDL allocators, and no-op Wine debug shims. Each translated file
//! does `use crate::prelude::*;`. Mirrors lsasrv-rust/src/prelude.rs. Curated
//! re-exports (not globs) where samsrv-sys / lsasrv-sys base types would collide
//! with ntdll_sys::nt.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;
pub use crate::consts::*;

pub use core::ffi::{c_char, c_int, c_long, c_void};
pub type VOID = core::ffi::c_void;
pub type size_t = usize;

// ============================================================================
// samsrv-sys — SAMPR_* RPC marshalled types + fixed-data + SAM client stubs
// (bindgen over sam_c.h). Curated re-export (explicit, not glob) so the RPC base
// types don't collide with the ntdll_sys::nt glob above.
// ============================================================================
pub use samsrv_sys::{
    SAMPR_HANDLE,
    PRPC_SID, PRPC_UNICODE_STRING,
    ENCRYPTED_NT_OWF_PASSWORD, ENCRYPTED_LM_OWF_PASSWORD,
    PENCRYPTED_NT_OWF_PASSWORD, PENCRYPTED_LM_OWF_PASSWORD,
    PGROUP_MEMBERSHIP,
    SAMPR_SID_INFORMATION, PSAMPR_SID_INFORMATION,
    SAMPR_DOMAIN_DISPLAY_USER,
    PSAMPR_LOGON_HOURS,
    PSAMPR_USER_INFO_BUFFER, PSAMPR_GROUP_INFO_BUFFER, PSAMPR_ALIAS_INFO_BUFFER,
    PSAMPR_DOMAIN_INFO_BUFFER, PSAMPR_DISPLAY_INFO_BUFFER, PSAMPR_ENUMERATION_BUFFER,
    PSAMPR_GET_MEMBERS_BUFFER, PSAMPR_GET_GROUPS_BUFFER,
    PSAMPR_ULONG_ARRAY, PSAMPR_PSID_ARRAY,
    PSAMPR_SR_SECURITY_DESCRIPTOR, PSAMPR_SERVER_NAME,
    PSAMPR_RETURNED_USTRING_ARRAY,
    // Wave-2 (samrpc.rs) value/by-value type uses.
    SAMPR_USER_INFO_BUFFER, SAMPR_GROUP_INFO_BUFFER, SAMPR_ALIAS_INFO_BUFFER,
    SAMPR_DOMAIN_INFO_BUFFER, SAMPR_ENUMERATION_BUFFER,
    SAMPR_RID_ENUMERATION, PSAMPR_RID_ENUMERATION,
    SAMPR_GET_MEMBERS_BUFFER, SAMPR_GET_GROUPS_BUFFER,
    SAMPR_SR_SECURITY_DESCRIPTOR, PSAMPR_PSID_ARRAY_OUT,
    SAMPR_REVISION_INFO, PSAMPR_ENCRYPTED_USER_PASSWORD,
    RPC_UNICODE_STRING, PRPC_STRING,
    PUSER_DOMAIN_PASSWORD_INFORMATION,
    PSAM_VALIDATE_INPUT_ARG, PSAM_VALIDATE_OUTPUT_ARG,
    handle_t,
};
// NT/LM OWF password (ntsecapi.h) — bindgen aliases both to the LM struct.
pub type NT_OWF_PASSWORD = samsrv_sys::ENCRYPTED_NT_OWF_PASSWORD;
pub type LM_OWF_PASSWORD = samsrv_sys::ENCRYPTED_LM_OWF_PASSWORD;
pub type USER_DOMAIN_PASSWORD_INFORMATION = samsrv_sys::_USER_DOMAIN_PASSWORD_INFORMATION;
// Full-prefix DOMAIN_INFORMATION_CLASS_* const aliases (some match arms use the
// bindgen enum-prefixed spelling rather than the bare Domain* form above).
pub use samsrv_sys::{
    _DOMAIN_INFORMATION_CLASS_DomainGeneralInformation as DOMAIN_INFORMATION_CLASS_DomainGeneralInformation,
    _DOMAIN_INFORMATION_CLASS_DomainGeneralInformation2 as DOMAIN_INFORMATION_CLASS_DomainGeneralInformation2,
    _DOMAIN_INFORMATION_CLASS_DomainLockoutInformation as DOMAIN_INFORMATION_CLASS_DomainLockoutInformation,
    _DOMAIN_INFORMATION_CLASS_DomainLogoffInformation as DOMAIN_INFORMATION_CLASS_DomainLogoffInformation,
    _DOMAIN_INFORMATION_CLASS_DomainModifiedInformation as DOMAIN_INFORMATION_CLASS_DomainModifiedInformation,
    _DOMAIN_INFORMATION_CLASS_DomainModifiedInformation2 as DOMAIN_INFORMATION_CLASS_DomainModifiedInformation2,
    _DOMAIN_INFORMATION_CLASS_DomainNameInformation as DOMAIN_INFORMATION_CLASS_DomainNameInformation,
    _DOMAIN_INFORMATION_CLASS_DomainOemInformation as DOMAIN_INFORMATION_CLASS_DomainOemInformation,
    _DOMAIN_INFORMATION_CLASS_DomainPasswordInformation as DOMAIN_INFORMATION_CLASS_DomainPasswordInformation,
    _DOMAIN_INFORMATION_CLASS_DomainReplicationInformation as DOMAIN_INFORMATION_CLASS_DomainReplicationInformation,
    _DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation as DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation,
    _DOMAIN_INFORMATION_CLASS_DomainStateInformation as DOMAIN_INFORMATION_CLASS_DomainStateInformation,
};
// PASSWORD_POLICY_VALIDATION_TYPE enum + bare variant consts (sam.idl).
pub type PASSWORD_POLICY_VALIDATION_TYPE = samsrv_sys::_PASSWORD_POLICY_VALIDATION_TYPE;

// Bare enum aliases bindgen emitted only with the `_`-tag form.
pub type GROUP_MEMBERSHIP = samsrv_sys::_GROUP_MEMBERSHIP;
pub type DOMAIN_INFORMATION_CLASS = samsrv_sys::_DOMAIN_INFORMATION_CLASS;
pub type USER_INFORMATION_CLASS = samsrv_sys::_USER_INFORMATION_CLASS;
pub type GROUP_INFORMATION_CLASS = samsrv_sys::_GROUP_INFORMATION_CLASS;
pub type ALIAS_INFORMATION_CLASS = samsrv_sys::_ALIAS_INFORMATION_CLASS;
pub type DOMAIN_DISPLAY_INFORMATION = samsrv_sys::_DOMAIN_DISPLAY_INFORMATION;

// c2rust match arms reference enum variants by their bare C name; alias the
// bindgen `_<ENUM>_<Variant>` consts so the match arms are const patterns (not
// irrefutable bindings — which would make every dispatch take the first arm).
pub use samsrv_sys::{
    _ALIAS_INFORMATION_CLASS_AliasAdminCommentInformation as AliasAdminCommentInformation,
    _ALIAS_INFORMATION_CLASS_AliasGeneralInformation as AliasGeneralInformation,
    _ALIAS_INFORMATION_CLASS_AliasNameInformation as AliasNameInformation,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayGroup as DomainDisplayGroup,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayMachine as DomainDisplayMachine,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayOemGroup as DomainDisplayOemGroup,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayOemUser as DomainDisplayOemUser,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayUser as DomainDisplayUser,
    _DOMAIN_INFORMATION_CLASS_DomainGeneralInformation as DomainGeneralInformation,
    _DOMAIN_INFORMATION_CLASS_DomainGeneralInformation2 as DomainGeneralInformation2,
    _DOMAIN_INFORMATION_CLASS_DomainLockoutInformation as DomainLockoutInformation,
    _DOMAIN_INFORMATION_CLASS_DomainLogoffInformation as DomainLogoffInformation,
    _DOMAIN_INFORMATION_CLASS_DomainModifiedInformation as DomainModifiedInformation,
    _DOMAIN_INFORMATION_CLASS_DomainModifiedInformation2 as DomainModifiedInformation2,
    _DOMAIN_INFORMATION_CLASS_DomainNameInformation as DomainNameInformation,
    _DOMAIN_INFORMATION_CLASS_DomainOemInformation as DomainOemInformation,
    _DOMAIN_INFORMATION_CLASS_DomainPasswordInformation as DomainPasswordInformation,
    _DOMAIN_INFORMATION_CLASS_DomainReplicationInformation as DomainReplicationInformation,
    _DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation as DomainServerRoleInformation,
    _DOMAIN_INFORMATION_CLASS_DomainStateInformation as DomainStateInformation,
    _GROUP_INFORMATION_CLASS_GroupAdminCommentInformation as GroupAdminCommentInformation,
    _GROUP_INFORMATION_CLASS_GroupAttributeInformation as GroupAttributeInformation,
    _GROUP_INFORMATION_CLASS_GroupGeneralInformation as GroupGeneralInformation,
    _GROUP_INFORMATION_CLASS_GroupNameInformation as GroupNameInformation,
    _USER_INFORMATION_CLASS_UserAccountInformation as UserAccountInformation,
    _USER_INFORMATION_CLASS_UserAccountNameInformation as UserAccountNameInformation,
    _USER_INFORMATION_CLASS_UserAdminCommentInformation as UserAdminCommentInformation,
    _USER_INFORMATION_CLASS_UserAllInformation as UserAllInformation,
    _USER_INFORMATION_CLASS_UserControlInformation as UserControlInformation,
    _USER_INFORMATION_CLASS_UserExpiresInformation as UserExpiresInformation,
    _USER_INFORMATION_CLASS_UserFullNameInformation as UserFullNameInformation,
    _USER_INFORMATION_CLASS_UserGeneralInformation as UserGeneralInformation,
    _USER_INFORMATION_CLASS_UserHomeInformation as UserHomeInformation,
    _USER_INFORMATION_CLASS_UserInternal1Information as UserInternal1Information,
    _USER_INFORMATION_CLASS_UserInternal2Information as UserInternal2Information,
    _USER_INFORMATION_CLASS_UserLogonHoursInformation as UserLogonHoursInformation,
    _USER_INFORMATION_CLASS_UserLogonInformation as UserLogonInformation,
    _USER_INFORMATION_CLASS_UserNameInformation as UserNameInformation,
    _USER_INFORMATION_CLASS_UserParametersInformation as UserParametersInformation,
    _USER_INFORMATION_CLASS_UserPreferencesInformation as UserPreferencesInformation,
    _USER_INFORMATION_CLASS_UserPrimaryGroupInformation as UserPrimaryGroupInformation,
    _USER_INFORMATION_CLASS_UserProfileInformation as UserProfileInformation,
    _USER_INFORMATION_CLASS_UserScriptInformation as UserScriptInformation,
    _USER_INFORMATION_CLASS_UserSetPasswordInformation as UserSetPasswordInformation,
    _USER_INFORMATION_CLASS_UserWorkStationsInformation as UserWorkStationsInformation,
};

// ============================================================================
// lsasrv-sys — shared LSA policy/handle/SD types absent from samsrv-sys.
// ============================================================================
pub use lsasrv_sys::{LSA_HANDLE, LSA_OBJECT_ATTRIBUTES, POLICY_ACCOUNT_DOMAIN_INFO};
pub type PPOLICY_ACCOUNT_DOMAIN_INFO = *mut POLICY_ACCOUNT_DOMAIN_INFO;

// ============================================================================
// samsrv-internal types — hand-declared verbatim from dll/win32/samsrv/samsrv.h
// (the private SAM database object model + fixed-data records). Their `SAM_`
// prefix is below the samsrv-sys bindgen allowlist (sam_c.h only), so they live
// here. Layout matches the C struct exactly.
// ============================================================================
pub type SAM_DB_OBJECT_TYPE = core::ffi::c_uint;
pub const SamDbIgnoreObject: SAM_DB_OBJECT_TYPE = 0;
pub const SamDbServerObject: SAM_DB_OBJECT_TYPE = 1;
pub const SamDbDomainObject: SAM_DB_OBJECT_TYPE = 2;
pub const SamDbAliasObject: SAM_DB_OBJECT_TYPE = 3;
pub const SamDbGroupObject: SAM_DB_OBJECT_TYPE = 4;
pub const SamDbUserObject: SAM_DB_OBJECT_TYPE = 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SAM_DB_OBJECT {
    pub Signature: ULONG,
    pub ObjectType: SAM_DB_OBJECT_TYPE,
    pub RefCount: ULONG,
    pub Access: ACCESS_MASK,
    pub Name: LPWSTR,
    pub KeyHandle: HANDLE,
    pub MembersKeyHandle: HANDLE,
    pub RelativeId: ULONG,
    pub Trusted: BOOLEAN,
    pub ParentObject: *mut _SAM_DB_OBJECT,
}
pub type SAM_DB_OBJECT = _SAM_DB_OBJECT;
pub type PSAM_DB_OBJECT = *mut _SAM_DB_OBJECT;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SAM_ALIAS_FIXED_DATA {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub AliasId: ULONG,
}
pub type SAM_ALIAS_FIXED_DATA = _SAM_ALIAS_FIXED_DATA;
pub type PSAM_ALIAS_FIXED_DATA = *mut _SAM_ALIAS_FIXED_DATA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SAM_DOMAIN_FIXED_DATA {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub CreationTime: LARGE_INTEGER,
    pub DomainModifiedCount: LARGE_INTEGER,
    pub MaxPasswordAge: LARGE_INTEGER,
    pub MinPasswordAge: LARGE_INTEGER,
    pub ForceLogoff: LARGE_INTEGER,
    pub LockoutDuration: LARGE_INTEGER,
    pub LockoutObservationWindow: LARGE_INTEGER,
    pub ModifiedCountAtLastPromotion: LARGE_INTEGER,
    pub NextRid: ULONG,
    pub PasswordProperties: ULONG,
    pub MinPasswordLength: USHORT,
    pub PasswordHistoryLength: USHORT,
    pub LockoutThreshold: USHORT,
    pub DomainServerState: DOMAIN_SERVER_ENABLE_STATE,
    pub DomainServerRole: DOMAIN_SERVER_ROLE,
    pub UasCompatibilityRequired: BOOLEAN,
}
pub type SAM_DOMAIN_FIXED_DATA = _SAM_DOMAIN_FIXED_DATA;
pub type PSAM_DOMAIN_FIXED_DATA = *mut _SAM_DOMAIN_FIXED_DATA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SAM_GROUP_FIXED_DATA {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub GroupId: ULONG,
    pub Attributes: ULONG,
}
pub type SAM_GROUP_FIXED_DATA = _SAM_GROUP_FIXED_DATA;
pub type PSAM_GROUP_FIXED_DATA = *mut _SAM_GROUP_FIXED_DATA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _SAM_USER_FIXED_DATA {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub LastLogon: LARGE_INTEGER,
    pub LastLogoff: LARGE_INTEGER,
    pub PasswordLastSet: LARGE_INTEGER,
    pub AccountExpires: LARGE_INTEGER,
    pub LastBadPasswordTime: LARGE_INTEGER,
    pub UserId: ULONG,
    pub PrimaryGroupId: ULONG,
    pub UserAccountControl: ULONG,
    pub CountryCode: USHORT,
    pub CodePage: USHORT,
    pub BadPasswordCount: USHORT,
    pub LogonCount: USHORT,
    pub AdminCount: USHORT,
    pub OperatorCount: USHORT,
}
pub type SAM_USER_FIXED_DATA = _SAM_USER_FIXED_DATA;
pub type PSAM_USER_FIXED_DATA = *mut _SAM_USER_FIXED_DATA;

// DOMAIN_SERVER_ENABLE_STATE / DOMAIN_SERVER_ROLE (ntsam.h) used by the fixed
// data above + the c2rust output (bare enum-variant names).
pub type DOMAIN_SERVER_ENABLE_STATE = core::ffi::c_uint;
pub const DomainServerEnabled: DOMAIN_SERVER_ENABLE_STATE = 1;
pub const DomainServerDisabled: DOMAIN_SERVER_ENABLE_STATE = 2;
pub type DOMAIN_SERVER_ROLE = core::ffi::c_uint;
pub const DomainServerRoleBackup: DOMAIN_SERVER_ROLE = 2;
pub const DomainServerRolePrimary: DOMAIN_SERVER_ROLE = 3;

// ============================================================================
// Cross-file globals (samsrv.c → samsrv.rs; database.c file-local SamKeyHandle).
// EmptyNtHash/EmptyLmHash/SampProductType are defined #[no_mangle] in samsrv.rs
// and referenced from setup.rs/user.rs (and the still-C samrpc.c at final link).
// pServerMapping is defined in samrpc.c (still C, Wave 2) → extern here.
// ============================================================================
pub use crate::samsrv::samsrv::{EmptyNtHash, EmptyLmHash, SampProductType, SampResource};
// pServerMapping + SampStartRpcServer are now defined in samrpc.rs (Wave 2).

// ============================================================================
// Cross-file SAM helper functions — defined in sibling modules; re-exported so
// every translated file resolves the bare-name calls (mirrors the C external
// linkage of these Samp*/Sam* helpers).
// ============================================================================
pub use crate::samsrv::utils::{
    AppendRidToSid, SampIsSetupRunning, SampLoadString, SampCheckAccountName, SampGetRidFromSid,
};
pub use crate::samsrv::samrpc::{SampStartRpcServer, pServerMapping};
pub use crate::samsrv::user::{
    SampAddGroupMembershipToUser, SampGetLogonHoursAttribute, SampGetUserGroupAttributes,
    SampOpenUserObject, SampRemoveGroupMembershipFromUser, SampRemoveUserFromAllAliases,
    SampRemoveUserFromAllGroups, SampSetLogonHoursAttribute, SampSetUserGroupAttributes,
    SampSetUserPassword,
};
pub use crate::samsrv::alias::{
    SampAddMemberToAlias, SampGetMembersInAlias, SampRemoveAllMembersFromAlias,
    SampRemoveMemberFromAlias,
};
pub use crate::samsrv::domain::{
    SampCheckAccountNameInDomain, SampCreateAccountSid, SampRemoveAccountNameFromDomain,
    SampRemoveMemberFromAllAliases, SampSetAccountNameInDomain,
};
pub use crate::samsrv::registry::{
    SampRegCloseKey, SampRegCreateKey, SampRegDeleteKey, SampRegDeleteValue,
    SampRegEnumerateSubKey, SampRegEnumerateValue, SampRegOpenKey, SampRegQueryKeyInfo,
    SampRegQueryValue, SampRegSetValue,
};
pub use crate::samsrv::database::{
    SampCloseDbObject, SampGetObjectAttribute, SampInitDatabase, SampOpenDbObject,
    SampSetObjectAttribute, SampCreateDbObject, SampDeleteAccountDbObject,
    SampGetObjectAttributeString, SampSetObjectAttributeString, SampValidateDbObject,
};
pub use crate::samsrv::group::{
    SampOpenGroupObject, SampRemoveMemberFromGroup, SampAddMemberToGroup,
};
pub use crate::samsrv::display::{SampInitializeDisplayCache, SampFillDisplayCache, SampShutdownDisplayCache};
pub use crate::samsrv::setup::SampInitializeSAM;
pub use crate::samsrv::security::{
    SampCreateAccountDomainSD, SampCreateBuiltinDomainSD, SampCreateUserSD,
    SampCreateAliasSD, SampCreateGroupSD,
};

// ============================================================================
// Win32 / advapi32 / kernel32 imports (WINAPI = stdcall → extern "system").
// ============================================================================
pub type HKEY = HANDLE;
pub type HRSRC = HANDLE;
pub type HGLOBAL = HANDLE;

// RPC runtime types (rpcdce.h). RPC_STATUS = long; RPC_IF_HANDLE = void*.
pub type RPC_STATUS = c_long;
pub type RPC_IF_HANDLE = *mut c_void;
// The MIDL-generated server interface spec (sam_s.c) — resolved at final link.
extern "C" {
    pub static samr_v1_0_s_ifspec: RPC_IF_HANDLE;
}

extern "system" {
    // MIDL RPC allocators (__RPC_USER = stdcall).
    pub fn MIDL_user_allocate(size: SIZE_T) -> *mut c_void;
    pub fn MIDL_user_free(ptr: *mut c_void);

    // kernel32 module + resource loading (utils.c SampLoadString).
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HMODULE;
    pub fn FindResourceW(hModule: HMODULE, lpName: *const u16, lpType: *const u16) -> HRSRC;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    pub fn LockResource(hResData: HGLOBAL) -> *mut c_void;
    pub fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;

    // advapi32 registry (utils.c SampIsSetupRunning).
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        ulOptions: DWORD,
        samDesired: ACCESS_MASK,
        phkResult: *mut HKEY,
    ) -> LONG;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut u8,
        lpcbData: *mut DWORD,
    ) -> LONG;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;

    // advapi32 LSA policy (setup.c SampGetAccountDomainInfo).
    pub fn LsaOpenPolicy(
        SystemName: *mut _UNICODE_STRING,
        ObjectAttributes: *mut LSA_OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: *mut LSA_HANDLE,
    ) -> NTSTATUS;
    pub fn LsaQueryInformationPolicy(
        PolicyHandle: LSA_HANDLE,
        InformationClass: ULONG,
        Buffer: *mut *mut c_void,
    ) -> NTSTATUS;
    pub fn LsaFreeMemory(Buffer: *mut c_void) -> NTSTATUS;
    pub fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS;

    // Undocumented advapi32 crypto (samsrv.h) — samsrv computes no crypto itself.
    pub fn SystemFunction006(password: *const c_char, hash: *mut c_char) -> NTSTATUS;
    pub fn SystemFunction007(string: *mut _UNICODE_STRING, hash: *mut u8) -> NTSTATUS;
    pub fn SystemFunction013(input: *const u8, key: *const u8, output: *mut u8) -> NTSTATUS;

    // sddl/kernel32 (alias.c / domain.c SID-string helpers; WINAPI = stdcall).
    pub fn ConvertSidToStringSidW(Sid: PSID, StringSid: *mut LPWSTR) -> BOOL;
    pub fn LocalFree(hMem: LPVOID) -> LPVOID;

    // kernel32 process heap (samrpc.c midl_user_allocate/free).
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;

    // rpcrt4 server runtime (samrpc.c SampStartRpcServer; RPC_ENTRY = stdcall).
    pub fn RpcServerUseProtseqEpW(
        Protseq: LPWSTR,
        MaxCalls: ULONG,
        Endpoint: LPWSTR,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
    pub fn RpcServerRegisterIf(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut c_void,
        MgrEpv: *mut c_void,
    ) -> RPC_STATUS;
    pub fn RpcServerListen(MinimumCallThreads: ULONG, MaxCalls: ULONG, DontWait: ULONG)
        -> RPC_STATUS;
    pub fn RpcMgmtStopServerListening(ServerIfHandle: *mut c_void) -> RPC_STATUS;
}

// ConvertSidToStringSid (sddl.h) is #define'd to the W variant.
#[inline]
pub unsafe fn ConvertSidToStringSid(Sid: PSID, StringSid: *mut LPWSTR) -> BOOL {
    ConvertSidToStringSidW(Sid, StringSid)
}

// RtlEqualMemory(dst,src,len) (winnt.h) == (!memcmp(dst,src,len)).
#[inline]
pub unsafe fn RtlEqualMemory(dst: *const VOID, src: *const VOID, len: usize) -> ULONG {
    (memcmp(dst, src, len) == 0) as ULONG
}


// ============================================================================
// CRT (cdecl → extern "C"); resolve against ntdll/CRT at link.
// ============================================================================
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
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG;
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> c_int;
}

// ---- inline helpers for C macros / FORCEINLINEs ----
#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

#[inline]
pub unsafe fn RtlCopyMemory(destination: *mut VOID, source: *const VOID, length: usize) {
    memcpy(destination, source, length);
}
#[inline]
pub unsafe fn RtlMoveMemory(destination: *mut VOID, source: *const VOID, length: usize) {
    memmove(destination, source, length);
}

// L"Members" wide-string literal the c2rust output references as a value
// (database.c SampCreateDbObject Aliases-members subkey).
pub static L_Members: &[u16] = crate::wide!("Members");

// Win32 RtlXxxMemory aliases (#define CopyMemory RtlCopyMemory, etc.).
#[inline]
pub unsafe fn CopyMemory(d: *mut VOID, s: *const VOID, n: usize) {
    memcpy(d, s, n);
}
#[inline]
pub unsafe fn MoveMemory(d: *mut VOID, s: *const VOID, n: usize) {
    memmove(d, s, n);
}
#[inline]
pub unsafe fn ZeroMemory(d: *mut VOID, n: usize) {
    memset(d, 0, n);
}

// min(a,b) C macro (utils.c SampLoadString).
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

// LOWORD / MAKEINTRESOURCEW (winuser.h macros, utils.c).
#[inline]
pub fn LOWORD(l: u32) -> u16 {
    (l & 0xffff) as u16
}
#[inline]
pub fn MAKEINTRESOURCEW(i: u16) -> LPWSTR {
    i as usize as LPWSTR
}

// midl_user_allocate/free lowercase #defines (rpc runtime).
#[inline]
pub unsafe fn midl_user_allocate(len: SIZE_T) -> *mut VOID {
    MIDL_user_allocate(len)
}
#[inline]
pub unsafe fn midl_user_free(ptr: *mut VOID) {
    MIDL_user_free(ptr)
}

// ---- common NT/Win32 constants the translations bare-reference ----
pub const UNICODE_NULL: u16 = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const MAXULONG: ULONG = 0xFFFF_FFFF;

// ---- DUMMY_PSEH: SEH filters/handlers are dead code ----
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
