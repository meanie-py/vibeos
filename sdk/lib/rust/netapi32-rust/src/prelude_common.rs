//! netapi32-rust prelude — the single import surface the translated modules use.
//!
//! Bridges the gaps between the bindgen `types` island and the c2rust-expanded translation:
//!   1. the wine debug channel (wine/debug.h) — c2rust expands TRACE/ERR/FIXME to the raw
//!      `if dbch.flags & (1<<__WINE_DBCL_x) { ros_dbg_log(...) }` form; shimmed here (the
//!      lsasrv-rust pattern) so it resolves to a release-inert channel + the real variadic
//!      ros_dbg_log at link.
//!   2. CRT (cdecl) + winsock (stdcall) functions the translation calls but never declared.
//!   3. cast-#define consts / typedefs bindgen skipped — transcribed with a citation.
//!   4. bindgen enum members are type-prefixed; the C uses the bare name — aliased at the
//!      grind (added below as the cargo-check surfaces them).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// NOTE: this is the ISLAND-AGNOSTIC half of the prelude (everything that is not a
// bindgen type glob). The types glob lives in the per-island preludes that re-export
// this module: prelude (types), prelude_rpc (types_rpc), prelude_browser (types_browser).
// The private `use crate::types::*` below only resolves THIS file's own signatures; the
// base NT types (ULONG/PVOID/NTSTATUS/...) are re-exported from ntdll-sys by every island,
// so they are ONE identity regardless of which island a module imports.
use crate::types::*;
pub use core::ffi::{c_char, c_int, c_uchar, c_void};

// ---- 1. wine debug internals (wine/debug.h) — the lsasrv-rust pattern. ros_dbg_log is a real
// ReactOS export (variadic); declare it and let it resolve at link, preserving the debug path.
// NOT bindgen-emitted (wine/debug.h is parsed but not allowlisted) so no duplicate. ----
pub type __wine_debug_class = c_int;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: c_uchar,
    pub name: [c_char; 15],
}
pub const WINE_DBG_DEFAULT_FLAGS: c_uchar = 3;
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
// wine debug string formatters — release-inert (NDEBUG); return NULL like lsasrv/kernel32.
#[inline]
pub fn debugstr_w(_s: *const u16) -> *const i8 {
    core::ptr::null()
}
#[inline]
pub fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}

// ---- 2. CRT (cdecl, extern "C") — exported by msvcrt/ntdll, which netapi32.dll links.
// NEVER hand-write these (LLVM folds a byte-copy loop back into a self-call) — import them. ----
extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: c_int) -> core::ffi::c_long;
    pub fn atoi(s: *const c_char) -> c_int;
}

// ---- 3a. winsock (stdcall, extern "system" → `_Name@N` on i386) — from ws2_32. ----
extern "system" {
    pub fn htons(hostshort: u16) -> u16;
    pub fn htonl(hostlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn closesocket(s: SOCKET) -> c_int;
    pub fn WSAGetLastError() -> c_int;
}

// ---- 3b. common consts / typedefs bindgen skipped ----
pub type NET_API_STATUS = DWORD; // lmcons.h — `typedef DWORD NET_API_STATUS;`
pub const NULL: PVOID = core::ptr::null_mut();
pub const INVALID_SOCKET: SOCKET = !0; // winsock2.h — `(SOCKET)(~0)`
pub const USHRT_MAX: u16 = 0xFFFF; // limits.h

#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- 4. bridges added as the cargo-check grind reveals them (enum aliases, tail consts) ----

// 4a. undefined-type bridges (not emitted by bindgen for this crate) --------------------------
pub use core::ffi::c_ulonglong;
pub type size_t = usize;
pub type MIB_IPADDRTABLE = _MIB_IPADDRTABLE; // iphlpapi.h — bindgen emitted only the _-prefixed struct

// 4b. bindgen enum members are type-prefixed; the C uses the bare name. The prefixed consts are
//     already in scope via `pub use crate::types::*`; alias the bare names here. Prefixes read
//     off src/types_i386.rs (SAM/LSA/iphlpapi enum types).
pub use crate::types::{
    _ALIAS_INFORMATION_CLASS_AliasAdminCommentInformation as AliasAdminCommentInformation,
    _ALIAS_INFORMATION_CLASS_AliasGeneralInformation as AliasGeneralInformation,
    _ALIAS_INFORMATION_CLASS_AliasNameInformation as AliasNameInformation,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayGroup as DomainDisplayGroup,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayMachine as DomainDisplayMachine,
    _DOMAIN_DISPLAY_INFORMATION_DomainDisplayUser as DomainDisplayUser,
    _DOMAIN_INFORMATION_CLASS_DomainLockoutInformation as DomainLockoutInformation,
    _DOMAIN_INFORMATION_CLASS_DomainLogoffInformation as DomainLogoffInformation,
    _DOMAIN_INFORMATION_CLASS_DomainNameInformation as DomainNameInformation,
    _DOMAIN_INFORMATION_CLASS_DomainPasswordInformation as DomainPasswordInformation,
    _DOMAIN_INFORMATION_CLASS_DomainReplicationInformation as DomainReplicationInformation,
    _DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation as DomainServerRoleInformation,
    _GROUP_INFORMATION_CLASS_GroupAdminCommentInformation as GroupAdminCommentInformation,
    _GROUP_INFORMATION_CLASS_GroupAttributeInformation as GroupAttributeInformation,
    _GROUP_INFORMATION_CLASS_GroupGeneralInformation as GroupGeneralInformation,
    _GROUP_INFORMATION_CLASS_GroupNameInformation as GroupNameInformation,
    _INTERNAL_IF_OPER_STATUS_IF_OPER_STATUS_NON_OPERATIONAL as IF_OPER_STATUS_NON_OPERATIONAL,
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0ChangePassword as MsV1_0ChangePassword,
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation as PolicyAccountDomainInformation,
    _USER_INFORMATION_CLASS_UserAllInformation as UserAllInformation,
};

// 4c. tail consts bindgen skipped (cast-#define / value macros) — each header-cited ------------
pub const TIMEQ_FOREVER: ULONG = 0xFFFF_FFFF; // lmaccess.h — `#define TIMEQ_FOREVER ((ULONG)-1)`
pub const USER_MAXSTORAGE_UNLIMITED: ULONG = 0xFFFF_FFFF; // lmaccess.h — `((ULONG)-1)`
pub const MAX_PREFERRED_LENGTH: DWORD = 0xFFFF_FFFF; // lmcons.h — `#define MAX_PREFERRED_LENGTH ((DWORD)-1)`
pub const MAX_DOMAIN_NAME_LEN: DWORD = 128; // wine/iptypes.h — `#define MAX_DOMAIN_NAME_LEN 128`
pub const MAX_INTERFACE_NAME_LEN: DWORD = 256; // mprapi.h / ipifcons.h — `#define MAX_INTERFACE_NAME_LEN 256`
pub const STATUS_NONE_MAPPED: NTSTATUS = 0xC000_0073u32 as NTSTATUS; // ntstatus.h — `((NTSTATUS)0xC0000073)`
pub const MIB_IF_TYPE_TOKENRING: DWORD = 9; // ipifcons.h — `#define MIB_IF_TYPE_TOKENRING 9`
pub const IPPROTO_TCP: c_int = 6; // winsock.h — `#define IPPROTO_TCP 6`
pub const IPPROTO_UDP: c_int = 17; // winsock.h — `#define IPPROTO_UDP 17`
// winreg.h — `((HKEY)(LONG_PTR)(LONG)0x8000000N)`
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;


// 4c-bis. TRUE/FALSE are bindgen u32, but BOOL is c_int (i32). Shadow as BOOL so the pervasive
// `let x: BOOL = FALSE` / `x == FALSE` sites type-check (explicit item beats the types::* glob).
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const NRC_ACTSES: u8 = crate::types::NRC_ACTSES as u8;
pub const NRC_BADDR: u8 = crate::types::NRC_BADDR as u8;
pub const NRC_BRIDGE: u8 = crate::types::NRC_BRIDGE as u8;
pub const NRC_BUFLEN: u8 = crate::types::NRC_BUFLEN as u8;
pub const NRC_CANCEL: u8 = crate::types::NRC_CANCEL as u8;
pub const NRC_CANOCCR: u8 = crate::types::NRC_CANOCCR as u8;
pub const NRC_CMDCAN: u8 = crate::types::NRC_CMDCAN as u8;
pub const NRC_CMDTMO: u8 = crate::types::NRC_CMDTMO as u8;
pub const NRC_DUPENV: u8 = crate::types::NRC_DUPENV as u8;
pub const NRC_DUPNAME: u8 = crate::types::NRC_DUPNAME as u8;
pub const NRC_ENVNOTDEF: u8 = crate::types::NRC_ENVNOTDEF as u8;
pub const NRC_GOODRET: u8 = crate::types::NRC_GOODRET as u8;
pub const NRC_IFBUSY: u8 = crate::types::NRC_IFBUSY as u8;
pub const NRC_ILLCMD: u8 = crate::types::NRC_ILLCMD as u8;
pub const NRC_ILLNN: u8 = crate::types::NRC_ILLNN as u8;
pub const NRC_INCOMP: u8 = crate::types::NRC_INCOMP as u8;
pub const NRC_INUSE: u8 = crate::types::NRC_INUSE as u8;
pub const NRC_INVADDRESS: u8 = crate::types::NRC_INVADDRESS as u8;
pub const NRC_INVDDID: u8 = crate::types::NRC_INVDDID as u8;
pub const NRC_LOCKFAIL: u8 = crate::types::NRC_LOCKFAIL as u8;
pub const NRC_LOCTFUL: u8 = crate::types::NRC_LOCTFUL as u8;
pub const NRC_MAXAPPS: u8 = crate::types::NRC_MAXAPPS as u8;
pub const NRC_NAMCONF: u8 = crate::types::NRC_NAMCONF as u8;
pub const NRC_NAMERR: u8 = crate::types::NRC_NAMERR as u8;
pub const NRC_NAMTFUL: u8 = crate::types::NRC_NAMTFUL as u8;
pub const NRC_NOCALL: u8 = crate::types::NRC_NOCALL as u8;
pub const NRC_NORES: u8 = crate::types::NRC_NORES as u8;
pub const NRC_NORESOURCES: u8 = crate::types::NRC_NORESOURCES as u8;
pub const NRC_NOSAPS: u8 = crate::types::NRC_NOSAPS as u8;
pub const NRC_NOWILD: u8 = crate::types::NRC_NOWILD as u8;
pub const NRC_OPENERR: u8 = crate::types::NRC_OPENERR as u8;
pub const NRC_OSRESNOTAV: u8 = crate::types::NRC_OSRESNOTAV as u8;
pub const NRC_PENDING: u8 = crate::types::NRC_PENDING as u8;
pub const NRC_REMTFUL: u8 = crate::types::NRC_REMTFUL as u8;
pub const NRC_SABORT: u8 = crate::types::NRC_SABORT as u8;
pub const NRC_SCLOSED: u8 = crate::types::NRC_SCLOSED as u8;
pub const NRC_SNUMOUT: u8 = crate::types::NRC_SNUMOUT as u8;
pub const NRC_SYSTEM: u8 = crate::types::NRC_SYSTEM as u8;
pub const NRC_TOOMANY: u8 = crate::types::NRC_TOOMANY as u8;

// 4d. min()/Rtl*Memory/Copy*/Zero* value-macros the C used (never hand-write memcpy — delegate) -
#[inline]
pub fn min<T: Ord>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline]
pub unsafe fn CopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    memcpy(dst, src, len);
}
#[inline]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    memset(dst, 0, len);
}
#[inline]
pub unsafe fn ZeroMemory(dst: *mut c_void, len: usize) {
    memset(dst, 0, len);
}
// RtlGetProcessHeap() ≡ NtCurrentPeb()->ProcessHeap ≡ kernel32 GetProcessHeap() (same handle).
#[inline]
pub unsafe fn RtlGetProcessHeap() -> HANDLE {
    GetProcessHeap()
}
// nbt.c — `#define NBR_GETWORD(p) ntohs(*(WORD *)(p))`
#[inline]
pub unsafe fn NBR_GETWORD(p: *const c_void) -> u16 {
    ntohs((p as *const u16).read_unaligned())
}
// wine strcmpiW ≡ case-insensitive wide compare ≡ _wcsicmp.
#[inline]
pub unsafe fn strcmpiW(a: *const u16, b: *const u16) -> c_int {
    _wcsicmp(a, b)
}
// winsock2.h FD_ZERO/FD_SET macros over `fd_set { fd_count: u_int, fd_array: [SOCKET; 64] }`.
// (fn lives in the value namespace, so it coexists with the `FD_SET` type alias.)
#[inline]
pub unsafe fn FD_ZERO(set: *mut fd_set) {
    (*set).fd_count = 0;
}
#[inline]
pub unsafe fn FD_SET(fd: SOCKET, set: *mut fd_set) {
    let n = (*set).fd_count as usize;
    let mut i = 0usize;
    while i < n {
        if (*set).fd_array[i] == fd {
            return; // already present (matches the winsock macro's dedup)
        }
        i += 1;
    }
    if n < (*set).fd_array.len() {
        (*set).fd_array[n] = fd;
        (*set).fd_count += 1;
    }
}

// 4e. SAM (samlib, NTAPI = extern "system") — prototypes from ddk/ntsam.h --------------------
extern "system" {
    pub fn SamCloseHandle(SamHandle: SAM_HANDLE) -> NTSTATUS;
    pub fn SamFreeMemory(Buffer: PVOID) -> NTSTATUS;
    pub fn SamConnect(
        ServerName: PUNICODE_STRING,
        ServerHandle: PSAM_HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: POBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
    pub fn SamOpenDomain(
        ServerHandle: SAM_HANDLE,
        DesiredAccess: ACCESS_MASK,
        DomainId: PSID,
        DomainHandle: PSAM_HANDLE,
    ) -> NTSTATUS;
    pub fn SamLookupNamesInDomain(
        DomainHandle: SAM_HANDLE,
        Count: ULONG,
        Names: PUNICODE_STRING,
        RelativeIds: *mut PULONG,
        Use: *mut PSID_NAME_USE,
    ) -> NTSTATUS;
    pub fn SamLookupIdsInDomain(
        DomainHandle: SAM_HANDLE,
        Count: ULONG,
        RelativeIds: PULONG,
        Names: *mut PUNICODE_STRING,
        Use: *mut PSID_NAME_USE,
    ) -> NTSTATUS;
    pub fn SamQueryInformationDomain(
        DomainHandle: SAM_HANDLE,
        DomainInformationClass: _DOMAIN_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn SamQueryInformationGroup(
        GroupHandle: SAM_HANDLE,
        GroupInformationClass: _GROUP_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn SamQueryInformationAlias(
        AliasHandle: SAM_HANDLE,
        AliasInformationClass: _ALIAS_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn SamQueryInformationUser(
        UserHandle: SAM_HANDLE,
        UserInformationClass: _USER_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn SamSetInformationGroup(
        GroupHandle: SAM_HANDLE,
        GroupInformationClass: _GROUP_INFORMATION_CLASS,
        Buffer: PVOID,
    ) -> NTSTATUS;
    pub fn SamSetInformationAlias(
        AliasHandle: SAM_HANDLE,
        AliasInformationClass: _ALIAS_INFORMATION_CLASS,
        Buffer: PVOID,
    ) -> NTSTATUS;
    pub fn SamSetInformationUser(
        UserHandle: SAM_HANDLE,
        UserInformationClass: _USER_INFORMATION_CLASS,
        Buffer: PVOID,
    ) -> NTSTATUS;
    pub fn SamOpenUser(
        DomainHandle: SAM_HANDLE,
        DesiredAccess: ACCESS_MASK,
        UserId: ULONG,
        UserHandle: PSAM_HANDLE,
    ) -> NTSTATUS;
    pub fn SamOpenGroup(
        DomainHandle: SAM_HANDLE,
        DesiredAccess: ACCESS_MASK,
        GroupId: ULONG,
        GroupHandle: PSAM_HANDLE,
    ) -> NTSTATUS;
    pub fn SamOpenAlias(
        DomainHandle: SAM_HANDLE,
        DesiredAccess: ACCESS_MASK,
        AliasId: ULONG,
        AliasHandle: PSAM_HANDLE,
    ) -> NTSTATUS;
    pub fn SamDeleteGroup(GroupHandle: SAM_HANDLE) -> NTSTATUS;
    pub fn SamDeleteAlias(AliasHandle: SAM_HANDLE) -> NTSTATUS;
    pub fn SamDeleteUser(UserHandle: SAM_HANDLE) -> NTSTATUS;
    pub fn SamGetAliasMembership(
        DomainHandle: SAM_HANDLE,
        PassedCount: ULONG,
        Sids: *mut PSID,
        MembershipCount: PULONG,
        Aliases: *mut PULONG,
    ) -> NTSTATUS;
    pub fn SamGetMembersInGroup(
        GroupHandle: SAM_HANDLE,
        MemberIds: *mut PULONG,
        Attributes: *mut PULONG,
        MemberCount: PULONG,
    ) -> NTSTATUS;
    pub fn SamGetMembersInAlias(
        AliasHandle: SAM_HANDLE,
        MemberIds: *mut *mut PSID,
        MemberCount: PULONG,
    ) -> NTSTATUS;
    pub fn SamGetGroupsForUser(
        UserHandle: SAM_HANDLE,
        Groups: *mut PGROUP_MEMBERSHIP,
        MembershipCount: PULONG,
    ) -> NTSTATUS;
    pub fn SamAddMemberToGroup(
        GroupHandle: SAM_HANDLE,
        MemberId: ULONG,
        Attributes: ULONG,
    ) -> NTSTATUS;
    pub fn SamRemoveMemberFromGroup(GroupHandle: SAM_HANDLE, MemberId: ULONG) -> NTSTATUS;
    pub fn SamSetMemberAttributesOfGroup(
        GroupHandle: SAM_HANDLE,
        MemberId: ULONG,
        Attributes: ULONG,
    ) -> NTSTATUS;
    pub fn SamAddMemberToAlias(AliasHandle: SAM_HANDLE, MemberId: PSID) -> NTSTATUS;
    pub fn SamRemoveMemberFromAlias(AliasHandle: SAM_HANDLE, MemberId: PSID) -> NTSTATUS;
    pub fn SamQuerySecurityObject(
        ObjectHandle: SAM_HANDLE,
        SecurityInformation: SECURITY_INFORMATION,
        SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    ) -> NTSTATUS;
    pub fn SamQueryDisplayInformation(
        DomainHandle: SAM_HANDLE,
        DisplayInformation: _DOMAIN_DISPLAY_INFORMATION,
        Index: ULONG,
        EntryCount: ULONG,
        PreferredMaximumLength: ULONG,
        TotalAvailable: PULONG,
        TotalReturned: PULONG,
        ReturnedEntryCount: PULONG,
        SortedBuffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn SamGetDisplayEnumerationIndex(
        DomainHandle: SAM_HANDLE,
        DisplayInformation: _DOMAIN_DISPLAY_INFORMATION,
        Prefix: PUNICODE_STRING,
        Index: PULONG,
    ) -> NTSTATUS;
    pub fn SamEnumerateUsersInDomain(
        DomainHandle: SAM_HANDLE,
        EnumerationContext: PSAM_ENUMERATE_HANDLE,
        UserAccountControl: ULONG,
        Buffer: *mut PVOID,
        PreferedMaximumLength: ULONG,
        CountReturned: PULONG,
    ) -> NTSTATUS;
    pub fn SamEnumerateGroupsInDomain(
        DomainHandle: SAM_HANDLE,
        EnumerationContext: PSAM_ENUMERATE_HANDLE,
        Buffer: *mut PVOID,
        PreferedMaximumLength: ULONG,
        CountReturned: PULONG,
    ) -> NTSTATUS;
    pub fn SamEnumerateAliasesInDomain(
        DomainHandle: SAM_HANDLE,
        EnumerationContext: PSAM_ENUMERATE_HANDLE,
        Buffer: *mut PVOID,
        PreferedMaximumLength: ULONG,
        CountReturned: PULONG,
    ) -> NTSTATUS;
    pub fn SamCreateUser2InDomain(
        DomainHandle: SAM_HANDLE,
        AccountName: PUNICODE_STRING,
        AccountType: ULONG,
        DesiredAccess: ACCESS_MASK,
        UserHandle: PSAM_HANDLE,
        GrantedAccess: PULONG,
        RelativeId: PULONG,
    ) -> NTSTATUS;
    pub fn SamCreateGroupInDomain(
        DomainHandle: SAM_HANDLE,
        AccountName: PUNICODE_STRING,
        DesiredAccess: ACCESS_MASK,
        GroupHandle: PSAM_HANDLE,
        RelativeId: PULONG,
    ) -> NTSTATUS;
    pub fn SamCreateAliasInDomain(
        DomainHandle: SAM_HANDLE,
        AccountName: PUNICODE_STRING,
        DesiredAccess: ACCESS_MASK,
        AliasHandle: PSAM_HANDLE,
        RelativeId: PULONG,
    ) -> NTSTATUS;
}

// 4f. LSA (secur32/advapi, NTAPI = extern "system") — prototypes from psdk/ntsecapi.h --------
extern "system" {
    pub fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaFreeReturnBuffer(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS;
    pub fn LsaConnectUntrusted(LsaHandle: PHANDLE) -> NTSTATUS;
    pub fn LsaOpenPolicy(
        SystemName: PLSA_UNICODE_STRING,
        ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: PLSA_HANDLE,
    ) -> NTSTATUS;
    pub fn LsaQueryInformationPolicy(
        PolicyHandle: LSA_HANDLE,
        InformationClass: _POLICY_INFORMATION_CLASS,
        Buffer: *mut PVOID,
    ) -> NTSTATUS;
    pub fn LsaLookupNames(
        PolicyHandle: LSA_HANDLE,
        Count: ULONG,
        Names: PLSA_UNICODE_STRING,
        ReferencedDomains: *mut PLSA_REFERENCED_DOMAIN_LIST,
        Sids: *mut PLSA_TRANSLATED_SID,
    ) -> NTSTATUS;
    pub fn LsaLookupSids(
        PolicyHandle: LSA_HANDLE,
        Count: ULONG,
        Sids: *mut PSID,
        ReferencedDomains: *mut PLSA_REFERENCED_DOMAIN_LIST,
        Names: *mut PLSA_TRANSLATED_NAME,
    ) -> NTSTATUS;
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
        ProtocolStatus: *mut NTSTATUS,
    ) -> NTSTATUS;
}

// 4g. kernel32 (Win32 = extern "system") — prototypes from psdk/winbase.h / heapapi.h ---------
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn HeapSize(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPCVOID) -> SIZE_T;
    pub fn EnterCriticalSection(lpCriticalSection: *mut _RTL_CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut _RTL_CRITICAL_SECTION);
    pub fn InitializeCriticalSection(lpCriticalSection: *mut _RTL_CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut _RTL_CRITICAL_SECTION);
    pub fn GetTickCount() -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn ResetEvent(hEvent: HANDLE) -> BOOL;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn ResumeThread(hThread: HANDLE) -> DWORD;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn CreateThread(
        lpThreadAttributes: LPVOID,
        dwStackSize: SIZE_T,
        lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    pub fn CreateEventW(
        lpEventAttributes: LPVOID,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL;
}
// Interlocked{Increment,Decrement}: kernel32 exports them on i386, but they are compiler
// intrinsics with NO linkable symbol on amd64/arm64 — provide inline fns (the ws2_32/ws2help
// pattern). The +1/-1 restores the "value AFTER the op" contract (fetch_* return the prior value).
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

// 4h. advapi32 registry (Win32 = extern "system") — prototypes from psdk/winreg.h -------------
extern "system" {
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegOpenKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LSTATUS;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
    pub fn RegQueryValueExA(
        hKey: HKEY,
        lpValueName: LPCSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
}

// 4i. iphlpapi (Win32 = extern "system") — prototypes from psdk/iphlpapi.h --------------------
extern "system" {
    pub fn GetIfEntry(pIfRow: PMIB_IFROW) -> DWORD;
    pub fn GetIpAddrTable(pIpAddrTable: PMIB_IPADDRTABLE, pdwSize: PULONG, bOrder: BOOL) -> DWORD;
}

// 4j. ws2_32 socket calls the translation uses (stdcall = extern "system") --------------------
extern "system" {
    pub fn socket(af: c_int, type_: c_int, protocol: c_int) -> SOCKET;
    pub fn connect(s: SOCKET, name: *const c_void, namelen: c_int) -> c_int;
    pub fn select(
        nfds: c_int,
        readfds: *mut FD_SET,
        writefds: *mut FD_SET,
        exceptfds: *mut FD_SET,
        timeout: *const c_void,
    ) -> c_int;
    pub fn setsockopt(
        s: SOCKET,
        level: c_int,
        optname: c_int,
        optval: *const c_char,
        optlen: c_int,
    ) -> c_int;
    pub fn inet_addr(cp: *const c_char) -> u32;
    pub fn inet_ntoa(inaddr: in_addr) -> *mut c_char;
    pub fn gethostbyname(name: *const c_char) -> PHOSTENT;
    pub fn WSASocketA(
        af: c_int,
        type_: c_int,
        protocol: c_int,
        lpProtocolInfo: *mut c_void,
        g: u32,
        dwFlags: DWORD,
    ) -> SOCKET;
    pub fn WSASend(
        s: SOCKET,
        lpBuffers: *mut c_void,
        dwBufferCount: DWORD,
        lpNumberOfBytesSent: LPDWORD,
        dwFlags: DWORD,
        lpOverlapped: *mut c_void,
        lpCompletionRoutine: *mut c_void,
    ) -> c_int;
    pub fn WSARecv(
        s: SOCKET,
        lpBuffers: *mut c_void,
        dwBufferCount: DWORD,
        lpNumberOfBytesRecvd: LPDWORD,
        lpFlags: LPDWORD,
        lpOverlapped: *mut c_void,
        lpCompletionRoutine: *mut c_void,
    ) -> c_int;
    pub fn WSASendTo(
        s: SOCKET,
        lpBuffers: *mut c_void,
        dwBufferCount: DWORD,
        lpNumberOfBytesSent: LPDWORD,
        dwFlags: DWORD,
        lpTo: *const c_void,
        iToLen: c_int,
        lpOverlapped: *mut c_void,
        lpCompletionRoutine: *mut c_void,
    ) -> c_int;
    pub fn WSARecvFrom(
        s: SOCKET,
        lpBuffers: *mut c_void,
        dwBufferCount: DWORD,
        lpNumberOfBytesRecvd: LPDWORD,
        lpFlags: LPDWORD,
        lpFrom: *mut c_void,
        lpFromlen: *mut c_int,
        lpOverlapped: *mut c_void,
        lpCompletionRoutine: *mut c_void,
    ) -> c_int;
}

// 4k. CRT (cdecl = extern "C") ----------------------------------------------------------------
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn isdigit(c: c_int) -> c_int;
    pub fn isalnum(c: c_int) -> c_int;
}

// 4l. intra-crate helpers referenced across modules. The C compiled these together in one TU;
//     in Rust each module is its own scope, so re-export the shared #[no_mangle] helpers here.
pub use crate::netapi32::apibuf::{NetApiBufferAllocate, NetApiBufferFree, NetApiBufferReallocate};
pub use crate::netapi32::group_new::OpenGroupByName;
pub use crate::netapi32::nbcmdqueue::{
    NBCmdQueue, NBCmdQueueAdd, NBCmdQueueCancel, NBCmdQueueCancelAll, NBCmdQueueComplete,
    NBCmdQueueCreate, NBCmdQueueDestroy,
};
pub use crate::netapi32::nbnamecache::{
    NBNameCacheAddEntry, NBNameCacheCreate, NBNameCacheDestroy, NBNameCacheFindEntry,
};
pub use crate::netapi32::nbt::NetBTInit;
pub use crate::netapi32::netbios::{
    NetBIOSEnableAdapter, NetBIOSEnumAdapters, NetBIOSHangupSession, NetBIOSInit,
    NetBIOSNumAdapters, NetBIOSRegisterAdapter, NetBIOSRegisterTransport, NetBIOSShutdown,
};
pub use crate::netapi32::user::{g_EnumContextListHead, g_EnumContextListLock, DeltaTimeToSeconds};
pub use crate::netapi32::utils::{
    BuildSidFromSidAndRid, CopySidFromSidAndRid, GetAccountDomainSid, GetBuiltinDomainSid,
    OpenAccountDomain, OpenBuiltinDomain,
};
pub use crate::netapi32::wksta::NetpGetComputerName;
// NetpNtStatusToApiStatus is defined by the (now Rust) misc.rs as `#[no_mangle] extern "system"`,
// so this stays an extern rather than a `pub use`: misc.rs belongs to the types_rpc ISLAND, and
// re-exporting its Rust item here would drag an island-typed signature into the lm-island prelude.
// The decorated symbol (`_NetpNtStatusToApiStatus@4` on i386) resolves to that definition at link.
extern "system" {
    pub fn NetpNtStatusToApiStatus(Status: NTSTATUS) -> NET_API_STATUS;
}

// ---- 5. RPC runtime (rpcrt4.dll), ISLAND-AGNOSTIC. Both the types_rpc tail (srvsvc/wkssvc/
// atsvc/dssetup/netlogon) and browser.rs bind their MIDL clients by hand, so these live here
// rather than in one island prelude.
//
// Types are spelled STRUCTURALLY (c_long / *mut u16 / *mut c_void) instead of by their island
// alias, because this module can only see the lm island (`use crate::types::*`) — which has no
// RPC_*. Rust type aliases are transparent, so these unify with each island's RPC_STATUS /
// RPC_WSTR / RPC_BINDING_HANDLE at every call site. `RPC_ENTRY` is __stdcall -> extern "system".
//   rpcdce.h: RPC_STATUS RPC_ENTRY RpcStringBindingComposeW(RPC_WSTR ObjUuid, RPC_WSTR Protseq,
//             RPC_WSTR NetworkAddr, RPC_WSTR Endpoint, RPC_WSTR Options, RPC_WSTR *StringBinding)
extern "system" {
    pub fn RpcStringBindingComposeW(
        ObjUuid: *mut u16,
        Protseq: *mut u16,
        NetworkAddr: *mut u16,
        Endpoint: *mut u16,
        Options: *mut u16,
        StringBinding: *mut *mut u16,
    ) -> core::ffi::c_long;
    pub fn RpcBindingFromStringBindingW(
        StringBinding: *mut u16,
        Binding: *mut *mut c_void,
    ) -> core::ffi::c_long;
    pub fn RpcBindingFree(Binding: *mut *mut c_void) -> core::ffi::c_long;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> core::ffi::c_long;
}

// ---- 6. Rtl string-SIZE helpers. In C these are MACROS (ndk/rtlfuncs.h, NTOS_MODE_USER):
//     #define RtlUnicodeStringToAnsiSize(S) (NLS_MB_CODE_PAGE_TAG \
//         ? RtlxUnicodeStringToAnsiSize(S) : ((S)->Length + sizeof(UNICODE_NULL)) / sizeof(WCHAR))
// The ternary is purely an SBCS FAST PATH: when NlsMbCodePageTag is FALSE the arithmetic branch
// and RtlxUnicodeStringToAnsiSize return the same value, so the two forms are equivalent.
// ntdll exports the macro's own name as a forwarder to exactly that Rtlx* target
//     ntdll.spec:1228  @ stdcall RtlUnicodeStringToAnsiSize(ptr) RtlxUnicodeStringToAnsiSize
// so importing the export is faithful AND avoids a cross-DLL DATA import of NlsMbCodePageTag
// (which would need #[link(kind = "raw-dylib")] and is a far easier thing to get subtly wrong).
// Declaring these as plain externs is therefore correct — NOT a stub: a real ntdll implementation
// backs each one. (Never translate such a macro into a no-op fn — see AGENTS.md.)
//   ntdll.spec:656/1068/1228/1232 — all four are @ stdcall forwarders to their Rtlx* twin.
extern "system" {
    pub fn RtlUnicodeStringToAnsiSize(UnicodeString: *const UNICODE_STRING) -> ULONG;
    pub fn RtlUnicodeStringToOemSize(UnicodeString: *const UNICODE_STRING) -> ULONG;
    pub fn RtlOemStringToUnicodeSize(OemString: *const STRING) -> ULONG;
    pub fn RtlAnsiStringToUnicodeSize(AnsiString: *const STRING) -> ULONG;
}

// ---- 7. LSA policy-change notification (advapi32/lsa). netlogon.c registers for DNS-domain
// change notifications. ntsecapi.h:776/798 —
//   NTSTATUS NTAPI LsaRegisterPolicyChangeNotification(POLICY_NOTIFICATION_INFORMATION_CLASS,
//                                                      HANDLE);
// The info-class enum is island-typed (a C `int`), so it is spelled structurally here too.
extern "system" {
    pub fn LsaRegisterPolicyChangeNotification(
        InformationClass: c_int,
        NotificationEventHandle: HANDLE,
    ) -> NTSTATUS;
    pub fn LsaUnregisterPolicyChangeNotification(
        InformationClass: c_int,
        NotificationEventHandle: HANDLE,
    ) -> NTSTATUS;
}

// ---- rpcrt4: the C's `RpcExcept` handlers map a caught RPC exception through it (RPC_ENTRY = stdcall). ----
extern "system" {
    pub fn I_RpcMapWin32Status(Status: core::ffi::c_long) -> core::ffi::c_long;
}
