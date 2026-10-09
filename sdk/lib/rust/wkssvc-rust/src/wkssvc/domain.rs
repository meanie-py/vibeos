use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn NetpSetPrimaryDomain(lpWorkgroupName: *const u16) -> u32 {
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES =
        core::mem::zeroed();
    ZeroMemory(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut core::ffi::c_void,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    (*core::ptr::addr_of_mut!(ObjectAttributes)).Length =
        (core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>()) as u32;

    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Status: i32 = LsaOpenPolicy(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObjectAttributes),
        POLICY_TRUST_ADMIN,
        core::ptr::addr_of_mut!(PolicyHandle),
    );

    if !LSA_SUCCESS(Status) {
        return LsaNtStatusToWinError(Status);
    }

    let mut PrimaryDomainInfo: POLICY_PRIMARY_DOMAIN_INFO = core::mem::zeroed();
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PrimaryDomainInfo.Name) as *mut UNICODE_STRING,
        lpWorkgroupName,
    );
    (*core::ptr::addr_of_mut!(PrimaryDomainInfo.Sid)) = core::ptr::null_mut();

    Status = LsaSetInformationPolicy(
        PolicyHandle,
        PolicyPrimaryDomainInformation,
        core::ptr::addr_of_mut!(PrimaryDomainInfo) as PVOID,
    );

    LsaClose(PolicyHandle);

    LsaNtStatusToWinError(Status)
}
#[no_mangle]
pub unsafe extern "C" fn NetpJoinWorkgroup(lpWorkgroupName: LPCWSTR) -> u32 {
    let status = NetpSetPrimaryDomain(lpWorkgroupName);
    if status != 0u32 {
        return status;
    }
    0u32
}
#[no_mangle]
pub unsafe extern "C" fn NetpGetJoinInformation(
    name_buffer: *mut LPWSTR,
    buffer_type: *mut u32,
) -> u32 {
    let mut object_attributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut primary_domain_info: PPOLICY_PRIMARY_DOMAIN_INFO = core::ptr::null_mut();
    let mut policy_handle: LSA_HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    *buffer_type = NetSetupUnknownStatus;
    *name_buffer = core::ptr::null_mut();

    object_attributes.Length = (core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u16) as u32;

    status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut object_attributes,
        POLICY_VIEW_LOCAL_INFORMATION,
        &mut policy_handle,
    );
    if !LSA_SUCCESS(status) {
        return LsaNtStatusToWinError(status);
    }

    status = LsaQueryInformationPolicy(
        policy_handle,
        PolicyPrimaryDomainInformation,
        &mut primary_domain_info as *mut *mut _ as *mut PVOID,
    );
    if LSA_SUCCESS(status) {
        if (*primary_domain_info).Name.Length > 0 {
            if !(*primary_domain_info).Sid.is_null() {
                *buffer_type = NetSetupDomainName;
            } else {
                *buffer_type = NetSetupWorkgroupName;
            }
            *name_buffer = midl_user_allocate(
                ((*primary_domain_info).Name.Length as usize + core::mem::size_of::<WCHAR>()) as SIZE_T,
            ) as LPWSTR;
            if !(*name_buffer).is_null() {
                wcscpy(*name_buffer, (*primary_domain_info).Name.Buffer);
            }
        } else {
            *buffer_type = NetSetupUnjoined;
        }

        if !(*primary_domain_info).Sid.is_null() {
            LsaFreeMemory((*primary_domain_info).Sid);
        }
        LsaFreeMemory(primary_domain_info as PVOID);
    }

    LsaClose(policy_handle);

    LsaNtStatusToWinError(status)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Services
//  * FILE:             base/services/wkssvc/domain.c
//  * PURPOSE:          Workstation service
//  * PROGRAMMER:       Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wkssvc);
//
// /* FUNCTIONS *****************************************************************/
//
// static
// NET_API_STATUS
// NetpSetPrimaryDomain(
//      LPCWSTR lpWorkgroupName)
// {
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     POLICY_PRIMARY_DOMAIN_INFO PrimaryDomainInfo;
//     LSA_HANDLE PolicyHandle = NULL;
//     NTSTATUS Status;
//
//     ZeroMemory(&ObjectAttributes, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_TRUST_ADMIN,
//                            &PolicyHandle);
//     if (!LSA_SUCCESS(Status))
//         return LsaNtStatusToWinError(Status);
//
//     RtlInitUnicodeString(&PrimaryDomainInfo.Name,
//                          lpWorkgroupName);
//     PrimaryDomainInfo.Sid = NULL;
//
//     Status = LsaSetInformationPolicy(PolicyHandle,
//                                      PolicyPrimaryDomainInformation,
//                                      &PrimaryDomainInfo);
//
//     LsaClose(PolicyHandle);
//
//     return LsaNtStatusToWinError(Status);
// }
//
//
// NET_API_STATUS
// NetpJoinWorkgroup(
//      LPCWSTR lpWorkgroupName)
// {
//     NET_API_STATUS status;
//
//     FIXME("NetpJoinWorkgroup(%S)\n", lpWorkgroupName);
//
//     status = NetpSetPrimaryDomain(lpWorkgroupName);
//     if (status != NERR_Success)
//     {
//         ERR("NetpSetPrimaryDomain failed (Status %lu)\n", status);
//         return status;
//     }
//
//     return NERR_Success;
// }
//
//
// NET_API_STATUS
// NetpGetJoinInformation(
//     LPWSTR *NameBuffer,
//     PNETSETUP_JOIN_STATUS BufferType)
// {
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     PPOLICY_PRIMARY_DOMAIN_INFO PrimaryDomainInfo = NULL;
//     LSA_HANDLE PolicyHandle = NULL;
//     NTSTATUS Status;
//
//     *BufferType = NetSetupUnknownStatus;
//     *NameBuffer = NULL;
//
//     ZeroMemory(&ObjectAttributes, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION,
//                            &PolicyHandle);
//     if (!LSA_SUCCESS(Status))
//         return LsaNtStatusToWinError(Status);
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyPrimaryDomainInformation,
//                                        (PVOID*)&PrimaryDomainInfo);
//     if (LSA_SUCCESS(Status))
//     {
//         TRACE("Sid: %p\n", PrimaryDomainInfo->Sid);
//         TRACE("Name: %S\n", PrimaryDomainInfo->Name.Buffer);
//
//         if (PrimaryDomainInfo->Name.Length > 0)
//         {
//             if (PrimaryDomainInfo->Sid != NULL)
//                 *BufferType = NetSetupDomainName;
//             else
//                 *BufferType = NetSetupWorkgroupName;
//
//             *NameBuffer = midl_user_allocate(PrimaryDomainInfo->Name.Length + sizeof(WCHAR));
//             if (*NameBuffer)
//                 wcscpy(*NameBuffer, PrimaryDomainInfo->Name.Buffer);
//         }
//         else
//         {
//             *BufferType = NetSetupUnjoined;
//         }
//
//         if (PrimaryDomainInfo->Sid)
//             LsaFreeMemory(PrimaryDomainInfo->Sid);
//
//         LsaFreeMemory(PrimaryDomainInfo);
//     }
//
//     LsaClose(PolicyHandle);
//
//     return LsaNtStatusToWinError(Status);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
