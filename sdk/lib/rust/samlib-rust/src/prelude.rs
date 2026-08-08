//! samlib-rust prelude — the single import surface the translated modules use.
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
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= UNICODE / macro aliases ====================================================
pub const UNICODE_NULL: u16 = 0;
// ntstatus.h value; not in the island because samlib pulls only the NDK subset.
pub const STATUS_PASSWORD_RESTRICTION: NTSTATUS = 0xC000006Cu32 as NTSTATUS;

// bindgen prefixes C enum members with their enum's tag. Alias, never re-declare, so these
// cannot drift from the island's values.
pub const DomainDisplayUser: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayUser;
pub const DomainDisplayMachine: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayMachine;
pub const DomainDisplayGroup: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayGroup;
pub const DomainDisplayOemUser: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayOemUser;
pub const DomainDisplayOemGroup: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayOemGroup;
pub const DomainDisplayServer: _DOMAIN_DISPLAY_INFORMATION = _DOMAIN_DISPLAY_INFORMATION_DomainDisplayServer;
pub const UserAllInformation: USER_INFORMATION_CLASS =
    _USER_INFORMATION_CLASS_UserAllInformation;
pub const UserInternal1Information: USER_INFORMATION_CLASS =
    _USER_INFORMATION_CLASS_UserInternal1Information;
pub const UserSetPasswordInformation: USER_INFORMATION_CLASS =
    _USER_INFORMATION_CLASS_UserSetPasswordInformation;

// winbase.h CopyMemory/ZeroMemory are MACROS onto RtlCopyMemory/RtlZeroMemory, which are in
// turn memcpy/memset. Keep them macros: an out-of-line no-op or forwarder is an optimization
// barrier (field manual §9.65), and NEVER hand-write the byte loop — LLVM folds it back into
// a call to itself (AGENTS.md).
#[macro_export]
macro_rules! CopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        $crate::prelude::memcpy($dst as *mut c_void, $src as *const c_void, $len as usize)
    };
}
#[macro_export]
macro_rules! RtlCopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        $crate::prelude::memcpy($dst as *mut c_void, $src as *const c_void, $len as usize)
    };
}
// The zeroing macros live in `win32macros-rust`, ported once from the header and
// known-answer tested there. Re-exported under the same names, so no call site changed.
pub use win32macros_rust::ZeroMemory;
pub use crate::{CopyMemory, RtlCopyMemory};

// ================= Win32 / RPC imports ========================================================
// Declared, never defined: rpcrt4, advapi32, kernel32 (samlib's add_importlibs).
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
    pub fn GetStringTypeW(
        dwInfoType: DWORD,
        lpSrcStr: *const u16,
        cchSrc: c_int,
        lpCharType: *mut u16,
    ) -> BOOL;
    pub fn I_RpcMapWin32Status(status: RPC_STATUS) -> c_long;
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
    pub fn RpcStringFreeW(String: *mut *mut u16) -> RPC_STATUS;
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS;
    // Diverges (raises a Windows exception); the C declares no noreturn, so neither do we.
    pub fn RpcRaiseException(exception: RPC_STATUS);
    // advapi32 crypto helpers (undocumented but exported): DES/MD4 password blobs.
    // Parameter types copied from samlib.c's own local prototypes (lines 37-52), NOT guessed:
    // 006 is LPCSTR/LPSTR (i8), but 007 and 012 are LPBYTE/const BYTE* (u8). Mixing those up
    // still links — every one is a pointer — and silently mistypes the password buffers.
    pub fn SystemFunction006(password: *const c_char, hash: *mut c_char) -> NTSTATUS;
    pub fn SystemFunction007(string: PUNICODE_STRING, hash: *mut u8) -> NTSTATUS;
    pub fn SystemFunction012(data: *const u8, key: *const u8, output: *mut u8) -> NTSTATUS;
}

// ================= sam.idl MIDL client stubs ==================================================
// Generated into sam_c.c by add_rpc_files(client ...) and linked alongside the Rust, exactly
// as wlansvc_c.c is for wlanapi. Prototypes are BINDGEN-DERIVED from sam_c.h rather than hand
// transcribed — 68 of them, and a hand-typed parameter list is a silent ABI bug waiting to
// happen. `extern "system"` (not the "stdcall" bindgen emits for i386) so one declaration is
// correct on all three arches; these are plain calls, not vtbl slots, so the type-level
// stdcall/system distinction that forces abi_enforce's cfg-dual does not apply.
extern "system" {
    pub fn SamrConnect(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    ) -> NTSTATUS;
    pub fn SamrCloseHandle(SamHandle: *mut SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrSetSecurityObject(
    ObjectHandle: SAMPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSAMPR_SR_SECURITY_DESCRIPTOR,
    ) -> NTSTATUS;
    pub fn SamrQuerySecurityObject(
    ObjectHandle: SAMPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PSAMPR_SR_SECURITY_DESCRIPTOR,
    ) -> NTSTATUS;
    pub fn SamrShutdownSamServer(ServerHandle: SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrLookupDomainInSamServer(
    ServerHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    DomainId: *mut PRPC_SID,
    ) -> NTSTATUS;
    pub fn SamrEnumerateDomainsInSamServer(
    ServerHandle: SAMPR_HANDLE,
    EnumerationContext: *mut ULONG,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: ULONG,
    CountReturned: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrOpenDomain(
    ServerHandle: SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    DomainId: PRPC_SID,
    DomainHandle: *mut SAMPR_HANDLE,
    ) -> NTSTATUS;
    pub fn SamrQueryInformationDomain(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetInformationDomain(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    DomainInformation: PSAMPR_DOMAIN_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrCreateGroupInDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    GroupHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrEnumerateGroupsInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut ULONG,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: ULONG,
    CountReturned: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrCreateUserInDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    UserHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrEnumerateUsersInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut ULONG,
    UserAccountControl: ULONG,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: ULONG,
    CountReturned: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrCreateAliasInDomain(
    DomainHandle: SAMPR_HANDLE,
    AccountName: PRPC_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    AliasHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrEnumerateAliasesInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut ULONG,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: ULONG,
    CountReturned: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrGetAliasMembership(
    DomainHandle: SAMPR_HANDLE,
    SidArray: PSAMPR_PSID_ARRAY,
    Membership: PSAMPR_ULONG_ARRAY,
    ) -> NTSTATUS;
    pub fn SamrLookupNamesInDomain(
    DomainHandle: SAMPR_HANDLE,
    Count: ULONG,
    Names: *mut RPC_UNICODE_STRING,
    RelativeIds: PSAMPR_ULONG_ARRAY,
    Use: PSAMPR_ULONG_ARRAY,
    ) -> NTSTATUS;
    pub fn SamrLookupIdsInDomain(
    DomainHandle: SAMPR_HANDLE,
    Count: ULONG,
    RelativeIds: *mut ULONG,
    Names: PSAMPR_RETURNED_USTRING_ARRAY,
    Use: PSAMPR_ULONG_ARRAY,
    ) -> NTSTATUS;
    pub fn SamrOpenGroup(
    DomainHandle: SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    GroupId: ULONG,
    GroupHandle: *mut SAMPR_HANDLE,
    ) -> NTSTATUS;
    pub fn SamrQueryInformationGroup(
    GroupHandle: SAMPR_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_GROUP_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetInformationGroup(
    GroupHandle: SAMPR_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: PSAMPR_GROUP_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrAddMemberToGroup(
    GroupHandle: SAMPR_HANDLE,
    MemberId: ULONG,
    Attributes: ULONG,
    ) -> NTSTATUS;
    pub fn SamrDeleteGroup(GroupHandle: *mut SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrRemoveMemberFromGroup(GroupHandle: SAMPR_HANDLE, MemberId: ULONG) -> NTSTATUS;
    pub fn SamrGetMembersInGroup(
    GroupHandle: SAMPR_HANDLE,
    Members: *mut PSAMPR_GET_MEMBERS_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetMemberAttributesOfGroup(
    GroupHandle: SAMPR_HANDLE,
    MemberId: ULONG,
    Attributes: ULONG,
    ) -> NTSTATUS;
    pub fn SamrOpenAlias(
    DomainHandle: SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    AliasId: ULONG,
    AliasHandle: *mut SAMPR_HANDLE,
    ) -> NTSTATUS;
    pub fn SamrQueryInformationAlias(
    AliasHandle: SAMPR_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_ALIAS_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetInformationAlias(
    AliasHandle: SAMPR_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: PSAMPR_ALIAS_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrDeleteAlias(AliasHandle: *mut SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrAddMemberToAlias(AliasHandle: SAMPR_HANDLE, MemberId: PRPC_SID) -> NTSTATUS;
    pub fn SamrRemoveMemberFromAlias(AliasHandle: SAMPR_HANDLE, MemberId: PRPC_SID) -> NTSTATUS;
    pub fn SamrGetMembersInAlias(
    AliasHandle: SAMPR_HANDLE,
    Members: PSAMPR_PSID_ARRAY_OUT,
    ) -> NTSTATUS;
    pub fn SamrOpenUser(
    DomainHandle: SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    UserId: ULONG,
    UserHandle: *mut SAMPR_HANDLE,
    ) -> NTSTATUS;
    pub fn SamrDeleteUser(UserHandle: *mut SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrQueryInformationUser(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetInformationUser(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: PSAMPR_USER_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrChangePasswordUser(
    UserHandle: SAMPR_HANDLE,
    LmPresent: ::core::ffi::c_uchar,
    OldLmEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD,
    NewLmEncryptedWithOldLm: PENCRYPTED_LM_OWF_PASSWORD,
    NtPresent: ::core::ffi::c_uchar,
    OldNtEncryptedWithNewNt: PENCRYPTED_NT_OWF_PASSWORD,
    NewNtEncryptedWithOldNt: PENCRYPTED_NT_OWF_PASSWORD,
    NtCrossEncryptionPresent: ::core::ffi::c_uchar,
    NewNtEncryptedWithNewLm: PENCRYPTED_NT_OWF_PASSWORD,
    LmCrossEncryptionPresent: ::core::ffi::c_uchar,
    NewLmEncryptedWithNewNt: PENCRYPTED_LM_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn SamrGetGroupsForUser(
    UserHandle: SAMPR_HANDLE,
    Groups: *mut PSAMPR_GET_GROUPS_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrQueryDisplayInformation(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Index: ULONG,
    EntryCount: ULONG,
    PreferredMaximumLength: ULONG,
    TotalAvailable: *mut ULONG,
    TotalReturned: *mut ULONG,
    Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrGetDisplayEnumerationIndex(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Prefix: PRPC_UNICODE_STRING,
    Index: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrTestPrivateFunctionsDomain(DomainHandle: SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrTestPrivateFunctionsUser(UserHandle: SAMPR_HANDLE) -> NTSTATUS;
    pub fn SamrGetUserDomainPasswordInformation(
    UserHandle: SAMPR_HANDLE,
    PasswordInformation: PUSER_DOMAIN_PASSWORD_INFORMATION,
    ) -> NTSTATUS;
    pub fn SamrRemoveMemberFromForeignDomain(
    DomainHandle: SAMPR_HANDLE,
    MemberSid: PRPC_SID,
    ) -> NTSTATUS;
    pub fn SamrQueryInformationDomain2(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrQueryInformationUser2(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrQueryDisplayInformation2(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Index: ULONG,
    EntryCount: ULONG,
    PreferredMaximumLength: ULONG,
    TotalAvailable: *mut ULONG,
    TotalReturned: *mut ULONG,
    Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrGetDisplayEnumerationIndex2(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Prefix: PRPC_UNICODE_STRING,
    Index: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrCreateUser2InDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    AccountType: ULONG,
    DesiredAccess: ACCESS_MASK,
    UserHandle: *mut SAMPR_HANDLE,
    GrantedAccess: *mut ULONG,
    RelativeId: *mut ULONG,
    ) -> NTSTATUS;
    pub fn SamrQueryDisplayInformation3(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Index: ULONG,
    EntryCount: ULONG,
    PreferredMaximumLength: ULONG,
    TotalAvailable: *mut ULONG,
    TotalReturned: *mut ULONG,
    Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrAddMultipleMembersToAlias(
    AliasHandle: SAMPR_HANDLE,
    MembersBuffer: PSAMPR_PSID_ARRAY,
    ) -> NTSTATUS;
    pub fn SamrRemoveMultipleMembersFromAlias(
    AliasHandle: SAMPR_HANDLE,
    MembersBuffer: PSAMPR_PSID_ARRAY,
    ) -> NTSTATUS;
    pub fn SamrOemChangePasswordUser2(
    BindingHandle: handle_t,
    ServerName: PRPC_STRING,
    UserName: PRPC_STRING,
    NewPasswordEncryptedWithOldLm: PSAMPR_ENCRYPTED_USER_PASSWORD,
    OldLmOwfPasswordEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn SamrUnicodeChangePasswordUser2(
    BindingHandle: handle_t,
    ServerName: PRPC_UNICODE_STRING,
    UserName: PRPC_UNICODE_STRING,
    NewPasswordEncryptedWithOldNt: PSAMPR_ENCRYPTED_USER_PASSWORD,
    OldNtOwfPasswordEncryptedWithNewNt: PENCRYPTED_NT_OWF_PASSWORD,
    LmPresent: ::core::ffi::c_uchar,
    NewPasswordEncryptedWithOldLm: PSAMPR_ENCRYPTED_USER_PASSWORD,
    OldLmOwfPasswordEncryptedWithNewNt: PENCRYPTED_LM_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn SamrGetDomainPasswordInformation(
    BindingHandle: handle_t,
    Unused: PRPC_UNICODE_STRING,
    PasswordInformation: PUSER_DOMAIN_PASSWORD_INFORMATION,
    ) -> NTSTATUS;
    pub fn SamrConnect2(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
    ) -> NTSTATUS;
    pub fn SamrSetInformationUser2(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: PSAMPR_USER_INFO_BUFFER,
    ) -> NTSTATUS;
    pub fn SamrSetBootKeyInformation(BindingHandle: handle_t) -> NTSTATUS;
    pub fn SamrGetBootKeyInformation(BindingHandle: handle_t) -> NTSTATUS;
    pub fn SamrConnect3(BindingHandle: handle_t) -> NTSTATUS;
    pub fn SamrConnect4(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    ClientRevision: ULONG,
    DesiredAccess: ACCESS_MASK,
    ) -> NTSTATUS;
    pub fn SamrUnicodeChangePasswordUser3(BindingHandle: handle_t) -> NTSTATUS;
    pub fn SamrConnect5(
    ServerName: PSAMPR_SERVER_NAME,
    DesiredAccess: ACCESS_MASK,
    InVersion: ULONG,
    InRevisionInfo: *mut SAMPR_REVISION_INFO,
    OutVersion: *mut ULONG,
    OutRevisionInfo: *mut SAMPR_REVISION_INFO,
    ServerHandle: *mut SAMPR_HANDLE,
    ) -> NTSTATUS;
    pub fn SamrRidToSid(ObjectHandle: SAMPR_HANDLE, Rid: ULONG, Sid: *mut PRPC_SID) -> NTSTATUS;
    pub fn SamrSetDSRMPassword(
    BindingHandle: handle_t,
    Unused: PRPC_UNICODE_STRING,
    UserId: ULONG,
    EncryptedNtOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn SamrValidatePassword(
    Handle: handle_t,
    ValidationType: PASSWORD_POLICY_VALIDATION_TYPE,
    InputArg: PSAM_VALIDATE_INPUT_ARG,
    OutputArg: *mut PSAM_VALIDATE_OUTPUT_ARG,
    ) -> NTSTATUS;
}

// A wide string literal, NUL-terminated, with one stable address for the program's life.
// `static` not `const`: a const is inlined at each use site, so `.as_ptr()` would hand the
// callee the address of a temporary. ASCII-only, so a non-ASCII literal fails at COMPILE time
// rather than silently truncating.
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
