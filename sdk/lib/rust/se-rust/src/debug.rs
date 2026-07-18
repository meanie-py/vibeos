use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, TRUE, FALSE};

// DbgPrint is a no-op stub - the real DbgPrint is a variadic C function.
// We declare it as an extern to avoid the unstable c_variadic feature.
// Since debug functions are #[cfg(debug_assertions)], they are dead code in release builds.
#[cfg(debug_assertions)]
unsafe fn SepGetAceTypeString(ace_type: u8) -> PCSTR {
    static ACE_TYPE_STRS: [&[u8]; 18] = [
        b"ACCESS_ALLOWED_ACE_TYPE\0",
        b"ACCESS_DENIED_ACE_TYPE\0",
        b"SYSTEM_AUDIT_ACE_TYPE\0",
        b"SYSTEM_ALARM_ACE_TYPE\0",
        b"ACCESS_ALLOWED_COMPOUND_ACE_TYPE\0",
        b"ACCESS_ALLOWED_OBJECT_ACE_TYPE\0",
        b"ACCESS_DENIED_OBJECT_ACE_TYPE\0",
        b"SYSTEM_AUDIT_OBJECT_ACE_TYPE\0",
        b"SYSTEM_ALARM_OBJECT_ACE_TYPE\0",
        b"ACCESS_ALLOWED_CALLBACK_ACE_TYPE\0",
        b"ACCESS_DENIED_CALLBACK_ACE_TYPE\0",
        b"ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE\0",
        b"ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE\0",
        b"SYSTEM_AUDIT_CALLBACK_ACE_TYPE\0",
        b"SYSTEM_ALARM_CALLBACK_ACE_TYPE\0",
        b"SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE\0",
        b"SYSTEM_ALARM_CALLBACK_OBJECT_ACE_TYPE\0",
        b"SYSTEM_MANDATORY_LABEL_ACE_TYPE\0",
    ];

    if (ace_type as usize) < ACE_TYPE_STRS.len() {
        ACE_TYPE_STRS[ace_type as usize].as_ptr() as PCSTR
    } else {
        b"UNKNOWN TYPE\0".as_ptr() as PCSTR
    }
}

#[cfg(debug_assertions)]
unsafe fn SepDumpAceFlags(ace_flags: u8) {
    use ntkrnl_sys::constants::*;

    if ace_flags & OBJECT_INHERIT_ACE != 0 {
        crate::dbg_print!(b"OBJECT_INHERIT_ACE\n\0");
    }
    if ace_flags & CONTAINER_INHERIT_ACE != 0 {
        crate::dbg_print!(b"CONTAINER_INHERIT_ACE\n\0");
    }
    if ace_flags & NO_PROPAGATE_INHERIT_ACE != 0 {
        crate::dbg_print!(b"NO_PROPAGATE_INHERIT_ACE\n\0");
    }
    if ace_flags & INHERIT_ONLY_ACE != 0 {
        crate::dbg_print!(b"INHERIT_ONLY_ACE\n\0");
    }
    if ace_flags & INHERITED_ACE != 0 {
        crate::dbg_print!(b"INHERITED_ACE\n\0");
    }
}

#[cfg(debug_assertions)]
unsafe fn SepDumpAces(acl: PACL) {
    use ntkrnl_sys::constants::*;

    let mut ace_index: ULONG = 0;
    while ace_index < (*acl).AceCount as ULONG {
        let mut ace: PACE = core::ptr::null_mut();
        let status = RtlGetAce(acl, ace_index, &mut ace as *mut _ as *mut PVOID);
        if status < 0 {
            crate::dbg_print!(b"SepDumpAces(): Failed to find the next ACE, stop dumping info...\n\0");
            return;
        }

        crate::dbg_print!(b"================== %lu# ACE DUMP INFO ==================\n\0", ace_index);
        crate::dbg_print!(b"Ace -> 0x%p\n\0", ace);
        crate::dbg_print!(b"Ace->Header -> 0x%p\n\0", core::ptr::addr_of!((*ace).Header));
        crate::dbg_print!(b"Ace->Header.AceType -> %s\n\0", SepGetAceTypeString((*ace).Header.AceType));
        crate::dbg_print!(b"Ace->AccessMask -> 0x%08lx\n\0", (*ace).AccessMask);

        let sid = SepGetSidFromAce(ace);
        debug_assert!(!sid.is_null());
        let mut sid_string: _UNICODE_STRING = core::mem::zeroed();
        RtlConvertSidToUnicodeString(&mut sid_string as *mut _UNICODE_STRING, sid, 1u8);
        crate::dbg_print!(b"Ace SID -> %wZ\n\0", &sid_string as *const _UNICODE_STRING);
        RtlFreeUnicodeString(&mut sid_string as *mut _UNICODE_STRING);

        crate::dbg_print!(b"Ace->Header.AceSize -> %u\n\0", (*ace).Header.AceSize as u32);
        crate::dbg_print!(b"Ace->Header.AceFlags:\n\0");
        SepDumpAceFlags((*ace).Header.AceFlags);

        ace_index += 1;
    }
}

#[cfg(debug_assertions)]
unsafe fn SepDumpAclInfo(acl: PACL, is_sacl: BOOLEAN) {
    crate::dbg_print!(b"================== %s DUMP INFO ==================\n\0",
        if is_sacl != 0 { b"SACL\0".as_ptr() as PCSTR } else { b"DACL\0".as_ptr() as PCSTR }
    );
    crate::dbg_print!(b"Acl->AclRevision -> %u\n\0", (*acl).AclRevision as u32);
    crate::dbg_print!(b"Acl->AclSize -> %u\n\0", (*acl).AclSize as u32);
    crate::dbg_print!(b"Acl->AceCount -> %u\n\0", (*acl).AceCount as u32);

    SepDumpAces(acl);
}

#[cfg(debug_assertions)]
unsafe fn SepDumpSdControlInfo(sd_control: SECURITY_DESCRIPTOR_CONTROL) {
    use ntkrnl_sys::constants::*;

    if sd_control & SE_OWNER_DEFAULTED != 0 {
        crate::dbg_print!(b"SE_OWNER_DEFAULTED\n\0");
    }
    if sd_control & SE_GROUP_DEFAULTED != 0 {
        crate::dbg_print!(b"SE_GROUP_DEFAULTED\n\0");
    }
    if sd_control & SE_DACL_PRESENT != 0 {
        crate::dbg_print!(b"SE_DACL_PRESENT\n\0");
    }
    if sd_control & SE_DACL_DEFAULTED != 0 {
        crate::dbg_print!(b"SE_DACL_DEFAULTED\n\0");
    }
    if sd_control & SE_SACL_PRESENT != 0 {
        crate::dbg_print!(b"SE_SACL_PRESENT\n\0");
    }
    if sd_control & SE_SACL_DEFAULTED != 0 {
        crate::dbg_print!(b"SE_SACL_DEFAULTED\n\0");
    }
    if sd_control & SE_DACL_UNTRUSTED != 0 {
        crate::dbg_print!(b"SE_DACL_UNTRUSTED\n\0");
    }
    if sd_control & SE_SERVER_SECURITY != 0 {
        crate::dbg_print!(b"SE_SERVER_SECURITY\n\0");
    }
    if sd_control & SE_DACL_AUTO_INHERIT_REQ != 0 {
        crate::dbg_print!(b"SE_DACL_AUTO_INHERIT_REQ\n\0");
    }
    if sd_control & SE_SACL_AUTO_INHERIT_REQ != 0 {
        crate::dbg_print!(b"SE_SACL_AUTO_INHERIT_REQ\n\0");
    }
    if sd_control & SE_DACL_AUTO_INHERITED != 0 {
        crate::dbg_print!(b"SE_DACL_AUTO_INHERITED\n\0");
    }
    if sd_control & SE_SACL_AUTO_INHERITED != 0 {
        crate::dbg_print!(b"SE_SACL_AUTO_INHERITED\n\0");
    }
    if sd_control & SE_DACL_PROTECTED != 0 {
        crate::dbg_print!(b"SE_DACL_PROTECTED\n\0");
    }
    if sd_control & SE_SACL_PROTECTED != 0 {
        crate::dbg_print!(b"SE_SACL_PROTECTED\n\0");
    }
    if sd_control & SE_RM_CONTROL_VALID != 0 {
        crate::dbg_print!(b"SE_RM_CONTROL_VALID\n\0");
    }
    if sd_control & SE_SELF_RELATIVE != 0 {
        crate::dbg_print!(b"SE_SELF_RELATIVE\n\0");
    }
}

#[cfg(debug_assertions)]
unsafe fn SepDumpSidsOfToken(sids: PSID_AND_ATTRIBUTES, sid_count: ULONG) {
    let mut sid_index: ULONG = 0;
    while sid_index < sid_count {
        let mut sid_string: _UNICODE_STRING = core::mem::zeroed();
        RtlConvertSidToUnicodeString(&mut sid_string as *mut _UNICODE_STRING, (*sids.offset(sid_index as isize)).Sid, 1u8);
        crate::dbg_print!(b"%lu# %wZ\n\0", sid_index, &sid_string as *const _UNICODE_STRING);
        RtlFreeUnicodeString(&mut sid_string as *mut _UNICODE_STRING);
        sid_index += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn SepDumpSdDebugInfo(security_descriptor: PISECURITY_DESCRIPTOR) {
    if security_descriptor.is_null() {
        return;
    }

    #[cfg(debug_assertions)]
    {
        let owner_sid = SepGetOwnerFromDescriptor(security_descriptor as PVOID);
        let group_sid = SepGetGroupFromDescriptor(security_descriptor as PVOID);
        let sacl = SepGetSaclFromDescriptor(security_descriptor as PVOID);
        let dacl = SepGetDaclFromDescriptor(security_descriptor as PVOID);

        crate::dbg_print!(b"================== SECURITY DESCRIPTOR DUMP INFO ==================\n\0");
        crate::dbg_print!(b"SecurityDescriptor -> 0x%p\n\0", security_descriptor as PVOID);
        crate::dbg_print!(b"SecurityDescriptor->Revision -> %u\n\0", (*security_descriptor).Revision as u32);
        crate::dbg_print!(b"SecurityDescriptor->Control:\n\0");
        SepDumpSdControlInfo((*security_descriptor).Control);

        if !owner_sid.is_null() {
            let mut sid_string: _UNICODE_STRING = core::mem::zeroed();
            RtlConvertSidToUnicodeString(&mut sid_string as *mut _UNICODE_STRING, owner_sid, 1u8);
            crate::dbg_print!(b"SD Owner SID -> %wZ\n\0", &sid_string as *const _UNICODE_STRING);
            RtlFreeUnicodeString(&mut sid_string as *mut _UNICODE_STRING);
        }

        if !group_sid.is_null() {
            let mut sid_string: _UNICODE_STRING = core::mem::zeroed();
            RtlConvertSidToUnicodeString(&mut sid_string as *mut _UNICODE_STRING, group_sid, 1u8);
            crate::dbg_print!(b"SD Group SID -> %wZ\n\0", &sid_string as *const _UNICODE_STRING);
            RtlFreeUnicodeString(&mut sid_string as *mut _UNICODE_STRING);
        }

        if !sacl.is_null() {
            SepDumpAclInfo(sacl, 1u8);
        }

        if !dacl.is_null() {
            SepDumpAclInfo(dacl, 0u8);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SepDumpTokenDebugInfo(token: PTOKEN) {
    let token = opaque_ptr(token);
    if token.is_null() {
        return;
    }

    #[cfg(debug_assertions)]
    {
        crate::dbg_print!(b"================== ACCESS TOKEN DUMP INFO ==================\n\0");
        crate::dbg_print!(b"Token -> 0x%p\n\0", token);
        crate::dbg_print!(b"Token->ImageFileName -> %s\n\0", (*token).ImageFileName.as_ptr());
        crate::dbg_print!(b"Token->TokenSource.SourceName -> \"%-.*s\"\n\0",
            (*token).TokenSource.SourceName.len(),
            (*token).TokenSource.SourceName.as_ptr()
        );
        crate::dbg_print!(b"Token->TokenSource.SourceIdentifier -> %lu.%lu\n\0",
            (*token).TokenSource.SourceIdentifier.HighPart,
            (*token).TokenSource.SourceIdentifier.LowPart
        );

        let mut sid_string: _UNICODE_STRING = core::mem::zeroed();
        RtlConvertSidToUnicodeString(&mut sid_string as *mut _UNICODE_STRING, (*token).PrimaryGroup, 1u8);
        crate::dbg_print!(b"Token primary group SID -> %wZ\n\0", &sid_string as *const _UNICODE_STRING);
        RtlFreeUnicodeString(&mut sid_string as *mut _UNICODE_STRING);

        crate::dbg_print!(b"Token user and groups SIDs:\n\0");
        SepDumpSidsOfToken((*token).UserAndGroups, (*token).UserAndGroupCount);

        if SeTokenIsRestricted(token as PVOID) != 0 {
            crate::dbg_print!(b"Token restricted SIDs:\n\0");
            SepDumpSidsOfToken((*token).RestrictedSids, (*token).RestrictedSidCount);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SepDumpAccessRightsStats(access_rights: PACCESS_CHECK_RIGHTS) {
    if (*access_rights).RemainingAccessRights == 0 {
        return;
    }

    #[cfg(debug_assertions)]
    {
        crate::dbg_print!(b"================== ACCESS CHECK RIGHTS STATISTICS ==================\n\0");
        crate::dbg_print!(b"Remaining access rights -> 0x%08lx\n\0", (*access_rights).RemainingAccessRights);
        crate::dbg_print!(b"Granted access rights -> 0x%08lx\n\0", (*access_rights).GrantedAccessRights);
        crate::dbg_print!(b"Denied access rights -> 0x%08lx\n\0", (*access_rights).DeniedAccessRights);
    }
}

#[no_mangle]
pub unsafe extern "C" fn SepDumpAccessAndStatusList(
    granted_access_list: PACCESS_MASK,
    access_status_list: PNTSTATUS,
    is_result_list: BOOLEAN,
    object_type_list: POBJECT_TYPE_LIST_INTERNAL,
    object_type_list_length: ULONG,
) {
    let access_status_list = opaque_ptr(access_status_list);
    #[cfg(debug_assertions)]
    {
        crate::dbg_print!(b"================== ACCESS & STATUS OBJECT TYPE LIST STATISTICS ==================\n\0");
        let result_list_length = if is_result_list != 0 { object_type_list_length } else { 1 };
        let mut result_list_index: ULONG = 0;
        while result_list_index < result_list_length {
            crate::dbg_print!(b"Result Index #%lu, Granted access rights -> 0x%08lx, Access status -> 0x%08lx\n\0",
                result_list_index,
                *granted_access_list.offset(result_list_index as isize),
                *access_status_list.offset(result_list_index as isize)
            );
            result_list_index += 1;
        }

        let mut object_type_index: ULONG = 0;
        while object_type_index < object_type_list_length {
            crate::dbg_print!(b"================== #%lu OBJECT ACCESS RIGHTS ==================\n\0", object_type_index);
            let obj = &*object_type_list.offset(object_type_index as isize);
            crate::dbg_print!(b"Remaining access rights -> 0x%08lx\n\0", obj.ObjectAccessRights.RemainingAccessRights);
            crate::dbg_print!(b"Granted access rights -> 0x%08lx\n\0", obj.ObjectAccessRights.GrantedAccessRights);
            crate::dbg_print!(b"Denied access rights -> 0x%08lx\n\0", obj.ObjectAccessRights.DeniedAccessRights);
            object_type_index += 1;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security subsystem debug routines support
//  * COPYRIGHT:   Copyright 2022-2023 George Bișoc <george.bisoc@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// #ifndef NDEBUG
// /**
//  * @brief
//  * Converts an Access Control Entry (ACE) type to a string.
//  *
//  * @return
//  * Returns a converted ACE type strings. If no
//  * known ACE type is found, it will return
//  * UNKNOWN TYPE.
//  */
// static
// PCSTR
// SepGetAceTypeString(
//     _In_ UCHAR AceType)
// {
// #define TOSTR(x)    #x
//     static const PCSTR AceTypes[] =
//     {
//         TOSTR(ACCESS_ALLOWED_ACE_TYPE),
//         TOSTR(ACCESS_DENIED_ACE_TYPE),
//         TOSTR(SYSTEM_AUDIT_ACE_TYPE),
//         TOSTR(SYSTEM_ALARM_ACE_TYPE),
//         TOSTR(ACCESS_ALLOWED_COMPOUND_ACE_TYPE),
//         TOSTR(ACCESS_ALLOWED_OBJECT_ACE_TYPE),
//         TOSTR(ACCESS_DENIED_OBJECT_ACE_TYPE),
//         TOSTR(SYSTEM_AUDIT_OBJECT_ACE_TYPE),
//         TOSTR(SYSTEM_ALARM_OBJECT_ACE_TYPE),
//         TOSTR(ACCESS_ALLOWED_CALLBACK_ACE_TYPE),
//         TOSTR(ACCESS_DENIED_CALLBACK_ACE_TYPE),
//         TOSTR(ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE),
//         TOSTR(ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE),
//         TOSTR(SYSTEM_AUDIT_CALLBACK_ACE_TYPE),
//         TOSTR(SYSTEM_ALARM_CALLBACK_ACE_TYPE),
//         TOSTR(SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE),
//         TOSTR(SYSTEM_ALARM_CALLBACK_OBJECT_ACE_TYPE),
//         TOSTR(SYSTEM_MANDATORY_LABEL_ACE_TYPE),
//     };
// #undef TOSTR
//
//     if (AceType < RTL_NUMBER_OF(AceTypes))
//         return AceTypes[AceType];
//     else
//         return "UNKNOWN TYPE";
// }
//
// /**
//  * @brief
//  * Dumps the ACE flags to the debugger output.
//  */
// static
// VOID
// SepDumpAceFlags(
//     _In_ UCHAR AceFlags)
// {
// #define ACE_FLAG_PRINT(x)  \
//     if (AceFlags & x)      \
//     {                      \
//         DbgPrint(#x "\n"); \
//     }
//
//     ACE_FLAG_PRINT(OBJECT_INHERIT_ACE);
//     ACE_FLAG_PRINT(CONTAINER_INHERIT_ACE);
//     ACE_FLAG_PRINT(NO_PROPAGATE_INHERIT_ACE);
//     ACE_FLAG_PRINT(INHERIT_ONLY_ACE);
//     ACE_FLAG_PRINT(INHERITED_ACE);
// #undef ACE_FLAG_PRINT
// }
//
// /**
//  * @brief
//  * Iterates and dumps each ACE debug info in an ACL.
//  */
// static
// VOID
// SepDumpAces(
//     _In_ PACL Acl)
// {
//     NTSTATUS Status;
//     PACE Ace;
//     ULONG AceIndex;
//     PSID Sid;
//     UNICODE_STRING SidString;
//
//     /* Loop all ACEs and dump their info */
//     for (AceIndex = 0; AceIndex < Acl->AceCount; AceIndex++)
//     {
//         /* Get the ACE at this index */
//         Status = RtlGetAce(Acl, AceIndex, (PVOID*)&Ace);
//         if (!NT_SUCCESS(Status))
//         {
//             /*
//              * Normally this should never happen.
//              * Just fail gracefully and stop further
//              * debugging of ACEs.
//              */
//             DbgPrint("SepDumpAces(): Failed to find the next ACE, stop dumping info...\n");
//             return;
//         }
//
//         DbgPrint("================== %lu# ACE DUMP INFO ==================\n", AceIndex);
//         DbgPrint("Ace -> 0x%p\n", Ace);
//         DbgPrint("Ace->Header -> 0x%p\n", Ace->Header);
//         DbgPrint("Ace->Header.AceType -> %s\n", SepGetAceTypeString(Ace->Header.AceType));
//         DbgPrint("Ace->AccessMask -> 0x%08lx\n", Ace->AccessMask);
//
//         Sid = SepGetSidFromAce(Ace);
//         ASSERT(Sid);
//         RtlConvertSidToUnicodeString(&SidString, Sid, TRUE);
//         DbgPrint("Ace SID -> %wZ\n", &SidString);
//         RtlFreeUnicodeString(&SidString);
//
//         DbgPrint("Ace->Header.AceSize -> %u\n", Ace->Header.AceSize);
//         DbgPrint("Ace->Header.AceFlags:\n");
//         SepDumpAceFlags(Ace->Header.AceFlags);
//     }
// }
//
// /**
//  * @brief
//  * Dumps debug info of an Access Control List (ACL).
//  */
// static
// VOID
// SepDumpAclInfo(
//     _In_ PACL Acl,
//     _In_ BOOLEAN IsSacl)
// {
//     /* Dump relevant info */
//     DbgPrint("================== %s DUMP INFO ==================\n", IsSacl ? "SACL" : "DACL");
//     DbgPrint("Acl->AclRevision -> %u\n", Acl->AclRevision);
//     DbgPrint("Acl->AclSize -> %u\n", Acl->AclSize);
//     DbgPrint("Acl->AceCount -> %u\n", Acl->AceCount);
//
//     /* Dump all the ACEs present on this ACL */
//     SepDumpAces(Acl);
// }
//
// /**
//  * @brief
//  * Dumps control flags of a security descriptor to the debugger.
//  */
// static
// VOID
// SepDumpSdControlInfo(
//     _In_ SECURITY_DESCRIPTOR_CONTROL SdControl)
// {
// #define SD_CONTROL_PRINT(x) \
//     if (SdControl & x)      \
//     {                       \
//         DbgPrint(#x "\n");  \
//     }
//
//     SD_CONTROL_PRINT(SE_OWNER_DEFAULTED);
//     SD_CONTROL_PRINT(SE_GROUP_DEFAULTED);
//     SD_CONTROL_PRINT(SE_DACL_PRESENT);
//     SD_CONTROL_PRINT(SE_DACL_DEFAULTED);
//     SD_CONTROL_PRINT(SE_SACL_PRESENT);
//     SD_CONTROL_PRINT(SE_SACL_DEFAULTED);
//     SD_CONTROL_PRINT(SE_DACL_UNTRUSTED);
//     SD_CONTROL_PRINT(SE_SERVER_SECURITY);
//     SD_CONTROL_PRINT(SE_DACL_AUTO_INHERIT_REQ);
//     SD_CONTROL_PRINT(SE_SACL_AUTO_INHERIT_REQ);
//     SD_CONTROL_PRINT(SE_DACL_AUTO_INHERITED);
//     SD_CONTROL_PRINT(SE_SACL_AUTO_INHERITED);
//     SD_CONTROL_PRINT(SE_DACL_PROTECTED);
//     SD_CONTROL_PRINT(SE_SACL_PROTECTED);
//     SD_CONTROL_PRINT(SE_RM_CONTROL_VALID);
//     SD_CONTROL_PRINT(SE_SELF_RELATIVE);
// #undef SD_CONTROL_PRINT
// }
//
// /**
//  * @brief
//  * Dumps each security identifier (SID) of an access token to debugger.
//  */
// static
// VOID
// SepDumpSidsOfToken(
//     _In_ PSID_AND_ATTRIBUTES Sids,
//     _In_ ULONG SidCount)
// {
//     ULONG SidIndex;
//     UNICODE_STRING SidString;
//
//     /* Loop all SIDs and dump them */
//     for (SidIndex = 0; SidIndex < SidCount; SidIndex++)
//     {
//         RtlConvertSidToUnicodeString(&SidString, Sids[SidIndex].Sid, TRUE);
//         DbgPrint("%lu# %wZ\n", SidIndex, &SidString);
//         RtlFreeUnicodeString(&SidString);
//     }
// }
// #endif
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Dumps debug information of a security descriptor to the debugger.
//  */
// VOID
// SepDumpSdDebugInfo(
//     _In_opt_ PISECURITY_DESCRIPTOR SecurityDescriptor)
// {
// #ifndef NDEBUG
//     UNICODE_STRING SidString;
//     PSID OwnerSid, GroupSid;
//     PACL Dacl, Sacl;
// #endif
//
//     /* Don't dump anything if no SD was provided */
//     if (!SecurityDescriptor)
//     {
//         return;
//     }
//
// #ifndef NDEBUG
//     /* Cache the necessary security buffers to dump info from */
//     OwnerSid = SepGetOwnerFromDescriptor(SecurityDescriptor);
//     GroupSid = SepGetGroupFromDescriptor(SecurityDescriptor);
//     Sacl = SepGetSaclFromDescriptor(SecurityDescriptor);
//     Dacl = SepGetDaclFromDescriptor(SecurityDescriptor);
//
//     DbgPrint("================== SECURITY DESCRIPTOR DUMP INFO ==================\n");
//     DbgPrint("SecurityDescriptor -> 0x%p\n", SecurityDescriptor);
//     DbgPrint("SecurityDescriptor->Revision -> %u\n", SecurityDescriptor->Revision);
//     DbgPrint("SecurityDescriptor->Control:\n");
//     SepDumpSdControlInfo(SecurityDescriptor->Control);
//
//     /* Dump the Owner SID if the SD belongs to an owner */
//     if (OwnerSid)
//     {
//         RtlConvertSidToUnicodeString(&SidString, OwnerSid, TRUE);
//         DbgPrint("SD Owner SID -> %wZ\n", &SidString);
//         RtlFreeUnicodeString(&SidString);
//     }
//
//     /* Dump the Group SID if the SD belongs to a group */
//     if (GroupSid)
//     {
//         RtlConvertSidToUnicodeString(&SidString, GroupSid, TRUE);
//         DbgPrint("SD Group SID -> %wZ\n", &SidString);
//         RtlFreeUnicodeString(&SidString);
//     }
//
//     /* Dump the ACL contents of SACL if this SD has one */
//     if (Sacl)
//     {
//         SepDumpAclInfo(Sacl, TRUE);
//     }
//
//     /* Dump the ACL contents of DACL if this SD has one */
//     if (Dacl)
//     {
//         SepDumpAclInfo(Dacl, FALSE);
//     }
// #endif
// }
//
// /**
//  * @brief
//  * Dumps debug information of an access token to the debugger.
//  */
// VOID
// SepDumpTokenDebugInfo(
//     _In_opt_ PTOKEN Token)
// {
// #ifndef NDEBUG
//     UNICODE_STRING SidString;
// #endif
//
//     /* Don't dump anything if no token was provided */
//     if (!Token)
//     {
//         return;
//     }
//
// #ifndef NDEBUG
//     /* Dump relevant token info */
//     DbgPrint("================== ACCESS TOKEN DUMP INFO ==================\n");
//     DbgPrint("Token -> 0x%p\n", Token);
//     DbgPrint("Token->ImageFileName -> %s\n", Token->ImageFileName);
//     DbgPrint("Token->TokenSource.SourceName -> \"%-.*s\"\n",
//              RTL_NUMBER_OF(Token->TokenSource.SourceName),
//              Token->TokenSource.SourceName);
//     DbgPrint("Token->TokenSource.SourceIdentifier -> %lu.%lu\n",
//              Token->TokenSource.SourceIdentifier.HighPart,
//              Token->TokenSource.SourceIdentifier.LowPart);
//
//     RtlConvertSidToUnicodeString(&SidString, Token->PrimaryGroup, TRUE);
//     DbgPrint("Token primary group SID -> %wZ\n", &SidString);
//     RtlFreeUnicodeString(&SidString);
//
//     DbgPrint("Token user and groups SIDs:\n");
//     SepDumpSidsOfToken(Token->UserAndGroups, Token->UserAndGroupCount);
//
//     if (SeTokenIsRestricted(Token))
//     {
//         DbgPrint("Token restricted SIDs:\n");
//         SepDumpSidsOfToken(Token->RestrictedSids, Token->RestrictedSidCount);
//     }
// #endif
// }
//
// /**
//  * @brief
//  * Dumps security access rights to the debugger.
//  */
// VOID
// SepDumpAccessRightsStats(
//     _In_ PACCESS_CHECK_RIGHTS AccessRights)
// {
//     /*
//      * Dump the access rights only if we have remaining rights
//      * to dump in the first place. RemainingAccessRights can be 0
//      * if access check procedure has failed prematurely and this
//      * member hasn't been filled yet.
//      */
//     if (!AccessRights->RemainingAccessRights)
//     {
//         return;
//     }
//
// #ifndef NDEBUG
//     DbgPrint("================== ACCESS CHECK RIGHTS STATISTICS ==================\n");
//     DbgPrint("Remaining access rights -> 0x%08lx\n", AccessRights->RemainingAccessRights);
//     DbgPrint("Granted access rights -> 0x%08lx\n", AccessRights->GrantedAccessRights);
//     DbgPrint("Denied access rights -> 0x%08lx\n", AccessRights->DeniedAccessRights);
// #endif
// }
//
// /**
//  * @brief
//  * Dumps access and status values of each object type
//  * in the result list.
//  */
// VOID
// SepDumpAccessAndStatusList(
//     _In_ PACCESS_MASK GrantedAccessList,
//     _In_ PNTSTATUS AccessStatusList,
//     _In_ BOOLEAN IsResultList,
//     _In_ POBJECT_TYPE_LIST_INTERNAL ObjectTypeList,
//     _In_ ULONG ObjectTypeListLength)
// {
// #ifndef NDEBUG
//     ULONG ResultListIndex;
//     ULONG ObjectTypeIndex;
//     ULONG ResultListLength;
//
//     DbgPrint("================== ACCESS & STATUS OBJECT TYPE LIST STATISTICS ==================\n");
//     ResultListLength = IsResultList ? ObjectTypeListLength : 1;
//     for (ResultListIndex = 0; ResultListIndex < ResultListLength; ResultListIndex++)
//     {
//         DbgPrint("Result Index #%lu, Granted access rights -> 0x%08lx, Access status -> 0x%08lx\n",
//             ResultListIndex, GrantedAccessList[ResultListIndex], AccessStatusList[ResultListIndex]);
//     }
//
//     for (ObjectTypeIndex = 0; ObjectTypeIndex < ObjectTypeListLength; ObjectTypeIndex++)
//     {
//         DbgPrint("================== #%lu OBJECT ACCESS RIGHTS ==================\n", ObjectTypeIndex);
//         DbgPrint("Remaining access rights -> 0x%08lx\n", ObjectTypeList[ObjectTypeIndex].ObjectAccessRights.RemainingAccessRights);
//         DbgPrint("Granted access rights -> 0x%08lx\n", ObjectTypeList[ObjectTypeIndex].ObjectAccessRights.GrantedAccessRights);
//         DbgPrint("Denied access rights -> 0x%08lx\n", ObjectTypeList[ObjectTypeIndex].ObjectAccessRights.DeniedAccessRights);
//     }
// #endif
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
