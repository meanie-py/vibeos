use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// ntmarta.h:20 carries `extern HINSTANCE hDllInstance;`, so the island emits that
// declaration and anything reaching it through the prelude glob binds to the C symbol.
// This is the one definition — it needs the C name or the declaration resolves to
// nothing at link time (the netcfgx_hInstance lesson, same shape).
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn SetSecurityAccessMask(
    SecurityInformation: SECURITY_INFORMATION,
    DesiredAccess: LPDWORD,
) {
    *DesiredAccess = 0;

    if (SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION)) != 0 {
        *DesiredAccess |= WRITE_OWNER;
    }

    if (SecurityInformation & DACL_SECURITY_INFORMATION) != 0 {
        *DesiredAccess |= WRITE_DAC;
    }

    if (SecurityInformation & SACL_SECURITY_INFORMATION) != 0 {
        *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn QuerySecurityAccessMask(
    SecurityInformation: SECURITY_INFORMATION,
    DesiredAccess: LPDWORD,
) {
    *DesiredAccess = 0;

    if (SecurityInformation & (OWNER_SECURITY_INFORMATION |
                               GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION)) != 0
    {
        *DesiredAccess |= READ_CONTROL;
    }

    if (SecurityInformation & SACL_SECURITY_INFORMATION) != 0 {
        *DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }
}
unsafe extern "C" fn AccpGetAceAccessMode(AceHeader: PACE_HEADER) -> ACCESS_MODE {
    let mut Mode = NOT_USED_ACCESS;

    match (*AceHeader).AceType as u32 {
        ACCESS_ALLOWED_ACE_TYPE |
        ACCESS_ALLOWED_CALLBACK_ACE_TYPE |
        ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE |
        ACCESS_ALLOWED_OBJECT_ACE_TYPE => {
            Mode = GRANT_ACCESS;
        }

        ACCESS_DENIED_ACE_TYPE |
        ACCESS_DENIED_CALLBACK_ACE_TYPE |
        ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE |
        ACCESS_DENIED_OBJECT_ACE_TYPE => {
            Mode = DENY_ACCESS;
        }

        SYSTEM_AUDIT_ACE_TYPE |
        SYSTEM_AUDIT_CALLBACK_ACE_TYPE |
        SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE |
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            if (*AceHeader).AceFlags as u32 & FAILED_ACCESS_ACE_FLAG != 0 {
                Mode = SET_AUDIT_FAILURE;
            } else if (*AceHeader).AceFlags as u32 & SUCCESSFUL_ACCESS_ACE_FLAG != 0 {
                Mode = SET_AUDIT_SUCCESS;
            }
        }

        _ => {}
    }

    Mode
}
unsafe extern "C" fn AccpGetAceStructureSize(AceHeader: PACE_HEADER) -> u32 {
    let mut Size: u32 = 0;

    match (*AceHeader).AceType as u32 {
        ACCESS_ALLOWED_ACE_TYPE | ACCESS_DENIED_ACE_TYPE => {
            Size = FIELD_OFFSET!(ACCESS_ALLOWED_ACE, SidStart) as DWORD;
        }
        ACCESS_ALLOWED_CALLBACK_ACE_TYPE | ACCESS_DENIED_CALLBACK_ACE_TYPE => {
            Size = FIELD_OFFSET!(ACCESS_ALLOWED_CALLBACK_ACE, SidStart) as DWORD;
        }
        ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE | ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_CALLBACK_OBJECT_ACE;
            Size = FIELD_OFFSET!(ACCESS_ALLOWED_CALLBACK_OBJECT_ACE, ObjectType) as DWORD;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).ObjectType) as u32;
            }
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).InheritedObjectType) as u32;
            }
        }
        ACCESS_ALLOWED_OBJECT_ACE_TYPE | ACCESS_DENIED_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_OBJECT_ACE;
            Size = FIELD_OFFSET!(ACCESS_ALLOWED_OBJECT_ACE, ObjectType) as DWORD;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).ObjectType) as u32;
            }
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).InheritedObjectType) as u32;
            }
        }
        SYSTEM_AUDIT_ACE_TYPE => {
            Size = FIELD_OFFSET!(SYSTEM_AUDIT_ACE, SidStart) as DWORD;
        }
        SYSTEM_AUDIT_CALLBACK_ACE_TYPE => {
            Size = FIELD_OFFSET!(SYSTEM_AUDIT_CALLBACK_ACE, SidStart) as DWORD;
        }
        SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE;
            Size = FIELD_OFFSET!(SYSTEM_AUDIT_CALLBACK_OBJECT_ACE, ObjectType) as DWORD;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).ObjectType) as u32;
            }
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).InheritedObjectType) as u32;
            }
        }
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_OBJECT_ACE;
            Size = FIELD_OFFSET!(SYSTEM_AUDIT_OBJECT_ACE, ObjectType) as DWORD;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).ObjectType) as u32;
            }
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                Size += core::mem::size_of_val(&(*Ace).InheritedObjectType) as u32;
            }
        }
        SYSTEM_MANDATORY_LABEL_ACE_TYPE => {
            Size = FIELD_OFFSET!(SYSTEM_MANDATORY_LABEL_ACE, SidStart) as DWORD;
        }
        _ => {}
    }

    Size
}
unsafe extern "C" fn AccpGetAceSid(AceHeader: PACE_HEADER) -> PSID {
    return ((AceHeader as usize) + AccpGetAceStructureSize(AceHeader) as usize) as PSID;
}
unsafe extern "C" fn AccpGetAceAccessMask(AceHeader: PACE_HEADER) -> ACCESS_MASK {
    *((AceHeader.offset(1 as isize)) as PACCESS_MASK)
}
unsafe extern "C" fn AccpIsObjectAce(AceHeader: PACE_HEADER) -> u32 {
    let Ret: u32;

    match (*AceHeader).AceType as u32 {
        ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE |
        ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE |
        ACCESS_ALLOWED_OBJECT_ACE_TYPE |
        ACCESS_DENIED_OBJECT_ACE_TYPE |
        SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE |
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            Ret = TRUE as DWORD;
        }
        _ => {
            Ret = FALSE as DWORD;
        }
    }

    Ret
}
unsafe extern "C" fn AccpGetTrusteeObjects(
    Trustee: PTRUSTEE_W,
    pObjectTypeGuid: *mut GUID,
    pInheritedObjectTypeGuid: *mut GUID,
) -> DWORD {
    let Ret: DWORD;

    match (*Trustee).TrusteeForm {
        TRUSTEE_IS_OBJECTS_AND_NAME => {
            let pOan = (*Trustee).ptstrName as POBJECTS_AND_NAME_W;

            Ret = (*pOan).ObjectsPresent;
        }

        TRUSTEE_IS_OBJECTS_AND_SID => {
            let pOas = (*Trustee).ptstrName as POBJECTS_AND_SID;

            if !pObjectTypeGuid.is_null() && (*pOas).ObjectsPresent & ACE_OBJECT_TYPE_PRESENT != 0 {
                *pObjectTypeGuid = (*pOas).ObjectTypeGuid;
            }

            if !pInheritedObjectTypeGuid.is_null() && (*pOas).ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                *pInheritedObjectTypeGuid = (*pOas).InheritedObjectTypeGuid;
            }

            Ret = (*pOas).ObjectsPresent;
        }

        _ => {
            Ret = 0;
        }
    }

    Ret
}
unsafe extern "C" fn AccpCalcNeededAceSize(
    Sid: PSID,
    ObjectsPresent: DWORD,
) -> DWORD {
    let mut Ret: DWORD;

    Ret = core::mem::size_of::<ACE>() as DWORD + GetLengthSid(Sid);

    if ObjectsPresent & ACE_OBJECT_TYPE_PRESENT != 0 {
        Ret += core::mem::size_of::<GUID>() as DWORD;
    }
    if ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
        Ret += core::mem::size_of::<GUID>() as DWORD;
    }

    if ObjectsPresent != 0 {
        Ret += core::mem::size_of::<DWORD>() as DWORD;
    }

    return Ret;
}
unsafe extern "C" fn AccpGetObjectAceObjectType(AceHeader: PACE_HEADER) -> *mut GUID {
    let mut ObjectType: *mut GUID = core::ptr::null_mut();

    match (*AceHeader).AceType as u32 {
        ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE | ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_CALLBACK_OBJECT_ACE;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
            }
        }
        ACCESS_ALLOWED_OBJECT_ACE_TYPE | ACCESS_DENIED_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_OBJECT_ACE;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
            }
        }
        SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
            }
        }
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_OBJECT_ACE;
            if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
            }
        }
        _ => {}
    }

    ObjectType
}
unsafe extern "C" fn AccpGetObjectAceInheritedObjectType(AceHeader: PACE_HEADER) -> *mut GUID {
    let mut ObjectType: *mut GUID = core::ptr::null_mut();

    match (*AceHeader).AceType as u32 {
        ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE | ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_CALLBACK_OBJECT_ACE;
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).InheritedObjectType);
                } else {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
                }
            }
        }
        ACCESS_ALLOWED_OBJECT_ACE_TYPE | ACCESS_DENIED_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PACCESS_ALLOWED_OBJECT_ACE;
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).InheritedObjectType);
                } else {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
                }
            }
        }
        SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE;
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).InheritedObjectType);
                } else {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
                }
            }
        }
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            let Ace = AceHeader as PSYSTEM_AUDIT_OBJECT_ACE;
            if (*Ace).Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT != 0 {
                if (*Ace).Flags & ACE_OBJECT_TYPE_PRESENT != 0 {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).InheritedObjectType);
                } else {
                    ObjectType = core::ptr::addr_of_mut!((*Ace).ObjectType);
                }
            }
        }
        _ => {}
    }

    ObjectType
}
unsafe extern "C" fn AccpOpenLSAPolicyHandle(
    SystemName: LPWSTR,
    DesiredAccess: ACCESS_MASK,
    pPolicyHandle: PLSA_HANDLE,
) -> DWORD {
    let mut LsaObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LsaSystemName: LSA_UNICODE_STRING = core::mem::zeroed();
    let psn: PLSA_UNICODE_STRING;
    let SystemNameLength: usize;
    let Status: NTSTATUS;

    if !SystemName.is_null() && *SystemName.offset(0) != 0 {
        SystemNameLength = wcslen(SystemName);
        if SystemNameLength > UNICODE_STRING_MAX_CHARS as usize {
            return ERROR_INVALID_PARAMETER;
        }

        LsaSystemName.Buffer = SystemName;
        LsaSystemName.Length = (SystemNameLength * core::mem::size_of::<u16>()) as USHORT;
        LsaSystemName.MaximumLength = LsaSystemName.Length + core::mem::size_of::<u16>() as USHORT;
        psn = &mut LsaSystemName;
    } else {
        psn = core::ptr::null_mut();
    }

    Status = LsaOpenPolicy(
        psn,
        &mut LsaObjectAttributes,
        DesiredAccess,
        pPolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        return LsaNtStatusToWinError(Status);
    }

    return ERROR_SUCCESS;
}
unsafe extern "C" fn AccpGetTrusteeName(Trustee: PTRUSTEE_W) -> LPWSTR {
    match (*Trustee).TrusteeForm {
        TRUSTEE_IS_NAME => (*Trustee).ptstrName,
        TRUSTEE_IS_OBJECTS_AND_NAME => {
            let objects_and_name = (*Trustee).ptstrName as POBJECTS_AND_NAME_W;
            (*objects_and_name).ptstrName
        }
        _ => core::ptr::null_mut(),
    }
}
unsafe extern "C" fn AccpLookupCurrentUser(ppSid: *mut PSID) -> DWORD {
    let mut Buffer: [u8; core::mem::size_of::<TOKEN_USER>() + core::mem::size_of::<SID>() + core::mem::size_of::<DWORD>() * SID_MAX_SUB_AUTHORITIES as usize] = core::mem::zeroed();
    let mut Length: DWORD;
    let mut Token: HANDLE = core::ptr::null_mut();
    let pSid: PSID;
    let mut Ret: DWORD;

    *ppSid = core::ptr::null_mut();
    if OpenThreadToken(GetCurrentThread(), TOKEN_READ, TRUE, &mut Token) == 0 {
        Ret = GetLastError();
        if Ret != ERROR_NO_TOKEN {
            return Ret;
        }

        if OpenProcessToken(GetCurrentProcess(), TOKEN_READ, &mut Token) == 0 {
            return GetLastError();
        }
    }

    Length = core::mem::size_of_val(&Buffer) as DWORD;
    if GetTokenInformation(Token, TokenUser, Buffer.as_mut_ptr() as LPVOID, Length, &mut Length) == 0 {
        Ret = GetLastError();
        CloseHandle(Token);
        return Ret;
    }
    CloseHandle(Token);

    pSid = (*(Buffer.as_ptr() as PTOKEN_USER)).User.Sid;
    Length = GetLengthSid(pSid);
    *ppSid = LocalAlloc(LMEM_FIXED, (Length) as SIZE_T) as PSID;
    if (*ppSid).is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }
    CopyMemory!(*ppSid as LPVOID, pSid as LPVOID, Length as usize);

    return ERROR_SUCCESS;
}
unsafe extern "C" fn AccpLookupSidByName(
    PolicyHandle: LSA_HANDLE,
    Name: LPWSTR,
    pSid: *mut PSID,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut LsaNames: [LSA_UNICODE_STRING; 1] = core::mem::zeroed();
    let mut ReferencedDomains: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut TranslatedSid: PLSA_TRANSLATED_SID2 = core::ptr::null_mut();
    let SidLen: DWORD;
    let NameLength: SIZE_T;
    let mut Ret: DWORD = ERROR_SUCCESS;

    NameLength = wcslen(Name) as SIZE_T;
    if NameLength > UNICODE_STRING_MAX_CHARS as SIZE_T {
        return ERROR_INVALID_PARAMETER;
    }

    LsaNames[0].Buffer = Name;
    LsaNames[0].Length = (NameLength as USHORT) * core::mem::size_of::<WCHAR>() as USHORT;
    LsaNames[0].MaximumLength = LsaNames[0].Length + core::mem::size_of::<WCHAR>() as USHORT;

    Status = LsaLookupNames2(
        PolicyHandle,
        0,
        (core::mem::size_of_val(&LsaNames) / core::mem::size_of::<LSA_UNICODE_STRING>()) as ULONG,
        LsaNames.as_mut_ptr(),
        &mut ReferencedDomains,
        &mut TranslatedSid,
    );

    if !NT_SUCCESS(Status) {
        return LsaNtStatusToWinError(Status);
    }

    'cleanup: loop {
        if (*TranslatedSid).Use == SidTypeUnknown || (*TranslatedSid).Use == SidTypeInvalid {
            Ret = LsaNtStatusToWinError(STATUS_NONE_MAPPED);
            break 'cleanup;
        }

        SidLen = GetLengthSid((*TranslatedSid).Sid);

        *pSid = LocalAlloc(LMEM_FIXED, SidLen as SIZE_T);
        if !(*pSid).is_null() {
            if CopySid(SidLen, *pSid, (*TranslatedSid).Sid) == 0 {
                Ret = GetLastError();

                LocalFree(*pSid as HLOCAL);
                *pSid = core::ptr::null_mut();
            }
        } else {
            Ret = ERROR_NOT_ENOUGH_MEMORY;
        }

        break 'cleanup;
    }

    LsaFreeMemory(ReferencedDomains as PVOID);
    LsaFreeMemory(TranslatedSid as PVOID);

    Ret
}
unsafe extern "C" fn AccpGetTrusteeSid(
    Trustee: PTRUSTEE_W,
    pPolicyHandle: PLSA_HANDLE,
    ppSid: *mut PSID,
    Allocated: *mut BOOL,
) -> DWORD {
    let mut Ret: DWORD = ERROR_SUCCESS;
    let mut TrusteeName: LPWSTR;

    *ppSid = core::ptr::null_mut();
    *Allocated = FALSE;

    match (*Trustee).TrusteeForm {
        TRUSTEE_IS_OBJECTS_AND_NAME => {
            if (*((*Trustee).ptstrName as POBJECTS_AND_NAME_W)).ObjectsPresent != 0 {
                Ret = ERROR_INVALID_PARAMETER;
            } else {
                TrusteeName = AccpGetTrusteeName(Trustee);
                if wcscmp(TrusteeName, b"C\0U\0R\0R\0E\0N\0T\0_\0U\0S\0E\0R\0\0".as_ptr() as LPCWSTR) == 0 {
                    Ret = AccpLookupCurrentUser(ppSid);
                    if Ret == ERROR_SUCCESS {
                        *Allocated = TRUE;
                    }
                } else {
                    if *pPolicyHandle == core::ptr::null_mut() {
                        Ret = AccpOpenLSAPolicyHandle(
                            core::ptr::null_mut(),
                            POLICY_LOOKUP_NAMES,
                            pPolicyHandle,
                        );
                        if Ret != ERROR_SUCCESS {
                            return Ret;
                        }
                    }

                    Ret = AccpLookupSidByName(*pPolicyHandle, TrusteeName, ppSid);
                    if Ret == ERROR_SUCCESS {
                        *Allocated = TRUE;
                    }
                }
            }
        }

        TRUSTEE_IS_NAME => {
            TrusteeName = AccpGetTrusteeName(Trustee);
            if wcscmp(TrusteeName, b"C\0U\0R\0R\0E\0N\0T\0_\0U\0S\0E\0R\0\0".as_ptr() as LPCWSTR) == 0 {
                Ret = AccpLookupCurrentUser(ppSid);
                if Ret == ERROR_SUCCESS {
                    *Allocated = TRUE;
                }
            } else {
                if *pPolicyHandle == core::ptr::null_mut() {
                    Ret = AccpOpenLSAPolicyHandle(
                        core::ptr::null_mut(),
                        POLICY_LOOKUP_NAMES,
                        pPolicyHandle,
                    );
                    if Ret != ERROR_SUCCESS {
                        return Ret;
                    }
                }

                Ret = AccpLookupSidByName(*pPolicyHandle, TrusteeName, ppSid);
                if Ret == ERROR_SUCCESS {
                    *Allocated = TRUE;
                }
            }
        }

        TRUSTEE_IS_OBJECTS_AND_SID => {
            *ppSid = (*((*Trustee).ptstrName as POBJECTS_AND_SID)).pSid as PSID;
        }

        TRUSTEE_IS_SID => {
            *ppSid = (*Trustee).ptstrName as PSID;
        }

        _ => {
            Ret = ERROR_INVALID_PARAMETER;
        }
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteGetHandleRights(
    handle: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut pSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SDSize: ULONG = 0;
    let mut Status: NTSTATUS;
    let LastErr: DWORD;
    let mut Ret: DWORD;

    LastErr = GetLastError();

    'main_loop: loop {
        Ret = ERROR_SUCCESS;

        SDSize += 0x100;
        if pSD.is_null() {
            pSD = LocalAlloc(LMEM_FIXED, SDSize as SIZE_T);
        } else {
            let newSD: PSECURITY_DESCRIPTOR;

            newSD = LocalReAlloc(pSD as HLOCAL, SDSize as SIZE_T, LMEM_MOVEABLE);
            if !newSD.is_null() {
                pSD = newSD;
            }
        }

        if pSD.is_null() {
            Ret = GetLastError();
            break;
        }

        match ObjectType {
            SE_REGISTRY_KEY => {
                Ret = RegGetKeySecurity(
                    handle as HKEY,
                    SecurityInfo,
                    pSD,
                    core::ptr::addr_of_mut!(SDSize),
                ) as DWORD;
            }

            SE_FILE_OBJECT | SE_KERNEL_OBJECT => {
                Status = NtQuerySecurityObject(
                    handle,
                    SecurityInfo,
                    pSD,
                    SDSize,
                    core::ptr::addr_of_mut!(SDSize),
                );
                if !NT_SUCCESS(Status) {
                    Ret = RtlNtStatusToDosError(Status);
                }
            }

            SE_SERVICE => {
                if QueryServiceObjectSecurity(
                    handle as SC_HANDLE,
                    SecurityInfo,
                    pSD,
                    SDSize,
                    core::ptr::addr_of_mut!(SDSize),
                ) == 0
                {
                    Ret = GetLastError();
                }
            }

            SE_WINDOW_OBJECT => {
                if GetUserObjectSecurity(
                    handle,
                    core::ptr::addr_of_mut!(SecurityInfo),
                    pSD,
                    SDSize,
                    core::ptr::addr_of_mut!(SDSize),
                ) == 0
                {
                    Ret = GetLastError();
                }
            }

            _ => {
                UNIMPLEMENTED!();
                Ret = ERROR_CALL_NOT_IMPLEMENTED;
            }
        }

        if Ret != ERROR_INSUFFICIENT_BUFFER {
            break 'main_loop;
        }
    }

    if Ret == ERROR_SUCCESS {
        let mut Present: BOOL = 0;
        let mut Defaulted: BOOL = 0;

        'cleanup: {
            if (SecurityInfo & OWNER_SECURITY_INFORMATION) != 0 && !ppsidOwner.is_null() {
                *ppsidOwner = core::ptr::null_mut();
                if GetSecurityDescriptorOwner(pSD, ppsidOwner, core::ptr::addr_of_mut!(Defaulted)) == 0 {
                    Ret = GetLastError();
                    break 'cleanup;
                }
            }

            if (SecurityInfo & GROUP_SECURITY_INFORMATION) != 0 && !ppsidGroup.is_null() {
                *ppsidGroup = core::ptr::null_mut();
                if GetSecurityDescriptorGroup(pSD, ppsidGroup, core::ptr::addr_of_mut!(Defaulted)) == 0 {
                    Ret = GetLastError();
                    break 'cleanup;
                }
            }

            if (SecurityInfo & DACL_SECURITY_INFORMATION) != 0 && !ppDacl.is_null() {
                *ppDacl = core::ptr::null_mut();
                if GetSecurityDescriptorDacl(
                    pSD,
                    core::ptr::addr_of_mut!(Present),
                    ppDacl,
                    core::ptr::addr_of_mut!(Defaulted),
                ) == 0
                {
                    Ret = GetLastError();
                    break 'cleanup;
                }
            }

            if (SecurityInfo & SACL_SECURITY_INFORMATION) != 0 && !ppSacl.is_null() {
                *ppSacl = core::ptr::null_mut();
                if GetSecurityDescriptorSacl(
                    pSD,
                    core::ptr::addr_of_mut!(Present),
                    ppSacl,
                    core::ptr::addr_of_mut!(Defaulted),
                ) == 0
                {
                    Ret = GetLastError();
                    break 'cleanup;
                }
            }

            *ppSecurityDescriptor = pSD;
        }

        if Ret != ERROR_SUCCESS {
            if !pSD.is_null() {
                LocalFree(pSD as HLOCAL);
            }
        }
    } else {
        if !pSD.is_null() {
            LocalFree(pSD as HLOCAL);
        }
    }

    SetLastError(LastErr);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteSetHandleRights(
    handle: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut Status: NTSTATUS;
    let LastErr: DWORD;
    let mut Ret: DWORD = ERROR_SUCCESS;

    LastErr = GetLastError();

    match ObjectType {
        SE_REGISTRY_KEY => {
            Ret = RegSetKeySecurity(
                handle as HKEY,
                SecurityInfo,
                pSecurityDescriptor,
            ) as DWORD;
        }

        SE_FILE_OBJECT | SE_KERNEL_OBJECT => {
            Status = NtSetSecurityObject(
                handle,
                SecurityInfo,
                pSecurityDescriptor,
            );
            if !NT_SUCCESS(Status) {
                Ret = RtlNtStatusToDosError(Status);
            }
        }

        SE_SERVICE => {
            if SetServiceObjectSecurity(
                handle as SC_HANDLE,
                SecurityInfo,
                pSecurityDescriptor,
            ) == 0
            {
                Ret = GetLastError();
            }
        }

        SE_WINDOW_OBJECT => {
            if SetUserObjectSecurity(
                handle,
                core::ptr::addr_of_mut!(SecurityInfo),
                pSecurityDescriptor,
            ) == 0
            {
                Ret = GetLastError();
            }
        }

        _ => {
            UNIMPLEMENTED!();
            Ret = ERROR_CALL_NOT_IMPLEMENTED;
        }
    }

    SetLastError(LastErr);

    return Ret;
}
unsafe extern "C" fn AccpOpenNamedObject(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    Handle: PHANDLE,
    Handle2: PHANDLE,
    Write: BOOL,
) -> DWORD {
    let mut lpPath: LPWSTR;
    let mut Status: NTSTATUS;
    let mut DesiredAccess: ACCESS_MASK = 0 as ACCESS_MASK;
    let mut Ret: DWORD = ERROR_SUCCESS;

    match ObjectType {
        SE_REGISTRY_KEY
        | SE_FILE_OBJECT
        | SE_KERNEL_OBJECT
        | SE_SERVICE
        | SE_WINDOW_OBJECT => {
            if Write != 0 {
                SetSecurityAccessMask(SecurityInfo, &mut DesiredAccess as *mut ACCESS_MASK as PDWORD);
            } else {
                QuerySecurityAccessMask(SecurityInfo, &mut DesiredAccess as *mut ACCESS_MASK as PDWORD);
            }
        }
        _ => {}
    }

    match ObjectType {
        SE_REGISTRY_KEY | SE_SERVICE => {
            lpPath = LocalAlloc(
                LMEM_FIXED,((wcslen(pObjectName) + 1)
                    * core::mem::size_of::<u16>()) as SIZE_T,
            ) as LPWSTR;
            if lpPath.is_null() {
                Ret = GetLastError();
                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }

            wcscpy(lpPath, pObjectName);
        }
        _ => {
            lpPath = pObjectName;
        }
    }

    match ObjectType {
        SE_FILE_OBJECT => {
            let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut FileName: _UNICODE_STRING = core::mem::zeroed();

            if RtlDosPathNameToNtPathName_U(
                pObjectName,
                &mut FileName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
            {
                Ret = ERROR_INVALID_NAME;
                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }

            InitializeObjectAttributes!(
                core::ptr::addr_of_mut!(ObjectAttributes),
                core::ptr::addr_of_mut!(FileName),
                OBJ_CASE_INSENSITIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            Status = NtOpenFile(
                Handle,
                DesiredAccess | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_SYNCHRONOUS_IO_NONALERT,
            );

            RtlFreeHeap(RtlGetProcessHeap!(), 0, FileName.Buffer as LPVOID);

            if !NT_SUCCESS(Status) {
                Ret = RtlNtStatusToDosError(Status);
            }
        }

        SE_REGISTRY_KEY => {
            #[repr(C)]
            struct AccRegRootKey {
                hRootKey: HKEY,
                szRootKey: LPCWSTR,
            }

            let AccRegRootKeys: [AccRegRootKey; 5] = [
                AccRegRootKey {
                    hRootKey: HKEY_CLASSES_ROOT,
                    szRootKey: b"C\0L\0A\0S\0S\0E\0S\0_\0R\0O\0O\0T\0\0\0".as_ptr() as LPCWSTR,
                },
                AccRegRootKey {
                    hRootKey: HKEY_CURRENT_USER,
                    szRootKey: b"C\0U\0R\0R\0E\0N\0T\0_\0U\0S\0E\0R\0\0\0".as_ptr() as LPCWSTR,
                },
                AccRegRootKey {
                    hRootKey: HKEY_LOCAL_MACHINE,
                    szRootKey: b"M\0A\0C\0H\0I\0N\0E\0\0\0".as_ptr() as LPCWSTR,
                },
                AccRegRootKey {
                    hRootKey: HKEY_USERS,
                    szRootKey: b"U\0S\0E\0R\0S\0\0\0".as_ptr() as LPCWSTR,
                },
                AccRegRootKey {
                    hRootKey: HKEY_CURRENT_CONFIG,
                    szRootKey: b"C\0O\0N\0F\0I\0G\0\0\0".as_ptr() as LPCWSTR,
                },
            ];

            let mut lpMachineName: LPWSTR;
            let mut lpRootKeyName: LPWSTR;
            let mut lpKeyName: LPWSTR;
            let mut hRootKey: HKEY = core::ptr::null_mut();
            let mut i: u32;

            if *lpPath.offset(0 as isize) == 0x5C
                && *lpPath.offset(1 as isize) == 0x5C
            {
                lpMachineName = lpPath;

                lpRootKeyName = wcschr(lpPath.offset(2 as isize), 0x5C);
                if lpRootKeyName.is_null() {
                    Ret = ERROR_INVALID_PARAMETER;
                    return 'Cleanup: {
                        if !lpPath.is_null() && lpPath != pObjectName {
                            LocalFree(lpPath as HLOCAL);
                        }
                        Ret
                    };
                } else {
                    *lpRootKeyName = 0;
                    lpRootKeyName = lpRootKeyName.offset(1);
                }
            } else {
                lpMachineName = core::ptr::null_mut();
                lpRootKeyName = lpPath;
            }

            lpKeyName = wcschr(lpRootKeyName, 0x5C);
            if !lpKeyName.is_null() {
                *lpKeyName = 0;
                lpKeyName = lpKeyName.offset(1);
            }

            i = 0;
            while i != 5 {
                if _wcsicmp(lpRootKeyName, AccRegRootKeys[i as usize].szRootKey) == 0 {
                    hRootKey = AccRegRootKeys[i as usize].hRootKey;
                    break;
                }
                i += 1;
            }

            if hRootKey.is_null() {
                Ret = ERROR_INVALID_PARAMETER;
                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }

            if !lpMachineName.is_null() {
                Ret = RegConnectRegistry(lpMachineName, hRootKey, Handle2 as PHKEY) as DWORD;

                if Ret != ERROR_SUCCESS {
                    return 'Cleanup: {
                        if !lpPath.is_null() && lpPath != pObjectName {
                            LocalFree(lpPath as HLOCAL);
                        }
                        Ret
                    };
                }

                hRootKey = *Handle2 as HKEY;
            }

            Ret = RegOpenKeyEx(
                hRootKey,
                lpKeyName,
                0,
                DesiredAccess as REGSAM,
                Handle as PHKEY,
            ) as DWORD;
            if Ret != ERROR_SUCCESS {
                if !(*Handle2).is_null() {
                    RegCloseKey(*Handle2 as HKEY);
                }

                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }
        }

        SE_SERVICE => {
            let mut lpServiceName: LPWSTR;
            let mut lpMachineName: LPWSTR;

            if *lpPath.offset(0 as isize) == 0x5C
                && *lpPath.offset(1 as isize) == 0x5C
            {
                DesiredAccess |= SC_MANAGER_CONNECT;

                lpMachineName = lpPath;

                lpServiceName = wcschr(lpPath.offset(2 as isize), 0x5C);
                if lpServiceName.is_null() {
                    Ret = ERROR_INVALID_PARAMETER;
                    return 'Cleanup: {
                        if !lpPath.is_null() && lpPath != pObjectName {
                            LocalFree(lpPath as HLOCAL);
                        }
                        Ret
                    };
                } else {
                    *lpServiceName = 0;
                    lpServiceName = lpServiceName.offset(1);
                }
            } else {
                lpMachineName = core::ptr::null_mut();
                lpServiceName = lpPath;
            }

            *Handle2 = OpenSCManager(lpMachineName, core::ptr::null_mut(), DesiredAccess as DWORD) as HANDLE;
            if (*Handle2).is_null() {
                Ret = GetLastError();

                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }

            DesiredAccess &= !SC_MANAGER_CONNECT;
            *Handle = OpenService(*Handle2 as SC_HANDLE, lpServiceName, DesiredAccess as DWORD) as HANDLE;
            if (*Handle).is_null() {
                Ret = GetLastError();

                CloseServiceHandle(*Handle2 as SC_HANDLE);

                return 'Cleanup: {
                    if !lpPath.is_null() && lpPath != pObjectName {
                        LocalFree(lpPath as HLOCAL);
                    }
                    Ret
                };
            }
        }

        _ => {
            UNIMPLEMENTED!();
            Ret = ERROR_CALL_NOT_IMPLEMENTED;
        }
    }

    if !lpPath.is_null() && lpPath != pObjectName {
        LocalFree(lpPath as HLOCAL);
    }

    Ret
}
unsafe extern "C" fn AccpCloseObjectHandle(ObjectType: SE_OBJECT_TYPE,
                                Handle: HANDLE,
                                Handle2: HANDLE) {
    match ObjectType {
        SE_REGISTRY_KEY => {
            RegCloseKey(Handle as HKEY);
            if Handle2 != core::ptr::null_mut() {
                RegCloseKey(Handle2 as HKEY);
            }
        }

        SE_FILE_OBJECT => {
            NtClose(Handle);
        }

        SE_KERNEL_OBJECT |
        SE_WINDOW_OBJECT => {
            CloseHandle(Handle);
        }

        SE_SERVICE => {
            CloseServiceHandle(Handle as SC_HANDLE);
            CloseServiceHandle(Handle2 as SC_HANDLE);
        }

        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteGetNamedRights(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Handle2: HANDLE = core::ptr::null_mut();
    let LastErr: DWORD;
    let mut Ret: DWORD;

    LastErr = GetLastError();

    Ret = AccpOpenNamedObject(
        pObjectName,
        ObjectType,
        SecurityInfo,
        core::ptr::addr_of_mut!(Handle),
        core::ptr::addr_of_mut!(Handle2),
        FALSE,
    );

    if Ret == ERROR_SUCCESS {
        Ret = AccRewriteGetHandleRights(
            Handle,
            ObjectType,
            SecurityInfo,
            ppsidOwner,
            ppsidGroup,
            ppDacl,
            ppSacl,
            ppSecurityDescriptor,
        );

        AccpCloseObjectHandle(ObjectType, Handle, Handle2);
    }

    SetLastError(LastErr);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteSetNamedRights(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Handle2: HANDLE = core::ptr::null_mut();
    let LastErr: DWORD;
    let mut Ret: DWORD;

    LastErr = GetLastError();

    Ret = AccpOpenNamedObject(
        pObjectName,
        ObjectType,
        SecurityInfo,
        &mut Handle,
        &mut Handle2,
        TRUE,
    );

    if Ret == ERROR_SUCCESS {
        Ret = AccRewriteSetHandleRights(
            Handle,
            ObjectType,
            SecurityInfo,
            pSecurityDescriptor,
        );

        AccpCloseObjectHandle(ObjectType, Handle, Handle2);
    }

    SetLastError(LastErr);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteSetEntriesInAcl(
    cCountOfExplicitEntries: ULONG,
    pListOfExplicitEntries: PEXPLICIT_ACCESS_W,
    OldAcl: PACL,
    NewAcl: *mut PACL,
) -> DWORD {
    let mut pNew: PACL = core::ptr::null_mut();
    let mut SizeInformation: ACL_SIZE_INFORMATION = core::mem::zeroed();
    let mut pAce: PACE_HEADER = core::ptr::null_mut();
    let mut KeepAceBuf: [BOOLEAN; 8] = [0; 8];
    let mut pKeepAce: *mut BOOLEAN = core::ptr::null_mut();
    let mut ObjectTypeGuid: GUID = core::mem::zeroed();
    let mut InheritedObjectTypeGuid: GUID = core::mem::zeroed();
    let mut ObjectsPresent: DWORD;
    let mut needToClean: BOOL = 0;
    let mut pSid1: PSID = core::ptr::null_mut();
    let mut pSid2: PSID = core::ptr::null_mut();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL;
    let mut LastErr: DWORD;
    let mut Ret: DWORD = ERROR_SUCCESS;

    LastErr = GetLastError();

    *NewAcl = core::ptr::null_mut();

    if !OldAcl.is_null() {
        if GetAclInformation(
            OldAcl,
            &mut SizeInformation as *mut _ as LPVOID,
            core::mem::size_of::<ACL_SIZE_INFORMATION>() as DWORD,
            AclSizeInformation,
        ) == 0
        {
            Ret = GetLastError();
            'Cleanup: loop {
                if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                    LocalFree(pKeepAce as HLOCAL);
                }

                if !pNew.is_null() && Ret != ERROR_SUCCESS {
                    LocalFree(pNew as HLOCAL);
                }

                if !PolicyHandle.is_null() {
                    LsaClose(PolicyHandle);
                }

                SetLastError(LastErr);

                return Ret;
            }
        }

        if SizeInformation.AceCount > (core::mem::size_of_val(&KeepAceBuf) / core::mem::size_of::<BOOLEAN>()) as DWORD {
            pKeepAce = LocalAlloc(
                LMEM_FIXED,((SizeInformation.AceCount as usize) * core::mem::size_of::<BOOLEAN>()) as SIZE_T,
            ) as *mut BOOLEAN;
            if pKeepAce.is_null() {
                Ret = ERROR_NOT_ENOUGH_MEMORY;
                'Cleanup: loop {
                    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                        LocalFree(pKeepAce as HLOCAL);
                    }

                    if !pNew.is_null() && Ret != ERROR_SUCCESS {
                        LocalFree(pNew as HLOCAL);
                    }

                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }

                    SetLastError(LastErr);

                    return Ret;
                }
            }
        } else {
            pKeepAce = KeepAceBuf.as_mut_ptr();
        }

        memset(
            pKeepAce as *mut _,
            TRUE,
            (SizeInformation.AceCount as usize) * core::mem::size_of::<BOOLEAN>(),
        );
    } else {
        ZeroMemory!(
            &mut SizeInformation as *mut _ as *mut _,
            core::mem::size_of::<ACL_SIZE_INFORMATION>(),
        );
        SizeInformation.AclBytesInUse = core::mem::size_of::<ACL>() as DWORD;
    }

    i = 0;
    while i < cCountOfExplicitEntries {
        Ret = AccpGetTrusteeSid(
            &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
            &mut PolicyHandle,
            &mut pSid1,
            &mut needToClean,
        );
        if Ret != ERROR_SUCCESS {
            'Cleanup: loop {
                if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                    LocalFree(pKeepAce as HLOCAL);
                }

                if !pNew.is_null() && Ret != ERROR_SUCCESS {
                    LocalFree(pNew as HLOCAL);
                }

                if !PolicyHandle.is_null() {
                    LsaClose(PolicyHandle);
                }

                SetLastError(LastErr);

                return Ret;
            }
        }

        ObjectsPresent = AccpGetTrusteeObjects(
            &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        match (*pListOfExplicitEntries.offset(i as isize)).grfAccessMode {
            REVOKE_ACCESS | SET_ACCESS => {
                j = 0;
                while j < SizeInformation.AceCount {
                    if *pKeepAce.offset(j as isize) == 0 {
                        j += 1;
                        continue;
                    }
                    if GetAce(OldAcl, j, &mut pAce as *mut _ as *mut LPVOID) == 0 {
                        Ret = GetLastError();
                        'Cleanup: loop {
                            if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                                LocalFree(pKeepAce as HLOCAL);
                            }

                            if !pNew.is_null() && Ret != ERROR_SUCCESS {
                                LocalFree(pNew as HLOCAL);
                            }

                            if !PolicyHandle.is_null() {
                                LsaClose(PolicyHandle);
                            }

                            SetLastError(LastErr);

                            return Ret;
                        }
                    }

                    pSid2 = AccpGetAceSid(pAce);
                    if RtlEqualSid(pSid1, pSid2) != 0 {
                        *pKeepAce.offset(j as isize) = FALSE as BOOLEAN;
                        SizeInformation.AclBytesInUse -= (*pAce).AceSize as DWORD;
                    }
                    j += 1;
                }
                if (*pListOfExplicitEntries.offset(i as isize)).grfAccessMode == REVOKE_ACCESS {
                    // break from switch
                } else {
                    SizeInformation.AclBytesInUse += AccpCalcNeededAceSize(pSid1, ObjectsPresent);
                }
            }
            GRANT_ACCESS | DENY_ACCESS => {
                SizeInformation.AclBytesInUse += AccpCalcNeededAceSize(pSid1, ObjectsPresent);
            }
            SET_AUDIT_SUCCESS | SET_AUDIT_FAILURE => {
                // DPRINT removed
            }
            _ => {
                // DPRINT removed
            }
        }

        if needToClean != 0 {
            LocalFree(pSid1 as HLOCAL);
        }
        i += 1;
    }

    if SizeInformation.AclBytesInUse == 0 {
        'Cleanup: loop {
            if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                LocalFree(pKeepAce as HLOCAL);
            }

            if !pNew.is_null() && Ret != ERROR_SUCCESS {
                LocalFree(pNew as HLOCAL);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }

            SetLastError(LastErr);

            return Ret;
        }
    }

    // DPRINT removed
    pNew = LocalAlloc(LMEM_FIXED, (SizeInformation.AclBytesInUse as usize) as SIZE_T) as PACL;
    if pNew.is_null() {
        Ret = ERROR_NOT_ENOUGH_MEMORY;
        'Cleanup: loop {
            if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                LocalFree(pKeepAce as HLOCAL);
            }

            if !pNew.is_null() && Ret != ERROR_SUCCESS {
                LocalFree(pNew as HLOCAL);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }

            SetLastError(LastErr);

            return Ret;
        }
    }
    if InitializeAcl(pNew, SizeInformation.AclBytesInUse, ACL_REVISION) == 0 {
        Ret = GetLastError();
        'Cleanup: loop {
            if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                LocalFree(pKeepAce as HLOCAL);
            }

            if !pNew.is_null() && Ret != ERROR_SUCCESS {
                LocalFree(pNew as HLOCAL);
            }

            if !PolicyHandle.is_null() {
                LsaClose(PolicyHandle);
            }

            SetLastError(LastErr);

            return Ret;
        }
    }

    i = 0;
    while i < cCountOfExplicitEntries {
        if (*pListOfExplicitEntries.offset(i as isize)).grfAccessMode == DENY_ACCESS {
            Ret = AccpGetTrusteeSid(
                &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
                &mut PolicyHandle,
                &mut pSid1,
                &mut needToClean,
            );
            if Ret != ERROR_SUCCESS {
                'Cleanup: loop {
                    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                        LocalFree(pKeepAce as HLOCAL);
                    }

                    if !pNew.is_null() && Ret != ERROR_SUCCESS {
                        LocalFree(pNew as HLOCAL);
                    }

                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }

                    SetLastError(LastErr);

                    return Ret;
                }
            }

            ObjectsPresent = AccpGetTrusteeObjects(
                &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
                &mut ObjectTypeGuid,
                &mut InheritedObjectTypeGuid,
            );

            if ObjectsPresent == 0 {
                bRet = AddAccessDeniedAce(
                    pNew,
                    ACL_REVISION,
                    (*pListOfExplicitEntries.offset(i as isize)).grfAccessPermissions,
                    pSid1,
                );
            } else {
                // DPRINT removed
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
                bRet = FALSE;
            }

            if needToClean != 0 {
                LocalFree(pSid1 as HLOCAL);
            }
            if bRet == 0 {
                Ret = GetLastError();
                'Cleanup: loop {
                    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                        LocalFree(pKeepAce as HLOCAL);
                    }

                    if !pNew.is_null() && Ret != ERROR_SUCCESS {
                        LocalFree(pNew as HLOCAL);
                    }

                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }

                    SetLastError(LastErr);

                    return Ret;
                }
            }
        }
        i += 1;
    }

    i = 0;
    while i < cCountOfExplicitEntries {
        if (*pListOfExplicitEntries.offset(i as isize)).grfAccessMode == SET_ACCESS
            || (*pListOfExplicitEntries.offset(i as isize)).grfAccessMode == GRANT_ACCESS
        {
            Ret = AccpGetTrusteeSid(
                &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
                &mut PolicyHandle,
                &mut pSid1,
                &mut needToClean,
            );
            if Ret != ERROR_SUCCESS {
                'Cleanup: loop {
                    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                        LocalFree(pKeepAce as HLOCAL);
                    }

                    if !pNew.is_null() && Ret != ERROR_SUCCESS {
                        LocalFree(pNew as HLOCAL);
                    }

                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }

                    SetLastError(LastErr);

                    return Ret;
                }
            }

            ObjectsPresent = AccpGetTrusteeObjects(
                &mut (*pListOfExplicitEntries.offset(i as isize)).Trustee,
                &mut ObjectTypeGuid,
                &mut InheritedObjectTypeGuid,
            );

            if ObjectsPresent == 0 {
                bRet = AddAccessAllowedAce(
                    pNew,
                    ACL_REVISION,
                    (*pListOfExplicitEntries.offset(i as isize)).grfAccessPermissions,
                    pSid1,
                );
            } else {
                // DPRINT removed
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
                bRet = FALSE;
            }

            if needToClean != 0 {
                LocalFree(pSid1 as HLOCAL);
            }
            if bRet == 0 {
                Ret = GetLastError();
                'Cleanup: loop {
                    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
                        LocalFree(pKeepAce as HLOCAL);
                    }

                    if !pNew.is_null() && Ret != ERROR_SUCCESS {
                        LocalFree(pNew as HLOCAL);
                    }

                    if !PolicyHandle.is_null() {
                        LsaClose(PolicyHandle);
                    }

                    SetLastError(LastErr);

                    return Ret;
                }
            }
        }
        i += 1;
    }

    *NewAcl = pNew;

    if !pKeepAce.is_null() && pKeepAce != KeepAceBuf.as_mut_ptr() {
        LocalFree(pKeepAce as HLOCAL);
    }

    if !pNew.is_null() && Ret != ERROR_SUCCESS {
        LocalFree(pNew as HLOCAL);
    }

    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }

    SetLastError(LastErr);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AccGetInheritanceSource(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    Container: BOOL,
    pObjectClassGuids: *mut *mut GUID,
    GuidCount: DWORD,
    pAcl: PACL,
    pfnArray: PFN_OBJECT_MGR_FUNCTS,
    pGenericMapping: PGENERIC_MAPPING,
    pInheritArray: PINHERITED_FROMW,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn AccFreeIndexArray(
    pInheritArray: PINHERITED_FROMW,
    AceCnt: USHORT,
    pfnArray: PFN_OBJECT_MGR_FUNCTS,
) -> DWORD {
    let mut pInheritArray = pInheritArray;
    let pLast = pInheritArray.offset(AceCnt as isize);

    UNREFERENCED_PARAMETER!(pfnArray);

    while pInheritArray != pLast {
        if (*pInheritArray).AncestorName != core::ptr::null_mut() {
            LocalFree((*pInheritArray).AncestorName as HLOCAL);
            (*pInheritArray).AncestorName = core::ptr::null_mut();
        }

        pInheritArray = pInheritArray.offset(1);
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn AccRewriteGetExplicitEntriesFromAcl(
    pacl: PACL,
    pcCountOfExplicitEntries: *mut u32,
    pListOfExplicitEntries: *mut PEXPLICIT_ACCESS_W,
) -> u32 {
    let mut AceHeader: PACE_HEADER = core::ptr::null_mut();
    let mut Sid: PSID;
    let mut SidTarget: PSID;
    let mut ObjectAceCount: u32 = 0;
    let mut ObjSid: *mut OBJECTS_AND_SID;
    let mut Size: usize;
    let mut peaw: *mut EXPLICIT_ACCESS_W;
    let mut LastErr: u32;
    let mut SidLen: u32;
    let mut AceIndex: u32 = 0;
    let mut ErrorCode: u32 = ERROR_SUCCESS;

    LastErr = GetLastError();

    if !pacl.is_null() {
        if (*pacl).AceCount != 0 {
            Size = ((*pacl).AceCount as usize) * core::mem::size_of::<EXPLICIT_ACCESS_W>();

            while GetAce(
                pacl,
                AceIndex,
                &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void,
            ) != 0
            {
                Sid = AccpGetAceSid(AceHeader);
                Size += GetLengthSid(Sid) as usize;

                if AccpIsObjectAce(AceHeader) != 0 {
                    ObjectAceCount += 1;
                }

                AceIndex += 1;
            }

            Size += (ObjectAceCount as usize) * core::mem::size_of::<OBJECTS_AND_SID>();

            peaw = LocalAlloc(LMEM_FIXED, (Size) as SIZE_T) as *mut EXPLICIT_ACCESS_W;
            if !peaw.is_null() {
                AceIndex = 0;
                ObjSid = peaw.add((*pacl).AceCount as usize) as *mut OBJECTS_AND_SID;
                SidTarget = (ObjSid as *mut u8).add(ObjectAceCount as usize * core::mem::size_of::<OBJECTS_AND_SID>()) as PSID;

                while GetAce(
                    pacl,
                    AceIndex,
                    &mut AceHeader as *mut _ as *mut *mut core::ffi::c_void,
                ) != 0
                {
                    Sid = AccpGetAceSid(AceHeader);
                    SidLen = GetLengthSid(Sid);

                    (*peaw.add(AceIndex as usize)).grfAccessPermissions = AccpGetAceAccessMask(AceHeader);
                    (*peaw.add(AceIndex as usize)).grfAccessMode = AccpGetAceAccessMode(AceHeader);
                    (*peaw.add(AceIndex as usize)).grfInheritance = (*AceHeader).AceFlags as u32 & VALID_INHERIT_FLAGS;

                    if CopySid(SidLen, SidTarget, Sid) != 0 {
                        if AccpIsObjectAce(AceHeader) != 0 {
                            BuildTrusteeWithObjectsAndSid(
                                core::ptr::addr_of_mut!((*peaw.add(AceIndex as usize)).Trustee),
                                ObjSid,
                                AccpGetObjectAceObjectType(AceHeader),
                                AccpGetObjectAceInheritedObjectType(AceHeader),
                                SidTarget,
                            );
                            ObjSid = ObjSid.add(1);
                        } else {
                            BuildTrusteeWithSid(
                                core::ptr::addr_of_mut!((*peaw.add(AceIndex as usize)).Trustee),
                                SidTarget,
                            );
                        }

                        SidTarget = (SidTarget as usize + SidLen as usize) as PSID;
                    } else {
                        ErrorCode = GetLastError();

                        LocalFree(peaw as HLOCAL);
                        peaw = core::ptr::null_mut();
                        AceIndex = 0;
                        break;
                    }

                    AceIndex += 1;
                }

                *pcCountOfExplicitEntries = AceIndex;
                *pListOfExplicitEntries = peaw;
            } else {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            }
        } else {
            *pcCountOfExplicitEntries = 0;
            *pListOfExplicitEntries = core::ptr::null_mut();
        }
    } else {
        *pcCountOfExplicitEntries = 0;
        *pListOfExplicitEntries = core::ptr::null_mut();
    }

    SetLastError(LastErr);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn AccTreeResetNamedSecurityInfo(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    mut SecurityInfo: SECURITY_INFORMATION,
    pOwner: PSID,
    pGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
    KeepExplicit: BOOL,
    fnProgress: FN_PROGRESSW,
    ProgressInvokeSetting: PROG_INVOKE_SETTING,
    Args: PVOID,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS MARTA provider
//  * Copyright (C) 2005 - 2006 ReactOS Team
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS MARTA provider
//  * FILE:            lib/ntmarta/ntmarta.c
//  * PURPOSE:         ReactOS MARTA provider
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      07/26/2005  Created
//  */
//
// #include "ntmarta.h"
//
// #define NDEBUG
// #include <debug.h>
//
// HINSTANCE hDllInstance;
//
// /* FIXME: Vista+ API */
// VOID
// WINAPI
// SetSecurityAccessMask( SECURITY_INFORMATION SecurityInformation,
//                        LPDWORD DesiredAccess)
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
// /* FIXME: Vista+ API */
// VOID
// WINAPI
// QuerySecurityAccessMask( SECURITY_INFORMATION SecurityInformation,
//                          LPDWORD DesiredAccess)
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
// static ACCESS_MODE
// AccpGetAceAccessMode( PACE_HEADER AceHeader)
// {
//     ACCESS_MODE Mode = NOT_USED_ACCESS;
//
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_ACE_TYPE:
//         case ACCESS_ALLOWED_CALLBACK_ACE_TYPE:
//         case ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//             Mode = GRANT_ACCESS;
//             break;
//
//         case ACCESS_DENIED_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//             Mode = DENY_ACCESS;
//             break;
//
//         case SYSTEM_AUDIT_ACE_TYPE:
//         case SYSTEM_AUDIT_CALLBACK_ACE_TYPE:
//         case SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE:
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//             if (AceHeader->AceFlags & FAILED_ACCESS_ACE_FLAG)
//                 Mode = SET_AUDIT_FAILURE;
//             else if (AceHeader->AceFlags & SUCCESSFUL_ACCESS_ACE_FLAG)
//                 Mode = SET_AUDIT_SUCCESS;
//             break;
//     }
//
//     return Mode;
// }
//
// static UINT
// AccpGetAceStructureSize( PACE_HEADER AceHeader)
// {
//     UINT Size = 0;
//
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_ACE_TYPE:
//         case ACCESS_DENIED_ACE_TYPE:
//             Size = FIELD_OFFSET(ACCESS_ALLOWED_ACE,
//                                 SidStart);
//             break;
//         case ACCESS_ALLOWED_CALLBACK_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_ACE_TYPE:
//             Size = FIELD_OFFSET(ACCESS_ALLOWED_CALLBACK_ACE,
//                                 SidStart);
//             break;
//         case ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_CALLBACK_OBJECT_ACE Ace = (PACCESS_ALLOWED_CALLBACK_OBJECT_ACE)AceHeader;
//             Size = FIELD_OFFSET(ACCESS_ALLOWED_CALLBACK_OBJECT_ACE,
//                                 ObjectType);
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->ObjectType);
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->InheritedObjectType);
//             break;
//         }
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_OBJECT_ACE Ace = (PACCESS_ALLOWED_OBJECT_ACE)AceHeader;
//             Size = FIELD_OFFSET(ACCESS_ALLOWED_OBJECT_ACE,
//                                 ObjectType);
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->ObjectType);
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->InheritedObjectType);
//             break;
//         }
//
//         case SYSTEM_AUDIT_ACE_TYPE:
//             Size = FIELD_OFFSET(SYSTEM_AUDIT_ACE,
//                                 SidStart);
//             break;
//         case SYSTEM_AUDIT_CALLBACK_ACE_TYPE:
//             Size = FIELD_OFFSET(SYSTEM_AUDIT_CALLBACK_ACE,
//                                 SidStart);
//             break;
//         case SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE Ace = (PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE)AceHeader;
//             Size = FIELD_OFFSET(SYSTEM_AUDIT_CALLBACK_OBJECT_ACE,
//                                 ObjectType);
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->ObjectType);
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->InheritedObjectType);
//             break;
//         }
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_OBJECT_ACE Ace = (PSYSTEM_AUDIT_OBJECT_ACE)AceHeader;
//             Size = FIELD_OFFSET(SYSTEM_AUDIT_OBJECT_ACE,
//                                 ObjectType);
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->ObjectType);
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//                 Size += sizeof(Ace->InheritedObjectType);
//             break;
//         }
//
//         case SYSTEM_MANDATORY_LABEL_ACE_TYPE:
//             Size = FIELD_OFFSET(SYSTEM_MANDATORY_LABEL_ACE,
//                                 SidStart);
//             break;
//     }
//
//     return Size;
// }
//
// static PSID
// AccpGetAceSid( PACE_HEADER AceHeader)
// {
//     return (PSID)((ULONG_PTR)AceHeader + AccpGetAceStructureSize(AceHeader));
// }
//
// static ACCESS_MASK
// AccpGetAceAccessMask( PACE_HEADER AceHeader)
// {
//     return *((PACCESS_MASK)(AceHeader + 1));
// }
//
// static BOOL
// AccpIsObjectAce( PACE_HEADER AceHeader)
// {
//     BOOL Ret;
//
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//         case SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE:
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//             Ret = TRUE as DWORD;
//             break;
//
//         default:
//             Ret = FALSE as DWORD;
//             break;
//     }
//
//     return Ret;
// }
//
// static DWORD
// AccpGetTrusteeObjects( PTRUSTEE_W Trustee,
//                        GUID *pObjectTypeGuid  ,
//                        GUID *pInheritedObjectTypeGuid  )
// {
//     DWORD Ret;
//
//     switch (Trustee->TrusteeForm)
//     {
//         case TRUSTEE_IS_OBJECTS_AND_NAME:
//         {
//             POBJECTS_AND_NAME_W pOan = (POBJECTS_AND_NAME_W)Trustee->ptstrName;
//
//             /* pOan->ObjectsPresent should always be 0 here because a previous
//                call to AccpGetTrusteeSid should have rejected these trustees
//                already. */
//             ASSERT(pOan->ObjectsPresent == 0);
//
//             Ret = pOan->ObjectsPresent;
//             break;
//         }
//
//         case TRUSTEE_IS_OBJECTS_AND_SID:
//         {
//             POBJECTS_AND_SID pOas = (POBJECTS_AND_SID)Trustee->ptstrName;
//
//             if (pObjectTypeGuid != NULL && pOas->ObjectsPresent & ACE_OBJECT_TYPE_PRESENT)
//                 *pObjectTypeGuid = pOas->ObjectTypeGuid;
//
//             if (pInheritedObjectTypeGuid != NULL && pOas->ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//                 *pInheritedObjectTypeGuid = pOas->InheritedObjectTypeGuid;
//
//             Ret = pOas->ObjectsPresent;
//             break;
//         }
//
//         default:
//             /* Any other trustee forms have no objects attached... */
//             Ret = 0;
//             break;
//     }
//
//     return Ret;
// }
//
// static DWORD
// AccpCalcNeededAceSize( PSID Sid,
//                        DWORD ObjectsPresent)
// {
//     DWORD Ret;
//
//     Ret = sizeof(ACE) + GetLengthSid(Sid);
//
//     /* This routine calculates the generic size of the ACE needed.
//        If no objects are present it is assumed that only a standard
//        ACE is to be created. */
//
//     if (ObjectsPresent & ACE_OBJECT_TYPE_PRESENT)
//         Ret += sizeof(GUID);
//     if (ObjectsPresent & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//         Ret += sizeof(GUID);
//
//     if (ObjectsPresent != 0)
//         Ret += sizeof(DWORD); /* Include the Flags member to make it an object ACE */
//
//     return Ret;
// }
//
// static GUID*
// AccpGetObjectAceObjectType( PACE_HEADER AceHeader)
// {
//     GUID *ObjectType = NULL;
//
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_CALLBACK_OBJECT_ACE Ace = (PACCESS_ALLOWED_CALLBACK_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 ObjectType = &Ace->ObjectType;
//             break;
//         }
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_OBJECT_ACE Ace = (PACCESS_ALLOWED_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 ObjectType = &Ace->ObjectType;
//             break;
//         }
//
//         case SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE Ace = (PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 ObjectType = &Ace->ObjectType;
//             break;
//         }
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_OBJECT_ACE Ace = (PSYSTEM_AUDIT_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                 ObjectType = &Ace->ObjectType;
//             break;
//         }
//     }
//
//     return ObjectType;
// }
//
// static GUID*
// AccpGetObjectAceInheritedObjectType( PACE_HEADER AceHeader)
// {
//     GUID *ObjectType = NULL;
//
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_CALLBACK_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_CALLBACK_OBJECT_ACE Ace = (PACCESS_ALLOWED_CALLBACK_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//             {
//                 if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                     ObjectType = &Ace->InheritedObjectType;
//                 else
//                     ObjectType = &Ace->ObjectType;
//             }
//             break;
//         }
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//         case ACCESS_DENIED_OBJECT_ACE_TYPE:
//         {
//             PACCESS_ALLOWED_OBJECT_ACE Ace = (PACCESS_ALLOWED_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//             {
//                 if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                     ObjectType = &Ace->InheritedObjectType;
//                 else
//                     ObjectType = &Ace->ObjectType;
//             }
//             break;
//         }
//
//         case SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE Ace = (PSYSTEM_AUDIT_CALLBACK_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//             {
//                 if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                     ObjectType = &Ace->InheritedObjectType;
//                 else
//                     ObjectType = &Ace->ObjectType;
//             }
//             break;
//         }
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//         {
//             PSYSTEM_AUDIT_OBJECT_ACE Ace = (PSYSTEM_AUDIT_OBJECT_ACE)AceHeader;
//             if (Ace->Flags & ACE_INHERITED_OBJECT_TYPE_PRESENT)
//             {
//                 if (Ace->Flags & ACE_OBJECT_TYPE_PRESENT)
//                     ObjectType = &Ace->InheritedObjectType;
//                 else
//                     ObjectType = &Ace->ObjectType;
//             }
//             break;
//         }
//     }
//
//     return ObjectType;
// }
//
// static DWORD
// AccpOpenLSAPolicyHandle( LPWSTR SystemName,
//                          ACCESS_MASK DesiredAccess,
//                          PLSA_HANDLE pPolicyHandle)
// {
//     LSA_OBJECT_ATTRIBUTES LsaObjectAttributes = {0};
//     LSA_UNICODE_STRING LsaSystemName, *psn;
//     SIZE_T SystemNameLength;
//     NTSTATUS Status;
//
//     if (SystemName != NULL && SystemName[0] != L'\0')
//     {
//         SystemNameLength = wcslen(SystemName);
//         if (SystemNameLength > UNICODE_STRING_MAX_CHARS)
//         {
//             return ERROR_INVALID_PARAMETER;
//         }
//
//         LsaSystemName.Buffer = SystemName;
//         LsaSystemName.Length = (USHORT)SystemNameLength * sizeof(WCHAR);
//         LsaSystemName.MaximumLength = LsaSystemName.Length + sizeof(WCHAR);
//         psn = &LsaSystemName;
//     }
//     else
//     {
//         psn = NULL;
//     }
//
//     Status = LsaOpenPolicy(psn,
//                            &LsaObjectAttributes,
//                            DesiredAccess,
//                            pPolicyHandle);
//     if (!NT_SUCCESS(Status))
//         return LsaNtStatusToWinError(Status);
//
//     return ERROR_SUCCESS;
// }
//
// static LPWSTR
// AccpGetTrusteeName( PTRUSTEE_W Trustee)
// {
//     switch (Trustee->TrusteeForm)
//     {
//         case TRUSTEE_IS_NAME:
//             return Trustee->ptstrName;
//
//         case TRUSTEE_IS_OBJECTS_AND_NAME:
//             return ((POBJECTS_AND_NAME_W)Trustee->ptstrName)->ptstrName;
//
//         default:
//             return NULL;
//     }
// }
//
// static DWORD
// AccpLookupCurrentUser( PSID *ppSid)
// {
//     DWORD Ret;
//     CHAR Buffer[sizeof(TOKEN_USER) + sizeof(SID) + sizeof(DWORD)*SID_MAX_SUB_AUTHORITIES];
//     DWORD Length;
//     HANDLE Token;
//     PSID pSid;
//
//     *ppSid = NULL;
//     if (!OpenThreadToken(GetCurrentThread(), TOKEN_READ, TRUE, &Token))
//     {
//         Ret = GetLastError();
//         if (Ret != ERROR_NO_TOKEN)
//         {
//             return Ret;
//         }
//
//         if (!OpenProcessToken(GetCurrentProcess(), TOKEN_READ, &Token))
//         {
//             return GetLastError();
//         }
//     }
//
//     Length = sizeof(Buffer);
//     if (!GetTokenInformation(Token, TokenUser, Buffer, Length, &Length))
//     {
//         Ret = GetLastError();
//         CloseHandle(Token);
//         return Ret;
//     }
//     CloseHandle(Token);
//
//     pSid = ((PTOKEN_USER)Buffer)->User.Sid;
//     Length = GetLengthSid(pSid);
//     *ppSid = LocalAlloc(LMEM_FIXED, Length);
//     if (!*ppSid)
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     CopyMemory(*ppSid, pSid, Length);
//
//     return ERROR_SUCCESS;
// }
//
// static DWORD
// AccpLookupSidByName( LSA_HANDLE PolicyHandle,
//                      LPWSTR Name,
//                      PSID *pSid)
// {
//     NTSTATUS Status;
//     LSA_UNICODE_STRING LsaNames[1];
//     PLSA_REFERENCED_DOMAIN_LIST ReferencedDomains = NULL;
//     PLSA_TRANSLATED_SID2 TranslatedSid = NULL;
//     DWORD SidLen;
//     SIZE_T NameLength;
//     DWORD Ret = ERROR_SUCCESS;
//
//     NameLength = wcslen(Name) as SIZE_T;
//     if (NameLength > UNICODE_STRING_MAX_CHARS)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     LsaNames[0].Buffer = Name;
//     LsaNames[0].Length = (USHORT)NameLength * sizeof(WCHAR);
//     LsaNames[0].MaximumLength = LsaNames[0].Length + sizeof(WCHAR);
//
//     Status = LsaLookupNames2(PolicyHandle,
//                              0,
//                              sizeof(LsaNames) / sizeof(LsaNames[0]),
//                              LsaNames,
//                              &ReferencedDomains,
//                              &TranslatedSid);
//
//     if (!NT_SUCCESS(Status))
//         return LsaNtStatusToWinError(Status);
//
//     if (TranslatedSid->Use == SidTypeUnknown || TranslatedSid->Use == SidTypeInvalid)
//     {
//         Ret = LsaNtStatusToWinError(STATUS_NONE_MAPPED); /* FIXME- what error code? */
//         goto Cleanup;
//     }
//
//     SidLen = GetLengthSid(TranslatedSid->Sid);
//     ASSERT(SidLen != 0);
//
//     *pSid = LocalAlloc(LMEM_FIXED, (SIZE_T)SidLen);
//     if (*pSid != NULL)
//     {
//         if (!CopySid(SidLen,
//                      *pSid,
//                      TranslatedSid->Sid))
//         {
//             Ret = GetLastError();
//
//             LocalFree((HLOCAL)*pSid);
//             *pSid = NULL;
//         }
//     }
//     else
//         Ret = ERROR_NOT_ENOUGH_MEMORY;
//
// Cleanup:
//     LsaFreeMemory(ReferencedDomains as PVOID);
//     LsaFreeMemory(TranslatedSid as PVOID);
//
//     return Ret;
// }
//
//
// static DWORD
// AccpGetTrusteeSid( PTRUSTEE_W Trustee,
//                     PLSA_HANDLE pPolicyHandle,
//                    PSID *ppSid,
//                    BOOL *Allocated)
// {
//     DWORD Ret = ERROR_SUCCESS;
//     LPWSTR TrusteeName;
//
//     *ppSid = NULL;
//     *Allocated = FALSE;
//
//     /* Windows ignores this */
//
//     switch (Trustee->TrusteeForm)
//     {
//         case TRUSTEE_IS_OBJECTS_AND_NAME:
//             if (((POBJECTS_AND_NAME_W)Trustee->ptstrName)->ObjectsPresent != 0)
//             {
//                 /* This is not supported as there is no way to interpret the
//                    strings provided, and we need GUIDs for the ACEs... */
//                 Ret = ERROR_INVALID_PARAMETER;
//                 break;
//             }
//             /* fall through */
//
//         case TRUSTEE_IS_NAME:
//             TrusteeName = AccpGetTrusteeName(Trustee);
//             if (!wcscmp(TrusteeName, L"CURRENT_USER"))
//             {
//                 Ret = AccpLookupCurrentUser(ppSid);
//                 if (Ret == ERROR_SUCCESS)
//                 {
//                     ASSERT(*ppSid != NULL);
//                     *Allocated = TRUE;
//                 }
//                 break;
//             }
//
//             if (*pPolicyHandle == NULL)
//             {
//                 Ret = AccpOpenLSAPolicyHandle(NULL, /* FIXME - always local? */
//                                               POLICY_LOOKUP_NAMES,
//                                               pPolicyHandle);
//                 if (Ret != ERROR_SUCCESS)
//                     return Ret;
//
//                 ASSERT(*pPolicyHandle != NULL);
//             }
//
//             Ret = AccpLookupSidByName(*pPolicyHandle,
//                                       TrusteeName,
//                                       ppSid);
//             if (Ret == ERROR_SUCCESS)
//             {
//                 ASSERT(*ppSid != NULL);
//                 *Allocated = TRUE;
//             }
//             break;
//
//         case TRUSTEE_IS_OBJECTS_AND_SID:
//             *ppSid = ((POBJECTS_AND_SID)Trustee->ptstrName)->pSid;
//             break;
//
//         case TRUSTEE_IS_SID:
//             *ppSid = (PSID)Trustee->ptstrName;
//             break;
//
//         default:
//             Ret = ERROR_INVALID_PARAMETER;
//             break;
//     }
//
//     return Ret;
// }
//
//
// /**********************************************************************
//  * AccRewriteGetHandleRights				EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccRewriteGetHandleRights(HANDLE handle,
//                           SE_OBJECT_TYPE ObjectType,
//                           SECURITY_INFORMATION SecurityInfo,
//                           PSID* ppsidOwner,
//                           PSID* ppsidGroup,
//                           PACL* ppDacl,
//                           PACL* ppSacl,
//                           PSECURITY_DESCRIPTOR* ppSecurityDescriptor)
// {
//     PSECURITY_DESCRIPTOR pSD = NULL;
//     ULONG SDSize = 0;
//     NTSTATUS Status;
//     DWORD LastErr;
//     DWORD Ret;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     do
//     {
//         Ret = ERROR_SUCCESS;
//
//         /* allocate a buffer large enough to hold the
//            security descriptor we need to return */
//         SDSize += 0x100;
//         if (pSD == NULL)
//         {
//             pSD = LocalAlloc(LMEM_FIXED,
//                              (SIZE_T)SDSize);
//         }
//         else
//         {
//             PSECURITY_DESCRIPTOR newSD;
//
//             newSD = LocalReAlloc((HLOCAL)pSD,
//                                  (SIZE_T)SDSize,
//                                  LMEM_MOVEABLE);
//             if (newSD != NULL)
//                 pSD = newSD;
//         }
//
//         if (pSD == NULL)
//         {
//             Ret = GetLastError();
//             break;
//         }
//
//         /* perform the actual query depending on the object type */
//         switch (ObjectType)
//         {
//             case SE_REGISTRY_KEY:
//             {
//                 Ret = (DWORD)RegGetKeySecurity((HKEY)handle,
//                                                SecurityInfo,
//                                                pSD,
//                                                &SDSize);
//                 break;
//             }
//
//             case SE_FILE_OBJECT:
//                 /* FIXME - handle console handles? */
//             case SE_KERNEL_OBJECT:
//             {
//                 Status = NtQuerySecurityObject(handle,
//                                                SecurityInfo,
//                                                pSD,
//                                                SDSize,
//                                                &SDSize);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     Ret = RtlNtStatusToDosError(Status);
//                 }
//                 break;
//             }
//
//             case SE_SERVICE:
//             {
//                 if (!QueryServiceObjectSecurity((SC_HANDLE)handle,
//                                                 SecurityInfo,
//                                                 pSD,
//                                                 SDSize,
//                                                 &SDSize))
//                 {
//                     Ret = GetLastError();
//                 }
//                 break;
//             }
//
//             case SE_WINDOW_OBJECT:
//             {
//                 if (!GetUserObjectSecurity(handle,
//                                            &SecurityInfo,
//                                            pSD,
//                                            SDSize,
//                                            &SDSize))
//                 {
//                     Ret = GetLastError();
//                 }
//                 break;
//             }
//
//             default:
//             {
//                 UNIMPLEMENTED;
//                 Ret = ERROR_CALL_NOT_IMPLEMENTED;
//                 break;
//             }
//         }
//
//     } while (Ret == ERROR_INSUFFICIENT_BUFFER);
//
//     if (Ret == ERROR_SUCCESS)
//     {
//         BOOL Present, Defaulted;
//
//         if (SecurityInfo & OWNER_SECURITY_INFORMATION && ppsidOwner != NULL)
//         {
//             *ppsidOwner = NULL;
//             if (!GetSecurityDescriptorOwner(pSD,
//                                             ppsidOwner,
//                                             &Defaulted))
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//
//         if (SecurityInfo & GROUP_SECURITY_INFORMATION && ppsidGroup != NULL)
//         {
//             *ppsidGroup = NULL;
//             if (!GetSecurityDescriptorGroup(pSD,
//                                             ppsidGroup,
//                                             &Defaulted))
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//
//         if (SecurityInfo & DACL_SECURITY_INFORMATION && ppDacl != NULL)
//         {
//             *ppDacl = NULL;
//             if (!GetSecurityDescriptorDacl(pSD,
//                                            &Present,
//                                            ppDacl,
//                                            &Defaulted))
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//
//         if (SecurityInfo & SACL_SECURITY_INFORMATION && ppSacl != NULL)
//         {
//             *ppSacl = NULL;
//             if (!GetSecurityDescriptorSacl(pSD,
//                                            &Present,
//                                            ppSacl,
//                                            &Defaulted))
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//
//         *ppSecurityDescriptor = pSD;
//     }
//     else
//     {
// Cleanup:
//         if (pSD != NULL)
//         {
//             LocalFree((HLOCAL)pSD);
//         }
//     }
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return Ret;
// }
//
//
// /**********************************************************************
//  * AccRewriteSetHandleRights				EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccRewriteSetHandleRights(HANDLE handle,
//                           SE_OBJECT_TYPE ObjectType,
//                           SECURITY_INFORMATION SecurityInfo,
//                           PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     NTSTATUS Status;
//     DWORD LastErr;
//     DWORD Ret = ERROR_SUCCESS;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     /* set the security according to the object type */
//     switch (ObjectType)
//     {
//         case SE_REGISTRY_KEY:
//         {
//             Ret = (DWORD)RegSetKeySecurity((HKEY)handle,
//                                            SecurityInfo,
//                                            pSecurityDescriptor);
//             break;
//         }
//
//         case SE_FILE_OBJECT:
//             /* FIXME - handle console handles? */
//         case SE_KERNEL_OBJECT:
//         {
//             Status = NtSetSecurityObject(handle,
//                                          SecurityInfo,
//                                          pSecurityDescriptor);
//             if (!NT_SUCCESS(Status))
//             {
//                 Ret = RtlNtStatusToDosError(Status);
//             }
//             break;
//         }
//
//         case SE_SERVICE:
//         {
//             if (!SetServiceObjectSecurity((SC_HANDLE)handle,
//                                           SecurityInfo,
//                                           pSecurityDescriptor))
//             {
//                 Ret = GetLastError();
//             }
//             break;
//         }
//
//         case SE_WINDOW_OBJECT:
//         {
//             if (!SetUserObjectSecurity(handle,
//                                        &SecurityInfo,
//                                        pSecurityDescriptor))
//             {
//                 Ret = GetLastError();
//             }
//             break;
//         }
//
//         default:
//         {
//             UNIMPLEMENTED;
//             Ret = ERROR_CALL_NOT_IMPLEMENTED;
//             break;
//         }
//     }
//
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return Ret;
// }
//
//
// static DWORD
// AccpOpenNamedObject(LPWSTR pObjectName,
//                     SE_OBJECT_TYPE ObjectType,
//                     SECURITY_INFORMATION SecurityInfo,
//                     PHANDLE Handle,
//                     PHANDLE Handle2,
//                     BOOL Write)
// {
//     LPWSTR lpPath;
//     NTSTATUS Status;
//     ACCESS_MASK DesiredAccess = (ACCESS_MASK)0;
//     DWORD Ret = ERROR_SUCCESS;
//
//     /* determine the required access rights */
//     switch (ObjectType)
//     {
//         case SE_REGISTRY_KEY:
//         case SE_FILE_OBJECT:
//         case SE_KERNEL_OBJECT:
//         case SE_SERVICE:
//         case SE_WINDOW_OBJECT:
//             if (Write)
//             {
//                 SetSecurityAccessMask(SecurityInfo,
//                                       (PDWORD)&DesiredAccess);
//             }
//             else
//             {
//                 QuerySecurityAccessMask(SecurityInfo,
//                                         (PDWORD)&DesiredAccess);
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     /* make a copy of the path if we're modifying the string */
//     switch (ObjectType)
//     {
//         case SE_REGISTRY_KEY:
//         case SE_SERVICE:
//             lpPath = (LPWSTR)LocalAlloc(LMEM_FIXED,
//                                         (wcslen(pObjectName) + 1) * sizeof(WCHAR));
//             if (lpPath == NULL)
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//
//             wcscpy(lpPath,
//                    pObjectName);
//             break;
//
//         default:
//             lpPath = pObjectName;
//             break;
//     }
//
//     /* open a handle to the path depending on the object type */
//     switch (ObjectType)
//     {
//         case SE_FILE_OBJECT:
//         {
//             IO_STATUS_BLOCK IoStatusBlock;
//             OBJECT_ATTRIBUTES ObjectAttributes;
//             UNICODE_STRING FileName;
//
//             if (!RtlDosPathNameToNtPathName_U(pObjectName,
//                                               &FileName,
//                                               NULL,
//                                               NULL))
//             {
//                 Ret = ERROR_INVALID_NAME;
//                 goto Cleanup;
//             }
//
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &FileName,
//                                        OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//
//             Status = NtOpenFile(Handle,
//                                 DesiredAccess | SYNCHRONIZE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                                 FILE_SYNCHRONOUS_IO_NONALERT);
//
//             RtlFreeHeap(RtlGetProcessHeap(),
//                         0,
//                         FileName.Buffer);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 Ret = RtlNtStatusToDosError(Status);
//             }
//             break;
//         }
//
//         case SE_REGISTRY_KEY:
//         {
//             static const struct
//             {
//                 HKEY hRootKey;
//                 LPCWSTR szRootKey;
//             } AccRegRootKeys[] =
//             {
//                 {HKEY_CLASSES_ROOT, L"CLASSES_ROOT"},
//                 {HKEY_CURRENT_USER, L"CURRENT_USER"},
//                 {HKEY_LOCAL_MACHINE, L"MACHINE"},
//                 {HKEY_USERS, L"USERS"},
//                 {HKEY_CURRENT_CONFIG, L"CONFIG"},
//             };
//             LPWSTR lpMachineName, lpRootKeyName, lpKeyName;
//             HKEY hRootKey = NULL;
//             UINT i;
//
//             /* parse the registry path */
//             if (lpPath[0] == L'\\' && lpPath[1] == L'\\')
//             {
//                 lpMachineName = lpPath;
//
//                 lpRootKeyName = wcschr(lpPath + 2,
//                                        L'\\');
//                 if (lpRootKeyName == NULL)
//                     goto ParseRegErr;
//                 else
//                     *(lpRootKeyName++) = L'\0';
//             }
//             else
//             {
//                 lpMachineName = NULL;
//                 lpRootKeyName = lpPath;
//             }
//
//             lpKeyName = wcschr(lpRootKeyName,
//                                L'\\');
//             if (lpKeyName != NULL)
//             {
//                 *(lpKeyName++) = L'\0';
//             }
//
//             for (i = 0;
//                  i != sizeof(AccRegRootKeys) / sizeof(AccRegRootKeys[0]);
//                  i++)
//             {
//                 if (!_wcsicmp(lpRootKeyName,
//                              AccRegRootKeys[i].szRootKey))
//                 {
//                     hRootKey = AccRegRootKeys[i].hRootKey;
//                     break;
//                 }
//             }
//
//             if (hRootKey == NULL)
//             {
// ParseRegErr:
//                 /* FIXME - right error code? */
//                 Ret = ERROR_INVALID_PARAMETER;
//                 goto Cleanup;
//             }
//
//             /* open the registry key */
//             if (lpMachineName != NULL)
//             {
//                 Ret = RegConnectRegistry(lpMachineName,
//                                          hRootKey,
//                                          (PHKEY)Handle2) as DWORD;
//
//                 if (Ret != ERROR_SUCCESS)
//                     goto Cleanup;
//
//                 hRootKey = (HKEY)(*Handle2);
//             }
//
//             Ret = RegOpenKeyEx(hRootKey,
//                                lpKeyName,
//                                0,
//                                (REGSAM)DesiredAccess,
//                                (PHKEY)Handle);
//             if (Ret != ERROR_SUCCESS)
//             {
//                 if (*Handle2 != NULL)
//                 {
//                     RegCloseKey((HKEY)(*Handle2));
//                 }
//
//                 goto Cleanup;
//             }
//             break;
//         }
//
//         case SE_SERVICE:
//         {
//             LPWSTR lpServiceName, lpMachineName;
//
//             /* parse the service path */
//             if (lpPath[0] == L'\\' && lpPath[1] == L'\\')
//             {
//                 DesiredAccess |= SC_MANAGER_CONNECT;
//
//                 lpMachineName = lpPath;
//
//                 lpServiceName = wcschr(lpPath + 2,
//                                        L'\\');
//                 if (lpServiceName == NULL)
//                 {
//                     /* FIXME - right error code? */
//                     Ret = ERROR_INVALID_PARAMETER;
//                     goto Cleanup;
//                 }
//                 else
//                     *(lpServiceName++) = L'\0';
//             }
//             else
//             {
//                 lpMachineName = NULL;
//                 lpServiceName = lpPath;
//             }
//
//             /* open the service */
//             *Handle2 = (HANDLE)OpenSCManager(lpMachineName,
//                                              NULL,
//                                              (DWORD)DesiredAccess);
//             if (*Handle2 == NULL)
//             {
//                 Ret = GetLastError();
//                 ASSERT(Ret != ERROR_SUCCESS);
//                 goto Cleanup;
//             }
//
//             DesiredAccess &= ~SC_MANAGER_CONNECT;
//             *Handle = (HANDLE)OpenService((SC_HANDLE)(*Handle2),
//                                           lpServiceName,
//                                           (DWORD)DesiredAccess);
//             if (*Handle == NULL)
//             {
//                 Ret = GetLastError();
//                 ASSERT(Ret != ERROR_SUCCESS);
//                 ASSERT(*Handle2 != NULL);
//                 CloseServiceHandle((SC_HANDLE)(*Handle2));
//
//                 goto Cleanup;
//             }
//             break;
//         }
//
//         default:
//         {
//             UNIMPLEMENTED;
//             Ret = ERROR_CALL_NOT_IMPLEMENTED;
//             break;
//         }
//     }
//
// Cleanup:
//     if (lpPath != NULL && lpPath != pObjectName)
//     {
//         LocalFree((HLOCAL)lpPath);
//     }
//
//     return Ret;
// }
//
//
// static VOID
// AccpCloseObjectHandle(SE_OBJECT_TYPE ObjectType,
//                       HANDLE Handle,
//                       HANDLE Handle2)
// {
//     ASSERT(Handle != NULL);
//
//     /* close allocated handles depending on the object type */
//     switch (ObjectType)
//     {
//         case SE_REGISTRY_KEY:
//             RegCloseKey((HKEY)Handle);
//             if (Handle2 != NULL)
//                 RegCloseKey((HKEY)Handle2);
//             break;
//
//         case SE_FILE_OBJECT:
//             NtClose(Handle);
//             break;
//
//         case SE_KERNEL_OBJECT:
//         case SE_WINDOW_OBJECT:
//             CloseHandle(Handle);
//             break;
//
//         case SE_SERVICE:
//             CloseServiceHandle((SC_HANDLE)Handle);
//             ASSERT(Handle2 != NULL);
//             CloseServiceHandle((SC_HANDLE)Handle2);
//             break;
//
//         default:
//             break;
//     }
// }
//
//
// /**********************************************************************
//  * AccRewriteGetNamedRights				EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccRewriteGetNamedRights(LPWSTR pObjectName,
//                          SE_OBJECT_TYPE ObjectType,
//                          SECURITY_INFORMATION SecurityInfo,
//                          PSID* ppsidOwner,
//                          PSID* ppsidGroup,
//                          PACL* ppDacl,
//                          PACL* ppSacl,
//                          PSECURITY_DESCRIPTOR* ppSecurityDescriptor)
// {
//     HANDLE Handle = NULL;
//     HANDLE Handle2 = NULL;
//     DWORD LastErr;
//     DWORD Ret;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     /* create the handle */
//     Ret = AccpOpenNamedObject(pObjectName,
//                               ObjectType,
//                               SecurityInfo,
//                               &Handle,
//                               &Handle2,
//                               FALSE);
//
//     if (Ret == ERROR_SUCCESS)
//     {
//         ASSERT(Handle != NULL);
//
//         /* perform the operation */
//         Ret = AccRewriteGetHandleRights(Handle,
//                                         ObjectType,
//                                         SecurityInfo,
//                                         ppsidOwner,
//                                         ppsidGroup,
//                                         ppDacl,
//                                         ppSacl,
//                                         ppSecurityDescriptor);
//
//         /* close opened handles */
//         AccpCloseObjectHandle(ObjectType,
//                               Handle,
//                               Handle2);
//     }
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return Ret;
// }
//
//
// /**********************************************************************
//  * AccRewriteSetNamedRights				EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccRewriteSetNamedRights(LPWSTR pObjectName,
//                          SE_OBJECT_TYPE ObjectType,
//                          SECURITY_INFORMATION SecurityInfo,
//                          PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     HANDLE Handle = NULL;
//     HANDLE Handle2 = NULL;
//     DWORD LastErr;
//     DWORD Ret;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     /* create the handle */
//     Ret = AccpOpenNamedObject(pObjectName,
//                               ObjectType,
//                               SecurityInfo,
//                               &Handle,
//                               &Handle2,
//                               TRUE);
//
//     if (Ret == ERROR_SUCCESS)
//     {
//         ASSERT(Handle != NULL);
//
//         /* perform the operation */
//         Ret = AccRewriteSetHandleRights(Handle,
//                                         ObjectType,
//                                         SecurityInfo,
//                                         pSecurityDescriptor);
//
//         /* close opened handles */
//         AccpCloseObjectHandle(ObjectType,
//                               Handle,
//                               Handle2);
//     }
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return Ret;
// }
//
//
// /**********************************************************************
//  * AccRewriteSetEntriesInAcl				EXPORTED
//  *
//  * @implemented
//  */
// DWORD WINAPI
// AccRewriteSetEntriesInAcl(ULONG cCountOfExplicitEntries,
//                           PEXPLICIT_ACCESS_W pListOfExplicitEntries,
//                           PACL OldAcl,
//                           PACL* NewAcl)
// {
//     PACL pNew = NULL;
//     ACL_SIZE_INFORMATION SizeInformation;
//     PACE_HEADER pAce;
//     BOOLEAN KeepAceBuf[8];
//     BOOLEAN *pKeepAce = NULL;
//     GUID ObjectTypeGuid, InheritedObjectTypeGuid;
//     DWORD ObjectsPresent;
//     BOOL needToClean;
//     PSID pSid1, pSid2;
//     ULONG i, j;
//     LSA_HANDLE PolicyHandle = NULL;
//     BOOL bRet;
//     DWORD LastErr;
//     DWORD Ret = ERROR_SUCCESS;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     *NewAcl = NULL;
//
//     /* Get information about previous ACL */
//     if (OldAcl)
//     {
//         if (!GetAclInformation(OldAcl, &SizeInformation, sizeof(ACL_SIZE_INFORMATION), AclSizeInformation))
//         {
//             Ret = GetLastError();
//             goto Cleanup;
//         }
//
//         if (SizeInformation.AceCount > sizeof(KeepAceBuf) / sizeof(KeepAceBuf[0]))
//         {
//             pKeepAce = (BOOLEAN *)LocalAlloc(LMEM_FIXED, SizeInformation.AceCount * sizeof(*pKeepAce));
//             if (!pKeepAce)
//             {
//                 Ret = ERROR_NOT_ENOUGH_MEMORY;
//                 goto Cleanup;
//             }
//         }
//         else
//             pKeepAce = KeepAceBuf;
//
//         memset(pKeepAce, TRUE, SizeInformation.AceCount * sizeof(*pKeepAce));
//     }
//     else
//     {
//         ZeroMemory(&SizeInformation, sizeof(ACL_SIZE_INFORMATION));
//         SizeInformation.AclBytesInUse = sizeof(ACL);
//     }
//
//     /* Get size required for new entries */
//     for (i = 0; i < cCountOfExplicitEntries; i++)
//     {
//         Ret = AccpGetTrusteeSid(&pListOfExplicitEntries[i].Trustee,
//                                 &PolicyHandle,
//                                 &pSid1,
//                                 &needToClean);
//         if (Ret != ERROR_SUCCESS)
//             goto Cleanup;
//
//         ObjectsPresent = AccpGetTrusteeObjects(&pListOfExplicitEntries[i].Trustee,
//                                                NULL,
//                                                NULL);
//
//         switch (pListOfExplicitEntries[i].grfAccessMode)
//         {
//             case REVOKE_ACCESS:
//             case SET_ACCESS:
//                 /* Discard all accesses for the trustee... */
//                 for (j = 0; j < SizeInformation.AceCount; j++)
//                 {
//                     if (!pKeepAce[j])
//                         continue;
//                     if (!GetAce(OldAcl, j, (PVOID*)&pAce))
//                     {
//                         Ret = GetLastError();
//                         goto Cleanup;
//                     }
//
//                     pSid2 = AccpGetAceSid(pAce);
//                     if (RtlEqualSid(pSid1, pSid2))
//                     {
//                         pKeepAce[j] = FALSE;
//                         SizeInformation.AclBytesInUse -= pAce->AceSize;
//                     }
//                 }
//                 if (pListOfExplicitEntries[i].grfAccessMode == REVOKE_ACCESS)
//                     break;
//                 /* ...and replace by the current access */
//             case GRANT_ACCESS:
//             case DENY_ACCESS:
//                 /* Add to ACL */
//                 SizeInformation.AclBytesInUse += AccpCalcNeededAceSize(pSid1, ObjectsPresent);
//                 break;
//             case SET_AUDIT_SUCCESS:
//             case SET_AUDIT_FAILURE:
//                 /* FIXME */
//                 DPRINT1("Case not implemented!\n");
//                 break;
//             default:
//                 DPRINT1("Unknown access mode 0x%x. Ignoring it\n", pListOfExplicitEntries[i].grfAccessMode);
//                 break;
//         }
//
//         if (needToClean)
//             LocalFree((HLOCAL)pSid1);
//     }
//
//     /* Succeed, if no ACL needs to be allocated */
//     if (SizeInformation.AclBytesInUse == 0)
//         goto Cleanup;
//
//     /* OK, now create the new ACL */
//     DPRINT("Allocating %u bytes for the new ACL\n", SizeInformation.AclBytesInUse);
//     pNew = (PACL)LocalAlloc(LMEM_FIXED, SizeInformation.AclBytesInUse);
//     if (!pNew)
//     {
//         Ret = ERROR_NOT_ENOUGH_MEMORY;
//         goto Cleanup;
//     }
//     if (!InitializeAcl(pNew, SizeInformation.AclBytesInUse, ACL_REVISION))
//     {
//         Ret = GetLastError();
//         goto Cleanup;
//     }
//
//     /* Fill it */
//     /* 1a) New audit entries (SET_AUDIT_SUCCESS, SET_AUDIT_FAILURE) */
//     /* FIXME */
//
//     /* 1b) Existing audit entries */
//     /* FIXME */
//
//     /* 2a) New denied entries (DENY_ACCESS) */
//     for (i = 0; i < cCountOfExplicitEntries; i++)
//     {
//         if (pListOfExplicitEntries[i].grfAccessMode == DENY_ACCESS)
//         {
//             /* FIXME: take care of pListOfExplicitEntries[i].grfInheritance */
//             Ret = AccpGetTrusteeSid(&pListOfExplicitEntries[i].Trustee,
//                                     &PolicyHandle,
//                                     &pSid1,
//                                     &needToClean);
//             if (Ret != ERROR_SUCCESS)
//                 goto Cleanup;
//
//             ObjectsPresent = AccpGetTrusteeObjects(&pListOfExplicitEntries[i].Trustee,
//                                                    &ObjectTypeGuid,
//                                                    &InheritedObjectTypeGuid);
//
//             if (ObjectsPresent == 0)
//             {
//                 /* FIXME: Call AddAccessDeniedAceEx instead! */
//                 bRet = AddAccessDeniedAce(pNew, ACL_REVISION, pListOfExplicitEntries[i].grfAccessPermissions, pSid1);
//             }
//             else
//             {
//                 /* FIXME: Call AddAccessDeniedObjectAce */
//                 DPRINT1("Object ACEs not yet supported!\n");
//                 SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//                 bRet = FALSE;
//             }
//
//             if (needToClean) LocalFree((HLOCAL)pSid1);
//             if (!bRet)
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//     }
//
//     /* 2b) Existing denied entries */
//     /* FIXME */
//
//     /* 3a) New allow entries (GRANT_ACCESS, SET_ACCESS) */
//     for (i = 0; i < cCountOfExplicitEntries; i++)
//     {
//         if (pListOfExplicitEntries[i].grfAccessMode == SET_ACCESS ||
//             pListOfExplicitEntries[i].grfAccessMode == GRANT_ACCESS)
//         {
//             /* FIXME: take care of pListOfExplicitEntries[i].grfInheritance */
//             Ret = AccpGetTrusteeSid(&pListOfExplicitEntries[i].Trustee,
//                                     &PolicyHandle,
//                                     &pSid1,
//                                     &needToClean);
//             if (Ret != ERROR_SUCCESS)
//                 goto Cleanup;
//
//             ObjectsPresent = AccpGetTrusteeObjects(&pListOfExplicitEntries[i].Trustee,
//                                                    &ObjectTypeGuid,
//                                                    &InheritedObjectTypeGuid);
//
//             if (ObjectsPresent == 0)
//             {
//                 /* FIXME: Call AddAccessAllowedAceEx instead! */
//                 bRet = AddAccessAllowedAce(pNew, ACL_REVISION, pListOfExplicitEntries[i].grfAccessPermissions, pSid1);
//             }
//             else
//             {
//                 /* FIXME: Call AddAccessAllowedObjectAce */
//                 DPRINT1("Object ACEs not yet supported!\n");
//                 SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//                 bRet = FALSE;
//             }
//
//             if (needToClean) LocalFree((HLOCAL)pSid1);
//             if (!bRet)
//             {
//                 Ret = GetLastError();
//                 goto Cleanup;
//             }
//         }
//     }
//
//     /* 3b) Existing allow entries */
//     /* FIXME */
//
//     *NewAcl = pNew;
//
// Cleanup:
//     if (pKeepAce && pKeepAce != KeepAceBuf)
//         LocalFree((HLOCAL)pKeepAce);
//
//     if (pNew && Ret != ERROR_SUCCESS)
//         LocalFree((HLOCAL)pNew);
//
//     if (PolicyHandle)
//         LsaClose(PolicyHandle);
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return Ret;
// }
//
//
// /**********************************************************************
//  * AccGetInheritanceSource				EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccGetInheritanceSource(LPWSTR pObjectName,
//                         SE_OBJECT_TYPE ObjectType,
//                         SECURITY_INFORMATION SecurityInfo,
//                         BOOL Container,
//                         GUID** pObjectClassGuids,
//                         DWORD GuidCount,
//                         PACL pAcl,
//                         PFN_OBJECT_MGR_FUNCTS pfnArray,
//                         PGENERIC_MAPPING pGenericMapping,
//                         PINHERITED_FROMW pInheritArray)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /**********************************************************************
//  * AccFreeIndexArray					EXPORTED
//  *
//  * @implemented
//  */
// DWORD WINAPI
// AccFreeIndexArray(PINHERITED_FROMW pInheritArray,
//                   USHORT AceCnt,
//                   PFN_OBJECT_MGR_FUNCTS pfnArray  )
// {
//     PINHERITED_FROMW pLast;
//
//     UNREFERENCED_PARAMETER(pfnArray);
//
//     pLast = pInheritArray + AceCnt;
//     while (pInheritArray != pLast)
//     {
//         if (pInheritArray->AncestorName != NULL)
//         {
//             LocalFree((HLOCAL)pInheritArray->AncestorName);
//             pInheritArray->AncestorName = NULL;
//         }
//
//         pInheritArray++;
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /**********************************************************************
//  * AccRewriteGetExplicitEntriesFromAcl			EXPORTED
//  *
//  * @implemented
//  */
// DWORD WINAPI
// AccRewriteGetExplicitEntriesFromAcl(PACL pacl,
//                                     PULONG pcCountOfExplicitEntries,
//                                     PEXPLICIT_ACCESS_W* pListOfExplicitEntries)
// {
//     PACE_HEADER AceHeader;
//     PSID Sid, SidTarget;
//     ULONG ObjectAceCount = 0;
//     POBJECTS_AND_SID ObjSid;
//     SIZE_T Size;
//     PEXPLICIT_ACCESS_W peaw;
//     DWORD LastErr, SidLen;
//     DWORD AceIndex = 0;
//     DWORD ErrorCode = ERROR_SUCCESS;
//
//     /* save the last error code */
//     LastErr = GetLastError();
//
//     if (pacl != NULL)
//     {
//         if (pacl->AceCount != 0)
//         {
//             Size = (SIZE_T)pacl->AceCount * sizeof(EXPLICIT_ACCESS_W);
//
//             /* calculate the space needed */
//             while (GetAce(pacl,
//                           AceIndex,
//                           (LPVOID*)&AceHeader))
//             {
//                 Sid = AccpGetAceSid(AceHeader);
//                 Size += GetLengthSid(Sid);
//
//                 if (AccpIsObjectAce(AceHeader))
//                     ObjectAceCount++;
//
//                 AceIndex++;
//             }
//
//             Size += ObjectAceCount * sizeof(OBJECTS_AND_SID);
//
//             ASSERT(pacl->AceCount == AceIndex);
//
//             /* allocate the array */
//             peaw = (PEXPLICIT_ACCESS_W)LocalAlloc(LMEM_FIXED,
//                                                   Size);
//             if (peaw != NULL)
//             {
//                 AceIndex = 0;
//                 ObjSid = (POBJECTS_AND_SID)(peaw + pacl->AceCount);
//                 SidTarget = (PSID)(ObjSid + ObjectAceCount);
//
//                 /* initialize the array */
//                 while (GetAce(pacl,
//                               AceIndex,
//                               (LPVOID*)&AceHeader))
//                 {
//                     Sid = AccpGetAceSid(AceHeader);
//                     SidLen = GetLengthSid(Sid);
//
//                     peaw[AceIndex].grfAccessPermissions = AccpGetAceAccessMask(AceHeader);
//                     peaw[AceIndex].grfAccessMode = AccpGetAceAccessMode(AceHeader);
//                     peaw[AceIndex].grfInheritance = AceHeader->AceFlags & VALID_INHERIT_FLAGS;
//
//                     if (CopySid(SidLen,
//                                 SidTarget,
//                                 Sid))
//                     {
//                         if (AccpIsObjectAce(AceHeader))
//                         {
//                             BuildTrusteeWithObjectsAndSid(&peaw[AceIndex].Trustee,
//                                                           ObjSid++,
//                                                           AccpGetObjectAceObjectType(AceHeader),
//                                                           AccpGetObjectAceInheritedObjectType(AceHeader),
//                                                           SidTarget);
//                         }
//                         else
//                         {
//                             BuildTrusteeWithSid(&peaw[AceIndex].Trustee,
//                                                 SidTarget);
//                         }
//
//                         SidTarget = (PSID)((ULONG_PTR)SidTarget + SidLen);
//                     }
//                     else
//                     {
//                         /* copying the SID failed, treat it as an fatal error... */
//                         ErrorCode = GetLastError();
//
//                         /* free allocated resources */
//                         LocalFree(peaw);
//                         peaw = NULL;
//                         AceIndex = 0;
//                         break;
//                     }
//
//                     AceIndex++;
//                 }
//
//                 *pcCountOfExplicitEntries = AceIndex;
//                 *pListOfExplicitEntries = peaw;
//             }
//             else
//                 ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//         }
//         else
//         {
//             goto EmptyACL;
//         }
//     }
//     else
//     {
// EmptyACL:
//         *pcCountOfExplicitEntries = 0;
//         *pListOfExplicitEntries = NULL;
//     }
//
//     /* restore the last error code */
//     SetLastError(LastErr);
//
//     return ErrorCode;
// }
//
//
// /**********************************************************************
//  * AccTreeResetNamedSecurityInfo			EXPORTED
//  *
//  * @unimplemented
//  */
// DWORD WINAPI
// AccTreeResetNamedSecurityInfo(LPWSTR pObjectName,
//                               SE_OBJECT_TYPE ObjectType,
//                               SECURITY_INFORMATION SecurityInfo,
//                               PSID pOwner,
//                               PSID pGroup,
//                               PACL pDacl,
//                               PACL pSacl,
//                               BOOL KeepExplicit,
//                               FN_PROGRESSW fnProgress,
//                               PROG_INVOKE_SETTING ProgressInvokeSetting,
//                               PVOID Args)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// BOOL WINAPI
// DllMain( HINSTANCE hinstDLL,
//          DWORD dwReason,
//          LPVOID lpvReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             hDllInstance = hinstDLL;
//             DisableThreadLibraryCalls(hinstDLL);
//             break;
//
//         case DLL_PROCESS_DETACH:
//             break;
//     }
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
