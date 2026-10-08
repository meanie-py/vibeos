use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

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


// Enum-variant bare-name consts (kept module-local — see prelude.rs note).
const LsapLookupWksta: LSAP_LOOKUP_LEVEL = advapi32_sys::_LSAP_LOOKUP_LEVEL_LsapLookupWksta;
const ForestTrustDomainInfo: advapi32_sys::LSA_FOREST_TRUST_RECORD_TYPE =
    advapi32_sys::_LSA_FOREST_TRUST_RECORD_TYPE_ForestTrustDomainInfo;
const TrustedDomainAuthInformationInternal: TRUSTED_INFORMATION_CLASS =
    advapi32_sys::_TRUSTED_INFORMATION_CLASS_TrustedDomainAuthInformationInternal;
const TrustedDomainFullInformationInternal: TRUSTED_INFORMATION_CLASS =
    advapi32_sys::_TRUSTED_INFORMATION_CLASS_TrustedDomainFullInformationInternal;

// LsarGetUserName/LsarClose (lsa_c.c RPC stubs, __stdcall), GetComputerNameW/
// GetProcessHeap/HeapAlloc/HeapFree/lstrcmpW (kernel32 WINAPI) and the TRACE!
// macro all come from crate::prelude::* (correct extern "system" decoration).

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, enum, or table declarations are present in this preamble.

unsafe fn LsapIsLocalComputer(ServerName: PLSA_UNICODE_STRING) -> BOOL {

    let mut dwSize: u32 = MAX_COMPUTERNAME_LENGTH + 1;
    let mut Result: i32;
    let buf: LPWSTR;
    let mut pSrvName: PCWSTR;

    if ServerName.is_null()
        || (*ServerName).Length == 0
        || (*ServerName).Buffer.is_null()
    {
        return 1;
    }

    pSrvName = (*ServerName).Buffer;
    buf = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwSize as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    Result = GetComputerNameW(buf, &mut dwSize);
    if Result != 0 && *pSrvName.offset(0) == ('\\' as u16) && *pSrvName.offset(1) == ('\\' as u16) {
        pSrvName = pSrvName.offset(2);
    }
    Result = (Result != 0 && lstrcmpW(pSrvName, buf as PCWSTR) == 0) as i32;
    HeapFree(GetProcessHeap(), 0, buf as *mut core::ffi::c_void);

    Result
}
#[no_mangle]
pub unsafe extern "system" fn PLSAPR_SERVER_NAME_bind(pszSystemName: PLSAPR_SERVER_NAME) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        w!("ncacn_np").as_ptr() as *mut u16,
        pszSystemName,
        w!("\\pipe\\lsarpc").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {}

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {}

    hBinding
}
#[no_mangle]
pub unsafe extern "system" fn PLSAPR_SERVER_NAME_unbind(
    pszSystemName: PLSAPR_SERVER_NAME,
    mut hBinding: handle_t,
) {
    let status: RPC_STATUS;

    TRACE!(b"PLSAPR_SERVER_NAME_unbind() called\n\0".as_ptr());

    status = RpcBindingFree(core::ptr::addr_of_mut!(hBinding));
    if status != 0 {
        TRACE!(b"RpcBindingFree returned 0x%x\n\0".as_ptr(), status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaAddAccountRights(
    PolicyHandle: LSA_HANDLE,
    AccountSid: PSID,
    UserRights: PLSA_UNICODE_STRING,
    CountOfRights: ULONG,
) -> NTSTATUS {
    let mut UserRightSet: LSAPR_USER_RIGHT_SET = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaAddAccountRights(%p %p %p 0x%08x)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        AccountSid,
        UserRights,
        CountOfRights,
    );

    UserRightSet.Entries = CountOfRights;
    UserRightSet.UserRights = UserRights as PRPC_UNICODE_STRING;

    Status = rpc_try(|| LsarAddAccountRights(
        PolicyHandle as LSAPR_HANDLE,
        AccountSid as PRPC_SID,
        core::ptr::addr_of_mut!(UserRightSet),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaAddPrivilegesToAccount(
    AccountHandle: LSA_HANDLE,
    PrivilegeSet: PPRIVILEGE_SET,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaAddPrivilegesToAccount(%p %p)\n\0".as_ptr() as *const i8,
        AccountHandle,
        PrivilegeSet,
    );

    Status = rpc_try(|| LsarAddPrivilegesToAccount(
        AccountHandle as LSAPR_HANDLE,
        PrivilegeSet as PLSAPR_PRIVILEGE_SET,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaClearAuditLog(PolicyHandle: LSA_HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| LsarClearAuditLog(PolicyHandle as LSAPR_HANDLE)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS {
    let mut ObjectHandle = ObjectHandle;
    let Status: NTSTATUS;

    TRACE!(b"LsaClose(%p) called\n\0".as_ptr(), ObjectHandle);

    Status = rpc_try(|| LsarClose(core::ptr::addr_of_mut!(ObjectHandle) as PLSAPR_HANDLE)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaCreateAccount(
    PolicyHandle: LSA_HANDLE,
    AccountSid: PSID,
    DesiredAccess: ACCESS_MASK,
    AccountHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaCreateAccount(%p %p 0x%08lx %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        AccountSid,
        DesiredAccess,
        AccountHandle,
    );

    Status = rpc_try(|| LsarCreateAccount(
        PolicyHandle as LSAPR_HANDLE,
        AccountSid as PRPC_SID,
        DesiredAccess,
        AccountHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaCreateSecret(
    PolicyHandle: LSA_HANDLE,
    SecretName: PLSA_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    SecretHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaCreateSecret(%p %p 0x%08lx %p)\n\0".as_ptr() as PCHAR,
        PolicyHandle,
        SecretName,
        DesiredAccess,
        SecretHandle,
    );

    Status = rpc_try(|| LsarCreateSecret(
        PolicyHandle as LSAPR_HANDLE,
        SecretName as PRPC_UNICODE_STRING,
        DesiredAccess,
        SecretHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaCreateTrustedDomain(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainInformation: PLSA_TRUST_INFORMATION,
    DesiredAccess: ACCESS_MASK,
    TrustedDomainHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaCreateTrustedDomain(%p %p 0x%08lx %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainInformation,
        DesiredAccess,
        TrustedDomainHandle,
    );

    Status = rpc_try(|| LsarCreateTrustedDomain(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainInformation as PLSAPR_TRUST_INFORMATION,
        DesiredAccess,
        TrustedDomainHandle as PLSAPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaCreateTrustedDomainEx(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainInformation: PTRUSTED_DOMAIN_INFORMATION_EX,
    AuthenticationInformation: PTRUSTED_DOMAIN_AUTH_INFORMATION,
    DesiredAccess: ACCESS_MASK,
    TrustedDomainHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let EncryptedAuthInfo: PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaCreateTrustedDomainEx(%p %p %p 0x%08lx %p) stub\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainInformation,
        AuthenticationInformation,
        DesiredAccess,
        TrustedDomainHandle,
    );

    Status = rpc_try(|| LsarCreateTrustedDomainEx2(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainInformation as PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX,
        EncryptedAuthInfo,
        DesiredAccess,
        TrustedDomainHandle as PLSAPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaDelete(ObjectHandle: LSA_HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(b"LsaDelete(%p)\n\0".as_ptr() as *const i8, ObjectHandle);

    Status = rpc_try(|| LsarDelete(ObjectHandle as LSAPR_HANDLE)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaDeleteTrustedDomain(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainSid: PSID,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaDeleteTrustedDomain(%p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainSid,
    );

    Status = rpc_try(|| LsarDeleteTrustedDomain(PolicyHandle as LSAPR_HANDLE, TrustedDomainSid as PRPC_SID)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateAccountRights(
    PolicyHandle: LSA_HANDLE,
    AccountSid: PSID,
    UserRights: *mut PLSA_UNICODE_STRING,
    CountOfRights: PULONG,
) -> NTSTATUS {
    let mut UserRightsSet: LSAPR_USER_RIGHT_SET = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumerateAccountRights(%p %p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        AccountSid,
        UserRights,
        CountOfRights,
    );

    UserRightsSet.Entries = 0;
    UserRightsSet.UserRights = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumerateAccountRights(
        PolicyHandle as LSAPR_HANDLE,
        AccountSid as PRPC_SID,
        &mut UserRightsSet,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *UserRights = UserRightsSet.UserRights as _;
    *CountOfRights = UserRightsSet.Entries;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateAccounts(
    PolicyHandle: LSA_HANDLE,
    EnumerationContext: PLSA_ENUMERATION_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut AccountEnumBuffer: LSAPR_ACCOUNT_ENUM_BUFFER = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumerateAccounts(%p %p %p %lu %p)\n\0".as_ptr() as PCHAR,
        PolicyHandle,
        EnumerationContext,
        Buffer,
        PreferedMaximumLength,
        CountReturned,
    );

    AccountEnumBuffer.EntriesRead = 0;
    AccountEnumBuffer.Information = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumerateAccounts(
        PolicyHandle as LSAPR_HANDLE,
        EnumerationContext,
        core::ptr::addr_of_mut!(AccountEnumBuffer),
        PreferedMaximumLength,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = AccountEnumBuffer.Information as _;
    *CountReturned = AccountEnumBuffer.EntriesRead;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateAccountsWithUserRight(
    PolicyHandle: LSA_HANDLE,
    UserRight: PLSA_UNICODE_STRING,
    Buffer: *mut PVOID,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut AccountEnumBuffer: LSAPR_ACCOUNT_ENUM_BUFFER = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumerateAccountsWithUserRight(%p %p %p %p) stub\n\0".as_ptr() as *const i8,
        PolicyHandle,
        UserRight,
        Buffer,
        CountReturned,
    );

    AccountEnumBuffer.EntriesRead = 0;
    AccountEnumBuffer.Information = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumerateAccountsWithUserRight(
        PolicyHandle as LSAPR_HANDLE,
        UserRight as PRPC_UNICODE_STRING,
        &mut AccountEnumBuffer,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = AccountEnumBuffer.Information as PVOID;
    *CountReturned = AccountEnumBuffer.EntriesRead;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumeratePrivileges(
    PolicyHandle: LSA_HANDLE,
    EnumerationContext: PLSA_ENUMERATION_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut PrivilegeEnumBuffer: LSAPR_PRIVILEGE_ENUM_BUFFER = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumeratePrivileges(%p %p %p %lu %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        EnumerationContext,
        Buffer,
        PreferedMaximumLength,
        CountReturned,
    );

    PrivilegeEnumBuffer.Entries = 0;
    PrivilegeEnumBuffer.Privileges = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumeratePrivileges(
        PolicyHandle as LSAPR_HANDLE,
        EnumerationContext,
        core::ptr::addr_of_mut!(PrivilegeEnumBuffer),
        PreferedMaximumLength,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = PrivilegeEnumBuffer.Privileges as PVOID;
    *CountReturned = PrivilegeEnumBuffer.Entries;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumeratePrivilegesOfAccount(
    AccountHandle: LSA_HANDLE,
    Privileges: *mut PPRIVILEGE_SET,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumeratePrivilegesOfAccount(%p %p)\n\0".as_ptr() as *const i8,
        AccountHandle,
        Privileges,
    );

    Status = rpc_try(|| LsarEnumeratePrivilegesAccount(
        AccountHandle as LSAPR_HANDLE,
        Privileges as *mut *mut LSAPR_PRIVILEGE_SET,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateTrustedDomains(
    PolicyHandle: LSA_HANDLE,
    EnumerationContext: PLSA_ENUMERATION_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut TrustedEnumBuffer: LSAPR_TRUSTED_ENUM_BUFFER = core::mem::zeroed();
    let Status: NTSTATUS;

    if Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    TrustedEnumBuffer.EntriesRead = 0;
    TrustedEnumBuffer.Information = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumerateTrustedDomains(
        PolicyHandle as LSAPR_HANDLE,
        EnumerationContext,
        &mut TrustedEnumBuffer,
        PreferedMaximumLength,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = TrustedEnumBuffer.Information as PVOID;
    *CountReturned = TrustedEnumBuffer.EntriesRead;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateTrustedDomainsEx(
    PolicyHandle: LSA_HANDLE,
    EnumerationContext: PLSA_ENUMERATION_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut TrustedEnumBuffer: LSAPR_TRUSTED_ENUM_BUFFER_EX = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaEnumerateTrustedDomainsEx(%p %p %p %lu %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        EnumerationContext,
        Buffer,
        PreferedMaximumLength,
        CountReturned,
    );

    if Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    TrustedEnumBuffer.EntriesRead = 0;
    TrustedEnumBuffer.EnumerationBuffer = core::ptr::null_mut();

    Status = rpc_try(|| LsarEnumerateTrustedDomainsEx(
        PolicyHandle as LSAPR_HANDLE,
        EnumerationContext,
        &mut TrustedEnumBuffer,
        PreferedMaximumLength,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = TrustedEnumBuffer.EnumerationBuffer as PVOID;
    *CountReturned = TrustedEnumBuffer.EntriesRead;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS {
    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer) as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn LsaGetQuotasForAccount(
    AccountHandle: LSA_HANDLE,
    QuotaLimits: PQUOTA_LIMITS,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaGetQuotasForAccount(%p %p)\n\0".as_ptr() as *const i8,
        AccountHandle,
        QuotaLimits,
    );

    Status = rpc_try(|| LsarGetQuotasForAccount(AccountHandle as LSAPR_HANDLE, QuotaLimits as *mut advapi32_sys::QUOTA_LIMITS)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaGetRemoteUserName(
    SystemName: PUNICODE_STRING,
    UserName: *mut PUNICODE_STRING,
    DomainName: *mut PUNICODE_STRING,
) -> NTSTATUS {
    let mut UserNameString: PUNICODE_STRING = core::ptr::null_mut();
    let mut DomainNameString: PUNICODE_STRING = core::ptr::null_mut();
    let Status: NTSTATUS;

    Status = rpc_try(|| LsarGetUserName(
        SystemName as PLSAPR_SERVER_NAME,
        core::ptr::addr_of_mut!(UserNameString) as *mut PRPC_UNICODE_STRING,
        if !DomainName.is_null() {
            core::ptr::addr_of_mut!(DomainNameString) as *mut PRPC_UNICODE_STRING
        } else {
            core::ptr::null_mut()
        },
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *UserName = UserNameString;

    if !DomainName.is_null() {
        *DomainName = DomainNameString;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaGetSystemAccessAccount(
    AccountHandle: LSA_HANDLE,
    SystemAccess: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaGetSystemAccessAccount(%p %p)\n\0".as_ptr() as *const i8,
        AccountHandle,
        SystemAccess,
    );

    Status = rpc_try(|| LsarGetSystemAccessAccount(
        AccountHandle as LSAPR_HANDLE,
        SystemAccess as *mut ACCESS_MASK,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaGetUserName(
    UserName: *mut PUNICODE_STRING,
    DomainName: *mut PUNICODE_STRING,
) -> NTSTATUS {
    let mut UserNameString: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let mut DomainNameString: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaGetUserName(%p %p)\n\0".as_ptr() as *const i8,
        UserName,
        DomainName,
    );

    Status = rpc_try(|| LsarGetUserName(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(UserNameString),
        if !DomainName.is_null() {
            core::ptr::addr_of_mut!(DomainNameString)
        } else {
            core::ptr::null_mut()
        },
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *UserName = UserNameString as PUNICODE_STRING;

    if !DomainName.is_null() {
        *DomainName = DomainNameString as PUNICODE_STRING;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupNames(
    PolicyHandle: LSA_HANDLE,
    Count: ULONG,
    Names: PLSA_UNICODE_STRING,
    ReferencedDomains: *mut PLSA_REFERENCED_DOMAIN_LIST,
    Sids: *mut PLSA_TRANSLATED_SID,
) -> NTSTATUS {
    let mut TranslatedSids: LSAPR_TRANSLATED_SIDS = LSAPR_TRANSLATED_SIDS {
        Entries: 0,
        Sids: core::ptr::null_mut(),
    };
    let mut MappedCount: ULONG = 0;
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupNames(%p %lu %p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Count,
        Names,
        ReferencedDomains,
        Sids,
    );

    if ReferencedDomains.is_null() || Sids.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ReferencedDomains = core::ptr::null_mut();
    *Sids = core::ptr::null_mut();

    TranslatedSids.Entries = Count;

    Status = rpc_try(|| LsarLookupNames(
        PolicyHandle as LSAPR_HANDLE,
        Count,
        Names as PRPC_UNICODE_STRING,
        ReferencedDomains as *mut PLSAPR_REFERENCED_DOMAIN_LIST,
        &mut TranslatedSids,
        LsapLookupWksta,
        &mut MappedCount,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Sids = TranslatedSids.Sids as PLSA_TRANSLATED_SID;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupNames2(
    PolicyHandle: LSA_HANDLE,
    Flags: ULONG,
    Count: ULONG,
    Names: PLSA_UNICODE_STRING,
    ReferencedDomains: *mut PLSA_REFERENCED_DOMAIN_LIST,
    Sids: *mut PLSA_TRANSLATED_SID2,
) -> NTSTATUS {
    let mut TranslatedSids: LSAPR_TRANSLATED_SIDS_EX2 = LSAPR_TRANSLATED_SIDS_EX2 {
        Entries: 0,
        Sids: core::ptr::null_mut(),
    };
    let mut MappedCount: ULONG = 0;
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupNames2(%p 0x%08x %lu %p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Flags,
        Count,
        Names,
        ReferencedDomains,
        Sids,
    );

    if ReferencedDomains.is_null() || Sids.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ReferencedDomains = core::ptr::null_mut();
    *Sids = core::ptr::null_mut();

    TranslatedSids.Entries = Count;

    Status = rpc_try(|| LsarLookupNames3(
        PolicyHandle as LSAPR_HANDLE,
        Count,
        Names as PRPC_UNICODE_STRING,
        ReferencedDomains as *mut PLSAPR_REFERENCED_DOMAIN_LIST,
        &mut TranslatedSids,
        LsapLookupWksta,
        &mut MappedCount,
        Flags,
        2,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Sids = TranslatedSids.Sids as PLSA_TRANSLATED_SID2;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupPrivilegeDisplayName(
    PolicyHandle: LSA_HANDLE,
    Name: PLSA_UNICODE_STRING,
    DisplayName: *mut PLSA_UNICODE_STRING,
    LanguageReturned: PUSHORT,
) -> NTSTATUS {
    let mut DisplayNameBuffer: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupPrivilegeDisplayName(%p %p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Name,
        DisplayName,
        LanguageReturned,
    );

    Status = rpc_try(|| LsarLookupPrivilegeDisplayName(
        PolicyHandle,
        Name as PRPC_UNICODE_STRING,
        GetUserDefaultUILanguage(),
        GetSystemDefaultUILanguage(),
        &mut DisplayNameBuffer,
        LanguageReturned,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *DisplayName = DisplayNameBuffer as PLSA_UNICODE_STRING;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupPrivilegeName(
    PolicyHandle: LSA_HANDLE,
    Value: PLUID,
    Name: *mut PUNICODE_STRING,
) -> NTSTATUS {
    let mut NameBuffer: PRPC_UNICODE_STRING = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupPrivilegeName(%p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Value,
        Name,
    );

    Status = rpc_try(|| LsarLookupPrivilegeName(PolicyHandle, Value as *mut advapi32_sys::LUID, &mut NameBuffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Name = NameBuffer as PUNICODE_STRING;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupPrivilegeValue(
    PolicyHandle: LSA_HANDLE,
    Name: PLSA_UNICODE_STRING,
    Value: PLUID,
) -> NTSTATUS {
    let mut Luid: LUID = core::mem::zeroed();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupPrivilegeValue(%p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Name,
        Value,
    );

    Status = rpc_try(|| LsarLookupPrivilegeValue(
        PolicyHandle,
        Name as PRPC_UNICODE_STRING,
        core::ptr::addr_of_mut!(Luid) as *mut advapi32_sys::LUID,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if Status == STATUS_SUCCESS {
        *Value = Luid;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaLookupSids(
    PolicyHandle: LSA_HANDLE,
    Count: ULONG,
    Sids: *mut PSID,
    ReferencedDomains: *mut PLSA_REFERENCED_DOMAIN_LIST,
    Names: *mut PLSA_TRANSLATED_NAME,
) -> NTSTATUS {
    let mut SidEnumBuffer: LSAPR_SID_ENUM_BUFFER = core::mem::zeroed();
    let mut TranslatedNames: LSAPR_TRANSLATED_NAMES = core::mem::zeroed();
    let mut MappedCount: ULONG = 0;
    let Status: NTSTATUS;

    TRACE!(
        b"LsaLookupSids(%p %lu %p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        Count,
        Sids,
        ReferencedDomains,
        Names,
    );

    if Count == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    SidEnumBuffer.Entries = Count;
    SidEnumBuffer.SidInfo = Sids as PLSAPR_SID_INFORMATION;

    *ReferencedDomains = core::ptr::null_mut();
    *Names = core::ptr::null_mut();

    TranslatedNames.Entries = 0;
    TranslatedNames.Names = core::ptr::null_mut();

    Status = rpc_try(|| LsarLookupSids(
        PolicyHandle as LSAPR_HANDLE,
        core::ptr::addr_of_mut!(SidEnumBuffer),
        ReferencedDomains as *mut PLSAPR_REFERENCED_DOMAIN_LIST,
        core::ptr::addr_of_mut!(TranslatedNames),
        LsapLookupWksta,
        core::ptr::addr_of_mut!(MappedCount),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Names = TranslatedNames.Names as PLSA_TRANSLATED_NAME;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaNtStatusToWinError(Status: NTSTATUS) -> ULONG {
    RtlNtStatusToDosError(Status)
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenAccount(
    PolicyHandle: LSA_HANDLE,
    AccountSid: PSID,
    DesiredAccess: ACCESS_MASK,
    AccountHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaOpenAccount(%p %p 0x%08lx %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        AccountSid,
        DesiredAccess,
        AccountHandle,
    );

    Status = rpc_try(|| LsarOpenAccount(
        PolicyHandle as LSAPR_HANDLE,
        AccountSid as PRPC_SID,
        DesiredAccess,
        AccountHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenPolicy(
    SystemName: PLSA_UNICODE_STRING,
    ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    PolicyHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if LsapIsLocalComputer(SystemName) == 0 {
        return RPC_NT_SERVER_UNAVAILABLE;
    }

    *PolicyHandle = core::ptr::null_mut();

    Status = rpc_try(|| LsarOpenPolicy(
        if !SystemName.is_null() {
            (*SystemName).Buffer
        } else {
            core::ptr::null_mut()
        },
        ObjectAttributes as PLSAPR_OBJECT_ATTRIBUTES,
        DesiredAccess,
        PolicyHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenPolicySce(
    SystemName: PLSA_UNICODE_STRING,
    ObjectAttributes: PLSA_OBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    PolicyHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if LsapIsLocalComputer(SystemName) == 0 {
        return RPC_NT_SERVER_UNAVAILABLE;
    }

    *PolicyHandle = core::ptr::null_mut();

    Status = rpc_try(|| LsarOpenPolicySce(
        if !SystemName.is_null() {
            (*SystemName).Buffer
        } else {
            core::ptr::null_mut()
        },
        ObjectAttributes as PLSAPR_OBJECT_ATTRIBUTES,
        DesiredAccess,
        PolicyHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenSecret(
    PolicyHandle: LSA_HANDLE,
    SecretName: PLSA_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    SecretHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    *SecretHandle = core::ptr::null_mut();

    Status = rpc_try(|| LsarOpenSecret(
        PolicyHandle as LSAPR_HANDLE,
        SecretName as PRPC_UNICODE_STRING,
        DesiredAccess,
        SecretHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenTrustedDomain(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainSid: PSID,
    DesiredAccess: ACCESS_MASK,
    TrustedDomainHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaOpenTrustedDomain(%p %p 0x%08lx %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainSid,
        DesiredAccess,
        TrustedDomainHandle,
    );

    Status = rpc_try(|| LsarOpenTrustedDomain(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainSid as PRPC_SID,
        DesiredAccess,
        TrustedDomainHandle as PLSAPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaOpenTrustedDomainByName(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainName: PLSA_UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    TrustedDomainHandle: PLSA_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaOpenTrustedDomainByName(%p %p 0x%08lx %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainName,
        DesiredAccess,
        TrustedDomainHandle,
    );

    Status = rpc_try(|| LsarOpenTrustedDomainByName(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainName as PRPC_UNICODE_STRING,
        DesiredAccess,
        TrustedDomainHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryDomainInformationPolicy(
    PolicyHandle: LSA_HANDLE,
    InformationClass: POLICY_DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut PolicyInformation: PLSAPR_POLICY_DOMAIN_INFORMATION = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaQueryDomainInformationPolicy(%p %lu %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        InformationClass,
        Buffer,
    );

    Status = rpc_try(|| LsarQueryDomainInformationPolicy(
        PolicyHandle as LSAPR_HANDLE,
        InformationClass,
        &mut PolicyInformation,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    *Buffer = PolicyInformation as PVOID;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryForestTrustInformation(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainName: PLSA_UNICODE_STRING,
    ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaQueryForestTrustInformation(%p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainName,
        ForestTrustInfo,
    );

    Status = rpc_try(|| LsarQueryForestTrustInformation(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainName,
        ForestTrustDomainInfo,
        ForestTrustInfo,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryInfoTrustedDomain(
    TrustedDomainHandle: LSA_HANDLE,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut TrustedDomainInformation: PLSAPR_TRUSTED_DOMAIN_INFO = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"LsaQueryInfoTrustedDomain(%p %d %p) stub\n\0".as_ptr() as *const i8,
        TrustedDomainHandle,
        InformationClass,
        Buffer,
    );

    if InformationClass == TrustedDomainAuthInformationInternal
        || InformationClass == TrustedDomainFullInformationInternal
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    Status = rpc_try(|| LsarQueryInfoTrustedDomain(
        TrustedDomainHandle as LSAPR_HANDLE,
        InformationClass,
        &mut TrustedDomainInformation,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    *Buffer = TrustedDomainInformation as PVOID;

    TRACE!(b"Done (Status: 0x%08x)\n\0".as_ptr() as *const i8, Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryInformationPolicy(
    PolicyHandle: LSA_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut PolicyInformation: PLSAPR_POLICY_INFORMATION = core::ptr::null_mut();
    let Status: NTSTATUS;

    Status = rpc_try(|| LsarQueryInformationPolicy(
        PolicyHandle as LSAPR_HANDLE,
        InformationClass,
        &mut PolicyInformation,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    *Buffer = PolicyInformation as PVOID;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQuerySecret(
    SecretHandle: LSA_HANDLE,
    CurrentValue: *mut PLSA_UNICODE_STRING,
    CurrentValueSetTime: PLARGE_INTEGER,
    OldValue: *mut PLSA_UNICODE_STRING,
    OldValueSetTime: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut EncryptedCurrentValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut EncryptedOldValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut DecryptedCurrentValue: PLSA_UNICODE_STRING = core::ptr::null_mut();
    let mut DecryptedOldValue: PLSA_UNICODE_STRING = core::ptr::null_mut();
    let mut BufferSize: SIZE_T;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsaQuerySecret(%p %p %p %p %p)\n\0".as_ptr() as *const i8,
        SecretHandle,
        CurrentValue,
        CurrentValueSetTime,
        OldValue,
        OldValueSetTime,
    );

    Status = rpc_try(|| LsarQuerySecret(
        SecretHandle as LSAPR_HANDLE,
        core::ptr::addr_of_mut!(EncryptedCurrentValue),
        CurrentValueSetTime as *mut advapi32_sys::LARGE_INTEGER,
        core::ptr::addr_of_mut!(EncryptedOldValue),
        OldValueSetTime as *mut advapi32_sys::LARGE_INTEGER,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if !CurrentValue.is_null() {
            if EncryptedCurrentValue.is_null() {
                *CurrentValue = core::ptr::null_mut();
            } else {
                BufferSize = (core::mem::size_of::<LSA_UNICODE_STRING>()
                    + (*EncryptedCurrentValue).MaximumLength as usize)
                    as SIZE_T;
                DecryptedCurrentValue = midl_user_allocate(BufferSize) as PLSA_UNICODE_STRING;
                if DecryptedCurrentValue.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done;
                }

                (*DecryptedCurrentValue).Length = (*EncryptedCurrentValue).Length as USHORT;
                (*DecryptedCurrentValue).MaximumLength =
                    (*EncryptedCurrentValue).MaximumLength as USHORT;
                (*DecryptedCurrentValue).Buffer = DecryptedCurrentValue.add(1) as PWSTR;
                RtlCopyMemory(
                    (*DecryptedCurrentValue).Buffer as *mut core::ffi::c_void,
                    (*EncryptedCurrentValue).Buffer as *const core::ffi::c_void,
                    (*EncryptedCurrentValue).Length as usize,
                );

                *CurrentValue = DecryptedCurrentValue;
            }
        }

        if !OldValue.is_null() {
            if EncryptedOldValue.is_null() {
                *OldValue = core::ptr::null_mut();
            } else {
                BufferSize = (core::mem::size_of::<LSA_UNICODE_STRING>()
                    + (*EncryptedOldValue).MaximumLength as usize)
                    as SIZE_T;
                DecryptedOldValue = midl_user_allocate(BufferSize) as PLSA_UNICODE_STRING;
                if DecryptedOldValue.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done;
                }

                (*DecryptedOldValue).Length = (*EncryptedOldValue).Length as USHORT;
                (*DecryptedOldValue).MaximumLength =
                    (*EncryptedOldValue).MaximumLength as USHORT;
                (*DecryptedOldValue).Buffer = DecryptedOldValue.add(1) as PWSTR;
                RtlCopyMemory(
                    (*DecryptedOldValue).Buffer as *mut core::ffi::c_void,
                    (*EncryptedOldValue).Buffer as *const core::ffi::c_void,
                    (*EncryptedOldValue).Length as usize,
                );

                *OldValue = DecryptedOldValue;
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !DecryptedCurrentValue.is_null() {
            midl_user_free(DecryptedCurrentValue as *mut core::ffi::c_void);
        }

        if !DecryptedOldValue.is_null() {
            midl_user_free(DecryptedOldValue as *mut core::ffi::c_void);
        }

        if !CurrentValue.is_null() {
            *CurrentValue = core::ptr::null_mut();
        }

        if !OldValue.is_null() {
            *OldValue = core::ptr::null_mut();
        }
    }

    if !EncryptedCurrentValue.is_null() {
        midl_user_free(EncryptedCurrentValue as *mut core::ffi::c_void);
    }

    if !EncryptedOldValue.is_null() {
        midl_user_free(EncryptedOldValue as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQuerySecurityObject(
    ObjectHandle: LSA_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut SdBuffer: LSAPR_SR_SECURITY_DESCRIPTOR = core::mem::zeroed();
    let SdPointer: PLSAPR_SR_SECURITY_DESCRIPTOR;
    let Status: NTSTATUS;

    SdBuffer.Length = 0;
    SdBuffer.SecurityDescriptor = core::ptr::null_mut();

    SdPointer = core::ptr::addr_of_mut!(SdBuffer);

    let mut SdPointer = SdPointer;

    Status = rpc_try(|| LsarQuerySecurityObject(
        ObjectHandle as LSAPR_HANDLE,
        SecurityInformation,
        core::ptr::addr_of_mut!(SdPointer),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        *SecurityDescriptor = SdBuffer.SecurityDescriptor as PSECURITY_DESCRIPTOR;
    } else {
        *SecurityDescriptor = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryTrustedDomainInfo(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainSid: PSID,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaQueryTrustedDomainInfo(%p %p %d %p) stub\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainSid,
        InformationClass,
        Buffer,
    );

    if InformationClass == TrustedDomainAuthInformationInternal
        || InformationClass == TrustedDomainFullInformationInternal
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    Status = rpc_try(|| LsarQueryTrustedDomainInfo(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainSid as PRPC_SID,
        InformationClass,
        Buffer as *mut PLSAPR_TRUSTED_DOMAIN_INFO,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaQueryTrustedDomainInfoByName(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainName: PLSA_UNICODE_STRING,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaQueryTrustedDomainInfoByName(%p %p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainName,
        InformationClass,
        Buffer,
    );

    if InformationClass == TrustedDomainAuthInformationInternal
        || InformationClass == TrustedDomainFullInformationInternal
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    Status = rpc_try(|| LsarQueryTrustedDomainInfoByName(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainName as PRPC_UNICODE_STRING,
        InformationClass,
        Buffer as *mut PLSAPR_TRUSTED_DOMAIN_INFO,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaRemoveAccountRights(
    PolicyHandle: LSA_HANDLE,
    AccountSid: PSID,
    AllRights: BOOLEAN,
    UserRights: PLSA_UNICODE_STRING,
    CountOfRights: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut UserRightSet: LSAPR_USER_RIGHT_SET = core::mem::zeroed();

    TRACE!(
        b"LsaRemoveAccountRights(%p %p %d %p %lu)\n\0".as_ptr() as PCHAR,
        PolicyHandle,
        AccountSid,
        AllRights as core::ffi::c_int,
        UserRights,
        CountOfRights,
    );

    UserRightSet.Entries = CountOfRights;
    UserRightSet.UserRights = UserRights as PRPC_UNICODE_STRING;

    Status = rpc_try(|| LsarRemoveAccountRights(
        PolicyHandle as LSAPR_HANDLE,
        AccountSid as PRPC_SID,
        AllRights,
        core::ptr::addr_of_mut!(UserRightSet),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaRemovePrivilegesFromAccount(
    AccountHandle: LSA_HANDLE,
    AllPrivileges: BOOLEAN,
    Privileges: PPRIVILEGE_SET,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| LsarRemovePrivilegesFromAccount(
        AccountHandle as LSAPR_HANDLE,
        AllPrivileges,
        Privileges as PLSAPR_PRIVILEGE_SET,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaRetrievePrivateData(
    PolicyHandle: LSA_HANDLE,
    KeyName: PLSA_UNICODE_STRING,
    PrivateData: *mut PLSA_UNICODE_STRING,
) -> NTSTATUS {
    let mut EncryptedData: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut DecryptedData: PLSA_UNICODE_STRING = core::ptr::null_mut();
    let BufferSize: SIZE_T;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsaRetrievePrivateData(%p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        KeyName,
        PrivateData,
    );

    Status = rpc_try(|| LsarRetrievePrivateData(
        PolicyHandle as LSAPR_HANDLE,
        KeyName as PRPC_UNICODE_STRING,
        &mut EncryptedData,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if EncryptedData.is_null() {
            *PrivateData = core::ptr::null_mut();
        } else {
            BufferSize = (core::mem::size_of::<LSA_UNICODE_STRING>()
                + (*EncryptedData).MaximumLength as usize) as SIZE_T;
            DecryptedData = midl_user_allocate(BufferSize) as PLSA_UNICODE_STRING;
            if DecryptedData.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*DecryptedData).Length = (*EncryptedData).Length as USHORT;
            (*DecryptedData).MaximumLength = (*EncryptedData).MaximumLength as USHORT;
            (*DecryptedData).Buffer = DecryptedData.add(1) as PWSTR;
            RtlCopyMemory(
                (*DecryptedData).Buffer as *mut core::ffi::c_void,
                (*EncryptedData).Buffer as *const core::ffi::c_void,
                (*EncryptedData).Length as usize,
            );

            *PrivateData = DecryptedData;
        }
    }

    if !NT_SUCCESS(Status) {
        if !DecryptedData.is_null() {
            midl_user_free(DecryptedData as *mut core::ffi::c_void);
        }

        *PrivateData = core::ptr::null_mut();
    }

    if !EncryptedData.is_null() {
        midl_user_free(EncryptedData as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetDomainInformationPolicy(
    PolicyHandle: LSA_HANDLE,
    InformationClass: POLICY_DOMAIN_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaSetDomainInformationPolicy(%p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        InformationClass,
        Buffer,
    );

    Status = rpc_try(|| LsarSetDomainInformationPolicy(
        PolicyHandle as LSAPR_HANDLE,
        InformationClass,
        Buffer as PLSAPR_POLICY_DOMAIN_INFORMATION,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetForestTrustInformation(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainName: PLSA_UNICODE_STRING,
    ForestTrustInfo: PLSA_FOREST_TRUST_INFORMATION,
    CheckOnly: BOOLEAN,
    CollisionInfo: *mut PLSA_FOREST_TRUST_COLLISION_INFORMATION,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaSetForestTrustInformation(%p %p %p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        TrustedDomainName,
        ForestTrustInfo,
        CheckOnly as i32,
        CollisionInfo,
    );

    Status = rpc_try(|| LsarSetForestTrustInformation(
        PolicyHandle as LSAPR_HANDLE,
        TrustedDomainName,
        ForestTrustDomainInfo,
        ForestTrustInfo,
        CheckOnly,
        CollisionInfo,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetInformationPolicy(
    PolicyHandle: LSA_HANDLE,
    InformationClass: POLICY_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaSetInformationPolicy(%p %d %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        InformationClass,
        Buffer,
    );

    Status = rpc_try(|| LsarSetInformationPolicy(
        PolicyHandle as LSAPR_HANDLE,
        InformationClass,
        Buffer as PLSAPR_POLICY_INFORMATION,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetInformationTrustedDomain(
    TrustedDomainHandle: LSA_HANDLE,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetQuotasForAccount(
    AccountHandle: LSA_HANDLE,
    QuotaLimits: PQUOTA_LIMITS,
) -> NTSTATUS {
    let Status: NTSTATUS;

    TRACE!(
        b"LsaSetQuotasForAccount(%p %p)\n\0".as_ptr() as *const i8,
        AccountHandle,
        QuotaLimits,
    );

    Status = rpc_try(|| LsarSetQuotasForAccount(AccountHandle as LSAPR_HANDLE, QuotaLimits as *mut advapi32_sys::QUOTA_LIMITS)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetSecret(
    SecretHandle: LSA_HANDLE,
    CurrentValue: PLSA_UNICODE_STRING,
    OldValue: PLSA_UNICODE_STRING,
) -> NTSTATUS {
    let mut EncryptedCurrentValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut EncryptedOldValue: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let mut BufferSize: SIZE_T;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsaSetSecret(%p %p %p)\n\0".as_ptr() as *const i8,
        SecretHandle,
        EncryptedCurrentValue,
        EncryptedOldValue,
    );

    'done: {
        if !CurrentValue.is_null() {
            BufferSize = (core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>() as SIZE_T)
                + (*CurrentValue).MaximumLength as SIZE_T;
            EncryptedCurrentValue = midl_user_allocate(BufferSize) as PLSAPR_CR_CIPHER_VALUE;
            if EncryptedCurrentValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*EncryptedCurrentValue).Length = (*CurrentValue).Length as u32;
            (*EncryptedCurrentValue).MaximumLength = (*CurrentValue).MaximumLength as u32;
            (*EncryptedCurrentValue).Buffer = EncryptedCurrentValue.add(1) as *mut u8;
            if !(*EncryptedCurrentValue).Buffer.is_null() {
                memcpy(
                    (*EncryptedCurrentValue).Buffer as *mut core::ffi::c_void,
                    (*CurrentValue).Buffer as *const core::ffi::c_void,
                    (*CurrentValue).Length as usize,
                );
            }
        }

        if !OldValue.is_null() {
            BufferSize = (core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>() as SIZE_T)
                + (*OldValue).MaximumLength as SIZE_T;
            EncryptedOldValue = midl_user_allocate(BufferSize) as PLSAPR_CR_CIPHER_VALUE;
            if EncryptedOldValue.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*EncryptedOldValue).Length = (*OldValue).Length as u32;
            (*EncryptedOldValue).MaximumLength = (*OldValue).MaximumLength as u32;
            (*EncryptedOldValue).Buffer = EncryptedOldValue.add(1) as *mut u8;
            if !(*EncryptedOldValue).Buffer.is_null() {
                memcpy(
                    (*EncryptedOldValue).Buffer as *mut core::ffi::c_void,
                    (*OldValue).Buffer as *const core::ffi::c_void,
                    (*OldValue).Length as usize,
                );
            }
        }

        Status = rpc_try(|| LsarSetSecret(
            SecretHandle as LSAPR_HANDLE,
            EncryptedCurrentValue,
            EncryptedOldValue,
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    }

    if !EncryptedCurrentValue.is_null() {
        midl_user_free(EncryptedCurrentValue as *mut core::ffi::c_void);
    }

    if !EncryptedOldValue.is_null() {
        midl_user_free(EncryptedOldValue as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetSecurityObject(
    ObjectHandle: LSA_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut SdBuffer: LSAPR_SR_SECURITY_DESCRIPTOR = LSAPR_SR_SECURITY_DESCRIPTOR {
        Length: 0,
        SecurityDescriptor: core::ptr::null_mut(),
    };
    let mut SdLength: ULONG = 0;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsaSetSecurityObject(%p %lx %p)\n\0".as_ptr() as *const i8,
        ObjectHandle,
        SecurityInformation,
        SecurityDescriptor,
    );

    Status = RtlMakeSelfRelativeSD(
        SecurityDescriptor,
        core::ptr::null_mut(),
        &mut SdLength,
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        return STATUS_INVALID_PARAMETER;
    }

    SdBuffer.SecurityDescriptor = MIDL_user_allocate(SdLength as SIZE_T) as *mut _;
    if SdBuffer.SecurityDescriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = RtlMakeSelfRelativeSD(
        SecurityDescriptor,
        SdBuffer.SecurityDescriptor as PSECURITY_DESCRIPTOR,
        &mut SdLength,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SdBuffer.Length = SdLength;

        Status = rpc_try(|| LsarSetSecurityObject(
            ObjectHandle as LSAPR_HANDLE,
            SecurityInformation,
            &mut SdBuffer,
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    }

    if !SdBuffer.SecurityDescriptor.is_null() {
        MIDL_user_free(SdBuffer.SecurityDescriptor as *mut _);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetSystemAccessAccount(
    AccountHandle: LSA_HANDLE,
    SystemAccess: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| LsarSetSystemAccessAccount(AccountHandle as LSAPR_HANDLE, SystemAccess)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetTrustedDomainInfoByName(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainName: PLSA_UNICODE_STRING,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsaSetTrustedDomainInformation(
    PolicyHandle: LSA_HANDLE,
    TrustedDomainSid: PSID,
    InformationClass: TRUSTED_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsaStorePrivateData(
    PolicyHandle: LSA_HANDLE,
    KeyName: PLSA_UNICODE_STRING,
    PrivateData: PLSA_UNICODE_STRING,
) -> NTSTATUS {
    let mut EncryptedData: PLSAPR_CR_CIPHER_VALUE = core::ptr::null_mut();
    let BufferSize: SIZE_T;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsaStorePrivateData(%p %p %p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
        KeyName,
        PrivateData,
    );

    'done: loop {
        if !PrivateData.is_null() {
            BufferSize = (core::mem::size_of::<LSAPR_CR_CIPHER_VALUE>()
                + (*PrivateData).MaximumLength as usize) as SIZE_T;
            EncryptedData = midl_user_allocate(BufferSize) as PLSAPR_CR_CIPHER_VALUE;
            if EncryptedData.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            (*EncryptedData).Length = (*PrivateData).Length as u32;
            (*EncryptedData).MaximumLength = (*PrivateData).MaximumLength as u32;
            (*EncryptedData).Buffer = EncryptedData.add(1) as *mut u8;
            if !(*EncryptedData).Buffer.is_null() {
                RtlCopyMemory(
                    (*EncryptedData).Buffer as *mut core::ffi::c_void,
                    (*PrivateData).Buffer as *const core::ffi::c_void,
                    (*PrivateData).Length as usize,
                );
            }
        }

        Status = rpc_try(|| LsarStorePrivateData(
            PolicyHandle as LSAPR_HANDLE,
            KeyName as PRPC_UNICODE_STRING,
            EncryptedData,
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

        break 'done;
    }

    if !EncryptedData.is_null() {
        midl_user_free(EncryptedData as *mut core::ffi::c_void);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/advapi32/sec/lsa.c
//  * PURPOSE:         Local security authority functions
//  * PROGRAMMER:      Emanuele Aliberti
//  * UPDATE HISTORY:
//  *      19990322 EA created
//  *      19990515 EA stubs
//  *      20030202 KJK compressed stubs
//  */
//
// #include <advapi32.h>
//
// #include <lsa_c.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// static
// BOOL
// LsapIsLocalComputer(PLSA_UNICODE_STRING ServerName)
// {
//     DWORD dwSize = MAX_COMPUTERNAME_LENGTH + 1;
//     BOOL Result;
//     LPWSTR buf;
//     PCWSTR pSrvName;
//
//     if (ServerName == NULL || ServerName->Length == 0 || ServerName->Buffer == NULL)
//         return TRUE;
//
//     pSrvName = ServerName->Buffer;
//     buf = HeapAlloc(GetProcessHeap(), 0, dwSize * sizeof(WCHAR));
//     Result = GetComputerNameW(buf, &dwSize);
//     if (Result && (pSrvName[0] == L'\\') && (pSrvName[1] == L'\\'))
//         pSrvName += 2;
//     Result = Result && !lstrcmpW(pSrvName, buf);
//     HeapFree(GetProcessHeap(), 0, buf);
//
//     return Result;
// }
//
//
// handle_t
// __RPC_USER
// PLSAPR_SERVER_NAME_bind(PLSAPR_SERVER_NAME pszSystemName)
// {
//     handle_t hBinding = NULL;
//     LPWSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("PLSAPR_SERVER_NAME_bind() called\n");
//
//     status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       pszSystemName,
//                                       L"\\pipe\\lsarpc",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status)
//     {
//         TRACE("RpcStringBindingCompose returned 0x%x\n", status);
//         return NULL;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hBinding);
//     if (status)
//     {
//         TRACE("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeW(&pszStringBinding);
//     if (status)
//     {
//         TRACE("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return hBinding;
// }
//
//
// void
// __RPC_USER
// PLSAPR_SERVER_NAME_unbind(PLSAPR_SERVER_NAME pszSystemName,
//                           handle_t hBinding)
// {
//     RPC_STATUS status;
//
//     TRACE("PLSAPR_SERVER_NAME_unbind() called\n");
//
//     status = RpcBindingFree(&hBinding);
//     if (status)
//     {
//         TRACE("RpcBindingFree returned 0x%x\n", status);
//     }
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaAddAccountRights(IN LSA_HANDLE PolicyHandle,
//                     IN PSID AccountSid,
//                     IN PLSA_UNICODE_STRING UserRights,
//                     IN ULONG CountOfRights)
// {
//     LSAPR_USER_RIGHT_SET UserRightSet;
//     NTSTATUS Status;
//
//     TRACE("LsaAddAccountRights(%p %p %p 0x%08x)\n",
//           PolicyHandle, AccountSid, UserRights, CountOfRights);
//
//     UserRightSet.Entries = CountOfRights;
//     UserRightSet.UserRights = (PRPC_UNICODE_STRING)UserRights;
//
//     RpcTryExcept
//     {
//         Status = LsarAddAccountRights((LSAPR_HANDLE)PolicyHandle,
//                                       (PRPC_SID)AccountSid,
//                                       &UserRightSet);
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaAddPrivilegesToAccount(IN LSA_HANDLE AccountHandle,
//                           IN PPRIVILEGE_SET PrivilegeSet)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaAddPrivilegesToAccount(%p %p)\n",
//           AccountHandle, PrivilegeSet);
//
//     RpcTryExcept
//     {
//         Status = LsarAddPrivilegesToAccount((LSAPR_HANDLE)AccountHandle,
//                                             (PLSAPR_PRIVILEGE_SET)PrivilegeSet);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaClearAuditLog(IN LSA_HANDLE PolicyHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaClearAuditLog(%p)\n", PolicyHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarClearAuditLog((LSAPR_HANDLE)PolicyHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaClose(IN LSA_HANDLE ObjectHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaClose(%p) called\n", ObjectHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarClose((PLSAPR_HANDLE)&ObjectHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaCreateAccount(IN LSA_HANDLE PolicyHandle,
//                  IN PSID AccountSid,
//                  IN ACCESS_MASK DesiredAccess,
//                  OUT PLSA_HANDLE AccountHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaCreateAccount(%p %p 0x%08lx %p)\n",
//           PolicyHandle, AccountSid, DesiredAccess, AccountHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarCreateAccount((LSAPR_HANDLE)PolicyHandle,
//                                    AccountSid,
//                                    DesiredAccess,
//                                    AccountHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaCreateSecret(IN LSA_HANDLE PolicyHandle,
//                 IN PLSA_UNICODE_STRING SecretName,
//                 IN ACCESS_MASK DesiredAccess,
//                 OUT PLSA_HANDLE SecretHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaCreateSecret(%p %p 0x%08lx %p)\n",
//           PolicyHandle, SecretName, DesiredAccess, SecretHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarCreateSecret((LSAPR_HANDLE)PolicyHandle,
//                                   (PRPC_UNICODE_STRING)SecretName,
//                                   DesiredAccess,
//                                   SecretHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaCreateTrustedDomain(IN LSA_HANDLE PolicyHandle,
//                        IN PLSA_TRUST_INFORMATION TrustedDomainInformation,
//                        IN ACCESS_MASK DesiredAccess,
//                        OUT PLSA_HANDLE TrustedDomainHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaCreateTrustedDomain(%p %p 0x%08lx %p)\n",
//           PolicyHandle, TrustedDomainInformation, DesiredAccess, TrustedDomainHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarCreateTrustedDomain((LSAPR_HANDLE)PolicyHandle,
//                                          (PLSAPR_TRUST_INFORMATION)TrustedDomainInformation,
//                                          DesiredAccess,
//                                          (PLSAPR_HANDLE)TrustedDomainHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// LsaCreateTrustedDomainEx(IN LSA_HANDLE PolicyHandle,
//                          IN PTRUSTED_DOMAIN_INFORMATION_EX TrustedDomainInformation,
//                          IN PTRUSTED_DOMAIN_AUTH_INFORMATION AuthenticationInformation,
//                          IN ACCESS_MASK DesiredAccess,
//                          OUT PLSA_HANDLE TrustedDomainHandle)
// {
//     PLSAPR_TRUSTED_DOMAIN_AUTH_INFORMATION_INTERNAL EncryptedAuthInfo = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaCreateTrustedDomainEx(%p %p %p 0x%08lx %p) stub\n",
//           PolicyHandle, TrustedDomainInformation, AuthenticationInformation,
//           DesiredAccess, TrustedDomainHandle);
//
//     RpcTryExcept
//     {
//         /* FIXME: Encrypt AuthenticationInformation */
//
//         Status = LsarCreateTrustedDomainEx2((LSAPR_HANDLE)PolicyHandle,
//                                             (PLSAPR_TRUSTED_DOMAIN_INFORMATION_EX)TrustedDomainInformation,
//                                             EncryptedAuthInfo,
//                                             DesiredAccess,
//                                             (PLSAPR_HANDLE)TrustedDomainHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaDelete(IN LSA_HANDLE ObjectHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaDelete(%p)\n", ObjectHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarDelete((LSAPR_HANDLE)ObjectHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaDeleteTrustedDomain(IN LSA_HANDLE PolicyHandle,
//                        IN PSID TrustedDomainSid)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaDeleteTrustedDomain(%p %p)\n",
//           PolicyHandle, TrustedDomainSid);
//
//     RpcTryExcept
//     {
//         Status = LsarDeleteTrustedDomain((LSAPR_HANDLE)PolicyHandle,
//                                          TrustedDomainSid);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumerateAccountRights(IN LSA_HANDLE PolicyHandle,
//                           IN PSID AccountSid,
//                           OUT PLSA_UNICODE_STRING *UserRights,
//                           OUT PULONG CountOfRights)
// {
//     LSAPR_USER_RIGHT_SET UserRightsSet;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateAccountRights(%p %p %p %p)\n",
//           PolicyHandle, AccountSid, UserRights, CountOfRights);
//
//     UserRightsSet.Entries = 0;
//     UserRightsSet.UserRights = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumerateAccountRights((LSAPR_HANDLE)PolicyHandle,
//                                             AccountSid,
//                                             &UserRightsSet);
//
//         *UserRights = (PUNICODE_STRING)UserRightsSet.UserRights;
//         *CountOfRights = UserRightsSet.Entries;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//
//         if (UserRightsSet.UserRights != NULL)
//             MIDL_user_free(UserRightsSet.UserRights);
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumerateAccounts(IN LSA_HANDLE PolicyHandle,
//                      IN OUT PLSA_ENUMERATION_HANDLE EnumerationContext,
//                      OUT PVOID *Buffer,
//                      IN ULONG PreferedMaximumLength,
//                      OUT PULONG CountReturned)
// {
//     LSAPR_ACCOUNT_ENUM_BUFFER AccountEnumBuffer;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateAccounts(%p %p %p %lu %p)\n",
//           PolicyHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     AccountEnumBuffer.EntriesRead = 0;
//     AccountEnumBuffer.Information = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumerateAccounts((LSAPR_HANDLE)PolicyHandle,
//                                        EnumerationContext,
//                                        &AccountEnumBuffer,
//                                        PreferedMaximumLength);
//
//         *Buffer = AccountEnumBuffer.Information as _;
//         *CountReturned = AccountEnumBuffer.EntriesRead;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (AccountEnumBuffer.Information != NULL)
//             MIDL_user_free(AccountEnumBuffer.Information);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumerateAccountsWithUserRight(IN LSA_HANDLE PolicyHandle,
//                                   IN PLSA_UNICODE_STRING UserRight OPTIONAL,
//                                   OUT PVOID *Buffer,
//                                   OUT PULONG CountReturned)
// {
//     LSAPR_ACCOUNT_ENUM_BUFFER AccountEnumBuffer;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateAccountsWithUserRight(%p %p %p %p) stub\n",
//           PolicyHandle, UserRight, Buffer, CountReturned);
//
//     AccountEnumBuffer.EntriesRead = 0;
//     AccountEnumBuffer.Information = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumerateAccountsWithUserRight((LSAPR_HANDLE)PolicyHandle,
//                                                     (PRPC_UNICODE_STRING)UserRight,
//                                                     &AccountEnumBuffer);
//
//         *Buffer = AccountEnumBuffer.Information as _;
//         *CountReturned = AccountEnumBuffer.EntriesRead;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (AccountEnumBuffer.Information != NULL)
//             MIDL_user_free(AccountEnumBuffer.Information);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumeratePrivileges(IN LSA_HANDLE PolicyHandle,
//                        IN OUT PLSA_ENUMERATION_HANDLE EnumerationContext,
//                        OUT PVOID *Buffer,
//                        IN ULONG PreferedMaximumLength,
//                        OUT PULONG CountReturned)
// {
//     LSAPR_PRIVILEGE_ENUM_BUFFER PrivilegeEnumBuffer;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumeratePrivileges(%p %p %p %lu %p)\n",
//           PolicyHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     PrivilegeEnumBuffer.Entries = 0;
//     PrivilegeEnumBuffer.Privileges = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumeratePrivileges((LSAPR_HANDLE)PolicyHandle,
//                                          EnumerationContext,
//                                          &PrivilegeEnumBuffer,
//                                          PreferedMaximumLength);
//
//         *Buffer = PrivilegeEnumBuffer.Privileges;
//         *CountReturned = PrivilegeEnumBuffer.Entries;
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (PrivilegeEnumBuffer.Privileges != NULL)
//             MIDL_user_free(PrivilegeEnumBuffer.Privileges);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumeratePrivilegesOfAccount(IN LSA_HANDLE AccountHandle,
//                                 OUT PPRIVILEGE_SET *Privileges)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaEnumeratePrivilegesOfAccount(%p %p)\n",
//           AccountHandle, Privileges);
//
//     RpcTryExcept
//     {
//         Status = LsarEnumeratePrivilegesAccount((LSAPR_HANDLE)AccountHandle,
//                                                 (LSAPR_PRIVILEGE_SET **)Privileges);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumerateTrustedDomains(IN LSA_HANDLE PolicyHandle,
//                            IN OUT PLSA_ENUMERATION_HANDLE EnumerationContext,
//                            OUT PVOID *Buffer,
//                            IN ULONG PreferedMaximumLength,
//                            OUT PULONG CountReturned)
// {
//     LSAPR_TRUSTED_ENUM_BUFFER TrustedEnumBuffer;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateTrustedDomains(%p %p %p %lu %p)\n",
//           PolicyHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     if (Buffer == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     TrustedEnumBuffer.EntriesRead = 0;
//     TrustedEnumBuffer.Information = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumerateTrustedDomains((LSAPR_HANDLE)PolicyHandle,
//                                              EnumerationContext,
//                                              &TrustedEnumBuffer,
//                                              PreferedMaximumLength);
//
//         *Buffer = TrustedEnumBuffer.Information;
//         *CountReturned = TrustedEnumBuffer.EntriesRead;
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TrustedEnumBuffer.Information != NULL)
//             MIDL_user_free(TrustedEnumBuffer.Information);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaEnumerateTrustedDomainsEx(IN LSA_HANDLE PolicyHandle,
//                              IN OUT PLSA_ENUMERATION_HANDLE EnumerationContext,
//                              OUT PVOID *Buffer,
//                              IN ULONG PreferedMaximumLength,
//                              OUT PULONG CountReturned)
// {
//     LSAPR_TRUSTED_ENUM_BUFFER_EX TrustedEnumBuffer;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateTrustedDomainsEx(%p %p %p %lu %p)\n",
//           PolicyHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     if (Buffer == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     TrustedEnumBuffer.EntriesRead = 0;
//     TrustedEnumBuffer.EnumerationBuffer = NULL;
//
//     RpcTryExcept
//     {
//         Status = LsarEnumerateTrustedDomainsEx((LSAPR_HANDLE)PolicyHandle,
//                                                EnumerationContext,
//                                                &TrustedEnumBuffer,
//                                                PreferedMaximumLength);
//
//         *Buffer = TrustedEnumBuffer.EnumerationBuffer;
//         *CountReturned = TrustedEnumBuffer.EntriesRead;
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TrustedEnumBuffer.EnumerationBuffer != NULL)
//             MIDL_user_free(TrustedEnumBuffer.EnumerationBuffer);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaFreeMemory(IN PVOID Buffer)
// {
//     TRACE("LsaFreeMemory(%p)\n", Buffer);
//     return RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaGetQuotasForAccount(IN LSA_HANDLE AccountHandle,
//                        OUT PQUOTA_LIMITS QuotaLimits)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaGetQuotasForAccount(%p %p)\n",
//           AccountHandle, QuotaLimits);
//
//     RpcTryExcept
//     {
//         Status = LsarGetQuotasForAccount((LSAPR_HANDLE)AccountHandle,
//                                          QuotaLimits);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaGetRemoteUserName(IN PLSA_UNICODE_STRING SystemName OPTIONAL,
//                      OUT PLSA_UNICODE_STRING *UserName,
//                      OUT PLSA_UNICODE_STRING *DomainName OPTIONAL)
// {
//     PRPC_UNICODE_STRING UserNameString = NULL;
//     PRPC_UNICODE_STRING DomainNameString = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaGetRemoteUserName(%s %p %p)\n",
//           SystemName ? debugstr_w(SystemName->Buffer) : "(null)",
//           UserName, DomainName);
//
//     RpcTryExcept
//     {
//         Status = LsarGetUserName((PLSAPR_SERVER_NAME)SystemName,
//                                  &UserNameString,
//                                  (DomainName != NULL) ? &DomainNameString : NULL);
//
//         *UserName = (PLSA_UNICODE_STRING)UserNameString;
//
//         if (DomainName != NULL)
//             *DomainName = (PLSA_UNICODE_STRING)DomainNameString;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (UserNameString != NULL)
//             MIDL_user_free(UserNameString);
//
//         if (DomainNameString != NULL)
//             MIDL_user_free(DomainNameString);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaGetSystemAccessAccount(IN LSA_HANDLE AccountHandle,
//                           OUT PULONG SystemAccess)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaGetSystemAccessAccount(%p %p)\n",
//           AccountHandle, SystemAccess);
//
//     RpcTryExcept
//     {
//         Status = LsarGetSystemAccessAccount((LSAPR_HANDLE)AccountHandle,
//                                             (ACCESS_MASK *)SystemAccess);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaGetUserName(OUT PUNICODE_STRING *UserName,
//                OUT PUNICODE_STRING *DomainName OPTIONAL)
// {
//     PRPC_UNICODE_STRING UserNameString = NULL;
//     PRPC_UNICODE_STRING DomainNameString = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaGetUserName(%p %p)\n",
//           UserName, DomainName);
//
//     RpcTryExcept
//     {
//         Status = LsarGetUserName(NULL,
//                                  &UserNameString,
//                                  (DomainName != NULL) ? &DomainNameString : NULL);
//
//         *UserName = (PUNICODE_STRING)UserNameString;
//
//         if (DomainName != NULL)
//             *DomainName = (PUNICODE_STRING)DomainNameString;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (UserNameString != NULL)
//             MIDL_user_free(UserNameString);
//
//         if (DomainNameString != NULL)
//             MIDL_user_free(DomainNameString);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupNames(IN LSA_HANDLE PolicyHandle,
//                IN ULONG Count,
//                IN PLSA_UNICODE_STRING Names,
//                OUT PLSA_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//                OUT PLSA_TRANSLATED_SID *Sids)
// {
//     LSAPR_TRANSLATED_SIDS TranslatedSids = {0, NULL};
//     ULONG MappedCount = 0;
//     NTSTATUS Status;
//
//     TRACE("LsaLookupNames(%p %lu %p %p %p)\n",
//           PolicyHandle, Count, Names, ReferencedDomains, Sids);
//
//     if (ReferencedDomains == NULL || Sids == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         *ReferencedDomains = NULL;
//         *Sids = NULL;
//
//         TranslatedSids.Entries = Count;
//
//         Status = LsarLookupNames((LSAPR_HANDLE)PolicyHandle,
//                                  Count,
//                                  (PRPC_UNICODE_STRING)Names,
//                                  (PLSAPR_REFERENCED_DOMAIN_LIST *)ReferencedDomains,
//                                  &TranslatedSids,
//                                  LsapLookupWksta,
//                                  &MappedCount);
//
//         *Sids = (PLSA_TRANSLATED_SID)TranslatedSids.Sids;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TranslatedSids.Sids != NULL)
//             MIDL_user_free(TranslatedSids.Sids);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupNames2(IN LSA_HANDLE PolicyHandle,
//                 IN ULONG Flags,
//                 IN ULONG Count,
//                 IN PLSA_UNICODE_STRING Names,
//                 OUT PLSA_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//                 OUT PLSA_TRANSLATED_SID2 *Sids)
// {
//     LSAPR_TRANSLATED_SIDS_EX2 TranslatedSids = {0, NULL};
//     ULONG MappedCount = 0;
//     NTSTATUS Status;
//
//     TRACE("LsaLookupNames2(%p 0x%08x %lu %p %p %p)\n",
//           PolicyHandle, Flags, Count, Names, ReferencedDomains, Sids);
//
//     if (ReferencedDomains == NULL || Sids == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         *ReferencedDomains = NULL;
//         *Sids = NULL;
//
//         TranslatedSids.Entries = Count;
//
//         Status = LsarLookupNames3((LSAPR_HANDLE)PolicyHandle,
//                                   Count,
//                                   (PRPC_UNICODE_STRING)Names,
//                                   (PLSAPR_REFERENCED_DOMAIN_LIST *)ReferencedDomains,
//                                   &TranslatedSids,
//                                   LsapLookupWksta,
//                                   &MappedCount,
//                                   Flags,
//                                   2);
//
//         *Sids = (PLSA_TRANSLATED_SID2)TranslatedSids.Sids;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TranslatedSids.Sids != NULL)
//             MIDL_user_free(TranslatedSids.Sids);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupPrivilegeDisplayName(IN LSA_HANDLE PolicyHandle,
//                               IN PLSA_UNICODE_STRING Name,
//                               OUT PLSA_UNICODE_STRING *DisplayName,
//                               OUT PUSHORT LanguageReturned)
// {
//     PRPC_UNICODE_STRING DisplayNameBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaLookupPrivilegeDisplayName(%p %p %p %p)\n",
//           PolicyHandle, Name, DisplayName, LanguageReturned);
//
//     RpcTryExcept
//     {
//         Status = LsarLookupPrivilegeDisplayName(PolicyHandle,
//                                                 (PRPC_UNICODE_STRING)Name,
//                                                 GetUserDefaultUILanguage(),
//                                                 GetSystemDefaultUILanguage(),
//                                                 &DisplayNameBuffer,
//                                                 LanguageReturned);
//
//         *DisplayName = (PUNICODE_STRING)DisplayNameBuffer;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (DisplayNameBuffer != NULL)
//             MIDL_user_free(DisplayNameBuffer);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupPrivilegeName(IN LSA_HANDLE PolicyHandle,
//                        IN PLUID Value,
//                        OUT PUNICODE_STRING *Name)
// {
//     PRPC_UNICODE_STRING NameBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaLookupPrivilegeName(%p %p %p)\n",
//           PolicyHandle, Value, Name);
//
//     RpcTryExcept
//     {
//         Status = LsarLookupPrivilegeName(PolicyHandle,
//                                          Value,
//                                          &NameBuffer);
//
//         *Name = (PUNICODE_STRING)NameBuffer;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (NameBuffer != NULL)
//             MIDL_user_free(NameBuffer);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupPrivilegeValue(IN LSA_HANDLE PolicyHandle,
//                         IN PLSA_UNICODE_STRING Name,
//                         OUT PLUID Value)
// {
//     LUID Luid;
//     NTSTATUS Status;
//
//     TRACE("LsaLookupPrivilegeValue(%p %p %p)\n",
//           PolicyHandle, Name, Value);
//
//     RpcTryExcept
//     {
//         Status = LsarLookupPrivilegeValue(PolicyHandle,
//                                           (PRPC_UNICODE_STRING)Name,
//                                           &Luid);
//         if (Status == STATUS_SUCCESS)
//             *Value = Luid;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaLookupSids(IN LSA_HANDLE PolicyHandle,
//               IN ULONG Count,
//               IN PSID *Sids,
//               OUT PLSA_REFERENCED_DOMAIN_LIST *ReferencedDomains,
//               OUT PLSA_TRANSLATED_NAME *Names)
// {
//     LSAPR_SID_ENUM_BUFFER SidEnumBuffer;
//     LSAPR_TRANSLATED_NAMES TranslatedNames;
//     ULONG MappedCount = 0;
//     NTSTATUS  Status;
//
//     TRACE("LsaLookupSids(%p %lu %p %p %p)\n",
//           PolicyHandle, Count, Sids, ReferencedDomains, Names);
//
//     if (Count == 0)
//         return STATUS_INVALID_PARAMETER;
//
//     SidEnumBuffer.Entries = Count;
//     SidEnumBuffer.SidInfo = (PLSAPR_SID_INFORMATION)Sids;
//
//     RpcTryExcept
//     {
//         *ReferencedDomains = NULL;
//         *Names = NULL;
//
//         TranslatedNames.Entries = 0;
//         TranslatedNames.Names = NULL;
//
//         Status = LsarLookupSids((LSAPR_HANDLE)PolicyHandle,
//                                 &SidEnumBuffer,
//                                 (PLSAPR_REFERENCED_DOMAIN_LIST *)ReferencedDomains,
//                                 &TranslatedNames,
//                                 LsapLookupWksta,
//                                 &MappedCount);
//
//         *Names = (PLSA_TRANSLATED_NAME)TranslatedNames.Names;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TranslatedNames.Names != NULL)
//         {
//             MIDL_user_free(TranslatedNames.Names);
//         }
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /******************************************************************************
//  * LsaNtStatusToWinError
//  *
//  * PARAMS
//  *   Status [I]
//  *
//  * @implemented
//  */
// ULONG
// WINAPI
// LsaNtStatusToWinError(IN NTSTATUS Status)
// {
//     TRACE("LsaNtStatusToWinError(0x%lx)\n", Status);
//     return RtlNtStatusToDosError(Status);
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenAccount(IN LSA_HANDLE PolicyHandle,
//                IN PSID AccountSid,
//                IN ACCESS_MASK DesiredAccess,
//                OUT PLSA_HANDLE AccountHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenAccount(%p %p 0x%08lx %p)\n",
//           PolicyHandle, AccountSid, DesiredAccess, AccountHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarOpenAccount((LSAPR_HANDLE)PolicyHandle,
//                                  AccountSid,
//                                  DesiredAccess,
//                                  AccountHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /******************************************************************************
//  * LsaOpenPolicy
//  *
//  * PARAMS
//  *   x1 []
//  *   x2 []
//  *   x3 []
//  *   x4 []
//  *
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenPolicy(IN PLSA_UNICODE_STRING SystemName OPTIONAL,
//               IN PLSA_OBJECT_ATTRIBUTES ObjectAttributes,
//               IN ACCESS_MASK DesiredAccess,
//               OUT PLSA_HANDLE PolicyHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenPolicy(%s %p 0x%08lx %p)\n",
//           SystemName ? debugstr_w(SystemName->Buffer) : "(null)",
//           ObjectAttributes, DesiredAccess, PolicyHandle);
//
//     /* FIXME: RPC should take care of this */
//     if (!LsapIsLocalComputer(SystemName))
//         return RPC_NT_SERVER_UNAVAILABLE;
//
//     RpcTryExcept
//     {
//         *PolicyHandle = NULL;
//
//         Status = LsarOpenPolicy(SystemName ? SystemName->Buffer : NULL,
//                                 (PLSAPR_OBJECT_ATTRIBUTES)ObjectAttributes,
//                                 DesiredAccess,
//                                 PolicyHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("LsaOpenPolicy() done (Status: 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenPolicySce(IN PLSA_UNICODE_STRING SystemName OPTIONAL,
//                  IN PLSA_OBJECT_ATTRIBUTES ObjectAttributes,
//                  IN ACCESS_MASK DesiredAccess,
//                  OUT PLSA_HANDLE PolicyHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenPolicySce(%s %p 0x%08lx %p)\n",
//           SystemName ? debugstr_w(SystemName->Buffer) : "(null)",
//           ObjectAttributes, DesiredAccess, PolicyHandle);
//
//     /* FIXME: RPC should take care of this */
//     if (!LsapIsLocalComputer(SystemName))
//         return RPC_NT_SERVER_UNAVAILABLE;
//
//     RpcTryExcept
//     {
//         *PolicyHandle = NULL;
//
//         Status = LsarOpenPolicySce(SystemName ? SystemName->Buffer : NULL,
//                                    (PLSAPR_OBJECT_ATTRIBUTES)ObjectAttributes,
//                                    DesiredAccess,
//                                    PolicyHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("LsaOpenPolicySce() done (Status: 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenSecret(IN LSA_HANDLE PolicyHandle,
//               IN PLSA_UNICODE_STRING SecretName,
//               IN ACCESS_MASK DesiredAccess,
//               OUT PLSA_HANDLE SecretHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenSecret(%p %p 0x%08lx %p)\n",
//           PolicyHandle, SecretName, DesiredAccess, SecretHandle);
//
//     RpcTryExcept
//     {
//         *SecretHandle = NULL;
//
//         Status = LsarOpenSecret((LSAPR_HANDLE)PolicyHandle,
//                                 (PRPC_UNICODE_STRING)SecretName,
//                                 DesiredAccess,
//                                 SecretHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("LsaOpenSecret() done (Status: 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenTrustedDomain(IN LSA_HANDLE PolicyHandle,
//                      IN PSID TrustedDomainSid,
//                      IN ACCESS_MASK DesiredAccess,
//                      OUT PLSA_HANDLE TrustedDomainHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenTrustedDomain(%p %p 0x%08lx %p)\n",
//           PolicyHandle, TrustedDomainSid, DesiredAccess, TrustedDomainHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarOpenTrustedDomain((LSAPR_HANDLE)PolicyHandle,
//                                        (PRPC_SID)TrustedDomainSid,
//                                        DesiredAccess,
//                                        (PLSAPR_HANDLE)TrustedDomainHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaOpenTrustedDomainByName(IN LSA_HANDLE PolicyHandle,
//                            IN PLSA_UNICODE_STRING TrustedDomainName,
//                            IN ACCESS_MASK DesiredAccess,
//                            OUT PLSA_HANDLE TrustedDomainHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaOpenTrustedDomainByName(%p %p 0x%08lx %p)\n",
//           PolicyHandle, TrustedDomainName, DesiredAccess, TrustedDomainHandle);
//
//     RpcTryExcept
//     {
//         Status = LsarOpenTrustedDomainByName((LSAPR_HANDLE)PolicyHandle,
//                                              (PRPC_UNICODE_STRING)TrustedDomainName,
//                                              DesiredAccess,
//                                              TrustedDomainHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryDomainInformationPolicy(IN LSA_HANDLE PolicyHandle,
//                                 IN POLICY_DOMAIN_INFORMATION_CLASS InformationClass,
//                                 OUT PVOID *Buffer)
// {
//     PLSAPR_POLICY_DOMAIN_INFORMATION PolicyInformation = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaQueryDomainInformationPolicy(%p %lu %p)\n",
//           PolicyHandle, InformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = LsarQueryDomainInformationPolicy((LSAPR_HANDLE)PolicyHandle,
//                                                   InformationClass,
//                                                   &PolicyInformation);
//
//         *Buffer = PolicyInformation;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (PolicyInformation != NULL)
//             MIDL_user_free(PolicyInformation);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryForestTrustInformation(IN LSA_HANDLE PolicyHandle,
//                                IN PLSA_UNICODE_STRING TrustedDomainName,
//                                OUT PLSA_FOREST_TRUST_INFORMATION *ForestTrustInfo)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaQueryForestTrustInformation(%p %p %p)\n",
//           PolicyHandle, TrustedDomainName, ForestTrustInfo);
//
//     RpcTryExcept
//     {
//         Status = LsarQueryForestTrustInformation((LSAPR_HANDLE)PolicyHandle,
//                                                  TrustedDomainName,
//                                                  ForestTrustDomainInfo,
//                                                  ForestTrustInfo);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryInfoTrustedDomain(IN LSA_HANDLE TrustedDomainHandle,
//                           IN TRUSTED_INFORMATION_CLASS InformationClass,
//                           OUT PVOID *Buffer)
// {
//     PLSAPR_TRUSTED_DOMAIN_INFO TrustedDomainInformation = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaQueryInfoTrustedDomain(%p %d %p) stub\n",
//           TrustedDomainHandle, InformationClass, Buffer);
//
//     if (InformationClass == TrustedDomainAuthInformationInternal ||
//         InformationClass == TrustedDomainFullInformationInternal)
//         return STATUS_INVALID_INFO_CLASS;
//
//     RpcTryExcept
//     {
//         Status = LsarQueryInfoTrustedDomain((LSAPR_HANDLE)TrustedDomainHandle,
//                                             InformationClass,
//                                             &TrustedDomainInformation);
//         *Buffer = TrustedDomainInformation;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TrustedDomainInformation != NULL)
//             MIDL_user_free(TrustedDomainInformation);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("Done (Status: 0x%08x)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryInformationPolicy(IN LSA_HANDLE PolicyHandle,
//                           IN POLICY_INFORMATION_CLASS InformationClass,
//                           OUT PVOID *Buffer)
// {
//     PLSAPR_POLICY_INFORMATION PolicyInformation = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsaQueryInformationPolicy(%p %d %p)\n",
//           PolicyHandle, InformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = LsarQueryInformationPolicy((LSAPR_HANDLE)PolicyHandle,
//                                             InformationClass,
//                                             &PolicyInformation);
//         *Buffer = PolicyInformation;
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (PolicyInformation != NULL)
//             MIDL_user_free(PolicyInformation);
//
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("Done (Status: 0x%08x)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQuerySecret(IN LSA_HANDLE SecretHandle,
//                OUT PLSA_UNICODE_STRING *CurrentValue OPTIONAL,
//                OUT PLARGE_INTEGER CurrentValueSetTime OPTIONAL,
//                OUT PLSA_UNICODE_STRING *OldValue OPTIONAL,
//                OUT PLARGE_INTEGER OldValueSetTime OPTIONAL)
// {
//     PLSAPR_CR_CIPHER_VALUE EncryptedCurrentValue = NULL;
//     PLSAPR_CR_CIPHER_VALUE EncryptedOldValue = NULL;
//     PLSA_UNICODE_STRING DecryptedCurrentValue = NULL;
//     PLSA_UNICODE_STRING DecryptedOldValue = NULL;
//     SIZE_T BufferSize;
//     NTSTATUS Status;
//
//     TRACE("LsaQuerySecret(%p %p %p %p %p)\n",
//           SecretHandle, CurrentValue, CurrentValueSetTime,
//           OldValue, OldValueSetTime);
//
//     RpcTryExcept
//     {
//         Status = LsarQuerySecret((PLSAPR_HANDLE)SecretHandle,
//                                  &EncryptedCurrentValue,
//                                  CurrentValueSetTime,
//                                  &EncryptedOldValue,
//                                  OldValueSetTime);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Decrypt the current value */
//     if (CurrentValue != NULL)
//     {
//          if (EncryptedCurrentValue == NULL)
//          {
//              *CurrentValue = NULL;
//          }
//          else
//          {
//              /* FIXME: Decrypt the current value */
//              BufferSize = sizeof(LSA_UNICODE_STRING) + EncryptedCurrentValue->MaximumLength;
//              DecryptedCurrentValue = midl_user_allocate(BufferSize);
//              if (DecryptedCurrentValue == NULL)
//              {
//                  Status = STATUS_INSUFFICIENT_RESOURCES;
//                  goto done;
//              }
//
//              DecryptedCurrentValue->Length = (USHORT)EncryptedCurrentValue->Length;
//              DecryptedCurrentValue->MaximumLength = (USHORT)EncryptedCurrentValue->MaximumLength;
//              DecryptedCurrentValue->Buffer = (PWSTR)(DecryptedCurrentValue + 1);
//              RtlCopyMemory(DecryptedCurrentValue->Buffer,
//                            EncryptedCurrentValue->Buffer,
//                            EncryptedCurrentValue->Length);
//
//              *CurrentValue = DecryptedCurrentValue;
//          }
//     }
//
//     /* Decrypt the old value */
//     if (OldValue != NULL)
//     {
//          if (EncryptedOldValue == NULL)
//          {
//              *OldValue = NULL;
//          }
//          else
//          {
//              /* FIXME: Decrypt the old value */
//              BufferSize = sizeof(LSA_UNICODE_STRING) + EncryptedOldValue->MaximumLength;
//              DecryptedOldValue = midl_user_allocate(BufferSize);
//              if (DecryptedOldValue == NULL)
//              {
//                  Status = STATUS_INSUFFICIENT_RESOURCES;
//                  goto done;
//              }
//
//              DecryptedOldValue->Length = (USHORT)EncryptedOldValue->Length;
//              DecryptedOldValue->MaximumLength = (USHORT)EncryptedOldValue->MaximumLength;
//              DecryptedOldValue->Buffer = (PWSTR)(DecryptedOldValue + 1);
//              RtlCopyMemory(DecryptedOldValue->Buffer,
//                            EncryptedOldValue->Buffer,
//                            EncryptedOldValue->Length);
//
//              *OldValue = DecryptedOldValue;
//          }
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (DecryptedCurrentValue != NULL)
//             midl_user_free(DecryptedCurrentValue);
//
//         if (DecryptedOldValue != NULL)
//             midl_user_free(DecryptedOldValue);
//
//         if (CurrentValue != NULL)
//             *CurrentValue = NULL;
//
//         if (OldValue != NULL)
//             *OldValue = NULL;
//     }
//
//     if (EncryptedCurrentValue != NULL)
//         midl_user_free(EncryptedCurrentValue);
//
//     if (EncryptedOldValue != NULL)
//         midl_user_free(EncryptedOldValue);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQuerySecurityObject(IN LSA_HANDLE ObjectHandle,
//                        IN SECURITY_INFORMATION SecurityInformation,
//                        OUT PSECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     LSAPR_SR_SECURITY_DESCRIPTOR SdBuffer;
//     PLSAPR_SR_SECURITY_DESCRIPTOR SdPointer;
//     NTSTATUS Status;
//
//     TRACE("LsaQuerySecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     SdBuffer.Length = 0;
//     SdBuffer.SecurityDescriptor = NULL;
//
//     SdPointer = &SdBuffer;
//
//     RpcTryExcept
//     {
//         Status = LsarQuerySecurityObject((LSAPR_HANDLE)ObjectHandle,
//                                          SecurityInformation,
//                                          &SdPointer);
//         if (NT_SUCCESS(Status))
//         {
//             *SecurityDescriptor = SdBuffer.SecurityDescriptor;
//         }
//         else
//         {
//             *SecurityDescriptor = NULL;
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryTrustedDomainInfo(IN LSA_HANDLE PolicyHandle,
//                           IN PSID TrustedDomainSid,
//                           IN TRUSTED_INFORMATION_CLASS InformationClass,
//                           OUT PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaQueryTrustedDomainInfo(%p %p %d %p) stub\n",
//           PolicyHandle, TrustedDomainSid, InformationClass, Buffer);
//
//     if (InformationClass == TrustedDomainAuthInformationInternal ||
//         InformationClass == TrustedDomainFullInformationInternal)
//         return STATUS_INVALID_INFO_CLASS;
//
//     RpcTryExcept
//     {
//         Status = LsarQueryTrustedDomainInfo((LSAPR_HANDLE)PolicyHandle,
//                                             (PRPC_SID)TrustedDomainSid,
//                                             InformationClass,
//                                             (PLSAPR_TRUSTED_DOMAIN_INFO *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaQueryTrustedDomainInfoByName(IN LSA_HANDLE PolicyHandle,
//                                 IN PLSA_UNICODE_STRING TrustedDomainName,
//                                 IN TRUSTED_INFORMATION_CLASS InformationClass,
//                                 OUT PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaQueryTrustedDomainInfoByName(%p %p %d %p)\n",
//           PolicyHandle, TrustedDomainName, InformationClass, Buffer);
//
//     if (InformationClass == TrustedDomainAuthInformationInternal ||
//         InformationClass == TrustedDomainFullInformationInternal)
//         return STATUS_INVALID_INFO_CLASS;
//
//     RpcTryExcept
//     {
//         Status = LsarQueryTrustedDomainInfoByName((LSAPR_HANDLE)PolicyHandle,
//                                                   (PRPC_UNICODE_STRING)TrustedDomainName,
//                                                   InformationClass,
//                                                   (PLSAPR_TRUSTED_DOMAIN_INFO *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaRemoveAccountRights(IN LSA_HANDLE PolicyHandle,
//                        IN PSID AccountSid,
//                        IN BOOLEAN AllRights,
//                        IN PLSA_UNICODE_STRING UserRights,
//                        IN ULONG CountOfRights)
// {
//     NTSTATUS Status;
//     LSAPR_USER_RIGHT_SET UserRightSet;
//
//     TRACE("LsaRemoveAccountRights(%p %p %d %p %lu)\n",
//           PolicyHandle, AccountSid, AllRights, UserRights, CountOfRights);
//
//     UserRightSet.Entries = CountOfRights;
//     UserRightSet.UserRights = (PRPC_UNICODE_STRING)UserRights;
//
//     RpcTryExcept
//     {
//         Status = LsarRemoveAccountRights((LSAPR_HANDLE)PolicyHandle,
//                                          (PRPC_SID)AccountSid,
//                                          AllRights,
//                                          &UserRightSet);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaRemovePrivilegesFromAccount(IN LSA_HANDLE AccountHandle,
//                                IN BOOLEAN AllPrivileges,
//                                IN PPRIVILEGE_SET Privileges OPTIONAL)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = LsarRemovePrivilegesFromAccount((LSAPR_HANDLE)AccountHandle,
//                                                  AllPrivileges,
//                                                  (PLSAPR_PRIVILEGE_SET)Privileges);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaRetrievePrivateData(IN LSA_HANDLE PolicyHandle,
//                        IN PLSA_UNICODE_STRING KeyName,
//                        OUT PLSA_UNICODE_STRING *PrivateData)
// {
//     PLSAPR_CR_CIPHER_VALUE EncryptedData = NULL;
//     PLSA_UNICODE_STRING DecryptedData = NULL;
//     SIZE_T BufferSize;
//     NTSTATUS Status;
//
//     TRACE("LsaRetrievePrivateData(%p %p %p)\n",
//           PolicyHandle, KeyName, PrivateData);
//
//     RpcTryExcept
//     {
//         Status = LsarRetrievePrivateData((LSAPR_HANDLE)PolicyHandle,
//                                          (PRPC_UNICODE_STRING)KeyName,
//                                          &EncryptedData);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (EncryptedData == NULL)
//     {
//         *PrivateData = NULL;
//     }
//     else
//     {
//         BufferSize = sizeof(LSA_UNICODE_STRING) + EncryptedData->MaximumLength;
//         DecryptedData = midl_user_allocate(BufferSize);
//         if (DecryptedData == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         DecryptedData->Length = (USHORT)EncryptedData->Length;
//         DecryptedData->MaximumLength = (USHORT)EncryptedData->MaximumLength;
//         DecryptedData->Buffer = (PWSTR)(DecryptedData + 1);
//         RtlCopyMemory(DecryptedData->Buffer,
//                       EncryptedData->Buffer,
//                       EncryptedData->Length);
//
//         *PrivateData = DecryptedData;
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (DecryptedData != NULL)
//             midl_user_free(DecryptedData);
//
//         *PrivateData = NULL;
//     }
//
//     if (EncryptedData != NULL)
//         midl_user_free(EncryptedData);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetDomainInformationPolicy(IN LSA_HANDLE PolicyHandle,
//                               IN POLICY_DOMAIN_INFORMATION_CLASS InformationClass,
//                               IN PVOID Buffer OPTIONAL)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaSetDomainInformationPolicy(%p %d %p)\n",
//           PolicyHandle, InformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = LsarSetDomainInformationPolicy((LSAPR_HANDLE)PolicyHandle,
//                                                 InformationClass,
//                                                 (PLSAPR_POLICY_DOMAIN_INFORMATION)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetForestTrustInformation(IN LSA_HANDLE PolicyHandle,
//                              IN PLSA_UNICODE_STRING TrustedDomainName,
//                              IN PLSA_FOREST_TRUST_INFORMATION ForestTrustInfo,
//                              IN BOOLEAN CheckOnly,
//                              OUT PLSA_FOREST_TRUST_COLLISION_INFORMATION *CollisionInfo)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaSetForestTrustInformation(%p %p %p %d %p)\n",
//           PolicyHandle, TrustedDomainName, ForestTrustInfo, CheckOnly, CollisionInfo);
//
//     RpcTryExcept
//     {
//         Status = LsarSetForestTrustInformation((LSAPR_HANDLE)PolicyHandle,
//                                                TrustedDomainName,
//                                                ForestTrustDomainInfo,
//                                                ForestTrustInfo,
//                                                CheckOnly,
//                                                CollisionInfo);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetInformationPolicy(IN LSA_HANDLE PolicyHandle,
//                         IN POLICY_INFORMATION_CLASS InformationClass,
//                         IN PVOID Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaSetInformationPolicy(%p %d %p)\n",
//           PolicyHandle, InformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = LsarSetInformationPolicy((LSAPR_HANDLE)PolicyHandle,
//                                           InformationClass,
//                                           (PLSAPR_POLICY_INFORMATION)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// LsaSetInformationTrustedDomain(IN LSA_HANDLE TrustedDomainHandle,
//                                IN TRUSTED_INFORMATION_CLASS InformationClass,
//                                IN PVOID Buffer)
// {
//     FIXME("LsaSetInformationTrustedDomain(%p %d %p)\n",
//           TrustedDomainHandle, InformationClass, Buffer);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetQuotasForAccount(IN LSA_HANDLE AccountHandle,
//                        IN PQUOTA_LIMITS QuotaLimits)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaSetQuotasForAccount(%p %p)\n",
//           AccountHandle, QuotaLimits);
//
//     RpcTryExcept
//     {
//         Status = LsarSetQuotasForAccount((LSAPR_HANDLE)AccountHandle,
//                                          QuotaLimits);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetSecret(IN LSA_HANDLE SecretHandle,
//              IN PLSA_UNICODE_STRING CurrentValue OPTIONAL,
//              IN PLSA_UNICODE_STRING OldValue OPTIONAL)
// {
//     PLSAPR_CR_CIPHER_VALUE EncryptedCurrentValue = NULL;
//     PLSAPR_CR_CIPHER_VALUE EncryptedOldValue = NULL;
//     SIZE_T BufferSize;
//     NTSTATUS Status;
//
//     TRACE("LsaSetSecret(%p %p %p)\n",
//           SecretHandle, EncryptedCurrentValue, EncryptedOldValue);
//
//     if (CurrentValue != NULL)
//     {
//         BufferSize = sizeof(LSAPR_CR_CIPHER_VALUE) + CurrentValue->MaximumLength;
//         EncryptedCurrentValue = midl_user_allocate(BufferSize);
//         if (EncryptedCurrentValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         EncryptedCurrentValue->Length = CurrentValue->Length;
//         EncryptedCurrentValue->MaximumLength = CurrentValue->MaximumLength;
//         EncryptedCurrentValue->Buffer = (BYTE *)(EncryptedCurrentValue + 1);
//         if (EncryptedCurrentValue->Buffer != NULL)
//             memcpy(EncryptedCurrentValue->Buffer, CurrentValue->Buffer, CurrentValue->Length);
//     }
//
//     if (OldValue != NULL)
//     {
//         BufferSize = sizeof(LSAPR_CR_CIPHER_VALUE) + OldValue->MaximumLength;
//         EncryptedOldValue = midl_user_allocate(BufferSize);
//         if (EncryptedOldValue == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         EncryptedOldValue->Length = OldValue->Length;
//         EncryptedOldValue->MaximumLength = OldValue->MaximumLength;
//         EncryptedOldValue->Buffer = (BYTE*)(EncryptedOldValue + 1);
//         if (EncryptedOldValue->Buffer != NULL)
//             memcpy(EncryptedOldValue->Buffer, OldValue->Buffer, OldValue->Length);
//     }
//
//     RpcTryExcept
//     {
//         Status = LsarSetSecret((LSAPR_HANDLE)SecretHandle,
//                                EncryptedCurrentValue,
//                                EncryptedOldValue);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
// done:
//     if (EncryptedCurrentValue != NULL)
//         midl_user_free(EncryptedCurrentValue);
//
//     if (EncryptedOldValue != NULL)
//         midl_user_free(EncryptedOldValue);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetSecurityObject(IN LSA_HANDLE ObjectHandle,
//                      IN SECURITY_INFORMATION SecurityInformation,
//                      IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     LSAPR_SR_SECURITY_DESCRIPTOR SdBuffer = {0, NULL};
//     ULONG SdLength = 0;
//     NTSTATUS Status;
//
//     TRACE("LsaSetSecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     Status = RtlMakeSelfRelativeSD(SecurityDescriptor,
//                                    NULL,
//                                    &SdLength);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         return STATUS_INVALID_PARAMETER;
//
//     SdBuffer.SecurityDescriptor = MIDL_user_allocate(SdLength);
//     if (SdBuffer.SecurityDescriptor == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = RtlMakeSelfRelativeSD(SecurityDescriptor,
//                                    (PSECURITY_DESCRIPTOR)SdBuffer.SecurityDescriptor,
//                                    &SdLength);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     SdBuffer.Length = SdLength;
//
//     RpcTryExcept
//     {
//         Status = LsarSetSecurityObject((LSAPR_HANDLE)ObjectHandle,
//                                        SecurityInformation,
//                                        &SdBuffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
// done:
//     if (SdBuffer.SecurityDescriptor != NULL)
//         MIDL_user_free(SdBuffer.SecurityDescriptor);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaSetSystemAccessAccount(IN LSA_HANDLE AccountHandle,
//                           IN ULONG SystemAccess)
// {
//     NTSTATUS Status;
//
//     TRACE("LsaSetSystemAccessAccount(%p 0x%lx)\n",
//           AccountHandle, SystemAccess);
//
//     RpcTryExcept
//     {
//         Status = LsarSetSystemAccessAccount((LSAPR_HANDLE)AccountHandle,
//                                             SystemAccess);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// LsaSetTrustedDomainInfoByName(IN LSA_HANDLE PolicyHandle,
//                               IN PLSA_UNICODE_STRING TrustedDomainName,
//                               IN TRUSTED_INFORMATION_CLASS InformationClass,
//                               IN PVOID Buffer)
// {
//     FIXME("LsaSetTrustedDomainInfoByName(%p %p %d %p) stub\n",
//           PolicyHandle, TrustedDomainName, InformationClass, Buffer);
//     return STATUS_SUCCESS;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// LsaSetTrustedDomainInformation(IN LSA_HANDLE PolicyHandle,
//                                IN PSID TrustedDomainSid,
//                                IN TRUSTED_INFORMATION_CLASS InformationClass,
//                                IN PVOID Buffer)
// {
//     FIXME("LsaSetTrustedDomainInformation(%p %p %d %p) stub\n",
//           PolicyHandle, TrustedDomainSid, InformationClass, Buffer);
//     return STATUS_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// LsaStorePrivateData(IN LSA_HANDLE PolicyHandle,
//                     IN PLSA_UNICODE_STRING KeyName,
//                     IN PLSA_UNICODE_STRING PrivateData OPTIONAL)
// {
//     PLSAPR_CR_CIPHER_VALUE EncryptedData = NULL;
//     SIZE_T BufferSize;
//     NTSTATUS Status;
//
//     TRACE("LsaStorePrivateData(%p %p %p)\n",
//           PolicyHandle, KeyName, PrivateData);
//
//     if (PrivateData != NULL)
//     {
//         BufferSize = sizeof(LSAPR_CR_CIPHER_VALUE) + PrivateData->MaximumLength;
//         EncryptedData = midl_user_allocate(BufferSize);
//         if (EncryptedData == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         EncryptedData->Length = PrivateData->Length;
//         EncryptedData->MaximumLength = PrivateData->MaximumLength;
//         EncryptedData->Buffer = (BYTE *)(EncryptedData + 1);
//         if (EncryptedData->Buffer != NULL)
//             RtlCopyMemory(EncryptedData->Buffer,
//                           PrivateData->Buffer,
//                           PrivateData->Length);
//     }
//
//     RpcTryExcept
//     {
//         Status = LsarStorePrivateData((LSAPR_HANDLE)PolicyHandle,
//                                       (PRPC_UNICODE_STRING)KeyName,
//                                       EncryptedData);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
// done:
//     if (EncryptedData != NULL)
//         midl_user_free(EncryptedData);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
