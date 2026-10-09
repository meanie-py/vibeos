use crate::prelude::*;

// ---- file-scope preamble (translated) ----
static mut WorldAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 1],
};
static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};
#[no_mangle]
pub unsafe extern "C" fn SampCreateServerSD(
    ServerSd: *mut PSECURITY_DESCRIPTOR,
    Size: PULONG,
) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    // ntsam.h:99-117. SAM_SERVER_CREATE_DOMAIN is 0x0008, not 0x0004 -- at 0x0004
    // it COLLIDED with SAM_SERVER_INITIALIZE below, so every
    // `SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE` in this file collapsed
    // to a single bit and the ACE granted one right where two were intended.
    const SAM_SERVER_READ: u32 = 0x00020010; // STANDARD_RIGHTS_READ | ENUMERATE_DOMAINS
    const SAM_SERVER_EXECUTE: u32 = 0x00020021; // STANDARD_RIGHTS_EXECUTE | CONNECT | LOOKUP_DOMAIN
    const SAM_SERVER_ALL_ACCESS: u32 = 0x000F003F;
    const SAM_SERVER_CREATE_DOMAIN: u32 = 0x0008;
    // ddk/ntsam.h:100-101. Both were shifted one bit left of the real values.
    const SAM_SERVER_INITIALIZE: u32 = 0x0004;
    const SAM_SERVER_SHUTDOWN: u32 = 0x0002;

    'done: {
        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(WorldAuthority) as *mut _,
            1,
            SECURITY_WORLD_RID as u32,
            0, 0, 0, 0, 0, 0, 0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as *mut _,
            1,
            SECURITY_ANONYMOUS_LOGON_RID as u32,
            0, 0, 0, 0, 0, 0, 0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of!(NtAuthority) as *mut _,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0, 0, 0, 0, 0, 0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AbsSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()
            + RtlLengthSid(EveryoneSid) as usize
            + RtlLengthSid(AdministratorsSid) as usize) as ULONG;

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            DaclSize as SIZE_T,
        ) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as u32,
            SAM_SERVER_READ | SAM_SERVER_EXECUTE,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as u32,
            SAM_SERVER_ALL_ACCESS,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()
            + RtlLengthSid(EveryoneSid) as usize
            + RtlLengthSid(AnonymousSid) as usize) as ULONG;

        Sacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            DaclSize as SIZE_T,
        ) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl, SaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE
                | SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE
                | SAM_SERVER_SHUTDOWN,
            EveryoneSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
            AnonymousSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetSaclSecurityDescriptor(AbsSD, 1u8, Sacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetGroupSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        RelSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            RelSDSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, RelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *ServerSd = RelSD;
        *Size = RelSDSize;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateBuiltinDomainSD(
    ServerSd: *mut PSECURITY_DESCRIPTOR,
    Size: *mut ULONG,
) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: {
        Status = RtlAllocateAndInitializeSid(
            &raw mut WorldAuthority,
            1,
            SECURITY_WORLD_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1,
            SECURITY_ANONYMOUS_LOGON_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0, 0, 0, 0, 0, 0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AbsSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        );
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()) as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AdministratorsSid);

        Dacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            DOMAIN_READ | DOMAIN_EXECUTE,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            SAM_SERVER_ALL_ACCESS,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()) as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AnonymousSid);

        Sacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl, SaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as ULONG,
            ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE
                | SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE
                | SAM_SERVER_SHUTDOWN,
            EveryoneSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as ULONG,
            STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
            AnonymousSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetSaclSecurityDescriptor(AbsSD, 1u8, Sacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetGroupSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        RelSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, RelSDSize as SIZE_T);
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, RelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *ServerSd = RelSD;
        *Size = RelSDSize;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateAccountDomainSD(ServerSd: *mut PSECURITY_DESCRIPTOR, Size: PULONG) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut UsersSid: PSID = core::ptr::null_mut();
    let mut GuestsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    // ntsam.h:33-72. CREATE_USER (0x10) and CREATE_GROUP (0x20) were SWAPPED,
    // so every ACE naming one granted the other.
    const DOMAIN_READ: ULONG = 0x00020084;
    const DOMAIN_EXECUTE: ULONG = 0x00020301; // STANDARD_RIGHTS_EXECUTE | READ_PASSWORD_PARAMETERS | LIST_ACCOUNTS | LOOKUP
    const DOMAIN_ALL_ACCESS: ULONG = 0x000F07FF;
    const DOMAIN_CREATE_USER: ULONG = 0x0010;
    const DOMAIN_CREATE_GROUP: ULONG = 0x0020;
    const DOMAIN_CREATE_ALIAS: ULONG = 0x0040;
    const SAM_SERVER_CREATE_DOMAIN: ULONG = 0x0008;
    // ddk/ntsam.h:101 -- INITIALIZE is 0x04 (0x01 is SAM_SERVER_CONNECT).
    const SAM_SERVER_INITIALIZE: ULONG = 0x0004;
    const SAM_SERVER_SHUTDOWN: ULONG = 0x0002;

    'done: {
        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(WorldAuthority),
            1,
            SECURITY_WORLD_RID as u32,
            0, 0, 0, 0, 0, 0, 0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            1,
            SECURITY_ANONYMOUS_LOGON_RID as u32,
            0, 0, 0, 0, 0, 0, 0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0, 0, 0, 0, 0, 0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_USERS as u32,
            0, 0, 0, 0, 0, 0,
            &mut UsersSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_GUESTS as u32,
            0, 0, 0, 0, 0, 0,
            &mut GuestsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AbsSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        );
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + 4 * core::mem::size_of::<ACE>()) as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AdministratorsSid)
            + RtlLengthSid(UsersSid)
            + RtlLengthSid(GuestsSid);

        Dacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION as u32, DOMAIN_READ | DOMAIN_EXECUTE, EveryoneSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION as u32, DOMAIN_READ | DOMAIN_EXECUTE, UsersSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION as u32, DOMAIN_ALL_ACCESS & !DOMAIN_CREATE_GROUP, AdministratorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION as u32, DOMAIN_READ | DOMAIN_EXECUTE | DOMAIN_CREATE_USER | DOMAIN_CREATE_ALIAS, GuestsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()) as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AnonymousSid);

        Sacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl, SaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE
                | SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE | SAM_SERVER_SHUTDOWN,
            EveryoneSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
            AnonymousSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetSaclSecurityDescriptor(AbsSD, 1u8, Sacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetGroupSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        RelSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, RelSDSize as SIZE_T);
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, RelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *ServerSd = RelSD;
        *Size = RelSDSize;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateAliasSD(AliasSd: *mut PSECURITY_DESCRIPTOR,
                            Size: PULONG) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut AccountOperatorsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: {
        /* Create the Everyone SID */
        Status = RtlAllocateAndInitializeSid(core::ptr::addr_of_mut!(WorldAuthority),
                                             1,
                                             SECURITY_WORLD_RID as ULONG,
                                             0, 0, 0, 0, 0, 0, 0,
                                             &mut EveryoneSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Create the Anonymous SID */
        Status = RtlAllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                             1,
                                             SECURITY_ANONYMOUS_LOGON_RID as ULONG,
                                             0, 0, 0, 0, 0, 0, 0,
                                             &mut AnonymousSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Create the Administrators SID */
        Status = RtlAllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                             2,
                                             SECURITY_BUILTIN_DOMAIN_RID as ULONG,
                                             DOMAIN_ALIAS_RID_ADMINS as ULONG,
                                             0, 0, 0, 0, 0, 0,
                                             &mut AdministratorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Create the Account Operators SID */
        Status = RtlAllocateAndInitializeSid(core::ptr::addr_of_mut!(NtAuthority),
                                             2,
                                             SECURITY_BUILTIN_DOMAIN_RID as ULONG,
                                             DOMAIN_ALIAS_RID_ACCOUNT_OPS as ULONG,
                                             0, 0, 0, 0, 0, 0,
                                             &mut AccountOperatorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Allocate a buffer for the absolute SD */
        AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
                                HEAP_ZERO_MEMORY as ULONG,
                                core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T);
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Create the absolute SD */
        Status = RtlCreateSecurityDescriptor(AbsSD,
                                             SECURITY_DESCRIPTOR_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* allocate and create the DACL */
        DaclSize = core::mem::size_of::<ACL>() as ULONG +
                   3 * core::mem::size_of::<ACE>() as ULONG +
                   RtlLengthSid(EveryoneSid) +
                   RtlLengthSid(AdministratorsSid) +
                   RtlLengthSid(AccountOperatorsSid);

        Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY as ULONG,
                               DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl,
                              DaclSize,
                              ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl,
                                        ACL_REVISION as ULONG,
                                        READ_CONTROL | ALIAS_READ_INFORMATION | ALIAS_LIST_MEMBERS,
                                        EveryoneSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl,
                                        ACL_REVISION as ULONG,
                                        ALIAS_ALL_ACCESS,
                                        AdministratorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl,
                                        ACL_REVISION as ULONG,
                                        ALIAS_ALL_ACCESS,
                                        AccountOperatorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Set the DACL */
        Status = RtlSetDaclSecurityDescriptor(AbsSD,
                                              1u8,
                                              Dacl,
                                              0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* allocate and create the SACL */
        SaclSize = core::mem::size_of::<ACL>() as ULONG +
                   2 * core::mem::size_of::<ACE>() as ULONG +
                   RtlLengthSid(EveryoneSid) +
                   RtlLengthSid(AnonymousSid);

        Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
                               HEAP_ZERO_MEMORY as ULONG,
                               DaclSize as SIZE_T) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl,
                              SaclSize,
                              ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(Sacl,
                                      ACL_REVISION as ULONG,
                                      ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
                                      ALIAS_WRITE_ACCOUNT | ALIAS_REMOVE_MEMBER |
                                      ALIAS_ADD_MEMBER,
                                      EveryoneSid,
                                      1u8,
                                      1u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(Sacl,
                                      ACL_REVISION as ULONG,
                                      STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
                                      AnonymousSid,
                                      1u8,
                                      1u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Set the SACL */
        Status = RtlSetSaclSecurityDescriptor(AbsSD,
                                              1u8,
                                              Sacl,
                                              0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Set the owner SID */
        Status = RtlSetOwnerSecurityDescriptor(AbsSD,
                                               AdministratorsSid,
                                               0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Set the group SID */
        Status = RtlSetGroupSecurityDescriptor(AbsSD,
                                               AdministratorsSid,
                                               0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Get the required buffer size for the self-relative SD */
        Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
                                             core::ptr::null_mut(),
                                             &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        /* Allocate a buffer for the self-relative SD */
        RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
                                HEAP_ZERO_MEMORY as ULONG,
                                RelSDSize as SIZE_T);
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Convert the absolute SD to self-relative format */
        Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
                                             RelSD,
                                             &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *AliasSd = RelSD;
        *Size = RelSDSize;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateGroupSD(
    GroupSd: *mut PSECURITY_DESCRIPTOR,
    Size: *mut ULONG,
) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut AccountOperatorsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    // ntsam.h:75-97. This family had been PERMUTED into the ALIAS_* numbering
    // (ntsam.h:9-13), which is a different set of rights on a different object
    // type. GROUP_ALL_ACCESS happened to come out right because it is the OR of
    // all five bits and a permutation preserves that -- which is exactly why a
    // whole-mask check cannot detect this class.
    const GROUP_READ_INFORMATION: ULONG = 0x0001;
    const GROUP_WRITE_ACCOUNT: ULONG = 0x0002;
    const GROUP_ADD_MEMBER: ULONG = 0x0004;
    const GROUP_REMOVE_MEMBER: ULONG = 0x0008;
    const GROUP_LIST_MEMBERS: ULONG = 0x0010;
    const GROUP_ALL_ACCESS: ULONG = 0x000F0000 | 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010;

    'done: loop {
        Status = RtlAllocateAndInitializeSid(
            &raw mut WorldAuthority,
            1,
            SECURITY_WORLD_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1,
            SECURITY_ANONYMOUS_LOGON_RID as ULONG,
            0, 0, 0, 0, 0, 0, 0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0, 0, 0, 0, 0, 0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ACCOUNT_OPS as ULONG,
            0, 0, 0, 0, 0, 0,
            &mut AccountOperatorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AbsSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        );
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        DaclSize = core::mem::size_of::<ACL>() as ULONG
            + 3 * core::mem::size_of::<ACE>() as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AdministratorsSid)
            + RtlLengthSid(AccountOperatorsSid);

        Dacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            READ_CONTROL | GROUP_LIST_MEMBERS | GROUP_READ_INFORMATION,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            GROUP_ALL_ACCESS,
            AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            GROUP_ALL_ACCESS,
            AccountOperatorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SaclSize = core::mem::size_of::<ACL>() as ULONG
            + 2 * core::mem::size_of::<ACE>() as ULONG
            + RtlLengthSid(EveryoneSid)
            + RtlLengthSid(AnonymousSid);

        Sacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl, SaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as ULONG,
            ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE
                | GROUP_REMOVE_MEMBER | GROUP_ADD_MEMBER | GROUP_WRITE_ACCOUNT,
            EveryoneSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as ULONG,
            STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
            AnonymousSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetSaclSecurityDescriptor(AbsSD, 1u8, Sacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetGroupSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        RelSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, RelSDSize as SIZE_T);
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, RelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *GroupSd = RelSD;
        *Size = RelSDSize;

        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateUserSD(
    UserSid: PSID,
    UserSd: *mut PSECURITY_DESCRIPTOR,
    Size: PULONG,
) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut AnonymousSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Sacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let SaclSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: {
        Status = RtlAllocateAndInitializeSid(
            &raw mut WorldAuthority,
            1,
            SECURITY_WORLD_RID as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            1,
            SECURITY_ANONYMOUS_LOGON_RID as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut AnonymousSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &raw mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut AdministratorsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        AbsSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
        );
        if AbsSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + 3 * core::mem::size_of::<ACE>()
            + RtlLengthSid(EveryoneSid) as usize
            + RtlLengthSid(AdministratorsSid) as usize
            + RtlLengthSid(UserSid) as usize) as ULONG;

        Dacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as u32,
            READ_CONTROL
                | USER_READ_GROUP_INFORMATION
                | USER_LIST_GROUPS
                | USER_CHANGE_PASSWORD
                | USER_READ_ACCOUNT
                | USER_READ_LOGON
                | USER_READ_PREFERENCES
                | USER_READ_GENERAL,
            EveryoneSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION as u32, USER_ALL_ACCESS, AdministratorsSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as u32,
            READ_CONTROL | USER_CHANGE_PASSWORD | USER_WRITE_PREFERENCES,
            UserSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SaclSize = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACE>()
            + RtlLengthSid(EveryoneSid) as usize
            + RtlLengthSid(AnonymousSid) as usize) as ULONG;

        Sacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Sacl.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlCreateAcl(Sacl, SaclSize, ACL_REVISION as u32);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE | USER_CHANGE_PASSWORD | USER_WRITE_PREFERENCES,
            EveryoneSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAddAuditAccessAce(
            Sacl,
            ACL_REVISION as u32,
            STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
            AnonymousSid,
            1u8,
            1u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetSaclSecurityDescriptor(AbsSD, 1u8, Sacl, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlSetGroupSecurityDescriptor(AbsSD, AdministratorsSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        RelSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, RelSDSize as SIZE_T);
        if RelSD.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, RelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        *UserSd = RelSD;
        *Size = RelSDSize;
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
        }
    }

    if !EveryoneSid.is_null() {
        RtlFreeSid(EveryoneSid);
    }

    if !AnonymousSid.is_null() {
        RtlFreeSid(AnonymousSid);
    }

    if !AdministratorsSid.is_null() {
        RtlFreeSid(AdministratorsSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    if !Sacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl as PVOID);
    }

    if !AbsSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/security.c
//  * PURPOSE:         Security descriptor functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* GLOBALS *****************************************************************/
//
// static SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
// static SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampCreateServerSD(OUT PSECURITY_DESCRIPTOR *ServerSd,
//                    OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SAM_SERVER_READ | SAM_SERVER_EXECUTE,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SAM_SERVER_ALL_ACCESS,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE |
//                                   SAM_SERVER_SHUTDOWN,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *ServerSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateBuiltinDomainSD(OUT PSECURITY_DESCRIPTOR *ServerSd,
//                           OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DOMAIN_READ | DOMAIN_EXECUTE,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SAM_SERVER_ALL_ACCESS,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE |
//                                   SAM_SERVER_SHUTDOWN,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *ServerSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateAccountDomainSD(OUT PSECURITY_DESCRIPTOR *ServerSd,
//                           OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PSID UsersSid = NULL;
//     PSID GuestsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Users SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_USERS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &UsersSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Guests SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_GUESTS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &GuestsSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                4 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid) +
//                RtlLengthSid(UsersSid) +
//                RtlLengthSid(GuestsSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DOMAIN_READ | DOMAIN_EXECUTE,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DOMAIN_READ | DOMAIN_EXECUTE,
//                                     UsersSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DOMAIN_ALL_ACCESS & ~DOMAIN_CREATE_GROUP,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DOMAIN_READ | DOMAIN_EXECUTE | DOMAIN_CREATE_USER | DOMAIN_CREATE_ALIAS,
//                                     GuestsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   SAM_SERVER_CREATE_DOMAIN | SAM_SERVER_INITIALIZE |
//                                   SAM_SERVER_SHUTDOWN,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *ServerSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateAliasSD(OUT PSECURITY_DESCRIPTOR *AliasSd,
//                   OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PSID AccountOperatorsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Account Operators SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ACCOUNT_OPS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AccountOperatorsSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                3 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid) +
//                RtlLengthSid(AccountOperatorsSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     READ_CONTROL | ALIAS_READ_INFORMATION | ALIAS_LIST_MEMBERS,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     ALIAS_ALL_ACCESS,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     ALIAS_ALL_ACCESS,
//                                     AccountOperatorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   ALIAS_WRITE_ACCOUNT | ALIAS_REMOVE_MEMBER |
//                                   ALIAS_ADD_MEMBER,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *AliasSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateGroupSD(OUT PSECURITY_DESCRIPTOR *GroupSd,
//                   OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PSID AccountOperatorsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Account Operators SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ACCOUNT_OPS,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &AccountOperatorsSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                3 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid) +
//                RtlLengthSid(AccountOperatorsSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     READ_CONTROL | GROUP_LIST_MEMBERS | GROUP_READ_INFORMATION,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     GROUP_ALL_ACCESS,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     GROUP_ALL_ACCESS,
//                                     AccountOperatorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   GROUP_REMOVE_MEMBER | GROUP_ADD_MEMBER |
//                                   GROUP_WRITE_ACCOUNT,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *GroupSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateUserSD(IN PSID UserSid,
//                  OUT PSECURITY_DESCRIPTOR *UserSd,
//                  OUT PULONG Size)
// {
//     PSECURITY_DESCRIPTOR AbsSD = NULL;
//     PSECURITY_DESCRIPTOR RelSD = NULL;
//     PSID EveryoneSid = NULL;
//     PSID AnonymousSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PACL Dacl = NULL;
//     PACL Sacl = NULL;
//     ULONG DaclSize;
//     ULONG SaclSize;
//     ULONG RelSDSize = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     /* Create the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Anonymous SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Administrators SID */
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
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate a buffer for the absolute SD */
//     AbsSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             sizeof(SECURITY_DESCRIPTOR));
//     if (AbsSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Create the absolute SD */
//     Status = RtlCreateSecurityDescriptor(AbsSD,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the DACL */
//     DaclSize = sizeof(ACL) +
//                3 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AdministratorsSid) +
//                RtlLengthSid(UserSid);
//
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Dacl,
//                           DaclSize,
//                           ACL_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     READ_CONTROL | USER_READ_GROUP_INFORMATION | USER_LIST_GROUPS |
//                                     USER_CHANGE_PASSWORD | USER_READ_ACCOUNT | USER_READ_LOGON |
//                                     USER_READ_PREFERENCES | USER_READ_GENERAL,
//                                     EveryoneSid);
//     ASSERT(NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     USER_ALL_ACCESS,
//                                     AdministratorsSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     READ_CONTROL | USER_CHANGE_PASSWORD | USER_WRITE_PREFERENCES,
//                                     UserSid);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* allocate and create the SACL */
//     SaclSize = sizeof(ACL) +
//                2 * sizeof(ACE) +
//                RtlLengthSid(EveryoneSid) +
//                RtlLengthSid(AnonymousSid);
//
//     Sacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Sacl == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     Status = RtlCreateAcl(Sacl,
//                           SaclSize,
//                           ACL_REVISION);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   ACCESS_SYSTEM_SECURITY | WRITE_DAC | DELETE |
//                                   USER_CHANGE_PASSWORD | USER_WRITE_PREFERENCES,
//                                   EveryoneSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = RtlAddAuditAccessAce(Sacl,
//                                   ACL_REVISION,
//                                   STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL,
//                                   AnonymousSid,
//                                   TRUE,
//                                   TRUE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SACL */
//     Status = RtlSetSaclSecurityDescriptor(AbsSD,
//                                           TRUE,
//                                           Sacl,
//                                           FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the owner SID */
//     Status = RtlSetOwnerSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the group SID */
//     Status = RtlSetGroupSecurityDescriptor(AbsSD,
//                                            AdministratorsSid,
//                                            FALSE);
//     ASSERT(Status == STATUS_SUCCESS);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the required buffer size for the self-relative SD */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          NULL,
//                                          &RelSDSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the self-relative SD */
//     RelSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             RelSDSize);
//     if (RelSD == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     /* Convert the absolute SD to self-relative format */
//     Status = RtlAbsoluteToSelfRelativeSD(AbsSD,
//                                          RelSD,
//                                          &RelSDSize);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//     ASSERT(Status == STATUS_SUCCESS);
//         goto done;
//     }
//
//     *UserSd = RelSD;
//     *Size = RelSDSize;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (RelSD != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelSD);
//     }
//
//     if (EveryoneSid != NULL)
//         RtlFreeSid(EveryoneSid);
//
//     if (AnonymousSid != NULL)
//         RtlFreeSid(AnonymousSid);
//
//     if (AdministratorsSid != NULL)
//         RtlFreeSid(AdministratorsSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     if (Sacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sacl);
//
//     if (AbsSD != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AbsSD);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
