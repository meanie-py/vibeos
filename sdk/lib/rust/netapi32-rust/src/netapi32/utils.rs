use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = _SID_IDENTIFIER_AUTHORITY {
    Value: [
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        5 as ::core::ffi::c_int as BYTE,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn GetAccountDomainSid(ServerName: PUNICODE_STRING, AccountDomainSid: *mut PSID) -> NTSTATUS {
    let mut AccountDomainInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = LSA_OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut u8,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );

    Status = LsaOpenPolicy(
        ServerName as PLSA_UNICODE_STRING,
        core::ptr::addr_of_mut!(ObjectAttributes),
        POLICY_VIEW_LOCAL_INFORMATION,
        core::ptr::addr_of_mut!(PolicyHandle),
    ) as NTSTATUS;

    if !NT_SUCCESS(Status) {
        ros_dbg_log(
            __WINE_DBCL_ERR,
            __wine_dbch___default,
            b"dll/win32/netapi32/utils.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"GetAccountDomainSid\0".as_ptr() as *const core::ffi::c_char,
            40,
            b"LsaOpenPolicy failed (Status %08lx)\n\0".as_ptr() as *const core::ffi::c_char,
            Status,
        );
        return Status;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        core::ptr::addr_of_mut!(AccountDomainInfo) as *mut PVOID,
    ) as NTSTATUS;

    if !NT_SUCCESS(Status) {
        ros_dbg_log(
            __WINE_DBCL_ERR,
            __wine_dbch___default,
            b"dll/win32/netapi32/utils.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"GetAccountDomainSid\0".as_ptr() as *const core::ffi::c_char,
            49,
            b"LsaQueryInformationPolicy failed (Status %08lx)\n\0".as_ptr() as *const core::ffi::c_char,
            Status,
        );
    } else {
        Length = RtlLengthSid((*AccountDomainInfo).DomainSid);
        *AccountDomainSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as SIZE_T) as PSID;

        if (*AccountDomainSid).is_null() {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/utils.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"GetAccountDomainSid\0".as_ptr() as *const core::ffi::c_char,
                58,
                b"Failed to allocate SID\n\0".as_ptr() as *const core::ffi::c_char,
            );
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            core::ptr::copy_nonoverlapping(
                (*AccountDomainInfo).DomainSid as *const u8,
                *AccountDomainSid as *mut u8,
                Length as usize,
            );
        }
    }

    if !AccountDomainInfo.is_null() {
        LsaFreeMemory(AccountDomainInfo as PVOID);
    }

    LsaClose(PolicyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn GetBuiltinDomainSid(BuiltinDomainSid: *mut PSID) -> NTSTATUS {
    let mut Sid: PSID = core::ptr::null_mut();
    *BuiltinDomainSid = core::ptr::null_mut();

    Sid = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        RtlLengthRequiredSid(1) as SIZE_T,
    );
    if Sid.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut status: NTSTATUS;
    loop {
        status = RtlInitializeSid(Sid, &raw mut NtAuthority, 1u8);
        if !NT_SUCCESS(status) {
            break;
        }

        let Ptr = RtlSubAuthoritySid(Sid, 0);
        *Ptr = SECURITY_BUILTIN_DOMAIN_RID as u32;
        *BuiltinDomainSid = Sid;
        break;
    }

    if !NT_SUCCESS(status) {
        if !Sid.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Sid as *mut core::ffi::c_void);
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn OpenAccountDomain(
    ServerHandle: SAM_HANDLE,
    ServerName: PUNICODE_STRING,
    DesiredAccess: ULONG,
    DomainHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let mut DomainSid: PSID = core::ptr::null_mut();
    let Status: NTSTATUS = GetAccountDomainSid(ServerName, &raw mut DomainSid);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let Status: NTSTATUS = SamOpenDomain(
        ServerHandle,
        DesiredAccess as ACCESS_MASK,
        DomainSid,
        DomainHandle,
    );
    RtlFreeHeap(RtlGetProcessHeap(), 0u32, DomainSid as *mut core::ffi::c_void);
    if !NT_SUCCESS(Status) {
        // Empty block preserved from C code
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn OpenBuiltinDomain(
    ServerHandle: SAM_HANDLE,
    DesiredAccess: ULONG,
    DomainHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let mut DomainSid: PSID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = GetBuiltinDomainSid(&raw mut DomainSid);
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/netapi32/utils.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"OpenBuiltinDomain\0".as_ptr() as *const ::core::ffi::c_char,
                156 as ::core::ffi::c_int,
                b"GetBuiltinDomainSid failed (Status %08lx)\n\0".as_ptr() as *const ::core::ffi::c_char,
                Status,
            );
        }
        return Status;
    }

    Status = SamOpenDomain(
        ServerHandle,
        DesiredAccess as ACCESS_MASK,
        DomainSid,
        DomainHandle,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid as PVOID);

    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/netapi32/utils.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"OpenBuiltinDomain\0".as_ptr() as *const ::core::ffi::c_char,
                169 as ::core::ffi::c_int,
                b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const ::core::ffi::c_char,
                Status,
            );
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn BuildSidFromSidAndRid(
    SrcSid: PSID,
    RelativeId: ULONG,
    DestSid: *mut PSID,
) -> DWORD {
    let RidCount: UCHAR = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8u8 {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    let DstSidSize: ULONG = RtlLengthRequiredSid((RidCount as ULONG + 1));
    let mut DstSid: PSID = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    ApiStatus = NetApiBufferAllocate(DstSidSize as DWORD, &mut DstSid);
    if ApiStatus != NERR_Success as DWORD {
        return ApiStatus;
    }
    RtlInitializeSid(
        DstSid,
        RtlIdentifierAuthoritySid(SrcSid),
        (RidCount + 1) as UCHAR,
    );
    for i in 0..RidCount as ULONG {
        let p: *mut ULONG = RtlSubAuthoritySid(SrcSid, i);
        let q: *mut ULONG = RtlSubAuthoritySid(DstSid, i);
        *q = *p;
    }
    let q: *mut ULONG = RtlSubAuthoritySid(DstSid, RidCount as ULONG);
    *q = RelativeId;
    *DestSid = DstSid;
    NERR_Success as DWORD
}
#[no_mangle]
pub unsafe extern "C" fn CopySidFromSidAndRid(
    DstSid: PSID,
    SrcSid: PSID,
    RelativeId: u32,
) {
    let RidCount: u8 = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8 {
        return;
    }

    RtlInitializeSid(
        DstSid,
        RtlIdentifierAuthoritySid(SrcSid),
        (RidCount + 1) as u8,
    );

    let mut i: u32 = 0;
    while i < RidCount as u32 {
        let p = RtlSubAuthoritySid(SrcSid, i);
        let q = RtlSubAuthoritySid(DstSid, i);
        *q = *p;
        i += 1;
    }

    let q = RtlSubAuthoritySid(DstSid, RidCount as u32);
    *q = RelativeId;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         NetAPI DLL
//  * FILE:            reactos/dll/win32/netapi32/utils.c
//  * PURPOSE:         Helper functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "netapi32.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// /* GLOBALS *******************************************************************/
//
// static SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// GetAccountDomainSid( PUNICODE_STRING ServerName,
//                      PSID *AccountDomainSid)
// {
//     PPOLICY_ACCOUNT_DOMAIN_INFO AccountDomainInfo = NULL;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle = NULL;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//
//     Status = LsaOpenPolicy(ServerName,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaOpenPolicy failed (Status %08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)&AccountDomainInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaQueryInformationPolicy failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Length = RtlLengthSid(AccountDomainInfo->DomainSid);
//
//     *AccountDomainSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length);
//     if (*AccountDomainSid == NULL)
//     {
//         ERR("Failed to allocate SID\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     memcpy(*AccountDomainSid, AccountDomainInfo->DomainSid, Length);
//
// done:
//     if (AccountDomainInfo != NULL)
//         LsaFreeMemory(AccountDomainInfo);
//
//     LsaClose(PolicyHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// GetBuiltinDomainSid( PSID *BuiltinDomainSid)
// {
//     PSID Sid = NULL;
//     PULONG Ptr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     *BuiltinDomainSid = NULL;
//
//     Sid = RtlAllocateHeap(RtlGetProcessHeap(),
//                           0,
//                           RtlLengthRequiredSid(1));
//     if (Sid == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = RtlInitializeSid(Sid,
//                               &NtAuthority,
//                               1);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Ptr = RtlSubAuthoritySid(Sid, 0);
//     *Ptr = SECURITY_BUILTIN_DOMAIN_RID;
//
//     *BuiltinDomainSid = Sid;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (Sid != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Sid);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// OpenAccountDomain( SAM_HANDLE ServerHandle,
//                    PUNICODE_STRING ServerName,
//                    ULONG DesiredAccess,
//                    PSAM_HANDLE DomainHandle)
// {
//     PSID DomainSid = NULL;
//     NTSTATUS Status;
//
//     Status = GetAccountDomainSid(ServerName,
//                                  &DomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//         return Status;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DesiredAccess,
//                            DomainSid,
//                            DomainHandle);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// OpenBuiltinDomain( SAM_HANDLE ServerHandle,
//                    ULONG DesiredAccess,
//                    PSAM_HANDLE DomainHandle)
// {
//     PSID DomainSid = NULL;
//     NTSTATUS Status;
//
//     Status = GetBuiltinDomainSid(&DomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetBuiltinDomainSid failed (Status %08lx)\n", Status);
//         return Status;
//     }
//
//     Status = SamOpenDomain(ServerHandle,
//                            DesiredAccess,
//                            DomainSid,
//                            DomainHandle);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// NET_API_STATUS
// BuildSidFromSidAndRid( PSID SrcSid,
//                        ULONG RelativeId,
//                        PSID *DestSid)
// {
//     UCHAR RidCount;
//     PSID DstSid;
//     ULONG i;
//     ULONG DstSidSize;
//     PULONG p, q;
//     NET_API_STATUS ApiStatus = NERR_Success;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     if (RidCount >= 8)
//         return ERROR_INVALID_PARAMETER;
//
//     DstSidSize = RtlLengthRequiredSid(RidCount + 1);
//
//     ApiStatus = NetApiBufferAllocate(DstSidSize,
//                                      &DstSid);
//     if (ApiStatus != NERR_Success)
//         return ApiStatus;
//
//     RtlInitializeSid(DstSid,
//                      RtlIdentifierAuthoritySid(SrcSid),
//                      RidCount + 1);
//
//     for (i = 0; i < (ULONG)RidCount; i++)
//     {
//         p = RtlSubAuthoritySid(SrcSid, i);
//         q = RtlSubAuthoritySid(DstSid, i);
//         *q = *p;
//     }
//
//     q = RtlSubAuthoritySid(DstSid, (ULONG)RidCount);
//     *q = RelativeId;
//
//     *DestSid = DstSid;
//
//     return NERR_Success;
// }
//
//
// VOID
// CopySidFromSidAndRid(
//      PSID DstSid,
//      PSID SrcSid,
//      ULONG RelativeId)
// {
//     UCHAR RidCount;
//     ULONG i;
//     PULONG p, q;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     if (RidCount >= 8)
//         return;
//
//     RtlInitializeSid(DstSid,
//                      RtlIdentifierAuthoritySid(SrcSid),
//                      RidCount + 1);
//
//     for (i = 0; i < (ULONG)RidCount; i++)
//     {
//         p = RtlSubAuthoritySid(SrcSid, i);
//         q = RtlSubAuthoritySid(DstSid, i);
//         *q = *p;
//     }
//
//     q = RtlSubAuthoritySid(DstSid, (ULONG)RidCount);
//     *q = RelativeId;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
