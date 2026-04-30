use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::inline::se::SeLengthSid;
use ntkrnl_sys::ddk::{
    ACE, ACL, CmpAcquireKcbLockExclusive, CmpLockRegistry, CmpReleaseKcbLock, CmpUnlockRegistry,
    PACE_HEADER, PACL, PCM_KEY_BODY, PCM_KEY_CONTROL_BLOCK, PGENERIC_MAPPING,
    PISECURITY_DESCRIPTOR, PISECURITY_DESCRIPTOR_RELATIVE, PSECURITY_DESCRIPTOR,
    PSECURITY_INFORMATION, PSID, RtlAddAccessAllowedAce, RtlCreateAcl, RtlCreateSecurityDescriptor,
    RtlCreateSecurityDescriptorRelative, RtlGetAce, RtlInitializeSid, RtlLengthRequiredSid,
    RtlLengthSid, RtlSetDaclSecurityDescriptor, RtlSubAuthoritySid, _SECURITY_DESCRIPTOR,
    _SECURITY_DESCRIPTOR_RELATIVE, SECURITY_DESCRIPTOR_CONTROL, SECURITY_INFORMATION,
    SECURITY_OPERATION_CODE, SID_IDENTIFIER_AUTHORITY,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
    _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
};

use ntkrnl_sys::opaque_ptr;
// SECURITY_SYSTEM bugcheck code (bugcodes.mc: 0x29)
const SECURITY_SYSTEM: u32 = 0x29;

extern "C" {
    fn CmpAcquireKcbLockShared(Kcb: PCM_KEY_CONTROL_BLOCK);
}

#[no_mangle]
pub unsafe extern "system" fn CmpHiveRootSecurityDescriptor() -> PSECURITY_DESCRIPTOR {
    let mut Status: NTSTATUS;
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut Acl: PACL;
    let AclCopy: PACL;
    let mut Sid: [PSID; 4] = [core::ptr::null_mut(); 4];
    let WorldAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };
    let NtAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut AceLength: ULONG;
    let mut AclLength: ULONG;
    let mut SidLength: ULONG;
    let mut AceHeader: PACE_HEADER = core::ptr::null_mut();
    let mut i: ULONG;

    SidLength = RtlLengthRequiredSid(1);
    Sid[0] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    Sid[1] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    Sid[2] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);
    SidLength = RtlLengthRequiredSid(2);
    Sid[3] = ExAllocatePoolWithTag(PagedPool, SidLength as usize, 0x44534D43u32);

    if Sid[0].is_null() || Sid[1].is_null() || Sid[2].is_null() || Sid[3].is_null() {
        KeBugCheckEx(0x51, 11, 1, 0, 0);
    }

    Status = RtlInitializeSid(Sid[0], &WorldAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[1], &NtAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[2], &NtAuthority as *const _ as *mut _, 1);
    Status |= RtlInitializeSid(Sid[3], &NtAuthority as *const _ as *mut _, 2);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(0x51, 11, 2, 0, 0);
    }

    *RtlSubAuthoritySid(Sid[0], 0) = 0;
    *RtlSubAuthoritySid(Sid[1], 0) = 33;
    *RtlSubAuthoritySid(Sid[2], 0) = 18;
    *RtlSubAuthoritySid(Sid[3], 0) = 32;
    *RtlSubAuthoritySid(Sid[3], 1) = 544;

    AclLength = core::mem::size_of::<ACL>() as ULONG;
    i = 0;
    while i < 4 {
        AceLength = 8;
        AceLength += SeLengthSid(Sid[i as usize]);
        AclLength += AceLength;
        i += 1;
    }

    Acl = ExAllocatePoolWithTag(PagedPool, AclLength as usize, 0x44534D43u32) as PACL;
    if Acl.is_null() {
        KeBugCheckEx(0x51, 11, 3, 0, 0);
    }

    Status = RtlCreateAcl(Acl, AclLength, 2);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(0x51, 11, 4, Status as ULONG_PTR, 0);
    }

    Status = RtlAddAccessAllowedAce(Acl, 2, 0xF003F, Sid[2]);
    Status |= RtlAddAccessAllowedAce(Acl, 2, 0xF003F, Sid[3]);
    Status |= RtlAddAccessAllowedAce(Acl, 2, 0x20019, Sid[0]);
    Status |= RtlAddAccessAllowedAce(Acl, 2, 0x20019, Sid[1]);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(0x51, 11, 5, Status as ULONG_PTR, 0);
    }

    Status = RtlGetAce(Acl, 0, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= 2;
    Status = RtlGetAce(Acl, 1, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= 2;
    Status = RtlGetAce(Acl, 2, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= 2;
    Status = RtlGetAce(Acl, 3, &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void);
    (*AceHeader).AceFlags |= 2;

    SecurityDescriptor = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SECURITY_DESCRIPTOR>() + AclLength as usize,
        0x44534D43u32,
    );
    if SecurityDescriptor.is_null() {
        KeBugCheckEx(0x51, 11, 6, 0, 0);
    }

    AclCopy = (SecurityDescriptor as PISECURITY_DESCRIPTOR).offset(1) as PACL;
    core::ptr::copy_nonoverlapping(Acl as *const u8, AclCopy as *mut u8, AclLength as usize);

    Status = RtlCreateSecurityDescriptor(SecurityDescriptor, 1);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(0x51, 11, 7, Status as ULONG_PTR, 0);
    }

    Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, 1u8, AclCopy, 0u8);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(0x51, 11, 8, Status as ULONG_PTR, 0);
    }

    i = 0;
    while i < 4 {
        ExFreePoolWithTag(Sid[i as usize] as *mut core::ffi::c_void, 0x44534D43u32);
        i += 1;
    }
    ExFreePoolWithTag(Acl as *mut core::ffi::c_void, 0x44534D43u32);

    SecurityDescriptor
}

#[no_mangle]
pub unsafe extern "system" fn CmpQuerySecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let BufferLength = opaque_ptr(BufferLength);
    let mut RelSd: PISECURITY_DESCRIPTOR_RELATIVE;
    let mut SidSize: u32;
    let mut AclSize: u32 = 0;
    let mut SdSize: u32;
    let mut Status: i32;
    let mut Control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut Owner: u32 = 0;
    let mut Group: u32 = 0;
    let mut Dacl: u32 = 0;

    if SecurityInformation == 0 {
        return STATUS_ACCESS_DENIED;
    }

    SidSize = RtlLengthSid(SeWorldSid);
    RelSd = SecurityDescriptor as PISECURITY_DESCRIPTOR_RELATIVE;
    SdSize = core::mem::size_of::<_SECURITY_DESCRIPTOR_RELATIVE>() as u32;

    if (SecurityInformation & OWNER_SECURITY_INFORMATION) != 0 {
        Owner = SdSize;
        SdSize += SidSize;
    }

    if (SecurityInformation & GROUP_SECURITY_INFORMATION) != 0 {
        Group = SdSize;
        SdSize += SidSize;
    }

    if (SecurityInformation & DACL_SECURITY_INFORMATION) != 0 {
        Control |= SE_DACL_PRESENT;
        Dacl = SdSize;
        AclSize = core::mem::size_of::<ACL>() as u32 + core::mem::size_of::<ACE>() as u32 + SidSize;
        SdSize += AclSize;
    }

    if (SecurityInformation & SACL_SECURITY_INFORMATION) != 0 {
        Control |= SE_SACL_PRESENT;
    }

    if *BufferLength < SdSize {
        *BufferLength = SdSize;
        return STATUS_BUFFER_TOO_SMALL;
    }

    *BufferLength = SdSize;

    Status = RtlCreateSecurityDescriptorRelative(
        RelSd,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*RelSd).Control |= Control;
    (*RelSd).Owner = Owner;
    (*RelSd).Group = Group;
    (*RelSd).Dacl = Dacl;

    if Owner != 0 {
        RtlCopyMemory(
            (RelSd as *mut u8).add(Owner as usize) as *mut core::ffi::c_void,
            SeWorldSid as *const core::ffi::c_void,
            SidSize as usize,
        );
    }

    if Group != 0 {
        RtlCopyMemory(
            (RelSd as *mut u8).add(Group as usize) as *mut core::ffi::c_void,
            SeWorldSid as *const core::ffi::c_void,
            SidSize as usize,
        );
    }

    if Dacl != 0 {
        Status = RtlCreateAcl(
            (RelSd as *mut u8).add(Dacl as usize) as PACL,
            AclSize,
            ACL_REVISION,
        );
        if NT_SUCCESS(Status) {
            Status = RtlAddAccessAllowedAce(
                (RelSd as *mut u8).add(Dacl as usize) as PACL,
                ACL_REVISION,
                GENERIC_ALL,
                SeWorldSid,
            );
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpSetSecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let SecurityInformation = opaque_ptr(SecurityInformation);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpAssignSecurityDescriptor(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let Kcb = opaque_ptr(Kcb);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpSecurityMethod(
    ObjectBody: *mut core::ffi::c_void,
    OperationCode: SECURITY_OPERATION_CODE,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    BufferLength: *mut u32,
    OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    let ObjectBody = opaque_ptr(ObjectBody);
    let SecurityInformation = opaque_ptr(SecurityInformation);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let BufferLength = opaque_ptr(BufferLength);
    let OldSecurityDescriptor = opaque_ptr(OldSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    let Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut Status: i32 = STATUS_SUCCESS;

    Kcb = (*(ObjectBody as PCM_KEY_BODY)).KeyControlBlock;

    CmpLockRegistry();

    if OperationCode == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        if (*(ObjectBody as PCM_KEY_BODY)).KcbLocked == 0 {
            CmpAcquireKcbLockShared(Kcb);
        }
    } else {
        CmpAcquireKcbLockExclusive(Kcb);
    }

    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        CmpReleaseKcbLock(Kcb);
        CmpUnlockRegistry();
        return STATUS_KEY_DELETED;
    }

    match OperationCode {
        _SECURITY_OPERATION_CODE_SetSecurityDescriptor => {
            Status = CmpSetSecurityDescriptor(
                Kcb,
                SecurityInformation,
                SecurityDescriptor,
                PoolType,
                GenericMapping,
            );
        }
        _SECURITY_OPERATION_CODE_QuerySecurityDescriptor => {
            Status = CmpQuerySecurityDescriptor(
                Kcb,
                *SecurityInformation,
                SecurityDescriptor,
                BufferLength,
            );
        }
        _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor => {
        }
        _SECURITY_OPERATION_CODE_AssignSecurityDescriptor => {
            Status = CmpAssignSecurityDescriptor(Kcb, SecurityDescriptor);
        }
        _ => {
            KeBugCheckEx(SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER as ULONG_PTR, 0, 0);
        }
    }

    if (*(ObjectBody as PCM_KEY_BODY)).KcbLocked == 0 {
        CmpReleaseKcbLock(Kcb);
    }

    CmpUnlockRegistry();

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmse.c
//  * PURPOSE:         Configuration Manager - Security Subsystem Interface
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// PSECURITY_DESCRIPTOR
// NTAPI
// CmpHiveRootSecurityDescriptor(VOID)
// {
//     NTSTATUS Status;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PACL Acl, AclCopy;
//     PSID Sid[4];
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     ULONG AceLength, AclLength, SidLength;
//     PACE_HEADER AceHeader;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Phase 1: Allocate SIDs */
//     SidLength = RtlLengthRequiredSid(1);
//     Sid[0] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     Sid[1] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     Sid[2] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//     SidLength = RtlLengthRequiredSid(2);
//     Sid[3] = ExAllocatePoolWithTag(PagedPool, SidLength, TAG_CMSD);
//
//     /* Make sure all SIDs were allocated */
//     if (!Sid[0] || !Sid[1] || !Sid[2] || !Sid[3])
//     {
//         /* Bugcheck */
//         KeBugCheckEx(REGISTRY_ERROR, 11, 1, 0, 0);
//     }
//
//     /* Phase 2: Initialize all SIDs */
//     Status = RtlInitializeSid(Sid[0], &WorldAuthority, 1);
//     Status |= RtlInitializeSid(Sid[1], &NtAuthority, 1);
//     Status |= RtlInitializeSid(Sid[2], &NtAuthority, 1);
//     Status |= RtlInitializeSid(Sid[3], &NtAuthority, 2);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 2, 0, 0);
//
//     /* Phase 2: Setup SID Sub Authorities */
//     *RtlSubAuthoritySid(Sid[0], 0) = SECURITY_WORLD_RID;
//     *RtlSubAuthoritySid(Sid[1], 0) = SECURITY_RESTRICTED_CODE_RID;
//     *RtlSubAuthoritySid(Sid[2], 0) = SECURITY_LOCAL_SYSTEM_RID;
//     *RtlSubAuthoritySid(Sid[3], 0) = SECURITY_BUILTIN_DOMAIN_RID;
//     *RtlSubAuthoritySid(Sid[3], 1) = DOMAIN_ALIAS_RID_ADMINS;
//
//     /* Make sure all SIDs are valid */
//     ASSERT(RtlValidSid(Sid[0]));
//     ASSERT(RtlValidSid(Sid[1]));
//     ASSERT(RtlValidSid(Sid[2]));
//     ASSERT(RtlValidSid(Sid[3]));
//
//     /* Phase 3: Calculate ACL Length */
//     AclLength = sizeof(ACL);
//     for (i = 0; i < 4; i++)
//     {
//         /* This is what MSDN says to do */
//         AceLength = FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart);
//         AceLength += SeLengthSid(Sid[i]);
//         AclLength += AceLength;
//     }
//
//     /* Phase 3: Allocate the ACL */
//     Acl = ExAllocatePoolWithTag(PagedPool, AclLength, TAG_CMSD);
//     if (!Acl) KeBugCheckEx(REGISTRY_ERROR, 11, 3, 0, 0);
//
//     /* Phase 4: Create the ACL */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 4, Status, 0);
//
//     /* Phase 5: Build the ACL */
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[2]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_ALL_ACCESS, Sid[3]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[0]);
//     Status |= RtlAddAccessAllowedAce(Acl, ACL_REVISION, KEY_READ, Sid[1]);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 5, Status, 0);
//
//     /* Phase 5: Make the ACEs inheritable */
//     Status = RtlGetAce(Acl, 0, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 1, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 2, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//     Status = RtlGetAce(Acl, 3, (PVOID*)&AceHeader);
//     ASSERT(NT_SUCCESS(Status));
//     AceHeader->AceFlags |= CONTAINER_INHERIT_ACE;
//
//     /* Phase 6: Allocate the security descriptor and make space for the ACL */
//     SecurityDescriptor = ExAllocatePoolWithTag(PagedPool,
//                                                sizeof(SECURITY_DESCRIPTOR) +
//                                                AclLength,
//                                                TAG_CMSD);
//     if (!SecurityDescriptor) KeBugCheckEx(REGISTRY_ERROR, 11, 6, 0, 0);
//
//     /* Phase 6: Make a copy of the ACL */
//     AclCopy = (PACL)((PISECURITY_DESCRIPTOR)SecurityDescriptor + 1);
//     RtlCopyMemory(AclCopy, Acl, AclLength);
//
//     /* Phase 7: Create the security descriptor */
//     Status = RtlCreateSecurityDescriptor(SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 7, Status, 0);
//
//     /* Phase 8: Set the ACL as a DACL */
//     Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor,
//                                           TRUE,
//                                           AclCopy,
//                                           FALSE);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(REGISTRY_ERROR, 11, 8, Status, 0);
//
//     /* Free the SIDs and original ACL */
//     for (i = 0; i < 4; i++) ExFreePoolWithTag(Sid[i], TAG_CMSD);
//     ExFreePoolWithTag(Acl, TAG_CMSD);
//
//     /* Return the security descriptor */
//     return SecurityDescriptor;
// }
//
// NTSTATUS
// CmpQuerySecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                            IN SECURITY_INFORMATION SecurityInformation,
//                            OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                            IN OUT PULONG BufferLength)
// {
//     PISECURITY_DESCRIPTOR_RELATIVE RelSd;
//     ULONG SidSize;
//     ULONG AclSize;
//     ULONG SdSize;
//     NTSTATUS Status;
//     SECURITY_DESCRIPTOR_CONTROL Control = 0;
//     ULONG Owner = 0;
//     ULONG Group = 0;
//     ULONG Dacl = 0;
//
//     DBG_UNREFERENCED_PARAMETER(Kcb);
//
//     DPRINT("CmpQuerySecurityDescriptor()\n");
//
//     if (SecurityInformation == 0)
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     SidSize = RtlLengthSid(SeWorldSid);
//     RelSd = SecurityDescriptor;
//     SdSize = sizeof(*RelSd);
//
//     if (SecurityInformation & OWNER_SECURITY_INFORMATION)
//     {
//         Owner = SdSize;
//         SdSize += SidSize;
//     }
//
//     if (SecurityInformation & GROUP_SECURITY_INFORMATION)
//     {
//         Group = SdSize;
//         SdSize += SidSize;
//     }
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//     {
//         Control |= SE_DACL_PRESENT;
//         Dacl = SdSize;
//         AclSize = sizeof(ACL) + sizeof(ACE) + SidSize;
//         SdSize += AclSize;
//     }
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//     {
//         Control |= SE_SACL_PRESENT;
//     }
//
//     if (*BufferLength < SdSize)
//     {
//         *BufferLength = SdSize;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     *BufferLength = SdSize;
//
//     Status = RtlCreateSecurityDescriptorRelative(RelSd,
//                                                  SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RelSd->Control |= Control;
//     RelSd->Owner = Owner;
//     RelSd->Group = Group;
//     RelSd->Dacl = Dacl;
//
//     if (Owner)
//         RtlCopyMemory((PUCHAR)RelSd + Owner,
//                       SeWorldSid,
//                       SidSize);
//
//     if (Group)
//         RtlCopyMemory((PUCHAR)RelSd + Group,
//                       SeWorldSid,
//                       SidSize);
//
//     if (Dacl)
//     {
//         Status = RtlCreateAcl((PACL)((PUCHAR)RelSd + Dacl),
//                               AclSize,
//                               ACL_REVISION);
//         if (NT_SUCCESS(Status))
//         {
//             Status = RtlAddAccessAllowedAce((PACL)((PUCHAR)RelSd + Dacl),
//                                             ACL_REVISION,
//                                             GENERIC_ALL,
//                                             SeWorldSid);
//         }
//     }
//
//     ASSERT(Status == STATUS_SUCCESS);
//     return Status;
// }
//
// NTSTATUS
// CmpSetSecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                          IN PSECURITY_INFORMATION SecurityInformation,
//                          IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                          IN POOL_TYPE PoolType,
//                          IN PGENERIC_MAPPING GenericMapping)
// {
//     DPRINT("CmpSetSecurityDescriptor()\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CmpAssignSecurityDescriptor(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                             IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     DPRINT("CmpAssignSecurityDescriptor(%p %p)\n",
//            Kcb, SecurityDescriptor);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CmpSecurityMethod(IN PVOID ObjectBody,
//                   IN SECURITY_OPERATION_CODE OperationCode,
//                   IN PSECURITY_INFORMATION SecurityInformation,
//                   IN OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                   IN OUT PULONG BufferLength,
//                   IN OUT PSECURITY_DESCRIPTOR *OldSecurityDescriptor,
//                   IN POOL_TYPE PoolType,
//                   IN PGENERIC_MAPPING GenericMapping)
// {
//     PCM_KEY_CONTROL_BLOCK Kcb;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DBG_UNREFERENCED_PARAMETER(OldSecurityDescriptor);
//     DBG_UNREFERENCED_PARAMETER(GenericMapping);
//
//     Kcb = ((PCM_KEY_BODY)ObjectBody)->KeyControlBlock;
//
//     /* Acquire the hive lock */
//     CmpLockRegistry();
//
//     /* Acquire the KCB lock */
//     if (OperationCode == QuerySecurityDescriptor)
//     {
//         /* Avoid recursive locking if somebody already holds it */
//         if (!((PCM_KEY_BODY)ObjectBody)->KcbLocked)
//         {
//             CmpAcquireKcbLockShared(Kcb);
//         }
//     }
//     else
//     {
//         ASSERT(!((PCM_KEY_BODY)ObjectBody)->KcbLocked);
//         CmpAcquireKcbLockExclusive(Kcb);
//     }
//
//     /* Don't touch deleted keys */
//     if (Kcb->Delete)
//     {
//         /* Release the KCB lock */
//         CmpReleaseKcbLock(Kcb);
//
//         /* Release the hive lock */
//         CmpUnlockRegistry();
//         return STATUS_KEY_DELETED;
//     }
//
//     switch (OperationCode)
//     {
//         case SetSecurityDescriptor:
//             DPRINT("Set security descriptor\n");
//             ASSERT((PoolType == PagedPool) || (PoolType == NonPagedPool));
//             Status = CmpSetSecurityDescriptor(Kcb,
//                                               SecurityInformation,
//                                               SecurityDescriptor,
//                                               PoolType,
//                                               GenericMapping);
//             break;
//
//         case QuerySecurityDescriptor:
//             DPRINT("Query security descriptor\n");
//             Status = CmpQuerySecurityDescriptor(Kcb,
//                                                 *SecurityInformation,
//                                                 SecurityDescriptor,
//                                                 BufferLength);
//             break;
//
//         case DeleteSecurityDescriptor:
//             DPRINT("Delete security descriptor\n");
//             /* HACK */
//             break;
//
//         case AssignSecurityDescriptor:
//             DPRINT("Assign security descriptor\n");
//             Status = CmpAssignSecurityDescriptor(Kcb,
//                                                  SecurityDescriptor);
//             break;
//
//         default:
//             KeBugCheckEx(SECURITY_SYSTEM, 0, STATUS_INVALID_PARAMETER, 0, 0);
//     }
//
//     /*
//      * Release the KCB lock, but only if we locked it ourselves and
//      * nobody else was locking it by themselves.
//      */
//     if (!((PCM_KEY_BODY)ObjectBody)->KcbLocked)
//     {
//         CmpReleaseKcbLock(Kcb);
//     }
//
//     /* Release the hive lock */
//     CmpUnlockRegistry();
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
