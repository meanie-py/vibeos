//! MIDL client-stub prototypes for the netapi32 `types_rpc` type island.
//!
//! @generated — do NOT edit by hand. Regenerate with a functions-enabled run of the
//! `netapi32-rpc` profile in tools/rustconv/gen_driver_types.sh (that profile sets
//! BLOCKLIST_FUNCS=1 so its `types_rpc` island carries the wire STRUCTS only; this file is
//! the matching FUNCTION half, extracted from the same bindgen pass and ABI-normalised).
//!
//! These symbols are DEFINED by the WIDL-generated `*_c.c` client stubs, which cmake still
//! compiles as C (dll/win32/netapi32/CMakeLists.txt) — the cohab flip moves only the
//! hand-written `.c` to Rust, never the generated MIDL marshalling. So these are genuine
//! cross-language imports, not translation surface.
//!
//! ABI: bindgen emits these `__stdcall` prototypes as `extern "stdcall"` on i386 and
//! `extern "C"` on amd64/arm64. `extern "system"` IS exactly that per-arch pair, so one
//! arch-neutral block replaces three — verified: the generated signatures are byte-identical
//! across all three arches, only the ABI string differed.
//!
//! EXCLUDED (10): SRVSVC_HANDLE_bind, SRVSVC_HANDLE_unbind, WKSSVC_IDENTIFY_HANDLE_bind, WKSSVC_IDENTIFY_HANDLE_unbind, WKSSVC_IMPERSONATE_HANDLE_bind, WKSSVC_IMPERSONATE_HANDLE_unbind, ATSVC_HANDLE_bind, ATSVC_HANDLE_unbind, LOGONSRV_HANDLE_bind, LOGONSRV_HANDLE_unbind.
//! These are declared in the same headers but DEFINED by the translated modules (they are the
//! client's binding-handle callbacks), so an extern decl here would collide with the definition.
#![allow(non_snake_case, non_camel_case_types)]

use crate::types_rpc::*;

extern "system" {
    pub fn Opnum0NotUsedOnWire();
    pub fn Opnum1NotUsedOnWire();
    pub fn Opnum2NotUsedOnWire();
    pub fn Opnum3NotUsedOnWire();
    pub fn Opnum4NotUsedOnWire();
    pub fn Opnum5NotUsedOnWire();
    pub fn Opnum6NotUsedOnWire();
    pub fn Opnum7NotUsedOnWire();
    pub fn NetrConnectionEnum(
        ServerName: SRVSVC_HANDLE,
        Qualifier: *mut WCHAR,
        InfoStruct: LPCONNECT_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrFileEnum(
        ServerName: SRVSVC_HANDLE,
        BasePath: *mut WCHAR,
        UserName: *mut WCHAR,
        InfoStruct: PFILE_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrFileGetInfo(
        ServerName: SRVSVC_HANDLE,
        FileId: DWORD,
        Level: DWORD,
        InfoStruct: LPFILE_INFO,
    ) -> DWORD;
    pub fn NetrFileClose(ServerName: SRVSVC_HANDLE, FileId: DWORD) -> DWORD;
    pub fn NetrSessionEnum(
        ServerName: SRVSVC_HANDLE,
        ClientName: *mut WCHAR,
        UserName: *mut WCHAR,
        InfoStruct: PSESSION_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrSessionDel(
        ServerName: SRVSVC_HANDLE,
        ClientName: *mut WCHAR,
        UserName: *mut WCHAR,
    ) -> DWORD;
    pub fn NetrShareAdd(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        InfoStruct: LPSHARE_INFO,
        ParmErr: *mut DWORD,
    ) -> DWORD;
    pub fn NetrShareEnum(
        ServerName: SRVSVC_HANDLE,
        InfoStruct: LPSHARE_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrShareGetInfo(
        ServerName: SRVSVC_HANDLE,
        NetName: *mut WCHAR,
        Level: DWORD,
        InfoStruct: LPSHARE_INFO,
    ) -> DWORD;
    pub fn NetrShareSetInfo(
        ServerName: SRVSVC_HANDLE,
        NetName: *mut WCHAR,
        Level: DWORD,
        ShareInfo: LPSHARE_INFO,
        ParmErr: *mut DWORD,
    ) -> DWORD;
    pub fn NetrShareDel(ServerName: SRVSVC_HANDLE, NetName: *mut WCHAR, Reserved: DWORD) -> DWORD;
    pub fn NetrShareDelSticky(
        ServerName: SRVSVC_HANDLE,
        NetName: *mut WCHAR,
        Reserved: DWORD,
    ) -> DWORD;
    pub fn NetrShareCheck(ServerName: SRVSVC_HANDLE, Device: *mut WCHAR, Type: *mut DWORD)
        -> DWORD;
    pub fn NetrServerGetInfo(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        InfoStruct: *mut LPSERVER_INFO,
    ) -> DWORD;
    pub fn NetrServerSetInfo(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        ServerInfo: LPSERVER_INFO,
        ParmErr: *mut DWORD,
    ) -> DWORD;
    pub fn NetrServerDiskEnum(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        DiskInfoStruct: *mut DISK_ENUM_CONTAINER,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrServerStatisticsGet(
        ServerName: SRVSVC_HANDLE,
        Service: *mut WCHAR,
        Level: DWORD,
        Options: DWORD,
        InfoStruct: *mut LPSTAT_SERVER_0,
    ) -> DWORD;
    pub fn NetrServerTransportAdd(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        Buffer: LPSERVER_TRANSPORT_INFO_0,
    ) -> DWORD;
    pub fn NetrServerTransportEnum(
        ServerName: SRVSVC_HANDLE,
        InfoStruct: LPSERVER_XPORT_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrServerTransportDel(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        Buffer: LPSERVER_TRANSPORT_INFO_0,
    ) -> DWORD;
    pub fn NetrRemoteTOD(ServerName: SRVSVC_HANDLE, BufferPtr: *mut LPTIME_OF_DAY_INFO) -> DWORD;
    pub fn NetrServerSetServiceBits(
        ServerName: SRVSVC_HANDLE,
        Transport: *mut WCHAR,
        ServiceBits: DWORD,
        UpdateImmediately: DWORD,
    ) -> DWORD;
    pub fn NetprPathType(
        ServerName: SRVSVC_HANDLE,
        PathName: *mut WCHAR,
        PathType: *mut DWORD,
        Flags: DWORD,
    ) -> DWORD;
    pub fn NetprPathCanonicalize(
        ServerName: SRVSVC_HANDLE,
        PathName: *mut WCHAR,
        Outbuf: *mut ::core::ffi::c_uchar,
        OutbufLen: DWORD,
        Prefix: *mut WCHAR,
        PathType: *mut DWORD,
        Flags: DWORD,
    ) -> DWORD;
    pub fn NetprPathCompare(
        ServerName: SRVSVC_HANDLE,
        PathName1: *mut WCHAR,
        PathName2: *mut WCHAR,
        PathType: DWORD,
        Flags: DWORD,
    ) -> LONG;
    pub fn NetprNameValidate(
        ServerName: SRVSVC_HANDLE,
        Name: *mut WCHAR,
        NameType: DWORD,
        Flags: DWORD,
    ) -> DWORD;
    pub fn NetprNameCanonicalize(
        ServerName: SRVSVC_HANDLE,
        Name: *mut WCHAR,
        Outbuf: *mut WCHAR,
        OutbufLen: DWORD,
        NameType: DWORD,
        Flags: DWORD,
    ) -> DWORD;
    pub fn NetprNameCompare(
        ServerName: SRVSVC_HANDLE,
        Name1: *mut WCHAR,
        Name2: *mut WCHAR,
        NameType: DWORD,
        Flags: DWORD,
    ) -> LONG;
    pub fn NetrShareEnumSticky(
        ServerName: SRVSVC_HANDLE,
        InfoStruct: LPSHARE_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: *mut DWORD,
        ResumeHandle: *mut DWORD,
    ) -> DWORD;
    pub fn NetrShareDelStart(
        ServerName: SRVSVC_HANDLE,
        NetName: *mut WCHAR,
        Reserved: DWORD,
        ContextHandle: PSHARE_DEL_HANDLE,
    ) -> DWORD;
    pub fn NetrShareDelCommit(ContextHandle: PSHARE_DEL_HANDLE) -> DWORD;
    pub fn NetrpGetFileSecurity(
        ServerName: SRVSVC_HANDLE,
        ShareName: *mut WCHAR,
        lpFileName: *mut WCHAR,
        RequestedInformation: SECURITY_INFORMATION,
        SecurityDescriptor: *mut PADT_SECURITY_DESCRIPTOR,
    ) -> DWORD;
    pub fn NetrpSetFileSecurity(
        ServerName: SRVSVC_HANDLE,
        ShareName: *mut WCHAR,
        lpFileName: *mut WCHAR,
        SecurityInformation: SECURITY_INFORMATION,
        SecurityDescriptor: PADT_SECURITY_DESCRIPTOR,
    ) -> DWORD;
    pub fn NetrServerTransportAddEx(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        Buffer: LPTRANSPORT_INFO,
    ) -> DWORD;
    pub fn NetrServerSetServiceBitsEx(
        ServerName: SRVSVC_HANDLE,
        EmulatedServer: *mut WCHAR,
        Transport: *mut WCHAR,
        ServiceBitsOfInterest: DWORD,
        ServiceBits: DWORD,
        UpdateImmediately: DWORD,
    ) -> DWORD;
    pub fn NetrDfsGetVersion(ServerName: SRVSVC_HANDLE, Version: *mut DWORD) -> DWORD;
    pub fn NetrDfsCreateLocalPartition(
        ServerName: SRVSVC_HANDLE,
        ShareName: *mut WCHAR,
        EntryUid: *mut GUID,
        EntryPrefix: *mut WCHAR,
        ShortName: *mut WCHAR,
        RelationInfo: LPNET_DFS_ENTRY_ID_CONTAINER,
        Force: ::core::ffi::c_int,
    ) -> DWORD;
    pub fn NetrDfsDeleteLocalPartition(
        ServerName: SRVSVC_HANDLE,
        Uid: *mut GUID,
        Prefix: *mut WCHAR,
    ) -> DWORD;
    pub fn NetrDfsSetLocalVolumeState(
        ServerName: SRVSVC_HANDLE,
        Uid: *mut GUID,
        Prefix: *mut WCHAR,
        State: ULONG,
    ) -> DWORD;
    pub fn NetrDfsSetServerInfo() -> DWORD;
    pub fn NetrDfsCreateExitPoint(
        ServerName: SRVSVC_HANDLE,
        Uid: *mut GUID,
        Prefix: *mut WCHAR,
        Type: ULONG,
        ShortPrefixLen: DWORD,
        ShortPrefix: *mut WCHAR,
    ) -> DWORD;
    pub fn NetrDfsDeleteExitPoint(
        ServerName: SRVSVC_HANDLE,
        Uid: *mut GUID,
        Prefix: *mut WCHAR,
        Type: ULONG,
    ) -> DWORD;
    pub fn NetrDfsModifyPrefix(
        ServerName: SRVSVC_HANDLE,
        Uid: *mut GUID,
        Prefix: *mut WCHAR,
    ) -> DWORD;
    pub fn NetrDfsFixLocalVolume(
        ServerName: SRVSVC_HANDLE,
        VolumeName: *mut WCHAR,
        EntryType: ULONG,
        ServiceType: ULONG,
        StgId: *mut WCHAR,
        EntryUid: *mut GUID,
        EntryPrefix: *mut WCHAR,
        RelationInfo: LPNET_DFS_ENTRY_ID_CONTAINER,
        CreateDisposition: ULONG,
    ) -> DWORD;
    pub fn NetrDfsManagerReportSiteInfo(
        ServerName: SRVSVC_HANDLE,
        ppSiteInfo: *mut LPDFS_SITELIST_INFO,
    ) -> DWORD;
    pub fn NetrServerTransportDelEx(
        ServerName: SRVSVC_HANDLE,
        Level: DWORD,
        Buffer: LPTRANSPORT_INFO,
    ) -> DWORD;
    pub fn SHARE_DEL_HANDLE_rundown(arg1: SHARE_DEL_HANDLE);
    pub fn NetrWkstaGetInfo(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        Level: ULONG,
        WkstaInfo: LPWKSTA_INFO,
    ) -> ULONG;
    pub fn NetrWkstaSetInfo(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        Level: ULONG,
        WkstaInfo: LPWKSTA_INFO,
        ErrorParameter: *mut ULONG,
    ) -> ULONG;
    pub fn NetrWkstaUserEnum(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        UserInfo: LPWKSTA_USER_ENUM_STRUCT,
        PreferredMaximumLength: ULONG,
        TotalEntries: *mut ULONG,
        ResumeHandle: *mut ULONG,
    ) -> ULONG;
    pub fn NetrWkstaUserGetInfo(
        Unused: WKSSVC_IDENTIFY_HANDLE,
        Level: ULONG,
        UserInfo: LPWKSTA_USER_INFO,
    ) -> ULONG;
    pub fn NetrWkstaUserSetInfo(
        Unused: WKSSVC_IDENTIFY_HANDLE,
        Level: ULONG,
        UserInfo: LPWKSTA_USER_INFO,
        ErrorParameter: *mut ULONG,
    ) -> ULONG;
    pub fn NetrWkstaTransportEnum(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        TransportInfo: LPWKSTA_TRANSPORT_ENUM_STRUCT,
        PreferredMaximumLength: ULONG,
        TotalEntries: *mut ULONG,
        ResumeHandle: *mut ULONG,
    ) -> ULONG;
    pub fn NetrWkstaTransportAdd(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        Level: ULONG,
        TransportInfo: LPWKSTA_TRANSPORT_INFO_0,
        ErrorParameter: *mut ULONG,
    ) -> ULONG;
    pub fn NetrWkstaTransportDel(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        TransportName: *mut wchar_t,
        ForceLevel: ULONG,
    ) -> ULONG;
    pub fn NetrUseAdd(
        ServerName: WKSSVC_IMPERSONATE_HANDLE,
        Level: ULONG,
        InfoStruct: LPUSE_INFO,
        ErrorParameter: *mut ULONG,
    ) -> ULONG;
    pub fn NetrUseGetInfo(
        ServerName: WKSSVC_IMPERSONATE_HANDLE,
        UseName: *mut wchar_t,
        Level: ULONG,
        InfoStruct: LPUSE_INFO,
    ) -> ULONG;
    pub fn NetrUseDel(
        ServerName: WKSSVC_IMPERSONATE_HANDLE,
        UseName: *mut wchar_t,
        ForceLevel: ULONG,
    ) -> ULONG;
    pub fn NetrUseEnum(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        InfoStruct: LPUSE_ENUM_STRUCT,
        PreferredMaximumLength: ULONG,
        TotalEntries: *mut ULONG,
        ResumeHandle: *mut ULONG,
    ) -> ULONG;
    pub fn NetrMessageBufferSend() -> ULONG;
    pub fn NetrWorkstationStatisticsGet(
        ServerName: WKSSVC_IDENTIFY_HANDLE,
        ServiceName: *mut wchar_t,
        Level: ULONG,
        Options: ULONG,
        Buffer: *mut LPSTAT_WORKSTATION_0,
    ) -> ULONG;
    pub fn NetrLogonDomainNameAdd(DomainName: WKSSVC_IDENTIFY_HANDLE) -> ULONG;
    pub fn NetrLogonDomainNameDel(DomainName: WKSSVC_IDENTIFY_HANDLE) -> ULONG;
    pub fn NetrJoinDomain() -> ULONG;
    pub fn NetrUnjoinDomain() -> ULONG;
    pub fn NetrValidateName() -> ULONG;
    pub fn NetrRenameMachineInDomain() -> ULONG;
    pub fn NetrGetJoinInformation(
        ServerName: WKSSVC_IMPERSONATE_HANDLE,
        NameBuffer: *mut *mut wchar_t,
        BufferType: PNETSETUP_JOIN_STATUS,
    ) -> ULONG;
    pub fn NetrGetJoinableOUs() -> ULONG;
    pub fn NetrJoinDomain2(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        DomainNameParam: *mut wchar_t,
        MachineAccountOU: *mut wchar_t,
        AccountName: *mut wchar_t,
        Password: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Options: ULONG,
    ) -> ULONG;
    pub fn NetrUnjoinDomain2(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        AccountName: *mut wchar_t,
        Password: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Options: ULONG,
    ) -> ULONG;
    pub fn NetrRenameMachineInDomain2(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        MachineName: *mut wchar_t,
        AccountName: *mut wchar_t,
        Password: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Options: ULONG,
    ) -> ULONG;
    pub fn NetrValidateName2(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        NameToValidate: *mut wchar_t,
        AccountName: *mut wchar_t,
        Password: PJOINPR_ENCRYPTED_USER_PASSWORD,
        NameType: NETSETUP_NAME_TYPE,
    ) -> ULONG;
    pub fn NetrGetJoinableOUs2(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        DomainNameParam: *mut wchar_t,
        AccountName: *mut wchar_t,
        Password: PJOINPR_ENCRYPTED_USER_PASSWORD,
        OUCount: *mut ULONG,
        OUs: *mut *mut *mut wchar_t,
    ) -> ULONG;
    pub fn NetrAddAlternateComputerName(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        AlternateName: *mut wchar_t,
        DomainAccount: *mut wchar_t,
        EncryptedPassword: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Reserved: ULONG,
    ) -> ULONG;
    pub fn NetrRemoveAlternateComputerName(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        AlternateName: *mut wchar_t,
        DomainAccount: *mut wchar_t,
        EncryptedPassword: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Reserved: ULONG,
    ) -> ULONG;
    pub fn NetrSetPrimaryComputerName(
        RpcBindingHandle: handle_t,
        ServerName: *mut wchar_t,
        PrimaryName: *mut wchar_t,
        DomainAccount: *mut wchar_t,
        EncryptedPassword: PJOINPR_ENCRYPTED_USER_PASSWORD,
        Reserved: ULONG,
    ) -> ULONG;
    pub fn NetrEnumerateComputerNames(
        ServerName: WKSSVC_IMPERSONATE_HANDLE,
        NameType: NET_COMPUTER_NAME_TYPE,
        Reserved: ULONG,
        ComputerNames: *mut PNET_COMPUTER_NAME_ARRAY,
    ) -> ULONG;
    pub fn NetrJobAdd(ServerName: ATSVC_HANDLE, pAtInfo: LPAT_INFO, pJobId: LPDWORD) -> DWORD;
    pub fn NetrJobDel(ServerName: ATSVC_HANDLE, MinJobId: DWORD, MaxJobId: DWORD) -> DWORD;
    pub fn NetrJobEnum(
        ServerName: ATSVC_HANDLE,
        pEnumContainer: LPAT_ENUM_CONTAINER,
        PreferedMaximumLength: DWORD,
        pTotalEntries: LPDWORD,
        pResumeHandle: LPDWORD,
    ) -> DWORD;
    pub fn NetrJobGetInfo(
        ServerName: ATSVC_HANDLE,
        JobId: DWORD,
        ppAtInfo: *mut LPAT_INFO,
    ) -> DWORD;
    pub fn DsRolerGetPrimaryDomainInformation(
        hBinding: handle_t,
        InfoLevel: DSROLE_PRIMARY_DOMAIN_INFO_LEVEL,
        DomainInfo: *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION,
    ) -> DWORD;
    pub fn NetrLogonUasLogon(
        ServerName: LOGONSRV_HANDLE,
        UserName: *mut wchar_t,
        Workstation: *mut wchar_t,
        ValidationInformation: *mut PNETLOGON_VALIDATION_UAS_INFO,
    ) -> DWORD;
    pub fn NetrLogonUasLogoff(
        ServerName: LOGONSRV_HANDLE,
        UserName: *mut wchar_t,
        Workstation: *mut wchar_t,
        LogoffInformation: PNETLOGON_LOGOFF_UAS_INFO,
    ) -> DWORD;
    pub fn NetrLogonSamLogon(
        LogonServer: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        LogonLevel: NETLOGON_LOGON_INFO_CLASS,
        LogonInformation: PNETLOGON_LEVEL,
        ValidationLevel: NETLOGON_VALIDATION_INFO_CLASS,
        ValidationInformation: PNETLOGON_VALIDATION,
        Authoritative: *mut UCHAR,
    ) -> NTSTATUS;
    pub fn NetrLogonSamLogoff(
        LogonServer: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        LogonLevel: NETLOGON_LOGON_INFO_CLASS,
        LogonInformation: PNETLOGON_LEVEL,
    ) -> NTSTATUS;
    pub fn NetrServerReqChallenge(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        ClientChallenge: PNETLOGON_CREDENTIAL,
        ServerChallenge: PNETLOGON_CREDENTIAL,
    ) -> NTSTATUS;
    pub fn NetrServerAuthenticate(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        ClientCredential: PNETLOGON_CREDENTIAL,
        ServerCredential: PNETLOGON_CREDENTIAL,
    ) -> NTSTATUS;
    pub fn NetrServerPasswordSet(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        UasNewPassword: PENCRYPTED_NT_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn NetrDatabaseDeltas(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        DatabaseID: DWORD,
        DomainModifiedCount: PNLPR_MODIFIED_COUNT,
        DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
        PreferredMaximumLength: DWORD,
    ) -> NTSTATUS;
    pub fn NetrDatabaseSync(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        DatabaseID: DWORD,
        SyncContext: *mut ULONG,
        DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
        PreferredMaximumLength: DWORD,
    ) -> NTSTATUS;
    pub fn NetrAccountDeltas(
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
    ) -> NTSTATUS;
    pub fn NetrAccountSync(
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
    ) -> NTSTATUS;
    pub fn NetrGetDCName(
        ServerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        Buffer: *mut *mut wchar_t,
    ) -> DWORD;
    pub fn NetrLogonControl(
        ServerName: LOGONSRV_HANDLE,
        FunctionCode: DWORD,
        QueryLevel: DWORD,
        Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
    ) -> DWORD;
    pub fn NetrGetAnyDCName(
        ServerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        Buffer: *mut *mut wchar_t,
    ) -> DWORD;
    pub fn NetrLogonControl2(
        ServerName: LOGONSRV_HANDLE,
        FunctionCode: DWORD,
        QueryLevel: DWORD,
        Data: PNETLOGON_CONTROL_DATA_INFORMATION,
        Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
    ) -> DWORD;
    pub fn NetrServerAuthenticate2(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        ClientCredential: PNETLOGON_CREDENTIAL,
        ServerCredential: PNETLOGON_CREDENTIAL,
        NegotiateFlags: *mut ULONG,
    ) -> NTSTATUS;
    pub fn NetrDatabaseSync2(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        DatabaseID: DWORD,
        RestartState: SYNC_STATE,
        SyncContext: *mut ULONG,
        DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
        PreferredMaximumLength: DWORD,
    ) -> NTSTATUS;
    pub fn NetrDatabaseRedo(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        ChangeLogEntry: *mut UCHAR,
        ChangeLogEntrySize: DWORD,
        DeltaArray: *mut PNETLOGON_DELTA_ENUM_ARRAY,
    ) -> NTSTATUS;
    pub fn NetrLogonControl2Ex(
        ServerName: LOGONSRV_HANDLE,
        FunctionCode: DWORD,
        QueryLevel: DWORD,
        Data: PNETLOGON_CONTROL_DATA_INFORMATION,
        Buffer: PNETLOGON_CONTROL_QUERY_INFORMATION,
    ) -> DWORD;
    pub fn NetrEnumerateTrustedDomains(
        ServerName: LOGONSRV_HANDLE,
        DomainNameBuffer: PDOMAIN_NAME_BUFFER,
    ) -> NTSTATUS;
    pub fn DsrGetDcName(
        ComputerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        DomainGuid: *mut GUID,
        SiteGuid: *mut GUID,
        Flags: ULONG,
        DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
    ) -> DWORD;
    pub fn NetrLogonGetCapabilities(
        ServerName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        QueryLevel: DWORD,
        ServerCapabilities: PNETLOGON_CAPABILITIES,
    ) -> NTSTATUS;
    pub fn NetrLogonSetServiceBits(
        ServerName: LOGONSRV_HANDLE,
        ServiceBitsOfInterest: DWORD,
        ServiceBits: DWORD,
    ) -> NTSTATUS;
    pub fn NetrLogonGetTrustRid(
        ServerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        Rid: *mut ULONG,
    ) -> DWORD;
    pub fn NetrLogonComputeServerDigest(
        ServerName: LOGONSRV_HANDLE,
        Rid: ULONG,
        Message: *mut UCHAR,
        MessageSize: ULONG,
        NewMessageDigest: *mut CHAR,
        OldMessageDigest: *mut CHAR,
    ) -> DWORD;
    pub fn NetrLogonComputeClientDigest(
        ServerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        Message: *mut UCHAR,
        MessageSize: ULONG,
        NewMessageDigest: *mut CHAR,
        OldMessageDigest: *mut CHAR,
    ) -> DWORD;
    pub fn NetrServerAuthenticate3(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        ClientCredential: PNETLOGON_CREDENTIAL,
        ServerCredential: PNETLOGON_CREDENTIAL,
        NegotiateFlags: *mut ULONG,
        AccountRid: *mut ULONG,
    ) -> NTSTATUS;
    pub fn DsrGetDcNameEx(
        ComputerName: LOGONSRV_HANDLE,
        DomainName: *mut wchar_t,
        DomainGuid: *mut GUID,
        SiteName: *mut wchar_t,
        Flags: ULONG,
        DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
    ) -> DWORD;
    pub fn DsrGetSiteName(ComputerName: LOGONSRV_HANDLE, SiteName: *mut *mut wchar_t) -> DWORD;
    pub fn NetrLogonGetDomainInfo(
        ServerName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        Level: DWORD,
        WkstaBuffer: PNETLOGON_WORKSTATION_INFORMATION,
        DomBuffer: PNETLOGON_DOMAIN_INFORMATION,
    ) -> NTSTATUS;
    pub fn NetrServerPasswordSet2(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        ClearNewPassword: PNL_TRUST_PASSWORD,
    ) -> NTSTATUS;
    pub fn NetrServerPasswordGet(
        PrimaryName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        AccountType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        EncryptedNtOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn NetrLogonSendToSam(
        PrimaryName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        OpaqueBuffer: *mut UCHAR,
        OpaqueBufferSize: ULONG,
    ) -> NTSTATUS;
    pub fn DsrAddressToSiteNamesW(
        ComputerName: LOGONSRV_HANDLE,
        EntryCount: DWORD,
        SocketAddresses: PNL_SOCKET_ADDRESS,
        SiteNames: *mut PNL_SITE_NAME_ARRAY,
    ) -> DWORD;
    pub fn DsrGetDcNameEx2(
        ComputerName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        AllowableAccountControlBits: ULONG,
        DomainName: *mut wchar_t,
        DomainGuid: *mut GUID,
        SiteName: *mut wchar_t,
        Flags: ULONG,
        DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
    ) -> DWORD;
    pub fn NetrLogonGetTimeServiceParentDomain(
        ServerName: LOGONSRV_HANDLE,
        DomainName: *mut *mut wchar_t,
        PdcSameSite: *mut ::core::ffi::c_int,
    ) -> DWORD;
    pub fn NetrEnumerateTrustedDomainsEx(
        ServerName: LOGONSRV_HANDLE,
        Domains: PNETLOGON_TRUSTED_DOMAIN_ARRAY,
    ) -> DWORD;
    pub fn DsrAddressToSiteNamesExW(
        ComputerName: LOGONSRV_HANDLE,
        EntryCount: DWORD,
        SocketAddresses: PNL_SOCKET_ADDRESS,
        SiteNames: *mut PNL_SITE_NAME_EX_ARRAY,
    ) -> DWORD;
    pub fn DsrGetDcSiteCoverageW(
        ServerName: LOGONSRV_HANDLE,
        SiteNames: *mut PNL_SITE_NAME_ARRAY,
    ) -> DWORD;
    pub fn NetrLogonSamLogonEx(
        ContextHandle: handle_t,
        LogonServer: *mut wchar_t,
        ComputerName: *mut wchar_t,
        LogonLevel: NETLOGON_LOGON_INFO_CLASS,
        LogonInformation: PNETLOGON_LEVEL,
        ValidationLevel: NETLOGON_VALIDATION_INFO_CLASS,
        ValidationInformation: PNETLOGON_VALIDATION,
        Authoritative: *mut UCHAR,
        ExtraFlags: *mut ULONG,
    ) -> NTSTATUS;
    pub fn DsrEnumerateDomainTrusts(
        ServerName: LOGONSRV_HANDLE,
        Flags: ULONG,
        Domains: PNETLOGON_TRUSTED_DOMAIN_ARRAY,
    ) -> DWORD;
    pub fn DsrDeregisterDnsHostRecords(
        ServerName: LOGONSRV_HANDLE,
        DnsDomainName: *mut wchar_t,
        DomainGuid: *mut GUID,
        DsaGuid: *mut GUID,
        DnsHostName: *mut wchar_t,
    ) -> DWORD;
    pub fn NetrServerTrustPasswordsGet(
        TrustedDcName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        EncryptedNewOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
        EncryptedOldOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
    ) -> NTSTATUS;
    pub fn DsrGetForestTrustInformation(
        ServerName: LOGONSRV_HANDLE,
        TrustedDomainName: *mut wchar_t,
        Flags: DWORD,
        ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
    ) -> DWORD;
    pub fn NetrGetForestTrustInformation(
        ServerName: LOGONSRV_HANDLE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        Flags: DWORD,
        ForestTrustInfo: *mut PLSA_FOREST_TRUST_INFORMATION,
    ) -> NTSTATUS;
    pub fn NetrLogonSamLogonWithFlags(
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
    ) -> NTSTATUS;
    pub fn NetrServerGetTrustInfo(
        TrustedDcName: LOGONSRV_HANDLE,
        AccountName: *mut wchar_t,
        SecureChannelType: NETLOGON_SECURE_CHANNEL_TYPE,
        ComputerName: *mut wchar_t,
        Authenticator: PNETLOGON_AUTHENTICATOR,
        ReturnAuthenticator: PNETLOGON_AUTHENTICATOR,
        EncryptedNewOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
        EncryptedOldOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
        TrustInfo: *mut PNL_GENERIC_RPC_DATA,
    ) -> NTSTATUS;
}
