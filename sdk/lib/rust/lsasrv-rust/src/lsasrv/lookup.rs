use crate::prelude::*;
use crate::lsasrv::utils::LsapLoadString;
use crate::lsasrv::database::{


    NtAuthority, NullSidAuthority, WorldSidAuthority, LocalSidAuthority, CreatorSidAuthority,
    BuiltinDomainSid, AccountDomainSid, BuiltinDomainName, AccountDomainName,
};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    fn TRACE(format: *const u8, ...);
}
// Win32 APIs are stdcall on i386 (_Name@N) — a cdecl decl leaves _Name undefined at link.
unsafe extern "system" {
    fn ConvertSidToStringSidW(sid: PSID, string: *mut LPWSTR) -> BOOL;
    fn LocalFree(h: *mut core::ffi::c_void) -> HLOCAL;
}
unsafe extern "system" {
    fn MIDL_user_allocate(size: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

pub static mut WellKnownSidListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

// Well-known SID globals — defined here (was `PSID LsapWorldSid = NULL;` in C
// lookup.c), exported unmangled so the still-C TUs (lsarpc.c) and Rust siblings
// (authpackage.rs) bind them by decorated C name.
#[no_mangle]
pub static mut LsapWorldSid: PSID = NULL;
#[no_mangle]
pub static mut LsapNetworkSid: PSID = NULL;
#[no_mangle]
pub static mut LsapBatchSid: PSID = NULL;
#[no_mangle]
pub static mut LsapInteractiveSid: PSID = NULL;
#[no_mangle]
pub static mut LsapServiceSid: PSID = NULL;
#[no_mangle]
pub static mut LsapLocalSystemSid: PSID = NULL;
#[no_mangle]
pub static mut LsapAdministratorsSid: PSID = NULL;

pub type PSAMPR_SERVER_NAME = *mut u16;
pub type SAMPR_HANDLE = *mut core::ffi::c_void;

#[repr(C)]
pub struct _SAMPR_RETURNED_USTRING_ARRAY {
    pub Count: ULONG,
    pub Element: PRPC_UNICODE_STRING,
}
pub type PSAMPR_RETURNED_USTRING_ARRAY = *mut _SAMPR_RETURNED_USTRING_ARRAY;

#[repr(C)]
pub struct _WELL_KNOWN_SID {
    pub ListEntry: LIST_ENTRY,
    pub Sid: PSID,
    pub AccountName: _UNICODE_STRING,
    pub DomainName: _UNICODE_STRING,
    pub Use: SID_NAME_USE,
}
pub type WELL_KNOWN_SID = _WELL_KNOWN_SID;
pub type PWELL_KNOWN_SID = *mut _WELL_KNOWN_SID;
#[no_mangle]
pub unsafe extern "C" fn LsapCreateSid(
    IdentifierAuthority: PSID_IDENTIFIER_AUTHORITY,
    SubAuthorityCount: UCHAR,
    SubAuthorities: PULONG,
    AccountName: PWSTR,
    DomainName: PWSTR,
    Use: SID_NAME_USE,
    SidPtr: *mut PSID,
) -> BOOLEAN {
    let AccountNameLength = wcslen(AccountName as *const u16) as SIZE_T;
    let DomainNameLength = wcslen(DomainName as *const u16) as SIZE_T;
    if AccountNameLength > UNICODE_STRING_MAX_CHARS as SIZE_T
        || DomainNameLength > UNICODE_STRING_MAX_CHARS as SIZE_T
    {
        return 0u8;
    }

    let SidEntry = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        core::mem::size_of::<WELL_KNOWN_SID>() as SIZE_T,
    ) as PWELL_KNOWN_SID;
    if SidEntry.is_null() {
        return 0u8;
    }

    InitializeListHead(&raw mut (*SidEntry).ListEntry);

    (*SidEntry).Sid = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0u32,
        RtlLengthRequiredSid(SubAuthorityCount as ULONG) as SIZE_T,
    ) as PSID;
    if (*SidEntry).Sid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, SidEntry as PVOID);
        return 0u8;
    }

    RtlInitializeSid((*SidEntry).Sid, IdentifierAuthority, SubAuthorityCount);

    let mut i: ULONG = 0u32;
    while i < SubAuthorityCount as ULONG {
        let p = RtlSubAuthoritySid((*SidEntry).Sid, i);
        *p = *SubAuthorities.add(i as usize);
        i = i.wrapping_add(1);
    }

    (*SidEntry).AccountName.Length = ((AccountNameLength as u16).wrapping_mul(core::mem::size_of::<u16>() as u16)) as u16;
    (*SidEntry).AccountName.MaximumLength = (*SidEntry).AccountName.Length.wrapping_add(core::mem::size_of::<u16>() as u16);
    (*SidEntry).AccountName.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0u32,
        (*SidEntry).AccountName.MaximumLength as SIZE_T,
    ) as PWSTR;
    if (*SidEntry).AccountName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, (*SidEntry).Sid as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, SidEntry as PVOID);
        return 0u8;
    }

    wcscpy(
        (*SidEntry).AccountName.Buffer as *mut u16,
        AccountName as *const u16,
    );

    (*SidEntry).DomainName.Length = ((DomainNameLength as u16).wrapping_mul(core::mem::size_of::<u16>() as u16)) as u16;
    (*SidEntry).DomainName.MaximumLength = (*SidEntry).DomainName.Length.wrapping_add(core::mem::size_of::<u16>() as u16);
    (*SidEntry).DomainName.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0u32,
        (*SidEntry).DomainName.MaximumLength as SIZE_T,
    ) as PWSTR;
    if (*SidEntry).DomainName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, (*SidEntry).AccountName.Buffer as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, (*SidEntry).Sid as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, SidEntry as PVOID);
        return 0u8;
    }

    wcscpy(
        (*SidEntry).DomainName.Buffer as *mut u16,
        DomainName as *const u16,
    );

    (*SidEntry).Use = Use;

    InsertTailList(
        core::ptr::addr_of_mut!(WellKnownSidListHead),
        &raw mut (*SidEntry).ListEntry,
    );

    if !SidPtr.is_null() {
        *SidPtr = (*SidEntry).Sid;
    }

    return 1u8;
}
#[no_mangle]
pub unsafe extern "C" fn LsapInitSids() -> NTSTATUS {
    let mut szAccountName: [u16; 80] = [0u16; 80];
    let mut szDomainName: [u16; 80] = [0u16; 80];
    let mut SubAuthorities: [u32; 8] = [0u32; 8];
    InitializeListHead(&raw mut WellKnownSidListHead);
    let hInstance = GetModuleHandleW(
        core::mem::transmute::<&[u8; 22], &[u16; 11]>(b"l\0s\0a\0s\0r\0v\0.\0d\0l\0l\0\0\0").as_ptr(),
    );
    LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szAccountName.as_mut_ptr(), 80);
    LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szDomainName.as_mut_ptr(), 80);
    LsapCreateSid(
        &raw mut NtAuthority,
        0u8,
        core::ptr::null_mut(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeDomain,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_NULL_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_NULL_RID as u32;
    LsapCreateSid(
        &raw mut NullSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_WORLD_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_WORLD_RID as u32;
    LsapCreateSid(
        &raw mut WorldSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        &raw mut LsapWorldSid,
    );
    LsapLoadString(hInstance, IDS_LOCAL_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_LOCAL_RID as u32;
    LsapCreateSid(
        &raw mut LocalSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_CREATOR_OWNER_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_CREATOR_OWNER_RID as u32;
    LsapCreateSid(
        &raw mut CreatorSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_CREATOR_GROUP_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_CREATOR_GROUP_RID as u32;
    LsapCreateSid(
        &raw mut CreatorSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_CREATOR_OWNER_SERVER_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_CREATOR_OWNER_SERVER_RID as u32;
    LsapCreateSid(
        &raw mut CreatorSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_CREATOR_GROUP_SERVER_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_CREATOR_GROUP_SERVER_RID as u32;
    LsapCreateSid(
        &raw mut CreatorSidAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        core::mem::transmute::<&[u8; 2], &[u16; 1]>(b"\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_DIALUP_RID as u32, szAccountName.as_mut_ptr(), 80);
    LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szDomainName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_DIALUP_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_NETWORK_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_NETWORK_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        &raw mut LsapNetworkSid,
    );
    LsapLoadString(hInstance, IDS_BATCH_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BATCH_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        &raw mut LsapBatchSid,
    );
    LsapLoadString(hInstance, IDS_INTERACTIVE_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_INTERACTIVE_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        &raw mut LsapInteractiveSid,
    );
    LsapLoadString(hInstance, IDS_SERVICE_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_SERVICE_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        &raw mut LsapServiceSid,
    );
    LsapLoadString(hInstance, IDS_ANONYMOUS_LOGON_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_ANONYMOUS_LOGON_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_PROXY_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_PROXY_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ENTERPRISE_CONTROLLERS_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_ENTERPRISE_CONTROLLERS_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_PRINCIPAL_SELF_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_PRINCIPAL_SELF_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_AUTHENTICATED_USER_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_AUTHENTICATED_USER_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_RESTRICTED_CODE_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_RESTRICTED_CODE_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_TERMINAL_SERVER_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_TERMINAL_SERVER_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_REMOTE_LOGON_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_REMOTE_LOGON_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_THIS_ORGANIZATION_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_THIS_ORGANIZATION_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_LOCAL_SYSTEM_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_LOCAL_SYSTEM_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        &raw mut LsapLocalSystemSid,
    );
    LsapLoadString(hInstance, IDS_LOCAL_SERVICE_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_LOCAL_SERVICE_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        core::mem::transmute::<&[u8; 26], &[u16; 13]>(b"L\0O\0C\0A\0L\0S\0E\0R\0V\0I\0C\0E\0\0\0").as_ptr() as PWSTR,
        core::mem::transmute::<&[u8; 26], &[u16; 13]>(b"N\0T\0 \0A\0U\0T\0H\0O\0R\0I\0T\0Y\0\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_NETWORK_SERVICE_RID as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_NETWORK_SERVICE_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        core::mem::transmute::<&[u8; 30], &[u16; 15]>(b"N\0E\0T\0W\0O\0R\0K\0S\0E\0R\0V\0I\0C\0E\0\0\0").as_ptr() as PWSTR,
        core::mem::transmute::<&[u8; 26], &[u16; 13]>(b"N\0T\0 \0A\0U\0T\0H\0O\0R\0I\0T\0Y\0\0\0").as_ptr() as PWSTR,
        SidTypeWellKnownGroup,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_BUILTIN_DOMAIN_RID as u32, szAccountName.as_mut_ptr(), 80);
    LsapLoadString(hInstance, IDS_BUILTIN_DOMAIN_RID as u32, szDomainName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        1u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeDomain,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_ADMINS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_ADMINS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        &raw mut LsapAdministratorsSid,
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_USERS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_USERS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_GUESTS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_GUESTS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_POWER_USERS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_POWER_USERS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_ACCOUNT_OPS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_ACCOUNT_OPS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_SYSTEM_OPS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_SYSTEM_OPS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_PRINT_OPS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_PRINT_OPS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_BACKUP_OPS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_BACKUP_OPS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_REPLICATOR as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_REPLICATOR as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_RAS_SERVERS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_RAS_SERVERS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_PREW2KCOMPACCESS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_PREW2KCOMPACCESS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_REMOTE_DESKTOP_USERS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    LsapLoadString(hInstance, IDS_ALIAS_RID_NETWORK_CONFIGURATION_OPS as u32, szAccountName.as_mut_ptr(), 80);
    SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID as u32;
    SubAuthorities[1] = DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS as u32;
    LsapCreateSid(
        &raw mut NtAuthority,
        2u8,
        SubAuthorities.as_mut_ptr(),
        szAccountName.as_mut_ptr(),
        szDomainName.as_mut_ptr(),
        SidTypeAlias,
        core::ptr::null_mut(),
    );
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupWellKnownSid(sid: PSID) -> *mut _WELL_KNOWN_SID {
    let mut list_entry = WellKnownSidListHead.Flink;
    while list_entry as *const _LIST_ENTRY != core::ptr::addr_of!(WellKnownSidListHead) {
        let ptr = (list_entry as usize)
            .wrapping_sub(core::mem::offset_of!(_WELL_KNOWN_SID, ListEntry))
            as *mut _WELL_KNOWN_SID;
        if RtlEqualSid(sid, (*ptr).Sid) != 0u8 {
            return ptr;
        }
        list_entry = (*list_entry).Flink;
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupIsolatedWellKnownName(
    AccountName: *mut _UNICODE_STRING,
) -> *mut _WELL_KNOWN_SID {
    let mut ListEntry: *mut _LIST_ENTRY = WellKnownSidListHead.Flink;
    let head_ptr: *mut _LIST_ENTRY = core::ptr::addr_of_mut!(WellKnownSidListHead);
    while ListEntry != head_ptr {
        let offset = core::mem::offset_of!(_WELL_KNOWN_SID, ListEntry);
        let Ptr: *mut _WELL_KNOWN_SID =
            (ListEntry as *mut u8).offset(-(offset as isize)) as *mut _WELL_KNOWN_SID;
        if RtlEqualUnicodeString(
            AccountName as *const _UNICODE_STRING,
            core::ptr::addr_of_mut!((*Ptr).AccountName) as *const _UNICODE_STRING,
            1u8,
        ) != 0
        {
            return Ptr;
        }
        ListEntry = (*ListEntry).Flink;
    }
    core::ptr::null_mut::<_WELL_KNOWN_SID>()
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupFullyQualifiedWellKnownName(
    AccountName: PUNICODE_STRING,
    DomainName: PUNICODE_STRING,
) -> PWELL_KNOWN_SID {
    let mut list_entry: *mut LIST_ENTRY = WellKnownSidListHead.Flink;
    while list_entry != core::ptr::addr_of_mut!(WellKnownSidListHead) {
        let offset = core::mem::offset_of!(WELL_KNOWN_SID, ListEntry);
        let ptr = ((list_entry as usize) - offset) as *mut WELL_KNOWN_SID;
        if RtlEqualUnicodeString(AccountName as *const _UNICODE_STRING, &raw mut (*ptr).AccountName as *const _UNICODE_STRING, 1u8) != 0
            && RtlEqualUnicodeString(DomainName as *const _UNICODE_STRING, &raw mut (*ptr).DomainName as *const _UNICODE_STRING, 1u8) != 0
        {
            return ptr;
        }
        list_entry = (*list_entry).Flink;
    }
    core::ptr::null_mut()
}

unsafe fn LsapSplitNames(
    Count: DWORD,
    Names: PRPC_UNICODE_STRING,
    DomainNames: *mut PRPC_UNICODE_STRING,
    AccountNames: *mut PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut DomainsBuffer: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let mut AccountsBuffer: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let mut DomainLength: u32;
    let mut AccountLength: u32;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DomainsBuffer = MIDL_user_allocate(
        Count as usize * core::mem::size_of::<RPC_UNICODE_STRING>(),
    ) as PRPC_UNICODE_STRING;
    if DomainsBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        AccountsBuffer = MIDL_user_allocate(
            Count as usize * core::mem::size_of::<RPC_UNICODE_STRING>(),
        ) as PRPC_UNICODE_STRING;
        if AccountsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            for i in 0..Count as usize {
                let names_i = Names.add(i);
                let names_buf = (*names_i).Buffer;
                let ptr = wcschr(names_buf, '\\' as u16);
                if ptr.is_null() {
                    AccountLength = ((*names_i).Length as usize / core::mem::size_of::<u16>()) as u32;
                    let acct_i = AccountsBuffer.add(i);
                    (*acct_i).Length = (*names_i).Length;
                    (*acct_i).MaximumLength = (*acct_i).Length + core::mem::size_of::<u16>() as u16;
                    (*acct_i).Buffer = MIDL_user_allocate((*acct_i).MaximumLength as usize)
                        as *mut u16;
                    if (*acct_i).Buffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    core::ptr::copy_nonoverlapping::<u8>(
                        (*names_i).Buffer as *const u8,
                        (*acct_i).Buffer as *mut u8,
                        (*acct_i).Length as usize,
                    );
                    *(*acct_i).Buffer.add(AccountLength as usize) = 0u16;
                } else {
                    DomainLength = ptr.offset_from((*names_i).Buffer) as u32;
                    let total_chars = (*names_i).Length as usize / core::mem::size_of::<u16>();
                    AccountLength = (total_chars - DomainLength as usize - 1) as u32;
                    if DomainLength > 0 {
                        let dom_i = DomainsBuffer.add(i);
                        (*dom_i).Length = (DomainLength as u16) * core::mem::size_of::<u16>() as u16;
                        (*dom_i).MaximumLength = (*dom_i).Length + core::mem::size_of::<u16>() as u16;
                        (*dom_i).Buffer = MIDL_user_allocate((*dom_i).MaximumLength as usize)
                            as *mut u16;
                        if (*dom_i).Buffer.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            break;
                        }
                        core::ptr::copy_nonoverlapping::<u8>(
                            (*names_i).Buffer as *const u8,
                            (*dom_i).Buffer as *mut u8,
                            (*dom_i).Length as usize,
                        );
                        *(*dom_i).Buffer.add(DomainLength as usize) = 0u16;
                    }
                    // Account part
                    let acct_i2 = AccountsBuffer.add(i);
                    (*acct_i2).Length = (AccountLength as u16) * core::mem::size_of::<u16>() as u16;
                    (*acct_i2).MaximumLength = (*acct_i2).Length + core::mem::size_of::<u16>() as u16;
                    (*acct_i2).Buffer = MIDL_user_allocate((*acct_i2).MaximumLength as usize)
                        as *mut u16;
                    if (*acct_i2).Buffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    let src = (*names_i).Buffer.add(DomainLength as usize + 1);
                    core::ptr::copy_nonoverlapping::<u8>(
                        src as *const u8,
                        (*acct_i2).Buffer as *mut u8,
                        (*acct_i2).Length as usize,
                    );
                    *(*acct_i2).Buffer.add(AccountLength as usize) = 0u16;
                }
            }
        }
    }
    if Status >= 0 {
        *DomainNames = DomainsBuffer;
        *AccountNames = AccountsBuffer;
    } else {
        if !AccountsBuffer.is_null() {
            for i in 0..Count as usize {
                let buf = (*AccountsBuffer.add(i)).Buffer;
                if !buf.is_null() {
                    MIDL_user_free(buf as *mut core::ffi::c_void);
                }
            }
            MIDL_user_free(AccountsBuffer as *mut core::ffi::c_void);
        }
        if !DomainsBuffer.is_null() {
            for i in 0..Count as usize {
                let buf = (*DomainsBuffer.add(i)).Buffer;
                if !buf.is_null() {
                    MIDL_user_free(buf as *mut core::ffi::c_void);
                }
            }
            MIDL_user_free(DomainsBuffer as *mut core::ffi::c_void);
        }
    }
    Status
}

unsafe fn LsapAddDomainToDomainsList(
    ReferencedDomains: PLSAPR_REFERENCED_DOMAIN_LIST,
    Name: PUNICODE_STRING,
    Sid: PSID,
    Index: *mut ULONG,
) -> NTSTATUS {
    let mut i: u32 = 0;
    while i < (*ReferencedDomains).Entries
        && !(*((*ReferencedDomains).Domains).add(i as usize))
            .Sid
            .is_null()
    {
        if RtlEqualSid(Sid, (*((*ReferencedDomains).Domains).add(i as usize)).Sid as PSID) != 0 {
            *Index = i;
            return STATUS_SUCCESS;
        }
        i = i.wrapping_add(1);
    }
    let domain_at_i = (*ReferencedDomains).Domains.add(i as usize);
    (*domain_at_i).Sid = MIDL_user_allocate(RtlLengthSid(Sid) as usize) as PRPC_SID;
    if (*domain_at_i).Sid.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopySid(
        RtlLengthSid(Sid),
        (*domain_at_i).Sid as PSID,
        Sid,
    );
    (*domain_at_i).Name.Length = (*Name).Length;
    (*domain_at_i).Name.MaximumLength = (*Name).MaximumLength;
    let name_buffer = MIDL_user_allocate((*Name).MaximumLength as usize) as LPWSTR;
    (*domain_at_i).Name.Buffer = name_buffer;
    if name_buffer.is_null() {
        MIDL_user_free((*domain_at_i).Sid as *mut core::ffi::c_void);
        (*domain_at_i).Sid = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        (*Name).Buffer as *const u8,
        name_buffer as *mut u8,
        (*Name).MaximumLength as usize,
    );
    (*ReferencedDomains).Entries = (*ReferencedDomains).Entries.wrapping_add(1);
    *Index = i;
    STATUS_SUCCESS
}

unsafe fn LsapAddAuthorityToDomainsList(
    ReferencedDomains: PLSAPR_REFERENCED_DOMAIN_LIST,
    Sid: PSID,
    Index: PULONG,
) -> NTSTATUS {
    let mut AuthoritySid: SID = core::mem::zeroed();
    let mut i: ULONG = 0;

    RtlInitializeSid(
        core::ptr::addr_of_mut!(AuthoritySid) as PSID,
        RtlIdentifierAuthoritySid(Sid),
        0u8,
    );

    i = 0;
    while i < (*ReferencedDomains).Entries
        && !(*(*ReferencedDomains).Domains.offset(i as isize))
            .Sid
            .is_null()
    {
        if RtlEqualSid(
            core::ptr::addr_of_mut!(AuthoritySid) as PSID,
            (*(*ReferencedDomains).Domains.offset(i as isize)).Sid as PSID,
        ) != 0
        {
            *Index = i;
            return STATUS_SUCCESS;
        }
        i = i.wrapping_add(1);
    }

    let sid_len = RtlLengthSid(core::ptr::addr_of_mut!(AuthoritySid) as PSID) as usize;
    (*(*ReferencedDomains).Domains.offset(i as isize)).Sid =
        MIDL_user_allocate(sid_len) as PRPC_SID;
    if (*(*ReferencedDomains).Domains.offset(i as isize))
        .Sid
        .is_null()
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopySid(
        RtlLengthSid(core::ptr::addr_of_mut!(AuthoritySid) as PSID),
        (*(*ReferencedDomains).Domains.offset(i as isize)).Sid as PSID,
        core::ptr::addr_of_mut!(AuthoritySid) as PSID,
    );

    (*(*ReferencedDomains).Domains.offset(i as isize))
        .Name
        .Length = 0 as USHORT;
    (*(*ReferencedDomains).Domains.offset(i as isize))
        .Name
        .MaximumLength = core::mem::size_of::<WCHAR>() as USHORT;
    (*(*ReferencedDomains).Domains.offset(i as isize))
        .Name
        .Buffer = MIDL_user_allocate(core::mem::size_of::<WCHAR>() as usize) as LPWSTR;

    if (*(*ReferencedDomains).Domains.offset(i as isize))
        .Name
        .Buffer
        .is_null()
    {
        MIDL_user_free(
            (*(*ReferencedDomains).Domains.offset(i as isize)).Sid as *mut core::ffi::c_void,
        );
        (*(*ReferencedDomains).Domains.offset(i as isize)).Sid = core::ptr::null_mut::<_RPC_SID>();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *(*(*ReferencedDomains).Domains.offset(i as isize))
        .Name
        .Buffer
        .offset(0 as isize) = 0u16; // UNICODE_NULL

    (*ReferencedDomains).Entries = (*ReferencedDomains).Entries.wrapping_add(1);
    *Index = i;
    STATUS_SUCCESS
}

unsafe fn LsapIsPrefixSid(PrefixSid: PSID, Sid: PSID) -> BOOLEAN {
    let Sid1: PISID = PrefixSid as PISID;
    let Sid2: PISID = Sid as PISID;

    if (*Sid1).Revision != (*Sid2).Revision {
        return 0u8;
    }

    if (*Sid1).IdentifierAuthority.Value != (*Sid2).IdentifierAuthority.Value {
        return 0u8;
    }

    if (*Sid1).SubAuthorityCount >= (*Sid2).SubAuthorityCount {
        return 0u8;
    }

    if (*Sid1).SubAuthorityCount == 0 {
        return 1u8;
    }

    let count = (*Sid1).SubAuthorityCount as usize;
    for i in 0..count {
        let sub1 = core::ptr::addr_of!((*Sid1).SubAuthority) as *const u32;
        let sub2 = core::ptr::addr_of!((*Sid2).SubAuthority) as *const u32;
        if *sub1.add(i) != *sub2.add(i) {
            return 0u8;
        }
    }

    1u8
}
#[no_mangle]
pub unsafe extern "C" fn LsapGetRelativeIdFromSid(Sid_: PSID) -> ULONG {
    let sid = Sid_ as *mut SID;
    if (*sid).SubAuthorityCount != 0 {
        return *(*sid).SubAuthority.as_ptr().add((*sid).SubAuthorityCount as usize - 1);
    }
    0
}

unsafe fn CreateSidFromSidAndRid(SrcSid: PSID, RelativeId: u32) -> PSID {
    let RidCount: u8 = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8 {
        return core::ptr::null_mut();
    }
    let DstSidSize = RtlLengthRequiredSid((RidCount + 1) as u32);
    let DstSid: PSID = MIDL_user_allocate(DstSidSize as usize) as PSID;
    if DstSid.is_null() {
        return core::ptr::null_mut();
    }
    RtlInitializeSid(DstSid, RtlIdentifierAuthoritySid(SrcSid), (RidCount + 1) as u8);
    for i in 0..(RidCount as u32) {
        let p = RtlSubAuthoritySid(SrcSid, i);
        let q = RtlSubAuthoritySid(DstSid, i);
        *q = *p;
    }
    let q = RtlSubAuthoritySid(DstSid, RidCount as u32);
    *q = RelativeId;
    DstSid
}

unsafe fn CreateDomainSidFromAccountSid(AccountSid: PSID) -> PSID {
    let mut RidCount: u8 = *RtlSubAuthorityCountSid(AccountSid);
    if RidCount > 0 {
        RidCount -= 1;
    }
    let DstSidSize: u32 = RtlLengthRequiredSid(RidCount as u32);
    let DomainSid: PSID = MIDL_user_allocate(DstSidSize as usize) as PSID;
    if DomainSid.is_null() {
        return core::ptr::null_mut();
    }
    RtlInitializeSid(
        DomainSid,
        RtlIdentifierAuthoritySid(AccountSid),
        RidCount,
    );
    for i in 0..RidCount as u32 {
        let p: *mut u32 = RtlSubAuthoritySid(AccountSid, i);
        let q: *mut u32 = RtlSubAuthoritySid(DomainSid, i);
        *q = *p;
    }
    DomainSid
}

unsafe fn LsapCopySid(src_sid: PSID) -> PSID {
    let rid_count: UCHAR = *RtlSubAuthorityCountSid(src_sid);
    let dst_sid_size: ULONG = RtlLengthRequiredSid(rid_count as ULONG);
    let dst_sid: PSID = MIDL_user_allocate(dst_sid_size as usize) as PSID;
    if dst_sid.is_null() {
        return core::ptr::null_mut();
    }
    RtlInitializeSid(
        dst_sid,
        RtlIdentifierAuthoritySid(src_sid),
        rid_count,
    );
    for i in 0..(rid_count as ULONG) {
        let p: PULONG = RtlSubAuthoritySid(src_sid, i);
        let q: PULONG = RtlSubAuthoritySid(dst_sid, i);
        *q = *p;
    }
    dst_sid
}

unsafe fn LsapLookupIsolatedNames(
    Count: u32,
    DomainNames: *mut RPC_UNICODE_STRING,
    AccountNames: *mut RPC_UNICODE_STRING,
    DomainsBuffer: *mut _LSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: *mut _LSAPR_TRANSLATED_SID_EX2,
    Mapped: *mut u32,
) -> NTSTATUS {
    let mut EmptyDomainName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of::<[u16; 1]>() as u16,
        Buffer: w!("").as_ptr() as *const u16 as *mut u16,
    };
    let mut ptr: *mut _WELL_KNOWN_SID = core::ptr::null_mut();
    let mut ptr2: *mut _WELL_KNOWN_SID = core::ptr::null_mut();
    let mut DomainSid: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DomainIndex: u32 = 0;
    let mut i: u32 = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    while i < Count {
        if (*SidsBuffer.add(i as usize)).Use as u32 != SidTypeUnknown as u32 {
            i = i.wrapping_add(1);
            continue;
        }
        if (*DomainNames.add(i as usize)).Length != 0 {
            i = i.wrapping_add(1);
            continue;
        }

        // TRACE("Mapping name: %wZ\n", &AccountNames[i]);
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                b"LsapLookupIsolatedNames\0".as_ptr() as *const i8,
                1178,
                b"Mapping name: %wZ\n\0".as_ptr() as *const i8,
                AccountNames.add(i as usize) as *mut _UNICODE_STRING,
            );
        }

        ptr = LsapLookupIsolatedWellKnownName(AccountNames.add(i as usize) as *mut _UNICODE_STRING);
        if !ptr.is_null() {
            (*SidsBuffer.add(i as usize)).Use = (*ptr).Use;
            (*SidsBuffer.add(i as usize)).Sid = LsapCopySid((*ptr).Sid) as *mut _RPC_SID;
            if (*SidsBuffer.add(i as usize)).Sid.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = -1i32 as i32;
            (*SidsBuffer.add(i as usize)).Flags = 0;

            if (*ptr).Use as u32 == SidTypeDomain as u32 {
                Status = LsapAddDomainToDomainsList(
                    DomainsBuffer,
                    &mut (*ptr).AccountName as *mut _UNICODE_STRING,
                    (*ptr).Sid,
                    &mut DomainIndex as *mut u32,
                );
                if !NT_SUCCESS(Status) {
                    break;
                }
                (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
            } else {
                ptr2 = LsapLookupIsolatedWellKnownName(&mut (*ptr).DomainName);
                if !ptr2.is_null() {
                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &mut (*ptr2).AccountName as *mut _UNICODE_STRING,
                        (*ptr2).Sid,
                        &mut DomainIndex as *mut u32,
                    );
                    if !NT_SUCCESS(Status) {
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
                } else {
                    DomainSid = CreateDomainSidFromAccountSid((*ptr).Sid);
                    if DomainSid.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &mut EmptyDomainName as *mut _UNICODE_STRING,
                        DomainSid,
                        &mut DomainIndex as *mut u32,
                    );
                    if !DomainSid.is_null() {
                        MIDL_user_free(DomainSid);
                        DomainSid = core::ptr::null_mut();
                    }
                    if !NT_SUCCESS(Status) {
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
                }
            }
            *Mapped = (*Mapped).wrapping_add(1);
            i = i.wrapping_add(1);
            continue;
        }

        if RtlEqualUnicodeString(
            AccountNames.add(i as usize) as *mut _UNICODE_STRING as PCUNICODE_STRING,
            &raw mut BuiltinDomainName as *mut _UNICODE_STRING as PCUNICODE_STRING,
            1u8,
        ) != 0
        {
            (*SidsBuffer.add(i as usize)).Use = SidTypeDomain;
            (*SidsBuffer.add(i as usize)).Sid = LsapCopySid(BuiltinDomainSid) as *mut _RPC_SID;
            if (*SidsBuffer.add(i as usize)).Sid.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = -1i32 as i32;
            (*SidsBuffer.add(i as usize)).Flags = 0;
            Status = LsapAddDomainToDomainsList(
                DomainsBuffer,
                &raw mut BuiltinDomainName as *mut _UNICODE_STRING,
                BuiltinDomainSid,
                &mut DomainIndex as *mut u32,
            );
            if !NT_SUCCESS(Status) {
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
            *Mapped = (*Mapped).wrapping_add(1);
            i = i.wrapping_add(1);
            continue;
        }

        if RtlEqualUnicodeString(
            AccountNames.add(i as usize) as *mut _UNICODE_STRING as PCUNICODE_STRING,
            &raw mut AccountDomainName as *mut _UNICODE_STRING as PCUNICODE_STRING,
            1u8,
        ) != 0
        {
            (*SidsBuffer.add(i as usize)).Use = SidTypeDomain;
            (*SidsBuffer.add(i as usize)).Sid = LsapCopySid(AccountDomainSid) as *mut _RPC_SID;
            if (*SidsBuffer.add(i as usize)).Sid.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = -1i32 as i32;
            (*SidsBuffer.add(i as usize)).Flags = 0;
            Status = LsapAddDomainToDomainsList(
                DomainsBuffer,
                &raw mut AccountDomainName as *mut _UNICODE_STRING,
                AccountDomainSid,
                &mut DomainIndex as *mut u32,
            );
            if !NT_SUCCESS(Status) {
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
            *Mapped = (*Mapped).wrapping_add(1);
            i = i.wrapping_add(1);
            continue;
        }

        i = i.wrapping_add(1);
    }

    Status
}

unsafe fn LsapLookupIsolatedBuiltinNames(
    Count: DWORD,
    DomainNames: PRPC_UNICODE_STRING,
    AccountNames: PRPC_UNICODE_STRING,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: PLSAPR_TRANSLATED_SID_EX2,
    Mapped: PULONG,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut DomainIndex: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SamrConnect(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ServerHandle),
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
    );
    if !(Status >= 0) {
        TRACE!(
            b"SamrConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    } else {
        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP as ACCESS_MASK,
            BuiltinDomainSid as samsrv_sys::PRPC_SID,
            core::ptr::addr_of_mut!(DomainHandle),
        );
        if !(Status >= 0) {
            TRACE!(
                b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        } else {
            for i in 0..Count {
                if (*SidsBuffer.add(i as usize)).Use != SidTypeUnknown {
                    continue;
                }
                if (*DomainNames.add(i as usize)).Length != 0 {
                    continue;
                }
                TRACE!(
                    b"Mapping name: %wZ\n\0".as_ptr() as *const i8,
                    AccountNames.add(i as usize) as *mut samsrv_sys::_RPC_UNICODE_STRING,
                );
                Status = SamrLookupNamesInDomain(
                    DomainHandle,
                    1,
                    AccountNames.add(i as usize) as *mut samsrv_sys::_RPC_UNICODE_STRING,
                    core::ptr::addr_of_mut!(RelativeIds),
                    core::ptr::addr_of_mut!(Use),
                );
                if Status >= 0 {
                    TRACE!(
                        b"Found relative ID: %lu\n\0".as_ptr() as *const i8,
                        *RelativeIds.Element.add(0),
                    );
                    (*SidsBuffer.add(i as usize)).Use = *Use.Element.add(0) as SID_NAME_USE;
                    let fresh8 = &mut (*SidsBuffer.add(i as usize)).Sid;
                    *fresh8 = CreateSidFromSidAndRid(
                        BuiltinDomainSid,
                        *RelativeIds.Element.add(0) as ULONG,
                    ) as PRPC_SID;
                    if (*SidsBuffer.add(i as usize)).Sid.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).DomainIndex = -1i32 as LONG;
                    (*SidsBuffer.add(i as usize)).Flags = 0;
                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        core::ptr::addr_of_mut!(BuiltinDomainName),
                        BuiltinDomainSid,
                        core::ptr::addr_of_mut!(DomainIndex),
                    );
                    if !(Status >= 0) {
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as LONG;
                    *Mapped += 1;
                }
                SamIFree_SAMPR_ULONG_ARRAY(core::ptr::addr_of_mut!(RelativeIds));
                SamIFree_SAMPR_ULONG_ARRAY(core::ptr::addr_of_mut!(Use));
            }
        }
    }

    if !DomainHandle.is_null() {
        SamrCloseHandle(core::ptr::addr_of_mut!(DomainHandle));
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(core::ptr::addr_of_mut!(ServerHandle));
    }
    Status
}

unsafe fn LsapLookupIsolatedAccountNames(
    Count: u32,
    DomainNames: *mut _UNICODE_STRING,
    AccountNames: *mut _UNICODE_STRING,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: PLSAPR_TRANSLATED_SID_EX2,
    Mapped: *mut u32,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: _SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY { Count: 0, Element: core::ptr::null_mut() };
    let mut Use: _SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY { Count: 0, Element: core::ptr::null_mut() };
    let mut DomainIndex: u32 = 0;
    let _i: u32;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
            b"LsapLookupIsolatedAccountNames\0".as_ptr() as *const i8,
            1429i32,
            b"()\n\0".as_ptr() as *const i8,
        );
    }

    Status = SamrConnect(core::ptr::null_mut(), &mut ServerHandle, (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32);
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                b"LsapLookupIsolatedAccountNames\0".as_ptr() as *const i8,
                1436i32,
                b"SamrConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    } else {
        Status = SamrOpenDomain(ServerHandle, DOMAIN_LOOKUP as u32, AccountDomainSid as samsrv_sys::PRPC_SID, &mut DomainHandle);
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                    b"LsapLookupIsolatedAccountNames\0".as_ptr() as *const i8,
                    1446i32,
                    b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        } else {
            for i in 0..Count {
                if (*SidsBuffer.offset(i as isize)).Use != SidTypeUnknown {
                    continue;
                }
                if (*DomainNames.offset(i as isize)).Length != 0 {
                    continue;
                }

                if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                        b"LsapLookupIsolatedAccountNames\0".as_ptr() as *const i8,
                        1460i32,
                        b"Mapping name: %wZ\n\0".as_ptr() as *const i8,
                        &mut (*AccountNames.offset(i as isize)) as *mut _UNICODE_STRING,
                    );
                }

                Status = SamrLookupNamesInDomain(DomainHandle, 1, &mut (*AccountNames.offset(i as isize)) as *mut _ as *mut samsrv_sys::_RPC_UNICODE_STRING, &mut RelativeIds, &mut Use);
                if NT_SUCCESS(Status) {
                    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                            b"LsapLookupIsolatedAccountNames\0".as_ptr() as *const i8,
                            1469i32,
                            b"Found relative ID: %lu\n\0".as_ptr() as *const i8,
                            *RelativeIds.Element.offset(0),
                        );
                    }

                    (*SidsBuffer.offset(i as isize)).Use = *Use.Element.offset(0) as SID_NAME_USE;
                    let fresh_sid = CreateSidFromSidAndRid(AccountDomainSid, *RelativeIds.Element.offset(0) as u32);
                    (*SidsBuffer.offset(i as isize)).Sid = fresh_sid as PRPC_SID;
                    if fresh_sid.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }

                    (*SidsBuffer.offset(i as isize)).DomainIndex = -1i32;
                    (*SidsBuffer.offset(i as isize)).Flags = 0;

                    Status = LsapAddDomainToDomainsList(DomainsBuffer, &raw mut AccountDomainName, AccountDomainSid, &mut DomainIndex);
                    if !NT_SUCCESS(Status) {
                        break;
                    }
                    (*SidsBuffer.offset(i as isize)).DomainIndex = DomainIndex as i32;
                    *Mapped = (*Mapped).wrapping_add(1);
                }

                SamIFree_SAMPR_ULONG_ARRAY(&mut RelativeIds);
                SamIFree_SAMPR_ULONG_ARRAY(&mut Use);
            }
        }
    }

    if !DomainHandle.is_null() {
        SamrCloseHandle(&mut DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&mut ServerHandle);
    }

    return Status;
}

unsafe fn LsapLookupFullyQualifiedWellKnownNames(
    Count: u32,
    DomainNames: *mut _UNICODE_STRING,
    AccountNames: *mut _UNICODE_STRING,
    DomainsBuffer: *mut LSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: *mut LSAPR_TRANSLATED_SID_EX2,
    Mapped: *mut u32,
) -> i32 {
    let mut EmptyDomainName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0u16,
        MaximumLength: 2u16,
        Buffer: (&0u16 as *const u16) as *mut u16,
    };
    let mut DomainIndex: ULONG = 0;
    let mut Status: i32 = STATUS_SUCCESS;

    for i in 0..Count {
        if (*SidsBuffer.add(i as usize)).Use != SidTypeUnknown {
            continue;
        }
        if (*DomainNames.add(i as usize)).Length == 0 {
            continue;
        }

        let ptr: *mut _WELL_KNOWN_SID = LsapLookupFullyQualifiedWellKnownName(
            &(*AccountNames.add(i as usize)) as *const _ as *mut _UNICODE_STRING,
            &(*DomainNames.add(i as usize)) as *const _ as *mut _UNICODE_STRING,
        );
        if ptr.is_null() {
            continue;
        }

        let sid_copy = LsapCopySid((*ptr).Sid);
        if sid_copy.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        (*SidsBuffer.add(i as usize)).Use = (*ptr).Use;
        (*SidsBuffer.add(i as usize)).Sid = sid_copy as PRPC_SID;
        (*SidsBuffer.add(i as usize)).DomainIndex = -1i32;
        (*SidsBuffer.add(i as usize)).Flags = 0u32;

        if (*ptr).Use == SidTypeDomain {
            Status = LsapAddDomainToDomainsList(
                DomainsBuffer,
                core::ptr::addr_of_mut!((*ptr).AccountName),
                (*ptr).Sid,
                core::ptr::addr_of_mut!(DomainIndex),
            );
            if !NT_SUCCESS(Status) {
                break;
            }
            (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
        } else {
            let ptr2: *mut _WELL_KNOWN_SID =
                LsapLookupIsolatedWellKnownName(core::ptr::addr_of_mut!((*ptr).DomainName));
            if !ptr2.is_null() {
                Status = LsapAddDomainToDomainsList(
                    DomainsBuffer,
                    core::ptr::addr_of_mut!((*ptr2).AccountName),
                    (*ptr2).Sid,
                    core::ptr::addr_of_mut!(DomainIndex),
                );
                if !NT_SUCCESS(Status) {
                    break;
                }
                (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
            } else {
                let DomainSid: PSID = CreateDomainSidFromAccountSid((*ptr).Sid);
                if DomainSid.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }
                Status = LsapAddDomainToDomainsList(
                    DomainsBuffer,
                    &mut EmptyDomainName,
                    DomainSid,
                    core::ptr::addr_of_mut!(DomainIndex),
                );
                if !DomainSid.is_null() {
                    MIDL_user_free(DomainSid as *mut core::ffi::c_void);
                }
                if !NT_SUCCESS(Status) {
                    break;
                }
                (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
            }
        }

        *Mapped += 1;
    }

    Status
}

unsafe fn LsapLookupBuiltinNames(
    Count: u32,
    DomainNames: *mut RPC_UNICODE_STRING,
    AccountNames: *mut RPC_UNICODE_STRING,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: PLSAPR_TRANSLATED_SID_EX2,
    Mapped: *mut u32,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY {
        Count: 0u32,
        Element: core::ptr::null_mut(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY {
        Count: 0u32,
        Element: core::ptr::null_mut(),
    };
    let mut DomainIndex: u32 = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let _result = 'main: {
        Status = SamrConnect(
            core::ptr::null_mut::<u16>(),
            &mut ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0"
                        .as_ptr() as *const i8,
                    b"LsapLookupBuiltinNames\0".as_ptr() as *const i8,
                    1640,
                    b"SamrConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            break 'main;
        }
        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP as ACCESS_MASK,
            BuiltinDomainSid as samsrv_sys::PRPC_SID,
            &mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0"
                        .as_ptr() as *const i8,
                    b"LsapLookupBuiltinNames\0".as_ptr() as *const i8,
                    1650,
                    b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            break 'main;
        }
        for i in 0..Count {
            if (*SidsBuffer.add(i as usize)).Use != SidTypeUnknown {
                continue;
            }
            if (*DomainNames.add(i as usize)).Length == 0 {
                continue;
            }
            if RtlEqualUnicodeString(
                DomainNames.add(i as usize) as *mut RPC_UNICODE_STRING as PUNICODE_STRING
                    as PCUNICODE_STRING,
                &raw const BuiltinDomainName as PCUNICODE_STRING,
                1u8,
            ) == 0
            {
                continue;
            }
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0"
                        .as_ptr() as *const i8,
                    b"LsapLookupBuiltinNames\0".as_ptr() as *const i8,
                    1667,
                    b"Mapping name: %wZ\\%wZ\n\0".as_ptr() as *const i8,
                    DomainNames.add(i as usize) as *mut RPC_UNICODE_STRING,
                    AccountNames.add(i as usize) as *mut samsrv_sys::_RPC_UNICODE_STRING,
                );
            }
            Status = SamrLookupNamesInDomain(
                DomainHandle,
                1u32,
                AccountNames.add(i as usize) as *mut samsrv_sys::_RPC_UNICODE_STRING,
                &mut RelativeIds,
                &mut Use,
            );
            if NT_SUCCESS(Status) {
                (*SidsBuffer.add(i as usize)).Use =
                    *Use.Element.offset(0isize) as SID_NAME_USE;
                {
                    let fresh5 = &mut (*SidsBuffer.add(i as usize)).Sid;
                    *fresh5 = CreateSidFromSidAndRid(
                        BuiltinDomainSid,
                        *RelativeIds.Element.offset(0isize) as u32,
                    ) as PRPC_SID;
                }
                if (*SidsBuffer.add(i as usize)).Sid.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'main;
                }
                (*SidsBuffer.add(i as usize)).DomainIndex = -(1i32);
                (*SidsBuffer.add(i as usize)).Flags = 0u32;
                Status = LsapAddDomainToDomainsList(
                    DomainsBuffer,
                    &raw mut BuiltinDomainName,
                    BuiltinDomainSid,
                    &mut DomainIndex,
                );
                if !NT_SUCCESS(Status) {
                    break 'main;
                }
                (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
                *Mapped = (*Mapped).wrapping_add(1);
            }
            SamIFree_SAMPR_ULONG_ARRAY(&mut RelativeIds);
            SamIFree_SAMPR_ULONG_ARRAY(&mut Use);
        }
    };
    if !DomainHandle.is_null() {
        SamrCloseHandle(&mut DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&mut ServerHandle);
    }
    Status
}

unsafe fn LsapLookupAccountNames(
    Count: u32,
    DomainNames: *mut RPC_UNICODE_STRING,
    AccountNames: *mut RPC_UNICODE_STRING,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    SidsBuffer: *mut LSAPR_TRANSLATED_SID_EX2,
    Mapped: *mut u32,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut::<u32>(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut::<u32>(),
    };
    let mut DomainIndex: u32 = 0;
    let mut Status: NTSTATUS;

    Status = SamrConnect(
        core::ptr::null_mut::<wchar_t>(),
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
    );
    if !(Status >= 0) {
        if (*__wine_dbch___default).flags as i32
            & ((1 as i32) << __WINE_DBCL_TRACE as i32)
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"lookup.c\0".as_ptr() as *const i8,
                b"LsapLookupAccountNames\0".as_ptr() as *const i8,
                1737i32,
                b"SamrConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    } else {
        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP as ACCESS_MASK,
            AccountDomainSid as samsrv_sys::PRPC_SID,
            &raw mut DomainHandle,
        );
        if !(Status >= 0) {
            if (*__wine_dbch___default).flags as i32
                & ((1 as i32) << __WINE_DBCL_TRACE as i32)
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"lookup.c\0".as_ptr() as *const i8,
                    b"LsapLookupAccountNames\0".as_ptr() as *const i8,
                    1747i32,
                    b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        } else {
            let mut i: u32 = 0;
            while i < Count {
                if (*SidsBuffer.add(i as usize)).Use as u32 != SidTypeUnknown as u32 {
                    i = i.wrapping_add(1);
                    continue;
                }
                if (*DomainNames.add(i as usize)).Length == 0 {
                    i = i.wrapping_add(1);
                    continue;
                }
                if RtlEqualUnicodeString(
                    DomainNames.add(i as usize) as PUNICODE_STRING,
                    &raw const AccountDomainName as PCUNICODE_STRING,
                    1u8,
                ) == 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }

                if (*__wine_dbch___default).flags as i32
                    & ((1 as i32) << __WINE_DBCL_TRACE as i32)
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"lookup.c\0".as_ptr() as *const i8,
                        b"LsapLookupAccountNames\0".as_ptr() as *const i8,
                        1764i32,
                        b"Mapping name: %wZ\\%wZ\n\0".as_ptr() as *const i8,
                        DomainNames.add(i as usize) as *mut _RPC_UNICODE_STRING,
                        AccountNames.add(i as usize) as *mut _RPC_UNICODE_STRING,
                    );
                }

                Status = SamrLookupNamesInDomain(
                    DomainHandle,
                    1,
                    AccountNames.add(i as usize) as *mut samsrv_sys::_RPC_UNICODE_STRING,
                    &raw mut RelativeIds,
                    &raw mut Use,
                );
                if Status >= 0 {
                    (*SidsBuffer.add(i as usize)).Use =
                        *Use.Element.add(0) as SID_NAME_USE;
                    let sid_ptr = CreateSidFromSidAndRid(
                        AccountDomainSid,
                        *RelativeIds.Element.add(0) as u32,
                    );
                    if sid_ptr.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).Sid = sid_ptr as PRPC_SID;
                    (*SidsBuffer.add(i as usize)).DomainIndex = -1i32;
                    (*SidsBuffer.add(i as usize)).Flags = 0u32;

                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &raw mut AccountDomainName,
                        AccountDomainSid,
                        &raw mut DomainIndex,
                    );
                    if !(Status >= 0) {
                        break;
                    }
                    (*SidsBuffer.add(i as usize)).DomainIndex = DomainIndex as i32;
                    *Mapped = (*Mapped).wrapping_add(1);
                }

                SamIFree_SAMPR_ULONG_ARRAY(&raw mut RelativeIds);
                SamIFree_SAMPR_ULONG_ARRAY(&raw mut Use);

                i = i.wrapping_add(1);
            }
        }
    }

    if !DomainHandle.is_null() {
        SamrCloseHandle(&raw mut DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&raw mut ServerHandle);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupNames(
    Count: u32,
    Names: *mut _RPC_UNICODE_STRING,
    ReferencedDomains: *mut *mut _LSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedSids: *mut _LSAPR_TRANSLATED_SIDS_EX2,
    _LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut u32,
    _LookupOptions: u32,
    _ClientRevision: u32,
) -> i32 {
    let mut DomainsBuffer: *mut _LSAPR_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut SidsBuffer: *mut _LSAPR_TRANSLATED_SID_EX2 = core::ptr::null_mut();
    let mut DomainNames: *mut _RPC_UNICODE_STRING = core::ptr::null_mut();
    let mut AccountNames: *mut _RPC_UNICODE_STRING = core::ptr::null_mut();
    let SidsBufferLength: usize;
    let mut i: u32;
    let mut Mapped: u32 = 0u32;
    let mut Status: i32 = STATUS_SUCCESS;

    (*TranslatedSids).Entries = 0u32;
    (*TranslatedSids).Sids = core::ptr::null_mut();
    *ReferencedDomains = core::ptr::null_mut();

    SidsBufferLength = (Count as usize).wrapping_mul(core::mem::size_of::<_LSAPR_TRANSLATED_SID_EX2>());
    SidsBuffer = MIDL_user_allocate(SidsBufferLength as usize) as *mut _LSAPR_TRANSLATED_SID_EX2;
    if SidsBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        DomainsBuffer = MIDL_user_allocate(core::mem::size_of::<_LSAPR_REFERENCED_DOMAIN_LIST>()) as *mut _LSAPR_REFERENCED_DOMAIN_LIST;
        if DomainsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            (*DomainsBuffer).Domains = MIDL_user_allocate(
                (Count as usize).wrapping_mul(core::mem::size_of::<_LSA_TRUST_INFORMATION>())
            ) as PLSAPR_TRUST_INFORMATION;
            if (*DomainsBuffer).Domains.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                (*DomainsBuffer).Entries = 0u32;
                (*DomainsBuffer).MaxEntries = Count;
                i = 0u32;
                while i < Count {
                    (*SidsBuffer.offset(i as isize)).Use = SidTypeUnknown;
                    (*SidsBuffer.offset(i as isize)).Sid = core::ptr::null_mut::<_RPC_SID>();
                    (*SidsBuffer.offset(i as isize)).DomainIndex = -1i32;
                    (*SidsBuffer.offset(i as isize)).Flags = 0u32;
                    i = i.wrapping_add(1);
                }
                Status = LsapSplitNames(Count, Names, core::ptr::addr_of_mut!(DomainNames), core::ptr::addr_of_mut!(AccountNames));
                if Status < 0 {
                    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                            b"LsapLookupNames\0".as_ptr() as *const i8,
                            1878i32,
                            b"LsapSplitNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                            Status,
                        );
                    }
                } else {
                    Status = LsapLookupIsolatedNames(
                        Count,
                        DomainNames,
                        AccountNames,
                        DomainsBuffer,
                        SidsBuffer,
                        core::ptr::addr_of_mut!(Mapped),
                    );
                    if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                        if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                            ros_dbg_log(
                                __dbcl_0,
                                __dbch_0,
                                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                b"LsapLookupNames\0".as_ptr() as *const i8,
                                1893i32,
                                b"LsapLookupIsolatedNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                Status,
                            );
                        }
                    } else if Mapped == Count {
                    } else {
                        Status = LsapLookupIsolatedBuiltinNames(
                            Count,
                            DomainNames,
                            AccountNames,
                            DomainsBuffer,
                            SidsBuffer,
                            core::ptr::addr_of_mut!(Mapped),
                        );
                        if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_1,
                                    __dbch_1,
                                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                    b"LsapLookupNames\0".as_ptr() as *const i8,
                                    1911i32,
                                    b"LsapLookupIsolatedBuiltinNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                    Status,
                                );
                            }
                        } else if Mapped == Count {
                        } else {
                            Status = LsapLookupIsolatedAccountNames(
                                Count,
                                DomainNames as *mut _UNICODE_STRING,
                                AccountNames as *mut _UNICODE_STRING,
                                DomainsBuffer,
                                SidsBuffer,
                                core::ptr::addr_of_mut!(Mapped),
                            );
                            if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                                if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                                    ros_dbg_log(
                                        __dbcl_2,
                                        __dbch_2,
                                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                        b"LsapLookupNames\0".as_ptr() as *const i8,
                                        1929i32,
                                        b"LsapLookupIsolatedAccountNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                        Status,
                                    );
                                }
                            } else if Mapped == Count {
                            } else {
                                Status = LsapLookupFullyQualifiedWellKnownNames(
                                    Count,
                                    DomainNames as *mut _UNICODE_STRING,
                                    AccountNames as *mut _UNICODE_STRING,
                                    DomainsBuffer,
                                    SidsBuffer,
                                    core::ptr::addr_of_mut!(Mapped),
                                );
                                if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                                    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                                        ros_dbg_log(
                                            __dbcl_3,
                                            __dbch_3,
                                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                            b"LsapLookupNames\0".as_ptr() as *const i8,
                                            1946i32,
                                            b"LsapLookupFullyQualifiedWellKnownNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                            Status,
                                        );
                                    }
                                } else if Mapped == Count {
                                } else {
                                    Status = LsapLookupBuiltinNames(
                                        Count,
                                        DomainNames,
                                        AccountNames,
                                        DomainsBuffer,
                                        SidsBuffer,
                                        core::ptr::addr_of_mut!(Mapped),
                                    );
                                    if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                                        if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                                            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                                            let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                                            ros_dbg_log(
                                                __dbcl_4,
                                                __dbch_4,
                                                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                                b"LsapLookupNames\0".as_ptr() as *const i8,
                                                1963i32,
                                                b"LsapLookupBuiltinNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                                Status,
                                            );
                                        }
                                    } else if Mapped == Count {
                                    } else {
                                        Status = LsapLookupAccountNames(
                                            Count,
                                            DomainNames,
                                            AccountNames,
                                            DomainsBuffer,
                                            SidsBuffer,
                                            core::ptr::addr_of_mut!(Mapped),
                                        );
                                        if Status < 0 && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
                                            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
                                                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                                                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                                                ros_dbg_log(
                                                    __dbcl_5,
                                                    __dbch_5,
                                                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lookup.c\0".as_ptr() as *const i8,
                                                    b"LsapLookupNames\0".as_ptr() as *const i8,
                                                    1981i32,
                                                    b"LsapLookupAccountNames failed! (Status %lx)\n\0".as_ptr() as *const i8,
                                                    Status,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !DomainNames.is_null() {
        i = 0u32;
        while i < Count {
            if !(*DomainNames.offset(i as isize)).Buffer.is_null() {
                MIDL_user_free((*DomainNames.offset(i as isize)).Buffer as *mut core::ffi::c_void);
            }
            i = i.wrapping_add(1);
        }
        MIDL_user_free(DomainNames as *mut core::ffi::c_void);
    }
    if !AccountNames.is_null() {
        i = 0u32;
        while i < Count {
            if !(*AccountNames.offset(i as isize)).Buffer.is_null() {
                MIDL_user_free((*AccountNames.offset(i as isize)).Buffer as *mut core::ffi::c_void);
            }
            i = i.wrapping_add(1);
        }
        MIDL_user_free(AccountNames as *mut core::ffi::c_void);
    }
    if Status < 0 {
        if !DomainsBuffer.is_null() {
            if !(*DomainsBuffer).Domains.is_null() {
                MIDL_user_free((*DomainsBuffer).Domains as *mut core::ffi::c_void);
            }
            MIDL_user_free(DomainsBuffer as *mut core::ffi::c_void);
        }
        if !SidsBuffer.is_null() {
            MIDL_user_free(SidsBuffer as *mut core::ffi::c_void);
        }
    } else {
        *ReferencedDomains = DomainsBuffer;
        (*TranslatedSids).Entries = Count;
        (*TranslatedSids).Sids = SidsBuffer;
        *MappedCount = Mapped;
        if Mapped == 0u32 {
            Status = STATUS_NONE_MAPPED;
        } else if Mapped < Count {
            Status = STATUS_SOME_NOT_MAPPED;
        }
    }
    return Status;
}

unsafe fn LsapLookupWellKnownSids(
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    NamesBuffer: PLSAPR_TRANSLATED_NAME_EX,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    Mapped: PULONG,
) -> NTSTATUS {

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut sid_string: LPWSTR = core::ptr::null_mut();
    let mut domain_index: ULONG = 0;
    let _i: ULONG;

    for i in 0..(*SidEnumBuffer).Entries {
        if (*NamesBuffer.offset(i as isize)).Use as u32 != SidTypeUnknown as u32 {
            continue;
        }

        ConvertSidToStringSidW(
            (*(*SidEnumBuffer).SidInfo.offset(i as isize)).Sid as PSID,
            &raw mut sid_string,
        );
        TRACE!(b"Mapping SID: %S\n\0".as_ptr() as _, sid_string);
        LocalFree(sid_string as *mut core::ffi::c_void);
        sid_string = core::ptr::null_mut();

        let ptr = LsapLookupWellKnownSid((*(*SidEnumBuffer).SidInfo.offset(i as isize)).Sid as PSID);
        if !ptr.is_null() {
            let name = &mut *NamesBuffer.offset(i as isize);
            name.Use = (*ptr).Use;
            name.Flags = 0;
            name.Name.Length = (*ptr).AccountName.Length;
            name.Name.MaximumLength = (*ptr).AccountName.MaximumLength;
            name.Name.Buffer = MIDL_user_allocate((*ptr).AccountName.MaximumLength as _) as LPWSTR;
            if name.Name.Buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
            core::ptr::copy_nonoverlapping(
                (*ptr).AccountName.Buffer as *const u8,
                name.Name.Buffer as *mut u8,
                (*ptr).AccountName.MaximumLength as usize,
            );

            if (*ptr).DomainName.Length == 0 {
                status = LsapAddAuthorityToDomainsList(
                    DomainsBuffer,
                    (*(*SidEnumBuffer).SidInfo.offset(i as isize)).Sid as PSID,
                    &raw mut domain_index,
                );
                if !NT_SUCCESS(status) {
                    break;
                }
                name.DomainIndex = domain_index as LONG;
            } else {
                let ptr2 = LsapLookupIsolatedWellKnownName(&raw mut (*ptr).DomainName);
                if !ptr2.is_null() {
                    status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &raw mut (*ptr2).AccountName,
                        (*ptr2).Sid,
                        &raw mut domain_index,
                    );
                    if !NT_SUCCESS(status) {
                        break;
                    }
                    name.DomainIndex = domain_index as LONG;
                }
            }

            TRACE!(b"Mapped to: %wZ\n\0".as_ptr() as _, &raw mut name.Name);
            *Mapped = (*Mapped).wrapping_add(1);
        }
    }
    status
}

unsafe fn LsapLookupBuiltinDomainSids(
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    NamesBuffer: PLSAPR_TRANSLATED_NAME_EX,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    Mapped: *mut ULONG,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut Names = core::mem::zeroed::<samsrv_sys::_SAMPR_RETURNED_USTRING_ARRAY>();
    let mut Use = core::mem::zeroed::<_SAMPR_ULONG_ARRAY>();
    let mut SidString: LPWSTR = core::ptr::null_mut();
    let mut DomainIndex: ULONG = 0;
    let mut RelativeIds: [ULONG; 1] = [0; 1];
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    'done_block: {
        Status = SamrConnect(
            core::ptr::null_mut(),
            &mut ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamrConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done_block;
        }

        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP as ACCESS_MASK,
            BuiltinDomainSid as samsrv_sys::PRPC_SID,
            &mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done_block;
        }

        for i in 0..(*SidEnumBuffer).Entries {
            let name_entry = &mut *NamesBuffer.add(i as usize);

            if name_entry.Use != SidTypeUnknown {
                continue;
            }

            ConvertSidToStringSidW(
                (*SidEnumBuffer).SidInfo.add(i as usize).read().Sid as PSID,
                &mut SidString,
            );
            TRACE!(b"Mapping SID: %S\n\0".as_ptr() as *const i8, SidString);
            LocalFree(SidString as *mut core::ffi::c_void);
            SidString = core::ptr::null_mut();

            if RtlEqualSid(
                BuiltinDomainSid,
                (*SidEnumBuffer).SidInfo.add(i as usize).read().Sid as PSID,
            ) != 0
            {
                TRACE!(b"Found builtin domain!\n\0".as_ptr() as *const i8);

                name_entry.Use = SidTypeDomain;
                name_entry.Flags = 0;
                name_entry.Name.Length = BuiltinDomainName.Length;
                name_entry.Name.MaximumLength = BuiltinDomainName.MaximumLength;
                name_entry.Name.Buffer =
                    MIDL_user_allocate(BuiltinDomainName.MaximumLength as usize) as LPWSTR;
                if name_entry.Name.Buffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done_block;
                }
                core::ptr::copy_nonoverlapping(
                    BuiltinDomainName.Buffer as *const u16,
                    name_entry.Name.Buffer,
                    BuiltinDomainName.MaximumLength as usize / core::mem::size_of::<u16>(),
                );

                Status = LsapAddDomainToDomainsList(
                    DomainsBuffer,
                    &raw mut BuiltinDomainName,
                    BuiltinDomainSid,
                    &mut DomainIndex,
                );
                if !NT_SUCCESS(Status) {
                    break 'done_block;
                }

                name_entry.DomainIndex = DomainIndex as LONG;
                TRACE!(
                    b"Mapped to: %wZ\n\0".as_ptr() as *const i8,
                    &name_entry.Name,
                );
                *Mapped = (*Mapped).wrapping_add(1);
            } else if LsapIsPrefixSid(
                BuiltinDomainSid,
                (*SidEnumBuffer).SidInfo.add(i as usize).read().Sid as PSID,
            ) != 0
            {
                TRACE!(b"Found builtin domain account!\n\0".as_ptr() as *const i8);

                RelativeIds[0] = LsapGetRelativeIdFromSid(
                    (*SidEnumBuffer).SidInfo.add(i as usize).read().Sid as PSID,
                );
                Status = SamrLookupIdsInDomain(
                    DomainHandle,
                    1,
                    RelativeIds.as_mut_ptr(),
                    &mut Names,
                    &mut Use,
                );
                if NT_SUCCESS(Status) {
                    name_entry.Use = *Use.Element as SID_NAME_USE;
                    name_entry.Flags = 0;

                    let name_element = &*Names.Element;
                    name_entry.Name.Length = name_element.Length;
                    name_entry.Name.MaximumLength = name_element.MaximumLength;
                    name_entry.Name.Buffer =
                        MIDL_user_allocate(name_element.MaximumLength as usize) as LPWSTR;
                    if name_entry.Name.Buffer.is_null() {
                        SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&mut Names);
                        SamIFree_SAMPR_ULONG_ARRAY(&mut Use);
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'done_block;
                    }
                    core::ptr::copy_nonoverlapping(
                        name_element.Buffer as *const u16,
                        name_entry.Name.Buffer,
                        name_element.MaximumLength as usize / core::mem::size_of::<u16>(),
                    );

                    SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&mut Names);
                    SamIFree_SAMPR_ULONG_ARRAY(&mut Use);

                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &raw mut BuiltinDomainName,
                        BuiltinDomainSid,
                        &mut DomainIndex,
                    );
                    if !NT_SUCCESS(Status) {
                        break 'done_block;
                    }

                    name_entry.DomainIndex = DomainIndex as LONG;
                    TRACE!(
                        b"Mapped to: %wZ\n\0".as_ptr() as *const i8,
                        &name_entry.Name,
                    );
                    *Mapped = (*Mapped).wrapping_add(1);
                }
            }
        }
    }

    if !DomainHandle.is_null() {
        SamrCloseHandle(&mut DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&mut ServerHandle);
    }
    Status
}

unsafe fn LsapLookupAccountDomainSids(
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    NamesBuffer: PLSAPR_TRANSLATED_NAME_EX,
    DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST,
    Mapped: PULONG,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut Names: SAMPR_RETURNED_USTRING_ARRAY = samsrv_sys::_SAMPR_RETURNED_USTRING_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = _SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut SidString: LPWSTR = core::ptr::null_mut();
    let mut DomainIndex: ULONG = 0;
    let mut RelativeIds: [ULONG; 1] = [0; 1];
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SamrConnect(
        core::ptr::null_mut(),
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
    );
    if Status < 0 {
        trace!("SamrConnect failed (Status {:08x})", Status);
    } else {
        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP as ACCESS_MASK,
            AccountDomainSid as samsrv_sys::PRPC_SID,
            &raw mut DomainHandle,
        );
        if Status < 0 {
            trace!("SamOpenDomain failed (Status {:08x})", Status);
        } else {
            for i in 0..(*SidEnumBuffer).Entries as usize {
                if (*NamesBuffer.add(i)).Use as u32 != SidTypeUnknown as u32 {
                    continue;
                }
                ConvertSidToStringSidW(
                    (*(*SidEnumBuffer).SidInfo.add(i)).Sid as PSID,
                    &raw mut SidString,
                );
                trace!("Mapping SID: %S", SidString);
                LocalFree(SidString as HLOCAL);
                SidString = core::ptr::null_mut();

                if RtlEqualSid(
                    AccountDomainSid,
                    (*(*SidEnumBuffer).SidInfo.add(i)).Sid as PSID,
                ) != 0
                {
                    trace!("Found account domain!");
                    (*NamesBuffer.add(i)).Use = SidTypeDomain;
                    (*NamesBuffer.add(i)).Flags = 0;
                    (*NamesBuffer.add(i)).Name.Length = AccountDomainName.Length;
                    (*NamesBuffer.add(i)).Name.MaximumLength = AccountDomainName.MaximumLength;
                    let buf = &mut (*NamesBuffer.add(i)).Name.Buffer;
                    *buf = MIDL_user_allocate(AccountDomainName.MaximumLength as usize) as LPWSTR;
                    if (*buf).is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }
                    core::ptr::copy_nonoverlapping(
                        AccountDomainName.Buffer as *const u16,
                        *buf as *mut u16,
                        AccountDomainName.MaximumLength as usize / core::mem::size_of::<u16>(),
                    );
                    Status = LsapAddDomainToDomainsList(
                        DomainsBuffer,
                        &raw mut AccountDomainName,
                        AccountDomainSid,
                        &raw mut DomainIndex,
                    );
                    if Status < 0 {
                        break;
                    }
                    (*NamesBuffer.add(i)).DomainIndex = DomainIndex as LONG;
                    trace!("Mapped to: %wZ", &raw mut (*NamesBuffer.add(i)).Name);
                    *Mapped += 1;
                } else if LsapIsPrefixSid(
                    AccountDomainSid,
                    (*(*SidEnumBuffer).SidInfo.add(i)).Sid as PSID,
                ) != 0
                {
                    trace!("Found account domain account!");
                    RelativeIds[0] = LsapGetRelativeIdFromSid(
                        (*(*SidEnumBuffer).SidInfo.add(i)).Sid as PSID,
                    );
                    Status = SamrLookupIdsInDomain(
                        DomainHandle,
                        1,
                        &raw mut RelativeIds as *mut ULONG,
                        &raw mut Names,
                        &raw mut Use,
                    );
                    if Status >= 0 {
                        (*NamesBuffer.add(i)).Use =
                            *Use.Element.add(0) as SID_NAME_USE;
                        (*NamesBuffer.add(i)).Flags = 0;
                        (*NamesBuffer.add(i)).Name.Length = (*Names.Element.add(0)).Length;
                        (*NamesBuffer.add(i)).Name.MaximumLength =
                            (*Names.Element.add(0)).MaximumLength;
                        let buf = &mut (*NamesBuffer.add(i)).Name.Buffer;
                        *buf = MIDL_user_allocate(
                            (*Names.Element.add(0)).MaximumLength as usize,
                        ) as LPWSTR;
                        if (*buf).is_null() {
                            SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&raw mut Names);
                            SamIFree_SAMPR_ULONG_ARRAY(&raw mut Use);
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            break;
                        }
                        core::ptr::copy_nonoverlapping(
                            (*Names.Element.add(0)).Buffer as *const u16,
                            *buf as *mut u16,
                            (*Names.Element.add(0)).MaximumLength as usize
                                / core::mem::size_of::<u16>(),
                        );
                        SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&raw mut Names);
                        SamIFree_SAMPR_ULONG_ARRAY(&raw mut Use);
                        Status = LsapAddDomainToDomainsList(
                            DomainsBuffer,
                            &raw mut AccountDomainName,
                            AccountDomainSid,
                            &raw mut DomainIndex,
                        );
                        if Status < 0 {
                            break;
                        }
                        (*NamesBuffer.add(i)).DomainIndex = DomainIndex as LONG;
                        trace!("Mapped to: %wZ", &raw mut (*NamesBuffer.add(i)).Name);
                        *Mapped += 1;
                    }
                }
            }
        }
    }
    if !DomainHandle.is_null() {
        SamrCloseHandle(&raw mut DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&raw mut ServerHandle);
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupSids(
    SidEnumBuffer: PLSAPR_SID_ENUM_BUFFER,
    ReferencedDomains: *mut PLSAPR_REFERENCED_DOMAIN_LIST,
    TranslatedNames: PLSAPR_TRANSLATED_NAMES_EX,
    _LookupLevel: LSAP_LOOKUP_LEVEL,
    MappedCount: *mut DWORD,
    _LookupOptions: DWORD,
    _ClientRevision: DWORD,
) -> NTSTATUS {
    let mut DomainsBuffer: PLSAPR_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut NamesBuffer: PLSAPR_TRANSLATED_NAME_EX = core::ptr::null_mut();
    let mut Mapped: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    'done: {
    let names_buffer_len = ((*SidEnumBuffer).Entries as usize)
        .wrapping_mul(core::mem::size_of::<LSAPR_TRANSLATED_NAME_EX>());
    NamesBuffer = MIDL_user_allocate(names_buffer_len) as PLSAPR_TRANSLATED_NAME_EX;
    if NamesBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break 'done;
    }

    DomainsBuffer = MIDL_user_allocate(core::mem::size_of::<LSAPR_REFERENCED_DOMAIN_LIST>()) as PLSAPR_REFERENCED_DOMAIN_LIST;
    if DomainsBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break 'done;
    }

    (*DomainsBuffer).Domains = MIDL_user_allocate(
        ((*SidEnumBuffer).Entries as usize)
            .wrapping_mul(core::mem::size_of::<LSA_TRUST_INFORMATION>()),
    ) as PLSAPR_TRUST_INFORMATION;
    if (*DomainsBuffer).Domains.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break 'done;
    }

    (*DomainsBuffer).Entries = 0;
    (*DomainsBuffer).MaxEntries = (*SidEnumBuffer).Entries;

    for i in 0..(*SidEnumBuffer).Entries {
        let name = NamesBuffer.add(i as usize);
        (*name).Use = SidTypeUnknown;
        (*name).Name.Length = 0;
        (*name).Name.MaximumLength = 0;
        (*name).Name.Buffer = core::ptr::null_mut::<WCHAR>();
        (*name).DomainIndex = -1i32;
        (*name).Flags = 0;
    }

    Status = LsapLookupWellKnownSids(
        SidEnumBuffer,
        NamesBuffer,
        DomainsBuffer,
        &raw mut Mapped,
    );
    if !NT_SUCCESS(Status) && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
        break 'done;
    }
    if Mapped == (*SidEnumBuffer).Entries {
        break 'done;
    }

    Status = LsapLookupBuiltinDomainSids(
        SidEnumBuffer,
        NamesBuffer,
        DomainsBuffer,
        &raw mut Mapped,
    );
    if !NT_SUCCESS(Status) && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
        break 'done;
    }
    if Mapped == (*SidEnumBuffer).Entries {
        break 'done;
    }

    Status = LsapLookupAccountDomainSids(
        SidEnumBuffer,
        NamesBuffer,
        DomainsBuffer,
        &raw mut Mapped,
    );
    if !NT_SUCCESS(Status) && Status != STATUS_NONE_MAPPED && Status != STATUS_SOME_NOT_MAPPED {
        break 'done;
    }
    if Mapped == (*SidEnumBuffer).Entries {
        break 'done;
    }
    } // 'done

    {
        TRACE!(b"done Status: %lx  Mapped: %lu\n\0".as_ptr() as *const i8, Status, Mapped);

        if !NT_SUCCESS(Status) {
            if !DomainsBuffer.is_null() {
                if !(*DomainsBuffer).Domains.is_null() {
                    MIDL_user_free((*DomainsBuffer).Domains as *mut core::ffi::c_void);
                }
                MIDL_user_free(DomainsBuffer as *mut core::ffi::c_void);
            }
            if !NamesBuffer.is_null() {
                MIDL_user_free(NamesBuffer as *mut core::ffi::c_void);
            }
        } else {
            *ReferencedDomains = DomainsBuffer;
            (*TranslatedNames).Entries = (*SidEnumBuffer).Entries;
            (*TranslatedNames).Names = NamesBuffer;
            *MappedCount = Mapped as DWORD;

            if Mapped == 0 {
                Status = STATUS_NONE_MAPPED;
            } else if Mapped < (*SidEnumBuffer).Entries {
                Status = STATUS_SOME_NOT_MAPPED;
            }
        }
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Local Security Authority (LSA) Server
//  * FILE:            reactos/dll/win32/lsasrv/lookup.c
//  * PURPOSE:         Sid / Name lookup functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// #include "resources.h"
//
// /* GLOBALS *****************************************************************/
//
// typedef wchar_t *PSAMPR_SERVER_NAME;
// typedef void *SAMPR_HANDLE;
//
// typedef struct _SAMPR_RETURNED_USTRING_ARRAY
// {
//     unsigned long Count;
//     PRPC_UNICODE_STRING Element;
// } SAMPR_RETURNED_USTRING_ARRAY, *PSAMPR_RETURNED_USTRING_ARRAY;
//
// VOID
// NTAPI
// SamIFree_SAMPR_RETURNED_USTRING_ARRAY(PSAMPR_RETURNED_USTRING_ARRAY Ptr);
//
// VOID
// NTAPI
// SamIFree_SAMPR_ULONG_ARRAY(PSAMPR_ULONG_ARRAY Ptr);
//
// NTSTATUS
// NTAPI
// SamrConnect(IN PSAMPR_SERVER_NAME ServerName,
//             OUT SAMPR_HANDLE *ServerHandle,
//             IN ACCESS_MASK DesiredAccess);
//
// NTSTATUS
// NTAPI
// SamrCloseHandle(IN OUT SAMPR_HANDLE *SamHandle);
//
// NTSTATUS
// NTAPI
// SamrOpenDomain(IN SAMPR_HANDLE ServerHandle,
//                IN ACCESS_MASK DesiredAccess,
//                IN PRPC_SID DomainId,
//                OUT SAMPR_HANDLE *DomainHandle);
//
// NTSTATUS
// NTAPI
// SamrLookupIdsInDomain(IN SAMPR_HANDLE DomainHandle,
//                       IN ULONG Count,
//                       IN ULONG *RelativeIds,
//                       OUT PSAMPR_RETURNED_USTRING_ARRAY Names,
//                       OUT PSAMPR_ULONG_ARRAY Use);
//
// NTSTATUS
// NTAPI
// SamrLookupNamesInDomain(IN SAMPR_HANDLE DomainHandle,
//                         IN ULONG Count,
//                         IN RPC_UNICODE_STRING Names[],
//                         OUT PSAMPR_ULONG_ARRAY RelativeIds,
//                         OUT PSAMPR_ULONG_ARRAY Use);
//
//
// typedef struct _WELL_KNOWN_SID
// {
//     LIST_ENTRY ListEntry;
//     PSID Sid;
//     UNICODE_STRING AccountName;
//     UNICODE_STRING DomainName;
//     SID_NAME_USE Use;
// } WELL_KNOWN_SID, *PWELL_KNOWN_SID;
//
//
// LIST_ENTRY WellKnownSidListHead;
// PSID LsapWorldSid = NULL;
// PSID LsapNetworkSid = NULL;
// PSID LsapBatchSid = NULL;
// PSID LsapInteractiveSid = NULL;
// PSID LsapServiceSid = NULL;
// PSID LsapLocalSystemSid = NULL;
// PSID LsapAdministratorsSid = NULL;
//
//
// /* FUNCTIONS ***************************************************************/
//
// BOOLEAN
// LsapCreateSid(PSID_IDENTIFIER_AUTHORITY IdentifierAuthority,
//               UCHAR SubAuthorityCount,
//               PULONG SubAuthorities,
//               PWSTR AccountName,
//               PWSTR DomainName,
//               SID_NAME_USE Use,
//               PSID *SidPtr)
// {
//     PWELL_KNOWN_SID SidEntry;
//     SIZE_T AccountNameLength, DomainNameLength;
//     PULONG p;
//     ULONG i;
//
//     AccountNameLength = wcslen(AccountName);
//     DomainNameLength = wcslen(DomainName);
//     if ((AccountNameLength > UNICODE_STRING_MAX_CHARS) ||
//         (DomainNameLength > UNICODE_STRING_MAX_CHARS))
//     {
//         return FALSE;
//     }
//
//     SidEntry = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(WELL_KNOWN_SID));
//     if (SidEntry == NULL)
//         return FALSE;
//
//     InitializeListHead(&SidEntry->ListEntry);
//
//     SidEntry->Sid = RtlAllocateHeap(RtlGetProcessHeap(),
//                                     0,
//                                     RtlLengthRequiredSid(SubAuthorityCount));
//     if (SidEntry->Sid == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry);
//         return FALSE;
//     }
//
//     RtlInitializeSid(SidEntry->Sid,
//                      IdentifierAuthority,
//                      SubAuthorityCount);
//
//     for (i = 0; i < (ULONG)SubAuthorityCount; i++)
//     {
//         p = RtlSubAuthoritySid(SidEntry->Sid, i);
//         *p = SubAuthorities[i];
//     }
//
// //    RtlInitUnicodeString(&SidEntry->AccountName,
// //                         AccountName);
//     SidEntry->AccountName.Length = (USHORT)AccountNameLength * sizeof(WCHAR);
//     SidEntry->AccountName.MaximumLength = SidEntry->AccountName.Length + sizeof(WCHAR);
//     SidEntry->AccountName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                                    SidEntry->AccountName.MaximumLength);
//     if (SidEntry->AccountName.Buffer == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry->Sid);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry);
//         return FALSE;
//     }
//
//     wcscpy(SidEntry->AccountName.Buffer,
//            AccountName);
//
// //    RtlInitUnicodeString(&SidEntry->DomainName,
// //                         DomainName);
//     SidEntry->DomainName.Length = (USHORT)DomainNameLength * sizeof(WCHAR);
//     SidEntry->DomainName.MaximumLength = SidEntry->DomainName.Length + sizeof(WCHAR);
//     SidEntry->DomainName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                                   SidEntry->DomainName.MaximumLength);
//     if (SidEntry->DomainName.Buffer == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry->AccountName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry->Sid);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SidEntry);
//         return FALSE;
//     }
//
//     wcscpy(SidEntry->DomainName.Buffer,
//            DomainName);
//
//     SidEntry->Use = Use;
//
//     InsertTailList(&WellKnownSidListHead,
//                    &SidEntry->ListEntry);
//
//     if (SidPtr != NULL)
//         *SidPtr = SidEntry->Sid;
//
//     return TRUE;
// }
//
//
// NTSTATUS
// LsapInitSids(VOID)
// {
//     WCHAR szAccountName[80];
//     WCHAR szDomainName[80];
//     ULONG SubAuthorities[8];
//     HINSTANCE hInstance;
//
//     InitializeListHead(&WellKnownSidListHead);
//
//     hInstance = GetModuleHandleW(L"lsasrv.dll");
//
//     /* NT Authority */
//     LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szAccountName, ARRAYSIZE(szAccountName));
//     LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szDomainName, ARRAYSIZE(szDomainName));
//     LsapCreateSid(&NtAuthority,
//                   0,
//                   NULL,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeDomain,
//                   NULL);
//
//     /* Null Sid */
//     LsapLoadString(hInstance, IDS_NULL_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_NULL_RID;
//     LsapCreateSid(&NullSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* World Sid */
//     LsapLoadString(hInstance, IDS_WORLD_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_WORLD_RID;
//     LsapCreateSid(&WorldSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   &LsapWorldSid);
//
//     /* Local Sid */
//     LsapLoadString(hInstance, IDS_LOCAL_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_LOCAL_RID;
//     LsapCreateSid(&LocalSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Creator Owner Sid */
//     LsapLoadString(hInstance, IDS_CREATOR_OWNER_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_CREATOR_OWNER_RID;
//     LsapCreateSid(&CreatorSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Creator Group Sid */
//     LsapLoadString(hInstance, IDS_CREATOR_GROUP_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_CREATOR_GROUP_RID;
//     LsapCreateSid(&CreatorSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Creator Owner Server Sid */
//     LsapLoadString(hInstance, IDS_CREATOR_OWNER_SERVER_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_CREATOR_OWNER_SERVER_RID;
//     LsapCreateSid(&CreatorSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Creator Group Server Sid */
//     LsapLoadString(hInstance, IDS_CREATOR_GROUP_SERVER_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_CREATOR_GROUP_SERVER_RID;
//     LsapCreateSid(&CreatorSidAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   L"",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Dialup Sid */
//     LsapLoadString(hInstance, IDS_DIALUP_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//     LsapLoadString(hInstance, IDS_NT_AUTHORITY as u32, szDomainName, ARRAYSIZE(szDomainName));
//
//     SubAuthorities[0] = SECURITY_DIALUP_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Network Sid */
//     LsapLoadString(hInstance, IDS_NETWORK_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_NETWORK_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   &LsapNetworkSid);
//
//     /* Batch Sid*/
//     LsapLoadString(hInstance, IDS_BATCH_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BATCH_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   &LsapBatchSid);
//
//     /* Interactive Sid */
//     LsapLoadString(hInstance, IDS_INTERACTIVE_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_INTERACTIVE_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   &LsapInteractiveSid);
//
//     /* Service Sid */
//     LsapLoadString(hInstance, IDS_SERVICE_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_SERVICE_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   &LsapServiceSid);
//
//     /* Anonymous Logon Sid */
//     LsapLoadString(hInstance, IDS_ANONYMOUS_LOGON_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_ANONYMOUS_LOGON_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Proxy Sid */
//     LsapLoadString(hInstance, IDS_PROXY_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_PROXY_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Enterprise Controllers Sid */
//     LsapLoadString(hInstance, IDS_ENTERPRISE_CONTROLLERS_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_ENTERPRISE_CONTROLLERS_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Principal Self Sid */
//     LsapLoadString(hInstance, IDS_PRINCIPAL_SELF_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_PRINCIPAL_SELF_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Authenticated Users Sid */
//     LsapLoadString(hInstance, IDS_AUTHENTICATED_USER_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_AUTHENTICATED_USER_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Restricted Code Sid */
//     LsapLoadString(hInstance, IDS_RESTRICTED_CODE_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_RESTRICTED_CODE_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Terminal Server Sid */
//     LsapLoadString(hInstance, IDS_TERMINAL_SERVER_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_TERMINAL_SERVER_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Remote Logon Sid */
//     LsapLoadString(hInstance, IDS_REMOTE_LOGON_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_REMOTE_LOGON_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* This Organization Sid */
//     LsapLoadString(hInstance, IDS_THIS_ORGANIZATION_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_THIS_ORGANIZATION_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Local System Sid */
//     LsapLoadString(hInstance, IDS_LOCAL_SYSTEM_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_LOCAL_SYSTEM_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   &LsapLocalSystemSid);
//
//     /* Local Service Sid */
//     LsapLoadString(hInstance, IDS_LOCAL_SERVICE_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_LOCAL_SERVICE_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   L"LOCALSERVICE",
//                   L"NT AUTHORITY",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Network Service Sid */
//     LsapLoadString(hInstance, IDS_NETWORK_SERVICE_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_NETWORK_SERVICE_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   L"NETWORKSERVICE",
//                   L"NT AUTHORITY",
//                   SidTypeWellKnownGroup,
//                   NULL);
//
//     /* Builtin Domain Sid */
//     LsapLoadString(hInstance, IDS_BUILTIN_DOMAIN_RID as u32, szAccountName, ARRAYSIZE(szAccountName));
//     LsapLoadString(hInstance, IDS_BUILTIN_DOMAIN_RID as u32, szDomainName, ARRAYSIZE(szDomainName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     LsapCreateSid(&NtAuthority,
//                   1,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeDomain,
//                   NULL);
//
//     /* Administrators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_ADMINS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_ADMINS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   &LsapAdministratorsSid);
//
//     /* Users Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_USERS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_USERS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Guests Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_GUESTS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_GUESTS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Power User Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_POWER_USERS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_POWER_USERS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Account Operators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_ACCOUNT_OPS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_ACCOUNT_OPS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* System Operators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_SYSTEM_OPS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_SYSTEM_OPS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Print Operators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_PRINT_OPS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_PRINT_OPS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Backup Operators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_BACKUP_OPS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_BACKUP_OPS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Replicators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_REPLICATOR as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_REPLICATOR;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* RAS Servers Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_RAS_SERVERS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_RAS_SERVERS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Pre-Windows 2000 Compatible Access Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_PREW2KCOMPACCESS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_PREW2KCOMPACCESS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Remote Desktop Users Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_REMOTE_DESKTOP_USERS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* Network Configuration Operators Alias Sid */
//     LsapLoadString(hInstance, IDS_ALIAS_RID_NETWORK_CONFIGURATION_OPS as u32, szAccountName, ARRAYSIZE(szAccountName));
//
//     SubAuthorities[0] = SECURITY_BUILTIN_DOMAIN_RID;
//     SubAuthorities[1] = DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS;
//     LsapCreateSid(&NtAuthority,
//                   2,
//                   SubAuthorities,
//                   szAccountName,
//                   szDomainName,
//                   SidTypeAlias,
//                   NULL);
//
//     /* FIXME: Add more well known sids */
//
//     return STATUS_SUCCESS;
// }
//
//
// PWELL_KNOWN_SID
// LsapLookupWellKnownSid(PSID Sid)
// {
//     PLIST_ENTRY ListEntry;
//     PWELL_KNOWN_SID Ptr;
//
//     ListEntry = WellKnownSidListHead.Flink;
//     while (ListEntry != &WellKnownSidListHead)
//     {
//         Ptr = CONTAINING_RECORD(ListEntry,
//                                 WELL_KNOWN_SID,
//                                 ListEntry);
//         if (RtlEqualSid(Sid, Ptr->Sid))
//         {
//             return Ptr;
//         }
//
//         ListEntry = ListEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// PWELL_KNOWN_SID
// LsapLookupIsolatedWellKnownName(PUNICODE_STRING AccountName)
// {
//     PLIST_ENTRY ListEntry;
//     PWELL_KNOWN_SID Ptr;
//
//     ListEntry = WellKnownSidListHead.Flink;
//     while (ListEntry != &WellKnownSidListHead)
//     {
//         Ptr = CONTAINING_RECORD(ListEntry,
//                                 WELL_KNOWN_SID,
//                                 ListEntry);
//         if (RtlEqualUnicodeString(AccountName, &Ptr->AccountName, TRUE))
//         {
//             return Ptr;
//         }
//
//         ListEntry = ListEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// PWELL_KNOWN_SID
// LsapLookupFullyQualifiedWellKnownName(PUNICODE_STRING AccountName,
//                                       PUNICODE_STRING DomainName)
// {
//     PLIST_ENTRY ListEntry;
//     PWELL_KNOWN_SID Ptr;
//
//     ListEntry = WellKnownSidListHead.Flink;
//     while (ListEntry != &WellKnownSidListHead)
//     {
//         Ptr = CONTAINING_RECORD(ListEntry,
//                                 WELL_KNOWN_SID,
//                                 ListEntry);
//         if (RtlEqualUnicodeString(AccountName, &Ptr->AccountName, TRUE) &&
//             RtlEqualUnicodeString(DomainName, &Ptr->DomainName, TRUE))
//         {
//             return Ptr;
//         }
//
//         ListEntry = ListEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// static
// NTSTATUS
// LsapSplitNames(DWORD Count,
//                PRPC_UNICODE_STRING Names,
//                PRPC_UNICODE_STRING *DomainNames,
//                PRPC_UNICODE_STRING *AccountNames)
// {
//     PRPC_UNICODE_STRING DomainsBuffer = NULL;
//     PRPC_UNICODE_STRING AccountsBuffer = NULL;
//     ULONG DomainLength;
//     ULONG AccountLength;
//     ULONG i;
//     LPWSTR Ptr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DomainsBuffer = MIDL_user_allocate(Count * sizeof(RPC_UNICODE_STRING));
//     if (DomainsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     AccountsBuffer = MIDL_user_allocate(Count * sizeof(RPC_UNICODE_STRING));
//     if (AccountsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     for (i = 0; i < Count; i++)
//     {
// //TRACE("Name: %wZ\n", &Names[i]);
//
//         Ptr = wcschr(Names[i].Buffer, L'\\');
//         if (Ptr == NULL)
//         {
//             AccountLength = Names[i].Length / sizeof(WCHAR);
//
//             AccountsBuffer[i].Length = Names[i].Length;
//             AccountsBuffer[i].MaximumLength = AccountsBuffer[i].Length + sizeof(WCHAR);
//             AccountsBuffer[i].Buffer = MIDL_user_allocate(AccountsBuffer[i].MaximumLength);
//             if (AccountsBuffer[i].Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             CopyMemory(AccountsBuffer[i].Buffer,
//                        Names[i].Buffer,
//                        AccountsBuffer[i].Length);
//             AccountsBuffer[i].Buffer[AccountLength] = UNICODE_NULL;
//
// //TRACE("Account name: %wZ\n", &AccountsBuffer[i]);
//         }
//         else
//         {
//             DomainLength = (ULONG)(ULONG_PTR)(Ptr - Names[i].Buffer);
//             AccountLength = (Names[i].Length / sizeof(WCHAR)) - DomainLength - 1;
// //TRACE("DomainLength: %u\n", DomainLength);
// //TRACE("AccountLength: %u\n", AccountLength);
//
//             if (DomainLength > 0)
//             {
//                 DomainsBuffer[i].Length = (USHORT)DomainLength * sizeof(WCHAR);
//                 DomainsBuffer[i].MaximumLength = DomainsBuffer[i].Length + sizeof(WCHAR);
//                 DomainsBuffer[i].Buffer = MIDL_user_allocate(DomainsBuffer[i].MaximumLength);
//                 if (DomainsBuffer[i].Buffer == NULL)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto done;
//                 }
//
//                 CopyMemory(DomainsBuffer[i].Buffer,
//                            Names[i].Buffer,
//                            DomainsBuffer[i].Length);
//                 DomainsBuffer[i].Buffer[DomainLength] = UNICODE_NULL;
//
// //TRACE("Domain name: %wZ\n", &DomainsBuffer[i]);
//             }
//
//             AccountsBuffer[i].Length = (USHORT)AccountLength * sizeof(WCHAR);
//             AccountsBuffer[i].MaximumLength = AccountsBuffer[i].Length + sizeof(WCHAR);
//             AccountsBuffer[i].Buffer = MIDL_user_allocate(AccountsBuffer[i].MaximumLength);
//             if (AccountsBuffer[i].Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             CopyMemory(AccountsBuffer[i].Buffer,
//                        &(Names[i].Buffer[DomainLength + 1]),
//                        AccountsBuffer[i].Length);
//             AccountsBuffer[i].Buffer[AccountLength] = UNICODE_NULL;
//
// //TRACE("Account name: %wZ\n", &AccountsBuffer[i]);
//         }
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (AccountsBuffer != NULL)
//         {
//             for (i = 0; i < Count; i++)
//             {
//                 if (AccountsBuffer[i].Buffer != NULL)
//                     MIDL_user_free(AccountsBuffer[i].Buffer);
//             }
//
//             MIDL_user_free(AccountsBuffer);
//         }
//
//         if (DomainsBuffer != NULL)
//         {
//             for (i = 0; i < Count; i++)
//             {
//                 if (DomainsBuffer[i].Buffer != NULL)
//                     MIDL_user_free(DomainsBuffer[i].Buffer);
//             }
//
//             MIDL_user_free(DomainsBuffer);
//         }
//     }
//     else
//     {
//         *DomainNames = DomainsBuffer;
//         *AccountNames = AccountsBuffer;
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// LsapAddDomainToDomainsList(PLSAPR_REFERENCED_DOMAIN_LIST ReferencedDomains,
//                            PUNICODE_STRING Name,
//                            PSID Sid,
//                            PULONG Index)
// {
//     ULONG i;
//
//     i = 0;
//     while (i < ReferencedDomains->Entries &&
//            ReferencedDomains->Domains[i].Sid != NULL)
//     {
//         if (RtlEqualSid(Sid, ReferencedDomains->Domains[i].Sid))
//         {
//             *Index = i;
//             return STATUS_SUCCESS;
//         }
//
//         i++;
//     }
//
//     ReferencedDomains->Domains[i].Sid = MIDL_user_allocate(RtlLengthSid(Sid));
//     if (ReferencedDomains->Domains[i].Sid == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlCopySid(RtlLengthSid(Sid), ReferencedDomains->Domains[i].Sid, Sid);
//
//     ReferencedDomains->Domains[i].Name.Length = Name->Length;
//     ReferencedDomains->Domains[i].Name.MaximumLength = Name->MaximumLength;
//     ReferencedDomains->Domains[i].Name.Buffer = MIDL_user_allocate(Name->MaximumLength);
//     if (ReferencedDomains->Domains[i].Name.Buffer == NULL)
//     {
//         MIDL_user_free(ReferencedDomains->Domains[i].Sid);
//         ReferencedDomains->Domains[i].Sid = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(ReferencedDomains->Domains[i].Name.Buffer,
//                   Name->Buffer,
//                   Name->MaximumLength);
//
//     ReferencedDomains->Entries++;
//     *Index = i;
//
//     return STATUS_SUCCESS;
// }
//
//
// static NTSTATUS
// LsapAddAuthorityToDomainsList(
//     PLSAPR_REFERENCED_DOMAIN_LIST ReferencedDomains,
//     PSID Sid,
//     PULONG Index)
// {
//     SID AuthoritySid;
//     ULONG i;
//
//     RtlInitializeSid(&AuthoritySid,
//                      RtlIdentifierAuthoritySid(Sid),
//                      0);
//
//     i = 0;
//     while (i < ReferencedDomains->Entries &&
//            ReferencedDomains->Domains[i].Sid != NULL)
//     {
//         if (RtlEqualSid(&AuthoritySid, ReferencedDomains->Domains[i].Sid))
//         {
//             *Index = i;
//             return STATUS_SUCCESS;
//         }
//
//         i++;
//     }
//
//     ReferencedDomains->Domains[i].Sid = MIDL_user_allocate(RtlLengthSid(&AuthoritySid));
//     if (ReferencedDomains->Domains[i].Sid == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlCopySid(RtlLengthSid(&AuthoritySid), ReferencedDomains->Domains[i].Sid, &AuthoritySid);
//
//     ReferencedDomains->Domains[i].Name.Length = 0;
//     ReferencedDomains->Domains[i].Name.MaximumLength = sizeof(WCHAR);
//     ReferencedDomains->Domains[i].Name.Buffer = MIDL_user_allocate(sizeof(WCHAR));
//     if (ReferencedDomains->Domains[i].Name.Buffer == NULL)
//     {
//         MIDL_user_free(ReferencedDomains->Domains[i].Sid);
//         ReferencedDomains->Domains[i].Sid = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     ReferencedDomains->Domains[i].Name.Buffer[0] = UNICODE_NULL;
//
//     ReferencedDomains->Entries++;
//     *Index = i;
//
//     return STATUS_SUCCESS;
// }
//
//
// static BOOLEAN
// LsapIsPrefixSid(IN PSID PrefixSid,
//                 IN PSID Sid)
// {
//     PISID Sid1 = PrefixSid, Sid2 = Sid;
//     ULONG i;
//
//     if (Sid1->Revision != Sid2->Revision)
//         return FALSE;
//
//     if ((Sid1->IdentifierAuthority.Value[0] != Sid2->IdentifierAuthority.Value[0]) ||
//         (Sid1->IdentifierAuthority.Value[1] != Sid2->IdentifierAuthority.Value[1]) ||
//         (Sid1->IdentifierAuthority.Value[2] != Sid2->IdentifierAuthority.Value[2]) ||
//         (Sid1->IdentifierAuthority.Value[3] != Sid2->IdentifierAuthority.Value[3]) ||
//         (Sid1->IdentifierAuthority.Value[4] != Sid2->IdentifierAuthority.Value[4]) ||
//         (Sid1->IdentifierAuthority.Value[5] != Sid2->IdentifierAuthority.Value[5]))
//         return FALSE;
//
//     if (Sid1->SubAuthorityCount >= Sid2->SubAuthorityCount)
//         return FALSE;
//
//     if (Sid1->SubAuthorityCount == 0)
//         return TRUE;
//
//     for (i = 0; i < Sid1->SubAuthorityCount; i++)
//     {
//         if (Sid1->SubAuthority[i] != Sid2->SubAuthority[i])
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// ULONG
// LsapGetRelativeIdFromSid(PSID Sid_)
// {
//     PISID Sid = Sid_;
//
//     if (Sid->SubAuthorityCount != 0)
//         return Sid->SubAuthority[Sid->SubAuthorityCount - 1];
//
//     return 0;
// }
//
//
// static PSID
// CreateSidFromSidAndRid(PSID SrcSid,
//                        ULONG RelativeId)
// {
//     UCHAR RidCount;
//     PSID DstSid;
//     ULONG i;
//     ULONG DstSidSize;
//     PULONG p, q;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     if (RidCount >= 8)
//         return NULL;
//
//     DstSidSize = RtlLengthRequiredSid(RidCount + 1);
//
//     DstSid = MIDL_user_allocate(DstSidSize);
//     if (DstSid == NULL)
//         return NULL;
//
//     RtlInitializeSid(DstSid,
//                      RtlIdentifierAuthoritySid(SrcSid),
//                      RidCount + 1);
//
//     for (i = 0; i < (ULONG)RidCount; i++)
//     {
//         p = RtlSubAuthoritySid(SrcSid, i);
//         q = RtlSubAuthoritySid(DstSid, i);
//         *q = *p;
//     }
//
//     q = RtlSubAuthoritySid(DstSid, (ULONG)RidCount);
//     *q = RelativeId;
//
//     return DstSid;
// }
//
//
// static PSID
// CreateDomainSidFromAccountSid(PSID AccountSid)
// {
//     UCHAR RidCount;
//     PSID DomainSid;
//     ULONG i;
//     ULONG DstSidSize;
//     PULONG p, q;
//
//     RidCount = *RtlSubAuthorityCountSid(AccountSid);
//     if (RidCount > 0)
//         RidCount--;
//
//     DstSidSize = RtlLengthRequiredSid(RidCount);
//
//     DomainSid = MIDL_user_allocate(DstSidSize);
//     if (DomainSid == NULL)
//         return NULL;
//
//     RtlInitializeSid(DomainSid,
//                      RtlIdentifierAuthoritySid(AccountSid),
//                      RidCount);
//
//     for (i = 0; i < (ULONG)RidCount; i++)
//     {
//         p = RtlSubAuthoritySid(AccountSid, i);
//         q = RtlSubAuthoritySid(DomainSid, i);
//         *q = *p;
//     }
//
//     return DomainSid;
// }
//
//
// static PSID
// LsapCopySid(PSID SrcSid)
// {
//     UCHAR RidCount;
//     PSID DstSid;
//     ULONG i;
//     ULONG DstSidSize;
//     PULONG p, q;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     DstSidSize = RtlLengthRequiredSid(RidCount);
//
//     DstSid = MIDL_user_allocate(DstSidSize);
//     if (DstSid == NULL)
//         return NULL;
//
//     RtlInitializeSid(DstSid,
//                      RtlIdentifierAuthoritySid(SrcSid),
//                      RidCount);
//
//     for (i = 0; i < (ULONG)RidCount; i++)
//     {
//         p = RtlSubAuthoritySid(SrcSid, i);
//         q = RtlSubAuthoritySid(DstSid, i);
//         *q = *p;
//     }
//
//     return DstSid;
// }
//
//
// static
// NTSTATUS
// LsapLookupIsolatedNames(DWORD Count,
//                         PRPC_UNICODE_STRING DomainNames,
//                         PRPC_UNICODE_STRING AccountNames,
//                         PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                         PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                         PULONG Mapped)
// {
//     UNICODE_STRING EmptyDomainName = RTL_CONSTANT_STRING(L"");
//     PWELL_KNOWN_SID ptr, ptr2;
//     PSID DomainSid;
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore fully qualified account names */
//         if (DomainNames[i].Length != 0)
//             continue;
//
//         TRACE("Mapping name: %wZ\n", &AccountNames[i]);
//
//         /* Look-up all well-known names */
//         ptr = LsapLookupIsolatedWellKnownName((PUNICODE_STRING)&AccountNames[i]);
//         if (ptr != NULL)
//         {
//             SidsBuffer[i].Use = ptr->Use;
//             SidsBuffer[i].Sid = LsapCopySid(ptr->Sid);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             if (ptr->Use == SidTypeDomain)
//             {
//                 Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                     &ptr->AccountName,
//                                                     ptr->Sid,
//                                                     &DomainIndex);
//                 if (!NT_SUCCESS(Status))
//                     goto done;
//
//                 SidsBuffer[i].DomainIndex = DomainIndex;
//             }
//             else
//             {
//                 ptr2= LsapLookupIsolatedWellKnownName(&ptr->DomainName);
//                 if (ptr2 != NULL)
//                 {
//                     Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                         &ptr2->AccountName,
//                                                         ptr2->Sid,
//                                                         &DomainIndex);
//                     if (!NT_SUCCESS(Status))
//                         goto done;
//
//                     SidsBuffer[i].DomainIndex = DomainIndex;
//                 }
//                 else
//                 {
//                     DomainSid = CreateDomainSidFromAccountSid(ptr->Sid);
//                     if (DomainSid == NULL)
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         goto done;
//                     }
//
//                     Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                         &EmptyDomainName,
//                                                         DomainSid,
//                                                         &DomainIndex);
//
//                     if (DomainSid != NULL)
//                     {
//                         MIDL_user_free(DomainSid);
//                         DomainSid = NULL;
//                     }
//
//                     if (!NT_SUCCESS(Status))
//                         goto done;
//
//                     SidsBuffer[i].DomainIndex = DomainIndex;
//                 }
//             }
//
//             (*Mapped)++;
//             continue;
//         }
//
//         /* Look-up the built-in domain */
//         if (RtlEqualUnicodeString((PUNICODE_STRING)&AccountNames[i], &BuiltinDomainName, TRUE))
//         {
//             SidsBuffer[i].Use = SidTypeDomain;
//             SidsBuffer[i].Sid = LsapCopySid(BuiltinDomainSid);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &BuiltinDomainName,
//                                                 BuiltinDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//             continue;
//         }
//
//         /* Look-up the account domain */
//         if (RtlEqualUnicodeString((PUNICODE_STRING)&AccountNames[i], &AccountDomainName, TRUE))
//         {
//             SidsBuffer[i].Use = SidTypeDomain;
//             SidsBuffer[i].Sid = LsapCopySid(AccountDomainSid);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &AccountDomainName,
//                                                 AccountDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//             continue;
//         }
//
//         /* FIXME: Look-up the primary domain */
//
//         /* FIXME: Look-up the trusted domains */
//
//     }
//
// done:
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapLookupIsolatedBuiltinNames(DWORD Count,
//                                PRPC_UNICODE_STRING DomainNames,
//                                PRPC_UNICODE_STRING AccountNames,
//                                PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                                PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                                PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             BuiltinDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore fully qualified account names */
//         if (DomainNames[i].Length != 0)
//             continue;
//
//         TRACE("Mapping name: %wZ\n", &AccountNames[i]);
//
//         Status = SamrLookupNamesInDomain(DomainHandle,
//                                          1,
//                                          &AccountNames[i],
//                                          &RelativeIds,
//                                          &Use);
//         if (NT_SUCCESS(Status))
//         {
//             TRACE("Found relative ID: %lu\n", RelativeIds.Element[0]);
//
//             SidsBuffer[i].Use = Use.Element[0];
//             SidsBuffer[i].Sid = CreateSidFromSidAndRid(BuiltinDomainSid,
//                                                        RelativeIds.Element[0]);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &BuiltinDomainName,
//                                                 BuiltinDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//         }
//
//         SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//         SamIFree_SAMPR_ULONG_ARRAY(&Use);
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapLookupIsolatedAccountNames(DWORD Count,
//                                PRPC_UNICODE_STRING DomainNames,
//                                PRPC_UNICODE_STRING AccountNames,
//                                PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                                PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                                PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("()\n");
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             AccountDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore fully qualified account names */
//         if (DomainNames[i].Length != 0)
//             continue;
//
//         TRACE("Mapping name: %wZ\n", &AccountNames[i]);
//
//         Status = SamrLookupNamesInDomain(DomainHandle,
//                                          1,
//                                          &AccountNames[i],
//                                          &RelativeIds,
//                                          &Use);
//         if (NT_SUCCESS(Status))
//         {
//             TRACE("Found relative ID: %lu\n", RelativeIds.Element[0]);
//
//             SidsBuffer[i].Use = Use.Element[0];
//             SidsBuffer[i].Sid = CreateSidFromSidAndRid(AccountDomainSid,
//                                                        RelativeIds.Element[0]);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &AccountDomainName,
//                                                 AccountDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//         }
//
//         SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//         SamIFree_SAMPR_ULONG_ARRAY(&Use);
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapLookupFullyQualifiedWellKnownNames(DWORD Count,
//                                        PRPC_UNICODE_STRING DomainNames,
//                                        PRPC_UNICODE_STRING AccountNames,
//                                        PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                                        PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                                        PULONG Mapped)
// {
//     UNICODE_STRING EmptyDomainName = RTL_CONSTANT_STRING(L"");
//     PWELL_KNOWN_SID ptr, ptr2;
//     PSID DomainSid;
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore isolated account names */
//         if (DomainNames[i].Length == 0)
//             continue;
//
//         TRACE("Mapping name: %wZ\\%wZ\n", &DomainNames[i], &AccountNames[i]);
//
//         /* Look-up all well-known names */
//         ptr = LsapLookupFullyQualifiedWellKnownName((PUNICODE_STRING)&AccountNames[i],
//                                                     (PUNICODE_STRING)&DomainNames[i]);
//         if (ptr != NULL)
//         {
//             TRACE("Found it! (%wZ\\%wZ)\n", &ptr->DomainName, &ptr->AccountName);
//
//             SidsBuffer[i].Use = ptr->Use;
//             SidsBuffer[i].Sid = LsapCopySid(ptr->Sid);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             if (ptr->Use == SidTypeDomain)
//             {
//                 Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                     &ptr->AccountName,
//                                                     ptr->Sid,
//                                                     &DomainIndex);
//                 if (!NT_SUCCESS(Status))
//                     goto done;
//
//                 SidsBuffer[i].DomainIndex = DomainIndex;
//             }
//             else
//             {
//                 ptr2= LsapLookupIsolatedWellKnownName(&ptr->DomainName);
//                 if (ptr2 != NULL)
//                 {
//                     Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                         &ptr2->AccountName,
//                                                         ptr2->Sid,
//                                                         &DomainIndex);
//                     if (!NT_SUCCESS(Status))
//                         goto done;
//
//                     SidsBuffer[i].DomainIndex = DomainIndex;
//                 }
//                 else
//                 {
//                     DomainSid = CreateDomainSidFromAccountSid(ptr->Sid);
//                     if (DomainSid == NULL)
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         goto done;
//                     }
//
//                     Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                         &EmptyDomainName,
//                                                         DomainSid,
//                                                         &DomainIndex);
//
//                     if (DomainSid != NULL)
//                     {
//                         MIDL_user_free(DomainSid);
//                         DomainSid = NULL;
//                     }
//
//                     if (!NT_SUCCESS(Status))
//                         goto done;
//
//                     SidsBuffer[i].DomainIndex = DomainIndex;
//                 }
//             }
//
//             (*Mapped)++;
//             continue;
//         }
//     }
//
// done:
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapLookupBuiltinNames(DWORD Count,
//                        PRPC_UNICODE_STRING DomainNames,
//                        PRPC_UNICODE_STRING AccountNames,
//                        PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                        PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                        PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             BuiltinDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore isolated account names */
//         if (DomainNames[i].Length == 0)
//             continue;
//
//         if (!RtlEqualUnicodeString((PUNICODE_STRING)&DomainNames[i], &BuiltinDomainName, TRUE))
//             continue;
//
//         TRACE("Mapping name: %wZ\\%wZ\n", &DomainNames[i], &AccountNames[i]);
//
//         Status = SamrLookupNamesInDomain(DomainHandle,
//                                          1,
//                                          &AccountNames[i],
//                                          &RelativeIds,
//                                          &Use);
//         if (NT_SUCCESS(Status))
//         {
//             SidsBuffer[i].Use = Use.Element[0];
//             SidsBuffer[i].Sid = CreateSidFromSidAndRid(BuiltinDomainSid,
//                                                        RelativeIds.Element[0]);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &BuiltinDomainName,
//                                                 BuiltinDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//         }
//
//         SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//         SamIFree_SAMPR_ULONG_ARRAY(&Use);
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapLookupAccountNames(DWORD Count,
//                        PRPC_UNICODE_STRING DomainNames,
//                        PRPC_UNICODE_STRING AccountNames,
//                        PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                        PLSAPR_TRANSLATED_SID_EX2 SidsBuffer,
//                        PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             AccountDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < Count; i++)
//     {
//         /* Ignore names which were already mapped */
//         if (SidsBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         /* Ignore isolated account names */
//         if (DomainNames[i].Length == 0)
//             continue;
//
//         if (!RtlEqualUnicodeString((PUNICODE_STRING)&DomainNames[i], &AccountDomainName, TRUE))
//             continue;
//
//         TRACE("Mapping name: %wZ\\%wZ\n", &DomainNames[i], &AccountNames[i]);
//
//         Status = SamrLookupNamesInDomain(DomainHandle,
//                                          1,
//                                          &AccountNames[i],
//                                          &RelativeIds,
//                                          &Use);
//         if (NT_SUCCESS(Status))
//         {
//             SidsBuffer[i].Use = Use.Element[0];
//             SidsBuffer[i].Sid = CreateSidFromSidAndRid(AccountDomainSid,
//                                                        RelativeIds.Element[0]);
//             if (SidsBuffer[i].Sid == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             SidsBuffer[i].DomainIndex = -1;
//             SidsBuffer[i].Flags = 0;
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &AccountDomainName,
//                                                 AccountDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             SidsBuffer[i].DomainIndex = DomainIndex;
//
//             (*Mapped)++;
//         }
//
//         SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//         SamIFree_SAMPR_ULONG_ARRAY(&Use);
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapLookupNames(DWORD Count,
//                 PRPC_UNICODE_STRING Names,
//                 PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//                 PLSAPR_TRANSLATED_SIDS_EX2 TranslatedSids,
//                 LSAP_LOOKUP_LEVEL LookupLevel,
//                 DWORD *MappedCount,
//                 DWORD LookupOptions,
//                 DWORD ClientRevision)
// {
//     PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer = NULL;
//     PLSAPR_TRANSLATED_SID_EX2 SidsBuffer = NULL;
//     PRPC_UNICODE_STRING DomainNames = NULL;
//     PRPC_UNICODE_STRING AccountNames = NULL;
//     ULONG SidsBufferLength;
//     ULONG i;
//     ULONG Mapped = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
// //TRACE("()\n");
//
//     TranslatedSids->Entries = 0;
//     TranslatedSids->Sids = NULL;
//     *ReferencedDomains = NULL;
//
//     SidsBufferLength = Count * sizeof(LSAPR_TRANSLATED_SID_EX2);
//     SidsBuffer = MIDL_user_allocate(SidsBufferLength);
//     if (SidsBuffer == NULL)
//     {
// //TRACE("\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     DomainsBuffer = MIDL_user_allocate(sizeof(LSAPR_REFERENCED_DOMAIN_LIST));
//     if (DomainsBuffer == NULL)
//     {
// //TRACE("\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     DomainsBuffer->Domains = MIDL_user_allocate(Count * sizeof(LSA_TRUST_INFORMATION));
//     if (DomainsBuffer->Domains == NULL)
//     {
// //TRACE("\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//     DomainsBuffer->Entries = 0;
//     DomainsBuffer->MaxEntries = Count;
//
//     for (i = 0; i < Count; i++)
//     {
//         SidsBuffer[i].Use = SidTypeUnknown;
//         SidsBuffer[i].Sid = NULL;
//         SidsBuffer[i].DomainIndex = -1;
//         SidsBuffer[i].Flags = 0;
//     }
//
//     Status = LsapSplitNames(Count,
//                             Names,
//                             &DomainNames,
//                             &AccountNames);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsapSplitNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//
//     Status = LsapLookupIsolatedNames(Count,
//                                      DomainNames,
//                                      AccountNames,
//                                      DomainsBuffer,
//                                      SidsBuffer,
//                                      &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupIsolatedNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
//
//     Status = LsapLookupIsolatedBuiltinNames(Count,
//                                             DomainNames,
//                                             AccountNames,
//                                             DomainsBuffer,
//                                             SidsBuffer,
//                                             &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupIsolatedBuiltinNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
//
//     Status = LsapLookupIsolatedAccountNames(Count,
//                                             DomainNames,
//                                             AccountNames,
//                                             DomainsBuffer,
//                                             SidsBuffer,
//                                             &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupIsolatedAccountNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
//     Status = LsapLookupFullyQualifiedWellKnownNames(Count,
//                                                     DomainNames,
//                                                     AccountNames,
//                                                     DomainsBuffer,
//                                                     SidsBuffer,
//                                                     &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupFullyQualifiedWellKnownNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
//     Status = LsapLookupBuiltinNames(Count,
//                                     DomainNames,
//                                     AccountNames,
//                                     DomainsBuffer,
//                                     SidsBuffer,
//                                     &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupBuiltinNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
//
//     Status = LsapLookupAccountNames(Count,
//                                     DomainNames,
//                                     AccountNames,
//                                     DomainsBuffer,
//                                     SidsBuffer,
//                                     &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//     {
//         TRACE("LsapLookupAccountNames failed! (Status %lx)\n", Status);
//         goto done;
//     }
//
//     if (Mapped == Count)
//         goto done;
//
// done:
// //    TRACE("done: Status %lx\n", Status);
//
//     if (DomainNames != NULL)
//     {
// //TRACE("Free DomainNames\n");
//         for (i = 0; i < Count; i++)
//         {
//             if (DomainNames[i].Buffer != NULL)
//                 MIDL_user_free(DomainNames[i].Buffer);
//         }
//
//         MIDL_user_free(DomainNames);
//     }
//
//     if (AccountNames != NULL)
//     {
// //TRACE("Free AccountNames\n");
//         for (i = 0; i < Count; i++)
//         {
// //TRACE("i: %lu\n", i);
//             if (AccountNames[i].Buffer != NULL)
//             {
//                 MIDL_user_free(AccountNames[i].Buffer);
//             }
//         }
//
//         MIDL_user_free(AccountNames);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
// //TRACE("Failure!\n");
//
// //TRACE("Free DomainsBuffer\n");
//         if (DomainsBuffer != NULL)
//         {
//             if (DomainsBuffer->Domains != NULL)
//                 MIDL_user_free(DomainsBuffer->Domains);
//
//             MIDL_user_free(DomainsBuffer);
//         }
//
// //TRACE("Free SidsBuffer\n");
//         if (SidsBuffer != NULL)
//             MIDL_user_free(SidsBuffer);
//     }
//     else
//     {
// //TRACE("Success!\n");
//
//         *ReferencedDomains = DomainsBuffer;
//         TranslatedSids->Entries = Count;
//         TranslatedSids->Sids = SidsBuffer;
//         *MappedCount = Mapped;
//
//         if (Mapped == 0)
//             Status = STATUS_NONE_MAPPED;
//         else if (Mapped < Count)
//             Status = STATUS_SOME_NOT_MAPPED;
//     }
//
// //    TRACE("done: Status %lx\n", Status);
//
//     return Status;
// }
//
//
// static NTSTATUS
// LsapLookupWellKnownSids(PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//                         PLSAPR_TRANSLATED_NAME_EX NamesBuffer,
//                         PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                         PULONG Mapped)
// {
//     PWELL_KNOWN_SID ptr, ptr2;
//     LPWSTR SidString = NULL;
//     ULONG DomainIndex;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     for (i = 0; i < SidEnumBuffer->Entries; i++)
//     {
//         /* Ignore SIDs which are already mapped */
//         if (NamesBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         ConvertSidToStringSidW(SidEnumBuffer->SidInfo[i].Sid, &SidString);
//         TRACE("Mapping SID: %S\n", SidString);
//         LocalFree(SidString);
//         SidString = NULL;
//
//         ptr = LsapLookupWellKnownSid(SidEnumBuffer->SidInfo[i].Sid);
//         if (ptr != NULL)
//         {
//             NamesBuffer[i].Use = ptr->Use;
//             NamesBuffer[i].Flags = 0;
//
//             NamesBuffer[i].Name.Length = ptr->AccountName.Length;
//             NamesBuffer[i].Name.MaximumLength = ptr->AccountName.MaximumLength;
//             NamesBuffer[i].Name.Buffer = MIDL_user_allocate(ptr->AccountName.MaximumLength);
//             if (NamesBuffer[i].Name.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             RtlCopyMemory(NamesBuffer[i].Name.Buffer, ptr->AccountName.Buffer, ptr->AccountName.MaximumLength);
//
//             if (ptr->DomainName.Length == 0)
//             {
//                 Status = LsapAddAuthorityToDomainsList(DomainsBuffer,
//                                                        SidEnumBuffer->SidInfo[i].Sid,
//                                                        &DomainIndex);
//                 if (!NT_SUCCESS(Status))
//                     goto done;
//
//                 NamesBuffer[i].DomainIndex = DomainIndex;
//             }
//             else
//             {
//                 ptr2= LsapLookupIsolatedWellKnownName(&ptr->DomainName);
//                 if (ptr2 != NULL)
//                 {
//                     Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                         &ptr2->AccountName,
//                                                         ptr2->Sid,
//                                                         &DomainIndex);
//                     if (!NT_SUCCESS(Status))
//                         goto done;
//
//                     NamesBuffer[i].DomainIndex = DomainIndex;
//                 }
//             }
//
//             TRACE("Mapped to: %wZ\n", &NamesBuffer[i].Name);
//
//             (*Mapped)++;
//         }
//     }
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// LsapLookupBuiltinDomainSids(PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//                             PLSAPR_TRANSLATED_NAME_EX NamesBuffer,
//                             PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                             PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_RETURNED_USTRING_ARRAY Names = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     LPWSTR SidString = NULL;
//     ULONG DomainIndex;
//     ULONG RelativeIds[1];
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             BuiltinDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < SidEnumBuffer->Entries; i++)
//     {
//         /* Ignore SIDs which are already mapped */
//         if (NamesBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         ConvertSidToStringSidW(SidEnumBuffer->SidInfo[i].Sid, &SidString);
//         TRACE("Mapping SID: %S\n", SidString);
//         LocalFree(SidString);
//         SidString = NULL;
//
//         if (RtlEqualSid(BuiltinDomainSid, SidEnumBuffer->SidInfo[i].Sid))
//         {
//             TRACE("Found builtin domain!\n");
//
//             NamesBuffer[i].Use = SidTypeDomain;
//             NamesBuffer[i].Flags = 0;
//
//             NamesBuffer[i].Name.Length = BuiltinDomainName.Length;
//             NamesBuffer[i].Name.MaximumLength = BuiltinDomainName.MaximumLength;
//             NamesBuffer[i].Name.Buffer = MIDL_user_allocate(BuiltinDomainName.MaximumLength);
//             if (NamesBuffer[i].Name.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             RtlCopyMemory(NamesBuffer[i].Name.Buffer, BuiltinDomainName.Buffer, BuiltinDomainName.MaximumLength);
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &BuiltinDomainName,
//                                                 BuiltinDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             NamesBuffer[i].DomainIndex = DomainIndex;
//
//             TRACE("Mapped to: %wZ\n", &NamesBuffer[i].Name);
//
//             (*Mapped)++;
//         }
//         else if (LsapIsPrefixSid(BuiltinDomainSid, SidEnumBuffer->SidInfo[i].Sid))
//         {
//             TRACE("Found builtin domain account!\n");
//
//             RelativeIds[0] = LsapGetRelativeIdFromSid(SidEnumBuffer->SidInfo[i].Sid);
//
//             Status = SamrLookupIdsInDomain(DomainHandle,
//                                            1,
//                                            RelativeIds,
//                                            &Names,
//                                            &Use);
//             if (NT_SUCCESS(Status))
//             {
//                 NamesBuffer[i].Use = Use.Element[0];
//                 NamesBuffer[i].Flags = 0;
//
//                 NamesBuffer[i].Name.Length = Names.Element[0].Length;
//                 NamesBuffer[i].Name.MaximumLength = Names.Element[0].MaximumLength;
//                 NamesBuffer[i].Name.Buffer = MIDL_user_allocate(Names.Element[0].MaximumLength);
//                 if (NamesBuffer[i].Name.Buffer == NULL)
//                 {
//                     SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&Names);
//                     SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto done;
//                 }
//
//                 RtlCopyMemory(NamesBuffer[i].Name.Buffer,
//                               Names.Element[0].Buffer,
//                               Names.Element[0].MaximumLength);
//
//                 SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&Names);
//                 SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//                 Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                     &BuiltinDomainName,
//                                                     BuiltinDomainSid,
//                                                     &DomainIndex);
//                 if (!NT_SUCCESS(Status))
//                     goto done;
//
//                 NamesBuffer[i].DomainIndex = DomainIndex;
//
//                 TRACE("Mapped to: %wZ\n", &NamesBuffer[i].Name);
//
//                 (*Mapped)++;
//             }
//         }
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static NTSTATUS
// LsapLookupAccountDomainSids(PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//                             PLSAPR_TRANSLATED_NAME_EX NamesBuffer,
//                             PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer,
//                             PULONG Mapped)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_RETURNED_USTRING_ARRAY Names = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     LPWSTR SidString = NULL;
//     ULONG DomainIndex;
//     ULONG RelativeIds[1];
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             AccountDomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     for (i = 0; i < SidEnumBuffer->Entries; i++)
//     {
//         /* Ignore SIDs which are already mapped */
//         if (NamesBuffer[i].Use != SidTypeUnknown)
//             continue;
//
//         ConvertSidToStringSidW(SidEnumBuffer->SidInfo[i].Sid, &SidString);
//         TRACE("Mapping SID: %S\n", SidString);
//         LocalFree(SidString);
//         SidString = NULL;
//
//         if (RtlEqualSid(AccountDomainSid, SidEnumBuffer->SidInfo[i].Sid))
//         {
//             TRACE("Found account domain!\n");
//
//             NamesBuffer[i].Use = SidTypeDomain;
//             NamesBuffer[i].Flags = 0;
//
//             NamesBuffer[i].Name.Length = AccountDomainName.Length;
//             NamesBuffer[i].Name.MaximumLength = AccountDomainName.MaximumLength;
//             NamesBuffer[i].Name.Buffer = MIDL_user_allocate(AccountDomainName.MaximumLength);
//             if (NamesBuffer[i].Name.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             RtlCopyMemory(NamesBuffer[i].Name.Buffer, AccountDomainName.Buffer, AccountDomainName.MaximumLength);
//
//             Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                 &AccountDomainName,
//                                                 AccountDomainSid,
//                                                 &DomainIndex);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             NamesBuffer[i].DomainIndex = DomainIndex;
//
//             TRACE("Mapped to: %wZ\n", &NamesBuffer[i].Name);
//
//             (*Mapped)++;
//         }
//         else if (LsapIsPrefixSid(AccountDomainSid, SidEnumBuffer->SidInfo[i].Sid))
//         {
//             TRACE("Found account domain account!\n");
//
//             RelativeIds[0] = LsapGetRelativeIdFromSid(SidEnumBuffer->SidInfo[i].Sid);
//
//             Status = SamrLookupIdsInDomain(DomainHandle,
//                                            1,
//                                            RelativeIds,
//                                            &Names,
//                                            &Use);
//             if (NT_SUCCESS(Status))
//             {
//                 NamesBuffer[i].Use = Use.Element[0];
//                 NamesBuffer[i].Flags = 0;
//
//                 NamesBuffer[i].Name.Length = Names.Element[0].Length;
//                 NamesBuffer[i].Name.MaximumLength = Names.Element[0].MaximumLength;
//                 NamesBuffer[i].Name.Buffer = MIDL_user_allocate(Names.Element[0].MaximumLength);
//                 if (NamesBuffer[i].Name.Buffer == NULL)
//                 {
//                     SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&Names);
//                     SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto done;
//                 }
//
//                 RtlCopyMemory(NamesBuffer[i].Name.Buffer,
//                               Names.Element[0].Buffer,
//                               Names.Element[0].MaximumLength);
//
//                 SamIFree_SAMPR_RETURNED_USTRING_ARRAY(&Names);
//                 SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//                 Status = LsapAddDomainToDomainsList(DomainsBuffer,
//                                                     &AccountDomainName,
//                                                     AccountDomainSid,
//                                                     &DomainIndex);
//                 if (!NT_SUCCESS(Status))
//                     goto done;
//
//                 NamesBuffer[i].DomainIndex = DomainIndex;
//
//                 TRACE("Mapped to: %wZ\n", &NamesBuffer[i].Name);
//
//                 (*Mapped)++;
//             }
//         }
//     }
//
// done:
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapLookupSids(PLSAPR_SID_ENUM_BUFFER SidEnumBuffer,
//                PLSAPR_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//                PLSAPR_TRANSLATED_NAMES_EX TranslatedNames,
//                LSAP_LOOKUP_LEVEL LookupLevel,
//                DWORD *MappedCount,
//                DWORD LookupOptions,
//                DWORD ClientRevision)
// {
//     PLSAPR_REFERENCED_DOMAIN_LIST DomainsBuffer = NULL;
//     PLSAPR_TRANSLATED_NAME_EX NamesBuffer = NULL;
//     ULONG NamesBufferLength;
//     ULONG i;
//     ULONG Mapped = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     NamesBufferLength = SidEnumBuffer->Entries * sizeof(LSAPR_TRANSLATED_NAME_EX);
//     NamesBuffer = MIDL_user_allocate(NamesBufferLength);
//     if (NamesBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     DomainsBuffer = MIDL_user_allocate(sizeof(LSAPR_REFERENCED_DOMAIN_LIST));
//     if (DomainsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     DomainsBuffer->Domains = MIDL_user_allocate(SidEnumBuffer->Entries * sizeof(LSA_TRUST_INFORMATION));
//     if (DomainsBuffer->Domains == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     DomainsBuffer->Entries = 0;
//     DomainsBuffer->MaxEntries = SidEnumBuffer->Entries;
//
//     /* Initialize all name entries */
//     for (i = 0; i < SidEnumBuffer->Entries; i++)
//     {
//         NamesBuffer[i].Use = SidTypeUnknown;
//         NamesBuffer[i].Name.Length = 0;
//         NamesBuffer[i].Name.MaximumLength = 0;
//         NamesBuffer[i].Name.Buffer = NULL;
//         NamesBuffer[i].DomainIndex = -1;
//         NamesBuffer[i].Flags = 0;
//     }
//
//     /* Look-up well-known SIDs */
//     Status = LsapLookupWellKnownSids(SidEnumBuffer,
//                                      NamesBuffer,
//                                      DomainsBuffer,
//                                      &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//         goto done;
//
//     if (Mapped == SidEnumBuffer->Entries)
//         goto done;
//
//     /* Look-up builtin domain SIDs */
//     Status = LsapLookupBuiltinDomainSids(SidEnumBuffer,
//                                          NamesBuffer,
//                                          DomainsBuffer,
//                                          &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//         goto done;
//
//     if (Mapped == SidEnumBuffer->Entries)
//         goto done;
//
//     /* Look-up account domain SIDs */
//     Status = LsapLookupAccountDomainSids(SidEnumBuffer,
//                                          NamesBuffer,
//                                          DomainsBuffer,
//                                          &Mapped);
//     if (!NT_SUCCESS(Status) &&
//         Status != STATUS_NONE_MAPPED &&
//         Status != STATUS_SOME_NOT_MAPPED)
//         goto done;
//
//     if (Mapped == SidEnumBuffer->Entries)
//         goto done;
//
// done:
//     TRACE("done Status: %lx  Mapped: %lu\n", Status, Mapped);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (DomainsBuffer != NULL)
//         {
//             if (DomainsBuffer->Domains != NULL)
//                 MIDL_user_free(DomainsBuffer->Domains);
//
//             MIDL_user_free(DomainsBuffer);
//         }
//
//         if (NamesBuffer != NULL)
//             MIDL_user_free(NamesBuffer);
//     }
//     else
//     {
//         *ReferencedDomains = DomainsBuffer;
//         TranslatedNames->Entries = SidEnumBuffer->Entries;
//         TranslatedNames->Names = NamesBuffer;
//         *MappedCount = Mapped;
//
//         if (Mapped == 0)
//             Status = STATUS_NONE_MAPPED;
//         else if (Mapped < SidEnumBuffer->Entries)
//             Status = STATUS_SOME_NOT_MAPPED;
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
