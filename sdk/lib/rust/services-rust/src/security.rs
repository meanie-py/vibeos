//! security.c — the SCM's security descriptors. Builds the well-known SIDs,
//! the default service DACL/SACL + control-pipe DACL, the default service SD
//! and the pipe SD, and converts the default SD to self-relative on demand.
//! Faithful translation: the static helpers stay private Rust fns; the 3 entry
//! points (ScmCreateDefaultServiceSD / ScmInitializeSecurity /
//! ScmShutdownSecurity) are cdecl `#[no_mangle]`. `pPipeSD` is an exported
//! global (rpcserver.c reads it), so it keeps `#[no_mangle]`.

use core::ptr::null_mut;
use winsrv_sys::winsrv::*;
// RtlGetProcessHeap (a hand-written PEB inline) + STATUS_BUFFER_TOO_SMALL live
// in winsrv_sys::nt, not the winsrv glob — import them explicitly (an explicit
// `use` shadows the glob, so no E0659 even if a name overlapped).
use winsrv_sys::nt::{RtlGetProcessHeap, STATUS_BUFFER_TOO_SMALL};

use crate::types::{
    SERVICE_ALL_ACCESS, SERVICE_ENUMERATE_DEPENDENTS, SERVICE_INTERROGATE,
    SERVICE_PAUSE_CONTINUE, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STOP, SERVICE_USER_DEFINED_CONTROL,
};

#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// File-local well-known SIDs / ACLs / SDs.
static mut pNullSid: PSID = null_mut();
static mut pWorldSid: PSID = null_mut();
static mut pLocalSystemSid: PSID = null_mut();
static mut pAuthenticatedUserSid: PSID = null_mut();
static mut pAliasAdminsSid: PSID = null_mut();

static mut pDefaultDacl: PACL = null_mut();
static mut pDefaultSacl: PACL = null_mut();
static mut pPipeDacl: PACL = null_mut();

static mut pDefaultSD: PSECURITY_DESCRIPTOR = null_mut();

/// Exported (rpcserver.c references it): the control-pipe security descriptor.
#[no_mangle]
pub static mut pPipeSD: PSECURITY_DESCRIPTOR = null_mut();

unsafe fn ScmFreeSids() {
    if !pNullSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pNullSid);
    }
    if !pWorldSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pWorldSid);
    }
    if !pLocalSystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pLocalSystemSid);
    }
    if !pAuthenticatedUserSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pAuthenticatedUserSid);
    }
    if !pAliasAdminsSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pAliasAdminsSid);
    }
}

unsafe fn ScmCreateSids() -> DWORD {
    // {SECURITY_NULL_SID_AUTHORITY}/{WORLD}/{NT} = 6-byte identifier authorities.
    let mut NullAuthority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 0] };
    let mut WorldAuthority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 1] };
    let mut NtAuthority = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] };
    let mut pSubAuthority: PULONG;
    let ulLength1: ULONG = RtlLengthRequiredSid(1);
    let ulLength2: ULONG = RtlLengthRequiredSid(2);

    /* Null SID */
    pNullSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ulLength1 as _) as PSID;
    if pNullSid.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlInitializeSid(pNullSid, &mut NullAuthority, 1);
    pSubAuthority = RtlSubAuthoritySid(pNullSid, 0);
    *pSubAuthority = SECURITY_NULL_RID;

    /* World SID */
    pWorldSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ulLength1 as _) as PSID;
    if pWorldSid.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlInitializeSid(pWorldSid, &mut WorldAuthority, 1);
    pSubAuthority = RtlSubAuthoritySid(pWorldSid, 0);
    *pSubAuthority = SECURITY_WORLD_RID;

    /* LocalSystem SID */
    pLocalSystemSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ulLength1 as _) as PSID;
    if pLocalSystemSid.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlInitializeSid(pLocalSystemSid, &mut NtAuthority, 1);
    pSubAuthority = RtlSubAuthoritySid(pLocalSystemSid, 0);
    *pSubAuthority = SECURITY_LOCAL_SYSTEM_RID;

    /* AuthenticatedUser SID */
    pAuthenticatedUserSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ulLength1 as _) as PSID;
    if pAuthenticatedUserSid.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlInitializeSid(pAuthenticatedUserSid, &mut NtAuthority, 1);
    pSubAuthority = RtlSubAuthoritySid(pAuthenticatedUserSid, 0);
    *pSubAuthority = SECURITY_AUTHENTICATED_USER_RID;

    /* AliasAdmins SID */
    pAliasAdminsSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ulLength2 as _) as PSID;
    if pAliasAdminsSid.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlInitializeSid(pAliasAdminsSid, &mut NtAuthority, 2);
    pSubAuthority = RtlSubAuthoritySid(pAliasAdminsSid, 0);
    *pSubAuthority = SECURITY_BUILTIN_DOMAIN_RID;
    pSubAuthority = RtlSubAuthoritySid(pAliasAdminsSid, 1);
    *pSubAuthority = DOMAIN_ALIAS_RID_ADMINS;

    ERROR_SUCCESS
}

unsafe fn ScmCreateAcls() -> DWORD {
    let acl = core::mem::size_of::<ACL>() as ULONG;
    let ace = core::mem::size_of::<ACE>() as ULONG;

    /* DACL */
    let mut ulLength = acl
        + (ace + RtlLengthSid(pLocalSystemSid))
        + (ace + RtlLengthSid(pAliasAdminsSid))
        + (ace + RtlLengthSid(pAuthenticatedUserSid));

    pDefaultDacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, ulLength as _) as PACL;
    if pDefaultDacl.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlCreateAcl(pDefaultDacl, ulLength, ACL_REVISION as _);

    RtlAddAccessAllowedAce(
        pDefaultDacl,
        ACL_REVISION as _,
        READ_CONTROL
            | SERVICE_ENUMERATE_DEPENDENTS
            | SERVICE_INTERROGATE
            | SERVICE_PAUSE_CONTINUE
            | SERVICE_QUERY_CONFIG
            | SERVICE_QUERY_STATUS
            | SERVICE_START
            | SERVICE_STOP
            | SERVICE_USER_DEFINED_CONTROL,
        pLocalSystemSid,
    );
    RtlAddAccessAllowedAce(pDefaultDacl, ACL_REVISION as _, SERVICE_ALL_ACCESS, pAliasAdminsSid);
    RtlAddAccessAllowedAce(
        pDefaultDacl,
        ACL_REVISION as _,
        READ_CONTROL
            | SERVICE_ENUMERATE_DEPENDENTS
            | SERVICE_INTERROGATE
            | SERVICE_QUERY_CONFIG
            | SERVICE_QUERY_STATUS
            | SERVICE_USER_DEFINED_CONTROL,
        pAuthenticatedUserSid,
    );

    /* SACL */
    ulLength = acl + (ace + RtlLengthSid(pNullSid));
    pDefaultSacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, ulLength as _) as PACL;
    if pDefaultSacl.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlCreateAcl(pDefaultSacl, ulLength, ACL_REVISION as _);
    RtlAddAuditAccessAce(pDefaultSacl, ACL_REVISION as _, SERVICE_ALL_ACCESS, pNullSid, 0, 1);

    /* Pipe DACL */
    ulLength = acl + (ace + RtlLengthSid(pWorldSid));
    pPipeDacl = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, ulLength as _) as PACL;
    if pPipeDacl.is_null() {
        return ERROR_OUTOFMEMORY;
    }
    RtlCreateAcl(pPipeDacl, ulLength, ACL_REVISION as _);
    RtlAddAccessAllowedAce(pPipeDacl, ACL_REVISION as _, GENERIC_ALL, pWorldSid);

    ERROR_SUCCESS
}

unsafe fn ScmFreeAcls() {
    if !pDefaultDacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pDefaultDacl as PVOID);
    }
    if !pDefaultSacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pDefaultSacl as PVOID);
    }
    if !pPipeDacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pPipeDacl as PVOID);
    }
}

unsafe fn ScmCreateDefaultSD() -> DWORD {
    pDefaultSD = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SECURITY_DESCRIPTOR>() as _,
    );
    if pDefaultSD.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    let mut Status = RtlCreateSecurityDescriptor(pDefaultSD, SECURITY_DESCRIPTOR_REVISION as _);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetOwnerSecurityDescriptor(pDefaultSD, pLocalSystemSid, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetGroupSecurityDescriptor(pDefaultSD, pLocalSystemSid, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetDaclSecurityDescriptor(pDefaultSD, 1, pDefaultDacl, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetSaclSecurityDescriptor(pDefaultSD, 1, pDefaultSacl, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }

    ERROR_SUCCESS
}

unsafe fn ScmFreeDefaultSD() {
    if !pDefaultSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pDefaultSD);
    }
}

unsafe fn ScmCreatePipeSD() -> DWORD {
    pPipeSD = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SECURITY_DESCRIPTOR>() as _,
    );
    if pPipeSD.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    let mut Status = RtlCreateSecurityDescriptor(pPipeSD, SECURITY_DESCRIPTOR_REVISION as _);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetOwnerSecurityDescriptor(pPipeSD, pLocalSystemSid, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetGroupSecurityDescriptor(pPipeSD, pLocalSystemSid, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }
    Status = RtlSetDaclSecurityDescriptor(pPipeSD, 1, pPipeDacl, 0);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }

    ERROR_SUCCESS
}

unsafe fn ScmFreePipeSD() {
    if !pPipeSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pPipeSD);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ScmCreateDefaultServiceSD(
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut pRelativeSD: PSECURITY_DESCRIPTOR = null_mut();
    let mut dwBufferLength: DWORD = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;

    loop {
        /* Sizing pass: expect STATUS_BUFFER_TOO_SMALL. */
        let mut Status = RtlAbsoluteToSelfRelativeSD(pDefaultSD, null_mut(), &mut dwBufferLength);
        if Status != STATUS_BUFFER_TOO_SMALL {
            dwError = RtlNtStatusToDosError(Status);
            break;
        }

        pRelativeSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, dwBufferLength as _);
        if pRelativeSD.is_null() {
            dwError = ERROR_OUTOFMEMORY;
            break;
        }

        Status = RtlAbsoluteToSelfRelativeSD(pDefaultSD, pRelativeSD, &mut dwBufferLength);
        if !NT_SUCCESS(Status) {
            dwError = RtlNtStatusToDosError(Status);
            break;
        }

        core::ptr::write_volatile(ppSecurityDescriptor, pRelativeSD);
        break;
    }

    // done:
    if dwError != ERROR_SUCCESS && !pRelativeSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pRelativeSD);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmInitializeSecurity() -> DWORD {
    let mut dwError: DWORD;

    dwError = ScmCreateSids();
    if dwError != ERROR_SUCCESS {
        return dwError;
    }
    dwError = ScmCreateAcls();
    if dwError != ERROR_SUCCESS {
        return dwError;
    }
    dwError = ScmCreateDefaultSD();
    if dwError != ERROR_SUCCESS {
        return dwError;
    }
    dwError = ScmCreatePipeSD();
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ScmShutdownSecurity() {
    ScmFreePipeSD();
    ScmFreeDefaultSD();
    ScmFreeAcls();
    ScmFreeSids();
}
