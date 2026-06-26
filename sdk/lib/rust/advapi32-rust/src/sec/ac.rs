use crate::prelude::*;
use core::ffi::c_void;

// ntmarta dynamic-load table + presence check live in sec/misc.rs (where the
// NtMartaStatic global is defined #[no_mangle]). The Acc* calls below are the
// advapi32.h `#define Acc… NtMartaStatic.<field>` macros → call through the table.
use crate::sec_misc::{CheckNtMartaPresent, NtMartaStatic};

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn AddAccessAllowedObjectAce(
    pAcl: PACL,
    dwAceRevision: u32,
    AceFlags: u32,
    AccessMask: u32,
    ObjectTypeGuid: *mut GUID,
    InheritedObjectTypeGuid: *mut GUID,
    pSid: PSID,
) -> i32 {

    let Status = RtlAddAccessAllowedObjectAce(
        pAcl,
        dwAceRevision,
        AceFlags,
        AccessMask,
        ObjectTypeGuid,
        InheritedObjectTypeGuid,
        pSid,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAccessDeniedObjectAce(
    pAcl: PACL,
    dwAceRevision: u32,
    AceFlags: u32,
    AccessMask: u32,
    ObjectTypeGuid: *mut GUID,
    InheritedObjectTypeGuid: *mut GUID,
    pSid: PSID,
) -> i32 {

    let Status = RtlAddAccessDeniedObjectAce(
        pAcl,
        dwAceRevision,
        AceFlags,
        AccessMask,
        ObjectTypeGuid,
        InheritedObjectTypeGuid,
        pSid,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AddAuditAccessObjectAce(
    pAcl: PACL,
    dwAceRevision: u32,
    AceFlags: u32,
    AccessMask: u32,
    ObjectTypeGuid: *mut GUID,
    InheritedObjectTypeGuid: *mut GUID,
    pSid: PSID,
    bAuditSuccess: i32,
    bAuditFailure: i32,
) -> i32 {

    let Status = RtlAddAuditAccessObjectAce(
        pAcl,
        dwAceRevision,
        AceFlags,
        AccessMask,
        ObjectTypeGuid,
        InheritedObjectTypeGuid,
        pSid,
        bAuditSuccess as u8,
        bAuditFailure as u8,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetInheritanceSourceW(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    Container: BOOL,
    pObjectClassGuids: *mut *mut GUID,
    GuidCount: DWORD,
    pAcl: PACL,
    pfnArray: PFN_OBJECT_MGR_FUNCTS,
    pGenericMapping: PGENERIC_MAPPING,
    pInheritArray: PINHERITED_FROMW,
) -> DWORD {
    let mut ErrorCode: DWORD;

    ErrorCode = CheckNtMartaPresent();
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = (NtMartaStatic.GetInheritanceSourceW.unwrap())(
            pObjectName,
            ObjectType,
            SecurityInfo,
            Container,
            pObjectClassGuids as _,
            GuidCount,
            pAcl as _,
            pfnArray as _,
            pGenericMapping as _,
            pInheritArray as _,
        );
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetInheritanceSourceA(
    pObjectName: LPSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    Container: BOOL,
    pObjectClassGuids: *mut *mut GUID,
    GuidCount: DWORD,
    pAcl: PACL,
    pfnArray: PFN_OBJECT_MGR_FUNCTS,
    pGenericMapping: PGENERIC_MAPPING,
    pInheritArray: PINHERITED_FROMA,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn FreeInheritedFromArray(
    pInheritArray: PINHERITED_FROMW,
    AceCnt: USHORT,
    pfnArray: PFN_OBJECT_MGR_FUNCTS,
) -> DWORD {

    let mut ErrorCode: DWORD;

    ErrorCode = CheckNtMartaPresent();
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = (NtMartaStatic.FreeIndexArray.unwrap())(pInheritArray, AceCnt, pfnArray);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn SetEntriesInAclW(
    cCountOfExplicitEntries: ULONG,
    pListOfExplicitEntries: PEXPLICIT_ACCESS_W,
    OldAcl: PACL,
    NewAcl: *mut PACL,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if NewAcl.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    ErrorCode = CheckNtMartaPresent();
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = (NtMartaStatic.RewriteSetEntriesInAcl.unwrap())(
            cCountOfExplicitEntries,
            pListOfExplicitEntries as _,
            OldAcl as _,
            NewAcl as _,
        );
    }

    ErrorCode
}

unsafe fn InternalTrusteeAToW(pTrusteeA: PTRUSTEE_A, pTrusteeW: *mut PTRUSTEE_W) -> DWORD {
    let TrusteeForm: TRUSTEE_FORM;
    let mut BufferSize: INT = 0;
    let lpStr: PSTR;
    let mut ErrorCode: DWORD = ERROR_SUCCESS;

    TrusteeForm = GetTrusteeFormA(pTrusteeA);

    'convert_err: {
        match TrusteeForm {
            TRUSTEE_IS_NAME => {
                lpStr = GetTrusteeNameA(pTrusteeA);
                if lpStr != core::ptr::null_mut() {
                    BufferSize = (strlen(lpStr) + 1) as INT;
                }

                *pTrusteeW = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (core::mem::size_of::<TRUSTEE_W>()
                        + (BufferSize as usize * core::mem::size_of::<WCHAR>())) as SIZE_T,
                ) as PTRUSTEE_W;
                if *pTrusteeW != core::ptr::null_mut() {
                    RtlCopyMemory(
                        *pTrusteeW as *mut core::ffi::c_void,
                        pTrusteeA as *const core::ffi::c_void,
                        crate::FIELD_OFFSET!(TRUSTEE_A, ptstrName),
                    );

                    if lpStr != core::ptr::null_mut() {
                        (**pTrusteeW).ptstrName = (*pTrusteeW).add(1) as PWSTR;

                        if MultiByteToWideChar(
                            CP_ACP,
                            0,
                            lpStr,
                            -1,
                            (**pTrusteeW).ptstrName,
                            BufferSize,
                        ) == 0
                        {
                            break 'convert_err;
                        }
                    } else {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, *pTrusteeW as PVOID);
                        *pTrusteeW = pTrusteeA as PTRUSTEE_W;
                        return ErrorCode;
                    }
                } else {
                    ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                }
            }

            TRUSTEE_IS_OBJECTS_AND_NAME => {
                let oanA: POBJECTS_AND_NAME_A = GetTrusteeNameA(pTrusteeA) as POBJECTS_AND_NAME_A;
                let oan: POBJECTS_AND_NAME_W;
                let mut StrBuf: PWSTR;

                if ((*oanA).ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) != 0
                    && (*oanA).InheritedObjectTypeName != core::ptr::null_mut()
                {
                    BufferSize = (strlen((*oanA).InheritedObjectTypeName) + 1) as INT;
                }
                if (*oanA).ptstrName != core::ptr::null_mut() {
                    BufferSize += (strlen((*oanA).ptstrName) + 1) as INT;
                }

                *pTrusteeW = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (core::mem::size_of::<TRUSTEE_W>()
                        + core::mem::size_of::<OBJECTS_AND_NAME_W>()
                        + (BufferSize as usize * core::mem::size_of::<WCHAR>())) as SIZE_T,
                ) as PTRUSTEE_W;

                if *pTrusteeW != core::ptr::null_mut() {
                    oan = (*pTrusteeW).add(1) as POBJECTS_AND_NAME_W;
                    StrBuf = oan.add(1) as PWSTR;

                    RtlCopyMemory(
                        *pTrusteeW as *mut core::ffi::c_void,
                        pTrusteeA as *const core::ffi::c_void,
                        crate::FIELD_OFFSET!(TRUSTEE_A, ptstrName),
                    );

                    (**pTrusteeW).ptstrName = oan as LPWSTR;

                    (*oan).ObjectsPresent = (*oanA).ObjectsPresent;
                    (*oan).ObjectType = (*oanA).ObjectType;

                    if ((*oanA).ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) != 0
                        && (*oanA).InheritedObjectTypeName != core::ptr::null_mut()
                    {
                        BufferSize = (strlen((*oanA).InheritedObjectTypeName) + 1) as INT;

                        if MultiByteToWideChar(
                            CP_ACP,
                            0,
                            (*oanA).InheritedObjectTypeName,
                            -1,
                            StrBuf,
                            BufferSize,
                        ) == 0
                        {
                            break 'convert_err;
                        }
                        (*oan).InheritedObjectTypeName = StrBuf;

                        StrBuf = StrBuf.add(BufferSize as usize);
                    } else {
                        (*oan).InheritedObjectTypeName = core::ptr::null_mut();
                    }

                    if (*oanA).ptstrName != core::ptr::null_mut() {
                        BufferSize = (strlen((*oanA).ptstrName) + 1) as INT;

                        if MultiByteToWideChar(
                            CP_ACP,
                            0,
                            (*oanA).ptstrName,
                            -1,
                            StrBuf,
                            BufferSize,
                        ) == 0
                        {
                            break 'convert_err;
                        }
                        (*oan).ptstrName = StrBuf;
                    } else {
                        (*oan).ptstrName = core::ptr::null_mut();
                    }
                } else {
                    ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                }
            }

            _ => {
                *pTrusteeW = pTrusteeA as PTRUSTEE_W;
            }
        }

        return ErrorCode;
    }

    ErrorCode = GetLastError();

    RtlFreeHeap(RtlGetProcessHeap(), 0, *pTrusteeW as PVOID);

    ErrorCode
}

unsafe fn InternalFreeConvertedTrustee(pTrusteeW: PTRUSTEE_W, pTrusteeA: PTRUSTEE_A) {
    if pTrusteeW as PVOID != pTrusteeA as PVOID {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pTrusteeW as PVOID);
    }
}

unsafe fn InternalExplicitAccessAToW(
    cCountOfExplicitEntries: ULONG,
    pListOfExplicitEntriesA: PEXPLICIT_ACCESS_A,
    pListOfExplicitEntriesW: *mut PEXPLICIT_ACCESS_W,
) -> DWORD {
    let mut TrusteeForm: TRUSTEE_FORM;
    let mut Size: SIZE_T;
    let mut i: ULONG;
    let mut ObjectsAndNameCount: ULONG = 0;
    let mut peaw: PEXPLICIT_ACCESS_W = core::ptr::null_mut();
    let mut ErrorCode: DWORD = ERROR_SUCCESS;
    let mut lpStr: LPSTR;

    'convert_err: {
        if cCountOfExplicitEntries != 0 {
            Size = cCountOfExplicitEntries as SIZE_T
                * core::mem::size_of::<EXPLICIT_ACCESS_W>() as SIZE_T;
            i = 0;
            while i != cCountOfExplicitEntries {
                TrusteeForm = GetTrusteeFormA(core::ptr::addr_of_mut!(
                    (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                ));

                match TrusteeForm {
                    TRUSTEE_IS_NAME => {
                        lpStr = GetTrusteeNameA(core::ptr::addr_of_mut!(
                            (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                        ));
                        if !lpStr.is_null() {
                            Size += (strlen(lpStr) + 1) as SIZE_T
                                * core::mem::size_of::<WCHAR>() as SIZE_T;
                        }
                    }

                    TRUSTEE_IS_OBJECTS_AND_NAME => {
                        let oan = GetTrusteeNameA(core::ptr::addr_of_mut!(
                            (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                        )) as POBJECTS_AND_NAME_A;

                        if ((*oan).ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) != 0
                            && !(*oan).InheritedObjectTypeName.is_null()
                        {
                            Size += (strlen((*oan).InheritedObjectTypeName) + 1) as SIZE_T
                                * core::mem::size_of::<WCHAR>() as SIZE_T;
                        }

                        if !(*oan).ptstrName.is_null() {
                            Size += (strlen((*oan).ptstrName) + 1) as SIZE_T
                                * core::mem::size_of::<WCHAR>() as SIZE_T;
                        }

                        ObjectsAndNameCount += 1;
                    }

                    _ => {}
                }
                i += 1;
            }

            peaw = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size) as PEXPLICIT_ACCESS_W;
            if !peaw.is_null() {
                let mut BufferSize: INT;
                let mut oan = peaw.add(cCountOfExplicitEntries as usize) as POBJECTS_AND_NAME_W;
                let mut StrBuf = oan.add(ObjectsAndNameCount as usize) as LPWSTR;

                i = 0;
                while i != cCountOfExplicitEntries {
                    (*peaw.add(i as usize)).grfAccessPermissions =
                        (*pListOfExplicitEntriesA.add(i as usize)).grfAccessPermissions;
                    (*peaw.add(i as usize)).grfAccessMode =
                        (*pListOfExplicitEntriesA.add(i as usize)).grfAccessMode;
                    (*peaw.add(i as usize)).grfInheritance =
                        (*pListOfExplicitEntriesA.add(i as usize)).grfInheritance;

                    TrusteeForm = GetTrusteeFormA(core::ptr::addr_of_mut!(
                        (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                    ));

                    let mut raw_copy = false;

                    match TrusteeForm {
                        TRUSTEE_IS_NAME => {
                            lpStr = GetTrusteeNameA(core::ptr::addr_of_mut!(
                                (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                            ));
                            if !lpStr.is_null() {
                                BufferSize = (strlen(lpStr) + 1) as INT;

                                if MultiByteToWideChar(
                                    CP_ACP, 0, lpStr, -1, StrBuf, BufferSize,
                                ) == 0
                                {
                                    break 'convert_err;
                                }
                                (*peaw.add(i as usize)).Trustee.ptstrName = StrBuf;

                                StrBuf = StrBuf.add(BufferSize as usize);
                            } else {
                                raw_copy = true;
                            }
                        }

                        TRUSTEE_IS_OBJECTS_AND_NAME => {
                            let oanA = GetTrusteeNameA(core::ptr::addr_of_mut!(
                                (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                            )) as POBJECTS_AND_NAME_A;

                            RtlCopyMemory(
                                core::ptr::addr_of_mut!((*peaw.add(i as usize)).Trustee)
                                    as *mut core::ffi::c_void,
                                core::ptr::addr_of!(
                                    (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                                ) as *const core::ffi::c_void,
                                FIELD_OFFSET!(TRUSTEE_A, ptstrName),
                            );

                            (*peaw.add(i as usize)).Trustee.ptstrName = oan as LPWSTR;

                            (*oan).ObjectsPresent = (*oanA).ObjectsPresent;
                            (*oan).ObjectType = (*oanA).ObjectType;

                            if ((*oanA).ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) != 0
                                && !(*oanA).InheritedObjectTypeName.is_null()
                            {
                                BufferSize =
                                    (strlen((*oanA).InheritedObjectTypeName) + 1) as INT;

                                if MultiByteToWideChar(
                                    CP_ACP,
                                    0,
                                    (*oanA).InheritedObjectTypeName,
                                    -1,
                                    StrBuf,
                                    BufferSize,
                                ) == 0
                                {
                                    break 'convert_err;
                                }
                                (*oan).InheritedObjectTypeName = StrBuf;

                                StrBuf = StrBuf.add(BufferSize as usize);
                            } else {
                                (*oan).InheritedObjectTypeName = core::ptr::null_mut();
                            }

                            if !(*oanA).ptstrName.is_null() {
                                BufferSize = (strlen((*oanA).ptstrName) + 1) as INT;

                                if MultiByteToWideChar(
                                    CP_ACP,
                                    0,
                                    (*oanA).ptstrName,
                                    -1,
                                    StrBuf,
                                    BufferSize,
                                ) == 0
                                {
                                    break 'convert_err;
                                }
                                (*oan).ptstrName = StrBuf;

                                StrBuf = StrBuf.add(BufferSize as usize);
                            } else {
                                (*oan).ptstrName = core::ptr::null_mut();
                            }

                            oan = oan.add(1);
                        }

                        _ => {
                            raw_copy = true;
                        }
                    }

                    if raw_copy {
                        RtlCopyMemory(
                            core::ptr::addr_of_mut!((*peaw.add(i as usize)).Trustee)
                                as *mut core::ffi::c_void,
                            core::ptr::addr_of!(
                                (*pListOfExplicitEntriesA.add(i as usize)).Trustee
                            ) as *const core::ffi::c_void,
                            core::mem::size_of::<TRUSTEE_A>(),
                        );
                    }

                    i += 1;
                }

                *pListOfExplicitEntriesW = peaw;
            } else {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            }
        }

        return ErrorCode;
    }

    ErrorCode = GetLastError();

    RtlFreeHeap(RtlGetProcessHeap(), 0, peaw as *mut core::ffi::c_void);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn SetEntriesInAclA(
    cCountOfExplicitEntries: ULONG,
    pListOfExplicitEntries: PEXPLICIT_ACCESS_A,
    OldAcl: PACL,
    NewAcl: *mut PACL,
) -> DWORD {
    let mut ListOfExplicitEntriesW: PEXPLICIT_ACCESS_W = core::ptr::null_mut();
    let mut ErrorCode: DWORD;

    ErrorCode = InternalExplicitAccessAToW(
        cCountOfExplicitEntries,
        pListOfExplicitEntries,
        &mut ListOfExplicitEntriesW,
    );
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = SetEntriesInAclW(
            cCountOfExplicitEntries,
            ListOfExplicitEntriesW,
            OldAcl,
            NewAcl,
        );

        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            ListOfExplicitEntriesW as PVOID,
        );
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetExplicitEntriesFromAclW(
    pacl: PACL,
    pcCountOfExplicitEntries: *mut u32,
    pListOfExplicitEntries: *mut *mut core::ffi::c_void,
) -> u32 {

    let mut ErrorCode: u32 = CheckNtMartaPresent();
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = (NtMartaStatic.RewriteGetExplicitEntriesFromAcl.unwrap())(
            pacl as _,
            pcCountOfExplicitEntries as _,
            pListOfExplicitEntries as _,
        );
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetEffectiveRightsFromAclW(
    pacl: PACL,
    pTrustee: PTRUSTEE_W,
    pAccessRights: PACCESS_MASK,
) -> DWORD {
    *pAccessRights = (STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL) as ACCESS_MASK;
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GetEffectiveRightsFromAclA(
    pacl: PACL,
    pTrustee: PTRUSTEE_A,
    pAccessRights: PACCESS_MASK,
) -> DWORD {
    let mut pTrusteeW: PTRUSTEE_W = core::ptr::null_mut();
    let mut ErrorCode: DWORD;

    ErrorCode = InternalTrusteeAToW(pTrustee, &mut pTrusteeW);
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = GetEffectiveRightsFromAclW(pacl, pTrusteeW, pAccessRights);

        InternalFreeConvertedTrustee(pTrusteeW, pTrustee);
    } else {
        ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetAuditedPermissionsFromAclW(
    pacl: PACL,
    pTrustee: PTRUSTEE_W,
    pSuccessfulAuditedRights: PACCESS_MASK,
    pFailedAuditRights: PACCESS_MASK,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn GetAuditedPermissionsFromAclA(
    pacl: PACL,
    pTrustee: PTRUSTEE_A,
    pSuccessfulAuditedRights: PACCESS_MASK,
    pFailedAuditRights: PACCESS_MASK,
) -> DWORD {
    let mut pTrusteeW: PTRUSTEE_W = core::ptr::null_mut();
    let mut ErrorCode: DWORD;

    ErrorCode = InternalTrusteeAToW(pTrustee, &mut pTrusteeW);
    if ErrorCode == ERROR_SUCCESS {
        ErrorCode = GetAuditedPermissionsFromAclW(
            pacl,
            pTrusteeW,
            pSuccessfulAuditedRights,
            pFailedAuditRights,
        );

        InternalFreeConvertedTrustee(pTrusteeW, pTrustee);
    } else {
        ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
    }

    ErrorCode
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/sec/ac.c
//  * PURPOSE:         ACL/ACE functions
//  */
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /* --- ACE --- */
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AddAccessAllowedObjectAce(PACL pAcl,
//                           DWORD dwAceRevision,
//                           DWORD AceFlags,
//                           DWORD AccessMask,
//                           GUID *ObjectTypeGuid,
//                           GUID *InheritedObjectTypeGuid,
//                           PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessAllowedObjectAce(pAcl,
//                                           dwAceRevision,
//                                           AceFlags,
//                                           AccessMask,
//                                           ObjectTypeGuid,
//                                           InheritedObjectTypeGuid,
//                                           pSid);
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
// AddAccessDeniedObjectAce(PACL pAcl,
//                          DWORD dwAceRevision,
//                          DWORD AceFlags,
//                          DWORD AccessMask,
//                          GUID* ObjectTypeGuid,
//                          GUID* InheritedObjectTypeGuid,
//                          PSID pSid)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAccessDeniedObjectAce(pAcl,
//                                          dwAceRevision,
//                                          AceFlags,
//                                          AccessMask,
//                                          ObjectTypeGuid,
//                                          InheritedObjectTypeGuid,
//                                          pSid);
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
// AddAuditAccessObjectAce(PACL pAcl,
//                         DWORD dwAceRevision,
//                         DWORD AceFlags,
//                         DWORD AccessMask,
//                         GUID *ObjectTypeGuid,
//                         GUID *InheritedObjectTypeGuid,
//                         PSID pSid,
//                         BOOL bAuditSuccess,
//                         BOOL bAuditFailure)
// {
//     NTSTATUS Status;
//
//     Status = RtlAddAuditAccessObjectAce(pAcl,
//                                         dwAceRevision,
//                                         AceFlags,
//                                         AccessMask,
//                                         ObjectTypeGuid,
//                                         InheritedObjectTypeGuid,
//                                         pSid,
//                                         bAuditSuccess,
//                                         bAuditFailure);
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
// DWORD
// WINAPI
// GetInheritanceSourceW(LPWSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       BOOL Container,
//                       GUID **pObjectClassGuids  OPTIONAL,
//                       DWORD GuidCount,
//                       PACL pAcl,
//                       PFN_OBJECT_MGR_FUNCTS pfnArray  OPTIONAL,
//                       PGENERIC_MAPPING pGenericMapping,
//                       PINHERITED_FROMW pInheritArray)
// {
//     DWORD ErrorCode;
//
//     ErrorCode = CheckNtMartaPresent();
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* call the MARTA provider */
//         ErrorCode = (NtMartaStatic.GetInheritanceSourceW.unwrap())(pObjectName,
//                                             ObjectType,
//                                             SecurityInfo,
//                                             Container,
//                                             pObjectClassGuids,
//                                             GuidCount,
//                                             pAcl,
//                                             pfnArray,
//                                             pGenericMapping,
//                                             pInheritArray);
//     }
//
//     return ErrorCode;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// GetInheritanceSourceA(LPSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       BOOL Container,
//                       GUID **pObjectClassGuids  OPTIONAL,
//                       DWORD GuidCount,
//                       PACL pAcl,
//                       PFN_OBJECT_MGR_FUNCTS pfnArray  OPTIONAL,
//                       PGENERIC_MAPPING pGenericMapping,
//                       PINHERITED_FROMA pInheritArray)
// {
//     /* That's all this function does, at least up to w2k3... Even MS was too
//        lazy to implement it... */
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// FreeInheritedFromArray(PINHERITED_FROMW pInheritArray,
//                        USHORT AceCnt,
//                        PFN_OBJECT_MGR_FUNCTS pfnArray  OPTIONAL)
// {
//     DWORD ErrorCode;
//
//     ErrorCode = CheckNtMartaPresent();
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* call the MARTA provider */
//         ErrorCode = AccFreeIndexArray(pInheritArray,
//                                       AceCnt,
//                                       pfnArray);
//     }
//
//     return ErrorCode;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetEntriesInAclW(ULONG cCountOfExplicitEntries,
//                  PEXPLICIT_ACCESS_W pListOfExplicitEntries,
//                  PACL OldAcl,
//                  PACL *NewAcl)
// {
//     DWORD ErrorCode;
//
//     if (!NewAcl)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     ErrorCode = CheckNtMartaPresent();
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* call the MARTA provider */
//         ErrorCode = (NtMartaStatic.RewriteSetEntriesInAcl.unwrap())(cCountOfExplicitEntries,
//                                               pListOfExplicitEntries,
//                                               OldAcl,
//                                               NewAcl);
//     }
//
//     return ErrorCode;
// }
//
//
// static DWORD
// InternalTrusteeAToW(IN PTRUSTEE_A pTrusteeA,
//                     OUT PTRUSTEE_W *pTrusteeW)
// {
//     TRUSTEE_FORM TrusteeForm;
//     INT BufferSize = 0;
//     PSTR lpStr;
//     DWORD ErrorCode = ERROR_SUCCESS;
//
//     //ASSERT(sizeof(TRUSTEE_W) == sizeof(TRUSTEE_A));
//
//     TrusteeForm = GetTrusteeFormA(pTrusteeA);
//     switch (TrusteeForm)
//     {
//         case TRUSTEE_IS_NAME:
//         {
//             /* directly copy the array, this works as the size of the EXPLICIT_ACCESS_A
//                structure matches the size of the EXPLICIT_ACCESS_W version */
//             lpStr = GetTrusteeNameA(pTrusteeA);
//             if (lpStr != NULL)
//                 BufferSize = strlen(lpStr) + 1;
//
//             *pTrusteeW = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          sizeof(TRUSTEE_W) + (BufferSize * sizeof(WCHAR)));
//             if (*pTrusteeW != NULL)
//             {
//                 RtlCopyMemory(*pTrusteeW,
//                               pTrusteeA,
//                               FIELD_OFFSET(TRUSTEE_A,
//                                            ptstrName));
//
//                 if (lpStr != NULL)
//                 {
//                     (*pTrusteeW)->ptstrName = (PWSTR)((*pTrusteeW) + 1);
//
//                     /* convert the trustee's name */
//                     if (MultiByteToWideChar(CP_ACP,
//                                             0,
//                                             lpStr,
//                                             -1,
//                                             (*pTrusteeW)->ptstrName,
//                                             BufferSize) == 0)
//                     {
//                         goto ConvertErr;
//                     }
//                 }
//                 else
//                 {
//                     RtlFreeHeap(RtlGetProcessHeap(),
//                                 0,
//                                 *pTrusteeW);
//                     goto NothingToConvert;
//                 }
//             }
//             else
//                 ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             break;
//         }
//
//         case TRUSTEE_IS_OBJECTS_AND_NAME:
//         {
//             POBJECTS_AND_NAME_A oanA = (POBJECTS_AND_NAME_A)GetTrusteeNameA(pTrusteeA);
//             POBJECTS_AND_NAME_W oan;
//             PWSTR StrBuf;
//
//             /* calculate the size needed */
//             if ((oanA->ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) &&
//                 oanA->InheritedObjectTypeName != NULL)
//             {
//                 BufferSize = strlen(oanA->InheritedObjectTypeName) + 1;
//             }
//             if (oanA->ptstrName != NULL)
//             {
//                 BufferSize += strlen(oanA->ptstrName) + 1;
//             }
//
//             *pTrusteeW = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          sizeof(TRUSTEE_W) + sizeof(OBJECTS_AND_NAME_W) +
//                                              (BufferSize * sizeof(WCHAR)));
//
//             if (*pTrusteeW != NULL)
//             {
//                 oan = (POBJECTS_AND_NAME_W)((*pTrusteeW) + 1);
//                 StrBuf = (PWSTR)(oan + 1);
//
//                 /* copy over the parts of the TRUSTEE structure that don't need
//                    to be touched */
//                 RtlCopyMemory(*pTrusteeW,
//                               pTrusteeA,
//                               FIELD_OFFSET(TRUSTEE_A,
//                                            ptstrName));
//
//                 (*pTrusteeW)->ptstrName = (LPWSTR)oan;
//
//                 /* convert the OBJECTS_AND_NAME_A structure */
//                 oan->ObjectsPresent = oanA->ObjectsPresent;
//                 oan->ObjectType = oanA->ObjectType;
//
//                 if ((oanA->ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) &&
//                     oanA->InheritedObjectTypeName != NULL)
//                 {
//                     /* convert inherited object type name */
//                     BufferSize = strlen(oanA->InheritedObjectTypeName) + 1;
//
//                     if (MultiByteToWideChar(CP_ACP,
//                                             0,
//                                             oanA->InheritedObjectTypeName,
//                                             -1,
//                                             StrBuf,
//                                             BufferSize) == 0)
//                     {
//                         goto ConvertErr;
//                     }
//                     oan->InheritedObjectTypeName = StrBuf;
//
//                     StrBuf += BufferSize;
//                 }
//                 else
//                     oan->InheritedObjectTypeName = NULL;
//
//                 if (oanA->ptstrName != NULL)
//                 {
//                     /* convert the trustee name */
//                     BufferSize = strlen(oanA->ptstrName) + 1;
//
//                     if (MultiByteToWideChar(CP_ACP,
//                                             0,
//                                             oanA->ptstrName,
//                                             -1,
//                                             StrBuf,
//                                             BufferSize) == 0)
//                     {
//                         goto ConvertErr;
//                     }
//                     oan->ptstrName = StrBuf;
//                 }
//                 else
//                     oan->ptstrName = NULL;
//             }
//             else
//                 ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             break;
//         }
//
//         default:
//         {
// NothingToConvert:
//             /* no need to convert anything to unicode */
//             *pTrusteeW = (PTRUSTEE_W)pTrusteeA;
//             break;
//         }
//     }
//
//     return ErrorCode;
//
// ConvertErr:
//     ErrorCode = GetLastError();
//
//     /* cleanup */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 *pTrusteeW);
//
//     return ErrorCode;
// }
//
//
// static __inline VOID
// InternalFreeConvertedTrustee(IN PTRUSTEE_W pTrusteeW,
//                              IN PTRUSTEE_A pTrusteeA)
// {
//     if ((PVOID)pTrusteeW != (PVOID)pTrusteeA)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(),
//                     0,
//                     pTrusteeW);
//     }
// }
//
//
// static DWORD
// InternalExplicitAccessAToW(IN ULONG cCountOfExplicitEntries,
//                            IN PEXPLICIT_ACCESS_A pListOfExplicitEntriesA,
//                            OUT PEXPLICIT_ACCESS_W *pListOfExplicitEntriesW)
// {
//     TRUSTEE_FORM TrusteeForm;
//     SIZE_T Size;
//     ULONG i;
//     ULONG ObjectsAndNameCount = 0;
//     PEXPLICIT_ACCESS_W peaw = NULL;
//     DWORD ErrorCode = ERROR_SUCCESS;
//     LPSTR lpStr;
//
//     /* NOTE: This code assumes that the size of the TRUSTEE_A and TRUSTEE_W structure matches! */
//     //ASSERT(sizeof(TRUSTEE_A) == sizeof(TRUSTEE_W));
//
//     if (cCountOfExplicitEntries != 0)
//     {
//         /* calculate the size needed */
//         Size = cCountOfExplicitEntries * sizeof(EXPLICIT_ACCESS_W);
//         for (i = 0; i != cCountOfExplicitEntries; i++)
//         {
//             TrusteeForm = GetTrusteeFormA(&pListOfExplicitEntriesA[i].Trustee);
//
//             switch (TrusteeForm)
//             {
//                 case TRUSTEE_IS_NAME:
//                 {
//                     lpStr = GetTrusteeNameA(&pListOfExplicitEntriesA[i].Trustee);
//                     if (lpStr != NULL)
//                         Size += (strlen(lpStr) + 1) * sizeof(WCHAR);
//                     break;
//                 }
//
//                 case TRUSTEE_IS_OBJECTS_AND_NAME:
//                 {
//                     POBJECTS_AND_NAME_A oan = (POBJECTS_AND_NAME_A)GetTrusteeNameA(&pListOfExplicitEntriesA[i].Trustee);
//
//                     if ((oan->ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) &&
//                         oan->InheritedObjectTypeName != NULL)
//                     {
//                         Size += (strlen(oan->InheritedObjectTypeName) + 1) * sizeof(WCHAR);
//                     }
//
//                     if (oan->ptstrName != NULL)
//                         Size += (strlen(oan->ptstrName) + 1) * sizeof(WCHAR);
//
//                     ObjectsAndNameCount++;
//                     break;
//                 }
//
//                 default:
//                     break;
//             }
//         }
//
//         /* allocate the array */
//         peaw = RtlAllocateHeap(RtlGetProcessHeap(),
//                                0,
//                                Size);
//         if (peaw != NULL)
//         {
//             INT BufferSize;
//             POBJECTS_AND_NAME_W oan = (POBJECTS_AND_NAME_W)(peaw + cCountOfExplicitEntries);
//             LPWSTR StrBuf = (LPWSTR)(oan + ObjectsAndNameCount);
//
//             /* convert the array to unicode */
//             for (i = 0; i != cCountOfExplicitEntries; i++)
//             {
//                 peaw[i].grfAccessPermissions = pListOfExplicitEntriesA[i].grfAccessPermissions;
//                 peaw[i].grfAccessMode = pListOfExplicitEntriesA[i].grfAccessMode;
//                 peaw[i].grfInheritance = pListOfExplicitEntriesA[i].grfInheritance;
//
//                 /* convert or copy the TRUSTEE structure */
//                 TrusteeForm = GetTrusteeFormA(&pListOfExplicitEntriesA[i].Trustee);
//                 switch (TrusteeForm)
//                 {
//                     case TRUSTEE_IS_NAME:
//                     {
//                         lpStr = GetTrusteeNameA(&pListOfExplicitEntriesA[i].Trustee);
//                         if (lpStr != NULL)
//                         {
//                             /* convert the trustee name */
//                             BufferSize = strlen(lpStr) + 1;
//
//                             if (MultiByteToWideChar(CP_ACP,
//                                                     0,
//                                                     lpStr,
//                                                     -1,
//                                                     StrBuf,
//                                                     BufferSize) == 0)
//                             {
//                                 goto ConvertErr;
//                             }
//                             peaw[i].Trustee.ptstrName = StrBuf;
//
//                             StrBuf += BufferSize;
//                         }
//                         else
//                             goto RawTrusteeCopy;
//
//                         break;
//                     }
//
//                     case TRUSTEE_IS_OBJECTS_AND_NAME:
//                     {
//                         POBJECTS_AND_NAME_A oanA = (POBJECTS_AND_NAME_A)GetTrusteeNameA(&pListOfExplicitEntriesA[i].Trustee);
//
//                         /* copy over the parts of the TRUSTEE structure that don't need
//                            to be touched */
//                         RtlCopyMemory(&peaw[i].Trustee,
//                                       &pListOfExplicitEntriesA[i].Trustee,
//                                       FIELD_OFFSET(TRUSTEE_A,
//                                                    ptstrName));
//
//                         peaw[i].Trustee.ptstrName = (LPWSTR)oan;
//
//                         /* convert the OBJECTS_AND_NAME_A structure */
//                         oan->ObjectsPresent = oanA->ObjectsPresent;
//                         oan->ObjectType = oanA->ObjectType;
//
//                         if ((oanA->ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT) &&
//                             oanA->InheritedObjectTypeName != NULL)
//                         {
//                             /* convert inherited object type name */
//                             BufferSize = strlen(oanA->InheritedObjectTypeName) + 1;
//
//                             if (MultiByteToWideChar(CP_ACP,
//                                                     0,
//                                                     oanA->InheritedObjectTypeName,
//                                                     -1,
//                                                     StrBuf,
//                                                     BufferSize) == 0)
//                             {
//                                 goto ConvertErr;
//                             }
//                             oan->InheritedObjectTypeName = StrBuf;
//
//                             StrBuf += BufferSize;
//                         }
//                         else
//                             oan->InheritedObjectTypeName = NULL;
//
//                         if (oanA->ptstrName != NULL)
//                         {
//                             /* convert the trustee name */
//                             BufferSize = strlen(oanA->ptstrName) + 1;
//
//                             if (MultiByteToWideChar(CP_ACP,
//                                                     0,
//                                                     oanA->ptstrName,
//                                                     -1,
//                                                     StrBuf,
//                                                     BufferSize) == 0)
//                             {
//                                 goto ConvertErr;
//                             }
//                             oan->ptstrName = StrBuf;
//
//                             StrBuf += BufferSize;
//                         }
//                         else
//                             oan->ptstrName = NULL;
//
//                         /* move on to the next OBJECTS_AND_NAME_A structure */
//                         oan++;
//                         break;
//                     }
//
//                     default:
//                     {
// RawTrusteeCopy:
//                         /* just copy over the TRUSTEE structure, they don't contain any
//                            ansi/unicode specific data */
//                         RtlCopyMemory(&peaw[i].Trustee,
//                                       &pListOfExplicitEntriesA[i].Trustee,
//                                       sizeof(TRUSTEE_A));
//                         break;
//                     }
//                 }
//             }
//
//             ASSERT(ErrorCode == ERROR_SUCCESS);
//             *pListOfExplicitEntriesW = peaw;
//         }
//         else
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     return ErrorCode;
//
// ConvertErr:
//     ErrorCode = GetLastError();
//
//     /* cleanup */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 peaw);
//
//     return ErrorCode;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetEntriesInAclA(ULONG cCountOfExplicitEntries,
//                  PEXPLICIT_ACCESS_A pListOfExplicitEntries,
//                  PACL OldAcl,
//                  PACL *NewAcl)
// {
//     PEXPLICIT_ACCESS_W ListOfExplicitEntriesW = NULL;
//     DWORD ErrorCode;
//
//     ErrorCode = InternalExplicitAccessAToW(cCountOfExplicitEntries,
//                                            pListOfExplicitEntries,
//                                            &ListOfExplicitEntriesW);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         ErrorCode = SetEntriesInAclW(cCountOfExplicitEntries,
//                                      ListOfExplicitEntriesW,
//                                      OldAcl,
//                                      NewAcl);
//
//         /* free the allocated array */
//         RtlFreeHeap(RtlGetProcessHeap(),
//                     0,
//                     ListOfExplicitEntriesW);
//     }
//
//     return ErrorCode;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetExplicitEntriesFromAclW(PACL pacl,
//                            PULONG pcCountOfExplicitEntries,
//                            PEXPLICIT_ACCESS_W *pListOfExplicitEntries)
// {
//     DWORD ErrorCode;
//
//     ErrorCode = CheckNtMartaPresent();
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* call the MARTA provider */
//         ErrorCode = (NtMartaStatic.RewriteGetExplicitEntriesFromAcl.unwrap())(pacl,
//                                                         pcCountOfExplicitEntries,
//                                                         pListOfExplicitEntries);
//     }
//
//     return ErrorCode;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// GetEffectiveRightsFromAclW(IN PACL pacl,
//                            IN PTRUSTEE_W pTrustee,
//                            OUT PACCESS_MASK pAccessRights)
// {
//     FIXME("%p %p %p - stub\n", pacl, pTrustee, pAccessRights);
//
//     *pAccessRights = STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL;
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetEffectiveRightsFromAclA(IN PACL pacl,
//                            IN PTRUSTEE_A pTrustee,
//                            OUT PACCESS_MASK pAccessRights)
// {
//     PTRUSTEE_W pTrusteeW = NULL;
//     DWORD ErrorCode;
//
//     ErrorCode = InternalTrusteeAToW(pTrustee,
//                                     &pTrusteeW);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         ErrorCode = GetEffectiveRightsFromAclW(pacl,
//                                                pTrusteeW,
//                                                pAccessRights);
//
//         InternalFreeConvertedTrustee(pTrusteeW,
//                                      pTrustee);
//     }
//     else
//         ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//
//     return ErrorCode;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// GetAuditedPermissionsFromAclW(IN PACL pacl,
//                               IN PTRUSTEE_W pTrustee,
//                               OUT PACCESS_MASK pSuccessfulAuditedRights,
//                               OUT PACCESS_MASK pFailedAuditRights)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetAuditedPermissionsFromAclA(IN PACL pacl,
//                               IN PTRUSTEE_A pTrustee,
//                               OUT PACCESS_MASK pSuccessfulAuditedRights,
//                               OUT PACCESS_MASK pFailedAuditRights)
// {
//     PTRUSTEE_W pTrusteeW = NULL;
//     DWORD ErrorCode;
//
//     ErrorCode = InternalTrusteeAToW(pTrustee,
//                                     &pTrusteeW);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         ErrorCode = GetAuditedPermissionsFromAclW(pacl,
//                                                   pTrusteeW,
//                                                   pSuccessfulAuditedRights,
//                                                   pFailedAuditRights);
//
//         InternalFreeConvertedTrustee(pTrusteeW,
//                                      pTrustee);
//     }
//     else
//         ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//
//     return ErrorCode;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
