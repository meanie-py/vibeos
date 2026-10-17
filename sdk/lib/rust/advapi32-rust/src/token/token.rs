use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::externs::HeapAlloc;
use core::ffi::c_void;

extern "system" {
    fn GetProcessHeap() -> PVOID;
    fn HeapFree(hHeap: PVOID, dwFlags: ULONG, lpMem: PVOID) -> i32;
}

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn CheckTokenMembership(
    ExistingTokenHandle: HANDLE,
    SidToCheck: PSID,
    IsMember: PBOOL,
) -> BOOL {
    #[repr(C)]
    struct PrivBuffer_t {
        PrivilegeSet: PRIVILEGE_SET,
        Privileges: [LUID_AND_ATTRIBUTES; 4],
    }

    let mut SecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut PrivBuffer: PrivBuffer_t = core::mem::zeroed();
    let mut PrivBufferSize: ULONG = core::mem::size_of::<PrivBuffer_t>() as ULONG;
    let mut GenericMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
        GenericRead: STANDARD_RIGHTS_READ,
        GenericWrite: STANDARD_RIGHTS_WRITE,
        GenericExecute: STANDARD_RIGHTS_EXECUTE,
        GenericAll: STANDARD_RIGHTS_ALL,
    };
    let Dacl: PACL;
    let SidLen: ULONG;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut AccessStatus: NTSTATUS = 0;

    *IsMember = 0;

    SidLen = RtlLengthSid(SidToCheck);

    if ExistingTokenHandle.is_null() {
        Status = NtOpenThreadToken(
            NtCurrentThread(),
            TOKEN_QUERY,
            0,
            &mut hToken,
        );

        if Status == STATUS_NO_TOKEN {
            Status = NtOpenProcessToken(
                NtCurrentProcess(),
                TOKEN_QUERY | TOKEN_DUPLICATE,
                &mut hToken,
            );
            if NT_SUCCESS(Status) {
                let mut hNewToken: HANDLE = core::ptr::null_mut();
                let DupRet: BOOL;

                DupRet = DuplicateTokenEx(
                    hToken,
                    TOKEN_QUERY | TOKEN_IMPERSONATE,
                    core::ptr::null_mut(),
                    SecurityImpersonation,
                    TokenImpersonation,
                    &mut hNewToken,
                );

                NtClose(hToken);

                if DupRet == 0 {
                    return 0;
                }

                hToken = hNewToken;
            }
        }

        if !NT_SUCCESS(Status) {
            return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
        }
    } else {
        hToken = ExistingTokenHandle;
    }

    SecurityDescriptor = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (core::mem::size_of::<_SECURITY_DESCRIPTOR>()
            + core::mem::size_of::<ACL>()
            + SidLen as usize
            + FIELD_OFFSET_ACCESS_ALLOWED_ACE_SidStart()) as SIZE_T,
    ) as PISECURITY_DESCRIPTOR;
    if SecurityDescriptor.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = RtlCreateSecurityDescriptor(
        SecurityDescriptor as PVOID,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = RtlSetOwnerSecurityDescriptor(SecurityDescriptor as PVOID, SidToCheck, 0);
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = RtlSetGroupSecurityDescriptor(SecurityDescriptor as PVOID, SidToCheck, 0);
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Dacl = SecurityDescriptor.add(1) as PACL;
    Status = RtlCreateAcl(
        Dacl,
        (core::mem::size_of::<ACL>()
            + SidLen as usize
            + FIELD_OFFSET_ACCESS_ALLOWED_ACE_SidStart()) as ULONG,
        ACL_REVISION,
    );
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, 0x1, SidToCheck);
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor as PVOID, 1, Dacl, 0);
    if !NT_SUCCESS(Status) {
        return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
    }

    Status = NtAccessCheck(
        SecurityDescriptor as PVOID,
        hToken,
        0x1,
        &mut GenericMapping,
        &mut PrivBuffer.PrivilegeSet,
        &mut PrivBufferSize,
        &mut GrantedAccess,
        &mut AccessStatus,
    );
    if NT_SUCCESS(Status) && NT_SUCCESS(AccessStatus) && GrantedAccess == 0x1 {
        *IsMember = 1;
    }

    return cleanup(Status, hToken, ExistingTokenHandle, SecurityDescriptor);
}

unsafe fn cleanup(
    Status: NTSTATUS,
    hToken: HANDLE,
    ExistingTokenHandle: HANDLE,
    SecurityDescriptor: PISECURITY_DESCRIPTOR,
) -> BOOL {
    if !hToken.is_null() && hToken != ExistingTokenHandle {
        NtClose(hToken);
    }

    if !SecurityDescriptor.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SecurityDescriptor as PVOID);
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

unsafe fn FIELD_OFFSET_ACCESS_ALLOWED_ACE_SidStart() -> usize {
    let base = core::ptr::null::<ACCESS_ALLOWED_ACE>();
    core::ptr::addr_of!((*base).SidStart) as usize
}

#[no_mangle]
pub unsafe extern "system" fn IsTokenRestricted(TokenHandle: HANDLE) -> BOOL {

    let mut RetLength: ULONG = 0;
    let mut lpGroups: PTOKEN_GROUPS;
    let mut Status: NTSTATUS;
    let mut Ret: BOOL = 0;

    Status = NtQueryInformationToken(
        TokenHandle,
        TokenRestrictedSids,
        core::ptr::null_mut(),
        0,
        &mut RetLength,
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    loop {
        lpGroups = HeapAlloc(GetProcessHeap(), 0, (RetLength) as SIZE_T) as PTOKEN_GROUPS;
        if lpGroups.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }

        Status = NtQueryInformationToken(
            TokenHandle,
            TokenRestrictedSids,
            lpGroups as PVOID,
            RetLength,
            &mut RetLength,
        );
        if NT_SUCCESS(Status) {
            Ret = ((*lpGroups).GroupCount != 0) as BOOL;
        } else if Status == STATUS_BUFFER_TOO_SMALL {
            HeapFree(GetProcessHeap(), 0, lpGroups as PVOID);
            continue;
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }

        HeapFree(GetProcessHeap(), 0, lpGroups as PVOID);

        return Ret;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetSiteSidFromToken(TokenHandle: HANDLE) -> PSID {
    let mut RestrictedSids: PTOKEN_GROUPS;
    let mut RetLen: ULONG = 0;
    let mut i: UINT;
    let mut Status: NTSTATUS;
    let mut PSiteSid: PSID = core::ptr::null_mut();
    let InternetSiteAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 7],
    };

    Status = NtQueryInformationToken(
        TokenHandle,
        TokenRestrictedSids,
        core::ptr::null_mut(),
        0,
        &mut RetLen,
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    RestrictedSids = RtlAllocateHeap(RtlGetProcessHeap(), 0, RetLen as SIZE_T) as PTOKEN_GROUPS;
    if RestrictedSids.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return core::ptr::null_mut();
    }

    Status = NtQueryInformationToken(
        TokenHandle,
        TokenRestrictedSids,
        RestrictedSids as PVOID,
        RetLen,
        &mut RetLen,
    );
    if NT_SUCCESS(Status) {
        i = 0;
        while i < (*RestrictedSids).GroupCount {
            let Groups = (*RestrictedSids).Groups.as_ptr();
            let RSSid = (*Groups.add(i as usize)).Sid as *mut SID;

            if RtlCompareMemory(
                core::ptr::addr_of!((*RSSid).IdentifierAuthority) as *const _,
                &InternetSiteAuthority as *const _ as *const _,
                core::mem::size_of::<SID_IDENTIFIER_AUTHORITY>() as SIZE_T,
            ) == core::mem::size_of::<SID_IDENTIFIER_AUTHORITY>() as SIZE_T
            {
                PSiteSid = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    RtlLengthSid((*Groups.add(i as usize)).Sid) as SIZE_T,
                );
                if PSiteSid.is_null() {
                    SetLastError(ERROR_OUTOFMEMORY);
                } else {
                    RtlCopySid(
                        RtlLengthSid((*Groups.add(i as usize)).Sid),
                        PSiteSid,
                        (*Groups.add(i as usize)).Sid,
                    );
                }

                break;
            }
            i += 1;
        }
    } else {
        SetLastError(RtlNtStatusToDosError(Status));
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, RestrictedSids as PVOID);
    PSiteSid
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/token/token.c
//  * PURPOSE:         Token functions
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// #include <advapi32.h>
//
// #include <ndk/setypes.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// CheckTokenMembership(IN HANDLE ExistingTokenHandle,
//                      IN PSID SidToCheck,
//                      OUT PBOOL IsMember)
// {
//     PISECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     ACCESS_MASK GrantedAccess;
//     struct
//     {
//         PRIVILEGE_SET PrivilegeSet;
//         LUID_AND_ATTRIBUTES Privileges[4];
//     } PrivBuffer;
//     ULONG PrivBufferSize = sizeof(PrivBuffer);
//     _GENERIC_MAPPING GenericMapping =
//     {
//         STANDARD_RIGHTS_READ,
//         STANDARD_RIGHTS_WRITE,
//         STANDARD_RIGHTS_EXECUTE,
//         STANDARD_RIGHTS_ALL
//     };
//     PACL Dacl;
//     ULONG SidLen;
//     HANDLE hToken = NULL;
//     NTSTATUS Status, AccessStatus;
//
//     /* doesn't return gracefully if IsMember is NULL! */
//     *IsMember = FALSE;
//
//     SidLen = RtlLengthSid(SidToCheck);
//
//     if (ExistingTokenHandle == NULL)
//     {
//         Status = NtOpenThreadToken(NtCurrentThread(),
//                                    TOKEN_QUERY,
//                                    FALSE,
//                                    &hToken);
//
//         if (Status == STATUS_NO_TOKEN)
//         {
//             /* we're not impersonating, open the primary token */
//             Status = NtOpenProcessToken(NtCurrentProcess(),
//                                         TOKEN_QUERY | TOKEN_DUPLICATE,
//                                         &hToken);
//             if (NT_SUCCESS(Status))
//             {
//                 HANDLE hNewToken = FALSE;
//                 BOOL DupRet;
//
//                 /* duplicate the primary token to create an impersonation token */
//                 DupRet = DuplicateTokenEx(hToken,
//                                           TOKEN_QUERY | TOKEN_IMPERSONATE,
//                                           NULL,
//                                           SecurityImpersonation,
//                                           TokenImpersonation,
//                                           &hNewToken);
//
//                 NtClose(hToken);
//
//                 if (!DupRet)
//                 {
//                     WARN("Failed to duplicate the primary token!\n");
//                     return FALSE;
//                 }
//
//                 hToken = hNewToken;
//             }
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//     }
//     else
//     {
//         hToken = ExistingTokenHandle;
//     }
//
//     /* create a security descriptor */
//     SecurityDescriptor = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          sizeof(_SECURITY_DESCRIPTOR) +
//                                              sizeof(ACL) + SidLen +
//                                              FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart));
//     if (SecurityDescriptor == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     Status = RtlCreateSecurityDescriptor(SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* set the owner and group */
//     Status = RtlSetOwnerSecurityDescriptor(SecurityDescriptor,
//                                            SidToCheck,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     Status = RtlSetGroupSecurityDescriptor(SecurityDescriptor,
//                                            SidToCheck,
//                                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* create the DACL */
//     Dacl = (PACL)(SecurityDescriptor + 1);
//     Status = RtlCreateAcl(Dacl,
//                           sizeof(ACL) + SidLen + FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart),
//                           ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     0x1,
//                                     SidToCheck);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* assign the DACL to the security descriptor */
//     Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* it's time to perform the access check. Just use _some_ desired access right
//        (same as for the ACE) and see if we're getting it granted. This indicates
//        our SID is a member of the token. We however can't use a generic access
//        right as those aren't mapped and return an error (STATUS_GENERIC_NOT_MAPPED). */
//     Status = NtAccessCheck(SecurityDescriptor,
//                            hToken,
//                            0x1,
//                            &GenericMapping,
//                            &PrivBuffer.PrivilegeSet,
//                            &PrivBufferSize,
//                            &GrantedAccess,
//                            &AccessStatus);
//     if (NT_SUCCESS(Status) && NT_SUCCESS(AccessStatus) && (GrantedAccess == 0x1))
//     {
//         *IsMember = TRUE;
//     }
//
// Cleanup:
//     if (hToken != NULL && hToken != ExistingTokenHandle)
//     {
//         NtClose(hToken);
//     }
//
//     if (SecurityDescriptor != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(),
//                     0,
//                     SecurityDescriptor);
//     }
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
// BOOL WINAPI
// IsTokenRestricted(HANDLE TokenHandle)
// {
//     ULONG RetLength;
//     PTOKEN_GROUPS lpGroups;
//     NTSTATUS Status;
//     BOOL Ret = FALSE;
//
//     /* determine the required buffer size and allocate enough memory to read the
//        list of restricted SIDs */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenRestrictedSids,
//                                      NULL,
//                                      0,
//                                      &RetLength);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
// AllocAndReadRestrictedSids:
//     lpGroups = (PTOKEN_GROUPS)HeapAlloc(GetProcessHeap(),
//                                         0,
//                                         RetLength);
//     if (lpGroups == NULL)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     /* actually read the list of the restricted SIDs */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenRestrictedSids,
//                                      lpGroups,
//                                      RetLength,
//                                      &RetLength);
//     if (NT_SUCCESS(Status))
//     {
//         Ret = (lpGroups->GroupCount != 0);
//     }
//     else if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* looks like the token was modified in the meanwhile, let's just try again */
//         HeapFree(GetProcessHeap(),
//                  0,
//                  lpGroups);
//
//         goto AllocAndReadRestrictedSids;
//     }
//     else
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//     }
//
//     /* free allocated memory */
//     HeapFree(GetProcessHeap(),
//              0,
//              lpGroups);
//
//     return Ret;
// }
//
// /*
//  * @unimplemented
//  */
// PSID
// WINAPI
// GetSiteSidFromToken(IN HANDLE TokenHandle)
// {
//     PTOKEN_GROUPS RestrictedSids;
//     ULONG RetLen;
//     UINT i;
//     NTSTATUS Status;
//     PSID PSiteSid = NULL;
//     SID_IDENTIFIER_AUTHORITY InternetSiteAuthority = {SECURITY_INTERNETSITE_AUTHORITY};
//
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenRestrictedSids,
//                                      NULL,
//                                      0,
//                                      &RetLen);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     RestrictedSids = (PTOKEN_GROUPS)RtlAllocateHeap(RtlGetProcessHeap(),
//                                                     0,
//                                                     RetLen);
//     if (RestrictedSids == NULL)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return NULL;
//     }
//
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenRestrictedSids,
//                                      RestrictedSids,
//                                      RetLen,
//                                      &RetLen);
//     if (NT_SUCCESS(Status))
//     {
//         for (i = 0; i < RestrictedSids->GroupCount; i++)
//         {
//             SID* RSSid = RestrictedSids->Groups[i].Sid;
//
//             if (RtlCompareMemory(&(RSSid->IdentifierAuthority),
//                                  &InternetSiteAuthority,
//                                  sizeof(SID_IDENTIFIER_AUTHORITY)) ==
//                                  sizeof(SID_IDENTIFIER_AUTHORITY))
//             {
//                 PSiteSid = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            0,
//                                            RtlLengthSid((RestrictedSids->
//                                                          Groups[i]).Sid));
//                 if (PSiteSid == NULL)
//                 {
//                     SetLastError(ERROR_OUTOFMEMORY);
//                 }
//                 else
//                 {
//                     RtlCopySid(RtlLengthSid(RestrictedSids->Groups[i].Sid),
//                                PSiteSid,
//                                RestrictedSids->Groups[i].Sid);
//                 }
//
//                 break;
//             }
//         }
//     }
//     else
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, RestrictedSids);
//     return PSiteSid;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
