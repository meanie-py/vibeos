// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_387E76FBE5B7: [u16; 15] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x006E, 0x0065, 0x0074, 0x006C,
    0x006F, 0x0067, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS;

    Status = RpcServerUseProtseqEpW(RUSTCONV_WSTR_12A6BF85117F.as_ptr() as *mut u16, 20, RUSTCONV_WSTR_387E76FBE5B7.as_ptr() as *mut u16, core::ptr::null_mut());
    if Status != RPC_S_OK {
        ERR!("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = RpcServerRegisterIf(logon_v1_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
    if Status != RPC_S_OK {
        ERR!("RpcServerRegisterIf() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
    if Status != RPC_S_OK {
        ERR!("RpcServerListen() failed (Status %lx)\n", Status);
    }

    return 0;
}

#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, len) as *mut core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut ::core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 as DWORD, ptr as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonUasLogon(
    ServerName: LOGONSRV_HANDLE,
    UserName: *mut wchar_t,
    Workstation: *mut wchar_t,
    ValidationInformation: *mut PNETLOGON_VALIDATION_UAS_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonUasLogoff(
    ServerName: LOGONSRV_HANDLE,
    UserName: *mut wchar_t,
    Workstation: *mut wchar_t,
    LogoffInformation: PNETLOGON_LOGOFF_UAS_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSamLogon(
    LogonServer: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    LogonLevel: NETLOGON_LOGON_INFO_CLASS,
    LogonInformation: PNETLOGON_LEVEL,
    ValidationLevel: NETLOGON_VALIDATION_INFO_CLASS,
    ValidationInformation: PNETLOGON_VALIDATION,
    Authoritative: *mut UCHAR,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSamLogoff(
    LogonServer: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    LogonLevel: NETLOGON_LOGON_INFO_CLASS,
    LogonInformation: PNETLOGON_LEVEL,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerReqChallenge(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    ClientChallenge: PNETLOGON_CREDENTIAL,
    ServerChallenge: PNETLOGON_CREDENTIAL,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerAuthenticate(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    ClientCredential: PNETLOGON_CREDENTIAL,
    ServerCredential: PNETLOGON_CREDENTIAL,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerPasswordSet(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    UasNewPassword: PENCRYPTED_NT_OWF_PASSWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrDatabaseDeltas(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    DatabaseID: DWORD,
    DomainModifiedCount: PNLPR_MODIFIED_COUNT,
    DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
    PreferredMaximumLength: DWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrDatabaseSync(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    DatabaseID: DWORD,
    SyncContext: *mut ULONG,
    DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
    PreferredMaximumLength: DWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrAccountDeltas(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    RecordId: PUAS_INFO_0,
    Count: DWORD,
    Level: DWORD,
    Buffer: *mut UCHAR,
    BufferSize: DWORD,
    CountReturned: *mut ULONG,
    TotalEntries: *mut ULONG,
    NextRecordId: PUAS_INFO_0,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrAccountSync(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    Reference: DWORD,
    Level: DWORD,
    Buffer: *mut UCHAR,
    BufferSize: DWORD,
    CountReturned: *mut ULONG,
    TotalEntries: *mut ULONG,
    NextReference: *mut ULONG,
    LastRecordId: PUAS_INFO_0,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetDCName(
    ServerName: LOGONSRV_HANDLE,
    DomainName: *mut wchar_t,
    Buffer: *mut *mut wchar_t,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonControl(
    ServerName: LOGONSRV_HANDLE,
    FunctionCode: DWORD,
    QueryLevel: DWORD,
    Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetAnyDCName(
    ServerName: LOGONSRV_HANDLE,
    DomainName: *mut wchar_t,
    Buffer: *mut *mut wchar_t,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonControl2(
    ServerName: LOGONSRV_HANDLE,
    FunctionCode: DWORD,
    QueryLevel: DWORD,
    Data: PNETLOGON_CONTROL_DATA_INFORMATION,
    Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerAuthenticate2(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    ClientCredential: PNETLOGON_CREDENTIAL,
    ServerCredential: PNETLOGON_CREDENTIAL,
    NegotiateFlags: *mut ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrDatabaseSync2(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    DatabaseID: DWORD,
    RestartState: SYNC_STATE,
    SyncContext: *mut ULONG,
    DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
    PreferredMaximumLength: DWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrDatabaseRedo(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    ChangeLogEntry: *mut UCHAR,
    ChangeLogEntrySize: DWORD,
    DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonControl2Ex(
    ServerName: LOGONSRV_HANDLE,
    FunctionCode: DWORD,
    QueryLevel: DWORD,
    Data: PNETLOGON_CONTROL_DATA_INFORMATION,
    Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrEnumerateTrustedDomains(
    ServerName: LOGONSRV_HANDLE,
    DomainNameBuffer: PDOMAIN_NAME_BUFFER,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetDcName(
    ComputerName: LOGONSRV_HANDLE,
    DomainName: *mut wchar_t,
    DomainGuid: *mut GUID,
    SiteGuid: *mut GUID,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonGetCapabilities(
    ServerName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    QueryLevel: DWORD,
    ServerCapabilities: PNETLOGON_CAPABILITIES,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSetServiceBits(
    ServerName: LOGONSRV_HANDLE,
    ServiceBitsOfInterest: DWORD,
    ServiceBits: DWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonGetTrustRid(
    ServerName: LOGONSRV_HANDLE,
    DomainName: *mut wchar_t,
    Rid: *mut ULONG,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonComputeServerDigest(
    ServerName: LOGONSRV_HANDLE,
    Rid: ULONG,
    Message: *mut UCHAR,
    MessageSize: ULONG,
    NewMessageDigest: *mut CHAR,
    OldMessageDigest: *mut CHAR,
) -> NET_API_STATUS {
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonComputeClientDigest(
    mut ServerName: LOGONSRV_HANDLE,
    mut DomainName: *mut wchar_t,
    mut Message: *mut UCHAR,
    mut MessageSize: ULONG,
    mut NewMessageDigest: *mut CHAR,
    mut OldMessageDigest: *mut CHAR,
) -> NET_API_STATUS {
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerAuthenticate3(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    ClientCredential: PNETLOGON_CREDENTIAL,
    ServerCredential: PNETLOGON_CREDENTIAL,
    NegotiateFlags: *mut ULONG,
    AccountRid: *mut ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetDcNameEx(
    ComputerName: LOGONSRV_HANDLE,
    DomainName: *mut wchar_t,
    DomainGuid: *mut GUID,
    SiteName: *mut wchar_t,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetSiteName(
    ComputerName: LOGONSRV_HANDLE,
    SiteName: *mut *mut wchar_t,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonGetDomainInfo(
    ServerName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    Level: DWORD,
    WkstaBuffer: PNETLOGON_WORKSTATION_INFORMATION,
    DomBuffer: PNETLOGON_DOMAIN_INFORMATION,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerPasswordSet2(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    ClearNewPassword: PNL_TRUST_PASSWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerPasswordGet(
    PrimaryName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    AccountType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    EncryptedNtOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSendToSam(
    PrimaryName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    OpaqueBuffer: *mut UCHAR,
    OpaqueBufferSize: ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrAddressToSiteNamesW(
    ComputerName: LOGONSRV_HANDLE,
    EntryCount: DWORD,
    SocketAddresses: PNL_SOCKET_ADDRESS,
    SiteNames: *mut PNL_SITE_NAME_ARRAY,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetDcNameEx2(
    ComputerName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    AllowableAccountControlBits: ULONG,
    DomainName: *mut wchar_t,
    DomainGuid: *mut GUID,
    SiteName: *mut wchar_t,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
) -> NET_API_STATUS {
    UNIMPLEMENTED!();
    return 2453 as NET_API_STATUS; // NERR_DCNotFound
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonGetTimeServiceParentDomain(
    ServerName: LOGONSRV_HANDLE,
    DomainName: *mut *mut wchar_t,
    PdcSameSite: *mut i32,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrEnumerateTrustedDomainsEx(
    ServerName: LOGONSRV_HANDLE,
    Domains: PNETLOGON_TRUSTED_DOMAIN_ARRAY,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrAddressToSiteNamesExW(
    ComputerName: LOGONSRV_HANDLE,
    EntryCount: DWORD,
    SocketAddresses: PNL_SOCKET_ADDRESS,
    SiteNames: *mut PNL_SITE_NAME_EX_ARRAY,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetDcSiteCoverageW(
    ServerName: LOGONSRV_HANDLE,
    SiteNames: *mut PNL_SITE_NAME_ARRAY,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSamLogonEx(
    ContextHandle: handle_t,
    LogonServer: *mut wchar_t,
    ComputerName: *mut wchar_t,
    LogonLevel: NETLOGON_LOGON_INFO_CLASS,
    LogonInformation: PNETLOGON_LEVEL,
    ValidationLevel: NETLOGON_VALIDATION_INFO_CLASS,
    ValidationInformation: PNETLOGON_VALIDATION,
    Authoritative: *mut UCHAR,
    ExtraFlags: *mut ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrEnumerateDomainTrusts(
    ServerName: LOGONSRV_HANDLE,
    Flags: ULONG,
    Domains: PNETLOGON_TRUSTED_DOMAIN_ARRAY,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrDeregisterDnsHostRecords(
    ServerName: LOGONSRV_HANDLE,
    DnsDomainName: *mut wchar_t,
    DomainGuid: *mut GUID,
    DsaGuid: *mut GUID,
    DnsHostName: *mut wchar_t,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTrustPasswordsGet(
    TrustedDcName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    EncryptedNewOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    EncryptedOldOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DsrGetForestTrustInformation(
    ServerName: LOGONSRV_HANDLE,
    TrustedDomainName: *mut wchar_t,
    Flags: DWORD,
    ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetForestTrustInformation(
    ServerName: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    Flags: DWORD,
    ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonSamLogonWithFlags(
    LogonServer: LOGONSRV_HANDLE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    LogonLevel: NETLOGON_LOGON_INFO_CLASS,
    LogonInformation: PNETLOGON_LEVEL,
    ValidationLevel: NETLOGON_VALIDATION_INFO_CLASS,
    ValidationInformation: PNETLOGON_VALIDATION,
    Authoritative: *mut UCHAR,
    ExtraFlags: *mut ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerGetTrustInfo(
    TrustedDcName: LOGONSRV_HANDLE,
    AccountName: *mut wchar_t,
    SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
    ComputerName: *mut wchar_t,
    Authenticator: PNETLOGON_AUTHENTICATOR,
    ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
    EncryptedNewOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    EncryptedOldOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    TrustInfo: *mut PNL_GENERIC_RPC_DATA,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    0
}
