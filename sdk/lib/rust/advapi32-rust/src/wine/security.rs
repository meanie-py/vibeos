use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn LocalAlloc(uFlags: u32, uBytes: usize) -> LPVOID;
    fn LocalFree(hMem: LPVOID) -> LPVOID;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCSTR, lpUsedDefaultChar: LPBOOL, ) -> i32;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32, ) -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _ACEFLAG {
    pub wstr: LPCWSTR,
    pub value: DWORD,
}
pub type ACEFLAG = _ACEFLAG;
pub type LPACEFLAG = *mut _ACEFLAG;
// The Ace* lookup tables are immutable file-scope `static`s of [_ACEFLAG; N];
// _ACEFLAG holds an LPCWSTR (*const u16) so it isn't auto-Sync. The tables are
// read-only constant data → Sync is sound (matches the C `static const` arrays).
unsafe impl Sync for _ACEFLAG {}

#[repr(C)]
pub struct _MAX_SID {
    pub Revision: BYTE,
    pub SubAuthorityCount: BYTE,
    pub IdentifierAuthority: SID_IDENTIFIER_AUTHORITY,
    pub SubAuthority: [DWORD; SID_MAX_SUB_AUTHORITIES as usize],
}
pub type MAX_SID = _MAX_SID;

#[repr(C)]
pub struct WELLKNOWNSID {
    pub wstr: [WCHAR; 2],
    pub Type: WELL_KNOWN_SID_TYPE,
    pub Sid: MAX_SID,
}

pub static WellKnownSids: [WELLKNOWNSID; 52] = [
    WELLKNOWNSID { wstr: [0, 0], Type: WinNullSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NULL_SID_AUTHORITY, SubAuthority: [SECURITY_NULL_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['W' as WCHAR, 'D' as WCHAR], Type: WinWorldSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_WORLD_SID_AUTHORITY, SubAuthority: [SECURITY_WORLD_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinLocalSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_LOCAL_SID_AUTHORITY, SubAuthority: [SECURITY_LOCAL_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['C' as WCHAR, 'O' as WCHAR], Type: WinCreatorOwnerSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_CREATOR_SID_AUTHORITY, SubAuthority: [SECURITY_CREATOR_OWNER_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['C' as WCHAR, 'G' as WCHAR], Type: WinCreatorGroupSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_CREATOR_SID_AUTHORITY, SubAuthority: [SECURITY_CREATOR_GROUP_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinCreatorOwnerServerSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_CREATOR_SID_AUTHORITY, SubAuthority: [SECURITY_CREATOR_OWNER_SERVER_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinCreatorGroupServerSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_CREATOR_SID_AUTHORITY, SubAuthority: [SECURITY_CREATOR_GROUP_SERVER_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinNtAuthoritySid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 0, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_NULL_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinDialupSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_DIALUP_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['N' as WCHAR, 'U' as WCHAR], Type: WinNetworkSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_NETWORK_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBatchSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BATCH_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['I' as WCHAR, 'U' as WCHAR], Type: WinInteractiveSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_INTERACTIVE_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['S' as WCHAR, 'U' as WCHAR], Type: WinServiceSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_SERVICE_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['A' as WCHAR, 'N' as WCHAR], Type: WinAnonymousSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_ANONYMOUS_LOGON_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinProxySid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_PROXY_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['E' as WCHAR, 'D' as WCHAR], Type: WinEnterpriseControllersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_ENTERPRISE_CONTROLLERS_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['P' as WCHAR, 'S' as WCHAR], Type: WinSelfSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_PRINCIPAL_SELF_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['A' as WCHAR, 'U' as WCHAR], Type: WinAuthenticatedUserSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_AUTHENTICATED_USER_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['R' as WCHAR, 'C' as WCHAR], Type: WinRestrictedCodeSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_RESTRICTED_CODE_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinTerminalServerSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_TERMINAL_SERVER_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinRemoteLogonIdSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_REMOTE_LOGON_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['S' as WCHAR, 'Y' as WCHAR], Type: WinLocalSystemSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_LOCAL_SYSTEM_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['L' as WCHAR, 'S' as WCHAR], Type: WinLocalServiceSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_LOCAL_SERVICE_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['N' as WCHAR, 'S' as WCHAR], Type: WinNetworkServiceSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_NETWORK_SERVICE_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinDomainSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['B' as WCHAR, 'A' as WCHAR], Type: WinBuiltinAdministratorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['B' as WCHAR, 'U' as WCHAR], Type: WinBuiltinUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['B' as WCHAR, 'G' as WCHAR], Type: WinBuiltinGuestsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_GUESTS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['P' as WCHAR, 'U' as WCHAR], Type: WinBuiltinPowerUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_POWER_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['A' as WCHAR, 'O' as WCHAR], Type: WinBuiltinAccountOperatorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ACCOUNT_OPS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['S' as WCHAR, 'O' as WCHAR], Type: WinBuiltinSystemOperatorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_SYSTEM_OPS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['P' as WCHAR, 'O' as WCHAR], Type: WinBuiltinPrintOperatorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_PRINT_OPS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['B' as WCHAR, 'O' as WCHAR], Type: WinBuiltinBackupOperatorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_BACKUP_OPS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['R' as WCHAR, 'E' as WCHAR], Type: WinBuiltinReplicatorSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_REPLICATOR, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['R' as WCHAR, 'U' as WCHAR], Type: WinBuiltinPreWindows2000CompatibleAccessSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_PREW2KCOMPACCESS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['R' as WCHAR, 'D' as WCHAR], Type: WinBuiltinRemoteDesktopUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['N' as WCHAR, 'O' as WCHAR], Type: WinBuiltinNetworkConfigurationOperatorsSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinNTLMAuthenticationSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_NTLM_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinDigestAuthenticationSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_DIGEST_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinSChannelAuthenticationSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_SCHANNEL_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinThisOrganizationSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_THIS_ORGANIZATION_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinOtherOrganizationSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_OTHER_ORGANIZATION_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinIncomingForestTrustBuildersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_INCOMING_FOREST_TRUST_BUILDERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinPerfMonitoringUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_MONITORING_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinPerfLoggingUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_LOGGING_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinAuthorizationAccessSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_AUTHORIZATIONACCESS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinTerminalServerLicenseServersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_TS_LICENSE_SERVERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: [0, 0], Type: WinBuiltinDCOMUsersSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 2, IdentifierAuthority: SECURITY_NT_AUTHORITY, SubAuthority: [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_DCOM_USERS, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['L' as WCHAR, 'W' as WCHAR], Type: WinLowLabelSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_MANDATORY_LABEL_AUTHORITY, SubAuthority: [SECURITY_MANDATORY_LOW_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['M' as WCHAR, 'E' as WCHAR], Type: WinMediumLabelSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_MANDATORY_LABEL_AUTHORITY, SubAuthority: [SECURITY_MANDATORY_MEDIUM_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['H' as WCHAR, 'I' as WCHAR], Type: WinHighLabelSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_MANDATORY_LABEL_AUTHORITY, SubAuthority: [SECURITY_MANDATORY_HIGH_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
    WELLKNOWNSID { wstr: ['S' as WCHAR, 'I' as WCHAR], Type: WinSystemLabelSid, Sid: MAX_SID { Revision: SID_REVISION as BYTE, SubAuthorityCount: 1, IdentifierAuthority: SECURITY_MANDATORY_LABEL_AUTHORITY, SubAuthority: [SECURITY_MANDATORY_SYSTEM_RID, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] } },
];

#[repr(C)]
pub struct WELLKNOWNRID {
    pub wstr: [WCHAR; 2],
    pub Type: WELL_KNOWN_SID_TYPE,
    pub Rid: DWORD,
}

pub static WellKnownRids: [WELLKNOWNRID; 13] = [
    WELLKNOWNRID { wstr: ['L' as WCHAR, 'A' as WCHAR], Type: WinAccountAdministratorSid, Rid: DOMAIN_USER_RID_ADMIN },
    WELLKNOWNRID { wstr: ['L' as WCHAR, 'G' as WCHAR], Type: WinAccountGuestSid, Rid: DOMAIN_USER_RID_GUEST },
    WELLKNOWNRID { wstr: [0, 0], Type: WinAccountKrbtgtSid, Rid: DOMAIN_USER_RID_KRBTGT },
    WELLKNOWNRID { wstr: ['D' as WCHAR, 'A' as WCHAR], Type: WinAccountDomainAdminsSid, Rid: DOMAIN_GROUP_RID_ADMINS },
    WELLKNOWNRID { wstr: ['D' as WCHAR, 'U' as WCHAR], Type: WinAccountDomainUsersSid, Rid: DOMAIN_GROUP_RID_USERS },
    WELLKNOWNRID { wstr: ['D' as WCHAR, 'G' as WCHAR], Type: WinAccountDomainGuestsSid, Rid: DOMAIN_GROUP_RID_GUESTS },
    WELLKNOWNRID { wstr: ['D' as WCHAR, 'C' as WCHAR], Type: WinAccountComputersSid, Rid: DOMAIN_GROUP_RID_COMPUTERS },
    WELLKNOWNRID { wstr: ['D' as WCHAR, 'D' as WCHAR], Type: WinAccountControllersSid, Rid: DOMAIN_GROUP_RID_CONTROLLERS },
    WELLKNOWNRID { wstr: ['C' as WCHAR, 'A' as WCHAR], Type: WinAccountCertAdminsSid, Rid: DOMAIN_GROUP_RID_CERT_ADMINS },
    WELLKNOWNRID { wstr: ['S' as WCHAR, 'A' as WCHAR], Type: WinAccountSchemaAdminsSid, Rid: DOMAIN_GROUP_RID_SCHEMA_ADMINS },
    WELLKNOWNRID { wstr: ['E' as WCHAR, 'A' as WCHAR], Type: WinAccountEnterpriseAdminsSid, Rid: DOMAIN_GROUP_RID_ENTERPRISE_ADMINS },
    WELLKNOWNRID { wstr: ['P' as WCHAR, 'A' as WCHAR], Type: WinAccountPolicyAdminsSid, Rid: DOMAIN_GROUP_RID_POLICY_ADMINS },
    WELLKNOWNRID { wstr: ['R' as WCHAR, 'S' as WCHAR], Type: WinAccountRasAndIasServersSid, Rid: DOMAIN_ALIAS_RID_RAS_SERVERS },
];

pub static SDDL_NO_READ_UP: [WCHAR; 3] = ['N' as WCHAR, 'R' as WCHAR, 0];
pub static SDDL_NO_WRITE_UP: [WCHAR; 3] = ['N' as WCHAR, 'W' as WCHAR, 0];
pub static SDDL_NO_EXECUTE_UP: [WCHAR; 3] = ['N' as WCHAR, 'X' as WCHAR, 0];
pub static SDDL_ACCESS_ALLOWED: [WCHAR; 2] = ['A' as WCHAR, 0];
pub static SDDL_ACCESS_DENIED: [WCHAR; 2] = ['D' as WCHAR, 0];
pub static SDDL_AUDIT: [WCHAR; 3] = ['A' as WCHAR, 'U' as WCHAR, 0];
pub static SDDL_ALARM: [WCHAR; 3] = ['A' as WCHAR, 'L' as WCHAR, 0];
pub static SDDL_MANDATORY_LABEL: [WCHAR; 3] = ['M' as WCHAR, 'L' as WCHAR, 0];

pub const ADS_RIGHT_DS_CREATE_CHILD: DWORD = 0x0001;
pub const ADS_RIGHT_DS_DELETE_CHILD: DWORD = 0x0002;
pub const ADS_RIGHT_ACTRL_DS_LIST: DWORD = 0x0004;
pub const ADS_RIGHT_DS_SELF: DWORD = 0x0008;
pub const ADS_RIGHT_DS_READ_PROP: DWORD = 0x0010;
pub const ADS_RIGHT_DS_WRITE_PROP: DWORD = 0x0020;
pub const ADS_RIGHT_DS_DELETE_TREE: DWORD = 0x0040;
pub const ADS_RIGHT_DS_LIST_OBJECT: DWORD = 0x0080;
pub const ADS_RIGHT_DS_CONTROL_ACCESS: DWORD = 0x0100;

pub static SDDL_CONTAINER_INHERIT: [WCHAR; 3] = ['C' as WCHAR, 'I' as WCHAR, 0];
pub static SDDL_OBJECT_INHERIT: [WCHAR; 3] = ['O' as WCHAR, 'I' as WCHAR, 0];
pub static SDDL_NO_PROPAGATE: [WCHAR; 3] = ['N' as WCHAR, 'P' as WCHAR, 0];
pub static SDDL_INHERIT_ONLY: [WCHAR; 3] = ['I' as WCHAR, 'O' as WCHAR, 0];
pub static SDDL_INHERITED: [WCHAR; 3] = ['I' as WCHAR, 'D' as WCHAR, 0];
pub static SDDL_AUDIT_SUCCESS: [WCHAR; 3] = ['S' as WCHAR, 'A' as WCHAR, 0];
pub static SDDL_AUDIT_FAILURE: [WCHAR; 3] = ['F' as WCHAR, 'A' as WCHAR, 0];

pub static AceType: [ACEFLAG; 6] = [
    ACEFLAG { wstr: SDDL_ALARM.as_ptr(), value: SYSTEM_ALARM_ACE_TYPE as DWORD },
    ACEFLAG { wstr: SDDL_AUDIT.as_ptr(), value: SYSTEM_AUDIT_ACE_TYPE as DWORD },
    ACEFLAG { wstr: SDDL_ACCESS_ALLOWED.as_ptr(), value: ACCESS_ALLOWED_ACE_TYPE as DWORD },
    ACEFLAG { wstr: SDDL_ACCESS_DENIED.as_ptr(), value: ACCESS_DENIED_ACE_TYPE as DWORD },
    ACEFLAG { wstr: SDDL_MANDATORY_LABEL.as_ptr(), value: SYSTEM_MANDATORY_LABEL_ACE_TYPE as DWORD },
    ACEFLAG { wstr: core::ptr::null(), value: 0 },
];

pub static AceFlags: [ACEFLAG; 8] = [
    ACEFLAG { wstr: SDDL_CONTAINER_INHERIT.as_ptr(), value: CONTAINER_INHERIT_ACE as DWORD },
    ACEFLAG { wstr: SDDL_AUDIT_FAILURE.as_ptr(), value: FAILED_ACCESS_ACE_FLAG as DWORD },
    ACEFLAG { wstr: SDDL_INHERITED.as_ptr(), value: INHERITED_ACE as DWORD },
    ACEFLAG { wstr: SDDL_INHERIT_ONLY.as_ptr(), value: INHERIT_ONLY_ACE as DWORD },
    ACEFLAG { wstr: SDDL_NO_PROPAGATE.as_ptr(), value: NO_PROPAGATE_INHERIT_ACE as DWORD },
    ACEFLAG { wstr: SDDL_OBJECT_INHERIT.as_ptr(), value: OBJECT_INHERIT_ACE as DWORD },
    ACEFLAG { wstr: SDDL_AUDIT_SUCCESS.as_ptr(), value: SUCCESSFUL_ACCESS_ACE_FLAG as DWORD },
    ACEFLAG { wstr: core::ptr::null(), value: 0 },
];

pub static AceRights: [ACEFLAG; 29] = [
    ACEFLAG { wstr: SDDL_GENERIC_ALL.as_ptr(), value: GENERIC_ALL as DWORD },
    ACEFLAG { wstr: SDDL_GENERIC_READ.as_ptr(), value: GENERIC_READ as DWORD },
    ACEFLAG { wstr: SDDL_GENERIC_WRITE.as_ptr(), value: GENERIC_WRITE as DWORD },
    ACEFLAG { wstr: SDDL_GENERIC_EXECUTE.as_ptr(), value: GENERIC_EXECUTE as DWORD },
    ACEFLAG { wstr: SDDL_READ_CONTROL.as_ptr(), value: READ_CONTROL as DWORD },
    ACEFLAG { wstr: SDDL_STANDARD_DELETE.as_ptr(), value: DELETE as DWORD },
    ACEFLAG { wstr: SDDL_WRITE_DAC.as_ptr(), value: WRITE_DAC as DWORD },
    ACEFLAG { wstr: SDDL_WRITE_OWNER.as_ptr(), value: WRITE_OWNER as DWORD },
    ACEFLAG { wstr: SDDL_READ_PROPERTY.as_ptr(), value: ADS_RIGHT_DS_READ_PROP },
    ACEFLAG { wstr: SDDL_WRITE_PROPERTY.as_ptr(), value: ADS_RIGHT_DS_WRITE_PROP },
    ACEFLAG { wstr: SDDL_CREATE_CHILD.as_ptr(), value: ADS_RIGHT_DS_CREATE_CHILD },
    ACEFLAG { wstr: SDDL_DELETE_CHILD.as_ptr(), value: ADS_RIGHT_DS_DELETE_CHILD },
    ACEFLAG { wstr: SDDL_LIST_CHILDREN.as_ptr(), value: ADS_RIGHT_ACTRL_DS_LIST },
    ACEFLAG { wstr: SDDL_SELF_WRITE.as_ptr(), value: ADS_RIGHT_DS_SELF },
    ACEFLAG { wstr: SDDL_LIST_OBJECT.as_ptr(), value: ADS_RIGHT_DS_LIST_OBJECT },
    ACEFLAG { wstr: SDDL_DELETE_TREE.as_ptr(), value: ADS_RIGHT_DS_DELETE_TREE },
    ACEFLAG { wstr: SDDL_CONTROL_ACCESS.as_ptr(), value: ADS_RIGHT_DS_CONTROL_ACCESS },
    ACEFLAG { wstr: SDDL_FILE_ALL.as_ptr(), value: FILE_ALL_ACCESS as DWORD },
    ACEFLAG { wstr: SDDL_FILE_READ.as_ptr(), value: FILE_GENERIC_READ as DWORD },
    ACEFLAG { wstr: SDDL_FILE_WRITE.as_ptr(), value: FILE_GENERIC_WRITE as DWORD },
    ACEFLAG { wstr: SDDL_FILE_EXECUTE.as_ptr(), value: FILE_GENERIC_EXECUTE as DWORD },
    ACEFLAG { wstr: SDDL_KEY_ALL.as_ptr(), value: KEY_ALL_ACCESS as DWORD },
    ACEFLAG { wstr: SDDL_KEY_READ.as_ptr(), value: KEY_READ as DWORD },
    ACEFLAG { wstr: SDDL_KEY_WRITE.as_ptr(), value: KEY_WRITE as DWORD },
    ACEFLAG { wstr: SDDL_KEY_EXECUTE.as_ptr(), value: KEY_EXECUTE as DWORD },
    ACEFLAG { wstr: SDDL_NO_READ_UP.as_ptr(), value: SYSTEM_MANDATORY_LABEL_NO_READ_UP as DWORD },
    ACEFLAG { wstr: SDDL_NO_WRITE_UP.as_ptr(), value: SYSTEM_MANDATORY_LABEL_NO_WRITE_UP as DWORD },
    ACEFLAG { wstr: SDDL_NO_EXECUTE_UP.as_ptr(), value: SYSTEM_MANDATORY_LABEL_NO_EXECUTE_UP as DWORD },
    ACEFLAG { wstr: core::ptr::null(), value: 0 },
];

pub static AceRightBitNames: SyncConst<[LPCWSTR; 32]> = SyncConst([
    SDDL_CREATE_CHILD.as_ptr() as LPCWSTR,
    SDDL_DELETE_CHILD.as_ptr() as LPCWSTR,
    SDDL_LIST_CHILDREN.as_ptr() as LPCWSTR,
    SDDL_SELF_WRITE.as_ptr() as LPCWSTR,
    SDDL_READ_PROPERTY.as_ptr() as LPCWSTR,
    SDDL_WRITE_PROPERTY.as_ptr() as LPCWSTR,
    SDDL_DELETE_TREE.as_ptr() as LPCWSTR,
    SDDL_LIST_OBJECT.as_ptr() as LPCWSTR,
    SDDL_CONTROL_ACCESS.as_ptr() as LPCWSTR,
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    SDDL_STANDARD_DELETE.as_ptr() as LPCWSTR,
    SDDL_READ_CONTROL.as_ptr() as LPCWSTR,
    SDDL_WRITE_DAC.as_ptr() as LPCWSTR,
    SDDL_WRITE_OWNER.as_ptr() as LPCWSTR,
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    SDDL_GENERIC_ALL.as_ptr() as LPCWSTR,
    SDDL_GENERIC_EXECUTE.as_ptr() as LPCWSTR,
    SDDL_GENERIC_WRITE.as_ptr() as LPCWSTR,
    SDDL_GENERIC_READ.as_ptr() as LPCWSTR,
]);

unsafe fn debugstr_sid(sid: PSID) -> *const core::ffi::c_char {
    let psid = sid as *mut SID;

    if psid.is_null() {
        return b"(null)\0".as_ptr() as *const core::ffi::c_char;
    }

    let auth: core::ffi::c_int = (*psid).IdentifierAuthority.Value[5] as core::ffi::c_int
        + (((*psid).IdentifierAuthority.Value[4] as core::ffi::c_int) << 8)
        + (((*psid).IdentifierAuthority.Value[3] as core::ffi::c_int) << 16)
        + (((*psid).IdentifierAuthority.Value[2] as core::ffi::c_int) << 24);

    let sub = core::ptr::addr_of!((*psid).SubAuthority) as *const u32;

    match (*psid).SubAuthorityCount {
        0 => wine_dbg_sprintf(
            b"S-%d-%d\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
        ),
        1 => wine_dbg_sprintf(
            b"S-%d-%d-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
        ),
        2 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
        ),
        3 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
            *sub.offset(2),
        ),
        4 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
            *sub.offset(2),
            *sub.offset(3),
        ),
        5 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
            *sub.offset(2),
            *sub.offset(3),
            *sub.offset(4),
        ),
        6 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(3),
            *sub.offset(1),
            *sub.offset(2),
            *sub.offset(0),
            *sub.offset(4),
            *sub.offset(5),
        ),
        7 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
            *sub.offset(2),
            *sub.offset(3),
            *sub.offset(4),
            *sub.offset(5),
            *sub.offset(6),
        ),
        8 => wine_dbg_sprintf(
            b"S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu-%lu-%lu\0".as_ptr() as *const core::ffi::c_char,
            (*psid).Revision as core::ffi::c_int,
            auth,
            *sub.offset(0),
            *sub.offset(1),
            *sub.offset(2),
            *sub.offset(3),
            *sub.offset(4),
            *sub.offset(5),
            *sub.offset(6),
            *sub.offset(7),
        ),
        _ => b"(too-big)\0".as_ptr() as *const core::ffi::c_char,
    }
}

unsafe fn set_ntstatus(status: NTSTATUS) -> BOOL {
    if !NT_SUCCESS(status) {
        SetLastError(RtlNtStatusToDosError(status));
    }
    NT_SUCCESS(status) as BOOL
}

unsafe fn SERV_dup(str_: LPCSTR) -> LPWSTR {
    let len: UINT;
    let wstr: LPWSTR;

    if str_.is_null() {
        return core::ptr::null_mut();
    }
    len = MultiByteToWideChar(CP_ACP, 0, str_, -1, core::ptr::null_mut(), 0) as UINT;
    wstr = heap_alloc((len as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
    MultiByteToWideChar(CP_ACP, 0, str_, -1, wstr, len as i32);
    wstr
}
#[no_mangle]
pub unsafe extern "C" fn ADVAPI_IsLocalComputer(mut ServerName: LPCWSTR) -> BOOL {
    let mut dwSize: DWORD = MAX_COMPUTERNAME_LENGTH + 1;
    let mut Result: BOOL;
    let buf: LPWSTR;

    if ServerName.is_null() || *ServerName == 0 {
        return 1;
    }

    buf = heap_alloc((dwSize as usize) * core::mem::size_of::<u16>()) as LPWSTR;
    Result = GetComputerNameW(buf, &mut dwSize);
    if Result != 0 && *ServerName == b'\\' as u16 && *ServerName.add(1) == b'\\' as u16 {
        ServerName = ServerName.add(2);
    }
    Result = if Result != 0 && lstrcmpW(ServerName, buf) == 0 { 1 } else { 0 };
    heap_free(buf as LPVOID);

    Result
}

#[no_mangle]
pub unsafe extern "C" fn ADVAPI_GetComputerSid(sid: PSID) -> BOOL {
    #[repr(C)]
    struct ComputerSid {
        Revision: BYTE,
        SubAuthorityCount: BYTE,
        IdentifierAuthority: SID_IDENTIFIER_AUTHORITY,
        SubAuthority: [DWORD; 4],
    }

    let computer_sid = ComputerSid {
        Revision: SID_REVISION as BYTE,
        SubAuthorityCount: 4,
        IdentifierAuthority: SID_IDENTIFIER_AUTHORITY {
            Value: [0, 0, 0, 0, 0, 5],
        },
        SubAuthority: [SECURITY_NT_NON_UNIQUE as DWORD, 0, 0, 0],
    };

    core::ptr::copy_nonoverlapping(
        &computer_sid as *const ComputerSid as *const u8,
        sid as *mut u8,
        core::mem::size_of::<ComputerSid>(),
    );
    1
}

#[no_mangle]
pub unsafe extern "system" fn OpenProcessToken(
    ProcessHandle: HANDLE,
    DesiredAccess: DWORD,
    TokenHandle: PHANDLE,
) -> BOOL {
    let Status: NTSTATUS = NtOpenProcessToken(ProcessHandle, DesiredAccess, TokenHandle);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn OpenThreadToken(
    ThreadHandle: HANDLE,
    DesiredAccess: DWORD,
    OpenAsSelf: BOOL,
    TokenHandle: *mut HANDLE,
) -> BOOL {
    set_ntstatus(NtOpenThreadToken(
        ThreadHandle,
        DesiredAccess,
        OpenAsSelf as u8,
        TokenHandle,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn AdjustTokenGroups(
    TokenHandle: HANDLE,
    ResetToDefault: BOOL,
    NewState: PTOKEN_GROUPS,
    BufferLength: DWORD,
    PreviousState: PTOKEN_GROUPS,
    ReturnLength: PDWORD,
) -> BOOL {
    let Status: NTSTATUS = NtAdjustGroupsToken(
        TokenHandle,
        ResetToDefault as u8,
        NewState,
        BufferLength,
        PreviousState,
        ReturnLength as PULONG,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AdjustTokenPrivileges(
    TokenHandle: HANDLE,
    DisableAllPrivileges: BOOL,
    NewState: PTOKEN_PRIVILEGES,
    BufferLength: DWORD,
    PreviousState: PTOKEN_PRIVILEGES,
    ReturnLength: PDWORD,
) -> BOOL {
    let Status: NTSTATUS;

    Status = NtAdjustPrivilegesToken(
        TokenHandle,
        DisableAllPrivileges as u8,
        NewState,
        BufferLength,
        PreviousState,
        ReturnLength as PULONG,
    );

    if STATUS_NOT_ALL_ASSIGNED == Status {
        SetLastError(ERROR_NOT_ALL_ASSIGNED);
        return 1;
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    SetLastError(ERROR_SUCCESS);

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetTokenInformation(
    TokenHandle: HANDLE,
    TokenInformationClass: TOKEN_INFORMATION_CLASS,
    TokenInformation: LPVOID,
    TokenInformationLength: DWORD,
    ReturnLength: PDWORD,
) -> BOOL {
    let Status: NTSTATUS = NtQueryInformationToken(
        TokenHandle,
        TokenInformationClass,
        TokenInformation,
        TokenInformationLength,
        ReturnLength as PULONG,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetTokenInformation(
    TokenHandle: HANDLE,
    TokenInformationClass: TOKEN_INFORMATION_CLASS,
    TokenInformation: LPVOID,
    TokenInformationLength: DWORD,
) -> BOOL {
    let Status = NtSetInformationToken(
        TokenHandle,
        TokenInformationClass,
        TokenInformation,
        TokenInformationLength,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadToken(
    ThreadHandle: PHANDLE,
    TokenHandle: HANDLE,
) -> BOOL {
    let mut TokenHandle = TokenHandle;

    let hThread: HANDLE = if !ThreadHandle.is_null() {
        *ThreadHandle
    } else {
        (-2isize) as HANDLE
    };

    let Status = NtSetInformationThread(
        hThread,
        ThreadImpersonationToken,
        core::ptr::addr_of_mut!(TokenHandle) as PVOID,
        core::mem::size_of::<HANDLE>() as u32,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CreateRestrictedToken(
    ExistingTokenHandle: HANDLE,
    Flags: DWORD,
    DisableSidCount: DWORD,
    SidsToDisable: PSID_AND_ATTRIBUTES,
    DeletePrivilegeCount: DWORD,
    PrivilegesToDelete: PLUID_AND_ATTRIBUTES,
    RestrictedSidCount: DWORD,
    SidsToRestrict: PSID_AND_ATTRIBUTES,
    NewTokenHandle: PHANDLE,
) -> BOOL {
    let Status: NTSTATUS;
    let mut Success: BOOL;
    let mut Index: ULONG;
    let mut DisableSids: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut RestrictedSids: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut DeletePrivileges: PTOKEN_PRIVILEGES = core::ptr::null_mut();

    'cleanup: {
        if SidsToDisable != core::ptr::null_mut() {
            DisableSids = LocalAlloc(
                LMEM_FIXED,
                (DisableSidCount as usize) * core::mem::size_of::<TOKEN_GROUPS>(),
            ) as PTOKEN_GROUPS;
            if DisableSids == core::ptr::null_mut() {
                SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
                return 0;
            }

            (*DisableSids).GroupCount = DisableSidCount;
            Index = 0;
            while Index < DisableSidCount {
                (*(&raw mut (*DisableSids).Groups[0]).add(Index as usize)).Sid = (*SidsToDisable.add(Index as usize)).Sid;
                (*(&raw mut (*DisableSids).Groups[0]).add(Index as usize)).Attributes =
                    (*SidsToDisable.add(Index as usize)).Attributes;
                Index += 1;
            }
        }

        if PrivilegesToDelete != core::ptr::null_mut() {
            DeletePrivileges = LocalAlloc(
                LMEM_FIXED,
                (DeletePrivilegeCount as usize) * core::mem::size_of::<TOKEN_PRIVILEGES>(),
            ) as PTOKEN_PRIVILEGES;
            if DeletePrivileges == core::ptr::null_mut() {
                SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
                Success = 0;
                break 'cleanup;
            }

            (*DeletePrivileges).PrivilegeCount = DeletePrivilegeCount;
            Index = 0;
            while Index < DeletePrivilegeCount {
                (*(&raw mut (*DeletePrivileges).Privileges[0]).add(Index as usize)).Luid =
                    (*PrivilegesToDelete.add(Index as usize)).Luid;
                (*(&raw mut (*DeletePrivileges).Privileges[0]).add(Index as usize)).Attributes =
                    (*PrivilegesToDelete.add(Index as usize)).Attributes;
                Index += 1;
            }
        }

        if SidsToRestrict != core::ptr::null_mut() {
            RestrictedSids = LocalAlloc(
                LMEM_FIXED,
                (RestrictedSidCount as usize) * core::mem::size_of::<TOKEN_GROUPS>(),
            ) as PTOKEN_GROUPS;
            if RestrictedSids == core::ptr::null_mut() {
                SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
                Success = 0;
                break 'cleanup;
            }

            (*RestrictedSids).GroupCount = RestrictedSidCount;
            Index = 0;
            while Index < RestrictedSidCount {
                (*(&raw mut (*RestrictedSids).Groups[0]).add(Index as usize)).Sid =
                    (*SidsToRestrict.add(Index as usize)).Sid;
                (*(&raw mut (*RestrictedSids).Groups[0]).add(Index as usize)).Attributes =
                    (*SidsToRestrict.add(Index as usize)).Attributes;
                Index += 1;
            }
        }

        Status = NtFilterToken(
            ExistingTokenHandle,
            Flags,
            DisableSids,
            DeletePrivileges,
            RestrictedSids,
            NewTokenHandle,
        );
        if !NT_SUCCESS(Status) {
            SetLastError(RtlNtStatusToDosError(Status));
            Success = 0;
            break 'cleanup;
        }

        Success = 1;
    }

    if DisableSids != core::ptr::null_mut() {
        LocalFree(DisableSids as LPVOID);
    }

    if DeletePrivileges != core::ptr::null_mut() {
        LocalFree(DeletePrivileges as LPVOID);
    }

    if RestrictedSids != core::ptr::null_mut() {
        LocalFree(RestrictedSids as LPVOID);
    }

    Success
}

#[no_mangle]
pub unsafe extern "system" fn AllocateAndInitializeSid(
    pIdentifierAuthority: PSID_IDENTIFIER_AUTHORITY,
    nSubAuthorityCount: BYTE,
    nSubAuthority0: DWORD,
    nSubAuthority1: DWORD,
    nSubAuthority2: DWORD,
    nSubAuthority3: DWORD,
    nSubAuthority4: DWORD,
    nSubAuthority5: DWORD,
    nSubAuthority6: DWORD,
    nSubAuthority7: DWORD,
    pSid: *mut PSID,
) -> BOOL {
    set_ntstatus(RtlAllocateAndInitializeSid(
        pIdentifierAuthority,
        nSubAuthorityCount,
        nSubAuthority0,
        nSubAuthority1,
        nSubAuthority2,
        nSubAuthority3,
        nSubAuthority4,
        nSubAuthority5,
        nSubAuthority6,
        nSubAuthority7,
        pSid,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn FreeSid(pSid: PSID) -> PVOID {
    RtlFreeSid(pSid)
}

#[no_mangle]
pub unsafe extern "system" fn CopySid(
    nDestinationSidLength: DWORD,
    pDestinationSid: PSID,
    pSourceSid: PSID,
) -> BOOL {
    set_ntstatus(RtlCopySid(nDestinationSidLength, pDestinationSid, pSourceSid))
}

#[no_mangle]
pub unsafe extern "system" fn CreateWellKnownSid(
    WellKnownSidType: WELL_KNOWN_SID_TYPE,
    DomainSid: PSID,
    pSid: PSID,
    cbSid: *mut DWORD,
) -> BOOL {
    let mut i: usize;
    TRACE!(
        b"(%d, %s, %p, %p)\n\0".as_ptr() as *const core::ffi::c_char,
        WellKnownSidType,
        debugstr_sid(DomainSid),
        pSid,
        cbSid,
    );

    if cbSid.is_null() || (!DomainSid.is_null() && IsValidSid(DomainSid) == 0) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let well_known_count = WellKnownSids.len();
    i = 0;
    while i < well_known_count {
        if WellKnownSids[i].Type == WellKnownSidType {
            let length = GetSidLengthRequired(WellKnownSids[i].Sid.SubAuthorityCount);

            if *cbSid < length {
                *cbSid = length;
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
            if pSid.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            CopyMemory(
                pSid,
                core::ptr::addr_of!(WellKnownSids[i].Sid.Revision) as LPVOID,
                length as usize,
            );
            *cbSid = length;
            return 1;
        }
        i += 1;
    }

    if DomainSid.is_null() || *GetSidSubAuthorityCount(DomainSid) as DWORD == SID_MAX_SUB_AUTHORITIES {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let well_known_rids_count = WellKnownRids.len();
    i = 0;
    while i < well_known_rids_count {
        if WellKnownRids[i].Type == WellKnownSidType {
            let domain_subauth = *GetSidSubAuthorityCount(DomainSid);
            let domain_sid_length = GetSidLengthRequired(domain_subauth);
            let output_sid_length = GetSidLengthRequired(domain_subauth + 1);

            if *cbSid < output_sid_length {
                *cbSid = output_sid_length;
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
            if pSid.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            CopyMemory(pSid, DomainSid as LPVOID, domain_sid_length as usize);
            *GetSidSubAuthorityCount(pSid) += 1;
            *GetSidSubAuthority(pSid, domain_subauth as DWORD) = WellKnownRids[i].Rid;
            *cbSid = output_sid_length;
            return 1;
        }
        i += 1;
    }

    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn IsWellKnownSid(pSid: PSID, WellKnownSidType: WELL_KNOWN_SID_TYPE) -> BOOL {
    let count = core::mem::size_of_val(&WellKnownSids) / core::mem::size_of::<WELLKNOWNSID>();
    let mut i: usize = 0;
    while i < count {
        if WellKnownSids[i].Type == WellKnownSidType {
            if EqualSid(pSid, core::ptr::addr_of!(WellKnownSids[i].Sid.Revision) as PSID) != 0 {
                return 1;
            }
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn IsValidSid(pSid: PSID) -> BOOL {
    RtlValidSid(pSid) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EqualSid(pSid1: PSID, pSid2: PSID) -> BOOL {
    SetLastError(ERROR_SUCCESS);
    RtlEqualSid(pSid1, pSid2) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EqualPrefixSid(pSid1: PSID, pSid2: PSID) -> BOOL {
    RtlEqualPrefixSid(pSid1, pSid2) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetSidLengthRequired(nSubAuthorityCount: UCHAR) -> DWORD {
    RtlLengthRequiredSid(nSubAuthorityCount as u32) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn InitializeSid(
    Sid: PSID,
    pIdentifierAuthority: PSID_IDENTIFIER_AUTHORITY,
    nSubAuthorityCount: BYTE,
) -> BOOL {
    let Status: NTSTATUS = RtlInitializeSid(Sid, pIdentifierAuthority, nSubAuthorityCount);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSidIdentifierAuthority(pSid: PSID) -> PSID_IDENTIFIER_AUTHORITY {
    SetLastError(ERROR_SUCCESS);
    RtlIdentifierAuthoritySid(pSid)
}

#[no_mangle]
pub unsafe extern "system" fn GetSidSubAuthority(pSid: PSID, nSubAuthority: DWORD) -> PDWORD {
    SetLastError(ERROR_SUCCESS);
    RtlSubAuthoritySid(pSid, nSubAuthority) as PDWORD
}

#[no_mangle]
pub unsafe extern "system" fn GetSidSubAuthorityCount(pSid: PSID) -> PUCHAR {
    SetLastError(ERROR_SUCCESS);
    RtlSubAuthorityCountSid(pSid)
}

#[no_mangle]
pub unsafe extern "system" fn GetLengthSid(pSid: PSID) -> DWORD {
    RtlLengthSid(pSid) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn InitializeSecurityDescriptor(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    dwRevision: DWORD,
) -> BOOL {
    let Status: NTSTATUS = RtlCreateSecurityDescriptor(pSecurityDescriptor, dwRevision);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn MakeAbsoluteSD(
    pSelfRelativeSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pAbsoluteSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpdwAbsoluteSecurityDescriptorSize: LPDWORD,
    pDacl: PACL,
    lpdwDaclSize: LPDWORD,
    pSacl: PACL,
    lpdwSaclSize: LPDWORD,
    pOwner: PSID,
    lpdwOwnerSize: LPDWORD,
    pPrimaryGroup: PSID,
    lpdwPrimaryGroupSize: LPDWORD,
) -> BOOL {
    let Status = RtlSelfRelativeToAbsoluteSD(
        pSelfRelativeSecurityDescriptor,
        pAbsoluteSecurityDescriptor,
        lpdwAbsoluteSecurityDescriptorSize,
        pDacl,
        lpdwDaclSize,
        pSacl,
        lpdwSaclSize,
        pOwner,
        lpdwOwnerSize,
        pPrimaryGroup,
        lpdwPrimaryGroupSize,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetKernelObjectSecurity(
    Handle: HANDLE,
    RequestedInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    nLength: DWORD,
    lpnLengthNeeded: LPDWORD,
) -> BOOL {
    set_ntstatus(NtQuerySecurityObject(
        Handle,
        RequestedInformation,
        pSecurityDescriptor,
        nLength,
        lpnLengthNeeded,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn InitializeAcl(
    pAcl: PACL,
    nAclLength: DWORD,
    dwAclRevision: DWORD,
) -> BOOL {
    let Status: NTSTATUS = RtlCreateAcl(pAcl, nAclLength, dwAclRevision);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ImpersonateNamedPipeClient(hNamedPipe: HANDLE) -> BOOL {
    let mut io_block: _IO_STATUS_BLOCK = core::mem::zeroed();

    set_ntstatus(NtFsControlFile(
        hNamedPipe,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io_block,
        FSCTL_PIPE_IMPERSONATE,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn AddAccessAllowedAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    AccessMask: DWORD,
    pSid: PSID,
) -> BOOL {
    let Status = RtlAddAccessAllowedAce(pAcl, dwAceRevision, AccessMask, pSid);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAccessAllowedAceEx(
    pAcl: PACL,
    dwAceRevision: DWORD,
    AceFlags_p: DWORD,
    AccessMask: DWORD,
    pSid: PSID,
) -> BOOL {
    let Status: NTSTATUS = RtlAddAccessAllowedAceEx(
        pAcl,
        dwAceRevision,
        AceFlags_p,
        AccessMask,
        pSid,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAccessDeniedAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    AccessMask: DWORD,
    pSid: PSID,
) -> BOOL {
    let Status: NTSTATUS = RtlAddAccessDeniedAce(pAcl, dwAceRevision, AccessMask, pSid);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAccessDeniedAceEx(
    pAcl: PACL,
    dwAceRevision: DWORD,
    AceFlags_p: DWORD,
    AccessMask: DWORD,
    pSid: PSID,
) -> BOOL {
    let Status = RtlAddAccessDeniedAceEx(pAcl, dwAceRevision, AceFlags_p, AccessMask, pSid);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

// SYSTEM_MANDATORY_LABEL_ACE (winnt.h): header(4) + mask(4) + SID. The mask holds the
// integrity policy (NO_WRITE_UP=1, NO_READ_UP=2, NO_EXECUTE_UP=4); anything else is rejected.
const SYSTEM_MANDATORY_LABEL_ACE_TYPE: u8 = 0x11;
const SYSTEM_MANDATORY_LABEL_VALID_MASK: DWORD = 0x7;
const SECURITY_MAX_SID_SIZE: usize = 68;

// Wine's kernelbase AddMandatoryAce is RtlAddMandatoryAce(...SYSTEM_MANDATORY_LABEL_ACE_TYPE...).
// Our ntdll exports RtlAddMandatoryAce only as a 0x600+ stub, so the ACE is built here and
// appended with AddAce (which does the ACL-capacity and revision checks), as the ntdll body does.
#[no_mangle]
pub unsafe extern "system" fn AddMandatoryAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    dwAceFlags: DWORD,
    MandatoryPolicy: DWORD,
    pSid: PSID,
) -> BOOL {
    if MandatoryPolicy & !SYSTEM_MANDATORY_LABEL_VALID_MASK != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }
    if RtlValidSid(pSid) == 0 {
        SetLastError(ERROR_INVALID_SID as DWORD);
        return FALSE as BOOL;
    }
    // A valid SID is at most SECURITY_MAX_SID_SIZE bytes, so the buffer below always fits it.
    let sid_len = RtlLengthSid(pSid) as usize;
    let ace_size = 4 + 4 + sid_len;
    let mut buf = [0u8; 4 + 4 + SECURITY_MAX_SID_SIZE];
    buf[0] = SYSTEM_MANDATORY_LABEL_ACE_TYPE;
    buf[1] = dwAceFlags as u8;
    buf[2..4].copy_from_slice(&(ace_size as u16).to_le_bytes());
    buf[4..8].copy_from_slice(&MandatoryPolicy.to_le_bytes());
    core::ptr::copy_nonoverlapping(pSid as *const u8, buf.as_mut_ptr().add(8), sid_len);
    AddAce(pAcl, dwAceRevision, MAXDWORD, buf.as_mut_ptr() as LPVOID, ace_size as DWORD)
}

// Conditional ACEs (Windows 10 claims-based conditions) need the ACE condition-expression
// parser, which this tree does not have. Wine's body is FIXME + FALSE with no error code;
// we report ERROR_CALL_NOT_IMPLEMENTED so a caller can tell "not supported" from a bad ACL.
#[no_mangle]
pub unsafe extern "system" fn AddConditionalAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    dwAceFlags: DWORD,
    ace_type: u8,
    AccessMask: DWORD,
    pSid: PSID,
    ConditionStr: *mut u16,
    ReturnLength: *mut DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn AddAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    dwStartingAceIndex: DWORD,
    pAceList: LPVOID,
    nAceListLength: DWORD,
) -> BOOL {
    let Status = RtlAddAce(
        pAcl,
        dwAceRevision,
        dwStartingAceIndex,
        pAceList,
        nAceListLength,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn DeleteAce(pAcl: PACL, dwAceIndex: DWORD) -> BOOL {
    set_ntstatus(RtlDeleteAce(pAcl, dwAceIndex))
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstFreeAce(pAcl: PACL, pAce: *mut LPVOID) -> BOOL {
    RtlFirstFreeAce(pAcl, pAce as *mut PACE) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetAce(pAcl: PACL, dwAceIndex: DWORD, pAce: *mut LPVOID) -> BOOL {
    set_ntstatus(RtlGetAce(pAcl, dwAceIndex, pAce))
}

#[no_mangle]
pub unsafe extern "system" fn GetAclInformation(
    pAcl: PACL,
    pAclInformation: LPVOID,
    nAclInformationLength: DWORD,
    dwAclInformationClass: ACL_INFORMATION_CLASS,
) -> BOOL {
    set_ntstatus(RtlQueryInformationAcl(
        pAcl,
        pAclInformation,
        nAclInformationLength,
        dwAclInformationClass,
    ))
}

#[no_mangle]
pub unsafe extern "system" fn IsValidAcl(pAcl: PACL) -> BOOL {
    RtlValidAcl(pAcl) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn AllocateLocallyUniqueId(Luid: PLUID) -> BOOL {
    let Status: NTSTATUS = NtAllocateLocallyUniqueId(Luid);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeDisplayNameA(
    lpSystemName: LPCSTR,
    lpName: LPCSTR,
    lpDisplayName: LPSTR,
    cchDisplayName: LPDWORD,
    lpLanguageId: LPDWORD,
) -> BOOL {
    let mut lpSystemNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut lpNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL;
    let mut wLen: DWORD = 0;

    TRACE!(
        b"%s %s %p %p %p\n\0".as_ptr() as LPCSTR,
        debugstr_a(lpSystemName),
        debugstr_a(lpName),
        lpName,
        cchDisplayName,
        lpLanguageId,
    );

    RtlCreateUnicodeStringFromAsciiz(&mut lpSystemNameW, lpSystemName);
    RtlCreateUnicodeStringFromAsciiz(&mut lpNameW, lpName);
    ret = LookupPrivilegeDisplayNameW(
        lpSystemNameW.Buffer,
        lpNameW.Buffer,
        core::ptr::null_mut(),
        &mut wLen,
        lpLanguageId,
    );
    if ret == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        let lpDisplayNameW: LPWSTR = HeapAlloc(
            GetProcessHeap(),
            0,
            (wLen as usize) * core::mem::size_of::<u16>(),
        ) as LPWSTR;

        ret = LookupPrivilegeDisplayNameW(
            lpSystemNameW.Buffer,
            lpNameW.Buffer,
            lpDisplayNameW,
            &mut wLen,
            lpLanguageId,
        );
        if ret != 0 {
            let len: u32 = WideCharToMultiByte(
                CP_ACP,
                0,
                lpDisplayNameW,
                -1,
                lpDisplayName,
                *cchDisplayName as INT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as u32;

            if len == 0 {
                ret = 0;
            } else if len > *cchDisplayName {
                *cchDisplayName = len;
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                ret = 0;
            } else {
                *cchDisplayName = len - 1;
            }
        }
        HeapFree(GetProcessHeap(), 0, lpDisplayNameW as LPVOID);
    }
    RtlFreeUnicodeString(&mut lpSystemNameW);
    RtlFreeUnicodeString(&mut lpNameW);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeNameA(
    lpSystemName: LPCSTR,
    lpLuid: PLUID,
    lpName: LPSTR,
    cchName: LPDWORD,
) -> BOOL {
    let mut lpSystemNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL;
    let mut wLen: DWORD = 0;

    TRACE!(
        b"%s %p %p %p\n\0".as_ptr() as LPCSTR,
        debugstr_a(lpSystemName),
        lpLuid,
        lpName,
        cchName,
    );

    RtlCreateUnicodeStringFromAsciiz(&mut lpSystemNameW, lpSystemName);
    ret = LookupPrivilegeNameW(
        lpSystemNameW.Buffer,
        lpLuid,
        core::ptr::null_mut(),
        &mut wLen,
    );
    if ret == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        let lpNameW: LPWSTR = HeapAlloc(
            GetProcessHeap(),
            0,
            (wLen as usize) * core::mem::size_of::<u16>(),
        ) as LPWSTR;

        ret = LookupPrivilegeNameW(lpSystemNameW.Buffer, lpLuid, lpNameW, &mut wLen);
        if ret != 0 {
            let len: u32 = WideCharToMultiByte(
                CP_ACP,
                0,
                lpNameW,
                -1,
                lpName,
                *cchName as INT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as u32;

            if len == 0 {
                ret = 0;
            } else if len > *cchName {
                *cchName = len;
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                ret = 0;
            } else {
                *cchName = len - 1;
            }
        }
        HeapFree(GetProcessHeap(), 0, lpNameW as LPVOID);
    }
    RtlFreeUnicodeString(&mut lpSystemNameW);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetFileSecurityA(
    lpFileName: LPCSTR,
    RequestedInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    nLength: DWORD,
    lpnLengthNeeded: LPDWORD,
) -> BOOL {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFileName as *const _) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let bResult = GetFileSecurityW(
        FileName.Buffer,
        RequestedInformation,
        pSecurityDescriptor,
        nLength,
        lpnLengthNeeded,
    );

    RtlFreeUnicodeString(&mut FileName);

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn GetFileSecurityW(
    lpFileName: LPCWSTR,
    RequestedInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    nLength: DWORD,
    lpnLengthNeeded: LPDWORD,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut AccessMask: ULONG = 0;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    QuerySecurityAccessMask(RequestedInformation, &mut AccessMask);

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        AccessMask,
        &mut ObjectAttributes,
        &mut StatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        0,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    Status = NtQuerySecurityObject(
        FileHandle,
        RequestedInformation,
        pSecurityDescriptor,
        nLength,
        lpnLengthNeeded,
    );
    NtClose(FileHandle);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetFileSecurityA(
    lpFileName: LPCSTR,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFileName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let bResult = SetFileSecurityW(FileName.Buffer, SecurityInformation, pSecurityDescriptor);

    RtlFreeUnicodeString(&mut FileName);

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn SetFileSecurityW(
    lpFileName: LPCWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut AccessMask: ULONG = 0;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    SetSecurityAccessMask(SecurityInformation, &mut AccessMask);

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        AccessMask,
        &mut ObjectAttributes,
        &mut StatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        0,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    Status = NtSetSecurityObject(FileHandle, SecurityInformation, pSecurityDescriptor);
    NtClose(FileHandle);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn QueryWindows31FilesMigration(_x1: DWORD) -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "system" fn SynchronizeWindows31FilesAndWindowsNTRegistry(
    x1: DWORD,
    x2: DWORD,
    x3: DWORD,
    x4: DWORD,
) -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "system" fn RevertToSelf() -> BOOL {
    let Status: NTSTATUS;
    let mut Token: HANDLE = core::ptr::null_mut();

    Status = NtSetInformationThread(
        -2isize as HANDLE,
        ThreadImpersonationToken,
        core::ptr::addr_of_mut!(Token) as *mut core::ffi::c_void,
        core::mem::size_of::<HANDLE>() as u32,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ImpersonateSelf(ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL) -> BOOL {
    let Status: NTSTATUS = RtlImpersonateSelf(ImpersonationLevel);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheck(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    GenericMapping: PGENERIC_MAPPING,
    PrivilegeSet: PPRIVILEGE_SET,
    PrivilegeSetLength: LPDWORD,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
) -> BOOL {
    let mut NtAccessStatus: NTSTATUS = 0;

    let Status = NtAccessCheck(
        pSecurityDescriptor,
        ClientToken,
        DesiredAccess,
        GenericMapping,
        PrivilegeSet,
        PrivilegeSetLength as PULONG,
        GrantedAccess as PACCESS_MASK,
        &mut NtAccessStatus,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    if !NT_SUCCESS(NtAccessStatus) {
        SetLastError(RtlNtStatusToDosError(NtAccessStatus));
        *AccessStatus = 0;
    } else {
        *AccessStatus = 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByType(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: DWORD,
    GenericMapping: PGENERIC_MAPPING,
    PrivilegeSet: PPRIVILEGE_SET,
    PrivilegeSetLength: LPDWORD,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
) -> BOOL {
    let mut NtAccessStatus: NTSTATUS = 0;

    let Status = NtAccessCheckByType(
        pSecurityDescriptor,
        PrincipalSelfSid,
        ClientToken,
        DesiredAccess,
        ObjectTypeList,
        ObjectTypeListLength,
        GenericMapping,
        PrivilegeSet,
        PrivilegeSetLength,
        GrantedAccess as PACCESS_MASK,
        &mut NtAccessStatus,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    if !NT_SUCCESS(NtAccessStatus) {
        SetLastError(RtlNtStatusToDosError(NtAccessStatus));
        *AccessStatus = 0;
    } else {
        *AccessStatus = 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeResultList(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: DWORD,
    GenericMapping: PGENERIC_MAPPING,
    PrivilegeSet: PPRIVILEGE_SET,
    PrivilegeSetLength: LPDWORD,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
) -> BOOL {
    let Status: NTSTATUS;
    let mut ResultListIndex: DWORD;
    let NtAccessStatus: PNTSTATUS = core::ptr::null_mut();

    Status = NtAccessCheckByTypeResultList(
        pSecurityDescriptor,
        PrincipalSelfSid,
        ClientToken,
        DesiredAccess,
        ObjectTypeList,
        ObjectTypeListLength,
        GenericMapping,
        PrivilegeSet,
        PrivilegeSetLength,
        GrantedAccess as PACCESS_MASK,
        NtAccessStatus,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    ResultListIndex = 0;
    while ResultListIndex < ObjectTypeListLength {
        *AccessStatus.add(ResultListIndex as usize) =
            RtlNtStatusToDosError(*NtAccessStatus.add(ResultListIndex as usize)) as BOOL;
        ResultListIndex += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetKernelObjectSecurity(
    Handle: HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOL {
    let Status = NtSetSecurityObject(Handle, SecurityInformation, SecurityDescriptor);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAuditAccessAce(
    pAcl: PACL,
    dwAceRevision: DWORD,
    dwAccessMask: DWORD,
    pSid: PSID,
    bAuditSuccess: BOOL,
    bAuditFailure: BOOL,
) -> BOOL {
    let Status: NTSTATUS = RtlAddAuditAccessAce(
        pAcl,
        dwAceRevision,
        dwAccessMask,
        pSid,
        bAuditSuccess as u8,
        bAuditFailure as u8,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAuditAccessAceEx(
    pAcl: PACL,
    dwAceRevision: DWORD,
    AceFlags_p: DWORD,
    dwAccessMask: DWORD,
    pSid: PSID,
    bAuditSuccess: BOOL,
    bAuditFailure: BOOL,
) -> BOOL {
    let Status: NTSTATUS = RtlAddAuditAccessAceEx(
        pAcl,
        dwAceRevision,
        AceFlags_p,
        dwAccessMask,
        pSid,
        bAuditSuccess as u8,
        bAuditFailure as u8,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn LookupAccountNameA(
    SystemName: LPCSTR,
    AccountName: LPCSTR,
    Sid: PSID,
    SidLength: LPDWORD,
    ReferencedDomainName: LPSTR,
    hReferencedDomainNameLength: LPDWORD,
    SidNameUse: PSID_NAME_USE,
) -> BOOL {
    let ret: BOOL;
    let mut lpSystemW: _UNICODE_STRING = core::mem::zeroed();
    let mut lpAccountW: _UNICODE_STRING = core::mem::zeroed();
    let mut lpReferencedDomainNameW: LPWSTR = core::ptr::null_mut();

    RtlCreateUnicodeStringFromAsciiz(&mut lpSystemW, SystemName);
    RtlCreateUnicodeStringFromAsciiz(&mut lpAccountW, AccountName);

    if !ReferencedDomainName.is_null() {
        lpReferencedDomainNameW = HeapAlloc(
            GetProcessHeap(),
            0,
            (*hReferencedDomainNameLength as usize) * core::mem::size_of::<u16>(),
        ) as LPWSTR;
    }

    ret = LookupAccountNameW(
        lpSystemW.Buffer,
        lpAccountW.Buffer,
        Sid,
        SidLength,
        lpReferencedDomainNameW,
        hReferencedDomainNameLength,
        SidNameUse,
    );

    if ret != 0 && !lpReferencedDomainNameW.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            lpReferencedDomainNameW,
            (*hReferencedDomainNameLength + 1) as i32,
            ReferencedDomainName,
            (*hReferencedDomainNameLength + 1) as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    RtlFreeUnicodeString(&mut lpSystemW);
    RtlFreeUnicodeString(&mut lpAccountW);
    HeapFree(GetProcessHeap(), 0, lpReferencedDomainNameW as LPVOID);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PrivilegeCheck(
    ClientToken: HANDLE,
    RequiredPrivileges: PPRIVILEGE_SET,
    pfResult: LPBOOL,
) -> BOOL {
    let mut Result: BOOLEAN = 0;
    let Status: NTSTATUS;

    Status = NtPrivilegeCheck(ClientToken, RequiredPrivileges, &mut Result);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    *pfResult = Result as BOOL;

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityInfoExA(
    hObject: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    lpProvider: LPCSTR,
    lpProperty: LPCSTR,
    ppAccessList: *mut PACTRL_ACCESSA,
    ppAuditList: *mut PACTRL_AUDITA,
    lppOwner: *mut LPSTR,
    lppGroup: *mut LPSTR,
) -> DWORD {
    ERROR_BAD_PROVIDER
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityInfoExW(
    hObject: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    lpProvider: LPCWSTR,
    lpProperty: LPCWSTR,
    ppAccessList: *mut PACTRL_ACCESSW,
    ppAuditList: *mut PACTRL_AUDITW,
    lppOwner: *mut LPWSTR,
    lppGroup: *mut LPWSTR,
) -> DWORD {
    ERROR_BAD_PROVIDER
}

#[no_mangle]
pub unsafe extern "system" fn BuildExplicitAccessWithNameA(
    pExplicitAccess: PEXPLICIT_ACCESSA,
    pTrusteeName: LPSTR,
    AccessPermissions: DWORD,
    AccessMode: ACCESS_MODE,
    Inheritance: DWORD,
) {
    (*pExplicitAccess).grfAccessPermissions = AccessPermissions;
    (*pExplicitAccess).grfAccessMode = AccessMode;
    (*pExplicitAccess).grfInheritance = Inheritance;

    (*pExplicitAccess).Trustee.pMultipleTrustee = core::ptr::null_mut();
    (*pExplicitAccess).Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pExplicitAccess).Trustee.TrusteeForm = TRUSTEE_IS_NAME;
    (*pExplicitAccess).Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pExplicitAccess).Trustee.ptstrName = pTrusteeName;
}

#[no_mangle]
pub unsafe extern "system" fn BuildExplicitAccessWithNameW(
    pExplicitAccess: PEXPLICIT_ACCESSW,
    pTrusteeName: LPWSTR,
    AccessPermissions: DWORD,
    AccessMode: ACCESS_MODE,
    Inheritance: DWORD,
) {
    (*pExplicitAccess).grfAccessPermissions = AccessPermissions;
    (*pExplicitAccess).grfAccessMode = AccessMode;
    (*pExplicitAccess).grfInheritance = Inheritance;

    (*pExplicitAccess).Trustee.pMultipleTrustee = core::ptr::null_mut();
    (*pExplicitAccess).Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pExplicitAccess).Trustee.TrusteeForm = TRUSTEE_IS_NAME;
    (*pExplicitAccess).Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pExplicitAccess).Trustee.ptstrName = pTrusteeName;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithObjectsAndNameA(
    pTrustee: PTRUSTEEA,
    pObjName: POBJECTS_AND_NAME_A,
    ObjectType: SE_OBJECT_TYPE,
    ObjectTypeName: LPSTR,
    InheritedObjectTypeName: LPSTR,
    Name: LPSTR,
) {
    let mut ObjectsPresent: DWORD = 0;

    (*pObjName).ObjectType = ObjectType;
    if !ObjectTypeName.is_null() {
        ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
    }

    (*pObjName).InheritedObjectTypeName = InheritedObjectTypeName;
    if !InheritedObjectTypeName.is_null() {
        ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
    }

    (*pObjName).ObjectsPresent = ObjectsPresent;
    (*pObjName).ptstrName = Name;

    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_OBJECTS_AND_NAME;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pObjName as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithObjectsAndNameW(
    pTrustee: PTRUSTEEW,
    pObjName: POBJECTS_AND_NAME_W,
    ObjectType: SE_OBJECT_TYPE,
    ObjectTypeName: LPWSTR,
    InheritedObjectTypeName: LPWSTR,
    Name: LPWSTR,
) {
    let mut ObjectsPresent: DWORD = 0;

    (*pObjName).ObjectType = ObjectType;
    if !ObjectTypeName.is_null() {
        ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
    }

    (*pObjName).InheritedObjectTypeName = InheritedObjectTypeName;
    if !InheritedObjectTypeName.is_null() {
        ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
    }

    (*pObjName).ObjectsPresent = ObjectsPresent;
    (*pObjName).ptstrName = Name;

    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_OBJECTS_AND_NAME;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pObjName as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithObjectsAndSidA(
    pTrustee: PTRUSTEEA,
    pObjSid: POBJECTS_AND_SID,
    pObjectGuid: *mut GUID,
    pInheritedObjectGuid: *mut GUID,
    pSid: PSID,
) {
    let mut ObjectsPresent: DWORD = 0;

    if !pObjectGuid.is_null() {
        (*pObjSid).ObjectTypeGuid = *(pObjectGuid as *const _);
        ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
    } else {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*pObjSid).ObjectTypeGuid),
            0,
            1,
        );
    }

    if !pInheritedObjectGuid.is_null() {
        (*pObjSid).InheritedObjectTypeGuid = *(pInheritedObjectGuid as *const _);
        ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
    } else {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*pObjSid).InheritedObjectTypeGuid),
            0,
            1,
        );
    }

    (*pObjSid).ObjectsPresent = ObjectsPresent;
    (*pObjSid).pSid = pSid as *mut _;

    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_OBJECTS_AND_SID;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pObjSid as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithObjectsAndSidW(
    pTrustee: PTRUSTEEW,
    pObjSid: POBJECTS_AND_SID,
    pObjectGuid: *mut GUID,
    pInheritedObjectGuid: *mut GUID,
    pSid: PSID,
) {
    let mut ObjectsPresent: DWORD = 0;

    if !pObjectGuid.is_null() {
        (*pObjSid).ObjectTypeGuid = *(pObjectGuid as *const _);
        ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
    } else {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*pObjSid).ObjectTypeGuid),
            0,
            1,
        );
    }

    if !pInheritedObjectGuid.is_null() {
        (*pObjSid).InheritedObjectTypeGuid = *(pInheritedObjectGuid as *const _);
        ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
    } else {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*pObjSid).InheritedObjectTypeGuid),
            0,
            1,
        );
    }

    (*pObjSid).ObjectsPresent = ObjectsPresent;
    (*pObjSid).pSid = pSid as *mut _;

    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_OBJECTS_AND_SID;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pObjSid as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithSidA(pTrustee: PTRUSTEE_A, pSid: PSID) {
    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_SID;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pSid as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithSidW(pTrustee: PTRUSTEE_W, pSid: PSID) {
    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_SID;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = pSid as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithNameA(
    pTrustee: PTRUSTEE_A,
    name: LPSTR,
) {
    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_NAME;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = name;
}

#[no_mangle]
pub unsafe extern "system" fn BuildTrusteeWithNameW(pTrustee: PTRUSTEE_W, name: LPWSTR) {
    (*pTrustee).pMultipleTrustee = core::ptr::null_mut();
    (*pTrustee).MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    (*pTrustee).TrusteeForm = TRUSTEE_IS_NAME;
    (*pTrustee).TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pTrustee).ptstrName = name;
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeFormA(pTrustee: PTRUSTEEA) -> TRUSTEE_FORM {
    if pTrustee.is_null() {
        return TRUSTEE_BAD_FORM;
    }

    (*pTrustee).TrusteeForm
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeFormW(pTrustee: PTRUSTEEW) -> TRUSTEE_FORM {
    if pTrustee.is_null() {
        return TRUSTEE_BAD_FORM;
    }

    (*pTrustee).TrusteeForm
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeNameA(pTrustee: PTRUSTEE_A) -> LPSTR {
    (*pTrustee).ptstrName
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeNameW(pTrustee: PTRUSTEE_W) -> LPWSTR {
    (*pTrustee).ptstrName
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeTypeA(pTrustee: PTRUSTEE_A) -> TRUSTEE_TYPE {
    (*pTrustee).TrusteeType
}

#[no_mangle]
pub unsafe extern "system" fn GetTrusteeTypeW(pTrustee: PTRUSTEE_W) -> TRUSTEE_TYPE {
    (*pTrustee).TrusteeType
}

#[no_mangle]
pub unsafe extern "system" fn SetAclInformation(
    pAcl: PACL,
    pAclInformation: LPVOID,
    nAclInformationLength: DWORD,
    dwAclInformationClass: ACL_INFORMATION_CLASS,
) -> BOOL {
    let Status = RtlSetInformationAcl(
        pAcl,
        pAclInformation,
        nAclInformationLength,
        dwAclInformationClass,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetNamedSecurityInfoA(
    pObjectName: LPSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    psidOwner: PSID,
    psidGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
) -> DWORD {
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let Ret: DWORD;

    if RtlCreateUnicodeStringFromAsciiz(&mut ObjectName, pObjectName) == 0 {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    Ret = SetNamedSecurityInfoW(
        ObjectName.Buffer,
        ObjectType,
        SecurityInfo,
        psidOwner,
        psidGroup,
        pDacl,
        pSacl,
    );

    RtlFreeUnicodeString(&mut ObjectName);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AreAllAccessesGranted(
    GrantedAccess: DWORD,
    DesiredAccess: DWORD,
) -> BOOL {
    RtlAreAllAccessesGranted(GrantedAccess, DesiredAccess) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn AreAnyAccessesGranted(
    GrantedAccess: DWORD,
    DesiredAccess: DWORD,
) -> BOOL {
    RtlAreAnyAccessesGranted(GrantedAccess, DesiredAccess) as BOOL
}

unsafe fn ParseAclStringFlags(StringAcl: *mut LPCWSTR) -> DWORD {
    let mut flags: DWORD = 0;
    let mut szAcl: LPCWSTR = *StringAcl;

    while *szAcl != 0 && *szAcl != b'(' as u16 {
        if *szAcl == b'P' as u16 {
            flags |= SE_DACL_PROTECTED;
        } else if *szAcl == b'A' as u16 {
            szAcl = szAcl.add(1);
            if *szAcl == b'R' as u16 {
                flags |= SE_DACL_AUTO_INHERIT_REQ;
            } else if *szAcl == b'I' as u16 {
                flags |= SE_DACL_AUTO_INHERITED;
            }
        }
        szAcl = szAcl.add(1);
    }

    *StringAcl = szAcl;
    flags
}

unsafe fn ParseAceStringType(StringAcl: *mut LPCWSTR) -> BYTE {
    let mut len: UINT = 0;
    let mut szAcl: LPCWSTR = *StringAcl;
    let mut lpaf: *const ACEFLAG = AceType.as_ptr();

    while *szAcl == b' ' as WCHAR {
        szAcl = szAcl.add(1);
    }

    while !(*lpaf).wstr.is_null() && {
        len = strlenW((*lpaf).wstr);
        len != 0
    } && strncmpW((*lpaf).wstr, szAcl, len) != 0
    {
        lpaf = lpaf.add(1);
    }

    if (*lpaf).wstr.is_null() {
        return 0;
    }

    *StringAcl = szAcl.add(len as usize);
    (*lpaf).value as BYTE
}

unsafe fn ParseAceStringFlags(StringAcl: *mut LPCWSTR) -> BYTE {
    let mut len: UINT = 0;
    let mut flags: BYTE = 0;
    let mut szAcl: LPCWSTR = *StringAcl;

    while *szAcl == b' ' as u16 {
        szAcl = szAcl.add(1);
    }

    while *szAcl != b';' as u16 {
        let mut lpaf: *const ACEFLAG = AceFlags.as_ptr();

        loop {
            if (*lpaf).wstr.is_null() {
                break;
            }
            len = strlenW((*lpaf).wstr);
            if len == 0 {
                break;
            }
            if strncmpW((*lpaf).wstr, szAcl, len) == 0 {
                break;
            }
            lpaf = lpaf.add(1);
        }

        if (*lpaf).wstr.is_null() {
            return 0;
        }

        flags |= (*lpaf).value as BYTE;
        szAcl = szAcl.add(len as usize);
    }

    *StringAcl = szAcl;
    flags
}

unsafe fn ParseAceStringRights(StringAcl: *mut LPCWSTR) -> DWORD {
    let mut len: UINT = 0;
    let mut rights: DWORD = 0;
    let mut szAcl: LPCWSTR = *StringAcl;

    while *szAcl == b' ' as u16 {
        szAcl = szAcl.add(1);
    }

    if *szAcl == b'0' as u16 && *szAcl.add(1) == b'x' as u16 {
        let mut p: LPCWSTR = szAcl;

        while *p != 0 && *p != b';' as u16 {
            p = p.add(1);
        }

        if p.offset_from(szAcl) <= 10 {
            rights = strtoulW(szAcl, core::ptr::null_mut(), 16);
            szAcl = p;
        } else {
            WARN!(
                b"Invalid rights string format: %s\n\0".as_ptr() as LPCSTR,
                debugstr_wn(szAcl, p.offset_from(szAcl) as i32),
            );
        }
    } else {
        while *szAcl != b';' as u16 {
            let mut lpaf: *const ACEFLAG = AceRights.as_ptr();

            loop {
                if (*lpaf).wstr.is_null() {
                    break;
                }
                len = strlenW((*lpaf).wstr);
                if len == 0 {
                    break;
                }
                if strncmpW((*lpaf).wstr, szAcl, len) == 0 {
                    break;
                }
                lpaf = lpaf.add(1);
            }

            if (*lpaf).wstr.is_null() {
                return 0;
            }

            rights |= (*lpaf).value;
            szAcl = szAcl.add(len as usize);
        }
    }

    *StringAcl = szAcl;
    rights
}

unsafe fn ParseStringAclToAcl(
    mut StringAcl: LPCWSTR,
    lpdwFlags: LPDWORD,
    pAcl: PACL,
    cBytes: LPDWORD,
) -> BOOL {
    let mut val: DWORD;
    let mut sidlen: DWORD = 0;
    let mut length: DWORD = core::mem::size_of::<ACL>() as DWORD;
    let mut acesize: DWORD;
    let mut acecount: DWORD = 0;
    let mut pAce: PACCESS_ALLOWED_ACE = core::ptr::null_mut();
    let mut error: DWORD = ERROR_INVALID_ACL;

    if StringAcl.is_null() {
        return 0;
    }

    if !pAcl.is_null() {
        pAce = pAcl.add(1) as PACCESS_ALLOWED_ACE;
    }

    *lpdwFlags = ParseAclStringFlags(&mut StringAcl);

    let result = 'lerr: {
        while *StringAcl == '(' as u16 {
            StringAcl = StringAcl.add(1);

            val = ParseAceStringType(&mut StringAcl) as DWORD;
            if !pAce.is_null() {
                (*pAce).Header.AceType = val as BYTE;
            }
            if *StringAcl != ';' as u16 {
                error = RPC_S_INVALID_STRING_UUID;
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            val = ParseAceStringFlags(&mut StringAcl) as DWORD;
            if !pAce.is_null() {
                (*pAce).Header.AceFlags = val as BYTE;
            }
            if *StringAcl != ';' as u16 {
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            val = ParseAceStringRights(&mut StringAcl);
            if !pAce.is_null() {
                (*pAce).Mask = val;
            }
            if *StringAcl != ';' as u16 {
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            while *StringAcl == ' ' as u16 {
                StringAcl = StringAcl.add(1);
            }
            if *StringAcl != ';' as u16 {
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            while *StringAcl == ' ' as u16 {
                StringAcl = StringAcl.add(1);
            }
            if *StringAcl != ';' as u16 {
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            let psid = if !pAce.is_null() {
                core::ptr::addr_of_mut!((*pAce).SidStart) as PSID
            } else {
                core::ptr::null_mut()
            };
            if ParseStringSidToSid(StringAcl, psid, &mut sidlen) != 0 {
                while *StringAcl != 0 && *StringAcl != ')' as u16 {
                    StringAcl = StringAcl.add(1);
                }
            }

            if *StringAcl != ')' as u16 {
                break 'lerr 0u8;
            }
            StringAcl = StringAcl.add(1);

            acesize = core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD
                - core::mem::size_of::<DWORD>() as DWORD
                + sidlen;
            length += acesize;
            if !pAce.is_null() {
                (*pAce).Header.AceSize = acesize as u16;
                pAce = ((pAce as LPBYTE).add(acesize as usize)) as PACCESS_ALLOWED_ACE;
            }
            acecount += 1;
        }

        *cBytes = length;

        if length > 0xffff {
            break 'lerr 0u8;
        }

        if !pAcl.is_null() {
            (*pAcl).AclRevision = ACL_REVISION as BYTE;
            (*pAcl).Sbz1 = 0;
            (*pAcl).AclSize = length as u16;
            (*pAcl).AceCount = acecount as u16;
            (*pAcl).Sbz2 = 0;
        }
        return 1;
    };

    SetLastError(error);
    result as BOOL
}

unsafe fn ParseStringSecurityDescriptorToSecurityDescriptor(
    mut StringSecurityDescriptor: LPCWSTR,
    SecurityDescriptor: *mut SECURITY_DESCRIPTOR_RELATIVE,
    cBytes: LPDWORD,
) -> BOOL {
    let mut bret: BOOL = 0;
    let mut toktype: WCHAR;
    let tok: *mut WCHAR;
    let mut lptoken: LPCWSTR;
    let mut lpNext: LPBYTE = core::ptr::null_mut();
    let mut len: DWORD;

    *cBytes = core::mem::size_of::<SECURITY_DESCRIPTOR_RELATIVE>() as DWORD;

    tok = heap_alloc(((lstrlenW(StringSecurityDescriptor) + 1) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;

    if !SecurityDescriptor.is_null() {
        lpNext = SecurityDescriptor.add(1) as LPBYTE;
    }

    while *StringSecurityDescriptor == b' ' as WCHAR {
        StringSecurityDescriptor = StringSecurityDescriptor.add(1);
    }

    'lend: loop {
        while *StringSecurityDescriptor != 0 {
            toktype = *StringSecurityDescriptor;

            StringSecurityDescriptor = StringSecurityDescriptor.add(1);
            if *StringSecurityDescriptor != b':' as WCHAR {
                SetLastError(ERROR_INVALID_PARAMETER);
                break 'lend;
            }
            StringSecurityDescriptor = StringSecurityDescriptor.add(1);

            lptoken = StringSecurityDescriptor;
            while *lptoken != 0 && *lptoken != b':' as WCHAR {
                lptoken = lptoken.add(1);
            }

            if *lptoken != 0 {
                lptoken = lptoken.sub(1);
            }

            len = lptoken.offset_from(StringSecurityDescriptor) as DWORD;
            memcpy(
                tok as LPVOID,
                StringSecurityDescriptor as LPVOID,
                (len as usize) * core::mem::size_of::<WCHAR>(),
            );
            *tok.add(len as usize) = 0;

            match toktype as u8 as char {
                'O' => {
                    let mut bytes: DWORD = 0;

                    if ParseStringSidToSid(tok, lpNext as PSID, &mut bytes) == 0 {
                        break 'lend;
                    }

                    if !SecurityDescriptor.is_null() {
                        (*SecurityDescriptor).Owner =
                            lpNext.offset_from(SecurityDescriptor as LPBYTE) as DWORD;
                        lpNext = lpNext.add(bytes as usize);
                    }

                    *cBytes += bytes;
                }
                'G' => {
                    let mut bytes: DWORD = 0;

                    if ParseStringSidToSid(tok, lpNext as PSID, &mut bytes) == 0 {
                        break 'lend;
                    }

                    if !SecurityDescriptor.is_null() {
                        (*SecurityDescriptor).Group =
                            lpNext.offset_from(SecurityDescriptor as LPBYTE) as DWORD;
                        lpNext = lpNext.add(bytes as usize);
                    }

                    *cBytes += bytes;
                }
                'D' => {
                    let mut flags: DWORD = 0;
                    let mut bytes: DWORD = 0;

                    if ParseStringAclToAcl(tok, &mut flags, lpNext as PACL, &mut bytes) == 0 {
                        break 'lend;
                    }

                    if !SecurityDescriptor.is_null() {
                        (*SecurityDescriptor).Control |= (SE_DACL_PRESENT | flags) as u16;
                        (*SecurityDescriptor).Dacl =
                            lpNext.offset_from(SecurityDescriptor as LPBYTE) as DWORD;
                        lpNext = lpNext.add(bytes as usize);
                    }

                    *cBytes += bytes;
                }
                'S' => {
                    let mut flags: DWORD = 0;
                    let mut bytes: DWORD = 0;

                    if ParseStringAclToAcl(tok, &mut flags, lpNext as PACL, &mut bytes) == 0 {
                        break 'lend;
                    }

                    if !SecurityDescriptor.is_null() {
                        (*SecurityDescriptor).Control |= (SE_SACL_PRESENT | flags) as u16;
                        (*SecurityDescriptor).Sacl =
                            lpNext.offset_from(SecurityDescriptor as LPBYTE) as DWORD;
                        lpNext = lpNext.add(bytes as usize);
                    }

                    *cBytes += bytes;
                }
                _ => {
                    SetLastError(ERROR_INVALID_PARAMETER);
                    break 'lend;
                }
            }

            StringSecurityDescriptor = lptoken;
        }

        bret = 1;
        break 'lend;
    }

    heap_free(tok as LPVOID);
    bret
}

#[no_mangle]
pub unsafe extern "system" fn ConvertStringSecurityDescriptorToSecurityDescriptorA(
    StringSecurityDescriptor: LPCSTR,
    StringSDRevision: DWORD,
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    SecurityDescriptorSize: PULONG,
) -> BOOL {

    let mut ret: BOOL = 0;

    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        StringSecurityDescriptor,
        -1,
        core::ptr::null_mut(),
        0,
    ) as UINT;

    let StringSecurityDescriptorW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<WCHAR>(),
    ) as LPWSTR;

    if !StringSecurityDescriptorW.is_null() {
        MultiByteToWideChar(
            CP_ACP,
            0,
            StringSecurityDescriptor,
            -1,
            StringSecurityDescriptorW,
            len as i32,
        );

        ret = ConvertStringSecurityDescriptorToSecurityDescriptorW(
            StringSecurityDescriptorW,
            StringSDRevision,
            SecurityDescriptor,
            SecurityDescriptorSize,
        );

        HeapFree(GetProcessHeap(), 0, StringSecurityDescriptorW as LPVOID);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
    StringSecurityDescriptor: LPCWSTR,
    StringSDRevision: DWORD,
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    SecurityDescriptorSize: PULONG,
) -> BOOL {
    let mut cBytes: DWORD = 0;
    let psd: *mut _SECURITY_DESCRIPTOR;
    let mut bret: BOOL = 0;

    'lend: {
        if (GetVersion() & 0x80000000) != 0 {
            SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'lend;
        } else if StringSecurityDescriptor.is_null() || SecurityDescriptor.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'lend;
        } else if StringSDRevision != SID_REVISION as DWORD {
            SetLastError(ERROR_UNKNOWN_REVISION);
            break 'lend;
        }

        if ParseStringSecurityDescriptorToSecurityDescriptor(
            StringSecurityDescriptor,
            core::ptr::null_mut(),
            &mut cBytes,
        ) == 0
        {
            break 'lend;
        }

        psd = LocalAlloc(GMEM_ZEROINIT, cBytes as usize) as *mut _SECURITY_DESCRIPTOR;
        *SecurityDescriptor = psd as PSECURITY_DESCRIPTOR;
        if psd.is_null() {
            break 'lend;
        }

        (*psd).Revision = SID_REVISION as u8;
        (*psd).Control |= SE_SELF_RELATIVE as u16;

        if ParseStringSecurityDescriptorToSecurityDescriptor(
            StringSecurityDescriptor,
            psd as *mut SECURITY_DESCRIPTOR_RELATIVE,
            &mut cBytes,
        ) == 0
        {
            LocalFree(psd as *mut _);
            break 'lend;
        }

        if !SecurityDescriptorSize.is_null() {
            *SecurityDescriptorSize = cBytes;
        }

        bret = 1;
    }

    bret
}
unsafe fn DumpString(string: LPCWSTR, mut cch: i32, pwptr: *mut *mut WCHAR, plen: *mut ULONG) {
    if cch == -1 {
        cch = strlenW(string) as i32;
    }

    if !plen.is_null() {
        *plen += cch as ULONG;
    }

    if !pwptr.is_null() {
        memcpy(
            *pwptr as LPVOID,
            string as LPVOID,
            core::mem::size_of::<WCHAR>() * cch as usize,
        );
        *pwptr = (*pwptr).add(cch as usize);
    }
}

unsafe fn DumpSidNumeric(psid: PSID, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let mut i: DWORD;
    let mut fmt: [WCHAR; 8] = [b'S' as WCHAR, b'-' as WCHAR, b'%' as WCHAR, b'u' as WCHAR, b'-' as WCHAR, b'%' as WCHAR, b'd' as WCHAR, 0];
    let mut subauthfmt: [WCHAR; 4] = [b'-' as WCHAR, b'%' as WCHAR, b'u' as WCHAR, 0];
    let mut buf: [WCHAR; 26] = [0; 26];
    let pisid: *mut SID = psid as *mut SID;

    if IsValidSid(psid) == 0 || (*pisid).Revision as DWORD != SDDL_REVISION {
        SetLastError(ERROR_INVALID_SID);
        return 0;
    }

    if (*pisid).IdentifierAuthority.Value[0] != 0 || (*pisid).IdentifierAuthority.Value[1] != 0 {
        FIXME!("not matching MS' bugs\n");
        SetLastError(ERROR_INVALID_SID);
        return 0;
    }

    sprintfW(
        buf.as_mut_ptr(),
        fmt.as_mut_ptr(),
        (*pisid).Revision as DWORD,
        MAKELONG(
            MAKEWORD((*pisid).IdentifierAuthority.Value[5], (*pisid).IdentifierAuthority.Value[4]),
            MAKEWORD((*pisid).IdentifierAuthority.Value[3], (*pisid).IdentifierAuthority.Value[2]),
        ),
    );
    DumpString(buf.as_ptr(), -1, pwptr, plen);

    i = 0;
    while i < (*pisid).SubAuthorityCount as DWORD {
        // SID.SubAuthority is `[size_is(SubAuthorityCount)] SubAuthority[ANYSIZE_ARRAY]`,
        // and this loop is bounded by SubAuthorityCount -- so bindgen's `[DWORD; 1]`
        // panicked on any SID with more than one sub-authority (nearly all of them).
        // NOT the same as WellKnownSids[].Sid.SubAuthority below: that is MAX_SID's
        // genuine `[DWORD; 15]`, whose bounds check is real protection.
        sprintfW(buf.as_mut_ptr(), subauthfmt.as_mut_ptr(),
                 (&raw const (*pisid).SubAuthority[0]).add(i as usize).read());
        DumpString(buf.as_ptr(), -1, pwptr, plen);
        i += 1;
    }
    1
}

unsafe fn DumpSid(psid: PSID, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let count = WellKnownSids.len();
    let mut i: usize = 0;
    while i < count {
        if WellKnownSids[i].wstr[0] != 0
            && EqualSid(psid, core::ptr::addr_of!(WellKnownSids[i].Sid.Revision) as PSID) != 0
        {
            DumpString(WellKnownSids[i].wstr.as_ptr(), 2, pwptr, plen);
            return 1;
        }
        i += 1;
    }

    DumpSidNumeric(psid, pwptr, plen)
}

unsafe fn DumpRights(mask: DWORD, pwptr: *mut *mut WCHAR, plen: *mut ULONG) {
    static fmtW: [WCHAR; 5] = [b'0' as WCHAR, b'x' as WCHAR, b'%' as WCHAR, b'x' as WCHAR, 0];
    let mut buf: [WCHAR; 15] = [0; 15];

    if mask == 0 {
        return;
    }

    let mut i: usize = 0;
    let ace_rights_count = core::mem::size_of_val(&AceRights) / core::mem::size_of_val(&AceRights[0]);
    while i < ace_rights_count {
        if AceRights[i].wstr.is_null() {
            break;
        }
        if mask == AceRights[i].value {
            DumpString(AceRights[i].wstr, -1, pwptr, plen);
            return;
        }
        i += 1;
    }

    i = 0;
    while i < 32 {
        if (mask & (1 << i)) != 0 && AceRightBitNames.0[i].is_null() {
            sprintfW(buf.as_mut_ptr(), fmtW.as_ptr(), mask);
            DumpString(buf.as_ptr(), -1, pwptr, plen);
            return;
        }
        i += 1;
    }

    i = 0;
    while i < 32 {
        if (mask & (1 << i)) != 0 {
            DumpString(AceRightBitNames.0[i], -1, pwptr, plen);
        }
        i += 1;
    }
}

unsafe fn DumpAce(pace: LPVOID, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let openbr: WCHAR = b'(' as WCHAR;
    let closebr: WCHAR = b')' as WCHAR;
    let semicolon: WCHAR = b';' as WCHAR;

    let phdr = pace as PACE_HEADER;
    if (*phdr).AceType as u32 > SYSTEM_ALARM_ACE_TYPE
        || ((*phdr).AceSize as usize) < core::mem::size_of::<ACCESS_ALLOWED_ACE>()
    {
        SetLastError(ERROR_INVALID_ACL);
        return 0;
    }

    let piace = pace as *mut ACCESS_ALLOWED_ACE;
    DumpString(&openbr, 1, pwptr, plen);
    match (*piace).Header.AceType as u32 {
        ACCESS_ALLOWED_ACE_TYPE => {
            DumpString(SDDL_ACCESS_ALLOWED.as_ptr(), -1, pwptr, plen);
        }
        ACCESS_DENIED_ACE_TYPE => {
            DumpString(SDDL_ACCESS_DENIED.as_ptr(), -1, pwptr, plen);
        }
        SYSTEM_AUDIT_ACE_TYPE => {
            DumpString(SDDL_AUDIT.as_ptr(), -1, pwptr, plen);
        }
        SYSTEM_ALARM_ACE_TYPE => {
            DumpString(SDDL_ALARM.as_ptr(), -1, pwptr, plen);
        }
        _ => {}
    }
    DumpString(&semicolon, 1, pwptr, plen);

    let flags = (*piace).Header.AceFlags as u32;
    if flags & OBJECT_INHERIT_ACE != 0 {
        DumpString(SDDL_OBJECT_INHERIT.as_ptr(), -1, pwptr, plen);
    }
    if flags & CONTAINER_INHERIT_ACE != 0 {
        DumpString(SDDL_CONTAINER_INHERIT.as_ptr(), -1, pwptr, plen);
    }
    if flags & NO_PROPAGATE_INHERIT_ACE != 0 {
        DumpString(SDDL_NO_PROPAGATE.as_ptr(), -1, pwptr, plen);
    }
    if flags & INHERIT_ONLY_ACE != 0 {
        DumpString(SDDL_INHERIT_ONLY.as_ptr(), -1, pwptr, plen);
    }
    if flags & INHERITED_ACE != 0 {
        DumpString(SDDL_INHERITED.as_ptr(), -1, pwptr, plen);
    }
    if flags & SUCCESSFUL_ACCESS_ACE_FLAG != 0 {
        DumpString(SDDL_AUDIT_SUCCESS.as_ptr(), -1, pwptr, plen);
    }
    if flags & FAILED_ACCESS_ACE_FLAG != 0 {
        DumpString(SDDL_AUDIT_FAILURE.as_ptr(), -1, pwptr, plen);
    }
    DumpString(&semicolon, 1, pwptr, plen);
    DumpRights((*piace).Mask, pwptr, plen);
    DumpString(&semicolon, 1, pwptr, plen);
    DumpString(&semicolon, 1, pwptr, plen);
    DumpString(&semicolon, 1, pwptr, plen);
    if DumpSid(core::ptr::addr_of_mut!((*piace).SidStart) as PSID, pwptr, plen) == 0 {
        return 0;
    }
    DumpString(&closebr, 1, pwptr, plen);
    1
}

unsafe fn DumpAcl(pacl: PACL, pwptr: *mut *mut WCHAR, plen: *mut ULONG, protected_: BOOL, autoInheritReq: BOOL, autoInherited: BOOL) -> BOOL {
    let count: WORD;
    let mut i: i32;

    if protected_ != 0 {
        DumpString(SDDL_PROTECTED.as_ptr(), -1, pwptr, plen);
    }
    if autoInheritReq != 0 {
        DumpString(SDDL_AUTO_INHERIT_REQ.as_ptr(), -1, pwptr, plen);
    }
    if autoInherited != 0 {
        DumpString(SDDL_AUTO_INHERITED.as_ptr(), -1, pwptr, plen);
    }

    if pacl.is_null() {
        return 1;
    }

    if IsValidAcl(pacl) == 0 {
        return 0;
    }

    count = (*pacl).AceCount;
    i = 0;
    while i < count as i32 {
        let mut ace: LPVOID = core::ptr::null_mut();
        if GetAce(pacl, i as DWORD, &mut ace) == 0 {
            return 0;
        }
        if DumpAce(ace, pwptr, plen) == 0 {
            return 0;
        }
        i += 1;
    }

    1
}

unsafe fn DumpOwner(SecurityDescriptor: PSECURITY_DESCRIPTOR, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let prefix: [WCHAR; 3] = ['O' as WCHAR, ':' as WCHAR, 0];
    let mut bDefaulted: BOOL = 0;
    let mut psid: PSID = core::ptr::null_mut();

    if GetSecurityDescriptorOwner(SecurityDescriptor, &mut psid, &mut bDefaulted) == 0 {
        return 0;
    }

    if psid.is_null() {
        return 1;
    }

    DumpString(prefix.as_ptr(), -1, pwptr, plen);
    if DumpSid(psid, pwptr, plen) == 0 {
        return 0;
    }
    1
}

unsafe fn DumpGroup(SecurityDescriptor: PSECURITY_DESCRIPTOR, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let prefix: [WCHAR; 3] = ['G' as WCHAR, ':' as WCHAR, 0];
    let mut bDefaulted: BOOL = 0;
    let mut psid: PSID = core::ptr::null_mut();

    if GetSecurityDescriptorGroup(SecurityDescriptor, &mut psid, &mut bDefaulted) == 0 {
        return 0;
    }

    if psid.is_null() {
        return 1;
    }

    DumpString(prefix.as_ptr(), -1, pwptr, plen);
    if DumpSid(psid, pwptr, plen) == 0 {
        return 0;
    }
    1
}

unsafe fn DumpDacl(SecurityDescriptor: PSECURITY_DESCRIPTOR, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let dacl: [WCHAR; 3] = [b'D' as WCHAR, b':' as WCHAR, 0];
    let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut present: BOOL = 0;
    let mut defaulted: BOOL = 0;
    let mut revision: DWORD = 0;
    let mut pacl: PACL = core::ptr::null_mut();

    if GetSecurityDescriptorDacl(
        SecurityDescriptor,
        &mut present,
        &mut pacl,
        &mut defaulted,
    ) == 0
    {
        return 0;
    }

    if GetSecurityDescriptorControl(SecurityDescriptor, &mut control, &mut revision) == 0 {
        return 0;
    }

    if present == 0 {
        return 1;
    }

    DumpString(dacl.as_ptr(), 2, pwptr, plen);
    if DumpAcl(
        pacl,
        pwptr,
        plen,
        (control & SE_DACL_PROTECTED as SECURITY_DESCRIPTOR_CONTROL) as BOOL as i32,
        (control & SE_DACL_AUTO_INHERIT_REQ as SECURITY_DESCRIPTOR_CONTROL) as BOOL as i32,
        (control & SE_DACL_AUTO_INHERITED as SECURITY_DESCRIPTOR_CONTROL) as BOOL as i32,
    ) == 0
    {
        return 0;
    }
    1
}

unsafe fn DumpSacl(SecurityDescriptor: PSECURITY_DESCRIPTOR, pwptr: *mut *mut WCHAR, plen: *mut ULONG) -> BOOL {
    let sacl: [WCHAR; 3] = ['S' as WCHAR, ':' as WCHAR, 0];
    let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut present: BOOL = 0;
    let mut defaulted: BOOL = 0;
    let mut revision: DWORD = 0;
    let mut pacl: PACL = core::ptr::null_mut();

    if GetSecurityDescriptorSacl(SecurityDescriptor, &mut present, &mut pacl, &mut defaulted) == 0 {
        return 0;
    }

    if GetSecurityDescriptorControl(SecurityDescriptor, &mut control, &mut revision) == 0 {
        return 0;
    }

    if present == 0 {
        return 1;
    }

    DumpString(sacl.as_ptr(), 2, pwptr, plen);
    if DumpAcl(
        pacl,
        pwptr,
        plen,
        (control & SE_SACL_PROTECTED as SECURITY_DESCRIPTOR_CONTROL) as BOOL,
        (control & SE_SACL_AUTO_INHERIT_REQ as SECURITY_DESCRIPTOR_CONTROL) as BOOL,
        (control & SE_SACL_AUTO_INHERITED as SECURITY_DESCRIPTOR_CONTROL) as BOOL,
    ) == 0
    {
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn ConvertSecurityDescriptorToStringSecurityDescriptorW(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    SDRevision: DWORD,
    RequestedInformation: SECURITY_INFORMATION,
    OutputString: *mut LPWSTR,
    OutputLen: PULONG,
) -> BOOL {
    let mut len: ULONG;
    let mut wptr: *mut WCHAR;
    let wstr: *mut WCHAR;

    if SDRevision != SDDL_REVISION_1 as DWORD {
        SetLastError(ERROR_UNKNOWN_REVISION);
        return 0;
    }

    len = 0;
    if RequestedInformation & OWNER_SECURITY_INFORMATION != 0 {
        if DumpOwner(SecurityDescriptor, core::ptr::null_mut(), &mut len) == 0 {
            return 0;
        }
    }
    if RequestedInformation & GROUP_SECURITY_INFORMATION != 0 {
        if DumpGroup(SecurityDescriptor, core::ptr::null_mut(), &mut len) == 0 {
            return 0;
        }
    }
    if RequestedInformation & DACL_SECURITY_INFORMATION != 0 {
        if DumpDacl(SecurityDescriptor, core::ptr::null_mut(), &mut len) == 0 {
            return 0;
        }
    }
    if RequestedInformation & SACL_SECURITY_INFORMATION != 0 {
        if DumpSacl(SecurityDescriptor, core::ptr::null_mut(), &mut len) == 0 {
            return 0;
        }
    }

    wstr = LocalAlloc(0, ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as usize) as *mut WCHAR;
    wptr = wstr;
    if wstr.is_null() {
        return 0;
    }

    if RequestedInformation & OWNER_SECURITY_INFORMATION != 0 {
        if DumpOwner(SecurityDescriptor, &mut wptr, core::ptr::null_mut()) == 0 {
            LocalFree(wstr as *mut _);
            return 0;
        }
    }
    if RequestedInformation & GROUP_SECURITY_INFORMATION != 0 {
        if DumpGroup(SecurityDescriptor, &mut wptr, core::ptr::null_mut()) == 0 {
            LocalFree(wstr as *mut _);
            return 0;
        }
    }
    if RequestedInformation & DACL_SECURITY_INFORMATION != 0 {
        if DumpDacl(SecurityDescriptor, &mut wptr, core::ptr::null_mut()) == 0 {
            LocalFree(wstr as *mut _);
            return 0;
        }
    }
    if RequestedInformation & SACL_SECURITY_INFORMATION != 0 {
        if DumpSacl(SecurityDescriptor, &mut wptr, core::ptr::null_mut()) == 0 {
            LocalFree(wstr as *mut _);
            return 0;
        }
    }
    *wptr = 0;

    *OutputString = wstr;
    if !OutputLen.is_null() {
        *OutputLen = strlenW(*OutputString) + 1;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn ConvertSecurityDescriptorToStringSecurityDescriptorA(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    SDRevision: DWORD,
    Information: SECURITY_INFORMATION,
    OutputString: *mut LPSTR,
    OutputLen: PULONG,
) -> BOOL {
    let mut wstr: LPWSTR = core::ptr::null_mut();
    let mut len: ULONG = 0;
    if ConvertSecurityDescriptorToStringSecurityDescriptorW(
        SecurityDescriptor,
        SDRevision,
        Information,
        &mut wstr,
        &mut len,
    ) != 0
    {
        let lenA: i32 = WideCharToMultiByte(
            CP_ACP,
            0,
            wstr,
            len as i32,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        *OutputString = heap_alloc(lenA as usize) as LPSTR;
        if (*OutputString).is_null() {
            LocalFree(wstr as HLOCAL);
            *OutputLen = 0;
            return 0;
        }
        WideCharToMultiByte(
            CP_ACP,
            0,
            wstr,
            len as i32,
            *OutputString,
            lenA,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        LocalFree(wstr as HLOCAL);

        if !OutputLen.is_null() {
            *OutputLen = lenA as ULONG;
        }
        1
    } else {
        *OutputString = core::ptr::null_mut();
        if !OutputLen.is_null() {
            *OutputLen = 0;
        }
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConvertStringSidToSidW(StringSid: LPCWSTR, Sid: *mut PSID) -> BOOL {
    let mut bret: BOOL = 0;
    let mut cBytes: DWORD = 0;

    if GetVersion() & 0x80000000 != 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    } else if StringSid.is_null() || Sid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if ParseStringSidToSid(StringSid, core::ptr::null_mut(), &mut cBytes) != 0 {
        let pSid: PSID = LocalAlloc(0, cBytes as _) as PSID;
        *Sid = pSid;

        bret = ParseStringSidToSid(StringSid, pSid, &mut cBytes);
        if bret == 0 {
            LocalFree(*Sid as _);
        }
    }
    bret
}

#[no_mangle]
pub unsafe extern "system" fn ConvertStringSidToSidA(StringSid: LPCSTR, Sid: *mut PSID) -> BOOL {
    let mut bret: BOOL = 0;

    TRACE!(b"%s, %p\n\0".as_ptr() as LPCSTR, debugstr_a(StringSid), Sid);
    if GetVersion() & 0x80000000 != 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    } else if StringSid.is_null() || Sid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        let wStringSid: *mut WCHAR = SERV_dup(StringSid);
        bret = ConvertStringSidToSidW(wStringSid, Sid);
        heap_free(wStringSid as LPVOID);
    }
    bret
}

#[no_mangle]
pub unsafe extern "system" fn ConvertSidToStringSidW(
    Sid: PSID,
    StringSid: *mut LPWSTR,
) -> BOOL {
    let Status: NTSTATUS;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut FixedBuffer: [WCHAR; 64] = [0; 64];

    if RtlValidSid(Sid) == 0 {
        SetLastError(ERROR_INVALID_SID);
        return 0;
    }

    UnicodeString.Length = 0;
    UnicodeString.MaximumLength = core::mem::size_of::<[WCHAR; 64]>() as u16;
    UnicodeString.Buffer = FixedBuffer.as_mut_ptr();
    let mut LocalStatus = RtlConvertSidToUnicodeString(&mut UnicodeString, Sid, 0);
    if STATUS_BUFFER_TOO_SMALL == LocalStatus {
        LocalStatus = RtlConvertSidToUnicodeString(&mut UnicodeString, Sid, 1);
    }
    Status = LocalStatus;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    *StringSid = LocalAlloc(
        LMEM_FIXED,
        (UnicodeString.Length as usize + core::mem::size_of::<WCHAR>()) as usize,
    ) as LPWSTR;
    if (*StringSid).is_null() {
        if UnicodeString.Buffer != FixedBuffer.as_mut_ptr() {
            RtlFreeUnicodeString(&mut UnicodeString);
        }
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    MoveMemory(
        *StringSid as LPVOID,
        UnicodeString.Buffer as LPVOID,
        UnicodeString.Length as usize,
    );
    ZeroMemory(
        (*StringSid as PCHAR).add(UnicodeString.Length as usize) as LPVOID,
        core::mem::size_of::<WCHAR>() as usize,
    );
    if UnicodeString.Buffer != FixedBuffer.as_mut_ptr() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ConvertSidToStringSidA(
    Sid: PSID,
    StringSid: *mut LPSTR,
) -> BOOL {

    const CP_ACP: u32 = 0;

    let mut StringSidW: LPWSTR = core::ptr::null_mut();

    if ConvertSidToStringSidW(Sid, &mut StringSidW) == 0 {
        return 0;
    }

    let Len = WideCharToMultiByte(
        CP_ACP,
        0,
        StringSidW,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Len <= 0 {
        LocalFree(StringSidW as LPVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    *StringSid = LocalAlloc(LMEM_FIXED, Len as usize) as LPSTR;
    if (*StringSid).is_null() {
        LocalFree(StringSidW as LPVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if WideCharToMultiByte(
        CP_ACP,
        0,
        StringSidW,
        -1,
        *StringSid,
        Len,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        LocalFree(StringSid as LPVOID);
        LocalFree(StringSidW as LPVOID);
        return 0;
    }

    LocalFree(StringSidW as LPVOID);

    1
}

unsafe fn GetUnicodeEnvironmentSize(pEnvironment: PVOID) -> DWORD {
    let mut TotalLength: INT = 0;
    let mut Length: INT;

    if pEnvironment.is_null() {
        return 0;
    }

    let mut Ptr = pEnvironment as PWCHAR;
    while *Ptr != UNICODE_NULL as u16 {
        Length = wcslen(Ptr) as INT + 1;
        TotalLength += Length;
        Ptr = Ptr.offset(Length as isize);
    }

    (TotalLength as DWORD + 1) * core::mem::size_of::<WCHAR>() as DWORD
}

unsafe fn GetAnsiEnvironmentSize(pEnvironment: PVOID) -> DWORD {
    let mut Length: INT;
    let mut TotalLength: INT = 0;
    let mut Ptr: PCHAR;

    if pEnvironment.is_null() {
        return 0;
    }

    Ptr = pEnvironment as PCHAR;
    while *Ptr != 0 {
        Length = strlen(Ptr) as INT + 1;
        TotalLength += Length;
        Ptr = Ptr.offset(Length as isize);
    }

    (TotalLength + 1) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn CreateProcessWithLogonW(
    lpUsername: LPCWSTR,
    lpDomain: LPCWSTR,
    lpPassword: LPCWSTR,
    dwLogonFlags: DWORD,
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut Request: SECL_REQUEST = core::mem::zeroed();
    let mut Response: SECL_RESPONSE = core::mem::zeroed();
    let mut Status: RPC_STATUS;

    let ncacn_np: [u16; 9] = [
        b'n' as u16, b'c' as u16, b'a' as u16, b'c' as u16, b'n' as u16,
        b'_' as u16, b'n' as u16, b'p' as u16, 0,
    ];
    let pipe_seclogon: [u16; 15] = [
        b'\\' as u16, b'p' as u16, b'i' as u16, b'p' as u16, b'e' as u16,
        b'\\' as u16, b's' as u16, b'e' as u16, b'c' as u16, b'l' as u16,
        b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16, 0,
    ];

    Status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        pipe_seclogon.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if Status != RPC_S_OK as RPC_STATUS {
        SetLastError(Status as DWORD);
        return 0;
    }

    Status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if Status != RPC_S_OK as RPC_STATUS {
    }

    Status = RpcStringFreeW(&mut pszStringBinding);
    if Status != RPC_S_OK as RPC_STATUS {
    }

    Request.Username = lpUsername as LPWSTR;
    Request.Domain = lpDomain as LPWSTR;
    Request.Password = lpPassword as LPWSTR;
    Request.ApplicationName = lpApplicationName as LPWSTR;
    Request.CommandLine = lpCommandLine;
    Request.CurrentDirectory = lpCurrentDirectory as LPWSTR;

    if dwCreationFlags & CREATE_UNICODE_ENVIRONMENT != 0 {
        Request.dwEnvironmentSize = GetUnicodeEnvironmentSize(lpEnvironment);
    } else {
        Request.dwEnvironmentSize = GetAnsiEnvironmentSize(lpEnvironment);
    }
    Request.Environment = lpEnvironment as *mut u8;

    Request.dwLogonFlags = dwLogonFlags;
    Request.dwCreationFlags = dwCreationFlags;

    Request.dwProcessId = GetCurrentProcessId();

    Response.hProcess = 0;
    Response.hThread = 0;
    Response.dwProcessId = 0;
    Response.dwThreadId = 0;
    Response.dwError = ERROR_SUCCESS;

    SeclCreateProcessWithLogonW(hBinding, &mut Request, &mut Response);

    if !hBinding.is_null() {
        Status = RpcBindingFree(&mut hBinding);
        if Status != RPC_S_OK as RPC_STATUS {
        }

        hBinding = core::ptr::null_mut();
    }

    if Response.dwError != ERROR_SUCCESS {
        SetLastError(Response.dwError);
    }

    (Response.dwError == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn CreateProcessWithTokenW(
    token: HANDLE,
    logon_flags: DWORD,
    application_name: LPCWSTR,
    command_line: LPWSTR,
    creation_flags: DWORD,
    environment: *mut core::ffi::c_void,
    current_directory: LPCWSTR,
    startup_info: LPSTARTUPINFOW,
    process_information: LPPROCESS_INFORMATION,
) -> BOOL {
    CreateProcessW(
        application_name,
        command_line,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8 as BOOL,
        creation_flags,
        environment,
        current_directory,
        startup_info,
        process_information,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DuplicateTokenEx(
    ExistingTokenHandle: HANDLE,
    dwDesiredAccess: DWORD,
    lpTokenAttributes: LPSECURITY_ATTRIBUTES,
    ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL,
    TokenType_p: TOKEN_TYPE,
    DuplicateTokenHandle: PHANDLE,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut Sqos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();

    Sqos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32;
    Sqos.ImpersonationLevel = ImpersonationLevel;
    Sqos.ContextTrackingMode = 0;
    Sqos.EffectiveOnly = 0u8;

    if !lpTokenAttributes.is_null() {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = if (*lpTokenAttributes).bInheritHandle != 0 {
            OBJ_INHERIT
        } else {
            0
        } as u32;
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.SecurityDescriptor = (*lpTokenAttributes).lpSecurityDescriptor;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    } else {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    }

    ObjectAttributes.SecurityQualityOfService =
        core::ptr::addr_of_mut!(Sqos) as *mut core::ffi::c_void;

    Status = NtDuplicateToken(
        ExistingTokenHandle,
        dwDesiredAccess,
        &mut ObjectAttributes,
        0u8,
        TokenType_p,
        DuplicateTokenHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn DuplicateToken(
    ExistingTokenHandle: HANDLE,
    ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL,
    DuplicateTokenHandle: PHANDLE,
) -> BOOL {
    DuplicateTokenEx(
        ExistingTokenHandle,
        TOKEN_IMPERSONATE | TOKEN_QUERY,
        core::ptr::null_mut(),
        ImpersonationLevel,
        TokenImpersonation,
        DuplicateTokenHandle,
    )
}
unsafe fn ComputeStringSidSize(StringSid: LPCWSTR) -> DWORD {
    let mut StringSid = StringSid;
    if *StringSid.offset(0) == 'S' as u16 && *StringSid.offset(1) == '-' as u16 {
        let mut ctok: i32 = 0;
        while *StringSid != 0 {
            if *StringSid == '-' as u16 {
                ctok += 1;
            }
            StringSid = StringSid.offset(1);
        }

        if ctok >= 3 {
            return GetSidLengthRequired((ctok - 2) as UCHAR);
        }
    } else {
        let mut i: usize;

        i = 0;
        while i < WellKnownSids.len() {
            if strncmpW(WellKnownSids[i].wstr.as_ptr(), StringSid, 2) == 0 {
                return GetSidLengthRequired(WellKnownSids[i].Sid.SubAuthorityCount);
            }
            i += 1;
        }

        i = 0;
        while i < WellKnownRids.len() {
            if strncmpW(WellKnownRids[i].wstr.as_ptr(), StringSid, 2) == 0 {
                let mut local: MAX_SID = core::mem::zeroed();
                ADVAPI_GetComputerSid(core::ptr::addr_of_mut!(local) as PSID);
                return GetSidLengthRequired(
                    *GetSidSubAuthorityCount(core::ptr::addr_of_mut!(local) as PSID) + 1,
                );
            }
            i += 1;
        }
    }

    GetSidLengthRequired(0)
}
unsafe fn ParseStringSidToSid(StringSid: LPCWSTR, pSid: PSID, cBytes: LPDWORD) -> BOOL {
    let mut bret: BOOL = 0;
    let pisid: *mut SID = pSid as *mut SID;
    let mut StringSid = StringSid;

    if StringSid.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    while *StringSid == b' ' as u16 {
        StringSid = StringSid.add(1);
    }

    'lend: {
        if *StringSid == 0 {
            break 'lend;
        }

        *cBytes = ComputeStringSidSize(StringSid);
        if pisid.is_null() {
            return 1;
        }

        if *StringSid == b'S' as u16 && *StringSid.add(1) == b'-' as u16 {
            let mut i: DWORD = 0;
            let identAuth: DWORD;
            let csubauth: DWORD = (*cBytes - GetSidLengthRequired(0)) / core::mem::size_of::<DWORD>() as DWORD;

            StringSid = StringSid.add(2);
            (*pisid).Revision = atoiW(StringSid) as u8;

            if (*pisid).Revision != SDDL_REVISION as u8 {
                break 'lend;
            }
            if csubauth == 0 {
                break 'lend;
            }

            (*pisid).SubAuthorityCount = csubauth as u8;

            while *StringSid != 0 && *StringSid != b'-' as u16 {
                StringSid = StringSid.add(1);
            }
            if *StringSid == b'-' as u16 {
                StringSid = StringSid.add(1);
            }

            (*pisid).IdentifierAuthority.Value[0] = 0;
            (*pisid).IdentifierAuthority.Value[1] = 0;
            identAuth = atoiW(StringSid) as u32;
            (*pisid).IdentifierAuthority.Value[5] = (identAuth & 0xff) as u8;
            (*pisid).IdentifierAuthority.Value[4] = ((identAuth & 0xff00) >> 8) as u8;
            (*pisid).IdentifierAuthority.Value[3] = ((identAuth & 0xff0000) >> 16) as u8;
            (*pisid).IdentifierAuthority.Value[2] = ((identAuth & 0xff000000) >> 24) as u8;

            while *StringSid != 0 && *StringSid != b'-' as u16 {
                StringSid = StringSid.add(1);
            }
            if *StringSid == b'-' as u16 {
                StringSid = StringSid.add(1);
            }

            while *StringSid != 0 {
                *(*pisid).SubAuthority.as_mut_ptr().add(i as usize) = atoiW(StringSid) as u32;
                i += 1;

                while *StringSid != 0 && *StringSid != b'-' as u16 {
                    StringSid = StringSid.add(1);
                }
                if *StringSid == b'-' as u16 {
                    StringSid = StringSid.add(1);
                }
            }

            if i != (*pisid).SubAuthorityCount as DWORD {
                break 'lend;
            }

            bret = 1;
        } else {
            (*pisid).Revision = SDDL_REVISION as u8;

            let nsids = WellKnownSids.len();
            let mut idx = 0;
            while idx < nsids {
                if strncmpW(WellKnownSids[idx].wstr.as_ptr(), StringSid, 2) == 0 {
                    (*pisid).SubAuthorityCount = WellKnownSids[idx].Sid.SubAuthorityCount;
                    (*pisid).IdentifierAuthority = WellKnownSids[idx].Sid.IdentifierAuthority;
                    let mut j: DWORD = 0;
                    while j < WellKnownSids[idx].Sid.SubAuthorityCount as DWORD {
                        *(*pisid).SubAuthority.as_mut_ptr().add(j as usize) =
                            WellKnownSids[idx].Sid.SubAuthority[j as usize];
                        j += 1;
                    }
                    bret = 1;
                }
                idx += 1;
            }

            let nrids = WellKnownRids.len();
            let mut ridx = 0;
            while ridx < nrids {
                if strncmpW(WellKnownRids[ridx].wstr.as_ptr(), StringSid, 2) == 0 {
                    ADVAPI_GetComputerSid(pisid as PSID);
                    *(*pisid).SubAuthority.as_mut_ptr().add((*pisid).SubAuthorityCount as usize) =
                        WellKnownRids[ridx].Rid;
                    (*pisid).SubAuthorityCount += 1;
                    bret = 1;
                }
                ridx += 1;
            }
        }
    }

    if bret == 0 {
        SetLastError(ERROR_INVALID_SID);
    }

    bret
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedSecurityInfoA(
    pObjectName: LPSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut len: INT;
    let mut wstr: LPWSTR = core::ptr::null_mut();
    let r: DWORD;

    if !pObjectName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pObjectName, -1, core::ptr::null_mut(), 0);
        wstr = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize * core::mem::size_of::<WCHAR>()) as _,
        ) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pObjectName, -1, wstr, len);
    }

    r = GetNamedSecurityInfoW(
        wstr,
        ObjectType,
        SecurityInfo,
        ppsidOwner,
        ppsidGroup,
        ppDacl,
        ppSacl,
        ppSecurityDescriptor,
    );

    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    r
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowsAccountDomainSid(
    sid: PSID,
    domain_sid: PSID,
    size: *mut DWORD,
) -> BOOL {
    let mut domain_ident: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let required_size: DWORD;
    let mut i: i32;

    if sid.is_null() || IsValidSid(sid) == 0 {
        SetLastError(ERROR_INVALID_SID);
        return 0;
    }

    if size.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if *GetSidSubAuthorityCount(sid) < 4 {
        SetLastError(ERROR_INVALID_SID);
        return 0;
    }

    required_size = GetSidLengthRequired(4);
    if *size < required_size || domain_sid.is_null() {
        *size = required_size;
        SetLastError(if !domain_sid.is_null() {
            ERROR_INSUFFICIENT_BUFFER
        } else {
            ERROR_INVALID_PARAMETER
        });
        return 0;
    }

    InitializeSid(domain_sid, core::ptr::addr_of_mut!(domain_ident), 4);
    i = 0;
    while i < 4 {
        *GetSidSubAuthority(domain_sid, i as DWORD) =
            *GetSidSubAuthority(sid, i as DWORD);
        i += 1;
    }

    *size = required_size;
    1
}

#[no_mangle]
pub unsafe extern "system" fn EqualDomainSid(
    pSid1: PSID,
    pSid2: PSID,
    pfEqual: *mut BOOL,
) -> BOOL {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     See COPYING in the top level directory
//  * WINE COPYRIGHT:
//  * Copyright 1999, 2000 Juergen Schmied <juergen.schmied@debitel.net>
//  * Copyright 2003 CodeWeavers Inc. (Ulrich Czekalla)
//  * Copyright 2006 Robert Reif
//  * Copyright 2006 Hervé Poussineau
//  */
//
// #include <advapi32.h>
//
// #include <sddl.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// static BOOL ParseStringSidToSid(LPCWSTR StringSid, PSID pSid, LPDWORD cBytes);
// #ifdef __REACTOS__
// VOID WINAPI QuerySecurityAccessMask(SECURITY_INFORMATION,LPDWORD);
// VOID WINAPI SetSecurityAccessMask(SECURITY_INFORMATION,LPDWORD);
// #endif
//
// typedef struct _ACEFLAG
// {
//    LPCWSTR wstr;
//    DWORD value;
// } ACEFLAG, *LPACEFLAG;
//
// typedef struct _MAX_SID
// {
//     /* same fields as struct _SID */
//     BYTE Revision;
//     BYTE SubAuthorityCount;
//     SID_IDENTIFIER_AUTHORITY IdentifierAuthority;
//     DWORD SubAuthority[SID_MAX_SUB_AUTHORITIES];
// } MAX_SID;
//
// typedef struct WELLKNOWNSID
// {
//     WCHAR wstr[2];
//     WELL_KNOWN_SID_TYPE Type;
//     MAX_SID Sid;
// } WELLKNOWNSID;
//
// static const WELLKNOWNSID WellKnownSids[] =
// {
//     { {0,0}, WinNullSid, { SID_REVISION, 1, { SECURITY_NULL_SID_AUTHORITY }, { SECURITY_NULL_RID } } },
//     { {'W','D'}, WinWorldSid, { SID_REVISION, 1, { SECURITY_WORLD_SID_AUTHORITY }, { SECURITY_WORLD_RID } } },
//     { {0,0}, WinLocalSid, { SID_REVISION, 1, { SECURITY_LOCAL_SID_AUTHORITY }, { SECURITY_LOCAL_RID } } },
//     { {'C','O'}, WinCreatorOwnerSid, { SID_REVISION, 1, { SECURITY_CREATOR_SID_AUTHORITY }, { SECURITY_CREATOR_OWNER_RID } } },
//     { {'C','G'}, WinCreatorGroupSid, { SID_REVISION, 1, { SECURITY_CREATOR_SID_AUTHORITY }, { SECURITY_CREATOR_GROUP_RID } } },
//     { {0,0}, WinCreatorOwnerServerSid, { SID_REVISION, 1, { SECURITY_CREATOR_SID_AUTHORITY }, { SECURITY_CREATOR_OWNER_SERVER_RID } } },
//     { {0,0}, WinCreatorGroupServerSid, { SID_REVISION, 1, { SECURITY_CREATOR_SID_AUTHORITY }, { SECURITY_CREATOR_GROUP_SERVER_RID } } },
//     { {0,0}, WinNtAuthoritySid, { SID_REVISION, 0, { SECURITY_NT_AUTHORITY }, { SECURITY_NULL_RID } } },
//     { {0,0}, WinDialupSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_DIALUP_RID } } },
//     { {'N','U'}, WinNetworkSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_NETWORK_RID } } },
//     { {0,0}, WinBatchSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_BATCH_RID } } },
//     { {'I','U'}, WinInteractiveSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_INTERACTIVE_RID } } },
//     { {'S','U'}, WinServiceSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_SERVICE_RID } } },
//     { {'A','N'}, WinAnonymousSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_ANONYMOUS_LOGON_RID } } },
//     { {0,0}, WinProxySid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_PROXY_RID } } },
//     { {'E','D'}, WinEnterpriseControllersSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_ENTERPRISE_CONTROLLERS_RID } } },
//     { {'P','S'}, WinSelfSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_PRINCIPAL_SELF_RID } } },
//     { {'A','U'}, WinAuthenticatedUserSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_AUTHENTICATED_USER_RID } } },
//     { {'R','C'}, WinRestrictedCodeSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_RESTRICTED_CODE_RID } } },
//     { {0,0}, WinTerminalServerSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_TERMINAL_SERVER_RID } } },
//     { {0,0}, WinRemoteLogonIdSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_REMOTE_LOGON_RID } } },
//     { {'S','Y'}, WinLocalSystemSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_LOCAL_SYSTEM_RID } } },
//     { {'L','S'}, WinLocalServiceSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_LOCAL_SERVICE_RID } } },
//     { {'N','S'}, WinNetworkServiceSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_NETWORK_SERVICE_RID } } },
//     { {0,0}, WinBuiltinDomainSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID } } },
//     { {'B','A'}, WinBuiltinAdministratorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS } } },
//     { {'B','U'}, WinBuiltinUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_USERS } } },
//     { {'B','G'}, WinBuiltinGuestsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_GUESTS } } },
//     { {'P','U'}, WinBuiltinPowerUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_POWER_USERS } } },
//     { {'A','O'}, WinBuiltinAccountOperatorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ACCOUNT_OPS } } },
//     { {'S','O'}, WinBuiltinSystemOperatorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_SYSTEM_OPS } } },
//     { {'P','O'}, WinBuiltinPrintOperatorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_PRINT_OPS } } },
//     { {'B','O'}, WinBuiltinBackupOperatorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_BACKUP_OPS } } },
//     { {'R','E'}, WinBuiltinReplicatorSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_REPLICATOR } } },
//     { {'R','U'}, WinBuiltinPreWindows2000CompatibleAccessSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_PREW2KCOMPACCESS } } },
//     { {'R','D'}, WinBuiltinRemoteDesktopUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS } } },
//     { {'N','O'}, WinBuiltinNetworkConfigurationOperatorsSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS } } },
//     { {0,0}, WinNTLMAuthenticationSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_NTLM_RID } } },
//     { {0,0}, WinDigestAuthenticationSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_DIGEST_RID } } },
//     { {0,0}, WinSChannelAuthenticationSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_PACKAGE_BASE_RID, SECURITY_PACKAGE_SCHANNEL_RID } } },
//     { {0,0}, WinThisOrganizationSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_THIS_ORGANIZATION_RID } } },
//     { {0,0}, WinOtherOrganizationSid, { SID_REVISION, 1, { SECURITY_NT_AUTHORITY }, { SECURITY_OTHER_ORGANIZATION_RID } } },
//     { {0,0}, WinBuiltinIncomingForestTrustBuildersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_INCOMING_FOREST_TRUST_BUILDERS  } } },
//     { {0,0}, WinBuiltinPerfMonitoringUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_MONITORING_USERS } } },
//     { {0,0}, WinBuiltinPerfLoggingUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_LOGGING_USERS } } },
//     { {0,0}, WinBuiltinAuthorizationAccessSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_AUTHORIZATIONACCESS } } },
//     { {0,0}, WinBuiltinTerminalServerLicenseServersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_TS_LICENSE_SERVERS } } },
//     { {0,0}, WinBuiltinDCOMUsersSid, { SID_REVISION, 2, { SECURITY_NT_AUTHORITY }, { SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_DCOM_USERS } } },
//     { {'L','W'}, WinLowLabelSid, { SID_REVISION, 1, { SECURITY_MANDATORY_LABEL_AUTHORITY}, { SECURITY_MANDATORY_LOW_RID} } },
//     { {'M','E'}, WinMediumLabelSid, { SID_REVISION, 1, { SECURITY_MANDATORY_LABEL_AUTHORITY}, { SECURITY_MANDATORY_MEDIUM_RID } } },
//     { {'H','I'}, WinHighLabelSid, { SID_REVISION, 1, { SECURITY_MANDATORY_LABEL_AUTHORITY}, { SECURITY_MANDATORY_HIGH_RID } } },
//     { {'S','I'}, WinSystemLabelSid, { SID_REVISION, 1, { SECURITY_MANDATORY_LABEL_AUTHORITY}, { SECURITY_MANDATORY_SYSTEM_RID } } },
// };
//
// /* these SIDs must be constructed as relative to some domain - only the RID is well-known */
// typedef struct WELLKNOWNRID
// {
//     WCHAR wstr[2];
//     WELL_KNOWN_SID_TYPE Type;
//     DWORD Rid;
// } WELLKNOWNRID;
//
// static const WELLKNOWNRID WellKnownRids[] = {
//     { {'L','A'}, WinAccountAdministratorSid,    DOMAIN_USER_RID_ADMIN },
//     { {'L','G'}, WinAccountGuestSid,            DOMAIN_USER_RID_GUEST },
//     { {0,0}, WinAccountKrbtgtSid,           DOMAIN_USER_RID_KRBTGT },
//     { {'D','A'}, WinAccountDomainAdminsSid,     DOMAIN_GROUP_RID_ADMINS },
//     { {'D','U'}, WinAccountDomainUsersSid,      DOMAIN_GROUP_RID_USERS },
//     { {'D','G'}, WinAccountDomainGuestsSid,     DOMAIN_GROUP_RID_GUESTS },
//     { {'D','C'}, WinAccountComputersSid,        DOMAIN_GROUP_RID_COMPUTERS },
//     { {'D','D'}, WinAccountControllersSid,      DOMAIN_GROUP_RID_CONTROLLERS },
//     { {'C','A'}, WinAccountCertAdminsSid,       DOMAIN_GROUP_RID_CERT_ADMINS },
//     { {'S','A'}, WinAccountSchemaAdminsSid,     DOMAIN_GROUP_RID_SCHEMA_ADMINS },
//     { {'E','A'}, WinAccountEnterpriseAdminsSid, DOMAIN_GROUP_RID_ENTERPRISE_ADMINS },
//     { {'P','A'}, WinAccountPolicyAdminsSid,     DOMAIN_GROUP_RID_POLICY_ADMINS },
//     { {'R','S'}, WinAccountRasAndIasServersSid, DOMAIN_ALIAS_RID_RAS_SERVERS },
// };
//
// #ifndef __REACTOS__
// static const SID sidWorld = { SID_REVISION, 1, { SECURITY_WORLD_SID_AUTHORITY} , { SECURITY_WORLD_RID } };
// #endif
//
// static const WCHAR SDDL_NO_READ_UP[]       = {'N','R',0};
// static const WCHAR SDDL_NO_WRITE_UP[]      = {'N','W',0};
// static const WCHAR SDDL_NO_EXECUTE_UP[]    = {'N','X',0};
//
// /*
//  * ACE types
//  */
// static const WCHAR SDDL_ACCESS_ALLOWED[]        = {'A',0};
// static const WCHAR SDDL_ACCESS_DENIED[]         = {'D',0};
// #ifndef __REACTOS__
// static const WCHAR SDDL_OBJECT_ACCESS_ALLOWED[] = {'O','A',0};
// static const WCHAR SDDL_OBJECT_ACCESS_DENIED[]  = {'O','D',0};
// #endif
// static const WCHAR SDDL_AUDIT[]                 = {'A','U',0};
// static const WCHAR SDDL_ALARM[]                 = {'A','L',0};
// static const WCHAR SDDL_MANDATORY_LABEL[]       = {'M','L',0};
// #ifndef __REACTOS__
// static const WCHAR SDDL_OBJECT_AUDIT[]          = {'O','U',0};
// static const WCHAR SDDL_OBJECT_ALARM[]          = {'O','L',0};
// #endif
//
// /*
//  * SDDL ADS Rights
//  */
// #define ADS_RIGHT_DS_CREATE_CHILD   0x0001
// #define ADS_RIGHT_DS_DELETE_CHILD   0x0002
// #define ADS_RIGHT_ACTRL_DS_LIST     0x0004
// #define ADS_RIGHT_DS_SELF           0x0008
// #define ADS_RIGHT_DS_READ_PROP      0x0010
// #define ADS_RIGHT_DS_WRITE_PROP     0x0020
// #define ADS_RIGHT_DS_DELETE_TREE    0x0040
// #define ADS_RIGHT_DS_LIST_OBJECT    0x0080
// #define ADS_RIGHT_DS_CONTROL_ACCESS 0x0100
//
// /*
//  * ACE flags
//  */
// static const WCHAR SDDL_CONTAINER_INHERIT[]  = {'C','I',0};
// static const WCHAR SDDL_OBJECT_INHERIT[]     = {'O','I',0};
// static const WCHAR SDDL_NO_PROPAGATE[]       = {'N','P',0};
// static const WCHAR SDDL_INHERIT_ONLY[]       = {'I','O',0};
// static const WCHAR SDDL_INHERITED[]          = {'I','D',0};
// static const WCHAR SDDL_AUDIT_SUCCESS[]      = {'S','A',0};
// static const WCHAR SDDL_AUDIT_FAILURE[]      = {'F','A',0};
//
// static const char * debugstr_sid(PSID sid)
// {
//     int auth = 0;
//     SID * psid = (SID *)sid;
//
//     if (psid == NULL)
//         return "(null)";
//
//     auth = psid->IdentifierAuthority.Value[5] +
//            (psid->IdentifierAuthority.Value[4] << 8) +
//            (psid->IdentifierAuthority.Value[3] << 16) +
//            (psid->IdentifierAuthority.Value[2] << 24);
//
//     switch (psid->SubAuthorityCount) {
//     case 0:
//         return wine_dbg_sprintf("S-%d-%d", psid->Revision, auth);
//     case 1:
//         return wine_dbg_sprintf("S-%d-%d-%lu", psid->Revision, auth,
//             psid->SubAuthority[0]);
//     case 2:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1]);
//     case 3:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1], psid->SubAuthority[2]);
//     case 4:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1], psid->SubAuthority[2],
//             psid->SubAuthority[3]);
//     case 5:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1], psid->SubAuthority[2],
//             psid->SubAuthority[3], psid->SubAuthority[4]);
//     case 6:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[3], psid->SubAuthority[1], psid->SubAuthority[2],
//             psid->SubAuthority[0], psid->SubAuthority[4], psid->SubAuthority[5]);
//     case 7:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1], psid->SubAuthority[2],
//             psid->SubAuthority[3], psid->SubAuthority[4], psid->SubAuthority[5],
//             psid->SubAuthority[6]);
//     case 8:
//         return wine_dbg_sprintf("S-%d-%d-%lu-%lu-%lu-%lu-%lu-%lu-%lu-%lu", psid->Revision, auth,
//             psid->SubAuthority[0], psid->SubAuthority[1], psid->SubAuthority[2],
//             psid->SubAuthority[3], psid->SubAuthority[4], psid->SubAuthority[5],
//             psid->SubAuthority[6], psid->SubAuthority[7]);
//     }
//     return "(too-big)";
// }
//
// /* set last error code from NT status and get the proper boolean return value */
// /* used for functions that are a simple wrapper around the corresponding ntdll API */
// static __inline BOOL set_ntstatus( NTSTATUS status )
// {
//     if (!NT_SUCCESS(status)) SetLastError( RtlNtStatusToDosError( status ));
//     return NT_SUCCESS(status);
// }
//
// static LPWSTR SERV_dup( LPCSTR str )
// {
//     UINT len;
//     LPWSTR wstr;
//
//     if( !str )
//         return NULL;
//     len = MultiByteToWideChar( CP_ACP, 0, str, -1, NULL, 0 );
//     wstr = heap_alloc( len*sizeof (WCHAR) );
//     MultiByteToWideChar( CP_ACP, 0, str, -1, wstr, len );
//     return wstr;
// }
//
// /************************************************************
//  *                ADVAPI_IsLocalComputer
//  *
//  * Checks whether the server name indicates local machine.
//  */
// BOOL ADVAPI_IsLocalComputer(LPCWSTR ServerName)
// {
//     DWORD dwSize = MAX_COMPUTERNAME_LENGTH + 1;
//     BOOL Result;
//     LPWSTR buf;
//
//     if (!ServerName || !ServerName[0])
//         return TRUE;
//
//     buf = heap_alloc(dwSize * sizeof(WCHAR));
//     Result = GetComputerNameW(buf,  &dwSize);
//     if (Result && (ServerName[0] == '\\') && (ServerName[1] == '\\'))
//         ServerName += 2;
//     Result = Result && !lstrcmpW(ServerName, buf);
//     heap_free(buf);
//
//     return Result;
// }
//
// /************************************************************
//  *                ADVAPI_GetComputerSid
//  */
// BOOL ADVAPI_GetComputerSid(PSID sid)
// {
//     static const struct /* same fields as struct SID */
//     {
//         BYTE Revision;
//         BYTE SubAuthorityCount;
//         SID_IDENTIFIER_AUTHORITY IdentifierAuthority;
//         DWORD SubAuthority[4];
//     } computer_sid =
//     { SID_REVISION, 4, { SECURITY_NT_AUTHORITY }, { SECURITY_NT_NON_UNIQUE, 0, 0, 0 } };
//
//     memcpy( sid, &computer_sid, sizeof(computer_sid) );
//     return TRUE;
// }
//
// /* Exported functions */
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// OpenProcessToken(HANDLE ProcessHandle,
//                  DWORD DesiredAccess,
//                  PHANDLE TokenHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %x, %p.\n", ProcessHandle, DesiredAccess, TokenHandle);
//
//     Status = NtOpenProcessToken(ProcessHandle,
//                                 DesiredAccess,
//                                 TokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("NtOpenProcessToken failed! Status %08x\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     TRACE("Returning token %p.\n", *TokenHandle);
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * OpenThreadToken [ADVAPI32.@]
//  *
//  * Opens the access token associated with a thread handle.
//  *
//  * PARAMS
//  *   ThreadHandle  [I] Handle to process
//  *   DesiredAccess [I] Desired access to the thread
//  *   OpenAsSelf    [I] ???
//  *   TokenHandle   [O] Destination for the token handle
//  *
//  * RETURNS
//  *  Success: TRUE. TokenHandle contains the access token.
//  *  Failure: FALSE.
//  *
//  * NOTES
//  *  See NtOpenThreadToken.
//  */
// BOOL WINAPI
// OpenThreadToken( HANDLE ThreadHandle, DWORD DesiredAccess,
// 		 BOOL OpenAsSelf, HANDLE *TokenHandle)
// {
// 	return set_ntstatus( NtOpenThreadToken(ThreadHandle, DesiredAccess, OpenAsSelf, TokenHandle));
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AdjustTokenGroups(HANDLE TokenHandle,
//                   BOOL ResetToDefault,
//                   PTOKEN_GROUPS NewState,
//                   DWORD BufferLength,
//                   PTOKEN_GROUPS PreviousState,
//                   PDWORD ReturnLength)
// {
//     NTSTATUS Status;
//
//     Status = NtAdjustGroupsToken(TokenHandle,
//                                  ResetToDefault,
//                                  NewState,
//                                  BufferLength,
//                                  PreviousState,
//                                  (PULONG)ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//        SetLastError(RtlNtStatusToDosError(Status));
//        return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AdjustTokenPrivileges(HANDLE TokenHandle,
//                       BOOL DisableAllPrivileges,
//                       PTOKEN_PRIVILEGES NewState,
//                       DWORD BufferLength,
//                       PTOKEN_PRIVILEGES PreviousState,
//                       PDWORD ReturnLength)
// {
//     NTSTATUS Status;
//
//     Status = NtAdjustPrivilegesToken(TokenHandle,
//                                      DisableAllPrivileges,
//                                      NewState,
//                                      BufferLength,
//                                      PreviousState,
//                                      (PULONG)ReturnLength);
//     if (STATUS_NOT_ALL_ASSIGNED == Status)
//     {
//         SetLastError(ERROR_NOT_ALL_ASSIGNED);
//         return TRUE;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     /* AdjustTokenPrivileges is documented to do this */
//     SetLastError(ERROR_SUCCESS);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetTokenInformation(HANDLE TokenHandle,
//                     TOKEN_INFORMATION_CLASS TokenInformationClass,
//                     LPVOID TokenInformation,
//                     DWORD TokenInformationLength,
//                     PDWORD ReturnLength)
// {
//     NTSTATUS Status;
//
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenInformationClass,
//                                      TokenInformation,
//                                      TokenInformationLength,
//                                      (PULONG)ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//   return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// SetTokenInformation(HANDLE TokenHandle,
//                     TOKEN_INFORMATION_CLASS TokenInformationClass,
//                     LPVOID TokenInformation,
//                     DWORD TokenInformationLength)
// {
//     NTSTATUS Status;
//
//     Status = NtSetInformationToken(TokenHandle,
//                                    TokenInformationClass,
//                                    TokenInformation,
//                                    TokenInformationLength);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// SetThreadToken(IN PHANDLE ThreadHandle  OPTIONAL,
//                IN HANDLE TokenHandle)
// {
//     NTSTATUS Status;
//     HANDLE hThread;
//
//     hThread = (ThreadHandle != NULL) ? *ThreadHandle : NtCurrentThread();
//
//     Status = NtSetInformationThread(hThread,
//                                     ThreadImpersonationToken,
//                                     &TokenHandle,
//                                     sizeof(HANDLE));
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Creates a filtered token that is a restricted one
//  * of the regular access token. A restricted token
//  * can have disabled SIDs, deleted privileges and/or
//  * restricted SIDs added.
//  *
//  * @param[in] ExistingTokenHandle
//  * An existing handle to a token where it's to be
//  * filtered.
//  *
//  * @param[in] Flags
//  * Privilege flag options. This parameter argument influences how the token
//  * is filtered. Such parameter can be 0.
//  *
//  * @param[in] DisableSidCount
//  * The count number of SIDs to disable.
//  *
//  * @param[in] SidsToDisable
//  * An array list with SIDs that have to be disabled in
//  * a token.
//  *
//  * @param[in] DeletePrivilegeCount
//  * The count number of privileges to be deleted.
//  *
//  * @param[in] PrivilegesToDelete
//  * An array list with privileges that have to be deleted
//  * in a token.
//  *
//  * @param[in] RestrictedSidCount
//  * The count number of restricted SIDs.
//  *
//  * @param[in] SidsToRestrict
//  * An array list with restricted SIDs to be added into
//  * the token. If the token already has restricted SIDs
//  * then the array provided by the caller is redundant
//  * information alongside with the existing restricted
//  * SIDs in the token.
//  *
//  * @param[out] NewTokenHandle
//  * The newly received handle to a restricted (filtered)
//  * token. The caller can use such handle to duplicate
//  * a new token.
//  *
//  * @return
//  * Returns TRUE if the function has successfully completed
//  * the operations, otherwise FALSE is returned to indicate
//  * failure. For further details the caller has to invoke
//  * GetLastError() API call for extended information
//  * about the failure.
//  */
// BOOL WINAPI CreateRestrictedToken(
//      HANDLE ExistingTokenHandle,
//      DWORD Flags,
//      DWORD DisableSidCount,
//      PSID_AND_ATTRIBUTES SidsToDisable,
//      DWORD DeletePrivilegeCount,
//      PLUID_AND_ATTRIBUTES PrivilegesToDelete,
//      DWORD RestrictedSidCount,
//      PSID_AND_ATTRIBUTES SidsToRestrict,
//      PHANDLE NewTokenHandle)
// {
//     NTSTATUS Status;
//     BOOL Success;
//     ULONG Index;
//     PTOKEN_GROUPS DisableSids = NULL;
//     PTOKEN_GROUPS RestrictedSids = NULL;
//     PTOKEN_PRIVILEGES DeletePrivileges = NULL;
//
//     /*
//      * Capture the elements we're being given from
//      * the caller and allocate the groups and/or
//      * privileges that have to be filtered in
//      * the token.
//      */
//     if (SidsToDisable != NULL)
//     {
//         DisableSids = (PTOKEN_GROUPS)LocalAlloc(LMEM_FIXED, DisableSidCount * sizeof(TOKEN_GROUPS));
//         if (DisableSids == NULL)
//         {
//             /* We failed, bail out */
//             SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
//             return FALSE;
//         }
//
//         /* Copy the counter and loop the elements to copy the rest */
//         DisableSids->GroupCount = DisableSidCount;
//         for (Index = 0; Index < DisableSidCount; Index++)
//         {
//             DisableSids->Groups[Index].Sid = SidsToDisable[Index].Sid;
//             DisableSids->Groups[Index].Attributes = SidsToDisable[Index].Attributes;
//         }
//     }
//
//     if (PrivilegesToDelete != NULL)
//     {
//         DeletePrivileges = (PTOKEN_PRIVILEGES)LocalAlloc(LMEM_FIXED, DeletePrivilegeCount * sizeof(TOKEN_PRIVILEGES));
//         if (DeletePrivileges == NULL)
//         {
//             /* We failed, bail out */
//             SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
//             Success = FALSE;
//             goto Cleanup;
//         }
//
//         /* Copy the counter and loop the elements to copy the rest */
//         DeletePrivileges->PrivilegeCount = DeletePrivilegeCount;
//         for (Index = 0; Index < DeletePrivilegeCount; Index++)
//         {
//             DeletePrivileges->Privileges[Index].Luid = PrivilegesToDelete[Index].Luid;
//             DeletePrivileges->Privileges[Index].Attributes = PrivilegesToDelete[Index].Attributes;
//         }
//     }
//
//     if (SidsToRestrict != NULL)
//     {
//         RestrictedSids = (PTOKEN_GROUPS)LocalAlloc(LMEM_FIXED, RestrictedSidCount * sizeof(TOKEN_GROUPS));
//         if (RestrictedSids == NULL)
//         {
//             /* We failed, bail out */
//             SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES));
//             Success = FALSE;
//             goto Cleanup;
//         }
//
//         /* Copy the counter and loop the elements to copy the rest */
//         RestrictedSids->GroupCount = RestrictedSidCount;
//         for (Index = 0; Index < RestrictedSidCount; Index++)
//         {
//             RestrictedSids->Groups[Index].Sid = SidsToRestrict[Index].Sid;
//             RestrictedSids->Groups[Index].Attributes = SidsToRestrict[Index].Attributes;
//         }
//     }
//
//     /*
//      * Call the NT API to request a token filtering
//      * operation for us.
//      */
//     Status = NtFilterToken(ExistingTokenHandle,
//                            Flags,
//                            DisableSids,
//                            DeletePrivileges,
//                            RestrictedSids,
//                            NewTokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed to do the job, bail out */
//         SetLastError(RtlNtStatusToDosError(Status));
//         Success = FALSE;
//         goto Cleanup;
//     }
//
//     /* If we reach here then we've successfully filtered the token */
//     Success = TRUE;
//
// Cleanup:
//     /* Free whatever we allocated before */
//     if (DisableSids != NULL)
//     {
//         LocalFree(DisableSids);
//     }
//
//     if (DeletePrivileges != NULL)
//     {
//         LocalFree(DeletePrivileges);
//     }
//
//     if (RestrictedSids != NULL)
//     {
//         LocalFree(RestrictedSids);
//     }
//
//     return Success;
// }
//
// /******************************************************************************
//  * AllocateAndInitializeSid [ADVAPI32.@]
//  *
//  * PARAMS
//  *   pIdentifierAuthority []
//  *   nSubAuthorityCount   []
//  *   nSubAuthority0       []
//  *   nSubAuthority1       []
//  *   nSubAuthority2       []
//  *   nSubAuthority3       []
//  *   nSubAuthority4       []
//  *   nSubAuthority5       []
//  *   nSubAuthority6       []
//  *   nSubAuthority7       []
//  *   pSid                 []
//  */
// BOOL WINAPI
// AllocateAndInitializeSid( PSID_IDENTIFIER_AUTHORITY pIdentifierAuthority,
//                           BYTE nSubAuthorityCount,
//                           DWORD nSubAuthority0, DWORD nSubAuthority1,
//                           DWORD nSubAuthority2, DWORD nSubAuthority3,
//                           DWORD nSubAuthority4, DWORD nSubAuthority5,
//                           DWORD nSubAuthority6, DWORD nSubAuthority7,
//                           PSID *pSid )
// {
//     return set_ntstatus( RtlAllocateAndInitializeSid(
//                              pIdentifierAuthority, nSubAuthorityCount,
//                              nSubAuthority0, nSubAuthority1, nSubAuthority2, nSubAuthority3,
//                              nSubAuthority4, nSubAuthority5, nSubAuthority6, nSubAuthority7,
//                              pSid ));
// }
//
// /*
//  * @implemented
//  *
//  * RETURNS
//  *  Docs says this function does NOT return a value
//  *  even thou it's defined to return a PVOID...
//  */
// PVOID
// WINAPI
// FreeSid(PSID pSid)
// {
//     return RtlFreeSid(pSid);
// }
//
// /******************************************************************************
//  * CopySid [ADVAPI32.@]
//  *
//  * PARAMS
//  *   nDestinationSidLength []
//  *   pDestinationSid       []
//  *   pSourceSid            []
//  */
// BOOL WINAPI
// CopySid( DWORD nDestinationSidLength, PSID pDestinationSid, PSID pSourceSid )
// {
// 	return set_ntstatus(RtlCopySid(nDestinationSidLength, pDestinationSid, pSourceSid));
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// CreateWellKnownSid(IN WELL_KNOWN_SID_TYPE WellKnownSidType,
//                    IN PSID DomainSid  OPTIONAL,
//                    OUT PSID pSid,
//                    IN OUT DWORD* cbSid)
// {
//     unsigned int i;
//     TRACE("(%d, %s, %p, %p)\n", WellKnownSidType, debugstr_sid(DomainSid), pSid, cbSid);
//
//     if (cbSid == NULL || (DomainSid && !IsValidSid(DomainSid)))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     for (i = 0; i < sizeof(WellKnownSids)/sizeof(WellKnownSids[0]); i++) {
//         if (WellKnownSids[i].Type == WellKnownSidType) {
//             DWORD length = GetSidLengthRequired(WellKnownSids[i].Sid.SubAuthorityCount);
//
//             if (*cbSid < length)
//             {
//                 *cbSid = length;
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return FALSE;
//             }
//             if (!pSid)
//             {
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return FALSE;
//             }
//             CopyMemory(pSid, &WellKnownSids[i].Sid.Revision, length);
//             *cbSid = length;
//             return TRUE;
//         }
//     }
//
//     if (DomainSid == NULL || *GetSidSubAuthorityCount(DomainSid) == SID_MAX_SUB_AUTHORITIES)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     for (i = 0; i < sizeof(WellKnownRids)/sizeof(WellKnownRids[0]); i++)
//         if (WellKnownRids[i].Type == WellKnownSidType) {
//             UCHAR domain_subauth = *GetSidSubAuthorityCount(DomainSid);
//             DWORD domain_sid_length = GetSidLengthRequired(domain_subauth);
//             DWORD output_sid_length = GetSidLengthRequired(domain_subauth + 1);
//
//             if (*cbSid < output_sid_length)
//             {
//                 *cbSid = output_sid_length;
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return FALSE;
//             }
//             if (!pSid)
//             {
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return FALSE;
//             }
//             CopyMemory(pSid, DomainSid, domain_sid_length);
//             (*GetSidSubAuthorityCount(pSid))++;
//             (*GetSidSubAuthority(pSid, domain_subauth)) = WellKnownRids[i].Rid;
//             *cbSid = output_sid_length;
//             return TRUE;
//         }
//
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// IsWellKnownSid(IN PSID pSid,
//                IN WELL_KNOWN_SID_TYPE WellKnownSidType)
// {
//     unsigned int i;
//     TRACE("(%s, %d)\n", debugstr_sid(pSid), WellKnownSidType);
//
//     for (i = 0; i < sizeof(WellKnownSids) / sizeof(WellKnownSids[0]); i++)
//     {
//         if (WellKnownSids[i].Type == WellKnownSidType)
//         {
//             if (EqualSid(pSid, (PSID)(&WellKnownSids[i].Sid.Revision)))
//                 return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsValidSid(PSID pSid)
// {
//     return (BOOL)RtlValidSid(pSid);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EqualSid(PSID pSid1,
//          PSID pSid2)
// {
//     SetLastError(ERROR_SUCCESS);
//     return RtlEqualSid (pSid1, pSid2);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EqualPrefixSid(PSID pSid1,
//                PSID pSid2)
// {
//     return RtlEqualPrefixSid (pSid1, pSid2);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetSidLengthRequired(UCHAR nSubAuthorityCount)
// {
//     return (DWORD)RtlLengthRequiredSid(nSubAuthorityCount);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InitializeSid(PSID Sid,
//               PSID_IDENTIFIER_AUTHORITY pIdentifierAuthority,
//               BYTE nSubAuthorityCount)
// {
//     NTSTATUS Status;
//
//     Status = RtlInitializeSid(Sid,
//                               pIdentifierAuthority,
//                               nSubAuthorityCount);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// PSID_IDENTIFIER_AUTHORITY
// WINAPI
// GetSidIdentifierAuthority(PSID pSid)
// {
//     SetLastError(ERROR_SUCCESS);
//     return RtlIdentifierAuthoritySid(pSid);
// }
//
// /*
//  * @implemented
//  */
// PDWORD
// WINAPI
// GetSidSubAuthority(PSID pSid,
//                    DWORD nSubAuthority)
// {
//     SetLastError(ERROR_SUCCESS);
//     return (PDWORD)RtlSubAuthoritySid(pSid, nSubAuthority);
// }
//
// /*
//  * @implemented
//  */
// PUCHAR
// WINAPI
// GetSidSubAuthorityCount(PSID pSid)
// {
//     SetLastError(ERROR_SUCCESS);
//     return RtlSubAuthorityCountSid(pSid);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetLengthSid(PSID pSid)
// {
//     return (DWORD)RtlLengthSid(pSid);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InitializeSecurityDescriptor(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                              DWORD dwRevision)
// {
//     NTSTATUS Status;
//
//     Status = RtlCreateSecurityDescriptor(pSecurityDescriptor,
//                                          dwRevision);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MakeAbsoluteSD(PSECURITY_DESCRIPTOR pSelfRelativeSecurityDescriptor,
//                PSECURITY_DESCRIPTOR pAbsoluteSecurityDescriptor,
//                LPDWORD lpdwAbsoluteSecurityDescriptorSize,
//                PACL pDacl,
//                LPDWORD lpdwDaclSize,
//                PACL pSacl,
//                LPDWORD lpdwSaclSize,
//                PSID pOwner,
//                LPDWORD lpdwOwnerSize,
//                PSID pPrimaryGroup,
//                LPDWORD lpdwPrimaryGroupSize)
// {
//     NTSTATUS Status;
//
//     Status = RtlSelfRelativeToAbsoluteSD(pSelfRelativeSecurityDescriptor,
//                                          pAbsoluteSecurityDescriptor,
//                                          lpdwAbsoluteSecurityDescriptorSize,
//                                          pDacl,
//                                          lpdwDaclSize,
//                                          pSacl,
//                                          lpdwSaclSize,
//                                          pOwner,
//                                          lpdwOwnerSize,
//                                          pPrimaryGroup,
//                                          lpdwPrimaryGroupSize);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * GetKernelObjectSecurity [ADVAPI32.@]
//  */
// BOOL WINAPI GetKernelObjectSecurity(
//         HANDLE Handle,
//         SECURITY_INFORMATION RequestedInformation,
//         PSECURITY_DESCRIPTOR pSecurityDescriptor,
//         DWORD nLength,
//         LPDWORD lpnLengthNeeded )
// {
//     TRACE("(%p,0x%08x,%p,0x%08x,%p)\n", Handle, RequestedInformation,
//           pSecurityDescriptor, nLength, lpnLengthNeeded);
//
//     return set_ntstatus( NtQuerySecurityObject(Handle, RequestedInformation, pSecurityDescriptor,
//                                                nLength, lpnLengthNeeded ));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InitializeAcl(PACL pAcl,
//               DWORD nAclLength,
//               DWORD dwAclRevision)
// {
//     NTSTATUS Status;
//
//     Status = RtlCreateAcl(pAcl,
//                           nAclLength,
//                           dwAclRevision);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOL WINAPI ImpersonateNamedPipeClient( HANDLE hNamedPipe )
// {
//     IO_STATUS_BLOCK io_block;
//
//     TRACE("(%p)\n", hNamedPipe);
//
//     return set_ntstatus( NtFsControlFile(hNamedPipe, NULL, NULL, NULL,
//                          &io_block, FSCTL_PIPE_IMPERSONATE, NULL, 0, NULL, 0) );
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AddAccessAllowedAce(PACL pAcl,
//                     DWORD dwAceRevision,
//                     DWORD AccessMask,
//                     PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessAllowedAce(pAcl,
//                                     dwAceRevision,
//                                     AccessMask,
//                                     pSid);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AddAccessAllowedAceEx(PACL pAcl,
//                       DWORD dwAceRevision,
//                       DWORD AceFlags,
//                       DWORD AccessMask,
//                       PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessAllowedAceEx(pAcl,
//                                       dwAceRevision,
//                                       AceFlags,
//                                       AccessMask,
//                                       pSid);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AddAccessDeniedAce(PACL pAcl,
//                    DWORD dwAceRevision,
//                    DWORD AccessMask,
//                    PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessDeniedAce(pAcl,
//                                    dwAceRevision,
//                                    AccessMask,
//                                    pSid);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AddAccessDeniedAceEx(PACL pAcl,
//                      DWORD dwAceRevision,
//                      DWORD AceFlags,
//                      DWORD AccessMask,
//                      PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessDeniedAceEx(pAcl,
//                                      dwAceRevision,
//                                      AceFlags,
//                                      AccessMask,
//                                      pSid);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AddAce(PACL pAcl,
//        DWORD dwAceRevision,
//        DWORD dwStartingAceIndex,
//        LPVOID pAceList,
//        DWORD nAceListLength)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAce(pAcl,
//                        dwAceRevision,
//                        dwStartingAceIndex,
//                        pAceList,
//                        nAceListLength);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * DeleteAce [ADVAPI32.@]
//  */
// BOOL WINAPI DeleteAce(PACL pAcl, DWORD dwAceIndex)
// {
//     return set_ntstatus(RtlDeleteAce(pAcl, dwAceIndex));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindFirstFreeAce(PACL pAcl,
//                  LPVOID *pAce)
// {
//     return RtlFirstFreeAce(pAcl,
//                            (PACE*)pAce);
// }
//
// /******************************************************************************
//  * GetAce [ADVAPI32.@]
//  */
// BOOL WINAPI GetAce(PACL pAcl,DWORD dwAceIndex,LPVOID *pAce )
// {
//     return set_ntstatus(RtlGetAce(pAcl, dwAceIndex, pAce));
// }
//
// /******************************************************************************
//  * GetAclInformation [ADVAPI32.@]
//  */
// BOOL WINAPI GetAclInformation(
//   PACL pAcl,
//   LPVOID pAclInformation,
//   DWORD nAclInformationLength,
//   ACL_INFORMATION_CLASS dwAclInformationClass)
// {
//     return set_ntstatus(RtlQueryInformationAcl(pAcl, pAclInformation,
//                                                nAclInformationLength, dwAclInformationClass));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsValidAcl(PACL pAcl)
// {
//     return RtlValidAcl (pAcl);
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AllocateLocallyUniqueId(PLUID Luid)
// {
//     NTSTATUS Status;
//
//     Status = NtAllocateLocallyUniqueId (Luid);
//     if (!NT_SUCCESS (Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**********************************************************************
//  * LookupPrivilegeDisplayNameA			EXPORTED
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeDisplayNameA(LPCSTR lpSystemName,
//                             LPCSTR lpName,
//                             LPSTR lpDisplayName,
//                             LPDWORD cchDisplayName,
//                             LPDWORD lpLanguageId)
// {
//     UNICODE_STRING lpSystemNameW;
//     UNICODE_STRING lpNameW;
//     BOOL ret;
//     DWORD wLen = 0;
//
//     TRACE("%s %s %p %p %p\n", debugstr_a(lpSystemName), debugstr_a(lpName), lpName, cchDisplayName, lpLanguageId);
//
//     RtlCreateUnicodeStringFromAsciiz(&lpSystemNameW, lpSystemName);
//     RtlCreateUnicodeStringFromAsciiz(&lpNameW, lpName);
//     ret = LookupPrivilegeDisplayNameW(lpSystemNameW.Buffer, lpNameW.Buffer, NULL, &wLen, lpLanguageId);
//     if (!ret && GetLastError() == ERROR_INSUFFICIENT_BUFFER)
//     {
//         LPWSTR lpDisplayNameW = HeapAlloc(GetProcessHeap(), 0, wLen * sizeof(WCHAR));
//
//         ret = LookupPrivilegeDisplayNameW(lpSystemNameW.Buffer, lpNameW.Buffer, lpDisplayNameW,
//                                           &wLen, lpLanguageId);
//         if (ret)
//         {
//             unsigned int len = WideCharToMultiByte(CP_ACP, 0, lpDisplayNameW, -1, lpDisplayName,
//                                                    *cchDisplayName, NULL, NULL);
//
//             if (len == 0)
//             {
//                 /* WideCharToMultiByte failed */
//                 ret = FALSE;
//             }
//             else if (len > *cchDisplayName)
//             {
//                 *cchDisplayName = len;
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 ret = FALSE;
//             }
//             else
//             {
//                 /* WideCharToMultiByte succeeded, output length needs to be
//                  * length not including NULL terminator
//                  */
//                 *cchDisplayName = len - 1;
//             }
//         }
//         HeapFree(GetProcessHeap(), 0, lpDisplayNameW);
//     }
//     RtlFreeUnicodeString(&lpSystemNameW);
//     RtlFreeUnicodeString(&lpNameW);
//     return ret;
// }
//
// /**********************************************************************
//  * LookupPrivilegeNameA				EXPORTED
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeNameA(LPCSTR lpSystemName,
//                      PLUID lpLuid,
//                      LPSTR lpName,
//                      LPDWORD cchName)
// {
//     UNICODE_STRING lpSystemNameW;
//     BOOL ret;
//     DWORD wLen = 0;
//
//     TRACE("%s %p %p %p\n", debugstr_a(lpSystemName), lpLuid, lpName, cchName);
//
//     RtlCreateUnicodeStringFromAsciiz(&lpSystemNameW, lpSystemName);
//     ret = LookupPrivilegeNameW(lpSystemNameW.Buffer, lpLuid, NULL, &wLen);
//     if (!ret && GetLastError() == ERROR_INSUFFICIENT_BUFFER)
//     {
//         LPWSTR lpNameW = HeapAlloc(GetProcessHeap(), 0, wLen * sizeof(WCHAR));
//
//         ret = LookupPrivilegeNameW(lpSystemNameW.Buffer, lpLuid, lpNameW,
//          &wLen);
//         if (ret)
//         {
//             /* Windows crashes if cchName is NULL, so will I */
//             unsigned int len = WideCharToMultiByte(CP_ACP, 0, lpNameW, -1, lpName,
//              *cchName, NULL, NULL);
//
//             if (len == 0)
//             {
//                 /* WideCharToMultiByte failed */
//                 ret = FALSE;
//             }
//             else if (len > *cchName)
//             {
//                 *cchName = len;
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 ret = FALSE;
//             }
//             else
//             {
//                 /* WideCharToMultiByte succeeded, output length needs to be
//                  * length not including NULL terminator
//                  */
//                 *cchName = len - 1;
//             }
//         }
//         HeapFree(GetProcessHeap(), 0, lpNameW);
//     }
//     RtlFreeUnicodeString(&lpSystemNameW);
//     return ret;
// }
//
// /******************************************************************************
//  * GetFileSecurityA [ADVAPI32.@]
//  *
//  * Obtains Specified information about the security of a file or directory.
//  *
//  * PARAMS
//  *  lpFileName           [I] Name of the file to get info for
//  *  RequestedInformation [I] SE_ flags from "winnt.h"
//  *  pSecurityDescriptor  [O] Destination for security information
//  *  nLength              [I] Length of pSecurityDescriptor
//  *  lpnLengthNeeded      [O] Destination for length of returned security information
//  *
//  * RETURNS
//  *  Success: TRUE. pSecurityDescriptor contains the requested information.
//  *  Failure: FALSE. lpnLengthNeeded contains the required space to return the info.
//  *
//  * NOTES
//  *  The information returned is constrained by the callers access rights and
//  *  privileges.
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// GetFileSecurityA(LPCSTR lpFileName,
//                  SECURITY_INFORMATION RequestedInformation,
//                  PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                  DWORD nLength,
//                  LPDWORD lpnLengthNeeded)
// {
//     UNICODE_STRING FileName;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFileName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = GetFileSecurityW(FileName.Buffer,
//                                RequestedInformation,
//                                pSecurityDescriptor,
//                                nLength,
//                                lpnLengthNeeded);
//
//     RtlFreeUnicodeString(&FileName);
//
//     return bResult;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetFileSecurityW(LPCWSTR lpFileName,
//                  SECURITY_INFORMATION RequestedInformation,
//                  PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                  DWORD nLength,
//                  LPDWORD lpnLengthNeeded)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK StatusBlock;
//     UNICODE_STRING FileName;
//     ULONG AccessMask = 0;
//     HANDLE FileHandle;
//     NTSTATUS Status;
//
//     TRACE("GetFileSecurityW() called\n");
//
//     QuerySecurityAccessMask(RequestedInformation, &AccessMask);
//
//     if (!RtlDosPathNameToNtPathName_U(lpFileName,
//                                       &FileName,
//                                       NULL,
//                                       NULL))
//     {
//         ERR("Invalid path\n");
//         SetLastError(ERROR_INVALID_NAME);
//         return FALSE;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenFile(&FileHandle,
//                         AccessMask,
//                         &ObjectAttributes,
//                         &StatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         0);
//
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtOpenFile() failed (Status %lx)\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     Status = NtQuerySecurityObject(FileHandle,
//                                    RequestedInformation,
//                                    pSecurityDescriptor,
//                                    nLength,
//                                    lpnLengthNeeded);
//     NtClose(FileHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtQuerySecurityObject() failed (Status %lx)\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * SetFileSecurityA [ADVAPI32.@]
//  * Sets the security of a file or directory
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileSecurityA(LPCSTR lpFileName,
//                  SECURITY_INFORMATION SecurityInformation,
//                  PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     UNICODE_STRING FileName;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFileName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = SetFileSecurityW(FileName.Buffer,
//                                SecurityInformation,
//                                pSecurityDescriptor);
//
//     RtlFreeUnicodeString(&FileName);
//
//     return bResult;
// }
//
// /******************************************************************************
//  * SetFileSecurityW [ADVAPI32.@]
//  * Sets the security of a file or directory
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileSecurityW(LPCWSTR lpFileName,
//                  SECURITY_INFORMATION SecurityInformation,
//                  PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK StatusBlock;
//     UNICODE_STRING FileName;
//     ULONG AccessMask = 0;
//     HANDLE FileHandle;
//     NTSTATUS Status;
//
//     TRACE("SetFileSecurityW() called\n");
//
//     SetSecurityAccessMask(SecurityInformation, &AccessMask);
//
//     if (!RtlDosPathNameToNtPathName_U(lpFileName,
//                                       &FileName,
//                                       NULL,
//                                       NULL))
//     {
//         ERR("Invalid path\n");
//         SetLastError(ERROR_INVALID_NAME);
//         return FALSE;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenFile(&FileHandle,
//                         AccessMask,
//                         &ObjectAttributes,
//                         &StatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         0);
//
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtOpenFile() failed (Status %lx)\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     Status = NtSetSecurityObject(FileHandle,
//                                  SecurityInformation,
//                                  pSecurityDescriptor);
//     NtClose(FileHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtSetSecurityObject() failed (Status %lx)\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * QueryWindows31FilesMigration [ADVAPI32.@]
//  *
//  * PARAMS
//  *   x1 []
//  */
// BOOL WINAPI
// QueryWindows31FilesMigration( DWORD x1 )
// {
// 	FIXME("(%d):stub\n",x1);
// 	return TRUE;
// }
//
// /******************************************************************************
//  * SynchronizeWindows31FilesAndWindowsNTRegistry [ADVAPI32.@]
//  *
//  * PARAMS
//  *   x1 []
//  *   x2 []
//  *   x3 []
//  *   x4 []
//  */
// BOOL WINAPI
// SynchronizeWindows31FilesAndWindowsNTRegistry( DWORD x1, DWORD x2, DWORD x3,
//                                                DWORD x4 )
// {
// 	FIXME("(0x%08x,0x%08x,0x%08x,0x%08x):stub\n",x1,x2,x3,x4);
// 	return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RevertToSelf(VOID)
// {
//     NTSTATUS Status;
//     HANDLE Token = NULL;
//
//     Status = NtSetInformationThread(NtCurrentThread(),
//                                     ThreadImpersonationToken,
//                                     &Token,
//                                     sizeof(HANDLE));
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ImpersonateSelf(SECURITY_IMPERSONATION_LEVEL ImpersonationLevel)
// {
//     NTSTATUS Status;
//
//     Status = RtlImpersonateSelf(ImpersonationLevel);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AccessCheck(IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//             IN HANDLE ClientToken,
//             IN DWORD DesiredAccess,
//             IN PGENERIC_MAPPING GenericMapping,
//             OUT PPRIVILEGE_SET PrivilegeSet OPTIONAL,
//             IN OUT LPDWORD PrivilegeSetLength,
//             OUT LPDWORD GrantedAccess,
//             OUT LPBOOL AccessStatus)
// {
//     NTSTATUS Status;
//     NTSTATUS NtAccessStatus;
//
//     /* Do the access check */
//     Status = NtAccessCheck(pSecurityDescriptor,
//                            ClientToken,
//                            DesiredAccess,
//                            GenericMapping,
//                            PrivilegeSet,
//                            (PULONG)PrivilegeSetLength,
//                            (PACCESS_MASK)GrantedAccess,
//                            &NtAccessStatus);
//
//     /* See if the access check operation succeeded */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Check failed */
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     /* Now check the access status  */
//     if (!NT_SUCCESS(NtAccessStatus))
//     {
//         /* Access denied */
//         SetLastError(RtlNtStatusToDosError(NtAccessStatus));
//         *AccessStatus = FALSE;
//     }
//     else
//     {
//         /* Access granted */
//         *AccessStatus = TRUE;
//     }
//
//     /* Check succeeded */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Determines whether security access can be granted to a client
//  * that requests such access on the object type list. The access
//  * is either granted or denied for the whole object hierarchy
//  * in the list.
//  *
//  * @param[in] pSecurityDescriptor
//  * A pointer to a security descriptor that identifies the security
//  * information of an object being accessed. This function walks
//  * through this descriptor for any ACLs and respective access
//  * rights if access can be granted.
//  *
//  * @param[in] PrincipalSelfSid
//  * A pointer to a principal self SID. This parameter can be NULL if
//  * the associated object being checked for access does not represent
//  * a principal.
//  *
//  * @param[in] ClientToken
//  * A handle to an access token, that identifies the client of which
//  * requests access to the target object.
//  *
//  * @param[in] DesiredAccess
//  * The access right bitmask where the client wants to acquire. This
//  * can be an OR'ed set of multiple access rights or MAXIMUM_ALLOWED
//  * to request all of possible access rights the target object allows.
//  * If only some rights were granted but not all the access is deemed
//  * as denied.
//  *
//  * @param[in] ObjectTypeList
//  * A pointer to a given object type list. If this parameter is not NULL
//  * the function will perform an access check against the main object
//  * and sub-objects of this list. If this parameter is NULL and
//  * ObjectTypeListLength is 0, the function will perform a normal
//  * access check instead.
//  *
//  * @param[in] ObjectTypeListLength
//  * The length of the object type list array, pointed by ObjectTypeList.
//  * This length in question represents the number of elements in such array.
//  * This parameter must be 0 if no array list is provided.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access rights of an object type.
//  *
//  * @param[out] PrivilegeSet
//  * A pointer to a set of privileges that were used to perform the
//  * access check, returned to caller. This function will return no
//  * privileges (privilege count set to 0) if no privileges were used
//  * to accomplish the access check. This parameter must not be NULL!
//  *
//  * @param[in,out] PrivilegeSetLength
//  * The total length size of a set of privileges. This length represents
//  * the count of elements in the privilege set array.
//  *
//  * @param[out] GrantedAccess
//  * A pointer to granted access rights, returned to the caller.
//  *
//  * @param[out] AccessStatus
//  * A pointer to a boolean value that indicates whether access is granted
//  * or denied to the client that requests access to the entire hierarchy
//  * of an object type list. If ObjectTypeList is NULL, this value represents
//  * the access that is granted or denied to the target object, just like
//  * in AccessCheck.
//  *
//  * @return
//  * The function returns TRUE if the access check operation has completed
//  * successfully, otherwise it returns FALSE.
//  */
// BOOL
// WINAPI
// AccessCheckByType(
//      PSECURITY_DESCRIPTOR pSecurityDescriptor,
//      PSID PrincipalSelfSid,
//      HANDLE ClientToken,
//      DWORD DesiredAccess,
//      POBJECT_TYPE_LIST ObjectTypeList,
//      DWORD ObjectTypeListLength,
//      PGENERIC_MAPPING GenericMapping,
//      PPRIVILEGE_SET PrivilegeSet,
//      LPDWORD PrivilegeSetLength,
//      LPDWORD GrantedAccess,
//      LPBOOL AccessStatus)
// {
// 	NTSTATUS Status;
//     NTSTATUS NtAccessStatus;
//
// 	Status = NtAccessCheckByType(pSecurityDescriptor,
//                                  PrincipalSelfSid,
//                                  ClientToken,
//                                  DesiredAccess,
//                                  ObjectTypeList,
//                                  ObjectTypeListLength,
//                                  GenericMapping,
//                                  PrivilegeSet,
//                                  PrivilegeSetLength,
//                                  (PACCESS_MASK)GrantedAccess,
//                                  &NtAccessStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     if (!NT_SUCCESS(NtAccessStatus))
//     {
//         SetLastError(RtlNtStatusToDosError(NtAccessStatus));
//         *AccessStatus = FALSE;
//     }
//     else
//     {
//         *AccessStatus = TRUE;
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Determines whether security access can be granted to a client
//  * that requests such access on the object type list. Unlike the
//  * AccessCheckByType variant, this function will grant or deny
//  * access to each individual object and sub-object in the list.
//  *
//  * @param[in] pSecurityDescriptor
//  * A pointer to a security descriptor that identifies the security
//  * information of an object being accessed. This function walks
//  * through this descriptor for any ACLs and respective access
//  * rights if access can be granted.
//  *
//  * @param[in] PrincipalSelfSid
//  * A pointer to a principal self SID. This parameter can be NULL if
//  * the associated object being checked for access does not represent
//  * a principal.
//  *
//  * @param[in] ClientToken
//  * A handle to an access token, that identifies the client of which
//  * requests access to the target object.
//  *
//  * @param[in] DesiredAccess
//  * The access right bitmask where the client wants to acquire. This
//  * can be an OR'ed set of multiple access rights or MAXIMUM_ALLOWED
//  * to request all of possible access rights the target object allows.
//  * If only some rights were granted but not all the access is deemed
//  * as denied.
//  *
//  * @param[in] ObjectTypeList
//  * A pointer to a given object type list. This function will perform an
//  * access check against the main object and sub-objects of this list.
//  * This parameter must not be NULL!
//  *
//  * @param[in] ObjectTypeListLength
//  * The length of the object type list array, pointed by ObjectTypeList.
//  * This length in question represents the number of elements in such array.
//  * This parameter must be 0 if no array list is provided.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access rights of an object type.
//  *
//  * @param[out] PrivilegeSet
//  * A pointer to a set of privileges that were used to perform the
//  * access check, returned to caller. This function will return no
//  * privileges (privilege count set to 0) if no privileges were used
//  * to accomplish the access check. This parameter must not be NULL!
//  *
//  * @param[in,out] PrivilegeSetLength
//  * The total length size of a set of privileges. This length represents
//  * the count of elements in the privilege set array.
//  *
//  * @param[out] GrantedAccess
//  * A pointer to granted access rights. This parameter is an array of granted
//  * rights for the object and each sub-object of an object type list.
//  *
//  * @param[out] AccessStatus
//  * A pointer to a boolean value that indicates whether access is granted
//  * or denied to the client that requests access to the object and sub-objects
//  * of an object type list. This parameter is an array of boolean values for the
//  * object and each individual sub-object of the list.
//  *
//  * @return
//  * The function returns TRUE if the access check operation has completed
//  * successfully, otherwise it returns FALSE.
//  */
// BOOL
// WINAPI
// AccessCheckByTypeResultList(
//      PSECURITY_DESCRIPTOR pSecurityDescriptor,
//      PSID PrincipalSelfSid,
//      HANDLE ClientToken,
//      DWORD DesiredAccess,
//      POBJECT_TYPE_LIST ObjectTypeList,
//      DWORD ObjectTypeListLength,
//      PGENERIC_MAPPING GenericMapping,
//      PPRIVILEGE_SET PrivilegeSet,
//      LPDWORD PrivilegeSetLength,
//      LPDWORD GrantedAccess,
//      LPBOOL AccessStatus)
// {
//     NTSTATUS Status;
//     DWORD ResultListIndex;
//     PNTSTATUS NtAccessStatus = NULL;
//
//     Status = NtAccessCheckByTypeResultList(pSecurityDescriptor,
//                                            PrincipalSelfSid,
//                                            ClientToken,
//                                            DesiredAccess,
//                                            ObjectTypeList,
//                                            ObjectTypeListLength,
//                                            GenericMapping,
//                                            PrivilegeSet,
//                                            PrivilegeSetLength,
//                                            (PACCESS_MASK)GrantedAccess,
//                                            NtAccessStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     for (ResultListIndex = 0; ResultListIndex < ObjectTypeListLength; ResultListIndex++)
//     {
//         AccessStatus[ResultListIndex] = RtlNtStatusToDosError(NtAccessStatus[ResultListIndex]);
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetKernelObjectSecurity(HANDLE Handle,
//                         SECURITY_INFORMATION SecurityInformation,
//                         PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     NTSTATUS Status;
//
//     Status = NtSetSecurityObject(Handle,
//                                  SecurityInformation,
//                                  SecurityDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AddAuditAccessAce(PACL pAcl,
//                   DWORD dwAceRevision,
//                   DWORD dwAccessMask,
//                   PSID pSid,
//                   BOOL bAuditSuccess,
//                   BOOL bAuditFailure)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAuditAccessAce(pAcl,
//                                   dwAceRevision,
//                                   dwAccessMask,
//                                   pSid,
//                                   bAuditSuccess,
//                                   bAuditFailure);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AddAuditAccessAceEx(PACL pAcl,
//                     DWORD dwAceRevision,
//                     DWORD AceFlags,
//                     DWORD dwAccessMask,
//                     PSID pSid,
//                     BOOL bAuditSuccess,
//                     BOOL bAuditFailure)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAuditAccessAceEx(pAcl,
//                                     dwAceRevision,
//                                     AceFlags,
//                                     dwAccessMask,
//                                     pSid,
//                                     bAuditSuccess,
//                                     bAuditFailure);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * LookupAccountNameA [ADVAPI32.@]
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupAccountNameA(LPCSTR SystemName,
//                    LPCSTR AccountName,
//                    PSID Sid,
//                    LPDWORD SidLength,
//                    LPSTR ReferencedDomainName,
//                    LPDWORD hReferencedDomainNameLength,
//                    PSID_NAME_USE SidNameUse)
// {
//     BOOL ret;
//     UNICODE_STRING lpSystemW;
//     UNICODE_STRING lpAccountW;
//     LPWSTR lpReferencedDomainNameW = NULL;
//
//     RtlCreateUnicodeStringFromAsciiz(&lpSystemW, SystemName);
//     RtlCreateUnicodeStringFromAsciiz(&lpAccountW, AccountName);
//
//     if (ReferencedDomainName)
//         lpReferencedDomainNameW = HeapAlloc(GetProcessHeap(),
//                                             0,
//                                             *hReferencedDomainNameLength * sizeof(WCHAR));
//
//     ret = LookupAccountNameW(lpSystemW.Buffer,
//                              lpAccountW.Buffer,
//                              Sid,
//                              SidLength,
//                              lpReferencedDomainNameW,
//                              hReferencedDomainNameLength,
//                              SidNameUse);
//
//     if (ret && lpReferencedDomainNameW)
//     {
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpReferencedDomainNameW,
//                             *hReferencedDomainNameLength + 1,
//                             ReferencedDomainName,
//                             *hReferencedDomainNameLength + 1,
//                             NULL,
//                             NULL);
//     }
//
//     RtlFreeUnicodeString(&lpSystemW);
//     RtlFreeUnicodeString(&lpAccountW);
//     HeapFree(GetProcessHeap(), 0, lpReferencedDomainNameW);
//
//     return ret;
// }
//
// /**********************************************************************
//  *	PrivilegeCheck					EXPORTED
//  *
//  * @implemented
//  */
// BOOL WINAPI
// PrivilegeCheck(HANDLE ClientToken,
//                PPRIVILEGE_SET RequiredPrivileges,
//                LPBOOL pfResult)
// {
//     BOOLEAN Result;
//     NTSTATUS Status;
//
//     Status = NtPrivilegeCheck(ClientToken,
//                               RequiredPrivileges,
//                               &Result);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     *pfResult = (BOOL)Result;
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * GetSecurityInfoExW         EXPORTED
//  */
// DWORD
// WINAPI
// GetSecurityInfoExA(HANDLE hObject,
//                    SE_OBJECT_TYPE ObjectType,
//                    SECURITY_INFORMATION SecurityInfo,
//                    LPCSTR lpProvider,
//                    LPCSTR lpProperty,
//                    PACTRL_ACCESSA *ppAccessList,
//                    PACTRL_AUDITA *ppAuditList,
//                    LPSTR *lppOwner,
//                    LPSTR *lppGroup)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_BAD_PROVIDER;
// }
//
//
// /******************************************************************************
//  * GetSecurityInfoExW         EXPORTED
//  */
// DWORD
// WINAPI
// GetSecurityInfoExW(HANDLE hObject,
//                    SE_OBJECT_TYPE ObjectType,
//                    SECURITY_INFORMATION SecurityInfo,
//                    LPCWSTR lpProvider,
//                    LPCWSTR lpProperty,
//                    PACTRL_ACCESSW *ppAccessList,
//                    PACTRL_AUDITW *ppAuditList,
//                    LPWSTR *lppOwner,
//                    LPWSTR *lppGroup)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_BAD_PROVIDER;
// }
//
// /******************************************************************************
//  * BuildExplicitAccessWithNameA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildExplicitAccessWithNameA(PEXPLICIT_ACCESSA pExplicitAccess,
//                              LPSTR pTrusteeName,
//                              DWORD AccessPermissions,
//                              ACCESS_MODE AccessMode,
//                              DWORD Inheritance)
// {
//     pExplicitAccess->grfAccessPermissions = AccessPermissions;
//     pExplicitAccess->grfAccessMode = AccessMode;
//     pExplicitAccess->grfInheritance = Inheritance;
//
//     pExplicitAccess->Trustee.pMultipleTrustee = NULL;
//     pExplicitAccess->Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pExplicitAccess->Trustee.TrusteeForm = TRUSTEE_IS_NAME;
//     pExplicitAccess->Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pExplicitAccess->Trustee.ptstrName = pTrusteeName;
// }
//
//
// /******************************************************************************
//  * BuildExplicitAccessWithNameW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildExplicitAccessWithNameW(PEXPLICIT_ACCESSW pExplicitAccess,
//                              LPWSTR pTrusteeName,
//                              DWORD AccessPermissions,
//                              ACCESS_MODE AccessMode,
//                              DWORD Inheritance)
// {
//     pExplicitAccess->grfAccessPermissions = AccessPermissions;
//     pExplicitAccess->grfAccessMode = AccessMode;
//     pExplicitAccess->grfInheritance = Inheritance;
//
//     pExplicitAccess->Trustee.pMultipleTrustee = NULL;
//     pExplicitAccess->Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pExplicitAccess->Trustee.TrusteeForm = TRUSTEE_IS_NAME;
//     pExplicitAccess->Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pExplicitAccess->Trustee.ptstrName = pTrusteeName;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithObjectsAndNameA [ADVAPI32.@]
//  */
// VOID WINAPI BuildTrusteeWithObjectsAndNameA( PTRUSTEEA pTrustee, POBJECTS_AND_NAME_A pObjName,
//                                              SE_OBJECT_TYPE ObjectType, LPSTR ObjectTypeName,
//                                              LPSTR InheritedObjectTypeName, LPSTR Name )
// {
//     DWORD ObjectsPresent = 0;
//
//     TRACE("%p %p 0x%08x %p %p %s\n", pTrustee, pObjName,
//           ObjectType, ObjectTypeName, InheritedObjectTypeName, debugstr_a(Name));
//
//     /* Fill the OBJECTS_AND_NAME structure */
//     pObjName->ObjectType = ObjectType;
//     if (ObjectTypeName != NULL)
//     {
//         ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
//     }
//
//     pObjName->InheritedObjectTypeName = InheritedObjectTypeName;
//     if (InheritedObjectTypeName != NULL)
//     {
//         ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
//     }
//
//     pObjName->ObjectsPresent = ObjectsPresent;
//     pObjName->ptstrName = Name;
//
//     /* Fill the TRUSTEE structure */
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_OBJECTS_AND_NAME;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPSTR)pObjName;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithObjectsAndNameW [ADVAPI32.@]
//  */
// VOID WINAPI BuildTrusteeWithObjectsAndNameW( PTRUSTEEW pTrustee, POBJECTS_AND_NAME_W pObjName,
//                                              SE_OBJECT_TYPE ObjectType, LPWSTR ObjectTypeName,
//                                              LPWSTR InheritedObjectTypeName, LPWSTR Name )
// {
//     DWORD ObjectsPresent = 0;
//
//     TRACE("%p %p 0x%08x %p %p %s\n", pTrustee, pObjName,
//           ObjectType, ObjectTypeName, InheritedObjectTypeName, debugstr_w(Name));
//
//     /* Fill the OBJECTS_AND_NAME structure */
//     pObjName->ObjectType = ObjectType;
//     if (ObjectTypeName != NULL)
//     {
//         ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
//     }
//
//     pObjName->InheritedObjectTypeName = InheritedObjectTypeName;
//     if (InheritedObjectTypeName != NULL)
//     {
//         ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
//     }
//
//     pObjName->ObjectsPresent = ObjectsPresent;
//     pObjName->ptstrName = Name;
//
//     /* Fill the TRUSTEE structure */
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_OBJECTS_AND_NAME;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPWSTR)pObjName;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithObjectsAndSidA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithObjectsAndSidA(PTRUSTEEA pTrustee,
//                                POBJECTS_AND_SID pObjSid,
//                                GUID *pObjectGuid,
//                                GUID *pInheritedObjectGuid,
//                                PSID pSid)
// {
//     DWORD ObjectsPresent = 0;
//
//     TRACE("%p %p %p %p %p\n", pTrustee, pObjSid, pObjectGuid, pInheritedObjectGuid, pSid);
//
//     /* Fill the OBJECTS_AND_SID structure */
//     if (pObjectGuid != NULL)
//     {
//         pObjSid->ObjectTypeGuid = *pObjectGuid;
//         ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
//     }
//     else
//     {
//         ZeroMemory(&pObjSid->ObjectTypeGuid,
//                    sizeof(GUID));
//     }
//
//     if (pInheritedObjectGuid != NULL)
//     {
//         pObjSid->InheritedObjectTypeGuid = *pInheritedObjectGuid;
//         ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
//     }
//     else
//     {
//         ZeroMemory(&pObjSid->InheritedObjectTypeGuid,
//                    sizeof(GUID));
//     }
//
//     pObjSid->ObjectsPresent = ObjectsPresent;
//     pObjSid->pSid = pSid;
//
//     /* Fill the TRUSTEE structure */
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_OBJECTS_AND_SID;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPSTR) pObjSid;
// }
//
//
// /******************************************************************************
//  * BuildTrusteeWithObjectsAndSidW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithObjectsAndSidW(PTRUSTEEW pTrustee,
//                                POBJECTS_AND_SID pObjSid,
//                                GUID *pObjectGuid,
//                                GUID *pInheritedObjectGuid,
//                                PSID pSid)
// {
//     DWORD ObjectsPresent = 0;
//
//     TRACE("%p %p %p %p %p\n", pTrustee, pObjSid, pObjectGuid, pInheritedObjectGuid, pSid);
//
//     /* Fill the OBJECTS_AND_SID structure */
//     if (pObjectGuid != NULL)
//     {
//         pObjSid->ObjectTypeGuid = *pObjectGuid;
//         ObjectsPresent |= ACE_OBJECT_TYPE_PRESENT;
//     }
//     else
//     {
//         ZeroMemory(&pObjSid->ObjectTypeGuid,
//                    sizeof(GUID));
//     }
//
//     if (pInheritedObjectGuid != NULL)
//     {
//         pObjSid->InheritedObjectTypeGuid = *pInheritedObjectGuid;
//         ObjectsPresent |= ACE_INHERITED_OBJECT_TYPE_PRESENT;
//     }
//     else
//     {
//         ZeroMemory(&pObjSid->InheritedObjectTypeGuid,
//                    sizeof(GUID));
//     }
//
//     pObjSid->ObjectsPresent = ObjectsPresent;
//     pObjSid->pSid = pSid;
//
//     /* Fill the TRUSTEE structure */
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_OBJECTS_AND_SID;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPWSTR) pObjSid;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithSidA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithSidA(PTRUSTEE_A pTrustee,
//                      PSID pSid)
// {
//     TRACE("%p %p\n", pTrustee, pSid);
//
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_SID;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPSTR) pSid;
// }
//
//
// /******************************************************************************
//  * BuildTrusteeWithSidW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithSidW(PTRUSTEE_W pTrustee,
//                      PSID pSid)
// {
//     TRACE("%p %p\n", pTrustee, pSid);
//
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_SID;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = (LPWSTR) pSid;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithNameA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithNameA(PTRUSTEE_A pTrustee,
//                       LPSTR name)
// {
//     TRACE("%p %s\n", pTrustee, name);
//
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_NAME;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = name;
// }
//
// /******************************************************************************
//  * BuildTrusteeWithNameW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildTrusteeWithNameW(PTRUSTEE_W pTrustee,
//                       LPWSTR name)
// {
//     TRACE("%p %s\n", pTrustee, name);
//
//     pTrustee->pMultipleTrustee = NULL;
//     pTrustee->MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
//     pTrustee->TrusteeForm = TRUSTEE_IS_NAME;
//     pTrustee->TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pTrustee->ptstrName = name;
// }
//
// /******************************************************************************
//  * GetTrusteeFormA [ADVAPI32.@]
//  */
// TRUSTEE_FORM WINAPI GetTrusteeFormA(PTRUSTEEA pTrustee)
// {
//     TRACE("(%p)\n", pTrustee);
//
//     if (!pTrustee)
//         return TRUSTEE_BAD_FORM;
//
//     return pTrustee->TrusteeForm;
// }
//
// /******************************************************************************
//  * GetTrusteeFormW [ADVAPI32.@]
//  */
// TRUSTEE_FORM WINAPI GetTrusteeFormW(PTRUSTEEW pTrustee)
// {
//     TRACE("(%p)\n", pTrustee);
//
//     if (!pTrustee)
//         return TRUSTEE_BAD_FORM;
//
//     return pTrustee->TrusteeForm;
// }
//
// /******************************************************************************
//  * GetTrusteeNameA [ADVAPI32.@]
//  */
// LPSTR WINAPI
// GetTrusteeNameA(PTRUSTEE_A pTrustee)
// {
//     return pTrustee->ptstrName;
// }
//
//
// /******************************************************************************
//  * GetTrusteeNameW [ADVAPI32.@]
//  */
// LPWSTR WINAPI
// GetTrusteeNameW(PTRUSTEE_W pTrustee)
// {
//     return pTrustee->ptstrName;
// }
//
// /******************************************************************************
//  * GetTrusteeTypeA [ADVAPI32.@]
//  */
// TRUSTEE_TYPE WINAPI
// GetTrusteeTypeA(PTRUSTEE_A pTrustee)
// {
//     return pTrustee->TrusteeType;
// }
//
// /******************************************************************************
//  * GetTrusteeTypeW [ADVAPI32.@]
//  */
// TRUSTEE_TYPE WINAPI
// GetTrusteeTypeW(PTRUSTEE_W pTrustee)
// {
//     return pTrustee->TrusteeType;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetAclInformation(PACL pAcl,
//                   LPVOID pAclInformation,
//                   DWORD nAclInformationLength,
//                   ACL_INFORMATION_CLASS dwAclInformationClass)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetInformationAcl(pAcl,
//                                   pAclInformation,
//                                   nAclInformationLength,
//                                   dwAclInformationClass);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**********************************************************************
//  * SetNamedSecurityInfoA			EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// SetNamedSecurityInfoA(LPSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       PSID psidOwner,
//                       PSID psidGroup,
//                       PACL pDacl,
//                       PACL pSacl)
// {
//     UNICODE_STRING ObjectName;
//     DWORD Ret;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ObjectName, pObjectName))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Ret = SetNamedSecurityInfoW(ObjectName.Buffer,
//                                 ObjectType,
//                                 SecurityInfo,
//                                 psidOwner,
//                                 psidGroup,
//                                 pDacl,
//                                 pSacl);
//
//     RtlFreeUnicodeString(&ObjectName);
//
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AreAllAccessesGranted(DWORD GrantedAccess,
//                       DWORD DesiredAccess)
// {
//     return (BOOL)RtlAreAllAccessesGranted(GrantedAccess,
//                                           DesiredAccess);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AreAnyAccessesGranted(DWORD GrantedAccess,
//                       DWORD DesiredAccess)
// {
//     return (BOOL)RtlAreAnyAccessesGranted(GrantedAccess,
//                                           DesiredAccess);
// }
//
// /******************************************************************************
//  * ParseAclStringFlags
//  */
// static DWORD ParseAclStringFlags(LPCWSTR* StringAcl)
// {
//     DWORD flags = 0;
//     LPCWSTR szAcl = *StringAcl;
//
//     while (*szAcl && *szAcl != '(')
//     {
//         if (*szAcl == 'P')
//         {
//             flags |= SE_DACL_PROTECTED;
//         }
//         else if (*szAcl == 'A')
//         {
//             szAcl++;
//             if (*szAcl == 'R')
//                 flags |= SE_DACL_AUTO_INHERIT_REQ;
//             else if (*szAcl == 'I')
//                 flags |= SE_DACL_AUTO_INHERITED;
//         }
//         szAcl++;
//     }
//
//     *StringAcl = szAcl;
//     return flags;
// }
//
// /******************************************************************************
//  * ParseAceStringType
//  */
// static const ACEFLAG AceType[] =
// {
//     { SDDL_ALARM,          SYSTEM_ALARM_ACE_TYPE },
//     { SDDL_AUDIT,          SYSTEM_AUDIT_ACE_TYPE },
//     { SDDL_ACCESS_ALLOWED, ACCESS_ALLOWED_ACE_TYPE },
//     { SDDL_ACCESS_DENIED,  ACCESS_DENIED_ACE_TYPE },
//     { SDDL_MANDATORY_LABEL,SYSTEM_MANDATORY_LABEL_ACE_TYPE },
//     /*
//     { SDDL_OBJECT_ACCESS_ALLOWED, ACCESS_ALLOWED_OBJECT_ACE_TYPE },
//     { SDDL_OBJECT_ACCESS_DENIED,  ACCESS_DENIED_OBJECT_ACE_TYPE },
//     { SDDL_OBJECT_ALARM,          SYSTEM_ALARM_OBJECT_ACE_TYPE },
//     { SDDL_OBJECT_AUDIT,          SYSTEM_AUDIT_OBJECT_ACE_TYPE },
//     */
//     { NULL, 0 },
// };
//
// static BYTE ParseAceStringType(LPCWSTR* StringAcl)
// {
//     UINT len = 0;
//     LPCWSTR szAcl = *StringAcl;
//     const ACEFLAG *lpaf = AceType;
//
//     while (*szAcl == ' ')
//         szAcl++;
//
//     while (lpaf->wstr &&
//         (len = strlenW(lpaf->wstr)) &&
//         strncmpW(lpaf->wstr, szAcl, len))
//         lpaf++;
//
//     if (!lpaf->wstr)
//         return 0;
//
//     *StringAcl = szAcl + len;
//     return lpaf->value;
// }
//
//
// /******************************************************************************
//  * ParseAceStringFlags
//  */
// static const ACEFLAG AceFlags[] =
// {
//     { SDDL_CONTAINER_INHERIT, CONTAINER_INHERIT_ACE },
//     { SDDL_AUDIT_FAILURE,     FAILED_ACCESS_ACE_FLAG },
//     { SDDL_INHERITED,         INHERITED_ACE },
//     { SDDL_INHERIT_ONLY,      INHERIT_ONLY_ACE },
//     { SDDL_NO_PROPAGATE,      NO_PROPAGATE_INHERIT_ACE },
//     { SDDL_OBJECT_INHERIT,    OBJECT_INHERIT_ACE },
//     { SDDL_AUDIT_SUCCESS,     SUCCESSFUL_ACCESS_ACE_FLAG },
//     { NULL, 0 },
// };
//
// static BYTE ParseAceStringFlags(LPCWSTR* StringAcl)
// {
//     UINT len = 0;
//     BYTE flags = 0;
//     LPCWSTR szAcl = *StringAcl;
//
//     while (*szAcl == ' ')
//         szAcl++;
//
//     while (*szAcl != ';')
//     {
//         const ACEFLAG *lpaf = AceFlags;
//
//         while (lpaf->wstr &&
//                (len = strlenW(lpaf->wstr)) &&
//                strncmpW(lpaf->wstr, szAcl, len))
//             lpaf++;
//
//         if (!lpaf->wstr)
//             return 0;
//
// 	flags |= lpaf->value;
//         szAcl += len;
//     }
//
//     *StringAcl = szAcl;
//     return flags;
// }
//
//
// /******************************************************************************
//  * ParseAceStringRights
//  */
// static const ACEFLAG AceRights[] =
// {
//     { SDDL_GENERIC_ALL,     GENERIC_ALL },
//     { SDDL_GENERIC_READ,    GENERIC_READ },
//     { SDDL_GENERIC_WRITE,   GENERIC_WRITE },
//     { SDDL_GENERIC_EXECUTE, GENERIC_EXECUTE },
//
//     { SDDL_READ_CONTROL,    READ_CONTROL },
//     { SDDL_STANDARD_DELETE, DELETE },
//     { SDDL_WRITE_DAC,       WRITE_DAC },
//     { SDDL_WRITE_OWNER,     WRITE_OWNER },
//
//     { SDDL_READ_PROPERTY,   ADS_RIGHT_DS_READ_PROP},
//     { SDDL_WRITE_PROPERTY,  ADS_RIGHT_DS_WRITE_PROP},
//     { SDDL_CREATE_CHILD,    ADS_RIGHT_DS_CREATE_CHILD},
//     { SDDL_DELETE_CHILD,    ADS_RIGHT_DS_DELETE_CHILD},
//     { SDDL_LIST_CHILDREN,   ADS_RIGHT_ACTRL_DS_LIST},
//     { SDDL_SELF_WRITE,      ADS_RIGHT_DS_SELF},
//     { SDDL_LIST_OBJECT,     ADS_RIGHT_DS_LIST_OBJECT},
//     { SDDL_DELETE_TREE,     ADS_RIGHT_DS_DELETE_TREE},
//     { SDDL_CONTROL_ACCESS,  ADS_RIGHT_DS_CONTROL_ACCESS},
//
//     { SDDL_FILE_ALL,        FILE_ALL_ACCESS },
//     { SDDL_FILE_READ,       FILE_GENERIC_READ },
//     { SDDL_FILE_WRITE,      FILE_GENERIC_WRITE },
//     { SDDL_FILE_EXECUTE,    FILE_GENERIC_EXECUTE },
//
//     { SDDL_KEY_ALL,         KEY_ALL_ACCESS },
//     { SDDL_KEY_READ,        KEY_READ },
//     { SDDL_KEY_WRITE,       KEY_WRITE },
//     { SDDL_KEY_EXECUTE,     KEY_EXECUTE },
//
//     { SDDL_NO_READ_UP,      SYSTEM_MANDATORY_LABEL_NO_READ_UP },
//     { SDDL_NO_WRITE_UP,     SYSTEM_MANDATORY_LABEL_NO_WRITE_UP },
//     { SDDL_NO_EXECUTE_UP,   SYSTEM_MANDATORY_LABEL_NO_EXECUTE_UP },
//     { NULL, 0 },
// };
//
// static DWORD ParseAceStringRights(LPCWSTR* StringAcl)
// {
//     UINT len = 0;
//     DWORD rights = 0;
//     LPCWSTR szAcl = *StringAcl;
//
//     while (*szAcl == ' ')
//         szAcl++;
//
//     if ((*szAcl == '0') && (*(szAcl + 1) == 'x'))
//     {
//         LPCWSTR p = szAcl;
//
// 	while (*p && *p != ';')
//             p++;
//
// 	if (p - szAcl <= 10 /* 8 hex digits + "0x" */ )
// 	{
// 	    rights = strtoulW(szAcl, NULL, 16);
// 	    szAcl = p;
// 	}
// 	else
//             WARN("Invalid rights string format: %s\n", debugstr_wn(szAcl, p - szAcl));
//     }
//     else
//     {
//         while (*szAcl != ';')
//         {
//             const ACEFLAG *lpaf = AceRights;
//
//             while (lpaf->wstr &&
//                (len = strlenW(lpaf->wstr)) &&
//                strncmpW(lpaf->wstr, szAcl, len))
// 	    {
//                lpaf++;
// 	    }
//
//             if (!lpaf->wstr)
//                 return 0;
//
// 	    rights |= lpaf->value;
//             szAcl += len;
//         }
//     }
//
//     *StringAcl = szAcl;
//     return rights;
// }
//
//
// /******************************************************************************
//  * ParseStringAclToAcl
//  *
//  * dacl_flags(string_ace1)(string_ace2)... (string_acen)
//  */
// static BOOL ParseStringAclToAcl(LPCWSTR StringAcl, LPDWORD lpdwFlags,
//     PACL pAcl, LPDWORD cBytes)
// {
//     DWORD val;
//     DWORD sidlen;
//     DWORD length = sizeof(ACL);
//     DWORD acesize = 0;
//     DWORD acecount = 0;
//     PACCESS_ALLOWED_ACE pAce = NULL; /* pointer to current ACE */
//     DWORD error = ERROR_INVALID_ACL;
//
//     TRACE("%s\n", debugstr_w(StringAcl));
//
//     if (!StringAcl)
// 	return FALSE;
//
//     if (pAcl) /* pAce is only useful if we're setting values */
//         pAce = (PACCESS_ALLOWED_ACE) (pAcl + 1);
//
//     /* Parse ACL flags */
//     *lpdwFlags = ParseAclStringFlags(&StringAcl);
//
//     /* Parse ACE */
//     while (*StringAcl == '(')
//     {
//         StringAcl++;
//
//         /* Parse ACE type */
//         val = ParseAceStringType(&StringAcl);
// 	if (pAce)
//             pAce->Header.AceType = (BYTE) val;
//         if (*StringAcl != ';')
//         {
//             error = RPC_S_INVALID_STRING_UUID;
//             goto lerr;
//         }
//         StringAcl++;
//
//         /* Parse ACE flags */
// 	val = ParseAceStringFlags(&StringAcl);
// 	if (pAce)
//             pAce->Header.AceFlags = (BYTE) val;
//         if (*StringAcl != ';')
//             goto lerr;
//         StringAcl++;
//
//         /* Parse ACE rights */
// 	val = ParseAceStringRights(&StringAcl);
// 	if (pAce)
//             pAce->Mask = val;
//         if (*StringAcl != ';')
//             goto lerr;
//         StringAcl++;
//
//         /* Parse ACE object guid */
//         while (*StringAcl == ' ')
//             StringAcl++;
//         if (*StringAcl != ';')
//         {
//             FIXME("Support for *_OBJECT_ACE_TYPE not implemented\n");
//             goto lerr;
//         }
//         StringAcl++;
//
//         /* Parse ACE inherit object guid */
//         while (*StringAcl == ' ')
//             StringAcl++;
//         if (*StringAcl != ';')
//         {
//             FIXME("Support for *_OBJECT_ACE_TYPE not implemented\n");
//             goto lerr;
//         }
//         StringAcl++;
//
//         /* Parse ACE account sid */
//         if (ParseStringSidToSid(StringAcl, pAce ? &pAce->SidStart : NULL, &sidlen))
// 	{
//             while (*StringAcl && *StringAcl != ')')
//                 StringAcl++;
// 	}
//
//         if (*StringAcl != ')')
//             goto lerr;
//         StringAcl++;
//
//         acesize = sizeof(ACCESS_ALLOWED_ACE) - sizeof(DWORD) + sidlen;
//         length += acesize;
//         if (pAce)
//         {
//             pAce->Header.AceSize = acesize;
//             pAce = (PACCESS_ALLOWED_ACE)((LPBYTE)pAce + acesize);
//         }
//         acecount++;
//     }
//
//     *cBytes = length;
//
//     if (length > 0xffff)
//     {
//         ERR("ACL too large\n");
//         goto lerr;
//     }
//
//     if (pAcl)
//     {
//         pAcl->AclRevision = ACL_REVISION;
//         pAcl->Sbz1 = 0;
//         pAcl->AclSize = length;
//         pAcl->AceCount = acecount;
//         pAcl->Sbz2 = 0;
//     }
//     return TRUE;
//
// lerr:
//     SetLastError(error);
//     WARN("Invalid ACE string format\n");
//     return FALSE;
// }
//
// /******************************************************************************
//  * ParseStringSecurityDescriptorToSecurityDescriptor
//  */
// static BOOL ParseStringSecurityDescriptorToSecurityDescriptor(
//     LPCWSTR StringSecurityDescriptor,
//     SECURITY_DESCRIPTOR_RELATIVE* SecurityDescriptor,
//     LPDWORD cBytes)
// {
//     BOOL bret = FALSE;
//     WCHAR toktype;
//     WCHAR *tok;
//     LPCWSTR lptoken;
//     LPBYTE lpNext = NULL;
//     DWORD len;
//
//     *cBytes = sizeof(SECURITY_DESCRIPTOR_RELATIVE);
//
//     tok = heap_alloc( (lstrlenW(StringSecurityDescriptor) + 1) * sizeof(WCHAR));
//
//     if (SecurityDescriptor)
//         lpNext = (LPBYTE)(SecurityDescriptor + 1);
//
//     while (*StringSecurityDescriptor == ' ')
//         StringSecurityDescriptor++;
//
//     while (*StringSecurityDescriptor)
//     {
//         toktype = *StringSecurityDescriptor;
//
// 	/* Expect char identifier followed by ':' */
// 	StringSecurityDescriptor++;
//         if (*StringSecurityDescriptor != ':')
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             goto lend;
//         }
// 	StringSecurityDescriptor++;
//
// 	/* Extract token */
// 	lptoken = StringSecurityDescriptor;
// 	while (*lptoken && *lptoken != ':')
//             lptoken++;
//
// 	if (*lptoken)
//             lptoken--;
//
//         len = lptoken - StringSecurityDescriptor;
//         memcpy( tok, StringSecurityDescriptor, len * sizeof(WCHAR) );
//         tok[len] = 0;
//
//         switch (toktype)
// 	{
//             case 'O':
//             {
//                 DWORD bytes;
//
//                 if (!ParseStringSidToSid(tok, lpNext, &bytes))
//                     goto lend;
//
//                 if (SecurityDescriptor)
//                 {
//                     SecurityDescriptor->Owner = lpNext - (LPBYTE)SecurityDescriptor;
//                     lpNext += bytes; /* Advance to next token */
//                 }
//
// 		*cBytes += bytes;
//
//                 break;
//             }
//
//             case 'G':
//             {
//                 DWORD bytes;
//
//                 if (!ParseStringSidToSid(tok, lpNext, &bytes))
//                     goto lend;
//
//                 if (SecurityDescriptor)
//                 {
//                     SecurityDescriptor->Group = lpNext - (LPBYTE)SecurityDescriptor;
//                     lpNext += bytes; /* Advance to next token */
//                 }
//
// 		*cBytes += bytes;
//
//                 break;
//             }
//
//             case 'D':
// 	    {
//                 DWORD flags;
//                 DWORD bytes;
//
//                 if (!ParseStringAclToAcl(tok, &flags, (PACL)lpNext, &bytes))
//                     goto lend;
//
//                 if (SecurityDescriptor)
//                 {
//                     SecurityDescriptor->Control |= SE_DACL_PRESENT | flags;
//                     SecurityDescriptor->Dacl = lpNext - (LPBYTE)SecurityDescriptor;
//                     lpNext += bytes; /* Advance to next token */
// 		}
//
// 		*cBytes += bytes;
//
// 		break;
//             }
//
//             case 'S':
//             {
//                 DWORD flags;
//                 DWORD bytes;
//
//                 if (!ParseStringAclToAcl(tok, &flags, (PACL)lpNext, &bytes))
//                     goto lend;
//
//                 if (SecurityDescriptor)
//                 {
//                     SecurityDescriptor->Control |= SE_SACL_PRESENT | flags;
//                     SecurityDescriptor->Sacl = lpNext - (LPBYTE)SecurityDescriptor;
//                     lpNext += bytes; /* Advance to next token */
// 		}
//
// 		*cBytes += bytes;
//
// 		break;
//             }
//
//             default:
//                 FIXME("Unknown token\n");
//                 SetLastError(ERROR_INVALID_PARAMETER);
// 		goto lend;
// 	}
//
//         StringSecurityDescriptor = lptoken;
//     }
//
//     bret = TRUE;
//
// lend:
//     heap_free(tok);
//     return bret;
// }
//
// /* Winehq cvs 20050916 */
// /******************************************************************************
//  * ConvertStringSecurityDescriptorToSecurityDescriptorA [ADVAPI32.@]
//  * @implemented
//  */
// BOOL
// WINAPI
// ConvertStringSecurityDescriptorToSecurityDescriptorA(LPCSTR StringSecurityDescriptor,
//                                                      DWORD StringSDRevision,
//                                                      PSECURITY_DESCRIPTOR* SecurityDescriptor,
//                                                      PULONG SecurityDescriptorSize)
// {
//     UINT len;
//     BOOL ret = FALSE;
//     LPWSTR StringSecurityDescriptorW;
//
//     len = MultiByteToWideChar(CP_ACP, 0, StringSecurityDescriptor, -1, NULL, 0);
//     StringSecurityDescriptorW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//
//     if (StringSecurityDescriptorW)
//     {
//         MultiByteToWideChar(CP_ACP, 0, StringSecurityDescriptor, -1, StringSecurityDescriptorW, len);
//
//         ret = ConvertStringSecurityDescriptorToSecurityDescriptorW(StringSecurityDescriptorW,
//                                                                    StringSDRevision, SecurityDescriptor,
//                                                                    SecurityDescriptorSize);
//         HeapFree(GetProcessHeap(), 0, StringSecurityDescriptorW);
//     }
//
//     return ret;
// }
//
// /******************************************************************************
//  * ConvertStringSecurityDescriptorToSecurityDescriptorW [ADVAPI32.@]
//  * @implemented
//  */
// BOOL WINAPI
// ConvertStringSecurityDescriptorToSecurityDescriptorW(LPCWSTR StringSecurityDescriptor,
//                                                      DWORD StringSDRevision,
//                                                      PSECURITY_DESCRIPTOR* SecurityDescriptor,
//                                                      PULONG SecurityDescriptorSize)
// {
//     DWORD cBytes;
//     SECURITY_DESCRIPTOR* psd;
//     BOOL bret = FALSE;
//
//     TRACE("%s\n", debugstr_w(StringSecurityDescriptor));
//
//     if (GetVersion() & 0x80000000)
//     {
//         SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//         goto lend;
//     }
//     else if (!StringSecurityDescriptor || !SecurityDescriptor)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         goto lend;
//     }
//     else if (StringSDRevision != SID_REVISION)
//     {
//         SetLastError(ERROR_UNKNOWN_REVISION);
//         goto lend;
//     }
//
//     /* Compute security descriptor length */
//     if (!ParseStringSecurityDescriptorToSecurityDescriptor(StringSecurityDescriptor,
//         NULL, &cBytes))
//         goto lend;
//
//     psd = *SecurityDescriptor = LocalAlloc(GMEM_ZEROINIT, cBytes);
//     if (!psd) goto lend;
//
//     psd->Revision = SID_REVISION;
//     psd->Control |= SE_SELF_RELATIVE;
//
//     if (!ParseStringSecurityDescriptorToSecurityDescriptor(StringSecurityDescriptor,
//              (SECURITY_DESCRIPTOR_RELATIVE *)psd, &cBytes))
//     {
//         LocalFree(psd);
//         goto lend;
//     }
//
//     if (SecurityDescriptorSize)
//         *SecurityDescriptorSize = cBytes;
//
//     bret = TRUE;
//
// lend:
//     TRACE(" ret=%d\n", bret);
//     return bret;
// }
//
// static void DumpString(LPCWSTR string, int cch, WCHAR **pwptr, ULONG *plen)
// {
//     if (cch == -1)
//         cch = strlenW(string);
//
//     if (plen)
//         *plen += cch;
//
//     if (pwptr)
//     {
//         memcpy(*pwptr, string, sizeof(WCHAR)*cch);
//         *pwptr += cch;
//     }
// }
//
// static BOOL DumpSidNumeric(PSID psid, WCHAR **pwptr, ULONG *plen)
// {
//     DWORD i;
//     WCHAR fmt[] = { 'S','-','%','u','-','%','d',0 };
//     WCHAR subauthfmt[] = { '-','%','u',0 };
//     WCHAR buf[26];
//     SID *pisid = psid;
//
//     if( !IsValidSid( psid ) || pisid->Revision != SDDL_REVISION)
//     {
//         SetLastError(ERROR_INVALID_SID);
//         return FALSE;
//     }
//
//     if (pisid->IdentifierAuthority.Value[0] ||
//      pisid->IdentifierAuthority.Value[1])
//     {
//         FIXME("not matching MS' bugs\n");
//         SetLastError(ERROR_INVALID_SID);
//         return FALSE;
//     }
//
//     sprintfW( buf, fmt, pisid->Revision,
//         MAKELONG(
//             MAKEWORD( pisid->IdentifierAuthority.Value[5],
//                     pisid->IdentifierAuthority.Value[4] ),
//             MAKEWORD( pisid->IdentifierAuthority.Value[3],
//                     pisid->IdentifierAuthority.Value[2] )
//         ) );
//     DumpString(buf, -1, pwptr, plen);
//
//     for( i=0; i<pisid->SubAuthorityCount; i++ )
//     {
//         sprintfW( buf, subauthfmt, pisid->SubAuthority[i] );
//         DumpString(buf, -1, pwptr, plen);
//     }
//     return TRUE;
// }
//
// static BOOL DumpSid(PSID psid, WCHAR **pwptr, ULONG *plen)
// {
//     size_t i;
//     for (i = 0; i < sizeof(WellKnownSids) / sizeof(WellKnownSids[0]); i++)
//     {
//         if (WellKnownSids[i].wstr[0] && EqualSid(psid, (PSID)&(WellKnownSids[i].Sid.Revision)))
//         {
//             DumpString(WellKnownSids[i].wstr, 2, pwptr, plen);
//             return TRUE;
//         }
//     }
//
//     return DumpSidNumeric(psid, pwptr, plen);
// }
//
// static const LPCWSTR AceRightBitNames[32] = {
//         SDDL_CREATE_CHILD,        /*  0 */
//         SDDL_DELETE_CHILD,
//         SDDL_LIST_CHILDREN,
//         SDDL_SELF_WRITE,
//         SDDL_READ_PROPERTY,       /*  4 */
//         SDDL_WRITE_PROPERTY,
//         SDDL_DELETE_TREE,
//         SDDL_LIST_OBJECT,
//         SDDL_CONTROL_ACCESS,      /*  8 */
//         NULL,
//         NULL,
//         NULL,
//         NULL,                     /* 12 */
//         NULL,
//         NULL,
//         NULL,
//         SDDL_STANDARD_DELETE,     /* 16 */
//         SDDL_READ_CONTROL,
//         SDDL_WRITE_DAC,
//         SDDL_WRITE_OWNER,
//         NULL,                     /* 20 */
//         NULL,
//         NULL,
//         NULL,
//         NULL,                     /* 24 */
//         NULL,
//         NULL,
//         NULL,
//         SDDL_GENERIC_ALL,         /* 28 */
//         SDDL_GENERIC_EXECUTE,
//         SDDL_GENERIC_WRITE,
//         SDDL_GENERIC_READ
// };
//
// static void DumpRights(DWORD mask, WCHAR **pwptr, ULONG *plen)
// {
//     static const WCHAR fmtW[] = {'0','x','%','x',0};
//     WCHAR buf[15];
//     size_t i;
//
//     if (mask == 0)
//         return;
//
//     /* first check if the right have name */
//     for (i = 0; i < sizeof(AceRights)/sizeof(AceRights[0]); i++)
//     {
//         if (AceRights[i].wstr == NULL)
//             break;
//         if (mask == AceRights[i].value)
//         {
//             DumpString(AceRights[i].wstr, -1, pwptr, plen);
//             return;
//         }
//     }
//
//     /* then check if it can be built from bit names */
//     for (i = 0; i < 32; i++)
//     {
//         if ((mask & (1 << i)) && (AceRightBitNames.0[i] == NULL))
//         {
//             /* can't be built from bit names */
//             sprintfW(buf, fmtW, mask);
//             DumpString(buf, -1, pwptr, plen);
//             return;
//         }
//     }
//
//     /* build from bit names */
//     for (i = 0; i < 32; i++)
//         if (mask & (1 << i))
//             DumpString(AceRightBitNames.0[i], -1, pwptr, plen);
// }
//
// static BOOL DumpAce(LPVOID pace, WCHAR **pwptr, ULONG *plen)
// {
//     ACCESS_ALLOWED_ACE *piace; /* all the supported ACEs have the same memory layout */
//     static const WCHAR openbr = '(';
//     static const WCHAR closebr = ')';
//     static const WCHAR semicolon = ';';
//
//     if (((PACE_HEADER)pace)->AceType > SYSTEM_ALARM_ACE_TYPE || ((PACE_HEADER)pace)->AceSize < sizeof(ACCESS_ALLOWED_ACE))
//     {
//         SetLastError(ERROR_INVALID_ACL);
//         return FALSE;
//     }
//
//     piace = pace;
//     DumpString(&openbr, 1, pwptr, plen);
//     switch (piace->Header.AceType)
//     {
//         case ACCESS_ALLOWED_ACE_TYPE:
//             DumpString(SDDL_ACCESS_ALLOWED, -1, pwptr, plen);
//             break;
//         case ACCESS_DENIED_ACE_TYPE:
//             DumpString(SDDL_ACCESS_DENIED, -1, pwptr, plen);
//             break;
//         case SYSTEM_AUDIT_ACE_TYPE:
//             DumpString(SDDL_AUDIT, -1, pwptr, plen);
//             break;
//         case SYSTEM_ALARM_ACE_TYPE:
//             DumpString(SDDL_ALARM, -1, pwptr, plen);
//             break;
//     }
//     DumpString(&semicolon, 1, pwptr, plen);
//
//     if (piace->Header.AceFlags & OBJECT_INHERIT_ACE)
//         DumpString(SDDL_OBJECT_INHERIT, -1, pwptr, plen);
//     if (piace->Header.AceFlags & CONTAINER_INHERIT_ACE)
//         DumpString(SDDL_CONTAINER_INHERIT, -1, pwptr, plen);
//     if (piace->Header.AceFlags & NO_PROPAGATE_INHERIT_ACE)
//         DumpString(SDDL_NO_PROPAGATE, -1, pwptr, plen);
//     if (piace->Header.AceFlags & INHERIT_ONLY_ACE)
//         DumpString(SDDL_INHERIT_ONLY, -1, pwptr, plen);
//     if (piace->Header.AceFlags & INHERITED_ACE)
//         DumpString(SDDL_INHERITED, -1, pwptr, plen);
//     if (piace->Header.AceFlags & SUCCESSFUL_ACCESS_ACE_FLAG)
//         DumpString(SDDL_AUDIT_SUCCESS, -1, pwptr, plen);
//     if (piace->Header.AceFlags & FAILED_ACCESS_ACE_FLAG)
//         DumpString(SDDL_AUDIT_FAILURE, -1, pwptr, plen);
//     DumpString(&semicolon, 1, pwptr, plen);
//     DumpRights(piace->Mask, pwptr, plen);
//     DumpString(&semicolon, 1, pwptr, plen);
//     /* objects not supported */
//     DumpString(&semicolon, 1, pwptr, plen);
//     /* objects not supported */
//     DumpString(&semicolon, 1, pwptr, plen);
//     if (!DumpSid((PSID)&piace->SidStart, pwptr, plen))
//         return FALSE;
//     DumpString(&closebr, 1, pwptr, plen);
//     return TRUE;
// }
//
// static BOOL DumpAcl(PACL pacl, WCHAR **pwptr, ULONG *plen, BOOL protected, BOOL autoInheritReq, BOOL autoInherited)
// {
//     WORD count;
//     int i;
//
//     if (protected)
//         DumpString(SDDL_PROTECTED, -1, pwptr, plen);
//     if (autoInheritReq)
//         DumpString(SDDL_AUTO_INHERIT_REQ, -1, pwptr, plen);
//     if (autoInherited)
//         DumpString(SDDL_AUTO_INHERITED, -1, pwptr, plen);
//
//     if (pacl == NULL)
//         return TRUE;
//
//     if (!IsValidAcl(pacl))
//         return FALSE;
//
//     count = pacl->AceCount;
//     for (i = 0; i < count; i++)
//     {
//         LPVOID ace;
//         if (!GetAce(pacl, i, &ace))
//             return FALSE;
//         if (!DumpAce(ace, pwptr, plen))
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// static BOOL DumpOwner(PSECURITY_DESCRIPTOR SecurityDescriptor, WCHAR **pwptr, ULONG *plen)
// {
//     static const WCHAR prefix[] = {'O',':',0};
//     BOOL bDefaulted;
//     PSID psid;
//
//     if (!GetSecurityDescriptorOwner(SecurityDescriptor, &psid, &bDefaulted))
//         return FALSE;
//
//     if (psid == NULL)
//         return TRUE;
//
//     DumpString(prefix, -1, pwptr, plen);
//     if (!DumpSid(psid, pwptr, plen))
//         return FALSE;
//     return TRUE;
// }
//
// static BOOL DumpGroup(PSECURITY_DESCRIPTOR SecurityDescriptor, WCHAR **pwptr, ULONG *plen)
// {
//     static const WCHAR prefix[] = {'G',':',0};
//     BOOL bDefaulted;
//     PSID psid;
//
//     if (!GetSecurityDescriptorGroup(SecurityDescriptor, &psid, &bDefaulted))
//         return FALSE;
//
//     if (psid == NULL)
//         return TRUE;
//
//     DumpString(prefix, -1, pwptr, plen);
//     if (!DumpSid(psid, pwptr, plen))
//         return FALSE;
//     return TRUE;
// }
//
// static BOOL DumpDacl(PSECURITY_DESCRIPTOR SecurityDescriptor, WCHAR **pwptr, ULONG *plen)
// {
//     static const WCHAR dacl[] = {'D',':',0};
//     SECURITY_DESCRIPTOR_CONTROL control;
//     BOOL present, defaulted;
//     DWORD revision;
//     PACL pacl;
//
//     if (!GetSecurityDescriptorDacl(SecurityDescriptor, &present, &pacl, &defaulted))
//         return FALSE;
//
//     if (!GetSecurityDescriptorControl(SecurityDescriptor, &control, &revision))
//         return FALSE;
//
//     if (!present)
//         return TRUE;
//
//     DumpString(dacl, 2, pwptr, plen);
//     if (!DumpAcl(pacl, pwptr, plen, control & SE_DACL_PROTECTED, control & SE_DACL_AUTO_INHERIT_REQ, control & SE_DACL_AUTO_INHERITED))
//         return FALSE;
//     return TRUE;
// }
//
// static BOOL DumpSacl(PSECURITY_DESCRIPTOR SecurityDescriptor, WCHAR **pwptr, ULONG *plen)
// {
//     static const WCHAR sacl[] = {'S',':',0};
//     SECURITY_DESCRIPTOR_CONTROL control;
//     BOOL present, defaulted;
//     DWORD revision;
//     PACL pacl;
//
//     if (!GetSecurityDescriptorSacl(SecurityDescriptor, &present, &pacl, &defaulted))
//         return FALSE;
//
//     if (!GetSecurityDescriptorControl(SecurityDescriptor, &control, &revision))
//         return FALSE;
//
//     if (!present)
//         return TRUE;
//
//     DumpString(sacl, 2, pwptr, plen);
//     if (!DumpAcl(pacl, pwptr, plen, control & SE_SACL_PROTECTED, control & SE_SACL_AUTO_INHERIT_REQ, control & SE_SACL_AUTO_INHERITED))
//         return FALSE;
//     return TRUE;
// }
//
// /******************************************************************************
//  * ConvertSecurityDescriptorToStringSecurityDescriptorW [ADVAPI32.@]
//  */
// BOOL WINAPI ConvertSecurityDescriptorToStringSecurityDescriptorW(PSECURITY_DESCRIPTOR SecurityDescriptor, DWORD SDRevision, SECURITY_INFORMATION RequestedInformation, LPWSTR *OutputString, PULONG OutputLen)
// {
//     ULONG len;
//     WCHAR *wptr, *wstr;
//
//     if (SDRevision != SDDL_REVISION_1)
//     {
//         ERR("Program requested unknown SDDL revision %d\n", SDRevision);
//         SetLastError(ERROR_UNKNOWN_REVISION);
//         return FALSE;
//     }
//
//     len = 0;
//     if (RequestedInformation & OWNER_SECURITY_INFORMATION)
//         if (!DumpOwner(SecurityDescriptor, NULL, &len))
//             return FALSE;
//     if (RequestedInformation & GROUP_SECURITY_INFORMATION)
//         if (!DumpGroup(SecurityDescriptor, NULL, &len))
//             return FALSE;
//     if (RequestedInformation & DACL_SECURITY_INFORMATION)
//         if (!DumpDacl(SecurityDescriptor, NULL, &len))
//             return FALSE;
//     if (RequestedInformation & SACL_SECURITY_INFORMATION)
//         if (!DumpSacl(SecurityDescriptor, NULL, &len))
//             return FALSE;
//
//     wstr = wptr = LocalAlloc(0, (len + 1)*sizeof(WCHAR));
// #ifdef __REACTOS__
//     if (wstr == NULL)
//         return FALSE;
// #endif
//
//     if (RequestedInformation & OWNER_SECURITY_INFORMATION)
//         if (!DumpOwner(SecurityDescriptor, &wptr, NULL)) {
//             LocalFree (wstr);
//             return FALSE;
//         }
//     if (RequestedInformation & GROUP_SECURITY_INFORMATION)
//         if (!DumpGroup(SecurityDescriptor, &wptr, NULL)) {
//             LocalFree (wstr);
//             return FALSE;
//         }
//     if (RequestedInformation & DACL_SECURITY_INFORMATION)
//         if (!DumpDacl(SecurityDescriptor, &wptr, NULL)) {
//             LocalFree (wstr);
//             return FALSE;
//         }
//     if (RequestedInformation & SACL_SECURITY_INFORMATION)
//         if (!DumpSacl(SecurityDescriptor, &wptr, NULL)) {
//             LocalFree (wstr);
//             return FALSE;
//         }
//     *wptr = 0;
//
//     TRACE("ret: %s, %d\n", wine_dbgstr_w(wstr), len);
//     *OutputString = wstr;
//     if (OutputLen)
//         *OutputLen = strlenW(*OutputString)+1;
//     return TRUE;
// }
//
// /******************************************************************************
//  * ConvertSecurityDescriptorToStringSecurityDescriptorA [ADVAPI32.@]
//  */
// BOOL WINAPI ConvertSecurityDescriptorToStringSecurityDescriptorA(PSECURITY_DESCRIPTOR SecurityDescriptor, DWORD SDRevision, SECURITY_INFORMATION Information, LPSTR *OutputString, PULONG OutputLen)
// {
//     LPWSTR wstr;
//     ULONG len;
//     if (ConvertSecurityDescriptorToStringSecurityDescriptorW(SecurityDescriptor, SDRevision, Information, &wstr, &len))
//     {
//         int lenA;
//
//         lenA = WideCharToMultiByte(CP_ACP, 0, wstr, len, NULL, 0, NULL, NULL);
//         *OutputString = heap_alloc(lenA);
// #ifdef __REACTOS__
//         if (*OutputString == NULL)
//         {
//             LocalFree(wstr);
//             *OutputLen = 0;
//             return FALSE;
//         }
// #endif
//         WideCharToMultiByte(CP_ACP, 0, wstr, len, *OutputString, lenA, NULL, NULL);
//         LocalFree(wstr);
//
//         if (OutputLen != NULL)
//             *OutputLen = lenA;
//         return TRUE;
//     }
//     else
//     {
//         *OutputString = NULL;
//         if (OutputLen)
//             *OutputLen = 0;
//         return FALSE;
//     }
// }
//
// /******************************************************************************
//  * ConvertStringSidToSidW [ADVAPI32.@]
//  */
// BOOL WINAPI ConvertStringSidToSidW(LPCWSTR StringSid, PSID* Sid)
// {
//     BOOL bret = FALSE;
//     DWORD cBytes;
//
//     TRACE("%s, %p\n", debugstr_w(StringSid), Sid);
//     if (GetVersion() & 0x80000000)
//         SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     else if (!StringSid || !Sid)
//         SetLastError(ERROR_INVALID_PARAMETER);
//     else if (ParseStringSidToSid(StringSid, NULL, &cBytes))
//     {
//         PSID pSid = *Sid = LocalAlloc(0, cBytes);
//
//         bret = ParseStringSidToSid(StringSid, pSid, &cBytes);
//         if (!bret)
//             LocalFree(*Sid);
//     }
//     return bret;
// }
//
// /******************************************************************************
//  * ConvertStringSidToSidA [ADVAPI32.@]
//  */
// BOOL WINAPI ConvertStringSidToSidA(LPCSTR StringSid, PSID* Sid)
// {
//     BOOL bret = FALSE;
//
//     TRACE("%s, %p\n", debugstr_a(StringSid), Sid);
//     if (GetVersion() & 0x80000000)
//         SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     else if (!StringSid || !Sid)
//         SetLastError(ERROR_INVALID_PARAMETER);
//     else
//     {
//         WCHAR *wStringSid = SERV_dup(StringSid);
//         bret = ConvertStringSidToSidW(wStringSid, Sid);
//         heap_free(wStringSid);
//     }
//     return bret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ConvertSidToStringSidW(PSID Sid,
//                        LPWSTR *StringSid)
// {
//     NTSTATUS Status;
//     UNICODE_STRING UnicodeString;
//     WCHAR FixedBuffer[64];
//
//     if (!RtlValidSid(Sid))
//     {
//         SetLastError(ERROR_INVALID_SID);
//         return FALSE;
//     }
//
//     UnicodeString.Length = 0;
//     UnicodeString.MaximumLength = sizeof(FixedBuffer);
//     UnicodeString.Buffer = FixedBuffer;
//     Status = RtlConvertSidToUnicodeString(&UnicodeString, Sid, FALSE);
//     if (STATUS_BUFFER_TOO_SMALL == Status)
//     {
//         Status = RtlConvertSidToUnicodeString(&UnicodeString, Sid, TRUE);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     *StringSid = LocalAlloc(LMEM_FIXED, UnicodeString.Length + sizeof(WCHAR));
//     if (NULL == *StringSid)
//     {
//         if (UnicodeString.Buffer != FixedBuffer)
//         {
//             RtlFreeUnicodeString(&UnicodeString);
//         }
//       SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//       return FALSE;
//     }
//
//     MoveMemory(*StringSid, UnicodeString.Buffer, UnicodeString.Length);
//     ZeroMemory((PCHAR) *StringSid + UnicodeString.Length, sizeof(WCHAR));
//     if (UnicodeString.Buffer != FixedBuffer)
//     {
//         RtlFreeUnicodeString(&UnicodeString);
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ConvertSidToStringSidA(PSID Sid,
//                        LPSTR *StringSid)
// {
//     LPWSTR StringSidW;
//     int Len;
//
//     if (!ConvertSidToStringSidW(Sid, &StringSidW))
//     {
//         return FALSE;
//     }
//
//     Len = WideCharToMultiByte(CP_ACP, 0, StringSidW, -1, NULL, 0, NULL, NULL);
//     if (Len <= 0)
//     {
//         LocalFree(StringSidW);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     *StringSid = LocalAlloc(LMEM_FIXED, Len);
//     if (NULL == *StringSid)
//     {
//         LocalFree(StringSidW);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!WideCharToMultiByte(CP_ACP, 0, StringSidW, -1, *StringSid, Len, NULL, NULL))
//     {
//         LocalFree(StringSid);
//         LocalFree(StringSidW);
//         return FALSE;
//     }
//
//     LocalFree(StringSidW);
//
//     return TRUE;
// }
//
//
// static
// DWORD
// GetUnicodeEnvironmentSize(
//     PVOID pEnvironment)
// {
//     INT Length, TotalLength = 0;
//     PWCHAR Ptr;
//
//     if (pEnvironment == NULL)
//         return 0;
//
//     Ptr = (PWCHAR)pEnvironment;
//     while (*Ptr != UNICODE_NULL)
//     {
//         Length = wcslen(Ptr) + 1;
//         TotalLength += Length;
//         Ptr = Ptr + Length;
//     }
//
//     return (TotalLength + 1) * sizeof(WCHAR);
// }
//
//
// static
// DWORD
// GetAnsiEnvironmentSize(
//     PVOID pEnvironment)
// {
//     INT Length, TotalLength = 0;
//     PCHAR Ptr;
//
//     if (pEnvironment == NULL)
//         return 0;
//
//     Ptr = (PCHAR)pEnvironment;
//     while (*Ptr != ANSI_NULL)
//     {
//         Length = strlen(Ptr) + 1;
//         TotalLength += Length;
//         Ptr = Ptr + Length;
//     }
//
//     return TotalLength + 1;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// CreateProcessWithLogonW(
//      LPCWSTR lpUsername,
//      LPCWSTR lpDomain,
//      LPCWSTR lpPassword,
//      DWORD dwLogonFlags,
//      LPCWSTR lpApplicationName,
//      LPWSTR lpCommandLine,
//      DWORD dwCreationFlags,
//      LPVOID lpEnvironment,
//      LPCWSTR lpCurrentDirectory,
//      LPSTARTUPINFOW lpStartupInfo,
//      LPPROCESS_INFORMATION lpProcessInformation)
// {
//     LPWSTR pszStringBinding = NULL;
//     handle_t hBinding = NULL;
//     SECL_REQUEST Request;
//     SECL_RESPONSE Response;
//     RPC_STATUS Status;
//
//     TRACE("CreateProcessWithLogonW(%s %s %s 0x%08x %s %s 0x%08x %p %s %p %p)\n", debugstr_w(lpUsername), debugstr_w(lpDomain),
//     debugstr_w(lpPassword), dwLogonFlags, debugstr_w(lpApplicationName),
//     debugstr_w(lpCommandLine), dwCreationFlags, lpEnvironment, debugstr_w(lpCurrentDirectory),
//     lpStartupInfo, lpProcessInformation);
//
//     Status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       NULL,
//                                       L"\\pipe\\seclogon",
//                                       NULL,
//                                       &pszStringBinding);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcStringBindingCompose returned 0x%x\n", Status);
//         SetLastError(Status);
//         return FALSE;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     Status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hBinding);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcBindingFromStringBinding returned 0x%x\n", Status);
//     }
//
//     Status = RpcStringFreeW(&pszStringBinding);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcStringFree returned 0x%x\n", Status);
//     }
//
//     Request.Username = (LPWSTR)lpUsername;
//     Request.Domain = (LPWSTR)lpDomain;
//     Request.Password = (LPWSTR)lpPassword;
//     Request.ApplicationName = (LPWSTR)lpApplicationName;
//     Request.CommandLine = (LPWSTR)lpCommandLine;
//     Request.CurrentDirectory = (LPWSTR)lpCurrentDirectory;
//
//     if (dwCreationFlags & CREATE_UNICODE_ENVIRONMENT)
//         Request.dwEnvironmentSize = GetUnicodeEnvironmentSize(lpEnvironment);
//     else
//         Request.dwEnvironmentSize = GetAnsiEnvironmentSize(lpEnvironment);
//     Request.Environment = lpEnvironment;
//
//     TRACE("Request.dwEnvironmentSize %lu\n", Request.dwEnvironmentSize);
//     TRACE("Request.Environment %p\n", Request.Environment);
//
//     Request.dwLogonFlags = dwLogonFlags;
//     Request.dwCreationFlags = dwCreationFlags;
//
//     Request.dwProcessId = GetCurrentProcessId();
//     TRACE("Request.dwProcessId %lu\n", Request.dwProcessId);
//
//     Response.hProcess = 0;
//     Response.hThread = 0;
//     Response.dwProcessId = 0;
//     Response.dwThreadId = 0;
//     Response.dwError = ERROR_SUCCESS;
//
//     RpcTryExcept
//     {
//         SeclCreateProcessWithLogonW(hBinding, &Request, &Response);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         WARN("Exception: %lx\n", RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (hBinding)
//     {
//         Status = RpcBindingFree(&hBinding);
//         if (Status != RPC_S_OK)
//         {
//             WARN("RpcBindingFree returned 0x%x\n", Status);
//         }
//
//         hBinding = NULL;
//     }
//
//     TRACE("Response.hProcess %p\n", Response.hProcess);
//     TRACE("Response.hThread %p\n", Response.hThread);
//     TRACE("Response.dwProcessId %lu\n", Response.dwProcessId);
//     TRACE("Response.dwThreadId %lu\n", Response.dwThreadId);
//     TRACE("Response.dwError %lu\n", Response.dwError);
//     if (Response.dwError != ERROR_SUCCESS)
//         SetLastError(Response.dwError);
//
//     TRACE("CreateProcessWithLogonW() done\n");
//
//     return (Response.dwError == ERROR_SUCCESS);
// }
//
// BOOL WINAPI CreateProcessWithTokenW(HANDLE token, DWORD logon_flags, LPCWSTR application_name, LPWSTR command_line,
//         DWORD creation_flags, void *environment, LPCWSTR current_directory, STARTUPINFOW *startup_info,
//         PROCESS_INFORMATION *process_information )
// {
//     FIXME("%p 0x%08x %s %s 0x%08x %p %s %p %p - semi-stub\n", token,
//           logon_flags, debugstr_w(application_name), debugstr_w(command_line),
//           creation_flags, environment, debugstr_w(current_directory),
//           startup_info, process_information);
//
//     /* FIXME: check if handles should be inherited */
//     return CreateProcessW( application_name, command_line, NULL, NULL, FALSE, creation_flags, environment,
//                            current_directory, startup_info, process_information );
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// DuplicateTokenEx(IN HANDLE ExistingTokenHandle,
//                  IN DWORD dwDesiredAccess,
//                  IN LPSECURITY_ATTRIBUTES lpTokenAttributes  OPTIONAL,
//                  IN SECURITY_IMPERSONATION_LEVEL ImpersonationLevel,
//                  IN TOKEN_TYPE TokenType,
//                  OUT PHANDLE DuplicateTokenHandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     SECURITY_QUALITY_OF_SERVICE Sqos;
//
//     TRACE("%p 0x%08x 0x%08x 0x%08x %p\n", ExistingTokenHandle, dwDesiredAccess,
//         ImpersonationLevel, TokenType, DuplicateTokenHandle);
//
//     Sqos.Length = sizeof(SECURITY_QUALITY_OF_SERVICE);
//     Sqos.ImpersonationLevel = ImpersonationLevel;
//     Sqos.ContextTrackingMode = 0;
//     Sqos.EffectiveOnly = FALSE;
//
//     if (lpTokenAttributes != NULL)
//     {
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    NULL,
//                                    lpTokenAttributes->bInheritHandle ? OBJ_INHERIT : 0,
//                                    NULL,
//                                    lpTokenAttributes->lpSecurityDescriptor);
//     }
//     else
//     {
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    NULL,
//                                    0,
//                                    NULL,
//                                    NULL);
//     }
//
//     ObjectAttributes.SecurityQualityOfService = &Sqos;
//
//     Status = NtDuplicateToken(ExistingTokenHandle,
//                               dwDesiredAccess,
//                               &ObjectAttributes,
//                               FALSE,
//                               TokenType,
//                               DuplicateTokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtDuplicateToken failed: Status %08x\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     TRACE("Returning token %p.\n", *DuplicateTokenHandle);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// DuplicateToken(IN HANDLE ExistingTokenHandle,
//                IN SECURITY_IMPERSONATION_LEVEL ImpersonationLevel,
//                OUT PHANDLE DuplicateTokenHandle)
// {
//     return DuplicateTokenEx(ExistingTokenHandle,
//                             TOKEN_IMPERSONATE | TOKEN_QUERY,
//                             NULL,
//                             ImpersonationLevel,
//                             TokenImpersonation,
//                             DuplicateTokenHandle);
// }
//
// /******************************************************************************
//  * ComputeStringSidSize
//  */
// static DWORD ComputeStringSidSize(LPCWSTR StringSid)
// {
//     if (StringSid[0] == 'S' && StringSid[1] == '-') /* S-R-I(-S)+ */
//     {
//         int ctok = 0;
//         while (*StringSid)
//         {
//             if (*StringSid == '-')
//                 ctok++;
//             StringSid++;
//         }
//
//         if (ctok >= 3)
//             return GetSidLengthRequired(ctok - 2);
//     }
//     else /* String constant format  - Only available in winxp and above */
//     {
//         unsigned int i;
//
//         for (i = 0; i < sizeof(WellKnownSids)/sizeof(WellKnownSids[0]); i++)
//             if (!strncmpW(WellKnownSids[i].wstr, StringSid, 2))
//                 return GetSidLengthRequired(WellKnownSids[i].Sid.SubAuthorityCount);
//
//         for (i = 0; i < sizeof(WellKnownRids)/sizeof(WellKnownRids[0]); i++)
//             if (!strncmpW(WellKnownRids[i].wstr, StringSid, 2))
//             {
//                 MAX_SID local;
//                 ADVAPI_GetComputerSid(&local);
//                 return GetSidLengthRequired(*GetSidSubAuthorityCount(&local) + 1);
//             }
//
//     }
//
//     return GetSidLengthRequired(0);
// }
//
// /******************************************************************************
//  * ParseStringSidToSid
//  */
// static BOOL ParseStringSidToSid(LPCWSTR StringSid, PSID pSid, LPDWORD cBytes)
// {
//     BOOL bret = FALSE;
//     SID* pisid=pSid;
//
//     TRACE("%s, %p, %p\n", debugstr_w(StringSid), pSid, cBytes);
//     if (!StringSid)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         TRACE("StringSid is NULL, returning FALSE\n");
//         return FALSE;
//     }
//
//     while (*StringSid == ' ')
//         StringSid++;
//
//     if (!*StringSid)
//         goto lend; /* ERROR_INVALID_SID */
//
//     *cBytes = ComputeStringSidSize(StringSid);
//     if (!pisid) /* Simply compute the size */
//     {
//         TRACE("only size requested, returning TRUE with %d\n", *cBytes);
//         return TRUE;
//     }
//
//     if (StringSid[0] == 'S' && StringSid[1] == '-') /* S-R-I-S-S */
//     {
//         DWORD i = 0, identAuth;
//         DWORD csubauth = ((*cBytes - GetSidLengthRequired(0)) / sizeof(DWORD));
//
//         StringSid += 2; /* Advance to Revision */
//         pisid->Revision = atoiW(StringSid);
//
//         if (pisid->Revision != SDDL_REVISION)
//         {
//             TRACE("Revision %d is unknown\n", pisid->Revision);
//             goto lend; /* ERROR_INVALID_SID */
//         }
//         if (csubauth == 0)
//         {
//             TRACE("SubAuthorityCount is 0\n");
//             goto lend; /* ERROR_INVALID_SID */
//         }
//
//         pisid->SubAuthorityCount = csubauth;
//
//         /* Advance to identifier authority */
//         while (*StringSid && *StringSid != '-')
//             StringSid++;
//         if (*StringSid == '-')
//             StringSid++;
//
//         /* MS' implementation can't handle values greater than 2^32 - 1, so
//          * we don't either; assume most significant bytes are always 0
//          */
//         pisid->IdentifierAuthority.Value[0] = 0;
//         pisid->IdentifierAuthority.Value[1] = 0;
//         identAuth = atoiW(StringSid);
//         pisid->IdentifierAuthority.Value[5] = identAuth & 0xff;
//         pisid->IdentifierAuthority.Value[4] = (identAuth & 0xff00) >> 8;
//         pisid->IdentifierAuthority.Value[3] = (identAuth & 0xff0000) >> 16;
//         pisid->IdentifierAuthority.Value[2] = (identAuth & 0xff000000) >> 24;
//
//         /* Advance to first sub authority */
//         while (*StringSid && *StringSid != '-')
//             StringSid++;
//         if (*StringSid == '-')
//             StringSid++;
//
//         while (*StringSid)
//         {
//             pisid->SubAuthority[i++] = atoiW(StringSid);
//
//             while (*StringSid && *StringSid != '-')
//                 StringSid++;
//             if (*StringSid == '-')
//                 StringSid++;
//         }
//
//         if (i != pisid->SubAuthorityCount)
//             goto lend; /* ERROR_INVALID_SID */
//
//         bret = TRUE;
//     }
//     else /* String constant format  - Only available in winxp and above */
//     {
//         unsigned int i;
//         pisid->Revision = SDDL_REVISION;
//
//         for (i = 0; i < sizeof(WellKnownSids)/sizeof(WellKnownSids[0]); i++)
//             if (!strncmpW(WellKnownSids[i].wstr, StringSid, 2))
//             {
//                 DWORD j;
//                 pisid->SubAuthorityCount = WellKnownSids[i].Sid.SubAuthorityCount;
//                 pisid->IdentifierAuthority = WellKnownSids[i].Sid.IdentifierAuthority;
//                 for (j = 0; j < WellKnownSids[i].Sid.SubAuthorityCount; j++)
//                     pisid->SubAuthority[j] = WellKnownSids[i].Sid.SubAuthority[j];
//                 bret = TRUE;
//             }
//
//         for (i = 0; i < sizeof(WellKnownRids)/sizeof(WellKnownRids[0]); i++)
//             if (!strncmpW(WellKnownRids[i].wstr, StringSid, 2))
//             {
//                 ADVAPI_GetComputerSid(pisid);
//                 pisid->SubAuthority[pisid->SubAuthorityCount] = WellKnownRids[i].Rid;
//                 pisid->SubAuthorityCount++;
//                 bret = TRUE;
//             }
//
//         if (!bret)
//             FIXME("String constant not supported: %s\n", debugstr_wn(StringSid, 2));
//     }
//
// lend:
//     if (!bret)
//         SetLastError(ERROR_INVALID_SID);
//
//     TRACE("returning %s\n", bret ? "TRUE" : "FALSE");
//     return bret;
// }
//
// /**********************************************************************
//  * GetNamedSecurityInfoA			EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// GetNamedSecurityInfoA(LPSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       PSID *ppsidOwner,
//                       PSID *ppsidGroup,
//                       PACL *ppDacl,
//                       PACL *ppSacl,
//                       PSECURITY_DESCRIPTOR *ppSecurityDescriptor)
// {
//     DWORD len;
//     LPWSTR wstr = NULL;
//     DWORD r;
//
//     TRACE("%s %d %d %p %p %p %p %p\n", pObjectName, ObjectType, SecurityInfo,
//         ppsidOwner, ppsidGroup, ppDacl, ppSacl, ppSecurityDescriptor);
//
//     if( pObjectName )
//     {
//         len = MultiByteToWideChar( CP_ACP, 0, pObjectName, -1, NULL, 0 );
//         wstr = HeapAlloc( GetProcessHeap(), 0, len*sizeof(WCHAR));
//         MultiByteToWideChar( CP_ACP, 0, pObjectName, -1, wstr, len );
//     }
//
//     r = GetNamedSecurityInfoW( wstr, ObjectType, SecurityInfo, ppsidOwner,
//                            ppsidGroup, ppDacl, ppSacl, ppSecurityDescriptor );
//
//     HeapFree( GetProcessHeap(), 0, wstr );
//
//     return r;
// }
//
// /******************************************************************************
//  * GetWindowsAccountDomainSid         [ADVAPI32.@]
//  */
// BOOL WINAPI GetWindowsAccountDomainSid( PSID sid, PSID domain_sid, DWORD *size )
// {
//     SID_IDENTIFIER_AUTHORITY domain_ident = { SECURITY_NT_AUTHORITY };
//     DWORD required_size;
//     int i;
//
//     FIXME( "(%p %p %p): semi-stub\n", sid, domain_sid, size );
//
//     if (!sid || !IsValidSid( sid ))
//     {
//         SetLastError( ERROR_INVALID_SID );
//         return FALSE;
//     }
//
//     if (!size)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     if (*GetSidSubAuthorityCount( sid ) < 4)
//     {
//         SetLastError( ERROR_INVALID_SID );
//         return FALSE;
//     }
//
//     required_size = GetSidLengthRequired( 4 );
//     if (*size < required_size || !domain_sid)
//     {
//         *size = required_size;
//         SetLastError( domain_sid ? ERROR_INSUFFICIENT_BUFFER :
//                                    ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     InitializeSid( domain_sid, &domain_ident, 4 );
//     for (i = 0; i < 4; i++)
//         *GetSidSubAuthority( domain_sid, i ) = *GetSidSubAuthority( sid, i );
//
//     *size = required_size;
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// EqualDomainSid(IN PSID pSid1,
//                IN PSID pSid2,
//                OUT BOOL* pfEqual)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
