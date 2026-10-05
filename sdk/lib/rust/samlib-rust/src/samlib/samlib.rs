use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided C preamble.
// The code snippet contains only function fragments, RPC exception handling,
// and inline code blocks, but no typedef structs, #define constants,
// enums, or static tables that require translation.
#[no_mangle]
// C: `void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)`. SIZE_T, not usize — they
// happen to coincide on all three targets, but naming the alias is what keeps it true.
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn PSAMPR_SERVER_NAME_bind(pszSystemName: PSAMPR_SERVER_NAME) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    if pszSystemName != core::ptr::null_mut() {
        let nLength: i32 = wcslen(pszSystemName) as i32;
        let mut nNameLength: i32 = nLength;

        if nLength >= 1 && *pszSystemName.offset(0 as isize) == '\\' as u16 {
            nNameLength -= 1;
        }

        if nLength >= 2 && *pszSystemName.offset(1 as isize) == '\\' as u16 {
            nNameLength -= 1;
        }

        if ((nLength - nNameLength != 0) &&
            (nLength - nNameLength != 2)) ||
           (nNameLength == 0) {
            RpcRaiseException(STATUS_OBJECT_NAME_INVALID);
        }
    }

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        w!("ncacn_np") as *mut u16,
        pszSystemName,
        w!("\\pipe\\samr") as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {
    }

    hBinding
}
#[no_mangle]
pub unsafe extern "system" fn PSAMPR_SERVER_NAME_unbind(
    pszSystemName: PSAMPR_SERVER_NAME,
    hBinding: handle_t,
) {
    let mut status: RPC_STATUS;
    let mut hBinding = hBinding;

    status = RpcBindingFree(core::ptr::addr_of_mut!(hBinding));
    if status != 0 {
    }
}
#[no_mangle]
pub unsafe extern "C" fn SampCheckPassword(
    UserHandle: SAMPR_HANDLE,
    Password: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DomainPasswordInformation: USER_DOMAIN_PASSWORD_INFORMATION = core::mem::zeroed();
    let mut CharTypeBuffer: LPWORD = core::ptr::null_mut();
    let mut PasswordLength: ULONG;
    let mut i: ULONG;
    let mut Upper: ULONG = 0;
    let mut Lower: ULONG = 0;
    let mut Digit: ULONG = 0;
    let mut Punct: ULONG = 0;
    let mut Alpha: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SamrGetUserDomainPasswordInformation(
        UserHandle,
        &mut DomainPasswordInformation,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PasswordLength = ((*Password).Length / core::mem::size_of::<WCHAR>() as u16) as ULONG;

    if (PasswordLength < DomainPasswordInformation.MinPasswordLength as ULONG)
        || (PasswordLength > 256)
    {
        return STATUS_PASSWORD_RESTRICTION;
    }

    if (DomainPasswordInformation.PasswordProperties & DOMAIN_PASSWORD_COMPLEX) != 0 {
        CharTypeBuffer = MIDL_user_allocate((
            PasswordLength as usize * core::mem::size_of::<WORD>()
        ) as SIZE_T) as LPWORD;
        if CharTypeBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        GetStringTypeW(
            CT_CTYPE1,
            (*Password).Buffer,
            PasswordLength as i32,
            CharTypeBuffer,
        );

        i = 0;
        while i < PasswordLength {
            if ((*CharTypeBuffer.add((i) as usize) as u32) & C1_UPPER) != 0 {
                Upper = 1;
            }

            if ((*CharTypeBuffer.add((i) as usize) as u32) & C1_LOWER) != 0 {
                Lower = 1;
            }

            if ((*CharTypeBuffer.add((i) as usize) as u32) & C1_DIGIT) != 0 {
                Digit = 1;
            }

            if ((*CharTypeBuffer.add((i) as usize) as u32) & C1_PUNCT) != 0 {
                Punct = 1;
            }

            if (((*CharTypeBuffer.add((i) as usize) as u32) & C1_ALPHA) != 0)
                && (((*CharTypeBuffer.add((i) as usize) as u32) & C1_UPPER) == 0)
                && (((*CharTypeBuffer.add((i) as usize) as u32) & C1_LOWER) == 0)
            {
                Alpha = 1;
            }

            i += 1;
        }

        if Upper + Lower + Digit + Punct + Alpha < 3 {
            Status = STATUS_PASSWORD_RESTRICTION;
        }
    }

    if !CharTypeBuffer.is_null() {
        MIDL_user_free(CharTypeBuffer as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamAddMemberToAlias(
    AliasHandle: SAM_HANDLE,
    MemberId: PSID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrAddMemberToAlias(AliasHandle as SAMPR_HANDLE, MemberId as PRPC_SID)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamAddMemberToGroup(
    GroupHandle: SAM_HANDLE,
    MemberId: ULONG,
    Attributes: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrAddMemberToGroup(GroupHandle as SAMPR_HANDLE, MemberId, Attributes)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SamAddMultipleMembersToAlias(
    AliasHandle: SAM_HANDLE,
    MemberIds: *mut PSID,
    MemberCount: ULONG,
) -> NTSTATUS {
    let mut Buffer: SAMPR_PSID_ARRAY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if MemberIds.is_null() {
        return STATUS_INVALID_PARAMETER_2;
    }

    Buffer.Count = MemberCount;
    Buffer.Sids = MemberIds as PSAMPR_SID_INFORMATION;

    Status = rpc_try(|| SamrAddMultipleMembersToAlias(AliasHandle as SAMPR_HANDLE, &mut Buffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamChangePasswordUser(
    UserHandle: SAM_HANDLE,
    OldPassword: PUNICODE_STRING,
    NewPassword: PUNICODE_STRING,
) -> NTSTATUS {
    let mut OldNtPassword: NT_OWF_PASSWORD = core::mem::zeroed();
    let mut NewNtPassword: NT_OWF_PASSWORD = core::mem::zeroed();
    let mut OldLmPassword: LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewLmPassword: LM_OWF_PASSWORD = core::mem::zeroed();
    let mut LmPwdString: OEM_STRING = core::mem::zeroed();
    let mut LmPwdBuffer: [i8; 15] = [0; 15];
    let mut OldLmPasswordPresent: BOOLEAN = (FALSE) as u8;
    let mut NewLmPasswordPresent: BOOLEAN = (FALSE) as u8;
    let mut Status: NTSTATUS;

    let mut OldLmEncryptedWithNewLm: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewLmEncryptedWithOldLm: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut OldNtEncryptedWithNewNt: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut NewNtEncryptedWithOldNt: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut pOldLmEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();
    let mut pNewLmEncryptedWithOldLm: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();

    Status = SystemFunction007(
        OldPassword,
        &mut OldNtPassword as *mut _ as LPBYTE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SystemFunction007(
        NewPassword,
        &mut NewNtPassword as *mut _ as LPBYTE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    LmPwdString.Length = 15;
    LmPwdString.MaximumLength = 15;
    LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
    ZeroMemory!(LmPwdString.Buffer as *mut core::ffi::c_void, LmPwdString.MaximumLength as usize);

    Status = RtlUpcaseUnicodeStringToOemString(
        &mut LmPwdString,
        OldPassword,
        (FALSE) as u8,
    );
    if NT_SUCCESS(Status) {
        Status = SystemFunction006(
            LmPwdString.Buffer,
            &mut OldLmPassword as *mut _ as LPSTR,
        );
        if NT_SUCCESS(Status) {
            OldLmPasswordPresent = (TRUE) as u8;
        }
    }

    LmPwdString.Length = 15;
    LmPwdString.MaximumLength = 15;
    LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
    ZeroMemory!(LmPwdString.Buffer as *mut core::ffi::c_void, LmPwdString.MaximumLength as usize);

    Status = RtlUpcaseUnicodeStringToOemString(
        &mut LmPwdString,
        NewPassword,
        (FALSE) as u8,
    );
    if NT_SUCCESS(Status) {
        Status = SystemFunction006(
            LmPwdString.Buffer,
            &mut NewLmPassword as *mut _ as LPSTR,
        );
        if NT_SUCCESS(Status) {
            NewLmPasswordPresent = (TRUE) as u8;
        }
    }

    if OldLmPasswordPresent != 0 && NewLmPasswordPresent != 0 {
        Status = SystemFunction012(
            &OldLmPassword as *const _ as *const u8,
            &NewLmPassword as *const _ as *const u8,
            &mut OldLmEncryptedWithNewLm as *mut _ as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = SystemFunction012(
            &NewLmPassword as *const _ as *const u8,
            &OldLmPassword as *const _ as *const u8,
            &mut NewLmEncryptedWithOldLm as *mut _ as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        pOldLmEncryptedWithNewLm = &mut OldLmEncryptedWithNewLm;
        pNewLmEncryptedWithOldLm = &mut NewLmEncryptedWithOldLm;
    }

    Status = SystemFunction012(
        &OldNtPassword as *const _ as *const u8,
        &NewNtPassword as *const _ as *const u8,
        &mut OldNtEncryptedWithNewNt as *mut _ as LPBYTE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SystemFunction012(
        &NewNtPassword as *const _ as *const u8,
        &OldNtPassword as *const _ as *const u8,
        &mut NewNtEncryptedWithOldNt as *mut _ as LPBYTE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = rpc_try(|| SamrChangePasswordUser(
        UserHandle as SAMPR_HANDLE,
        (OldLmPasswordPresent != 0 && NewLmPasswordPresent != 0) as u8,
        pOldLmEncryptedWithNewLm,
        pNewLmEncryptedWithOldLm,
        (TRUE) as u8,
        &mut OldNtEncryptedWithNewNt,
        &mut NewNtEncryptedWithOldNt,
        (FALSE) as u8,
        core::ptr::null_mut(),
        (FALSE) as u8,
        core::ptr::null_mut(),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    // C: this is the body of RpcExcept(EXCEPTION_EXECUTE_HANDLER). We build with
    // -D_USE_DUMMY_PSEH=1, so it is compiled but NEVER entered. The translation
    // emitted it as straight-line code, which unconditionally clobbered the real
    // RPC status with the mapped exception code. Dropped, exactly as the other 36
    // Sam* wrappers in this TU correctly dropped theirs.

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamChangePasswordUser2(
    ServerName: PUNICODE_STRING,
    UserName: PUNICODE_STRING,
    OldPassword: PUNICODE_STRING,
    NewPassword: PUNICODE_STRING,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamChangePasswordUser3(
    ServerName: PUNICODE_STRING,
    UserName: PUNICODE_STRING,
    OldPassword: PUNICODE_STRING,
    NewPassword: PUNICODE_STRING,
    EffectivePasswordPolicy: *mut PDOMAIN_PASSWORD_INFORMATION,
    PasswordChangeFailureInfo: *mut PUSER_PWD_CHANGE_FAILURE_INFORMATION,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamCloseHandle(mut SamHandle: SAM_HANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrCloseHandle(core::ptr::addr_of_mut!(SamHandle) as *mut SAMPR_HANDLE)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SamCreateAliasInDomain(
    DomainHandle: SAM_HANDLE,
    AccountName: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    AliasHandle: PSAM_HANDLE,
    RelativeId: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    *AliasHandle = core::ptr::null_mut();
    *RelativeId = 0;

    Status = rpc_try(|| SamrCreateAliasInDomain(
        DomainHandle as SAMPR_HANDLE,
        AccountName as PRPC_UNICODE_STRING,
        DesiredAccess,
        AliasHandle as *mut SAMPR_HANDLE,
        RelativeId,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamCreateGroupInDomain(
    DomainHandle: SAM_HANDLE,
    AccountName: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    GroupHandle: PSAM_HANDLE,
    RelativeId: *mut u32,
) -> i32 {
    let mut Status: i32;

    *GroupHandle = core::ptr::null_mut();
    *RelativeId = 0;

    Status = rpc_try(|| SamrCreateGroupInDomain(
        DomainHandle as SAMPR_HANDLE,
        AccountName as PRPC_UNICODE_STRING,
        DesiredAccess,
        GroupHandle as *mut SAMPR_HANDLE,
        RelativeId,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamCreateUser2InDomain(
    DomainHandle: SAM_HANDLE,
    AccountName: PUNICODE_STRING,
    AccountType: ULONG,
    DesiredAccess: ACCESS_MASK,
    UserHandle: PSAM_HANDLE,
    GrantedAccess: PULONG,
    RelativeId: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    *UserHandle = core::ptr::null_mut();
    *RelativeId = 0;

    Status = rpc_try(|| SamrCreateUser2InDomain(
        DomainHandle as SAMPR_HANDLE,
        AccountName as PRPC_UNICODE_STRING,
        AccountType,
        DesiredAccess,
        UserHandle as *mut SAMPR_HANDLE,
        GrantedAccess,
        RelativeId,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamCreateUserInDomain(
    DomainHandle: SAM_HANDLE,
    AccountName: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    UserHandle: PSAM_HANDLE,
    RelativeId: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    *UserHandle = core::ptr::null_mut();
    *RelativeId = 0;

    Status = rpc_try(|| SamrCreateUserInDomain(
        DomainHandle as SAMPR_HANDLE,
        AccountName as PRPC_UNICODE_STRING,
        DesiredAccess,
        UserHandle as *mut SAMPR_HANDLE,
        RelativeId,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamDeleteAlias(AliasHandle: SAM_HANDLE) -> NTSTATUS {
    let mut LocalAliasHandle: SAMPR_HANDLE;
    let mut Status: NTSTATUS;

    LocalAliasHandle = AliasHandle as SAMPR_HANDLE;

    if LocalAliasHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    Status = rpc_try(|| SamrDeleteAlias(core::ptr::addr_of_mut!(LocalAliasHandle))).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamDeleteGroup(GroupHandle: SAM_HANDLE) -> NTSTATUS {
    let mut LocalGroupHandle: SAMPR_HANDLE;
    let mut Status: NTSTATUS;

    LocalGroupHandle = GroupHandle as SAMPR_HANDLE;

    if LocalGroupHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    Status = rpc_try(|| SamrDeleteGroup(core::ptr::addr_of_mut!(LocalGroupHandle))).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamDeleteUser(UserHandle: SAM_HANDLE) -> NTSTATUS {
    let mut LocalUserHandle: SAMPR_HANDLE;
    let mut Status: NTSTATUS;

    LocalUserHandle = UserHandle as SAMPR_HANDLE;

    if LocalUserHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    Status = rpc_try(|| SamrDeleteUser(core::ptr::addr_of_mut!(LocalUserHandle))).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamEnumerateAliasesInDomain(
    DomainHandle: SAM_HANDLE,
    EnumerationContext: PSAM_ENUMERATE_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if EnumerationContext.is_null() || Buffer.is_null() || CountReturned.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = rpc_try(|| SamrEnumerateAliasesInDomain(
        DomainHandle as SAMPR_HANDLE,
        EnumerationContext,
        core::ptr::addr_of_mut!(EnumBuffer),
        PreferedMaximumLength,
        CountReturned,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !EnumBuffer.is_null() {
        if !(*EnumBuffer).Buffer.is_null() {
            *Buffer = (*EnumBuffer).Buffer as PVOID;
        }

        MIDL_user_free(EnumBuffer as *mut core::ffi::c_void);
    }

    Status = I_RpcMapWin32Status(Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamEnumerateDomainsInSamServer(
    ServerHandle: SAM_HANDLE,
    EnumerationContext: PSAM_ENUMERATE_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if EnumerationContext.is_null() || Buffer.is_null() || CountReturned.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = rpc_try(|| SamrEnumerateDomainsInSamServer(
        ServerHandle as SAMPR_HANDLE,
        EnumerationContext,
        core::ptr::addr_of_mut!(EnumBuffer),
        PreferedMaximumLength,
        CountReturned,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !EnumBuffer.is_null() {
        if !(*EnumBuffer).Buffer.is_null() {
            *Buffer = (*EnumBuffer).Buffer as PVOID;
        }

        MIDL_user_free(EnumBuffer as *mut core::ffi::c_void);
    }

    Status = I_RpcMapWin32Status(Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamEnumerateGroupsInDomain(
    DomainHandle: SAM_HANDLE,
    EnumerationContext: PSAM_ENUMERATE_HANDLE,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if EnumerationContext.is_null() || Buffer.is_null() || CountReturned.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = rpc_try(|| SamrEnumerateGroupsInDomain(
        DomainHandle as SAMPR_HANDLE,
        EnumerationContext,
        core::ptr::addr_of_mut!(EnumBuffer),
        PreferedMaximumLength,
        CountReturned,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if !EnumBuffer.is_null() {
        if !(*EnumBuffer).Buffer.is_null() {
            *Buffer = (*EnumBuffer).Buffer as PVOID;
        }

        MIDL_user_free(EnumBuffer as *mut core::ffi::c_void);
    }

    Status = I_RpcMapWin32Status(Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamEnumerateUsersInDomain(
    DomainHandle: SAM_HANDLE,
    EnumerationContext: PSAM_ENUMERATE_HANDLE,
    UserAccountControl: ULONG,
    Buffer: *mut PVOID,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if EnumerationContext.is_null() || Buffer.is_null() || CountReturned.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = rpc_try(|| SamrEnumerateUsersInDomain(
        DomainHandle as SAMPR_HANDLE,
        EnumerationContext,
        UserAccountControl,
        &mut EnumBuffer,
        PreferedMaximumLength,
        CountReturned,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if !EnumBuffer.is_null() {
        if !(*EnumBuffer).Buffer.is_null() {
            *Buffer = (*EnumBuffer).Buffer as PVOID;
        }

        MIDL_user_free(EnumBuffer as *mut core::ffi::c_void);
    }

    Status = I_RpcMapWin32Status(Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamFreeMemory(Buffer: PVOID) -> NTSTATUS {
    if !Buffer.is_null() {
        MIDL_user_free(Buffer);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SamGetAliasMembership(
    DomainHandle: SAM_HANDLE,
    PassedCount: ULONG,
    Sids: *mut PSID,
    MembershipCount: PULONG,
    Aliases: *mut PULONG,
) -> NTSTATUS {
    let mut SidArray: SAMPR_PSID_ARRAY = core::mem::zeroed();
    let mut Membership: SAMPR_ULONG_ARRAY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if Sids.is_null() || MembershipCount.is_null() || Aliases.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Membership.Element = core::ptr::null_mut();

    SidArray.Count = PassedCount;
    SidArray.Sids = Sids as PSAMPR_SID_INFORMATION;

    Status = rpc_try(|| SamrGetAliasMembership(
        DomainHandle as SAMPR_HANDLE,
        &mut SidArray,
        &mut Membership,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        *MembershipCount = Membership.Count;
        *Aliases = Membership.Element;
    } else {
        if !Membership.Element.is_null() {
            MIDL_user_free(Membership.Element as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamGetCompatibilityMode(
    ObjectHandle: SAM_HANDLE,
    Mode: *mut ULONG,
) -> NTSTATUS {
    if Mode.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Mode = SAM_SID_COMPATIBILITY_ALL;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SamGetDisplayEnumerationIndex(
    DomainHandle: SAM_HANDLE,
    DisplayInformation: DOMAIN_DISPLAY_INFORMATION,
    Prefix: PUNICODE_STRING,
    Index: *mut ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if Prefix.is_null() || Index.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| SamrGetDisplayEnumerationIndex2(
        DomainHandle as SAMPR_HANDLE,
        DisplayInformation,
        Prefix as PRPC_UNICODE_STRING,
        Index,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamGetGroupsForUser(
    UserHandle: SAM_HANDLE,
    Groups: *mut PGROUP_MEMBERSHIP,
    MembershipCount: *mut ULONG,
) -> NTSTATUS {
    let mut GroupsBuffer: PSAMPR_GET_GROUPS_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrGetGroupsForUser(UserHandle as SAMPR_HANDLE, &mut GroupsBuffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        *Groups = (*GroupsBuffer).Groups;
        *MembershipCount = (*GroupsBuffer).MembershipCount;

        MIDL_user_free(GroupsBuffer as *mut core::ffi::c_void);
    } else {
        if !GroupsBuffer.is_null() {
            if !(*GroupsBuffer).Groups.is_null() {
                MIDL_user_free((*GroupsBuffer).Groups as *mut core::ffi::c_void);
            }

            MIDL_user_free(GroupsBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamGetMembersInAlias(
    AliasHandle: SAM_HANDLE,
    MemberIds: *mut *mut PSID,
    MemberCount: *mut ULONG,
) -> NTSTATUS {
    let mut SidArray: SAMPR_PSID_ARRAY_OUT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if MemberIds.is_null() || MemberCount.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *MemberIds = core::ptr::null_mut();
    *MemberCount = 0;

    SidArray.Sids = core::ptr::null_mut();

    Status = rpc_try(|| SamrGetMembersInAlias(
        AliasHandle as SAMPR_HANDLE,
        &mut SidArray,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        *MemberCount = SidArray.Count;
        *MemberIds = SidArray.Sids as *mut PSID;
    }

    // C: this is the body of RpcExcept(EXCEPTION_EXECUTE_HANDLER). We build with
    // -D_USE_DUMMY_PSEH=1, so it is compiled but NEVER entered. The translation
    // emitted it as straight-line code, which unconditionally clobbered the real
    // RPC status with the mapped exception code. Dropped, exactly as the other 36
    // Sam* wrappers in this TU correctly dropped theirs.

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamGetMembersInGroup(
    GroupHandle: SAM_HANDLE,
    MemberIds: *mut PULONG,
    Attributes: *mut PULONG,
    MemberCount: PULONG,
) -> NTSTATUS {
    let mut MembersBuffer: PSAMPR_GET_MEMBERS_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrGetMembersInGroup(GroupHandle as SAMPR_HANDLE, &mut MembersBuffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        *MemberIds = (*MembersBuffer).Members;
        *Attributes = (*MembersBuffer).Attributes;
        *MemberCount = (*MembersBuffer).MemberCount;

        MIDL_user_free(MembersBuffer as *mut core::ffi::c_void);
    } else {
        if !MembersBuffer.is_null() {
            if !(*MembersBuffer).Members.is_null() {
                MIDL_user_free((*MembersBuffer).Members as *mut core::ffi::c_void);
            }

            if !(*MembersBuffer).Attributes.is_null() {
                MIDL_user_free((*MembersBuffer).Attributes as *mut core::ffi::c_void);
            }

            MIDL_user_free(MembersBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamLookupDomainInSamServer(
    ServerHandle: SAM_HANDLE,
    Name: PUNICODE_STRING,
    DomainId: *mut PSID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrLookupDomainInSamServer(
        ServerHandle as SAMPR_HANDLE,
        Name as PRPC_UNICODE_STRING,
        DomainId as *mut PRPC_SID,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamLookupIdsInDomain(
    DomainHandle: SAM_HANDLE,
    Count: ULONG,
    RelativeIds: PULONG,
    Names: *mut PUNICODE_STRING,
    // C: `OUT PSID_NAME_USE *Use OPTIONAL` — a pointer TO a PSID_NAME_USE. The translation
    // dropped one level of indirection here (SamLookupNamesInDomain, the sibling with the
    // same parameter, got it right). It would still have linked: the argument is one pointer
    // either way, so the ABI matches and only the `*Use = ...` stores go to the wrong place.
    Use: *mut PSID_NAME_USE,
) -> NTSTATUS {
    let mut NamesBuffer = SAMPR_RETURNED_USTRING_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut UseBuffer = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    *Names = core::ptr::null_mut();

    if !Use.is_null() {
        *Use = core::ptr::null_mut();
    }

    Status = rpc_try(|| SamrLookupIdsInDomain(
        DomainHandle as SAMPR_HANDLE,
        Count,
        RelativeIds,
        core::ptr::addr_of_mut!(NamesBuffer),
        core::ptr::addr_of_mut!(UseBuffer),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if NT_SUCCESS(Status) {
        *Names = (MIDL_user_allocate((Count as usize * core::mem::size_of::<RPC_UNICODE_STRING>()) as SIZE_T) as *mut RPC_UNICODE_STRING).cast();
        if (*Names).is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'done: loop {
                if !NT_SUCCESS(Status) {
                    if !(*Names).is_null() {
                        i = 0;
                        while i < Count {
                            if !(*(*Names).add(i as usize)).Buffer.is_null() {
                                MIDL_user_free((*(*Names).add(i as usize)).Buffer as *mut core::ffi::c_void);
                            }
                            i += 1;
                        }
                        MIDL_user_free(*Names as *mut core::ffi::c_void);
                    }
                    if !Use.is_null() && !(*Use).is_null() {
                        MIDL_user_free(*Use as *mut core::ffi::c_void);
                    }
                }
                if !NamesBuffer.Element.is_null() {
                    i = 0;
                    while i < NamesBuffer.Count {
                        if !(*NamesBuffer.Element.add(i as usize)).Buffer.is_null() {
                            MIDL_user_free((*NamesBuffer.Element.add(i as usize)).Buffer as *mut core::ffi::c_void);
                        }
                        i += 1;
                    }
                    MIDL_user_free(NamesBuffer.Element as *mut core::ffi::c_void);
                }
                if !UseBuffer.Element.is_null() {
                    MIDL_user_free(UseBuffer.Element as *mut core::ffi::c_void);
                }
                return Status;
            }
        }

        i = 0;
        while i < Count {
            (*(*Names).add(i as usize)).Buffer = MIDL_user_allocate(((*NamesBuffer.Element.add(i as usize)).MaximumLength as usize) as SIZE_T) as LPWSTR;
            if (*(*Names).add(i as usize)).Buffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                'done: loop {
                    if !NT_SUCCESS(Status) {
                        if !(*Names).is_null() {
                            i = 0;
                            while i < Count {
                                if !(*(*Names).add(i as usize)).Buffer.is_null() {
                                    MIDL_user_free((*(*Names).add(i as usize)).Buffer as *mut core::ffi::c_void);
                                }
                                i += 1;
                            }
                            MIDL_user_free(*Names as *mut core::ffi::c_void);
                        }
                        if !Use.is_null() && !(*Use).is_null() {
                            MIDL_user_free(*Use as *mut core::ffi::c_void);
                        }
                    }
                    if !NamesBuffer.Element.is_null() {
                        i = 0;
                        while i < NamesBuffer.Count {
                            if !(*NamesBuffer.Element.add(i as usize)).Buffer.is_null() {
                                MIDL_user_free((*NamesBuffer.Element.add(i as usize)).Buffer as *mut core::ffi::c_void);
                            }
                            i += 1;
                        }
                        MIDL_user_free(NamesBuffer.Element as *mut core::ffi::c_void);
                    }
                    if !UseBuffer.Element.is_null() {
                        MIDL_user_free(UseBuffer.Element as *mut core::ffi::c_void);
                    }
                    return Status;
                }
            }
            i += 1;
        }

        i = 0;
        while i < Count {
            (*(*Names).add(i as usize)).Length = (*NamesBuffer.Element.add(i as usize)).Length;
            (*(*Names).add(i as usize)).MaximumLength = (*NamesBuffer.Element.add(i as usize)).MaximumLength;
            core::ptr::copy_nonoverlapping(
                (*NamesBuffer.Element.add(i as usize)).Buffer,
                (*(*Names).add(i as usize)).Buffer,
                (*NamesBuffer.Element.add(i as usize)).Length as usize / core::mem::size_of::<u16>(),
            );
            i += 1;
        }

        if !Use.is_null() {
            *Use = MIDL_user_allocate((Count as usize * core::mem::size_of::<SID_NAME_USE>()) as SIZE_T) as PSID_NAME_USE;
            if (*Use).is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                'done: loop {
                    if !NT_SUCCESS(Status) {
                        if !(*Names).is_null() {
                            i = 0;
                            while i < Count {
                                if !(*(*Names).add(i as usize)).Buffer.is_null() {
                                    MIDL_user_free((*(*Names).add(i as usize)).Buffer as *mut core::ffi::c_void);
                                }
                                i += 1;
                            }
                            MIDL_user_free(*Names as *mut core::ffi::c_void);
                        }
                        if !Use.is_null() && !(*Use).is_null() {
                            MIDL_user_free(*Use as *mut core::ffi::c_void);
                        }
                    }
                    if !NamesBuffer.Element.is_null() {
                        i = 0;
                        while i < NamesBuffer.Count {
                            if !(*NamesBuffer.Element.add(i as usize)).Buffer.is_null() {
                                MIDL_user_free((*NamesBuffer.Element.add(i as usize)).Buffer as *mut core::ffi::c_void);
                            }
                            i += 1;
                        }
                        MIDL_user_free(NamesBuffer.Element as *mut core::ffi::c_void);
                    }
                    if !UseBuffer.Element.is_null() {
                        MIDL_user_free(UseBuffer.Element as *mut core::ffi::c_void);
                    }
                    return Status;
                }
            }
            core::ptr::copy_nonoverlapping(
                UseBuffer.Element,
                *Use,
                Count as usize,
            );
        }
    }

    if !NT_SUCCESS(Status) {
        if !(*Names).is_null() {
            i = 0;
            while i < Count {
                if !(*(*Names).add(i as usize)).Buffer.is_null() {
                    MIDL_user_free((*(*Names).add(i as usize)).Buffer as *mut core::ffi::c_void);
                }
                i += 1;
            }
            MIDL_user_free(*Names as *mut core::ffi::c_void);
        }
        if !Use.is_null() && !(*Use).is_null() {
            MIDL_user_free(*Use as *mut core::ffi::c_void);
        }
    }

    if !NamesBuffer.Element.is_null() {
        i = 0;
        while i < NamesBuffer.Count {
            if !(*NamesBuffer.Element.add(i as usize)).Buffer.is_null() {
                MIDL_user_free((*NamesBuffer.Element.add(i as usize)).Buffer as *mut core::ffi::c_void);
            }
            i += 1;
        }
        MIDL_user_free(NamesBuffer.Element as *mut core::ffi::c_void);
    }

    if !UseBuffer.Element.is_null() {
        MIDL_user_free(UseBuffer.Element as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamLookupNamesInDomain(
    DomainHandle: SAM_HANDLE,
    Count: ULONG,
    Names: PUNICODE_STRING,
    RelativeIds: *mut PULONG,
    Use: *mut PSID_NAME_USE,
) -> NTSTATUS {
    let mut RidBuffer = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut UseBuffer = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    *RelativeIds = core::ptr::null_mut();
    *Use = core::ptr::null_mut();

    Status = rpc_try(|| SamrLookupNamesInDomain(
        DomainHandle as SAMPR_HANDLE,
        Count,
        Names as PRPC_UNICODE_STRING,
        core::ptr::addr_of_mut!(RidBuffer),
        core::ptr::addr_of_mut!(UseBuffer),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if NT_SUCCESS(Status) {
        *RelativeIds = MIDL_user_allocate((Count as usize * core::mem::size_of::<ULONG>()) as SIZE_T) as PULONG;
        if (*RelativeIds).is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            *Use = MIDL_user_allocate((Count as usize * core::mem::size_of::<SID_NAME_USE>()) as SIZE_T) as PSID_NAME_USE;
            if (*Use).is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                core::ptr::copy_nonoverlapping(
                    RidBuffer.Element,
                    *RelativeIds,
                    Count as usize,
                );
                core::ptr::copy_nonoverlapping(
                    UseBuffer.Element,
                    *Use,
                    Count as usize,
                );
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !(*RelativeIds).is_null() {
            MIDL_user_free(*RelativeIds as *mut core::ffi::c_void);
        }
        if !(*Use).is_null() {
            MIDL_user_free(*Use as *mut core::ffi::c_void);
        }
    }

    if !RidBuffer.Element.is_null() {
        MIDL_user_free(RidBuffer.Element as *mut core::ffi::c_void);
    }

    if !UseBuffer.Element.is_null() {
        MIDL_user_free(UseBuffer.Element as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamOpenAlias(
    DomainHandle: SAM_HANDLE,
    DesiredAccess: ACCESS_MASK,
    AliasId: ULONG,
    AliasHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrOpenAlias(
        DomainHandle as SAMPR_HANDLE,
        DesiredAccess,
        AliasId,
        AliasHandle as *mut SAMPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SamOpenDomain(
    ServerHandle: SAM_HANDLE,
    DesiredAccess: ACCESS_MASK,
    DomainId: PSID,
    DomainHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrOpenDomain(
        ServerHandle as SAMPR_HANDLE,
        DesiredAccess,
        DomainId as PRPC_SID,
        DomainHandle as *mut SAMPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamOpenGroup(
    DomainHandle: SAM_HANDLE,
    DesiredAccess: ACCESS_MASK,
    GroupId: ULONG,
    GroupHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrOpenGroup(
        DomainHandle as SAMPR_HANDLE,
        DesiredAccess,
        GroupId,
        GroupHandle as *mut SAMPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamOpenUser(
    DomainHandle: SAM_HANDLE,
    DesiredAccess: ACCESS_MASK,
    UserId: ULONG,
    UserHandle: PSAM_HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| SamrOpenUser(
        DomainHandle as SAMPR_HANDLE,
        DesiredAccess,
        UserId,
        UserHandle as *mut SAMPR_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQueryDisplayInformation(
    DomainHandle: SAM_HANDLE,
    DisplayInformation: DOMAIN_DISPLAY_INFORMATION,
    Index: ULONG,
    EntryCount: ULONG,
    PreferredMaximumLength: ULONG,
    TotalAvailable: PULONG,
    TotalReturned: PULONG,
    ReturnedEntryCount: PULONG,
    SortedBuffer: *mut PVOID,
) -> NTSTATUS {
    let mut LocalBuffer: SAMPR_DISPLAY_INFO_BUFFER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if TotalAvailable.is_null() || TotalReturned.is_null() || ReturnedEntryCount.is_null() || SortedBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| SamrQueryDisplayInformation3(
        DomainHandle as SAMPR_HANDLE,
        DisplayInformation,
        Index,
        EntryCount,
        PreferredMaximumLength,
        TotalAvailable,
        TotalReturned,
        &mut LocalBuffer,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if NT_SUCCESS(Status) {
        match DisplayInformation {
            DomainDisplayUser => {
                *ReturnedEntryCount = LocalBuffer.UserInformation.EntriesRead;
                *SortedBuffer = LocalBuffer.UserInformation.Buffer as PVOID;
            }
            DomainDisplayMachine => {
                *ReturnedEntryCount = LocalBuffer.MachineInformation.EntriesRead;
                *SortedBuffer = LocalBuffer.MachineInformation.Buffer as PVOID;
            }
            DomainDisplayGroup => {
                *ReturnedEntryCount = LocalBuffer.GroupInformation.EntriesRead;
                *SortedBuffer = LocalBuffer.GroupInformation.Buffer as PVOID;
            }
            DomainDisplayOemUser => {
                *ReturnedEntryCount = LocalBuffer.OemUserInformation.EntriesRead;
                *SortedBuffer = LocalBuffer.OemUserInformation.Buffer as PVOID;
            }
            DomainDisplayOemGroup => {
                *ReturnedEntryCount = LocalBuffer.OemGroupInformation.EntriesRead;
                *SortedBuffer = LocalBuffer.OemGroupInformation.Buffer as PVOID;
            }
            DomainDisplayServer => {}
            _ => {}
        }
    } else {
        *ReturnedEntryCount = 0;
        *SortedBuffer = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQueryInformationAlias(
    AliasHandle: SAM_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrQueryInformationAlias(
        AliasHandle as SAMPR_HANDLE,
        AliasInformationClass,
        Buffer as *mut PSAMPR_ALIAS_INFO_BUFFER,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQueryInformationDomain(
    DomainHandle: SAM_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrQueryInformationDomain(
        DomainHandle as SAMPR_HANDLE,
        DomainInformationClass,
        Buffer as *mut PSAMPR_DOMAIN_INFO_BUFFER,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQueryInformationGroup(
    GroupHandle: SAM_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrQueryInformationGroup(
        GroupHandle as SAMPR_HANDLE,
        GroupInformationClass,
        Buffer as *mut PSAMPR_GROUP_INFO_BUFFER,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQueryInformationUser(
    UserHandle: SAM_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: *mut PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrQueryInformationUser(
        UserHandle as SAMPR_HANDLE,
        UserInformationClass,
        Buffer as *mut PSAMPR_USER_INFO_BUFFER,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamQuerySecurityObject(
    ObjectHandle: SAM_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut SamSecurityDescriptor: PSAMPR_SR_SECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    *SecurityDescriptor = core::ptr::null_mut();

    Status = rpc_try(|| SamrQuerySecurityObject(
        ObjectHandle as SAMPR_HANDLE,
        SecurityInformation,
        &mut SamSecurityDescriptor,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !SamSecurityDescriptor.is_null() {
        *SecurityDescriptor = ((*SamSecurityDescriptor).SecurityDescriptor).cast();

        MIDL_user_free(SamSecurityDescriptor as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamRemoveMemberFromAlias(
    AliasHandle: SAM_HANDLE,
    MemberId: PSID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrRemoveMemberFromAlias(AliasHandle as SAMPR_HANDLE, (MemberId).cast())).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamRemoveMemberFromForeignDomain(
    DomainHandle: SAM_HANDLE,
    MemberId: PSID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrRemoveMemberFromForeignDomain(DomainHandle as SAMPR_HANDLE, (MemberId).cast())).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamRemoveMemberFromGroup(
    GroupHandle: SAM_HANDLE,
    MemberId: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrRemoveMemberFromGroup(GroupHandle as SAMPR_HANDLE, MemberId)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamRemoveMultipleMembersFromAlias(
    AliasHandle: SAM_HANDLE,
    MemberIds: *mut PSID,
    MemberCount: ULONG,
) -> NTSTATUS {
    let mut Buffer: SAMPR_PSID_ARRAY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if MemberIds.is_null() {
        return STATUS_INVALID_PARAMETER_2;
    }

    Buffer.Count = MemberCount;
    Buffer.Sids = MemberIds as PSAMPR_SID_INFORMATION;

    Status = rpc_try(|| SamrRemoveMultipleMembersFromAlias(AliasHandle as SAMPR_HANDLE, &mut Buffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamRidToSid(
    ObjectHandle: SAM_HANDLE,
    Rid: ULONG,
    Sid: *mut PSID,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamSetInformationAlias(
    AliasHandle: SAM_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrSetInformationAlias(
        AliasHandle as SAMPR_HANDLE,
        AliasInformationClass,
        (Buffer).cast(),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamSetInformationDomain(
    DomainHandle: SAM_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrSetInformationDomain(
        DomainHandle as SAMPR_HANDLE,
        DomainInformationClass,
        (Buffer).cast(),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamSetInformationGroup(
    GroupHandle: SAM_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrSetInformationGroup(
        GroupHandle as SAMPR_HANDLE,
        GroupInformationClass,
        (Buffer).cast(),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamSetInformationUser(
    UserHandle: SAM_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: PVOID,
) -> NTSTATUS {
    let mut Internal1Buffer: SAMPR_USER_INTERNAL1_INFORMATION = core::mem::zeroed();
    let mut InternalAllBuffer: USER_ALL_INFORMATION = core::mem::zeroed();
    let mut LmPwdString: OEM_STRING = core::mem::zeroed();
    let mut LmPwdBuffer: [i8; 15] = [0; 15];
    let mut Status: NTSTATUS;

    if UserInformationClass == UserSetPasswordInformation {
        let PasswordBuffer = Buffer as PSAMPR_USER_SET_PASSWORD_INFORMATION;

        Status = SampCheckPassword(
            UserHandle,
            core::ptr::addr_of_mut!((*PasswordBuffer).Password) as PUNICODE_STRING,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = SystemFunction007(
            core::ptr::addr_of_mut!((*PasswordBuffer).Password) as PUNICODE_STRING,
            core::ptr::addr_of_mut!(Internal1Buffer.EncryptedNtOwfPassword) as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Internal1Buffer.NtPasswordPresent = (TRUE) as u8;
        Internal1Buffer.LmPasswordPresent = (FALSE) as u8;

        LmPwdString.Length = 15;
        LmPwdString.MaximumLength = 15;
        LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
        ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);

        Status = RtlUpcaseUnicodeStringToOemString(
            core::ptr::addr_of_mut!(LmPwdString),
            core::ptr::addr_of_mut!((*PasswordBuffer).Password) as PUNICODE_STRING,
            (FALSE) as u8,
        );
        if NT_SUCCESS(Status) {
            Status = SystemFunction006(
                LmPwdString.Buffer,
                core::ptr::addr_of_mut!(Internal1Buffer.EncryptedLmOwfPassword) as LPSTR,
            );
            if NT_SUCCESS(Status) {
                Internal1Buffer.LmPasswordPresent = (TRUE) as u8;
            }
        }

        Internal1Buffer.PasswordExpired = (*PasswordBuffer).PasswordExpired;

        Status = rpc_try(|| SamrSetInformationUser(
            UserHandle as SAMPR_HANDLE,
            UserInternal1Information,
            (core::ptr::addr_of_mut!(Internal1Buffer) as PVOID).cast(),
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
        // C: this is the body of RpcExcept(EXCEPTION_EXECUTE_HANDLER). We build with
        // -D_USE_DUMMY_PSEH=1, so it is compiled but NEVER entered. The translation
        // emitted it as straight-line code, which unconditionally clobbered the real
        // RPC status with the mapped exception code. Dropped, exactly as the other 36
        // Sam* wrappers in this TU correctly dropped theirs.

        if !NT_SUCCESS(Status) {
        }

        return Status;
    } else if UserInformationClass == UserAllInformation {
        RtlCopyMemory!(
            core::ptr::addr_of_mut!(InternalAllBuffer) as *mut core::ffi::c_void,
            Buffer,
            core::mem::size_of::<USER_ALL_INFORMATION>(),
        );

        if (InternalAllBuffer.WhichFields & (USER_ALL_LMPASSWORDPRESENT | USER_ALL_NTPASSWORDPRESENT)) != 0 {
            if (InternalAllBuffer.WhichFields & USER_ALL_OWFPASSWORD) != 0 {
                if (InternalAllBuffer.WhichFields & USER_ALL_NTPASSWORDPRESENT) != 0 {
                    if InternalAllBuffer.NtPassword.Length != core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as u16 {
                        return STATUS_INVALID_PARAMETER;
                    }
                }

                if (InternalAllBuffer.WhichFields & USER_ALL_LMPASSWORDPRESENT) != 0 {
                    if InternalAllBuffer.LmPassword.Length != core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as u16 {
                        return STATUS_INVALID_PARAMETER;
                    }
                }
            } else {
                if (InternalAllBuffer.WhichFields & USER_ALL_LMPASSWORDPRESENT) != 0 {
                    return STATUS_INVALID_PARAMETER;
                }

                if (InternalAllBuffer.WhichFields & USER_ALL_NTPASSWORDPRESENT) != 0 {
                    Status = SampCheckPassword(
                        UserHandle,
                        core::ptr::addr_of_mut!(InternalAllBuffer.NtPassword),
                    );
                    if !NT_SUCCESS(Status) {
                        return Status;
                    }

                    Status = SystemFunction007(
                        core::ptr::addr_of_mut!(InternalAllBuffer.NtPassword) as PUNICODE_STRING,
                        core::ptr::addr_of_mut!(Internal1Buffer.EncryptedNtOwfPassword) as LPBYTE,
                    );
                    if !NT_SUCCESS(Status) {
                        return Status;
                    }

                    InternalAllBuffer.NtPasswordPresent = (TRUE) as u8;
                    InternalAllBuffer.LmPasswordPresent = (FALSE) as u8;

                    InternalAllBuffer.NtPassword.Length = core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as u16;
                    InternalAllBuffer.NtPassword.MaximumLength = core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as u16;
                    InternalAllBuffer.NtPassword.Buffer = core::ptr::addr_of_mut!(Internal1Buffer.EncryptedNtOwfPassword) as LPWSTR;

                    LmPwdString.Length = 15;
                    LmPwdString.MaximumLength = 15;
                    LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
                    ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);

                    Status = RtlUpcaseUnicodeStringToOemString(
                        core::ptr::addr_of_mut!(LmPwdString),
                        core::ptr::addr_of_mut!(InternalAllBuffer.NtPassword) as PUNICODE_STRING,
                        (FALSE) as u8,
                    );
                    if NT_SUCCESS(Status) {
                        Status = SystemFunction006(
                            LmPwdString.Buffer,
                            core::ptr::addr_of_mut!(Internal1Buffer.EncryptedLmOwfPassword) as LPSTR,
                        );
                        if NT_SUCCESS(Status) {
                            InternalAllBuffer.WhichFields |= USER_ALL_LMPASSWORDPRESENT;
                            InternalAllBuffer.LmPasswordPresent = (TRUE) as u8;

                            InternalAllBuffer.LmPassword.Length = core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as u16;
                            InternalAllBuffer.LmPassword.MaximumLength = core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as u16;
                            InternalAllBuffer.LmPassword.Buffer = core::ptr::addr_of_mut!(Internal1Buffer.EncryptedLmOwfPassword) as LPWSTR;
                        }
                    }
                }
            }
        }

        Status = rpc_try(|| SamrSetInformationUser(
            UserHandle as SAMPR_HANDLE,
            UserAllInformation,
            (core::ptr::addr_of_mut!(InternalAllBuffer) as PVOID).cast(),
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
        // C: this is the body of RpcExcept(EXCEPTION_EXECUTE_HANDLER). We build with
        // -D_USE_DUMMY_PSEH=1, so it is compiled but NEVER entered. The translation
        // emitted it as straight-line code, which unconditionally clobbered the real
        // RPC status with the mapped exception code. Dropped, exactly as the other 36
        // Sam* wrappers in this TU correctly dropped theirs.

        if !NT_SUCCESS(Status) {
        }

        return Status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SamSetMemberAttributesOfGroup(
    GroupHandle: SAM_HANDLE,
    MemberId: ULONG,
    Attributes: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrSetMemberAttributesOfGroup(GroupHandle as SAMPR_HANDLE, MemberId, Attributes)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamSetSecurityObject(
    ObjectHandle: SAM_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut DescriptorToPass: SAMPR_SR_SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = 0;
    Status = RtlMakeSelfRelativeSD(
        SecurityDescriptor,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Length),
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        return STATUS_INVALID_PARAMETER;
    }

    DescriptorToPass.Length = Length;
    DescriptorToPass.SecurityDescriptor = (MIDL_user_allocate(((Length) as usize) as SIZE_T)).cast();
    if DescriptorToPass.SecurityDescriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = RtlMakeSelfRelativeSD(
        SecurityDescriptor,
        DescriptorToPass.SecurityDescriptor as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(Length),
    );
    if !NT_SUCCESS(Status) {
        if !DescriptorToPass.SecurityDescriptor.is_null() {
            MIDL_user_free((DescriptorToPass.SecurityDescriptor).cast());
        }
        return Status;
    }

    Status = rpc_try(|| SamrSetSecurityObject(
        ObjectHandle as SAMPR_HANDLE,
        SecurityInformation,
        core::ptr::addr_of_mut!(DescriptorToPass),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !DescriptorToPass.SecurityDescriptor.is_null() {
        MIDL_user_free((DescriptorToPass.SecurityDescriptor).cast());
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamShutdownSamServer(ServerHandle: SAM_HANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = rpc_try(|| SamrShutdownSamServer(ServerHandle as SAMPR_HANDLE)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamTestPrivateFunctionsDomain(
    DomainHandle: SAM_HANDLE,
) -> NTSTATUS {
    // TRACE!("SamTestPrivateFunctionsDomain(%p)\n", DomainHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamTestPrivateFunctionsUser(UserHandle: SAM_HANDLE) -> NTSTATUS {
    // TRACE!("SamTestPrivateFunctionsUser(%p)\n", UserHandle);
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS system libraries
//  * PURPOSE:           SAM interface library
//  * FILE:              lib/samlib/samlib.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// #include "precomp.h"
//
// #define NTOS_MODE_USER
// #include <ndk/rtlfuncs.h>
// #include <ntsam.h>
// #include <sam_c.h>
//
// #include <wine/debug.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(samlib);
//
// NTSTATUS
// WINAPI
// SystemFunction006(LPCSTR password,
//                   LPSTR hash);
//
// NTSTATUS
// WINAPI
// SystemFunction007(PUNICODE_STRING string,
//                   LPBYTE hash);
//
// NTSTATUS
// WINAPI
// SystemFunction012(const BYTE *in,
//                   const BYTE *key,
//                   LPBYTE out);
//
// /* GLOBALS *******************************************************************/
//
//
// /* FUNCTIONS *****************************************************************/
//
// void __RPC_FAR * __RPC_USER MIDL_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER MIDL_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// handle_t __RPC_USER
// PSAMPR_SERVER_NAME_bind(PSAMPR_SERVER_NAME pszSystemName)
// {
//     handle_t hBinding = NULL;
//     LPWSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("PSAMPR_SERVER_NAME_bind(%S)\n", pszSystemName);
//
//     /* Check the server name prefix and server name length */
//     if (pszSystemName != NULL)
//     {
//         int nLength = wcslen(pszSystemName);
//         int nNameLength = nLength;
//
//         if (nLength >= 1 && pszSystemName[0] == '\\')
//             nNameLength--;
//
//         if (nLength >= 2 && pszSystemName[1] == '\\')
//             nNameLength--;
//
//         if (((nLength - nNameLength != 0) &&
//              (nLength - nNameLength != 2)) ||
//             (nNameLength == 0))
//         {
//             WARN("Invalid server name %S\n", pszSystemName);
//             RpcRaiseException(STATUS_OBJECT_NAME_INVALID);
//         }
//     }
//
//     status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       pszSystemName,
//                                       L"\\pipe\\samr",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status)
//     {
//         TRACE("RpcStringBindingCompose returned 0x%x\n", status);
//         return NULL;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hBinding);
//     if (status)
//     {
//         TRACE("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeW(&pszStringBinding);
//     if (status)
//     {
// //        TRACE("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return hBinding;
// }
//
//
// void __RPC_USER
// PSAMPR_SERVER_NAME_unbind(PSAMPR_SERVER_NAME pszSystemName,
//                           handle_t hBinding)
// {
//     RPC_STATUS status;
//
//     TRACE("PSAMPR_SERVER_NAME_unbind(%S)\n", pszSystemName);
//
//     status = RpcBindingFree(&hBinding);
//     if (status)
//     {
//         TRACE("RpcBindingFree returned 0x%x\n", status);
//     }
// }
//
//
// NTSTATUS
// SampCheckPassword( SAMPR_HANDLE UserHandle,
//                    PUNICODE_STRING Password)
// {
//     USER_DOMAIN_PASSWORD_INFORMATION DomainPasswordInformation;
//     LPWORD CharTypeBuffer = NULL;
//     ULONG PasswordLength;
//     ULONG i;
//     ULONG Upper = 0, Lower = 0, Digit = 0, Punct = 0, Alpha = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("SampCheckPassword(%p %p)\n", UserHandle, Password);
//
//     /* Get the domain password information */
//     Status = SamrGetUserDomainPasswordInformation(UserHandle,
//                                                   &DomainPasswordInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrGetUserDomainPasswordInformation failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     PasswordLength = (ULONG)(Password->Length / sizeof(WCHAR));
//
//     /* Fail if the password is too short or too long */
//     if ((PasswordLength < DomainPasswordInformation.MinPasswordLength) ||
//         (PasswordLength > 256))
//         return STATUS_PASSWORD_RESTRICTION;
//
//     /* Check the password complexity */
//     if (DomainPasswordInformation.PasswordProperties & DOMAIN_PASSWORD_COMPLEX)
//     {
//         CharTypeBuffer = MIDL_user_allocate((PasswordLength * sizeof(WORD)) as SIZE_T);
//         if (CharTypeBuffer == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         GetStringTypeW(CT_CTYPE1,
//                        Password->Buffer,
//                        PasswordLength,
//                        CharTypeBuffer);
//
//         for (i = 0; i < PasswordLength; i++)
//         {
//             TRACE("%lu: %C %s %s %s %s\n", i, Password->Buffer[i],
//                   (CharTypeBuffer[i] & C1_UPPER) ? "C1_UPPER" : "        ",
//                   (CharTypeBuffer[i] & C1_LOWER) ? "C1_LOWER" : "        ",
//                   (CharTypeBuffer[i] & C1_DIGIT) ? "C1_DIGIT" : "        ",
//                   (CharTypeBuffer[i] & C1_PUNCT) ? "C1_PUNCT" : "        ",
//                   (CharTypeBuffer[i] & C1_ALPHA) ? "C1_ALPHA" : "        ");
//
//             if (CharTypeBuffer[i] & C1_UPPER)
//                 Upper = 1;
//
//             if (CharTypeBuffer[i] & C1_LOWER)
//                 Lower = 1;
//
//             if (CharTypeBuffer[i] & C1_DIGIT)
//                 Digit = 1;
//
//             if (CharTypeBuffer[i] & C1_PUNCT)
//                 Punct = 1;
//
//             if ((CharTypeBuffer[i] & C1_ALPHA) &&
//                 !(CharTypeBuffer[i] & C1_UPPER) &&
//                 !(CharTypeBuffer[i] & C1_LOWER))
//                 Alpha = 1;
//         }
//
//         TRACE("Upper: %lu\n", Upper);
//         TRACE("Lower: %lu\n", Lower);
//         TRACE("Digit: %lu\n", Digit);
//         TRACE("Punct: %lu\n", Punct);
//         TRACE("Alpha: %lu\n", Alpha);
//
//         TRACE("Total: %lu\n", Upper + Lower + Digit + Punct + Alpha);
//         if (Upper + Lower + Digit + Punct + Alpha < 3)
//             Status = STATUS_PASSWORD_RESTRICTION;
//     }
//
//     if (CharTypeBuffer != NULL)
//         MIDL_user_free(CharTypeBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamAddMemberToAlias( SAM_HANDLE AliasHandle,
//                      PSID MemberId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamAddMemberToAlias(%p %p)\n",
//           AliasHandle, MemberId);
//
//     RpcTryExcept
//     {
//         Status = SamrAddMemberToAlias((SAMPR_HANDLE)AliasHandle,
//                                       (PRPC_SID)MemberId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamAddMemberToGroup( SAM_HANDLE GroupHandle,
//                      ULONG MemberId,
//                      ULONG Attributes)
// {
//     NTSTATUS Status;
//
//     TRACE("SamAddMemberToGroup(%p %lu %lx)\n",
//           GroupHandle, MemberId, Attributes);
//
//     RpcTryExcept
//     {
//         Status = SamrAddMemberToGroup((SAMPR_HANDLE)GroupHandle,
//                                       MemberId,
//                                       Attributes);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamAddMultipleMembersToAlias( SAM_HANDLE AliasHandle,
//                               PSID *MemberIds,
//                               ULONG MemberCount)
// {
//     SAMPR_PSID_ARRAY Buffer;
//     NTSTATUS Status;
//
//     TRACE("SamAddMultipleMembersToAlias(%p %p %lu)\n",
//           AliasHandle, MemberIds, MemberCount);
//
//     if (MemberIds == NULL)
//         return STATUS_INVALID_PARAMETER_2;
//
//     Buffer.Count = MemberCount;
//     Buffer.Sids = (PSAMPR_SID_INFORMATION)MemberIds;
//
//     RpcTryExcept
//     {
//         Status = SamrAddMultipleMembersToAlias((SAMPR_HANDLE)AliasHandle,
//                                                &Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamChangePasswordUser( SAM_HANDLE UserHandle,
//                        PUNICODE_STRING OldPassword,
//                        PUNICODE_STRING NewPassword)
// {
//     NT_OWF_PASSWORD OldNtPassword;
//     NT_OWF_PASSWORD NewNtPassword;
//     LM_OWF_PASSWORD OldLmPassword;
//     LM_OWF_PASSWORD NewLmPassword;
//     OEM_STRING LmPwdString;
//     CHAR LmPwdBuffer[15];
//     BOOLEAN OldLmPasswordPresent = FALSE;
//     BOOLEAN NewLmPasswordPresent = FALSE;
//     NTSTATUS Status;
//
//     ENCRYPTED_LM_OWF_PASSWORD OldLmEncryptedWithNewLm;
//     ENCRYPTED_LM_OWF_PASSWORD NewLmEncryptedWithOldLm;
//     ENCRYPTED_NT_OWF_PASSWORD OldNtEncryptedWithNewNt;
//     ENCRYPTED_NT_OWF_PASSWORD NewNtEncryptedWithOldNt;
//     PENCRYPTED_LM_OWF_PASSWORD pOldLmEncryptedWithNewLm = NULL;
//     PENCRYPTED_LM_OWF_PASSWORD pNewLmEncryptedWithOldLm = NULL;
//
//     /* Calculate the NT hash for the old password */
//     Status = SystemFunction007(OldPassword,
//                                (LPBYTE)&OldNtPassword);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Calculate the NT hash for the new password */
//     Status = SystemFunction007(NewPassword,
//                                (LPBYTE)&NewNtPassword);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Calculate the LM password and hash for the old password */
//     LmPwdString.Length = 15;
//     LmPwdString.MaximumLength = 15;
//     LmPwdString.Buffer = LmPwdBuffer;
//     ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//     Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                OldPassword,
//                                                FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         /* Calculate the LM hash value of the password */
//         Status = SystemFunction006(LmPwdString.Buffer,
//                                    (LPSTR)&OldLmPassword);
//         if (NT_SUCCESS(Status))
//         {
//             OldLmPasswordPresent = TRUE;
//         }
//     }
//
//     /* Calculate the LM password and hash for the new password */
//     LmPwdString.Length = 15;
//     LmPwdString.MaximumLength = 15;
//     LmPwdString.Buffer = LmPwdBuffer;
//     ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//     Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                NewPassword,
//                                                FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         /* Calculate the LM hash value of the password */
//         Status = SystemFunction006(LmPwdString.Buffer,
//                                    (LPSTR)&NewLmPassword);
//         if (NT_SUCCESS(Status))
//         {
//             NewLmPasswordPresent = TRUE;
//         }
//     }
//
//     if (OldLmPasswordPresent && NewLmPasswordPresent)
//     {
//         /* Encrypt the old LM hash with the new LM hash */
//         Status = SystemFunction012((const BYTE *)&OldLmPassword,
//                                    (const BYTE *)&NewLmPassword,
//                                    (LPBYTE)&OldLmEncryptedWithNewLm);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         /* Encrypt the new LM hash with the old LM hash */
//         Status = SystemFunction012((const BYTE *)&NewLmPassword,
//                                    (const BYTE *)&OldLmPassword,
//                                    (LPBYTE)&NewLmEncryptedWithOldLm);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         pOldLmEncryptedWithNewLm = &OldLmEncryptedWithNewLm;
//         pNewLmEncryptedWithOldLm = &NewLmEncryptedWithOldLm;
//     }
//
//     /* Encrypt the old NT hash with the new NT hash */
//     Status = SystemFunction012((const BYTE *)&OldNtPassword,
//                                (const BYTE *)&NewNtPassword,
//                                (LPBYTE)&OldNtEncryptedWithNewNt);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Encrypt the new NT hash with the old NT hash */
//     Status = SystemFunction012((const BYTE *)&NewNtPassword,
//                                (const BYTE *)&OldNtPassword,
//                                (LPBYTE)&NewNtEncryptedWithOldNt);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     RpcTryExcept
//     {
//         Status = SamrChangePasswordUser((SAMPR_HANDLE)UserHandle,
//                                         OldLmPasswordPresent && NewLmPasswordPresent,
//                                         pOldLmEncryptedWithNewLm,
//                                         pNewLmEncryptedWithOldLm,
//                                         TRUE,
//                                         &OldNtEncryptedWithNewNt,
//                                         &NewNtEncryptedWithOldNt,
//                                         FALSE,
//                                         NULL,
//                                         FALSE,
//                                         NULL);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamChangePasswordUser2( PUNICODE_STRING ServerName,
//                         PUNICODE_STRING UserName,
//                         PUNICODE_STRING OldPassword,
//                         PUNICODE_STRING NewPassword)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// SamChangePasswordUser3( PUNICODE_STRING ServerName,
//                         PUNICODE_STRING UserName,
//                         PUNICODE_STRING OldPassword,
//                         PUNICODE_STRING NewPassword,
//                         PDOMAIN_PASSWORD_INFORMATION *EffectivePasswordPolicy,
//                         PUSER_PWD_CHANGE_FAILURE_INFORMATION *PasswordChangeFailureInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// SamCloseHandle( SAM_HANDLE SamHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamCloseHandle(%p)\n", SamHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrCloseHandle((SAMPR_HANDLE *)&SamHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamConnect(  PUNICODE_STRING ServerName ,
//             PSAM_HANDLE ServerHandle,
//             ACCESS_MASK DesiredAccess,
//             POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     PSAMPR_SERVER_NAME pServerName = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamConnect(%p %p 0x%08x %p)\n",
//           ServerName, ServerHandle, DesiredAccess, ObjectAttributes);
//
//     if (ServerName != NULL && ServerName->Length > 0 && ServerName->Buffer != NULL)
//     {
//         /* Create a zero-terminated server name */
//         pServerName = MIDL_user_allocate((ServerName->Length + sizeof(WCHAR)) as SIZE_T);
//         if (pServerName == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         CopyMemory!(pServerName, ServerName->Buffer, ServerName->Length);
//         pServerName[ServerName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     RpcTryExcept
//     {
//         Status = SamrConnect(pServerName,
//                              (SAMPR_HANDLE *)ServerHandle,
//                              DesiredAccess);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (pServerName)
//         MIDL_user_free(pServerName);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamCreateAliasInDomain( SAM_HANDLE DomainHandle,
//                         PUNICODE_STRING AccountName,
//                         ACCESS_MASK DesiredAccess,
//                         PSAM_HANDLE AliasHandle,
//                         PULONG RelativeId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamCreateAliasInDomain(%p %p 0x%08x %p %p)\n",
//           DomainHandle, AccountName, DesiredAccess, AliasHandle, RelativeId);
//
//     *AliasHandle = NULL;
//     *RelativeId = 0;
//
//     RpcTryExcept
//     {
//         Status = SamrCreateAliasInDomain((SAMPR_HANDLE)DomainHandle,
//                                          (PRPC_UNICODE_STRING)AccountName,
//                                          DesiredAccess,
//                                          (SAMPR_HANDLE *)AliasHandle,
//                                          RelativeId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamCreateGroupInDomain( SAM_HANDLE DomainHandle,
//                         PUNICODE_STRING AccountName,
//                         ACCESS_MASK DesiredAccess,
//                         PSAM_HANDLE GroupHandle,
//                         PULONG RelativeId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamCreateGroupInDomain(%p %p 0x%08x %p %p)\n",
//           DomainHandle, AccountName, DesiredAccess, GroupHandle, RelativeId);
//
//     *GroupHandle = NULL;
//     *RelativeId = 0;
//
//     RpcTryExcept
//     {
//         Status = SamrCreateGroupInDomain((SAMPR_HANDLE)DomainHandle,
//                                          (PRPC_UNICODE_STRING)AccountName,
//                                          DesiredAccess,
//                                          (SAMPR_HANDLE *)GroupHandle,
//                                          RelativeId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamCreateUser2InDomain( SAM_HANDLE DomainHandle,
//                         PUNICODE_STRING AccountName,
//                         ULONG AccountType,
//                         ACCESS_MASK DesiredAccess,
//                         PSAM_HANDLE UserHandle,
//                         PULONG GrantedAccess,
//                         PULONG RelativeId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamCreateUser2InDomain(%p %p %lu 0x%08x %p %p %p)\n",
//           DomainHandle, AccountName, AccountType, DesiredAccess,
//           UserHandle, GrantedAccess, RelativeId);
//
//     *UserHandle = NULL;
//     *RelativeId = 0;
//
//     RpcTryExcept
//     {
//         Status = SamrCreateUser2InDomain((SAMPR_HANDLE)DomainHandle,
//                                          (PRPC_UNICODE_STRING)AccountName,
//                                          AccountType,
//                                          DesiredAccess,
//                                          (SAMPR_HANDLE *)UserHandle,
//                                          GrantedAccess,
//                                          RelativeId);
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamCreateUserInDomain( SAM_HANDLE DomainHandle,
//                        PUNICODE_STRING AccountName,
//                        ACCESS_MASK DesiredAccess,
//                        PSAM_HANDLE UserHandle,
//                        PULONG RelativeId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamCreateUserInDomain(%p %p 0x%08x %p %p)\n",
//           DomainHandle, AccountName, DesiredAccess, UserHandle, RelativeId);
//
//     *UserHandle = NULL;
//     *RelativeId = 0;
//
//     RpcTryExcept
//     {
//         Status = SamrCreateUserInDomain((SAMPR_HANDLE)DomainHandle,
//                                         (PRPC_UNICODE_STRING)AccountName,
//                                         DesiredAccess,
//                                         (SAMPR_HANDLE *)UserHandle,
//                                         RelativeId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamDeleteAlias( SAM_HANDLE AliasHandle)
// {
//     SAMPR_HANDLE LocalAliasHandle;
//     NTSTATUS Status;
//
//     TRACE("SamDeleteAlias(%p)\n", AliasHandle);
//
//     LocalAliasHandle = (SAMPR_HANDLE)AliasHandle;
//
//     if (LocalAliasHandle == NULL)
//         return STATUS_INVALID_HANDLE;
//
//     RpcTryExcept
//     {
//         Status = SamrDeleteAlias(&LocalAliasHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamDeleteGroup( SAM_HANDLE GroupHandle)
// {
//     SAMPR_HANDLE LocalGroupHandle;
//     NTSTATUS Status;
//
//     TRACE("SamDeleteGroup(%p)\n", GroupHandle);
//
//     LocalGroupHandle = (SAMPR_HANDLE)GroupHandle;
//
//     if (LocalGroupHandle == NULL)
//         return STATUS_INVALID_HANDLE;
//
//     RpcTryExcept
//     {
//         Status = SamrDeleteGroup(&LocalGroupHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamDeleteUser( SAM_HANDLE UserHandle)
// {
//     SAMPR_HANDLE LocalUserHandle;
//     NTSTATUS Status;
//
//     TRACE("SamDeleteUser(%p)\n", UserHandle);
//
//     LocalUserHandle = (SAMPR_HANDLE)UserHandle;
//
//     if (LocalUserHandle == NULL)
//         return STATUS_INVALID_HANDLE;
//
//     RpcTryExcept
//     {
//         Status = SamrDeleteUser(&LocalUserHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamEnumerateAliasesInDomain( SAM_HANDLE DomainHandle,
//                               PSAM_ENUMERATE_HANDLE EnumerationContext,
//                              PVOID *Buffer,
//                              ULONG PreferedMaximumLength,
//                              PULONG CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamEnumerateAliasesInDomain(%p %p %p %lu %p)\n",
//           DomainHandle, EnumerationContext, Buffer, PreferedMaximumLength,
//           CountReturned);
//
//     if ((EnumerationContext == NULL) ||
//         (Buffer == NULL) ||
//         (CountReturned == NULL))
//         return STATUS_INVALID_PARAMETER;
//
//     *Buffer = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrEnumerateAliasesInDomain((SAMPR_HANDLE)DomainHandle,
//                                               EnumerationContext,
//                                               &EnumBuffer,
//                                               PreferedMaximumLength,
//                                               CountReturned);
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 *Buffer = EnumBuffer->Buffer;
//             }
//
//             MIDL_user_free(EnumBuffer);
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamEnumerateDomainsInSamServer( SAM_HANDLE ServerHandle,
//                                  PSAM_ENUMERATE_HANDLE EnumerationContext,
//                                 PVOID *Buffer,
//                                 ULONG PreferedMaximumLength,
//                                 PULONG CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamEnumerateDomainsInSamServer(%p %p %p %lu %p)\n",
//           ServerHandle, EnumerationContext, Buffer, PreferedMaximumLength,
//           CountReturned);
//
//     if ((EnumerationContext == NULL) ||
//         (Buffer == NULL) ||
//         (CountReturned == NULL))
//         return STATUS_INVALID_PARAMETER;
//
//     *Buffer = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrEnumerateDomainsInSamServer((SAMPR_HANDLE)ServerHandle,
//                                                  EnumerationContext,
//                                                  &EnumBuffer,
//                                                  PreferedMaximumLength,
//                                                  CountReturned);
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 *Buffer = EnumBuffer->Buffer;
//             }
//
//             MIDL_user_free(EnumBuffer);
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamEnumerateGroupsInDomain( SAM_HANDLE DomainHandle,
//                              PSAM_ENUMERATE_HANDLE EnumerationContext,
//                             PVOID *Buffer,
//                             ULONG PreferedMaximumLength,
//                             PULONG CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamEnumerateGroupsInDomain(%p %p %p %lu %p)\n",
//           DomainHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     if (EnumerationContext == NULL || Buffer == NULL || CountReturned == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *Buffer = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrEnumerateGroupsInDomain((SAMPR_HANDLE)DomainHandle,
//                                              EnumerationContext,
//                                              &EnumBuffer,
//                                              PreferedMaximumLength,
//                                              CountReturned);
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//                 *Buffer = EnumBuffer->Buffer;
//
//             MIDL_user_free(EnumBuffer);
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamEnumerateUsersInDomain( SAM_HANDLE DomainHandle,
//                             PSAM_ENUMERATE_HANDLE EnumerationContext,
//                            ULONG UserAccountControl,
//                            PVOID *Buffer,
//                            ULONG PreferedMaximumLength,
//                            PULONG CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamEnumerateUsersInDomain(%p %p %lx %p %lu %p)\n",
//           DomainHandle, EnumerationContext, UserAccountControl, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     if (EnumerationContext == NULL || Buffer == NULL || CountReturned == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *Buffer = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrEnumerateUsersInDomain((SAMPR_HANDLE)DomainHandle,
//                                             EnumerationContext,
//                                             UserAccountControl,
//                                             &EnumBuffer,
//                                             PreferedMaximumLength,
//                                             CountReturned);
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 *Buffer = EnumBuffer->Buffer;
//             }
//
//             MIDL_user_free(EnumBuffer);
//         }
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamFreeMemory( PVOID Buffer)
// {
//     if (Buffer != NULL)
//         MIDL_user_free(Buffer);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetAliasMembership( SAM_HANDLE DomainHandle,
//                        ULONG PassedCount,
//                        PSID *Sids,
//                        PULONG MembershipCount,
//                        PULONG *Aliases)
// {
//     SAMPR_PSID_ARRAY SidArray;
//     SAMPR_ULONG_ARRAY Membership;
//     NTSTATUS Status;
//
//     TRACE("SamAliasMembership(%p %lu %p %p %p)\n",
//           DomainHandle, PassedCount, Sids, MembershipCount, Aliases);
//
//     if (Sids == NULL ||
//         MembershipCount == NULL ||
//         Aliases == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     Membership.Element = NULL;
//
//     RpcTryExcept
//     {
//         SidArray.Count = PassedCount;
//         SidArray.Sids = (PSAMPR_SID_INFORMATION)Sids;
//
//         Status = SamrGetAliasMembership((SAMPR_HANDLE)DomainHandle,
//                                         &SidArray,
//                                         &Membership);
//         if (NT_SUCCESS(Status))
//         {
//             *MembershipCount = Membership.Count;
//             *Aliases = Membership.Element;
//         }
//         else
//         {
//             if (Membership.Element != NULL)
//                 MIDL_user_free(Membership.Element);
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetCompatibilityMode( SAM_HANDLE ObjectHandle,
//                          PULONG Mode)
// {
//     TRACE("SamGetCompatibilityMode(%p %p)\n", ObjectHandle, Mode);
//
//     if (Mode == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *Mode = SAM_SID_COMPATIBILITY_ALL;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetDisplayEnumerationIndex( SAM_HANDLE DomainHandle,
//                                DOMAIN_DISPLAY_INFORMATION DisplayInformation,
//                                PUNICODE_STRING Prefix,
//                                PULONG Index)
// {
//     NTSTATUS Status;
//
//     TRACE("SamGetDisplayEnumerationIndex(%p %lu %wZ %p)\n",
//            DomainHandle, DisplayInformation, Prefix, Index);
//
//     if ((Prefix == NULL) ||
//         (Index == NULL))
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = SamrGetDisplayEnumerationIndex2((SAMPR_HANDLE)DomainHandle,
//                                                  DisplayInformation,
//                                                  (PRPC_UNICODE_STRING)Prefix,
//                                                  Index);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetGroupsForUser( SAM_HANDLE UserHandle,
//                      PGROUP_MEMBERSHIP *Groups,
//                      PULONG MembershipCount)
// {
//     PSAMPR_GET_GROUPS_BUFFER GroupsBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamGetGroupsForUser(%p %p %p)\n",
//           UserHandle, Groups, MembershipCount);
//
//     RpcTryExcept
//     {
//         Status = SamrGetGroupsForUser((SAMPR_HANDLE)UserHandle,
//                                       &GroupsBuffer);
//         if (NT_SUCCESS(Status))
//         {
//             *Groups = GroupsBuffer->Groups;
//             *MembershipCount = GroupsBuffer->MembershipCount;
//
//             MIDL_user_free(GroupsBuffer);
//         }
//         else
//         {
//             if (GroupsBuffer != NULL)
//             {
//                 if (GroupsBuffer->Groups != NULL)
//                     MIDL_user_free(GroupsBuffer->Groups);
//
//                 MIDL_user_free(GroupsBuffer);
//             }
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetMembersInAlias( SAM_HANDLE AliasHandle,
//                       PSID **MemberIds,
//                       PULONG MemberCount)
// {
//     SAMPR_PSID_ARRAY_OUT SidArray;
//     NTSTATUS Status;
//
//     TRACE("SamGetMembersInAlias(%p %p %p)\n",
//           AliasHandle, MemberIds, MemberCount);
//
//     if ((MemberIds == NULL) ||
//         (MemberCount == NULL))
//         return STATUS_INVALID_PARAMETER;
//
//     *MemberIds = NULL;
//     *MemberCount = 0;
//
//     SidArray.Sids = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrGetMembersInAlias((SAMPR_HANDLE)AliasHandle,
//                                        &SidArray);
//         if (NT_SUCCESS(Status))
//         {
//             *MemberCount = SidArray.Count;
//             *MemberIds = (PSID *)SidArray.Sids;
//         }
//
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamGetMembersInGroup( SAM_HANDLE GroupHandle,
//                       PULONG *MemberIds,
//                       PULONG *Attributes,
//                       PULONG MemberCount)
// {
//     PSAMPR_GET_MEMBERS_BUFFER MembersBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamGetMembersInGroup(%p %p %p %p)\n",
//           GroupHandle, MemberIds, Attributes, MemberCount);
//
//     RpcTryExcept
//     {
//         Status = SamrGetMembersInGroup((SAMPR_HANDLE)GroupHandle,
//                                        &MembersBuffer);
//         if (NT_SUCCESS(Status))
//         {
//             *MemberIds = MembersBuffer->Members;
//             *Attributes = MembersBuffer->Attributes;
//             *MemberCount = MembersBuffer->MemberCount;
//
//             MIDL_user_free(MembersBuffer);
//         }
//         else
//         {
//             if (MembersBuffer != NULL)
//             {
//                 if (MembersBuffer->Members != NULL)
//                     MIDL_user_free(MembersBuffer->Members);
//
//                 if (MembersBuffer->Attributes != NULL)
//                     MIDL_user_free(MembersBuffer->Attributes);
//
//                 MIDL_user_free(MembersBuffer);
//             }
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamLookupDomainInSamServer( SAM_HANDLE ServerHandle,
//                             PUNICODE_STRING Name,
//                             PSID *DomainId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamLookupDomainInSamServer(%p %p %p)\n",
//           ServerHandle, Name, DomainId);
//
//     RpcTryExcept
//     {
//         Status = SamrLookupDomainInSamServer((SAMPR_HANDLE)ServerHandle,
//                                              (PRPC_UNICODE_STRING)Name,
//                                              (PRPC_SID *)DomainId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamLookupIdsInDomain( SAM_HANDLE DomainHandle,
//                       ULONG Count,
//                       PULONG RelativeIds,
//                       PUNICODE_STRING *Names,
//                       PSID_NAME_USE *Use )
// {
//     SAMPR_RETURNED_USTRING_ARRAY NamesBuffer = {0, NULL};
//     SAMPR_ULONG_ARRAY UseBuffer = {0, NULL};
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("SamLookupIdsInDomain(%p %lu %p %p %p)\n",
//           DomainHandle, Count, RelativeIds, Names, Use);
//
//     *Names = NULL;
//
//     if (Use != NULL)
//         *Use = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrLookupIdsInDomain((SAMPR_HANDLE)DomainHandle,
//                                        Count,
//                                        RelativeIds,
//                                        &NamesBuffer,
//                                        &UseBuffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (NT_SUCCESS(Status))
//     {
//         *Names = MIDL_user_allocate((Count * sizeof(RPC_UNICODE_STRING)) as SIZE_T);
//         if (*Names == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         for (i = 0; i < Count; i++)
//         {
//             (*Names)[i].Buffer = MIDL_user_allocate((NamesBuffer.Element[i].MaximumLength) as SIZE_T);
//             if ((*Names)[i].Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//         }
//
//         for (i = 0; i < Count; i++)
//         {
//             (*Names)[i].Length = NamesBuffer.Element[i].Length;
//             (*Names)[i].MaximumLength = NamesBuffer.Element[i].MaximumLength;
//
//             RtlCopyMemory!((*Names)[i].Buffer,
//                           NamesBuffer.Element[i].Buffer,
//                           NamesBuffer.Element[i].Length);
//         }
//
//         if (Use != NULL)
//         {
//             *Use = MIDL_user_allocate((Count * sizeof(SID_NAME_USE)) as SIZE_T);
//             if (*Use == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             RtlCopyMemory!(*Use,
//                           UseBuffer.Element,
//                           Count * sizeof(SID_NAME_USE));
//         }
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (*Names != NULL)
//         {
//             for (i = 0; i < Count; i++)
//             {
//                 if ((*Names)[i].Buffer != NULL)
//                     MIDL_user_free((*Names)[i].Buffer);
//             }
//
//             MIDL_user_free(*Names);
//         }
//
//         if (Use != NULL && *Use != NULL)
//             MIDL_user_free(*Use);
//     }
//
//     if (NamesBuffer.Element != NULL)
//     {
//         for (i = 0; i < NamesBuffer.Count; i++)
//         {
//             if (NamesBuffer.Element[i].Buffer != NULL)
//                 MIDL_user_free(NamesBuffer.Element[i].Buffer);
//         }
//
//         MIDL_user_free(NamesBuffer.Element);
//     }
//
//     if (UseBuffer.Element != NULL)
//         MIDL_user_free(UseBuffer.Element);
//
//     return 0;
// }
//
//
// NTSTATUS
// NTAPI
// SamLookupNamesInDomain( SAM_HANDLE DomainHandle,
//                         ULONG Count,
//                         PUNICODE_STRING Names,
//                         PULONG *RelativeIds,
//                         PSID_NAME_USE *Use)
// {
//     SAMPR_ULONG_ARRAY RidBuffer = {0, NULL};
//     SAMPR_ULONG_ARRAY UseBuffer = {0, NULL};
//     NTSTATUS Status;
//
//     TRACE("SamLookupNamesInDomain(%p %lu %p %p %p)\n",
//           DomainHandle, Count, Names, RelativeIds, Use);
//
//     *RelativeIds = NULL;
//     *Use = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrLookupNamesInDomain((SAMPR_HANDLE)DomainHandle,
//                                          Count,
//                                          (PRPC_UNICODE_STRING)Names,
//                                          &RidBuffer,
//                                          &UseBuffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (NT_SUCCESS(Status))
//     {
//         *RelativeIds = MIDL_user_allocate((Count * sizeof(ULONG)) as SIZE_T);
//         if (*RelativeIds == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         *Use = MIDL_user_allocate((Count * sizeof(SID_NAME_USE)) as SIZE_T);
//         if (*Use == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         RtlCopyMemory!(*RelativeIds,
//                       RidBuffer.Element,
//                       Count * sizeof(ULONG));
//
//         RtlCopyMemory!(*Use,
//                       UseBuffer.Element,
//                       Count * sizeof(SID_NAME_USE));
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (*RelativeIds != NULL)
//             MIDL_user_free(*RelativeIds);
//
//         if (*Use != NULL)
//             MIDL_user_free(*Use);
//     }
//
//     if (RidBuffer.Element != NULL)
//         MIDL_user_free(RidBuffer.Element);
//
//     if (UseBuffer.Element != NULL)
//         MIDL_user_free(UseBuffer.Element);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamOpenAlias( SAM_HANDLE DomainHandle,
//               ACCESS_MASK DesiredAccess,
//               ULONG AliasId,
//               PSAM_HANDLE AliasHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamOpenAlias(%p 0x%08x %lx %p)\n",
//           DomainHandle, DesiredAccess, AliasId, AliasHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrOpenAlias((SAMPR_HANDLE)DomainHandle,
//                                DesiredAccess,
//                                AliasId,
//                                (SAMPR_HANDLE *)AliasHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamOpenDomain( SAM_HANDLE ServerHandle,
//                ACCESS_MASK DesiredAccess,
//                PSID DomainId,
//                PSAM_HANDLE DomainHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamOpenDomain(%p 0x%08x %p %p)\n",
//           ServerHandle, DesiredAccess, DomainId, DomainHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrOpenDomain((SAMPR_HANDLE)ServerHandle,
//                                 DesiredAccess,
//                                 (PRPC_SID)DomainId,
//                                 (SAMPR_HANDLE *)DomainHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamOpenGroup( SAM_HANDLE DomainHandle,
//               ACCESS_MASK DesiredAccess,
//               ULONG GroupId,
//               PSAM_HANDLE GroupHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamOpenGroup(%p 0x%08x %p %p)\n",
//           DomainHandle, DesiredAccess, GroupId, GroupHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrOpenGroup((SAMPR_HANDLE)DomainHandle,
//                                DesiredAccess,
//                                GroupId,
//                                (SAMPR_HANDLE *)GroupHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamOpenUser( SAM_HANDLE DomainHandle,
//              ACCESS_MASK DesiredAccess,
//              ULONG UserId,
//              PSAM_HANDLE UserHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamOpenUser(%p 0x%08x %lx %p)\n",
//           DomainHandle, DesiredAccess, UserId, UserHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrOpenUser((SAMPR_HANDLE)DomainHandle,
//                               DesiredAccess,
//                               UserId,
//                               (SAMPR_HANDLE *)UserHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQueryDisplayInformation( SAM_HANDLE DomainHandle,
//                             DOMAIN_DISPLAY_INFORMATION DisplayInformation,
//                             ULONG Index,
//                             ULONG EntryCount,
//                             ULONG PreferredMaximumLength,
//                             PULONG TotalAvailable,
//                             PULONG TotalReturned,
//                             PULONG ReturnedEntryCount,
//                             PVOID *SortedBuffer)
// {
//     SAMPR_DISPLAY_INFO_BUFFER LocalBuffer;
//     NTSTATUS Status;
//
//     TRACE("SamQueryDisplayInformation(%p %lu %lu %lu %lu %p %p %p %p)\n",
//           DomainHandle, DisplayInformation, Index, EntryCount,
//           PreferredMaximumLength, TotalAvailable, TotalReturned,
//           ReturnedEntryCount, SortedBuffer);
//
//     if ((TotalAvailable == NULL) ||
//         (TotalReturned == NULL) ||
//         (ReturnedEntryCount == NULL) ||
//         (SortedBuffer == NULL))
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = SamrQueryDisplayInformation3((SAMPR_HANDLE)DomainHandle,
//                                               DisplayInformation,
//                                               Index,
//                                               EntryCount,
//                                               PreferredMaximumLength,
//                                               TotalAvailable,
//                                               TotalReturned,
//                                               &LocalBuffer);
//         if (NT_SUCCESS(Status))
//         {
//             switch (DisplayInformation)
//             {
//                 case DomainDisplayUser:
//                     *ReturnedEntryCount = LocalBuffer.UserInformation.EntriesRead;
//                     *SortedBuffer = LocalBuffer.UserInformation.Buffer;
//                     break;
//
//                 case DomainDisplayMachine:
//                     *ReturnedEntryCount = LocalBuffer.MachineInformation.EntriesRead;
//                     *SortedBuffer = LocalBuffer.MachineInformation.Buffer;
//                     break;
//
//                 case DomainDisplayGroup:
//                     *ReturnedEntryCount = LocalBuffer.GroupInformation.EntriesRead;
//                     *SortedBuffer = LocalBuffer.GroupInformation.Buffer;
//                     break;
//
//                 case DomainDisplayOemUser:
//                     *ReturnedEntryCount = LocalBuffer.OemUserInformation.EntriesRead;
//                     *SortedBuffer = LocalBuffer.OemUserInformation.Buffer;
//                     break;
//
//                 case DomainDisplayOemGroup:
//                     *ReturnedEntryCount = LocalBuffer.OemGroupInformation.EntriesRead;
//                     *SortedBuffer = LocalBuffer.OemGroupInformation.Buffer;
//                     break;
//
//                 case DomainDisplayServer:
//                     /* FIXME */
//                     break;
//             }
//         }
//         else
//         {
//             *ReturnedEntryCount = 0;
//             *SortedBuffer = NULL;
//         }
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQueryInformationAlias( SAM_HANDLE AliasHandle,
//                           ALIAS_INFORMATION_CLASS AliasInformationClass,
//                           PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamQueryInformationAlias(%p %lu %p)\n",
//           AliasHandle, AliasInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrQueryInformationAlias((SAMPR_HANDLE)AliasHandle,
//                                            AliasInformationClass,
//                                            (PSAMPR_ALIAS_INFO_BUFFER *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQueryInformationDomain( SAM_HANDLE DomainHandle,
//                            DOMAIN_INFORMATION_CLASS DomainInformationClass,
//                            PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamQueryInformationDomain(%p %lu %p)\n",
//           DomainHandle, DomainInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrQueryInformationDomain((SAMPR_HANDLE)DomainHandle,
//                                             DomainInformationClass,
//                                             (PSAMPR_DOMAIN_INFO_BUFFER *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQueryInformationGroup( SAM_HANDLE GroupHandle,
//                           GROUP_INFORMATION_CLASS GroupInformationClass,
//                           PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamQueryInformationGroup(%p %lu %p)\n",
//           GroupHandle, GroupInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrQueryInformationGroup((SAMPR_HANDLE)GroupHandle,
//                                            GroupInformationClass,
//                                            (PSAMPR_GROUP_INFO_BUFFER *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQueryInformationUser( SAM_HANDLE UserHandle,
//                          USER_INFORMATION_CLASS UserInformationClass,
//                          PVOID *Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamQueryInformationUser(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrQueryInformationUser((SAMPR_HANDLE)UserHandle,
//                                           UserInformationClass,
//                                           (PSAMPR_USER_INFO_BUFFER *)Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamQuerySecurityObject( SAM_HANDLE ObjectHandle,
//                         SECURITY_INFORMATION SecurityInformation,
//                         PSECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     PSAMPR_SR_SECURITY_DESCRIPTOR SamSecurityDescriptor = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamQuerySecurityObject(%p %lu %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     *SecurityDescriptor = NULL;
//
//     RpcTryExcept
//     {
//         Status = SamrQuerySecurityObject((SAMPR_HANDLE)ObjectHandle,
//                                          SecurityInformation,
//                                          &SamSecurityDescriptor);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     TRACE("SamSecurityDescriptor: %p\n", SamSecurityDescriptor);
//
//     if (SamSecurityDescriptor != NULL)
//     {
//         TRACE("SamSecurityDescriptor->Length: %lu\n", SamSecurityDescriptor->Length);
//         TRACE("SamSecurityDescriptor->SecurityDescriptor: %p\n", SamSecurityDescriptor->SecurityDescriptor);
//
//         *SecurityDescriptor = SamSecurityDescriptor->SecurityDescriptor;
//
//         MIDL_user_free(SamSecurityDescriptor);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamRemoveMemberFromAlias( SAM_HANDLE AliasHandle,
//                           PSID MemberId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamRemoveMemberFromAlias(%p %ul)\n",
//           AliasHandle, MemberId);
//
//     RpcTryExcept
//     {
//         Status = SamrRemoveMemberFromAlias((SAMPR_HANDLE)AliasHandle,
//                                            MemberId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamRemoveMemberFromForeignDomain( SAM_HANDLE DomainHandle,
//                                   PSID MemberId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamRemoveMemberFromForeignDomain(%p %ul)\n",
//           DomainHandle, MemberId);
//
//     RpcTryExcept
//     {
//         Status = SamrRemoveMemberFromForeignDomain((SAMPR_HANDLE)DomainHandle,
//                                                    MemberId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamRemoveMemberFromGroup( SAM_HANDLE GroupHandle,
//                           ULONG MemberId)
// {
//     NTSTATUS Status;
//
//     TRACE("SamRemoveMemberFromGroup(%p %ul)\n",
//           GroupHandle, MemberId);
//
//     RpcTryExcept
//     {
//         Status = SamrRemoveMemberFromGroup((SAMPR_HANDLE)GroupHandle,
//                                            MemberId);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamRemoveMultipleMembersFromAlias( SAM_HANDLE AliasHandle,
//                                    PSID *MemberIds,
//                                    ULONG MemberCount)
// {
//     SAMPR_PSID_ARRAY Buffer;
//     NTSTATUS Status;
//
//     TRACE("SamRemoveMultipleMembersFromAlias(%p %p %lu)\n",
//           AliasHandle, MemberIds, MemberCount);
//
//     if (MemberIds == NULL)
//         return STATUS_INVALID_PARAMETER_2;
//
//     Buffer.Count = MemberCount;
//     Buffer.Sids = (PSAMPR_SID_INFORMATION)MemberIds;
//
//     RpcTryExcept
//     {
//         Status = SamrRemoveMultipleMembersFromAlias((SAMPR_HANDLE)AliasHandle,
//                                                     &Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamRidToSid( SAM_HANDLE ObjectHandle,
//              ULONG Rid,
//              PSID *Sid)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetInformationAlias( SAM_HANDLE AliasHandle,
//                         ALIAS_INFORMATION_CLASS AliasInformationClass,
//                         PVOID Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamSetInformationAlias(%p %lu %p)\n",
//           AliasHandle, AliasInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrSetInformationAlias((SAMPR_HANDLE)AliasHandle,
//                                          AliasInformationClass,
//                                          Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetInformationDomain( SAM_HANDLE DomainHandle,
//                          DOMAIN_INFORMATION_CLASS DomainInformationClass,
//                          PVOID Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamSetInformationDomain(%p %lu %p)\n",
//           DomainHandle, DomainInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrSetInformationDomain((SAMPR_HANDLE)DomainHandle,
//                                           DomainInformationClass,
//                                           Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetInformationGroup( SAM_HANDLE GroupHandle,
//                         GROUP_INFORMATION_CLASS GroupInformationClass,
//                         PVOID Buffer)
// {
//     NTSTATUS Status;
//
//     TRACE("SamSetInformationGroup(%p %lu %p)\n",
//           GroupHandle, GroupInformationClass, Buffer);
//
//     RpcTryExcept
//     {
//         Status = SamrSetInformationGroup((SAMPR_HANDLE)GroupHandle,
//                                          GroupInformationClass,
//                                          Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetInformationUser( SAM_HANDLE UserHandle,
//                        USER_INFORMATION_CLASS UserInformationClass,
//                        PVOID Buffer)
// {
//     PSAMPR_USER_SET_PASSWORD_INFORMATION PasswordBuffer;
//     SAMPR_USER_INTERNAL1_INFORMATION Internal1Buffer;
//     USER_ALL_INFORMATION InternalAllBuffer;
//     OEM_STRING LmPwdString;
//     CHAR LmPwdBuffer[15];
//     NTSTATUS Status;
//
//     TRACE("SamSetInformationUser(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     if (UserInformationClass == UserSetPasswordInformation)
//     {
//         PasswordBuffer = (PSAMPR_USER_SET_PASSWORD_INFORMATION)Buffer;
//
//         Status = SampCheckPassword(UserHandle,
//                                    (PUNICODE_STRING)&PasswordBuffer->Password);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampCheckPassword failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         /* Calculate the NT hash value of the password */
//         Status = SystemFunction007((PUNICODE_STRING)&PasswordBuffer->Password,
//                                    (LPBYTE)&Internal1Buffer.EncryptedNtOwfPassword);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         Internal1Buffer.NtPasswordPresent = TRUE;
//         Internal1Buffer.LmPasswordPresent = FALSE;
//
//         /* Build the LM password */
//         LmPwdString.Length = 15;
//         LmPwdString.MaximumLength = 15;
//         LmPwdString.Buffer = LmPwdBuffer;
//         ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//         Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                    (PUNICODE_STRING)&PasswordBuffer->Password,
//                                                    FALSE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Calculate the LM hash value of the password */
//             Status = SystemFunction006(LmPwdString.Buffer,
//                                        (LPSTR)&Internal1Buffer.EncryptedLmOwfPassword);
//             if (NT_SUCCESS(Status))
//                 Internal1Buffer.LmPasswordPresent = TRUE;
//         }
//
//         Internal1Buffer.PasswordExpired = PasswordBuffer->PasswordExpired;
//
//         RpcTryExcept
//         {
//             Status = SamrSetInformationUser((SAMPR_HANDLE)UserHandle,
//                                             UserInternal1Information,
//                                             (PVOID)&Internal1Buffer);
//         }
//         RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = I_RpcMapWin32Status(RpcExceptionCode());
//         }
//         RpcEndExcept;
//
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SamrSetInformation() failed (Status 0x%08lx)\n", Status);
//         }
//
//         return Status;
//     }
//     else if (UserInformationClass == UserAllInformation)
//     {
//         RtlCopyMemory!(&InternalAllBuffer,
//                       Buffer,
//                       sizeof(USER_ALL_INFORMATION));
//
//         if (InternalAllBuffer.WhichFields & (USER_ALL_LMPASSWORDPRESENT | USER_ALL_NTPASSWORDPRESENT))
//         {
//             if (InternalAllBuffer.WhichFields & USER_ALL_OWFPASSWORD)
//             {
//                 /* Check NT password hash */
//                 if (InternalAllBuffer.WhichFields & USER_ALL_NTPASSWORDPRESENT)
//                 {
//                     if (InternalAllBuffer.NtPassword.Length != sizeof(ENCRYPTED_NT_OWF_PASSWORD))
//                         return STATUS_INVALID_PARAMETER;
//                 }
//
//                 /* Check LM password hash */
//                 if (InternalAllBuffer.WhichFields & USER_ALL_LMPASSWORDPRESENT)
//                 {
//                     if (InternalAllBuffer.LmPassword.Length != sizeof(ENCRYPTED_LM_OWF_PASSWORD))
//                         return STATUS_INVALID_PARAMETER;
//                 }
//             }
//             else
//             {
//                 /*
//                  * Only allow the NT password to be set.
//                  * The LM password will be created here.
//                  */
//                 if (InternalAllBuffer.WhichFields & USER_ALL_LMPASSWORDPRESENT)
//                 {
//                     TRACE("Do not try to set a clear text LM password!\n");
//                     return STATUS_INVALID_PARAMETER;
//                 }
//
//                 if (InternalAllBuffer.WhichFields & USER_ALL_NTPASSWORDPRESENT)
//                 {
//                     Status = SampCheckPassword(UserHandle,
//                                                &InternalAllBuffer.NtPassword);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         TRACE("SampCheckPassword failed (Status 0x%08lx)\n", Status);
//                         return Status;
//                     }
//
//                     /* Calculate the NT password hash */
//                     Status = SystemFunction007((PUNICODE_STRING)&InternalAllBuffer.NtPassword,
//                                                (LPBYTE)&Internal1Buffer.EncryptedNtOwfPassword);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//                         return Status;
//                     }
//
//                     InternalAllBuffer.NtPasswordPresent = TRUE;
//                     InternalAllBuffer.LmPasswordPresent = FALSE;
//
//                     InternalAllBuffer.NtPassword.Length = sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//                     InternalAllBuffer.NtPassword.MaximumLength = sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//                     InternalAllBuffer.NtPassword.Buffer = (LPWSTR)&Internal1Buffer.EncryptedNtOwfPassword;
//
//                     /* Build the LM password */
//                     LmPwdString.Length = 15;
//                     LmPwdString.MaximumLength = 15;
//                     LmPwdString.Buffer = LmPwdBuffer;
//                     ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//                     Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                                (PUNICODE_STRING)&InternalAllBuffer.NtPassword,
//                                                                FALSE);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* Calculate the LM password hash */
//                         Status = SystemFunction006(LmPwdString.Buffer,
//                                                    (LPSTR)&Internal1Buffer.EncryptedLmOwfPassword);
//                         if (NT_SUCCESS(Status))
//                         {
//                             InternalAllBuffer.WhichFields |= USER_ALL_LMPASSWORDPRESENT;
//                             InternalAllBuffer.LmPasswordPresent = TRUE;
//
//                             InternalAllBuffer.LmPassword.Length = sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//                             InternalAllBuffer.LmPassword.MaximumLength = sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//                             InternalAllBuffer.LmPassword.Buffer = (LPWSTR)&Internal1Buffer.EncryptedLmOwfPassword;
//                         }
//                     }
//                 }
//             }
//         }
//
//         RpcTryExcept
//         {
//             Status = SamrSetInformationUser((SAMPR_HANDLE)UserHandle,
//                                             UserAllInformation,
//                                             (PVOID)&InternalAllBuffer);
//         }
//         RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = I_RpcMapWin32Status(RpcExceptionCode());
//         }
//         RpcEndExcept;
//
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SamrSetInformation() failed (Status 0x%08lx)\n", Status);
//         }
//
//         return Status;
//     }
//
//     RpcTryExcept
//     {
//         Status = SamrSetInformationUser((SAMPR_HANDLE)UserHandle,
//                                         UserInformationClass,
//                                         Buffer);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetMemberAttributesOfGroup( SAM_HANDLE GroupHandle,
//                                ULONG MemberId,
//                                ULONG Attributes)
// {
//     NTSTATUS Status;
//
//     TRACE("SamSetMemberAttributesOfGroup(%p %lu 0x%lx)\n",
//           GroupHandle, MemberId, Attributes);
//
//     RpcTryExcept
//     {
//         Status = SamrSetMemberAttributesOfGroup((SAMPR_HANDLE)GroupHandle,
//                                                 MemberId,
//                                                 Attributes);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamSetSecurityObject( SAM_HANDLE ObjectHandle,
//                       SECURITY_INFORMATION SecurityInformation,
//                       PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     SAMPR_SR_SECURITY_DESCRIPTOR DescriptorToPass;
//     ULONG Length;
//     NTSTATUS Status;
//
//     TRACE("SamSetSecurityObject(%p %lu %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     /* Retrieve the length of the relative security descriptor */
//     Length = 0;
//     Status = RtlMakeSelfRelativeSD(SecurityDescriptor,
//                                    NULL,
//                                    &Length);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         return STATUS_INVALID_PARAMETER;
//
//
//     /* Allocate a buffer for the security descriptor */
//     DescriptorToPass.Length = Length;
//     DescriptorToPass.SecurityDescriptor = MIDL_user_allocate((Length) as SIZE_T);
//     if (DescriptorToPass.SecurityDescriptor == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Convert the given security descriptor to a relative security descriptor */
//     Status = RtlMakeSelfRelativeSD(SecurityDescriptor,
//                                    (PSECURITY_DESCRIPTOR)DescriptorToPass.SecurityDescriptor,
//                                    &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     RpcTryExcept
//     {
//         Status = SamrSetSecurityObject((SAMPR_HANDLE)ObjectHandle,
//                                        SecurityInformation,
//                                        &DescriptorToPass);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
// done:
//     if (DescriptorToPass.SecurityDescriptor != NULL)
//         MIDL_user_free(DescriptorToPass.SecurityDescriptor);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamShutdownSamServer( SAM_HANDLE ServerHandle)
// {
//     NTSTATUS Status;
//
//     TRACE("SamShutdownSamServer(%p)\n", ServerHandle);
//
//     RpcTryExcept
//     {
//         Status = SamrShutdownSamServer((SAMPR_HANDLE)ServerHandle);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamTestPrivateFunctionsDomain(
//      SAM_HANDLE DomainHandle)
// {
//     TRACE("SamTestPrivateFunctionsDomain(%p)\n",
//           DomainHandle);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// SamTestPrivateFunctionsUser(
//      SAM_HANDLE UserHandle)
// {
//     TRACE("SamTestPrivateFunctionsUser(%p)\n",
//           UserHandle);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// HAND-AUTHORED — not model output. The model was rejected 3/3 on this body (the most
// complex in the TU); every attempt hoisted the RpcExcept handler into live code and wrote
// `I_RpcMapWin32Status(MIDL_user_free())`, passing a free() where the C passes
// RpcExceptionCode(). One rejected worker voids the whole TU, so this is written by hand to
// let the other 52 banked functions assemble. Translated from dll/win32/samlib/samlib.c.
//
// The RpcExcept block is intentionally absent: under -D_USE_DUMMY_PSEH=1 it is compiled but
// never entered, which is exactly how the other 36 Sam* wrappers in this TU came out.
#[no_mangle]
pub unsafe extern "system" fn SamConnect(
    ServerName: PUNICODE_STRING,
    ServerHandle: PSAM_HANDLE,
    DesiredAccess: ACCESS_MASK,
    _ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let mut pServerName: PSAMPR_SERVER_NAME = core::ptr::null_mut();
    let Status: NTSTATUS;

    if !ServerName.is_null() && (*ServerName).Length > 0 && !(*ServerName).Buffer.is_null() {
        // Length is in BYTES; the +sizeof(WCHAR) is the room for the terminator.
        pServerName = MIDL_user_allocate(
            (*ServerName).Length as SIZE_T + core::mem::size_of::<u16>() as SIZE_T,
        ) as PSAMPR_SERVER_NAME;
        if pServerName.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        CopyMemory!(
            pServerName as *mut core::ffi::c_void,
            (*ServerName).Buffer as *const core::ffi::c_void,
            (*ServerName).Length as SIZE_T,
        );
        // Index is in WCHARs, so the byte Length is halved. .add() rather than [i]: a bindgen
        // pointer indexed with [i] bounds-check-panics past 0.
        *pServerName.add((*ServerName).Length as usize / core::mem::size_of::<u16>()) =
            UNICODE_NULL;
    }

    Status = rpc_try(|| SamrConnect(
        pServerName,
        ServerHandle as *mut SAMPR_HANDLE,
        DesiredAccess,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !pServerName.is_null() {
        MIDL_user_free(pServerName as *mut core::ffi::c_void);
    }

    Status
}
