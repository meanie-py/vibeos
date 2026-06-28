use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
}

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorControl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pControl: PSECURITY_DESCRIPTOR_CONTROL,
    lpdwRevision: LPDWORD,
) -> i32 {

    let Status = RtlGetControlSecurityDescriptor(
        pSecurityDescriptor,
        pControl,
        lpdwRevision as *mut u32,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorDacl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpbDaclPresent: LPBOOL,
    pDacl: *mut PACL,
    lpbDaclDefaulted: LPBOOL,
) -> BOOL {
    let mut DaclPresent: BOOLEAN = 0;
    let mut DaclDefaulted: BOOLEAN = 0;

    let Status = RtlGetDaclSecurityDescriptor(
        pSecurityDescriptor,
        &mut DaclPresent,
        pDacl,
        &mut DaclDefaulted,
    );
    *lpbDaclPresent = DaclPresent as BOOL;
    *lpbDaclDefaulted = DaclDefaulted as BOOL;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorGroup(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pGroup: *mut PSID,
    lpbGroupDefaulted: LPBOOL,
) -> i32 {
    let mut GroupDefaulted: u8 = 0;
    let Status;

    Status = RtlGetGroupSecurityDescriptor(
        pSecurityDescriptor,
        pGroup,
        &mut GroupDefaulted,
    );
    *lpbGroupDefaulted = GroupDefaulted as i32;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorOwner(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pOwner: *mut PSID,
    lpbOwnerDefaulted: LPBOOL,
) -> BOOL {
    let mut OwnerDefaulted: BOOLEAN = 0;
    let Status: NTSTATUS;

    Status = RtlGetOwnerSecurityDescriptor(
        pSecurityDescriptor,
        pOwner,
        &mut OwnerDefaulted,
    );
    *lpbOwnerDefaulted = OwnerDefaulted as BOOL;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorRMControl(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    RMControl: PUCHAR,
) -> DWORD {
    if RtlGetSecurityDescriptorRMControl(SecurityDescriptor, RMControl) == 0 {
        return ERROR_INVALID_DATA;
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityDescriptorSacl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpbSaclPresent: LPBOOL,
    pSacl: *mut PACL,
    lpbSaclDefaulted: LPBOOL,
) -> i32 {
    let mut SaclPresent: u8 = 0;
    let mut SaclDefaulted: u8 = 0;

    let Status = RtlGetSaclSecurityDescriptor(
        pSecurityDescriptor,
        &mut SaclPresent,
        pSacl,
        &mut SaclDefaulted,
    );
    *lpbSaclPresent = SaclPresent as i32;
    *lpbSaclDefaulted = SaclDefaulted as i32;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn IsValidSecurityDescriptor(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOL {

    let Result: BOOLEAN = RtlValidSecurityDescriptor(pSecurityDescriptor);
    if Result == 0u8 {
        SetLastError(RtlNtStatusToDosError(STATUS_INVALID_SECURITY_DESCR));
    }

    Result as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn MakeAbsoluteSD2(
    pSelfRelativeSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpdwBufferSize: LPDWORD,
) -> i32 {

    let Status = RtlSelfRelativeToAbsoluteSD2(pSelfRelativeSecurityDescriptor, lpdwBufferSize);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn MakeSelfRelativeSD(
    pAbsoluteSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pSelfRelativeSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpdwBufferLength: LPDWORD,
) -> BOOL {

    let Status = RtlAbsoluteToSelfRelativeSD(
        pAbsoluteSecurityDescriptor,
        pSelfRelativeSecurityDescriptor,
        lpdwBufferLength as PULONG,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorControl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    ControlBitsOfInterest: SECURITY_DESCRIPTOR_CONTROL,
    ControlBitsToSet: SECURITY_DESCRIPTOR_CONTROL,
) -> BOOL {
    let Status = RtlSetControlSecurityDescriptor(
        pSecurityDescriptor,
        ControlBitsOfInterest,
        ControlBitsToSet,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorDacl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    bDaclPresent: BOOL,
    pDacl: PACL,
    bDaclDefaulted: BOOL,
) -> BOOL {

    let Status = RtlSetDaclSecurityDescriptor(
        pSecurityDescriptor,
        bDaclPresent as u8,
        pDacl,
        bDaclDefaulted as u8,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorGroup(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pGroup: PSID,
    bGroupDefaulted: BOOL,
) -> BOOL {

    let Status = RtlSetGroupSecurityDescriptor(pSecurityDescriptor, pGroup, bGroupDefaulted as u8);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorOwner(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pOwner: PSID,
    bOwnerDefaulted: i32,
) -> i32 {

    let Status = RtlSetOwnerSecurityDescriptor(pSecurityDescriptor, pOwner, bOwnerDefaulted as u8);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorRMControl(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    RMControl: PUCHAR,
) -> DWORD {
    RtlSetSecurityDescriptorRMControl(SecurityDescriptor, RMControl);

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityDescriptorSacl(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    bSaclPresent: i32,
    pSacl: PACL,
    bSaclDefaulted: i32,
) -> i32 {

    let Status = RtlSetSaclSecurityDescriptor(
        pSecurityDescriptor,
        bSaclPresent as u8,
        pSacl,
        bSaclDefaulted as u8,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn QuerySecurityAccessMask(
    SecurityInformation: SECURITY_INFORMATION,
    DesiredAccess: LPDWORD,
) {
    *DesiredAccess = 0;

    if SecurityInformation
        & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION)
        != 0
    {
        *DesiredAccess |= READ_CONTROL;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }
}
#[no_mangle]
pub unsafe extern "system" fn SetSecurityAccessMask(SecurityInformation: SECURITY_INFORMATION, DesiredAccess: LPDWORD) {
    *DesiredAccess = 0;

    if SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION) != 0 {
        *DesiredAccess |= WRITE_OWNER;
    }

    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        *DesiredAccess |= WRITE_DAC;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConvertToAutoInheritPrivateObjectSecurity(
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    CurrentSecurityDescriptor: PSECURITY_DESCRIPTOR,
    NewSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    ObjectType: *mut GUID,
    IsDirectoryObject: u8,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn BuildSecurityDescriptorW(
    pOwner: PTRUSTEE_W,
    pGroup: PTRUSTEE_W,
    cCountOfAccessEntries: ULONG,
    pListOfAccessEntries: PEXPLICIT_ACCESS_W,
    cCountOfAuditEntries: ULONG,
    pListOfAuditEntries: PEXPLICIT_ACCESS_W,
    pOldSD: PSECURITY_DESCRIPTOR,
    pSizeNewSD: PULONG,
    pNewSD: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn BuildSecurityDescriptorA(
    pOwner: PTRUSTEE_A,
    pGroup: PTRUSTEE_A,
    cCountOfAccessEntries: ULONG,
    pListOfAccessEntries: PEXPLICIT_ACCESS_A,
    cCountOfAuditEntries: ULONG,
    pListOfAuditEntries: PEXPLICIT_ACCESS_A,
    pOldSD: PSECURITY_DESCRIPTOR,
    pSizeNewSD: PULONG,
    pNewSD: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/sec/sec.c
//  * PURPOSE:         Security descriptor functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Steven Edwards ( Steven_Ed4153@yahoo.com )
//  *                  Andrew Greenwood ( silverblade_uk@hotmail.com )
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSecurityDescriptorControl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                              PSECURITY_DESCRIPTOR_CONTROL pControl,
//                              LPDWORD lpdwRevision)
// {
//     NTSTATUS Status;
//
//     Status = RtlGetControlSecurityDescriptor(pSecurityDescriptor,
//                                              pControl,
//                                              (PULONG)lpdwRevision);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSecurityDescriptorDacl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                           LPBOOL lpbDaclPresent,
//                           PACL *pDacl,
//                           LPBOOL lpbDaclDefaulted)
// {
//     BOOLEAN DaclPresent;
//     BOOLEAN DaclDefaulted;
//     NTSTATUS Status;
//
//     Status = RtlGetDaclSecurityDescriptor(pSecurityDescriptor,
//                                           &DaclPresent,
//                                           pDacl,
//                                           &DaclDefaulted);
//     *lpbDaclPresent = (BOOL)DaclPresent;
//     *lpbDaclDefaulted = (BOOL)DaclDefaulted;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSecurityDescriptorGroup(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                            PSID *pGroup,
//                            LPBOOL lpbGroupDefaulted)
// {
//     BOOLEAN GroupDefaulted;
//     NTSTATUS Status;
//
//     Status = RtlGetGroupSecurityDescriptor(pSecurityDescriptor,
//                                            pGroup,
//                                            &GroupDefaulted);
//     *lpbGroupDefaulted = (BOOL)GroupDefaulted;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSecurityDescriptorOwner(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                            PSID *pOwner,
//                            LPBOOL lpbOwnerDefaulted)
// {
//     BOOLEAN OwnerDefaulted;
//     NTSTATUS Status;
//
//     Status = RtlGetOwnerSecurityDescriptor(pSecurityDescriptor,
//                                            pOwner,
//                                            &OwnerDefaulted);
//     *lpbOwnerDefaulted = (BOOL)OwnerDefaulted;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetSecurityDescriptorRMControl(PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                PUCHAR RMControl)
// {
//     if (!RtlGetSecurityDescriptorRMControl(SecurityDescriptor,
//                                            RMControl))
//         return ERROR_INVALID_DATA;
//
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSecurityDescriptorSacl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                           LPBOOL lpbSaclPresent,
//                           PACL *pSacl,
//                           LPBOOL lpbSaclDefaulted)
// {
//     BOOLEAN SaclPresent;
//     BOOLEAN SaclDefaulted;
//     NTSTATUS Status;
//
//     Status = RtlGetSaclSecurityDescriptor(pSecurityDescriptor,
//                                           &SaclPresent,
//                                           pSacl,
//                                           &SaclDefaulted);
//     *lpbSaclPresent = (BOOL)SaclPresent;
//     *lpbSaclDefaulted = (BOOL)SaclDefaulted;
//
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
// IsValidSecurityDescriptor(PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     BOOLEAN Result;
//
//     Result = RtlValidSecurityDescriptor (pSecurityDescriptor);
//     if (Result == FALSE)
//         SetLastError(RtlNtStatusToDosError(STATUS_INVALID_SECURITY_DESCR));
//
//     return (BOOL)Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MakeAbsoluteSD2(IN OUT PSECURITY_DESCRIPTOR pSelfRelativeSecurityDescriptor,
//                 OUT LPDWORD lpdwBufferSize)
// {
//     NTSTATUS Status;
//
//     Status = RtlSelfRelativeToAbsoluteSD2(pSelfRelativeSecurityDescriptor,
//                                           lpdwBufferSize);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MakeSelfRelativeSD(PSECURITY_DESCRIPTOR pAbsoluteSecurityDescriptor,
//                    PSECURITY_DESCRIPTOR pSelfRelativeSecurityDescriptor,
//                    LPDWORD lpdwBufferLength)
// {
//     NTSTATUS Status;
//
//     Status = RtlAbsoluteToSelfRelativeSD(pAbsoluteSecurityDescriptor,
//                                          pSelfRelativeSecurityDescriptor,
//                                          (PULONG)lpdwBufferLength);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSecurityDescriptorControl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                              SECURITY_DESCRIPTOR_CONTROL ControlBitsOfInterest,
//                              SECURITY_DESCRIPTOR_CONTROL ControlBitsToSet)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetControlSecurityDescriptor(pSecurityDescriptor,
//                                              ControlBitsOfInterest,
//                                              ControlBitsToSet);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSecurityDescriptorDacl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                           BOOL bDaclPresent,
//                           PACL pDacl,
//                           BOOL bDaclDefaulted)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetDaclSecurityDescriptor(pSecurityDescriptor,
//                                           bDaclPresent,
//                                           pDacl,
//                                           bDaclDefaulted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSecurityDescriptorGroup(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                            PSID pGroup,
//                            BOOL bGroupDefaulted)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetGroupSecurityDescriptor(pSecurityDescriptor,
//                                            pGroup,
//                                            bGroupDefaulted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSecurityDescriptorOwner(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                            PSID pOwner,
//                            BOOL bOwnerDefaulted)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetOwnerSecurityDescriptor(pSecurityDescriptor,
//                                            pOwner,
//                                            bOwnerDefaulted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetSecurityDescriptorRMControl(PSECURITY_DESCRIPTOR SecurityDescriptor,
//                                PUCHAR RMControl)
// {
//     RtlSetSecurityDescriptorRMControl(SecurityDescriptor,
//                                       RMControl);
//
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSecurityDescriptorSacl(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                           BOOL bSaclPresent,
//                           PACL pSacl,
//                           BOOL bSaclDefaulted)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetSaclSecurityDescriptor(pSecurityDescriptor,
//                                           bSaclPresent,
//                                           pSacl,
//                                           bSaclDefaulted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// QuerySecurityAccessMask(IN SECURITY_INFORMATION SecurityInformation,
//                         OUT LPDWORD DesiredAccess)
// {
//     *DesiredAccess = 0;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION |
//                                GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION))
//     {
//         *DesiredAccess |= READ_CONTROL;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// SetSecurityAccessMask(IN SECURITY_INFORMATION SecurityInformation,
//                       OUT LPDWORD DesiredAccess)
// {
//     *DesiredAccess = 0;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION))
//         *DesiredAccess |= WRITE_OWNER;
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//         *DesiredAccess |= WRITE_DAC;
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// ConvertToAutoInheritPrivateObjectSecurity(IN PSECURITY_DESCRIPTOR ParentDescriptor,
//                                           IN PSECURITY_DESCRIPTOR CurrentSecurityDescriptor,
//                                           OUT PSECURITY_DESCRIPTOR* NewSecurityDescriptor,
//                                           IN GUID* ObjectType,
//                                           IN BOOLEAN IsDirectoryObject,
//                                           IN PGENERIC_MAPPING GenericMapping)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// BuildSecurityDescriptorW(IN PTRUSTEE_W pOwner  OPTIONAL,
//                          IN PTRUSTEE_W pGroup  OPTIONAL,
//                          IN ULONG cCountOfAccessEntries,
//                          IN PEXPLICIT_ACCESS_W pListOfAccessEntries  OPTIONAL,
//                          IN ULONG cCountOfAuditEntries,
//                          IN PEXPLICIT_ACCESS_W pListOfAuditEntries  OPTIONAL,
//                          IN PSECURITY_DESCRIPTOR pOldSD  OPTIONAL,
//                          OUT PULONG pSizeNewSD,
//                          OUT PSECURITY_DESCRIPTOR* pNewSD)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// BuildSecurityDescriptorA(IN PTRUSTEE_A pOwner  OPTIONAL,
//                          IN PTRUSTEE_A pGroup  OPTIONAL,
//                          IN ULONG cCountOfAccessEntries,
//                          IN PEXPLICIT_ACCESS_A pListOfAccessEntries  OPTIONAL,
//                          IN ULONG cCountOfAuditEntries,
//                          IN PEXPLICIT_ACCESS_A pListOfAuditEntries  OPTIONAL,
//                          IN PSECURITY_DESCRIPTOR pOldSD  OPTIONAL,
//                          OUT PULONG pSizeNewSD,
//                          OUT PSECURITY_DESCRIPTOR* pNewSD)
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
