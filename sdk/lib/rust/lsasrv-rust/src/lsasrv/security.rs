use crate::prelude::*;
// NtAuthority/WorldSidAuthority are now defined in the Rust database module
// (same crate) — import them rather than re-declaring extern (which would clash
// with the in-crate #[no_mangle] definitions).
use crate::lsasrv::database::{NtAuthority, WorldSidAuthority};
#[no_mangle]
pub unsafe extern "C" fn LsapCreatePolicySd(
    PolicySd: *mut PSECURITY_DESCRIPTOR,
    PolicySdSize: *mut ULONG,
) -> NTSTATUS {
    if PolicySd.is_null() || PolicySdSize.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *PolicySd = core::ptr::null_mut();
    *PolicySdSize = 0;

    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelativeSdSize: ULONG = 0;
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut LocalServiceSid: PSID = core::ptr::null_mut();
    let mut NetworkServiceSid: PSID = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = RtlCreateSecurityDescriptor(
        core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'main: {
        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as PSID_IDENTIFIER_AUTHORITY,
            1u8,
            SECURITY_ANONYMOUS_LOGON_RID,
            0, 0, 0, 0, 0, 0, 0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as PSID_IDENTIFIER_AUTHORITY,
            2u8,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0, 0, 0, 0, 0, 0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(WorldSidAuthority) as PSID_IDENTIFIER_AUTHORITY,
            1u8,
            SECURITY_WORLD_RID,
            0, 0, 0, 0, 0, 0, 0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as PSID_IDENTIFIER_AUTHORITY,
            1u8,
            SECURITY_LOCAL_SERVICE_RID,
            0, 0, 0, 0, 0, 0, 0,
            &mut LocalServiceSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as PSID_IDENTIFIER_AUTHORITY,
            1u8,
            SECURITY_NETWORK_SERVICE_RID,
            0, 0, 0, 0, 0, 0, 0,
            &mut NetworkServiceSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as PSID_IDENTIFIER_AUTHORITY,
            1u8,
            SECURITY_LOCAL_SYSTEM_RID,
            0, 0, 0, 0, 0, 0, 0,
            &mut LocalSystemSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        let DaclSize = (core::mem::size_of::<ACL>()
            + core::mem::size_of::<ACCESS_DENIED_ACE>()  - core::mem::size_of::<ULONG>() + RtlLengthSid(AnonymousSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() - core::mem::size_of::<ULONG>() + RtlLengthSid(AdministratorsSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() - core::mem::size_of::<ULONG>() + RtlLengthSid(EveryoneSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() - core::mem::size_of::<ULONG>() + RtlLengthSid(AnonymousSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() - core::mem::size_of::<ULONG>() + RtlLengthSid(LocalServiceSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() - core::mem::size_of::<ULONG>() + RtlLengthSid(NetworkServiceSid) as usize) as ULONG;

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            DaclSize as SIZE_T,
        ) as PACL;

        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'main;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessDeniedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_LOOKUP_NAMES,
            AnonymousSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_ALL_ACCESS | POLICY_NOTIFICATION,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_EXECUTE,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION,
            AnonymousSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_NOTIFICATION,
            LocalServiceSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            POLICY_NOTIFICATION,
            NetworkServiceSid,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlSetDaclSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
            1u8,
            Dacl,
            0u8,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlSetGroupSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
            LocalSystemSid,
            0u8,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlSetOwnerSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
            AdministratorsSid,
            0u8,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &mut RelativeSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'main;
        }

        RelativeSd = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            RelativeSdSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelativeSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'main;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsoluteSd) as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) { break 'main; }

        *PolicySd = RelativeSd;
        *PolicySdSize = RelativeSdSize;
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }
    if !AnonymousSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AnonymousSid as PVOID);
    }
    if !AdministratorsSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid as PVOID);
    }
    if !EveryoneSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, EveryoneSid as PVOID);
    }
    if !LocalServiceSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalServiceSid as PVOID);
    }
    if !NetworkServiceSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NetworkServiceSid as PVOID);
    }
    if !LocalSystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd as PVOID);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapCreateAccountSd(
    AccountSd: *mut PSECURITY_DESCRIPTOR,
    AccountSdSize: PULONG,
) -> NTSTATUS {
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelativeSdSize: ULONG = 0;
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let mut Status: NTSTATUS;

    if AccountSd.is_null() || AccountSdSize.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    *AccountSd = core::ptr::null_mut();
    *AccountSdSize = 0;

    Status = RtlCreateSecurityDescriptor(
        &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'main: {
        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0, 0, 0, 0, 0, 0,
            &raw mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut WorldSidAuthority,
            1,
            SECURITY_WORLD_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &raw mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1,
            SECURITY_LOCAL_SYSTEM_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &raw mut LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            - core::mem::size_of::<ULONG>()
            + RtlLengthSid(AdministratorsSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            - core::mem::size_of::<ULONG>()
            + RtlLengthSid(EveryoneSid) as usize) as ULONG;

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            DaclSize as SIZE_T,
        ) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'main;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            ACCOUNT_ALL_ACCESS,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            ACCOUNT_EXECUTE,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlSetDaclSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            1u8,
            Dacl,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlSetGroupSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            LocalSystemSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlSetOwnerSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            AdministratorsSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'main;
        }

        RelativeSd = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            RelativeSdSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelativeSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'main;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'main;
        }

        *AccountSd = RelativeSd;
        *AccountSdSize = RelativeSdSize;
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }
    if !AdministratorsSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid as PVOID);
    }
    if !EveryoneSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, EveryoneSid as PVOID);
    }
    if !LocalSystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid as PVOID);
    }
    if !NT_SUCCESS(Status) {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd as PVOID);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapCreateSecretSd(
    SecretSd: *mut PSECURITY_DESCRIPTOR,
    SecretSdSize: PULONG,
) -> NTSTATUS {
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = _SECURITY_DESCRIPTOR {
        Revision: 0,
        Sbz1: 0,
        Control: 0,
        Owner: core::ptr::null_mut(),
        Group: core::ptr::null_mut(),
        Sacl: core::ptr::null_mut(),
        Dacl: core::ptr::null_mut(),
    };
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelativeSdSize: ULONG = 0;
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let mut Status: NTSTATUS;

    if SecretSd.is_null() || SecretSdSize.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    *SecretSd = core::ptr::null_mut();
    *SecretSdSize = 0;

    Status = RtlCreateSecurityDescriptor(
        &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = loop {
        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2u8,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            &raw mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut WorldSidAuthority,
            1u8,
            SECURITY_WORLD_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &raw mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1u8,
            SECURITY_LOCAL_SYSTEM_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &raw mut LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            .wrapping_add(core::mem::size_of::<ACCESS_ALLOWED_ACE>())
            .wrapping_sub(core::mem::size_of::<ULONG>())
            .wrapping_add(RtlLengthSid(AdministratorsSid) as usize)
            .wrapping_add(core::mem::size_of::<ACCESS_ALLOWED_ACE>())
            .wrapping_sub(core::mem::size_of::<ULONG>())
            .wrapping_add(RtlLengthSid(EveryoneSid) as usize)) as ULONG;

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap() as HANDLE,
            HEAP_ZERO_MEMORY as ULONG,
            DaclSize as SIZE_T,
        ) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break Status;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break Status;
        }
        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            SECRET_ALL_ACCESS as ACCESS_MASK,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }
        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            SECRET_EXECUTE as ACCESS_MASK,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        Status = RtlSetDaclSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            1u8,
            Dacl,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }
        Status = RtlSetGroupSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            LocalSystemSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }
        Status = RtlSetOwnerSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            AdministratorsSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break Status;
        }

        RelativeSd = RtlAllocateHeap(
            RtlGetProcessHeap() as HANDLE,
            HEAP_ZERO_MEMORY as ULONG,
            RelativeSdSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelativeSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break Status;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break Status;
        }

        *SecretSd = RelativeSd;
        *SecretSdSize = RelativeSdSize;
        break Status;
    };

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, Dacl as PVOID);
    }
    if !AdministratorsSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, AdministratorsSid as PVOID);
    }
    if !EveryoneSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, EveryoneSid as PVOID);
    }
    if !LocalSystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, LocalSystemSid as PVOID);
    }
    if !NT_SUCCESS(Status) {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, RelativeSd as PVOID);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn LsapCreateTokenSd(
    User: *const TOKEN_USER,
    TokenSd: *mut PSECURITY_DESCRIPTOR,
    TokenSdSize: PULONG,
) -> NTSTATUS {
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelativeSdSize: ULONG = 0;
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let mut Status: NTSTATUS;

    if TokenSd.is_null() || TokenSdSize.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    *TokenSd = core::ptr::null_mut();
    *TokenSdSize = 0;

    Status = RtlCreateSecurityDescriptor(
        &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = 'block: {
        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1u8,
            SECURITY_LOCAL_SYSTEM_RID,
            0, 0, 0, 0, 0, 0, 0,
            &raw mut LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2u8,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0, 0, 0, 0, 0, 0,
            &raw mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            + RtlLengthSid(LocalSystemSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            + RtlLengthSid(AdministratorsSid) as usize
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            + RtlLengthSid((*User).User.Sid) as usize) as ULONG;

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            DaclSize as SIZE_T,
        ) as PACL;
        if Dacl.is_null() {
            break 'block STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION);
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            TOKEN_ALL_ACCESS,
            LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            TOKEN_READ,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            TOKEN_ALL_ACCESS,
            (*User).User.Sid,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlSetDaclSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            1u8,
            Dacl,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlSetOwnerSecurityDescriptor(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            AdministratorsSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'block Status;
        }

        RelativeSd = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            RelativeSdSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelativeSd.is_null() {
            break 'block STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            &raw mut AbsoluteSd as PSECURITY_DESCRIPTOR,
            RelativeSd,
            &raw mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'block Status;
        }

        *TokenSd = RelativeSd;
        *TokenSdSize = RelativeSdSize;
        Status
    };

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }
    if !AdministratorsSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid as PVOID);
    }
    if !LocalSystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid as PVOID);
    }
    if !NT_SUCCESS(Status) {
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd as PVOID);
        }
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/security.c
//  * PURPOSE:     LSA object security functions
//  * COPYRIGHT:   Copyright 2012 Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// LsapCreatePolicySd(PSECURITY_DESCRIPTOR *PolicySd,
//                    PULONG PolicySdSize)
// {
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PSID EveryoneSid = NULL;
//     PSID LocalServiceSid = NULL;
//     PSID NetworkServiceSid = NULL;
//     PSID LocalSystemSid = NULL;
//     PACL Dacl = NULL;
//     ULONG DaclSize;
//     NTSTATUS Status;
//
//     if (PolicySd == NULL || PolicySdSize == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *PolicySd = NULL;
//     *PolicySdSize = 0;
//
//     /* Initialize the SD */
//     Status = RtlCreateSecurityDescriptor(&AbsoluteSd,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_ANONYMOUS_LOGON_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AnonymousSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&WorldSidAuthority,
//                                          1,
//                                          SECURITY_WORLD_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SERVICE_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &LocalServiceSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_NETWORK_SERVICE_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &NetworkServiceSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &LocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate and initialize the DACL */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_DENIED_ACE)  - sizeof(ULONG) + RtlLengthSid(AnonymousSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(AdministratorsSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(EveryoneSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(AnonymousSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(LocalServiceSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(NetworkServiceSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessDeniedAce(Dacl,
//                                    ACL_REVISION,
//                                    POLICY_LOOKUP_NAMES,
//                                    AnonymousSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     POLICY_ALL_ACCESS | POLICY_NOTIFICATION,
//                                     AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     POLICY_EXECUTE,
//                                     EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION,
//                                     AnonymousSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     POLICY_NOTIFICATION,
//                                     LocalServiceSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     POLICY_NOTIFICATION,
//                                     NetworkServiceSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetDaclSecurityDescriptor(&AbsoluteSd,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetGroupSecurityDescriptor(&AbsoluteSd,
//                                            LocalSystemSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetOwnerSecurityDescriptor(&AbsoluteSd,
//                                            AdministratorsSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  RelativeSdSize);
//     if (RelativeSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *PolicySd = RelativeSd;
//     *PolicySdSize = RelativeSdSize;
//
// done:
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (AnonymousSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid);
//
//     if (EveryoneSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EveryoneSid);
//
//     if (LocalServiceSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalServiceSid);
//
//     if (NetworkServiceSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NetworkServiceSid);
//
//     if (LocalSystemSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelativeSd != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapCreateAccountSd(PSECURITY_DESCRIPTOR *AccountSd,
//                     PULONG AccountSdSize)
// {
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     PSID AdministratorsSid = NULL;
//     PSID EveryoneSid = NULL;
//     PSID LocalSystemSid = NULL;
//     PACL Dacl = NULL;
//     ULONG DaclSize;
//     NTSTATUS Status;
//
//     if (AccountSd == NULL || AccountSdSize == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *AccountSd = NULL;
//     *AccountSdSize = 0;
//
//     /* Initialize the SD */
//     Status = RtlCreateSecurityDescriptor(&AbsoluteSd,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&WorldSidAuthority,
//                                          1,
//                                          SECURITY_WORLD_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &LocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate and initialize the DACL */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(AdministratorsSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(EveryoneSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     ACCOUNT_ALL_ACCESS,
//                                     AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     ACCOUNT_EXECUTE,
//                                     EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetDaclSecurityDescriptor(&AbsoluteSd,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetGroupSecurityDescriptor(&AbsoluteSd,
//                                            LocalSystemSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetOwnerSecurityDescriptor(&AbsoluteSd,
//                                            AdministratorsSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  RelativeSdSize);
//     if (RelativeSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *AccountSd = RelativeSd;
//     *AccountSdSize = RelativeSdSize;
//
// done:
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid);
//
//     if (EveryoneSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EveryoneSid);
//
//     if (LocalSystemSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelativeSd != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapCreateSecretSd(PSECURITY_DESCRIPTOR *SecretSd,
//                    PULONG SecretSdSize)
// {
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     PSID AdministratorsSid = NULL;
//     PSID EveryoneSid = NULL;
//     PSID LocalSystemSid = NULL;
//     PACL Dacl = NULL;
//     ULONG DaclSize;
//     NTSTATUS Status;
//
//     if (SecretSd == NULL || SecretSdSize == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *SecretSd = NULL;
//     *SecretSdSize = 0;
//
//     /* Initialize the SD */
//     Status = RtlCreateSecurityDescriptor(&AbsoluteSd,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&WorldSidAuthority,
//                                          1,
//                                          SECURITY_WORLD_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &LocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate and initialize the DACL */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(AdministratorsSid) +
//                sizeof(ACCESS_ALLOWED_ACE) - sizeof(ULONG) + RtlLengthSid(EveryoneSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SECRET_ALL_ACCESS,
//                                     AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SECRET_EXECUTE,
//                                     EveryoneSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetDaclSecurityDescriptor(&AbsoluteSd,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetGroupSecurityDescriptor(&AbsoluteSd,
//                                            LocalSystemSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetOwnerSecurityDescriptor(&AbsoluteSd,
//                                            AdministratorsSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  RelativeSdSize);
//     if (RelativeSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *SecretSd = RelativeSd;
//     *SecretSdSize = RelativeSdSize;
//
// done:
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid);
//
//     if (EveryoneSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EveryoneSid);
//
//     if (LocalSystemSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelativeSd != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
//     }
//
//     return Status;
// }
//
//
// /**
//  * @brief
//  * Creates a security descriptor for the token
//  * object.
//  *
//  * @param[in] User
//  * A primary user to be given to the function.
//  * This user represents the owner that is in
//  * charge of this object.
//  *
//  * @param[out] TokenSd
//  * A pointer to an allocated security descriptor
//  * for the token object.
//  *
//  * @param[out] TokenSdSize
//  * A pointer to a returned size of the descriptor.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the function has
//  * successfully created the security descriptor.
//  * STATUS_INVALID_PARAMETER is returned if one of the
//  * parameters are not valid. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if memory heap allocation for specific
//  * security buffers couldn't be done. A NTSTATUS status
//  * code is returned otherwise.
//  *
//  * @remarks
//  * Bot the local system and user are given full access rights
//  * for the token (they can open it, read and write into it, etc.)
//  * whereas admins can only read from the token. This security
//  * descriptor is TO NOT BE confused with the default DACL of the
//  * token which is another thing that serves different purpose.
//  */
// NTSTATUS
// LsapCreateTokenSd(
//      const TOKEN_USER *User,
//      PSECURITY_DESCRIPTOR *TokenSd,
//      PULONG TokenSdSize)
// {
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     PSID AdministratorsSid = NULL;
//     PSID LocalSystemSid = NULL;
//     PACL Dacl = NULL;
//     ULONG DaclSize;
//     NTSTATUS Status;
//
//     if (TokenSd == NULL || TokenSdSize == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *TokenSd = NULL;
//     *TokenSdSize = 0;
//
//     /* Initialize the SD */
//     Status = RtlCreateSecurityDescriptor(&AbsoluteSd,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &LocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0, 0, 0, 0, 0, 0,
//                                          &AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate and initialize the DACL */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_ALLOWED_ACE) + RtlLengthSid(LocalSystemSid) +
//                sizeof(ACCESS_ALLOWED_ACE) + RtlLengthSid(AdministratorsSid) +
//                sizeof(ACCESS_ALLOWED_ACE) + RtlLengthSid(User->User.Sid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     TOKEN_ALL_ACCESS,
//                                     LocalSystemSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     TOKEN_READ,
//                                     AdministratorsSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     TOKEN_ALL_ACCESS,
//                                     User->User.Sid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetDaclSecurityDescriptor(&AbsoluteSd,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlSetOwnerSecurityDescriptor(&AbsoluteSd,
//                                            AdministratorsSid,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  RelativeSdSize);
//     if (RelativeSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = RtlAbsoluteToSelfRelativeSD(&AbsoluteSd,
//                                          RelativeSd,
//                                          &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *TokenSd = RelativeSd;
//     *TokenSdSize = RelativeSdSize;
//
// done:
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdministratorsSid);
//
//     if (LocalSystemSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSystemSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelativeSd != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
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
