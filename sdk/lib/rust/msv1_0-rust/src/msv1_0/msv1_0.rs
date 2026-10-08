use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LOGON_LIST_ENTRY {
    pub ListEntry: LIST_ENTRY,
    pub LogonId: LUID,
    pub EnumHandle: ULONG,
    pub UserName: _UNICODE_STRING,
    pub LogonDomainName: _UNICODE_STRING,
    pub LogonServer: _UNICODE_STRING,
    pub LogonType: SECURITY_LOGON_TYPE,
}

pub type LOGON_LIST_ENTRY = _LOGON_LIST_ENTRY;
pub type PLOGON_LIST_ENTRY = *mut _LOGON_LIST_ENTRY;

pub static mut PackageInitialized: BOOL = 0;
pub static mut LogonListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
// The C is a bare `RTL_RESOURCE LogonListResource;` — a tentative definition, so zeroed, and
// RtlInitializeResource fills it at PackageInitialize. The translation had instead expanded a
// field-by-field initializer with SIX field names RTL_RESOURCE does not have
// (CriticalSection/NumberOfWaitingShared/NumberOfWaitingExclusive/NumberOfActive/
// ExclusiveOwnerThread/Flags, vs the real Lock/SharedWaiters/ExclusiveWaiters/NumberActive/
// OwningThread/TimeoutBoost) — recalled, not read. Zeroed is both faithful and immune to the
// layout drift that made those names plausible.
pub static mut LogonListResource: RTL_RESOURCE = unsafe { core::mem::zeroed() };
pub static mut EnumCounter: ULONG = 0;
unsafe extern "C" fn GetLogonByLogonId(
    LogonId: PLUID,
) -> *mut LOGON_LIST_ENTRY {
    let mut LogonEntry: *mut LOGON_LIST_ENTRY;
    let mut CurrentEntry: PLIST_ENTRY;

    CurrentEntry = LogonListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(LogonListHead) {
        LogonEntry = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(LOGON_LIST_ENTRY, ListEntry) as isize))
            as *mut LOGON_LIST_ENTRY;

        if ((*LogonEntry).LogonId.HighPart == (*LogonId).HighPart) &&
           ((*LogonEntry).LogonId.LowPart == (*LogonId).LowPart) {
            return LogonEntry;
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn BuildInteractiveProfileBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    UserInfo: PSAMPR_USER_INFO_BUFFER,
    ComputerName: PWSTR,
    ProfileBuffer: *mut PMSV1_0_INTERACTIVE_PROFILE,
    ProfileBufferLength: PULONG,
) -> NTSTATUS {
    let mut LocalBuffer: PMSV1_0_INTERACTIVE_PROFILE = core::ptr::null_mut();
    let mut ClientBaseAddress: PVOID = core::ptr::null_mut();
    let mut Ptr: LPWSTR;
    let mut BufferLength: ULONG;
    let mut ComputerNameLength: USHORT = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    *ProfileBuffer = core::ptr::null_mut();
    *ProfileBufferLength = 0;

    if UIntPtrToUShort(wcslen(ComputerName) as UINT_PTR, core::ptr::addr_of_mut!(ComputerNameLength)) != S_OK {
        return STATUS_INVALID_PARAMETER;
    }

    BufferLength = core::mem::size_of::<MSV1_0_INTERACTIVE_PROFILE>() as ULONG +
                   (*UserInfo).All.FullName.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG +
                   (*UserInfo).All.HomeDirectory.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG +
                   (*UserInfo).All.HomeDirectoryDrive.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG +
                   (*UserInfo).All.ScriptPath.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG +
                   (*UserInfo).All.ProfilePath.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG +
                   ((ComputerNameLength as ULONG + 3) * core::mem::size_of::<WCHAR>() as ULONG);

    LocalBuffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(BufferLength) as PMSV1_0_INTERACTIVE_PROFILE;
    if LocalBuffer == core::ptr::null_mut() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        return 'done: {
            if LocalBuffer != core::ptr::null_mut() {
                DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer as PVOID);
            }

            if !NT_SUCCESS(Status) {
                if ClientBaseAddress != core::ptr::null_mut() {
                    DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, ClientBaseAddress);
                }
            }

            break 'done Status;
        };
    }

    Status = DispatchTable.AllocateClientBuffer.unwrap_unchecked()(
        ClientRequest,
        BufferLength,
        core::ptr::addr_of_mut!(ClientBaseAddress),
    );
    if !NT_SUCCESS(Status) {
        return 'done: {
            if LocalBuffer != core::ptr::null_mut() {
                DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer as PVOID);
            }

            if !NT_SUCCESS(Status) {
                if ClientBaseAddress != core::ptr::null_mut() {
                    DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, ClientBaseAddress);
                }
            }

            break 'done Status;
        };
    }

    Ptr = (LocalBuffer as ULONG_PTR
        + core::mem::size_of::<MSV1_0_INTERACTIVE_PROFILE>() as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).MessageType = MsV1_0InteractiveProfile;
    (*LocalBuffer).LogonCount = (*UserInfo).All.LogonCount;
    (*LocalBuffer).BadPasswordCount = (*UserInfo).All.BadPasswordCount;

    (*LocalBuffer).LogonTime.__bindgen_anon_1.LowPart = (*UserInfo).All.LastLogon.LowPart;
    (*LocalBuffer).LogonTime.__bindgen_anon_1.HighPart = (*UserInfo).All.LastLogon.HighPart;

    (*LocalBuffer).LogoffTime.__bindgen_anon_1.LowPart = (*UserInfo).All.AccountExpires.LowPart;
    (*LocalBuffer).LogoffTime.__bindgen_anon_1.HighPart = (*UserInfo).All.AccountExpires.HighPart;

    (*LocalBuffer).KickOffTime.__bindgen_anon_1.LowPart = (*UserInfo).All.AccountExpires.LowPart;
    (*LocalBuffer).KickOffTime.__bindgen_anon_1.HighPart = (*UserInfo).All.AccountExpires.HighPart;

    (*LocalBuffer).PasswordLastSet.__bindgen_anon_1.LowPart = (*UserInfo).All.PasswordLastSet.LowPart;
    (*LocalBuffer).PasswordLastSet.__bindgen_anon_1.HighPart = (*UserInfo).All.PasswordLastSet.HighPart;

    (*LocalBuffer).PasswordCanChange.__bindgen_anon_1.LowPart = (*UserInfo).All.PasswordCanChange.LowPart;
    (*LocalBuffer).PasswordCanChange.__bindgen_anon_1.HighPart = (*UserInfo).All.PasswordCanChange.HighPart;

    (*LocalBuffer).PasswordMustChange.__bindgen_anon_1.LowPart = (*UserInfo).All.PasswordMustChange.LowPart;
    (*LocalBuffer).PasswordMustChange.__bindgen_anon_1.HighPart = (*UserInfo).All.PasswordMustChange.HighPart;

    (*LocalBuffer).LogonScript.Length = (*UserInfo).All.ScriptPath.Length;
    (*LocalBuffer).LogonScript.MaximumLength = (*UserInfo).All.ScriptPath.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).LogonScript.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    memcpy(
        Ptr as PVOID,
        (*UserInfo).All.ScriptPath.Buffer as PVOID,
        (*UserInfo).All.ScriptPath.Length as usize,
    );

    Ptr = (Ptr as ULONG_PTR + (*LocalBuffer).LogonScript.MaximumLength as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).HomeDirectory.Length = (*UserInfo).All.HomeDirectory.Length;
    (*LocalBuffer).HomeDirectory.MaximumLength = (*UserInfo).All.HomeDirectory.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).HomeDirectory.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    memcpy(
        Ptr as PVOID,
        (*UserInfo).All.HomeDirectory.Buffer as PVOID,
        (*UserInfo).All.HomeDirectory.Length as usize,
    );

    Ptr = (Ptr as ULONG_PTR + (*LocalBuffer).HomeDirectory.MaximumLength as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).FullName.Length = (*UserInfo).All.FullName.Length;
    (*LocalBuffer).FullName.MaximumLength = (*UserInfo).All.FullName.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).FullName.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    memcpy(
        Ptr as PVOID,
        (*UserInfo).All.FullName.Buffer as PVOID,
        (*UserInfo).All.FullName.Length as usize,
    );

    Ptr = (Ptr as ULONG_PTR + (*LocalBuffer).FullName.MaximumLength as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).ProfilePath.Length = (*UserInfo).All.ProfilePath.Length;
    (*LocalBuffer).ProfilePath.MaximumLength = (*UserInfo).All.ProfilePath.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).ProfilePath.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    memcpy(
        Ptr as PVOID,
        (*UserInfo).All.ProfilePath.Buffer as PVOID,
        (*UserInfo).All.ProfilePath.Length as usize,
    );

    Ptr = (Ptr as ULONG_PTR + (*LocalBuffer).ProfilePath.MaximumLength as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).HomeDirectoryDrive.Length = (*UserInfo).All.HomeDirectoryDrive.Length;
    (*LocalBuffer).HomeDirectoryDrive.MaximumLength = (*UserInfo).All.HomeDirectoryDrive.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).HomeDirectoryDrive.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    memcpy(
        Ptr as PVOID,
        (*UserInfo).All.HomeDirectoryDrive.Buffer as PVOID,
        (*UserInfo).All.HomeDirectoryDrive.Length as usize,
    );

    Ptr = (Ptr as ULONG_PTR + (*LocalBuffer).HomeDirectoryDrive.MaximumLength as ULONG_PTR) as LPWSTR;

    (*LocalBuffer).LogonServer.Length = ((ComputerNameLength as ULONG + 2) * core::mem::size_of::<WCHAR>() as ULONG) as USHORT;
    (*LocalBuffer).LogonServer.MaximumLength = (*LocalBuffer).LogonServer.Length + core::mem::size_of::<WCHAR>() as USHORT;
    (*LocalBuffer).LogonServer.Buffer = (ClientBaseAddress as ULONG_PTR + Ptr as ULONG_PTR - LocalBuffer as ULONG_PTR) as LPWSTR;
    wcscpy(Ptr, w!("\\") as LPWSTR);
    wcscat(Ptr, ComputerName);

    (*LocalBuffer).UserFlags = 0;

    Status = DispatchTable.CopyToClientBuffer.unwrap_unchecked()(
        ClientRequest,
        BufferLength,
        ClientBaseAddress,
        LocalBuffer as PVOID,
    );
    if !NT_SUCCESS(Status) {
        return 'done: {
            if LocalBuffer != core::ptr::null_mut() {
                DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer as PVOID);
            }

            if !NT_SUCCESS(Status) {
                if ClientBaseAddress != core::ptr::null_mut() {
                    DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, ClientBaseAddress);
                }
            }

            break 'done Status;
        };
    }

    *ProfileBuffer = ClientBaseAddress as PMSV1_0_INTERACTIVE_PROFILE;
    *ProfileBufferLength = BufferLength;

    if LocalBuffer != core::ptr::null_mut() {
        DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if ClientBaseAddress != core::ptr::null_mut() {
            DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, ClientBaseAddress);
        }
    }

    Status
}
unsafe extern "C" fn BuildLm20LogonProfileBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    UserInfo: PSAMPR_USER_INFO_BUFFER,
    LogonPwdData: PLSA_SAM_PWD_DATA,
    ProfileBuffer: *mut PMSV1_0_LM20_LOGON_PROFILE,
    ProfileBufferLength: PULONG,
) -> NTSTATUS {
    let mut LocalBuffer: PMSV1_0_LM20_LOGON_PROFILE;
    let mut Buffer: NTLM_CLIENT_BUFFER = core::mem::zeroed();
    let mut PtrOffset: PBYTE;
    let mut BufferLength: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ComputerNameUCS: _UNICODE_STRING = core::mem::zeroed();

    *ProfileBuffer = core::ptr::null_mut();
    *ProfileBufferLength = 0;

    'done: loop {
        if !NtlmUStrAlloc(&mut ComputerNameUCS, (*(*LogonPwdData).ComputerName).Length + core::mem::size_of::<WCHAR>() as u16 * 3, 0) {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }
        Status = RtlAppendUnicodeToString(&mut ComputerNameUCS, w!("\\\\") as PWSTR);
        if !NT_SUCCESS(Status) {
            break 'done;
        }
        Status = RtlAppendUnicodeStringToString(&mut ComputerNameUCS, (*LogonPwdData).ComputerName);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        BufferLength = core::mem::size_of::<MSV1_0_LM20_LOGON_PROFILE>() as u32 + ComputerNameUCS.Length as u32 + core::mem::size_of::<WCHAR>() as u32;

        Status = NtlmAllocateClientBuffer(ClientRequest, BufferLength, &mut Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        LocalBuffer = Buffer.LocalBuffer as PMSV1_0_LM20_LOGON_PROFILE;
        PtrOffset = (LocalBuffer as usize + 1) as PBYTE;

        (*LocalBuffer).MessageType = MsV1_0Lm20LogonProfile;
        (*LocalBuffer).KickOffTime.__bindgen_anon_1.LowPart = (*UserInfo).All.AccountExpires.LowPart;
        (*LocalBuffer).KickOffTime.__bindgen_anon_1.HighPart = (*UserInfo).All.AccountExpires.HighPart;
        (*LocalBuffer).LogoffTime.__bindgen_anon_1.LowPart = (*UserInfo).All.AccountExpires.LowPart;
        (*LocalBuffer).LogoffTime.__bindgen_anon_1.HighPart = (*UserInfo).All.AccountExpires.HighPart;

        memcpy(
            ((*LocalBuffer).UserSessionKey.as_mut_ptr()).cast(),
            &(*LogonPwdData).UserSessionKey as *const _ as *const _,
            (MSV1_0_USER_SESSION_KEY_LENGTH) as usize,
        );

        RtlInitUnicodeString(
            &mut (*LocalBuffer).LogonDomainName,
            core::ptr::null_mut(),
        );

        memcpy(
            ((*LocalBuffer).LanmanSessionKey.as_mut_ptr()).cast(),
            &(*LogonPwdData).LanmanSessionKey as *const _ as *const _,
            (MSV1_0_LANMAN_SESSION_KEY_LENGTH) as usize,
        );

        if NtlmUStrWriteToStruct(
            LocalBuffer as *mut _,
            BufferLength,
            &mut (*LocalBuffer).LogonServer,
            &mut ComputerNameUCS,
            &mut PtrOffset,
            true,
        ) {
            Status = (ERROR_INTERNAL_ERROR) as i32;
            break 'done;
        }

        RtlInitUnicodeString(&mut (*LocalBuffer).UserParameters, core::ptr::null_mut());

        (*LocalBuffer).UserFlags = 0x0;
        if (*LogonPwdData).LogonType == NetLogonLmKey {
            (*LocalBuffer).UserFlags |= LOGON_USED_LM_PASSWORD;
        }

        Status = NtlmCopyToClientBuffer(ClientRequest, BufferLength, &mut Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *ProfileBuffer = Buffer.ClientBaseAddress as PMSV1_0_LM20_LOGON_PROFILE;
        *ProfileBufferLength = BufferLength;
        break 'done;
    }

    NtlmFreeClientBuffer(ClientRequest, !NT_SUCCESS(Status), &mut Buffer);
    NtlmUStrFree(&mut ComputerNameUCS);
    Status
}
unsafe extern "C" fn AppendRidToSid(
    SrcSid: PSID,
    Rid: u32,
) -> PSID {
    let mut DstSid: PSID = core::ptr::null_mut();
    let mut RidCount: u8;

    RidCount = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8 {
        return core::ptr::null_mut();
    }

    DstSid = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(RtlLengthRequiredSid((RidCount + 1) as u32));
    if DstSid == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    core::ptr::copy_nonoverlapping(
        SrcSid as *const u8,
        DstSid as *mut u8,
        RtlLengthRequiredSid((RidCount) as u32) as usize,
    );

    *RtlSubAuthorityCountSid(DstSid) = RidCount + 1;
    *RtlSubAuthoritySid(DstSid, (RidCount) as u32) = Rid;

    return DstSid;
}
unsafe extern "C" fn BuildTokenUser(
    User: PTOKEN_USER,
    AccountDomainSid: PSID,
    RelativeId: u32,
) -> i32 {
    (*User).User.Sid = AppendRidToSid(AccountDomainSid, RelativeId);
    if (*User).User.Sid == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*User).User.Attributes = 0;

    return STATUS_SUCCESS;
}
unsafe extern "C" fn BuildTokenPrimaryGroup(
    PrimaryGroup: *mut TOKEN_PRIMARY_GROUP,
    AccountDomainSid: PSID,
    RelativeId: u32,
) -> i32 {
    (*PrimaryGroup).PrimaryGroup = AppendRidToSid(
        AccountDomainSid,
        RelativeId,
    );
    if (*PrimaryGroup).PrimaryGroup == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    return STATUS_SUCCESS;
}
unsafe extern "C" fn BuildTokenGroups(
    Groups: *mut PTOKEN_GROUPS,
    AccountDomainSid: PSID,
    RelativeId: u32,
    SpecialAccount: BOOL,
) -> i32 {
    let mut SystemAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    // Renamed from the C's `TokenGroups`: ntdll-sys exports a unit const of that exact
    // name (TOKEN_INFORMATION_CLASS::TokenGroups), so through the prelude glob a `let
    // TokenGroups` is a constant PATTERN, not a new binding — the later assignment then
    // fails with "cannot assign to this expression". Local only, so no ABI consequence.
    let TokenGroupsBuf: PTOKEN_GROUPS;
    let mut GroupCount: u32 = 0;
    let mut MaxGroups: u32 = 2;
    let Sid: PSID;
    let Status: i32 = STATUS_SUCCESS;

    if SpecialAccount != 0 {
        MaxGroups += 1;
    }

    TokenGroupsBuf = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(
        (core::mem::size_of::<TOKEN_GROUPS>()
            + MaxGroups as usize * core::mem::size_of::<SID_AND_ATTRIBUTES>()) as ULONG,
    ) as PTOKEN_GROUPS;

    if TokenGroupsBuf.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if SpecialAccount != 0 {
        let Sid = AppendRidToSid(AccountDomainSid, RelativeId);
        if Sid.is_null() {

        }

        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Sid = Sid;
        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Attributes =
            SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
        GroupCount += 1;

        let mut Sid: PSID = core::ptr::null_mut();
        RtlAllocateAndInitializeSid(
            &mut SystemAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_USERS,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            &mut Sid,
        );
        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Sid = Sid;
        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Attributes =
            SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
        GroupCount += 1;
    } else {
        let Sid = AppendRidToSid(AccountDomainSid, DOMAIN_GROUP_RID_USERS);
        if Sid.is_null() {

        }

        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Sid = Sid;
        (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Attributes =
            SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
        GroupCount += 1;
    }

    let mut Sid: PSID = core::ptr::null_mut();
    RtlAllocateAndInitializeSid(
        &mut SystemAuthority,
        1,
        SECURITY_AUTHENTICATED_USER_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        &mut Sid,
    );
    (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Sid = Sid;
    (*(&raw mut (*TokenGroupsBuf).Groups[0]).add(GroupCount as usize)).Attributes =
        SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
    GroupCount += 1;

    (*TokenGroupsBuf).GroupCount = GroupCount;

    *Groups = TokenGroupsBuf;

    Status
}
unsafe extern "C" fn BuildTokenInformationBuffer(
    TokenInformation: *mut PLSA_TOKEN_INFORMATION_V1,
    AccountDomainSid: PRPC_SID,
    UserInfo: PSAMPR_USER_INFO_BUFFER,
    SpecialAccount: BOOL,
) -> i32 {
    let mut Buffer: PLSA_TOKEN_INFORMATION_V1 = core::ptr::null_mut();
    let mut i: u32;
    let mut Status: i32 = STATUS_SUCCESS;

    Buffer = (DispatchTable.AllocateLsaHeap.unwrap_unchecked())(
        core::mem::size_of::<LSA_TOKEN_INFORMATION_V1>() as ULONG,
    ) as PLSA_TOKEN_INFORMATION_V1;
    if Buffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        'done: loop {
            if !NT_SUCCESS(Status) {
                if !Buffer.is_null() {
                    if !(*Buffer).User.User.Sid.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).User.User.Sid as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).Groups.is_null() {
                        i = 0;
                        while i < (*(*Buffer).Groups).GroupCount {
                            if !(*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                                (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid as *mut core::ffi::c_void);
                            }
                            i = i.wrapping_add(1);
                        }

                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).Groups as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).PrimaryGroup.PrimaryGroup.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).PrimaryGroup.PrimaryGroup as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).DefaultDacl.DefaultDacl.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).DefaultDacl.DefaultDacl as *mut core::ffi::c_void);
                    }

                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())(Buffer as *mut core::ffi::c_void);
                }
            }

            return Status;
        }
    }

    (*Buffer).ExpirationTime.__bindgen_anon_1.LowPart = (*UserInfo).All.AccountExpires.LowPart;
    (*Buffer).ExpirationTime.__bindgen_anon_1.HighPart = (*UserInfo).All.AccountExpires.HighPart;

    Status = BuildTokenUser(
        core::ptr::addr_of_mut!((*Buffer).User),
        AccountDomainSid as PSID,
        (*UserInfo).All.UserId,
    );
    if !NT_SUCCESS(Status) {
        'done: loop {
            if !NT_SUCCESS(Status) {
                if !Buffer.is_null() {
                    if !(*Buffer).User.User.Sid.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).User.User.Sid as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).Groups.is_null() {
                        i = 0;
                        while i < (*(*Buffer).Groups).GroupCount {
                            if !(*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                                (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid as *mut core::ffi::c_void);
                            }
                            i = i.wrapping_add(1);
                        }

                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).Groups as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).PrimaryGroup.PrimaryGroup.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).PrimaryGroup.PrimaryGroup as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).DefaultDacl.DefaultDacl.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).DefaultDacl.DefaultDacl as *mut core::ffi::c_void);
                    }

                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())(Buffer as *mut core::ffi::c_void);
                }
            }

            return Status;
        }
    }

    Status = BuildTokenPrimaryGroup(
        core::ptr::addr_of_mut!((*Buffer).PrimaryGroup),
        AccountDomainSid as PSID,
        (*UserInfo).All.PrimaryGroupId,
    );
    if !NT_SUCCESS(Status) {
        'done: loop {
            if !NT_SUCCESS(Status) {
                if !Buffer.is_null() {
                    if !(*Buffer).User.User.Sid.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).User.User.Sid as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).Groups.is_null() {
                        i = 0;
                        while i < (*(*Buffer).Groups).GroupCount {
                            if !(*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                                (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid as *mut core::ffi::c_void);
                            }
                            i = i.wrapping_add(1);
                        }

                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).Groups as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).PrimaryGroup.PrimaryGroup.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).PrimaryGroup.PrimaryGroup as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).DefaultDacl.DefaultDacl.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).DefaultDacl.DefaultDacl as *mut core::ffi::c_void);
                    }

                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())(Buffer as *mut core::ffi::c_void);
                }
            }

            return Status;
        }
    }

    Status = BuildTokenGroups(
        core::ptr::addr_of_mut!((*Buffer).Groups),
        AccountDomainSid as PSID,
        (*UserInfo).All.UserId,
        SpecialAccount,
    );
    if !NT_SUCCESS(Status) {
        'done: loop {
            if !NT_SUCCESS(Status) {
                if !Buffer.is_null() {
                    if !(*Buffer).User.User.Sid.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).User.User.Sid as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).Groups.is_null() {
                        i = 0;
                        while i < (*(*Buffer).Groups).GroupCount {
                            if !(*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                                (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid as *mut core::ffi::c_void);
                            }
                            i = i.wrapping_add(1);
                        }

                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).Groups as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).PrimaryGroup.PrimaryGroup.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).PrimaryGroup.PrimaryGroup as *mut core::ffi::c_void);
                    }

                    if !(*Buffer).DefaultDacl.DefaultDacl.is_null() {
                        (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).DefaultDacl.DefaultDacl as *mut core::ffi::c_void);
                    }

                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())(Buffer as *mut core::ffi::c_void);
                }
            }

            return Status;
        }
    }

    *TokenInformation = Buffer;

    'done: loop {
        if !NT_SUCCESS(Status) {
            if !Buffer.is_null() {
                if !(*Buffer).User.User.Sid.is_null() {
                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).User.User.Sid as *mut core::ffi::c_void);
                }

                if !(*Buffer).Groups.is_null() {
                    i = 0;
                    while i < (*(*Buffer).Groups).GroupCount {
                        if !(*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                            (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*(&raw const (*(*Buffer).Groups).Groups[0]).add(i as usize)).Sid as *mut core::ffi::c_void);
                        }
                        i = i.wrapping_add(1);
                    }

                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).Groups as *mut core::ffi::c_void);
                }

                if !(*Buffer).PrimaryGroup.PrimaryGroup.is_null() {
                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).PrimaryGroup.PrimaryGroup as *mut core::ffi::c_void);
                }

                if !(*Buffer).DefaultDacl.DefaultDacl.is_null() {
                    (DispatchTable.FreeLsaHeap.unwrap_unchecked())((*Buffer).DefaultDacl.DefaultDacl as *mut core::ffi::c_void);
                }

                (DispatchTable.FreeLsaHeap.unwrap_unchecked())(Buffer as *mut core::ffi::c_void);
            }
        }

        return Status;
    }
}
unsafe extern "C" fn MsvpChangePassword(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut RequestBuffer: PMSV1_0_CHANGEPASSWORD_REQUEST;
    let PtrOffset: ULONG_PTR;

    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainSid: PRPC_SID = core::ptr::null_mut();
    let mut Names: [RPC_UNICODE_STRING; 1] = core::mem::zeroed();
    let mut RelativeIds: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };

    let mut OldNtPassword: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut NewNtPassword: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut OldLmPassword: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewLmPassword: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut LmPwdString: OEM_STRING = core::mem::zeroed();
    let mut LmPwdBuffer: [i8; 15] = [0; 15];
    let mut OldLmPasswordPresent: BOOLEAN = (FALSE) as u8;
    let mut NewLmPasswordPresent: BOOLEAN = (FALSE) as u8;

    let mut OldLmEncryptedWithNewLm: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewLmEncryptedWithOldLm: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut OldNtEncryptedWithNewNt: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewNtEncryptedWithOldNt: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut pOldLmEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();
    let mut pNewLmEncryptedWithOldLm: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();

    'done: {
        if SubmitBufferLength < core::mem::size_of::<MSV1_0_CHANGEPASSWORD_REQUEST>() as ULONG {
            return STATUS_INVALID_PARAMETER;
        }

        RequestBuffer = ProtocolSubmitBuffer as PMSV1_0_CHANGEPASSWORD_REQUEST;

        PtrOffset = ProtocolSubmitBuffer as ULONG_PTR - ClientBufferBase as ULONG_PTR;

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*RequestBuffer).DomainName));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }
        (*RequestBuffer).DomainName.Buffer = FIXUP_POINTER!((*RequestBuffer).DomainName.Buffer, PtrOffset);
        (*RequestBuffer).DomainName.MaximumLength = (*RequestBuffer).DomainName.Length;

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*RequestBuffer).AccountName));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }
        (*RequestBuffer).AccountName.Buffer = FIXUP_POINTER!((*RequestBuffer).AccountName.Buffer, PtrOffset);
        (*RequestBuffer).AccountName.MaximumLength = (*RequestBuffer).AccountName.Length;

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*RequestBuffer).OldPassword));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }
        (*RequestBuffer).OldPassword.Buffer = FIXUP_POINTER!((*RequestBuffer).OldPassword.Buffer, PtrOffset);
        (*RequestBuffer).OldPassword.MaximumLength = (*RequestBuffer).OldPassword.Length;

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*RequestBuffer).NewPassword));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }
        (*RequestBuffer).NewPassword.Buffer = FIXUP_POINTER!((*RequestBuffer).NewPassword.Buffer, PtrOffset);
        (*RequestBuffer).NewPassword.MaximumLength = (*RequestBuffer).NewPassword.Length;

        Status = SamIConnect(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(ServerHandle),
            SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
            (TRUE) as u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamrLookupDomainInSamServer(
            ServerHandle,
            core::ptr::addr_of_mut!((*RequestBuffer).DomainName) as PRPC_UNICODE_STRING,
            core::ptr::addr_of_mut!(DomainSid),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP,
            DomainSid,
            core::ptr::addr_of_mut!(DomainHandle),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Names[0].Length = (*RequestBuffer).AccountName.Length;
        Names[0].MaximumLength = (*RequestBuffer).AccountName.MaximumLength;
        Names[0].Buffer = (*RequestBuffer).AccountName.Buffer;

        Status = SamrLookupNamesInDomain(
            DomainHandle,
            1,
            Names.as_mut_ptr(),
            core::ptr::addr_of_mut!(RelativeIds),
            core::ptr::addr_of_mut!(Use),
        );
        if !NT_SUCCESS(Status) {
            Status = STATUS_NO_SUCH_USER;
            break 'done;
        }

        if Use.Element.offset(0 as isize).read() != SidTypeUser {
            Status = STATUS_NO_SUCH_USER;
            break 'done;
        }

        Status = SamrOpenUser(
            DomainHandle,
            USER_CHANGE_PASSWORD,
            RelativeIds.Element.offset(0 as isize).read(),
            core::ptr::addr_of_mut!(UserHandle),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SystemFunction007(
            core::ptr::addr_of_mut!((*RequestBuffer).OldPassword),
            core::ptr::addr_of_mut!(OldNtPassword) as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SystemFunction007(
            core::ptr::addr_of_mut!((*RequestBuffer).NewPassword),
            core::ptr::addr_of_mut!(NewNtPassword) as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        LmPwdString.Length = 15;
        LmPwdString.MaximumLength = 15;
        LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
        ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength as usize);

        Status = RtlUpcaseUnicodeStringToOemString(
            core::ptr::addr_of_mut!(LmPwdString),
            core::ptr::addr_of_mut!((*RequestBuffer).OldPassword),
            (FALSE) as u8,
        );
        if NT_SUCCESS(Status) {
            Status = SystemFunction006(
                LmPwdString.Buffer,
                core::ptr::addr_of_mut!(OldLmPassword) as LPSTR,
            );
            if NT_SUCCESS(Status) {
                OldLmPasswordPresent = (TRUE) as u8;
            }
        }

        LmPwdString.Length = 15;
        LmPwdString.MaximumLength = 15;
        LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
        ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength as usize);

        Status = RtlUpcaseUnicodeStringToOemString(
            core::ptr::addr_of_mut!(LmPwdString),
            core::ptr::addr_of_mut!((*RequestBuffer).NewPassword),
            (FALSE) as u8,
        );
        if NT_SUCCESS(Status) {
            Status = SystemFunction006(
                LmPwdString.Buffer,
                core::ptr::addr_of_mut!(NewLmPassword) as LPSTR,
            );
            if NT_SUCCESS(Status) {
                NewLmPasswordPresent = (TRUE) as u8;
            }
        }

        if OldLmPasswordPresent != 0 && NewLmPasswordPresent != 0 {
            Status = SystemFunction012(
                core::ptr::addr_of!(OldLmPassword) as *const BYTE,
                core::ptr::addr_of!(NewLmPassword) as *const BYTE,
                core::ptr::addr_of_mut!(OldLmEncryptedWithNewLm) as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SystemFunction012(
                core::ptr::addr_of!(NewLmPassword) as *const BYTE,
                core::ptr::addr_of!(OldLmPassword) as *const BYTE,
                core::ptr::addr_of_mut!(NewLmEncryptedWithOldLm) as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            pOldLmEncryptedWithNewLm = core::ptr::addr_of_mut!(OldLmEncryptedWithNewLm);
            pNewLmEncryptedWithOldLm = core::ptr::addr_of_mut!(NewLmEncryptedWithOldLm);
        }

        Status = SystemFunction012(
            core::ptr::addr_of!(OldNtPassword) as *const BYTE,
            core::ptr::addr_of!(NewNtPassword) as *const BYTE,
            core::ptr::addr_of_mut!(OldNtEncryptedWithNewNt) as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SystemFunction012(
            core::ptr::addr_of!(NewNtPassword) as *const BYTE,
            core::ptr::addr_of!(OldNtPassword) as *const BYTE,
            core::ptr::addr_of_mut!(NewNtEncryptedWithOldNt) as LPBYTE,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamrChangePasswordUser(
            UserHandle,
            (OldLmPasswordPresent != 0 && NewLmPasswordPresent != 0) as BOOLEAN,
            pOldLmEncryptedWithNewLm,
            pNewLmEncryptedWithOldLm,
            (TRUE) as u8,
            core::ptr::addr_of_mut!(OldNtEncryptedWithNewNt),
            core::ptr::addr_of_mut!(NewNtEncryptedWithOldNt),
            (FALSE) as u8,
            core::ptr::null_mut(),
            (FALSE) as u8,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }
    }

    if !UserHandle.is_null() {
        SamrCloseHandle(core::ptr::addr_of_mut!(UserHandle));
    }

    SamIFree_SAMPR_ULONG_ARRAY(core::ptr::addr_of_mut!(RelativeIds));
    SamIFree_SAMPR_ULONG_ARRAY(core::ptr::addr_of_mut!(Use));

    if !DomainHandle.is_null() {
        SamrCloseHandle(core::ptr::addr_of_mut!(DomainHandle));
    }

    if !DomainSid.is_null() {
        SamIFreeVoid((DomainSid).cast());
    }

    if !ServerHandle.is_null() {
        SamrCloseHandle(core::ptr::addr_of_mut!(ServerHandle));
    }

    Status
}
unsafe extern "C" fn MsvpEnumerateUsers(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut LocalBuffer: PMSV1_0_ENUMUSERS_RESPONSE = core::ptr::null_mut();
    let mut ClientBaseAddress: PVOID = core::ptr::null_mut();
    let BufferLength: ULONG;
    let mut CurrentEntry: PLIST_ENTRY;
    let LogonEntry: PLOGON_LIST_ENTRY;
    let mut LogonCount: ULONG = 0;
    let mut LuidPtr: PLUID;
    let mut EnumPtr: PULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if SubmitBufferLength < core::mem::size_of::<MSV1_0_ENUMUSERS_REQUEST>() as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    RtlAcquireResourceShared(
        core::ptr::addr_of_mut!(LogonListResource),
        (TRUE) as u8,
    );

    CurrentEntry = LogonListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(LogonListHead) {
        let LogonEntry = ((CurrentEntry as usize)
            - core::mem::offset_of!(LOGON_LIST_ENTRY, ListEntry))
            as PLOGON_LIST_ENTRY;

        LogonCount += 1;

        CurrentEntry = (*CurrentEntry).Flink;
    }

    BufferLength = core::mem::size_of::<MSV1_0_ENUMUSERS_RESPONSE>() as ULONG
        + (LogonCount * core::mem::size_of::<LUID>() as ULONG)
        + (LogonCount * core::mem::size_of::<ULONG>() as ULONG);

    LocalBuffer = (DispatchTable.AllocateLsaHeap.unwrap_unchecked())(BufferLength)
        as PMSV1_0_ENUMUSERS_RESPONSE;
    if LocalBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        Status = (DispatchTable.AllocateClientBuffer.unwrap_unchecked())(
            ClientRequest,
            BufferLength,
            core::ptr::addr_of_mut!(ClientBaseAddress),
        );
        if !NT_SUCCESS(Status) {
        } else {
            (*LocalBuffer).MessageType = MsV1_0EnumerateUsers;
            (*LocalBuffer).NumberOfLoggedOnUsers = LogonCount;

            LuidPtr = (LocalBuffer as usize
                + core::mem::size_of::<MSV1_0_ENUMUSERS_RESPONSE>())
                as PLUID;
            EnumPtr = (LuidPtr as usize
                + LogonCount as usize * core::mem::size_of::<LUID>())
                as PULONG;

            (*LocalBuffer).LogonIds = (ClientBaseAddress as usize
                + (LuidPtr as usize - LocalBuffer as usize))
                as PLUID;
            (*LocalBuffer).EnumHandles = (ClientBaseAddress as usize
                + (EnumPtr as usize - LocalBuffer as usize))
                as PULONG;

            CurrentEntry = LogonListHead.Flink;
            while CurrentEntry != core::ptr::addr_of_mut!(LogonListHead) {
                let LogonEntry = ((CurrentEntry as usize)
                    - core::mem::offset_of!(LOGON_LIST_ENTRY, ListEntry))
                    as PLOGON_LIST_ENTRY;

                RtlCopyMemory!(
                    LuidPtr as PVOID,
                    core::ptr::addr_of!((*LogonEntry).LogonId) as PVOID,
                    core::mem::size_of::<LUID>(),
                );
                LuidPtr = LuidPtr.offset(1);

                *EnumPtr = (*LogonEntry).EnumHandle;
                EnumPtr = EnumPtr.offset(1);

                CurrentEntry = (*CurrentEntry).Flink;
            }

            Status = (DispatchTable.CopyToClientBuffer.unwrap_unchecked())(
                ClientRequest,
                BufferLength,
                ClientBaseAddress,
                LocalBuffer as PVOID,
            );
            if !NT_SUCCESS(Status) {
            } else {
                *ProtocolReturnBuffer = ClientBaseAddress;
                *ReturnBufferLength = BufferLength;
                *ProtocolStatus = STATUS_SUCCESS;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

    if !LocalBuffer.is_null() {
        (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if !ClientBaseAddress.is_null() {
            (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
        }
    }

    Status
}
unsafe extern "C" fn MsvpGetUserInfo(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let RequestBuffer: PMSV1_0_GETUSERINFO_REQUEST;
    let LogonEntry: *mut LOGON_LIST_ENTRY;
    let mut LocalBuffer: PMSV1_0_GETUSERINFO_RESPONSE = core::ptr::null_mut();
    let mut ClientBaseAddress: PVOID = core::ptr::null_mut();
    let BufferLength: ULONG;
    let mut BufferPtr: PWSTR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if SubmitBufferLength < core::mem::size_of::<MSV1_0_GETUSERINFO_REQUEST>() as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    RequestBuffer = ProtocolSubmitBuffer as PMSV1_0_GETUSERINFO_REQUEST;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(LogonListResource), (TRUE) as u8);

    LogonEntry = GetLogonByLogonId(core::ptr::addr_of_mut!((*RequestBuffer).LogonId));
    if LogonEntry.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        // goto done
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

        if !LocalBuffer.is_null() {
            (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
        }

        if !NT_SUCCESS(Status) {
            if !ClientBaseAddress.is_null() {
                (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
            }
        }

        return Status;
    }

    BufferLength = core::mem::size_of::<MSV1_0_GETUSERINFO_RESPONSE>() as ULONG +
                   (*LogonEntry).UserName.MaximumLength as ULONG +
                   (*LogonEntry).LogonDomainName.MaximumLength as ULONG +
                   (*LogonEntry).LogonServer.MaximumLength as ULONG;

    LocalBuffer = (DispatchTable.AllocateLsaHeap.unwrap_unchecked())(BufferLength) as PMSV1_0_GETUSERINFO_RESPONSE;
    if LocalBuffer.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        // goto done
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

        if !LocalBuffer.is_null() {
            (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
        }

        if !NT_SUCCESS(Status) {
            if !ClientBaseAddress.is_null() {
                (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
            }
        }

        return Status;
    }

    Status = (DispatchTable.AllocateClientBuffer.unwrap_unchecked())(
        ClientRequest,
        BufferLength,
        core::ptr::addr_of_mut!(ClientBaseAddress),
    );
    if !NT_SUCCESS(Status) {
        // goto done
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

        if !LocalBuffer.is_null() {
            (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
        }

        if !NT_SUCCESS(Status) {
            if !ClientBaseAddress.is_null() {
                (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
            }
        }

        return Status;
    }

    (*LocalBuffer).MessageType = MsV1_0GetUserInfo;

    BufferPtr = (LocalBuffer as ULONG_PTR
        + core::mem::size_of::<MSV1_0_GETUSERINFO_RESPONSE>() as ULONG_PTR) as PWSTR;

    (*LocalBuffer).UserName.Length = (*LogonEntry).UserName.Length;
    (*LocalBuffer).UserName.MaximumLength = (*LogonEntry).UserName.MaximumLength;
    (*LocalBuffer).UserName.Buffer = (ClientBaseAddress as ULONG_PTR + BufferPtr as ULONG_PTR - LocalBuffer as ULONG_PTR) as PWSTR;

    RtlCopyMemory!(
        BufferPtr as PVOID,
        (*LogonEntry).UserName.Buffer as PVOID,
        (*LogonEntry).UserName.MaximumLength as usize,
    );
    BufferPtr = (BufferPtr as ULONG_PTR + (*LocalBuffer).UserName.MaximumLength as ULONG_PTR) as PWSTR;

    (*LocalBuffer).LogonDomainName.Length = (*LogonEntry).LogonDomainName.Length;
    (*LocalBuffer).LogonDomainName.MaximumLength = (*LogonEntry).LogonDomainName.MaximumLength;
    (*LocalBuffer).LogonDomainName.Buffer = (ClientBaseAddress as ULONG_PTR + BufferPtr as ULONG_PTR - LocalBuffer as ULONG_PTR) as PWSTR;

    RtlCopyMemory!(
        BufferPtr as PVOID,
        (*LogonEntry).LogonDomainName.Buffer as PVOID,
        (*LogonEntry).LogonDomainName.MaximumLength as usize,
    );
    BufferPtr = (BufferPtr as ULONG_PTR + (*LocalBuffer).LogonDomainName.MaximumLength as ULONG_PTR) as PWSTR;

    (*LocalBuffer).LogonServer.Length = (*LogonEntry).LogonServer.Length;
    (*LocalBuffer).LogonServer.MaximumLength = (*LogonEntry).LogonServer.MaximumLength;
    (*LocalBuffer).LogonServer.Buffer = (ClientBaseAddress as ULONG_PTR + BufferPtr as ULONG_PTR - LocalBuffer as ULONG_PTR) as PWSTR;

    RtlCopyMemory!(
        BufferPtr as PVOID,
        (*LogonEntry).LogonServer.Buffer as PVOID,
        (*LogonEntry).LogonServer.MaximumLength as usize,
    );

    (*LocalBuffer).LogonType = (*LogonEntry).LogonType;

    Status = (DispatchTable.CopyToClientBuffer.unwrap_unchecked())(
        ClientRequest,
        BufferLength,
        ClientBaseAddress,
        LocalBuffer as PVOID,
    );
    if !NT_SUCCESS(Status) {
        // goto done
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

        if !LocalBuffer.is_null() {
            (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
        }

        if !NT_SUCCESS(Status) {
            if !ClientBaseAddress.is_null() {
                (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
            }
        }

        return Status;
    }

    *ProtocolReturnBuffer = ClientBaseAddress;
    *ReturnBufferLength = BufferLength;
    *ProtocolStatus = STATUS_SUCCESS;

    RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

    if !LocalBuffer.is_null() {
        (DispatchTable.FreeLsaHeap.unwrap_unchecked())(LocalBuffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if !ClientBaseAddress.is_null() {
            (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, ClientBaseAddress);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaApCallPackage(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let MessageType: MSV1_0_PROTOCOL_MESSAGE_TYPE;

    if SubmitBufferLength < core::mem::size_of::<MSV1_0_PROTOCOL_MESSAGE_TYPE>() as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    MessageType = *(ProtocolSubmitBuffer as PMSV1_0_PROTOCOL_MESSAGE_TYPE);

    *ProtocolReturnBuffer = core::ptr::null_mut();
    *ReturnBufferLength = 0;

    match MessageType {
        MsV1_0Lm20ChallengeRequest | MsV1_0Lm20GetChallengeResponse => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        MsV1_0EnumerateUsers => {
            Status = MsvpEnumerateUsers(
                ClientRequest,
                ProtocolSubmitBuffer,
                ClientBufferBase,
                SubmitBufferLength,
                ProtocolReturnBuffer,
                ReturnBufferLength,
                ProtocolStatus,
            );
        }

        MsV1_0GetUserInfo => {
            Status = MsvpGetUserInfo(
                ClientRequest,
                ProtocolSubmitBuffer,
                ClientBufferBase,
                SubmitBufferLength,
                ProtocolReturnBuffer,
                ReturnBufferLength,
                ProtocolStatus,
            );
        }

        MsV1_0ReLogonUsers => {
            Status = STATUS_INVALID_PARAMETER;
        }

        MsV1_0ChangePassword => {
            Status = MsvpChangePassword(
                ClientRequest,
                ProtocolSubmitBuffer,
                ClientBufferBase,
                SubmitBufferLength,
                ProtocolReturnBuffer,
                ReturnBufferLength,
                ProtocolStatus,
            );
        }

        MsV1_0ChangeCachedPassword
        | MsV1_0GenericPassthrough
        | MsV1_0CacheLogon
        | MsV1_0SubAuth
        | MsV1_0DeriveCredential
        | MsV1_0CacheLookup => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaApCallPackagePassthrough(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    // TRACE!("LsaApCallPackagePassthrough()\n");
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaApCallPackageUntrusted(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferLength: ULONG,
    ProtocolReturnBuffer: *mut PVOID,
    ReturnBufferLength: PULONG,
    ProtocolStatus: PNTSTATUS,
) -> NTSTATUS {
    let MessageType: ULONG;
    let Status: NTSTATUS;

    if (SubmitBufferLength as usize) < core::mem::size_of::<MSV1_0_PROTOCOL_MESSAGE_TYPE>() {
        return STATUS_INVALID_PARAMETER;
    }

    MessageType = *(ProtocolSubmitBuffer as PMSV1_0_PROTOCOL_MESSAGE_TYPE) as ULONG;

    *ProtocolReturnBuffer = core::ptr::null_mut();
    *ReturnBufferLength = 0;

    if MessageType == MsV1_0ChangePassword {
        Status = MsvpChangePassword(
            ClientRequest,
            ProtocolSubmitBuffer,
            ClientBufferBase,
            SubmitBufferLength,
            ProtocolReturnBuffer,
            ReturnBufferLength,
            ProtocolStatus,
        );
    } else {
        Status = STATUS_ACCESS_DENIED;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaApInitializePackage(
    AuthenticationPackageId: u32,
    LsaDispatchTable: PLSA_DISPATCH_TABLE,
    Database: PLSA_STRING,
    Confidentiality: PLSA_STRING,
    AuthenticationPackageName: *mut PLSA_STRING,
) -> i32 {
    let NameString: PANSI_STRING;
    let NameBuffer: PCHAR;

    if PackageInitialized == 0 {
        InitializeListHead(&mut LogonListHead);
        RtlInitializeResource(&mut LogonListResource);
        EnumCounter = 0;
        PackageInitialized = (TRUE) as i32;
    }

    DispatchTable.CreateLogonSession = (*LsaDispatchTable).CreateLogonSession;
    DispatchTable.DeleteLogonSession = (*LsaDispatchTable).DeleteLogonSession;
    DispatchTable.AddCredential = (*LsaDispatchTable).AddCredential;
    DispatchTable.GetCredentials = (*LsaDispatchTable).GetCredentials;
    DispatchTable.DeleteCredential = (*LsaDispatchTable).DeleteCredential;
    DispatchTable.AllocateLsaHeap = (*LsaDispatchTable).AllocateLsaHeap;
    DispatchTable.FreeLsaHeap = (*LsaDispatchTable).FreeLsaHeap;
    DispatchTable.AllocateClientBuffer = (*LsaDispatchTable).AllocateClientBuffer;
    DispatchTable.FreeClientBuffer = (*LsaDispatchTable).FreeClientBuffer;
    DispatchTable.CopyToClientBuffer = (*LsaDispatchTable).CopyToClientBuffer;
    DispatchTable.CopyFromClientBuffer = (*LsaDispatchTable).CopyFromClientBuffer;

    NameString = (DispatchTable.AllocateLsaHeap.unwrap_unchecked())(
        core::mem::size_of::<LSA_STRING>() as ULONG,
    ) as PANSI_STRING;
    if NameString.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    NameBuffer = (DispatchTable.AllocateLsaHeap.unwrap_unchecked())(MSV1_0_PACKAGE_NAME.len() as ULONG)
        as PCHAR;
    if NameBuffer.is_null() {
        (DispatchTable.FreeLsaHeap.unwrap_unchecked())(NameString as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    strcpy(NameBuffer, MSV1_0_PACKAGE_NAME.as_ptr() as *const c_char);

    RtlInitAnsiString(NameString, NameBuffer);

    *AuthenticationPackageName = NameString as PLSA_STRING;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn LsaApLogonTerminated(
    LogonId: PLUID,
) {
    let LogonEntry: *mut LOGON_LIST_ENTRY;

    LogonEntry = GetLogonByLogonId(LogonId);
    if LogonEntry != core::ptr::null_mut() {
        RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(LogonListResource), (TRUE) as u8);
        RemoveEntryList(core::ptr::addr_of_mut!((*LogonEntry).ListEntry));
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));

        if (*LogonEntry).UserName.Buffer != core::ptr::null_mut() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, (*LogonEntry).UserName.Buffer as *mut core::ffi::c_void);
        }

        if (*LogonEntry).LogonDomainName.Buffer != core::ptr::null_mut() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, (*LogonEntry).LogonDomainName.Buffer as *mut core::ffi::c_void);
        }

        if (*LogonEntry).LogonServer.Buffer != core::ptr::null_mut() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, (*LogonEntry).LogonServer.Buffer as *mut core::ffi::c_void);
        }

        RtlFreeHeap(RtlGetProcessHeap!(), 0, LogonEntry as *mut core::ffi::c_void);
    }
}
unsafe extern "C" fn LsaApLogonUserEx2_Network(
    ClientRequest: PLSA_CLIENT_REQUEST,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferSize: ULONG,
    ComputerName: PUNICODE_STRING,
    LogonUserRef: *mut PUNICODE_STRING,
    LogonDomainRef: *mut PUNICODE_STRING,
    LogonPwdData: PLSA_SAM_PWD_DATA,
    UserHandlePtr: *mut SAMPR_HANDLE,
    UserInfoPtr: *mut PSAMPR_USER_INFO_BUFFER,
    AccountDomainSidPtr: *mut PRPC_SID,
    SpecialAccount: PBOOL,
    LogonProfile: *mut PMSV1_0_LM20_LOGON_PROFILE,
    LogonProfileSize: PULONG,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let LogonInfo: PMSV1_0_LM20_LOGON;
    let PtrOffset: ULONG_PTR;

    *LogonProfile = core::ptr::null_mut();
    *LogonProfileSize = 0;
    *UserInfoPtr = core::ptr::null_mut();
    *AccountDomainSidPtr = core::ptr::null_mut();
    *SpecialAccount = (FALSE) as i32;
    LogonInfo = ProtocolSubmitBuffer as PMSV1_0_LM20_LOGON;

    if SubmitBufferSize < core::mem::size_of::<MSV1_0_LM20_LOGON>() as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    PtrOffset = (ProtocolSubmitBuffer as ULONG_PTR).wrapping_sub(ClientBufferBase as ULONG_PTR);

    if (!NtlmFixupAndValidateUStr(
        core::ptr::addr_of_mut!((*LogonInfo).LogonDomainName),
        (PtrOffset) as usize,
    ))
        || (!NtlmFixupAndValidateUStr(core::ptr::addr_of_mut!((*LogonInfo).UserName), (PtrOffset) as usize))
        || (!NtlmFixupAndValidateUStr(core::ptr::addr_of_mut!((*LogonInfo).Workstation), (PtrOffset) as usize))
        || (!NtlmFixupAStr(
            core::ptr::addr_of_mut!((*LogonInfo).CaseSensitiveChallengeResponse),
            (PtrOffset) as usize,
        ))
        || (!NtlmFixupAStr(
            core::ptr::addr_of_mut!((*LogonInfo).CaseInsensitiveChallengeResponse),
            (PtrOffset) as usize,
        ))
    {
        return STATUS_INVALID_PARAMETER;
    }

    (*LogonPwdData).IsNetwork = (TRUE) as i32;
    (*LogonPwdData).LogonInfo = LogonInfo;
    (*LogonPwdData).ComputerName = ComputerName;
    Status = SamValidateUser(
        Network,
        core::ptr::addr_of_mut!((*LogonInfo).UserName),
        core::ptr::addr_of_mut!((*LogonInfo).LogonDomainName),
        LogonPwdData,
        ComputerName,
        SpecialAccount,
        AccountDomainSidPtr,
        UserHandlePtr,
        UserInfoPtr,
        SubStatus,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if ((*LogonInfo).ParameterControl & MSV1_0_RETURN_PROFILE_PATH) != 0 {
        Status = BuildLm20LogonProfileBuffer(
            ClientRequest,
            *UserInfoPtr,
            LogonPwdData,
            LogonProfile,
            LogonProfileSize,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    *LogonUserRef = core::ptr::addr_of_mut!((*LogonInfo).UserName);
    *LogonDomainRef = core::ptr::addr_of_mut!((*LogonInfo).LogonDomainName);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn LsaApLogonUserEx2(
    ClientRequest: PLSA_CLIENT_REQUEST,
    LogonType: SECURITY_LOGON_TYPE,
    ProtocolSubmitBuffer: PVOID,
    ClientBufferBase: PVOID,
    SubmitBufferSize: ULONG,
    ProfileBuffer: *mut PVOID,
    ProfileBufferSize: PULONG,
    LogonId: PLUID,
    SubStatus: PNTSTATUS,
    TokenInformationType: PLSA_TOKEN_INFORMATION_TYPE,
    TokenInformation: *mut PVOID,
    AccountName: *mut PUNICODE_STRING,
    AuthenticatingAuthority: *mut PUNICODE_STRING,
    MachineName: *mut PUNICODE_STRING,
    PrimaryCredentials: PSECPKG_PRIMARY_CRED,
    SupplementalCredentials: *mut PSECPKG_SUPPLEMENTAL_CRED_ARRAY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ComputerName: _UNICODE_STRING = core::mem::zeroed();
    let mut ComputerNameData: [u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize] = core::mem::zeroed();
    let mut LogonUserName: PUNICODE_STRING = core::ptr::null_mut();
    let mut LogonPwdData: LSA_SAM_PWD_DATA = core::mem::zeroed();
    let mut LogonDomain: PUNICODE_STRING = core::ptr::null_mut();
    let mut UserHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut AccountDomainSid: PRPC_SID = core::ptr::null_mut();
    let mut UserInfo: PSAMPR_USER_INFO_BUFFER = core::ptr::null_mut();
    let mut SessionCreated: BOOLEAN = (FALSE) as u8;
    let mut ComputerNameSize: DWORD;
    let mut SpecialAccount: BOOL = (FALSE) as i32;
    let mut LogonPassHash: UCHAR;
    let mut ErasePassword: PUNICODE_STRING = core::ptr::null_mut();
    let mut LogonEntry: PLOGON_LIST_ENTRY = core::ptr::null_mut();

    *ProfileBuffer = core::ptr::null_mut();
    *ProfileBufferSize = 0;
    *SubStatus = STATUS_SUCCESS;
    *AccountName = core::ptr::null_mut();
    *AuthenticatingAuthority = core::ptr::null_mut();

    ComputerNameSize = ARRAYSIZE!(ComputerNameData);
    if GetComputerNameW(ComputerNameData.as_mut_ptr(), core::ptr::addr_of_mut!(ComputerNameSize)) == 0 {
        return STATUS_INTERNAL_ERROR;
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ComputerName), ComputerNameData.as_mut_ptr());

    'done: {
    if LogonType == Interactive ||
        LogonType == Batch ||
        LogonType == Service {
        let mut LogonInfo: PMSV1_0_INTERACTIVE_LOGON;
        let mut PtrOffset: ULONG_PTR;

        if SubmitBufferSize < core::mem::size_of::<MSV1_0_INTERACTIVE_LOGON>() as ULONG {
            return STATUS_INVALID_PARAMETER;
        }

        LogonInfo = ProtocolSubmitBuffer as PMSV1_0_INTERACTIVE_LOGON;

        if (*LogonInfo).MessageType != MsV1_0InteractiveLogon &&
            (*LogonInfo).MessageType != MsV1_0WorkstationUnlockLogon {
            return STATUS_BAD_VALIDATION_CLASS;
        }

        PtrOffset = ProtocolSubmitBuffer as ULONG_PTR - ClientBufferBase as ULONG_PTR;

        if (*LogonInfo).LogonDomainName.Length != 0 {
            (*LogonInfo).LogonDomainName.Buffer = FIXUP_POINTER!((*LogonInfo).LogonDomainName.Buffer, PtrOffset);
            (*LogonInfo).LogonDomainName.MaximumLength = (*LogonInfo).LogonDomainName.Length;
        } else {
            (*LogonInfo).LogonDomainName.Buffer = core::ptr::null_mut();
            (*LogonInfo).LogonDomainName.MaximumLength = 0;
        }
        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*LogonInfo).LogonDomainName));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }

        (*LogonInfo).UserName.Buffer = FIXUP_POINTER!((*LogonInfo).UserName.Buffer, PtrOffset);
        (*LogonInfo).UserName.MaximumLength = (*LogonInfo).UserName.Length;

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*LogonInfo).UserName));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }

        LogonPassHash = (((*LogonInfo).Password.Length >> 8) & 0xFF) as UCHAR;
        (*LogonInfo).Password.Length = (*LogonInfo).Password.Length & 0xFF;

        if (*LogonInfo).Password.Length != 0 {
            (*LogonInfo).Password.Buffer = FIXUP_POINTER!((*LogonInfo).Password.Buffer, PtrOffset);
            (*LogonInfo).Password.MaximumLength = (*LogonInfo).Password.Length;
        } else {
            (*LogonInfo).Password.Buffer = core::ptr::null_mut();
            (*LogonInfo).Password.MaximumLength = 0;
        }

        if LogonPassHash > 0 {
            RtlRunDecodeUnicodeString(LogonPassHash, core::ptr::addr_of_mut!((*LogonInfo).Password));
        }

        ErasePassword = core::ptr::addr_of_mut!((*LogonInfo).Password);

        Status = RtlValidateUnicodeString(0, core::ptr::addr_of_mut!((*LogonInfo).Password));
        if !NT_SUCCESS(Status) {
            return STATUS_INVALID_PARAMETER;
        }

        LogonUserName = core::ptr::addr_of_mut!((*LogonInfo).UserName);
        LogonDomain = core::ptr::addr_of_mut!((*LogonInfo).LogonDomainName);
        LogonPwdData.IsNetwork = (FALSE) as i32;
        LogonPwdData.PlainPwd = core::ptr::addr_of_mut!((*LogonInfo).Password);
        LogonPwdData.ComputerName = core::ptr::addr_of_mut!(ComputerName);
    } else if LogonType == Network {
        Status = LsaApLogonUserEx2_Network(
            ClientRequest,
            ProtocolSubmitBuffer,
            ClientBufferBase,
            SubmitBufferSize,
            core::ptr::addr_of_mut!(ComputerName),
            core::ptr::addr_of_mut!(LogonUserName),
            core::ptr::addr_of_mut!(LogonDomain),
            core::ptr::addr_of_mut!(LogonPwdData),
            core::ptr::addr_of_mut!(UserHandle),
            core::ptr::addr_of_mut!(UserInfo),
            core::ptr::addr_of_mut!(AccountDomainSid),
            core::ptr::addr_of_mut!(SpecialAccount),
            ProfileBuffer as *mut PMSV1_0_LM20_LOGON_PROFILE,
            ProfileBufferSize,
            SubStatus,
        );
        if !NT_SUCCESS(Status) {
            break 'done; // C: goto done
        }
    } else {
        return STATUS_NOT_IMPLEMENTED;
    }

    Status = SamValidateUser(
        LogonType,
        LogonUserName,
        LogonDomain,
        core::ptr::addr_of_mut!(LogonPwdData),
        core::ptr::addr_of_mut!(ComputerName),
        core::ptr::addr_of_mut!(SpecialAccount),
        core::ptr::addr_of_mut!(AccountDomainSid),
        core::ptr::addr_of_mut!(UserHandle),
        core::ptr::addr_of_mut!(UserInfo),
        SubStatus,
    );
    if !NT_SUCCESS(Status) {
        break 'done; // C: goto done
    }

    Status = NtAllocateLocallyUniqueId(LogonId);
    if !NT_SUCCESS(Status) {
        break 'done; // C: goto done
    }

    Status = DispatchTable.CreateLogonSession.unwrap_unchecked()(LogonId);
    if !NT_SUCCESS(Status) {
        break 'done; // C: goto done
    }

    SessionCreated = (TRUE) as u8;

    LogonEntry = (RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, core::mem::size_of::<LOGON_LIST_ENTRY>() as SIZE_T)).cast();
    if !LogonEntry.is_null() {
        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*LogonEntry).LogonId) as PVOID,
            LogonId as PVOID,
            core::mem::size_of::<LUID>() as SIZE_T,
        );
        (*LogonEntry).EnumHandle = EnumCounter;
        EnumCounter = EnumCounter.wrapping_add(1);

        (*LogonEntry).UserName.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, (*LogonUserName).MaximumLength as SIZE_T) as PWSTR;
        if !(*LogonEntry).UserName.Buffer.is_null() {
            (*LogonEntry).UserName.MaximumLength = (*LogonUserName).MaximumLength;
            RtlCopyUnicodeString(core::ptr::addr_of_mut!((*LogonEntry).UserName), LogonUserName);
        }

        (*LogonEntry).LogonDomainName.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, (*LogonDomain).MaximumLength as SIZE_T) as PWSTR;
        if !(*LogonEntry).LogonDomainName.Buffer.is_null() {
            (*LogonEntry).LogonDomainName.MaximumLength = (*LogonDomain).MaximumLength;
            RtlCopyUnicodeString(core::ptr::addr_of_mut!((*LogonEntry).LogonDomainName), LogonDomain);
        }

        (*LogonEntry).LogonServer.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, ComputerName.MaximumLength as SIZE_T) as PWSTR;
        if !(*LogonEntry).LogonServer.Buffer.is_null() {
            (*LogonEntry).LogonServer.MaximumLength = ComputerName.MaximumLength;
            RtlCopyUnicodeString(core::ptr::addr_of_mut!((*LogonEntry).LogonServer), core::ptr::addr_of_mut!(ComputerName));
        }

        (*LogonEntry).LogonType = LogonType;

        RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(LogonListResource), (TRUE) as u8);
        InsertTailList(core::ptr::addr_of_mut!(LogonListHead), core::ptr::addr_of_mut!((*LogonEntry).ListEntry));
        RtlReleaseResource(core::ptr::addr_of_mut!(LogonListResource));
    }

    if LogonType == Interactive || LogonType == Batch || LogonType == Service {
        Status = BuildInteractiveProfileBuffer(
            ClientRequest,
            UserInfo,
            ComputerName.Buffer,
            ProfileBuffer as *mut PMSV1_0_INTERACTIVE_PROFILE,
            ProfileBufferSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done; // C: goto done
        }
    } else if LogonType == Network {
    }

    *TokenInformationType = LsaTokenInformationV1;

    Status = BuildTokenInformationBuffer(
        TokenInformation as *mut PLSA_TOKEN_INFORMATION_V1,
        AccountDomainSid,
        UserInfo,
        SpecialAccount,
    );
    if !NT_SUCCESS(Status) {
        break 'done; // C: goto done
    }

    } // 'done

    {
        if !ErasePassword.is_null() {
            RtlEraseUnicodeString(ErasePassword);
        }

        if !UserHandle.is_null() && (Status == STATUS_SUCCESS || Status == STATUS_WRONG_PASSWORD) {
            let mut InternalInfo: SAMPR_USER_INFO_BUFFER = core::mem::zeroed();

            RtlZeroMemory!(
                core::ptr::addr_of_mut!(InternalInfo) as PVOID,
                core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T,
            );

            if Status == STATUS_SUCCESS {
                InternalInfo.Internal2.Flags = USER_LOGON_SUCCESS;
            } else {
                InternalInfo.Internal2.Flags = USER_LOGON_BAD_PASSWORD;
            }

            SamrSetInformationUser(UserHandle, UserInternal2Information, core::ptr::addr_of_mut!(InternalInfo));
        }

        if NT_SUCCESS(Status) {
            *AccountName = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(
                core::mem::size_of::<_UNICODE_STRING>() as ULONG,
            ) as PUNICODE_STRING;
            if !LogonUserName.is_null() && !(*AccountName).is_null() {
                (**AccountName).Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()((*LogonUserName).Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG) as PWSTR;
                if !(**AccountName).Buffer.is_null() {
                    (**AccountName).MaximumLength = (*LogonUserName).Length + core::mem::size_of::<WCHAR>() as USHORT;
                    RtlCopyUnicodeString(*AccountName, LogonUserName);
                }
            }

            *AuthenticatingAuthority = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(
                core::mem::size_of::<_UNICODE_STRING>() as ULONG,
            ) as PUNICODE_STRING;
            if !LogonDomain.is_null() && !(*AuthenticatingAuthority).is_null() {
                (**AuthenticatingAuthority).Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()((*LogonDomain).Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG) as PWSTR;
                if !(**AuthenticatingAuthority).Buffer.is_null() {
                    (**AuthenticatingAuthority).MaximumLength = (*LogonDomain).Length + core::mem::size_of::<WCHAR>() as USHORT;
                    RtlCopyUnicodeString(*AuthenticatingAuthority, LogonDomain);
                }
            }

            *MachineName = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(
                core::mem::size_of::<_UNICODE_STRING>() as ULONG,
            ) as PUNICODE_STRING;
            if !(*MachineName).is_null() {
                (**MachineName).Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(ComputerName.MaximumLength as ULONG) as PWSTR;
                if !(**MachineName).Buffer.is_null() {
                    (**MachineName).MaximumLength = ComputerName.MaximumLength;
                    (**MachineName).Length = ComputerName.Length;
                    RtlCopyMemory!(
                        (**MachineName).Buffer as PVOID,
                        ComputerName.Buffer as PVOID,
                        ComputerName.MaximumLength as SIZE_T,
                    );
                }
            }
        }

        if !NT_SUCCESS(Status) {
            if SessionCreated != (FALSE) as u8 {
                DispatchTable.DeleteLogonSession.unwrap_unchecked()(LogonId);
            }

            if !(*ProfileBuffer).is_null() {
                DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, *ProfileBuffer);
                *ProfileBuffer = core::ptr::null_mut();
            }
        }

        if !UserHandle.is_null() {
            SamrCloseHandle(core::ptr::addr_of_mut!(UserHandle));
        }

        SamIFree_SAMPR_USER_INFO_BUFFER(UserInfo, UserAllInformation);

        if !AccountDomainSid.is_null() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, AccountDomainSid as PVOID);
        }

        if Status == STATUS_NO_SUCH_USER || Status == STATUS_WRONG_PASSWORD {
            *SubStatus = Status;
            Status = STATUS_LOGON_FAILURE;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SpLsaModeInitialize(
    LsaVersion: ULONG,
    PackageVersion: PULONG,
    ppTables: *mut PSECPKG_FUNCTION_TABLE,
    pcTables: PULONG,
) -> NTSTATUS {
    if LsaVersion != SECPKG_INTERFACE_VERSION {
        return STATUS_INVALID_PARAMETER;
    }

    *PackageVersion = SECPKG_INTERFACE_VERSION;

    *ppTables = NtlmLsaFn.as_mut_ptr();
    *pcTables = 1;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SpUserModeInitialize(
    LsaVersion: u32,
    PackageVersion: *mut u32,
    ppTables: *mut *mut SECPKG_USER_FUNCTION_TABLE,
    pcTables: *mut u32,
) -> i32 {
    if LsaVersion != SECPKG_INTERFACE_VERSION {
        return STATUS_INVALID_PARAMETER;
    }

    *PackageVersion = SECPKG_INTERFACE_VERSION;

    *ppTables = NtlmUsrFn.as_mut_ptr();
    *pcTables = 1;

    return STATUS_SUCCESS;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/msv1_0/msv1_0.c
//  * PURPOSE:     Main file
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msv1_0);
//
// typedef struct _LOGON_LIST_ENTRY
// {
//     LIST_ENTRY ListEntry;
//     LUID LogonId;
//     ULONG EnumHandle;
//     UNICODE_STRING UserName;
//     UNICODE_STRING LogonDomainName;
//     UNICODE_STRING LogonServer;
//     SECURITY_LOGON_TYPE LogonType;
// } LOGON_LIST_ENTRY, *PLOGON_LIST_ENTRY;
//
// /* GLOBALS *****************************************************************/
//
// BOOL PackageInitialized = FALSE;
// LIST_ENTRY LogonListHead;
// RTL_RESOURCE LogonListResource;
// ULONG EnumCounter;
//
// /* FUNCTIONS ***************************************************************/
//
// static
// PLOGON_LIST_ENTRY
// GetLogonByLogonId(
//      PLUID LogonId)
// {
//     PLOGON_LIST_ENTRY LogonEntry;
//     PLIST_ENTRY CurrentEntry;
//
//     CurrentEntry = LogonListHead.Flink;
//     while (CurrentEntry != &LogonListHead)
//     {
//         LogonEntry = CONTAINING_RECORD(CurrentEntry,
//                                        LOGON_LIST_ENTRY,
//                                        ListEntry);
//
//         if ((LogonEntry->LogonId.HighPart == LogonId->HighPart) &&
//             (LogonEntry->LogonId.LowPart == LogonId->LowPart))
//             return LogonEntry;
//
//         CurrentEntry = CurrentEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// static
// NTSTATUS
// BuildInteractiveProfileBuffer( PLSA_CLIENT_REQUEST ClientRequest,
//                                PSAMPR_USER_INFO_BUFFER UserInfo,
//                                PWSTR ComputerName,
//                                PMSV1_0_INTERACTIVE_PROFILE *ProfileBuffer,
//                                PULONG ProfileBufferLength)
// {
//     PMSV1_0_INTERACTIVE_PROFILE LocalBuffer = NULL;
//     PVOID ClientBaseAddress = NULL;
//     LPWSTR Ptr;
//     ULONG BufferLength;
//     USHORT ComputerNameLength;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     *ProfileBuffer = NULL;
//     *ProfileBufferLength = 0;
//
//     if (UIntPtrToUShort(wcslen(ComputerName), &ComputerNameLength) != S_OK)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     BufferLength = sizeof(MSV1_0_INTERACTIVE_PROFILE) +
//                    UserInfo->All.FullName.Length + sizeof(WCHAR) +
//                    UserInfo->All.HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->All.HomeDirectoryDrive.Length + sizeof(WCHAR) +
//                    UserInfo->All.ScriptPath.Length + sizeof(WCHAR) +
//                    UserInfo->All.ProfilePath.Length + sizeof(WCHAR) +
//                    ((ComputerNameLength + 3) * sizeof(WCHAR));
//
//     LocalBuffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(BufferLength);
//     if (LocalBuffer == NULL)
//     {
//         TRACE("Failed to allocate the local buffer!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = DispatchTable.AllocateClientBuffer.unwrap_unchecked()(ClientRequest,
//                                                 BufferLength,
//                                                 &ClientBaseAddress);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("DispatchTable.AllocateClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("ClientBaseAddress: %p\n", ClientBaseAddress);
//
//     Ptr = (LPWSTR)((ULONG_PTR)LocalBuffer + sizeof(MSV1_0_INTERACTIVE_PROFILE));
//
//     LocalBuffer->MessageType = MsV1_0InteractiveProfile;
//     LocalBuffer->LogonCount = UserInfo->All.LogonCount;
//     LocalBuffer->BadPasswordCount = UserInfo->All.BadPasswordCount;
//
//     LocalBuffer->LogonTime.LowPart = UserInfo->All.LastLogon.LowPart;
//     LocalBuffer->LogonTime.HighPart = UserInfo->All.LastLogon.HighPart;
//
//     LocalBuffer->LogoffTime.LowPart = UserInfo->All.AccountExpires.LowPart;
//     LocalBuffer->LogoffTime.HighPart = UserInfo->All.AccountExpires.HighPart;
//
//     LocalBuffer->KickOffTime.LowPart = UserInfo->All.AccountExpires.LowPart;
//     LocalBuffer->KickOffTime.HighPart = UserInfo->All.AccountExpires.HighPart;
//
//     LocalBuffer->PasswordLastSet.LowPart = UserInfo->All.PasswordLastSet.LowPart;
//     LocalBuffer->PasswordLastSet.HighPart = UserInfo->All.PasswordLastSet.HighPart;
//
//     LocalBuffer->PasswordCanChange.LowPart = UserInfo->All.PasswordCanChange.LowPart;
//     LocalBuffer->PasswordCanChange.HighPart = UserInfo->All.PasswordCanChange.HighPart;
//
//     LocalBuffer->PasswordMustChange.LowPart = UserInfo->All.PasswordMustChange.LowPart;
//     LocalBuffer->PasswordMustChange.HighPart = UserInfo->All.PasswordMustChange.HighPart;
//
//     LocalBuffer->LogonScript.Length = UserInfo->All.ScriptPath.Length;
//     LocalBuffer->LogonScript.MaximumLength = UserInfo->All.ScriptPath.Length + sizeof(WCHAR);
//     LocalBuffer->LogonScript.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     memcpy(Ptr,
//            UserInfo->All.ScriptPath.Buffer,
//            UserInfo->All.ScriptPath.Length);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + LocalBuffer->LogonScript.MaximumLength);
//
//     LocalBuffer->HomeDirectory.Length = UserInfo->All.HomeDirectory.Length;
//     LocalBuffer->HomeDirectory.MaximumLength = UserInfo->All.HomeDirectory.Length + sizeof(WCHAR);
//     LocalBuffer->HomeDirectory.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     memcpy(Ptr,
//            UserInfo->All.HomeDirectory.Buffer,
//            UserInfo->All.HomeDirectory.Length);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + LocalBuffer->HomeDirectory.MaximumLength);
//
//     LocalBuffer->FullName.Length = UserInfo->All.FullName.Length;
//     LocalBuffer->FullName.MaximumLength = UserInfo->All.FullName.Length + sizeof(WCHAR);
//     LocalBuffer->FullName.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     memcpy(Ptr,
//            UserInfo->All.FullName.Buffer,
//            UserInfo->All.FullName.Length);
//     TRACE("FullName.Buffer: %p\n", LocalBuffer->FullName.Buffer);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + LocalBuffer->FullName.MaximumLength);
//
//     LocalBuffer->ProfilePath.Length = UserInfo->All.ProfilePath.Length;
//     LocalBuffer->ProfilePath.MaximumLength = UserInfo->All.ProfilePath.Length + sizeof(WCHAR);
//     LocalBuffer->ProfilePath.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     memcpy(Ptr,
//            UserInfo->All.ProfilePath.Buffer,
//            UserInfo->All.ProfilePath.Length);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + LocalBuffer->ProfilePath.MaximumLength);
//
//     LocalBuffer->HomeDirectoryDrive.Length = UserInfo->All.HomeDirectoryDrive.Length;
//     LocalBuffer->HomeDirectoryDrive.MaximumLength = UserInfo->All.HomeDirectoryDrive.Length + sizeof(WCHAR);
//     LocalBuffer->HomeDirectoryDrive.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     memcpy(Ptr,
//            UserInfo->All.HomeDirectoryDrive.Buffer,
//            UserInfo->All.HomeDirectoryDrive.Length);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + LocalBuffer->HomeDirectoryDrive.MaximumLength);
//
//     LocalBuffer->LogonServer.Length = (ComputerNameLength + 2) * sizeof(WCHAR);
//     LocalBuffer->LogonServer.MaximumLength = LocalBuffer->LogonServer.Length + sizeof(WCHAR);
//     LocalBuffer->LogonServer.Buffer = (LPWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)Ptr - (ULONG_PTR)LocalBuffer);
//     wcscpy(Ptr, L"\\");
//     wcscat(Ptr, ComputerName);
//
//     LocalBuffer->UserFlags = 0;
//
//     Status = DispatchTable.CopyToClientBuffer.unwrap_unchecked()(ClientRequest,
//                                               BufferLength,
//                                               ClientBaseAddress,
//                                               LocalBuffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("DispatchTable.CopyToClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *ProfileBuffer = (PMSV1_0_INTERACTIVE_PROFILE)ClientBaseAddress;
//     *ProfileBufferLength = BufferLength;
//
// done:
//     if (LocalBuffer != NULL)
//         DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (ClientBaseAddress != NULL)
//             DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest,
//                                            ClientBaseAddress);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// BuildLm20LogonProfileBuffer(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      PSAMPR_USER_INFO_BUFFER UserInfo,
//      PLSA_SAM_PWD_DATA LogonPwdData,
//      PMSV1_0_LM20_LOGON_PROFILE *ProfileBuffer,
//      PULONG ProfileBufferLength)
// {
//     PMSV1_0_LM20_LOGON_PROFILE LocalBuffer;
//     NTLM_CLIENT_BUFFER Buffer;
//     PBYTE PtrOffset;
//     ULONG BufferLength;
//     NTSTATUS Status = STATUS_SUCCESS;
//     UNICODE_STRING ComputerNameUCS;
//
//     *ProfileBuffer = NULL;
//     *ProfileBufferLength = 0;
//
//     if (!NtlmUStrAlloc(&ComputerNameUCS, LogonPwdData->ComputerName->Length + sizeof(WCHAR) * 3, 0))
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//     Status = RtlAppendUnicodeToString(&ComputerNameUCS, L"\\\\");
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlAppendUnicodeToString failed 0x%lx\n", Status);
//         goto done;
//     }
//     Status = RtlAppendUnicodeStringToString(&ComputerNameUCS, LogonPwdData->ComputerName);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlAppendUnicodeStringToString failed 0x%lx\n", Status);
//         goto done;
//     }
//
//     BufferLength = sizeof(MSV1_0_LM20_LOGON_PROFILE) + ComputerNameUCS.Length + sizeof(WCHAR);
//
//     Status = NtlmAllocateClientBuffer(ClientRequest, BufferLength, &Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("DispatchTable.AllocateClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("ClientBaseAddress: %p\n", Buffer.ClientBaseAddress);
//
//     LocalBuffer = (PMSV1_0_LM20_LOGON_PROFILE)Buffer.LocalBuffer;
//     PtrOffset = (PBYTE)(LocalBuffer + 1);
//
//     LocalBuffer->MessageType = MsV1_0Lm20LogonProfile;
//     LocalBuffer->KickOffTime.LowPart = UserInfo->All.AccountExpires.LowPart;
//     LocalBuffer->KickOffTime.HighPart = UserInfo->All.AccountExpires.HighPart;
//     LocalBuffer->LogoffTime.LowPart = UserInfo->All.AccountExpires.LowPart;
//     LocalBuffer->LogoffTime.HighPart = UserInfo->All.AccountExpires.HighPart;
//
//     memcpy(LocalBuffer->UserSessionKey,
//            &LogonPwdData->UserSessionKey,
//            MSV1_0_USER_SESSION_KEY_LENGTH);
//
//     //FIXME: Set Domainname if we domain joined
//     //       what to do if not? WORKGROUP
//     RtlInitUnicodeString(&LocalBuffer->LogonDomainName, NULL);
//
//     memcpy(LocalBuffer->LanmanSessionKey,
//            &LogonPwdData->LanmanSessionKey,
//            MSV1_0_LANMAN_SESSION_KEY_LENGTH);
//
//     if (!NtlmUStrWriteToStruct(LocalBuffer,
//                                BufferLength,
//                                &LocalBuffer->LogonServer,
//                                &ComputerNameUCS,
//                                &PtrOffset,
//                                TRUE))
//     {
//         ERR("NtlmStructWriteUCS failed.\n");
//         Status = ERROR_INTERNAL_ERROR;
//         goto done;
//     }
//     /* not supported */
//     RtlInitUnicodeString(&LocalBuffer->UserParameters, NULL);
//     /* Build user flags */
//     LocalBuffer->UserFlags = 0x0;
//     if (LogonPwdData->LogonType == NetLogonLmKey)
//         LocalBuffer->UserFlags |= LOGON_USED_LM_PASSWORD;
//
//     /* copy data to client buffer */
//     Status = NtlmCopyToClientBuffer(ClientRequest, BufferLength, &Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("DispatchTable.CopyToClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *ProfileBuffer = (PMSV1_0_LM20_LOGON_PROFILE)Buffer.ClientBaseAddress;
//     *ProfileBufferLength = BufferLength;
// done:
//     /* On success Buffer.ClientBaseAddress will not be free */
//     NtlmFreeClientBuffer(ClientRequest, !NT_SUCCESS(Status), &Buffer);
//     NtlmUStrFree(&ComputerNameUCS);
//     return Status;
// }
//
//
// static
// PSID
// AppendRidToSid(PSID SrcSid,
//                ULONG Rid)
// {
//     PSID DstSid = NULL;
//     UCHAR RidCount;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     if (RidCount >= 8)
//         return NULL;
//
//     DstSid = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(RtlLengthRequiredSid(RidCount + 1));
//     if (DstSid == NULL)
//         return NULL;
//
//     RtlCopyMemory!(DstSid,
//                   SrcSid,
//                   RtlLengthRequiredSid(RidCount));
//
//     *RtlSubAuthorityCountSid(DstSid) = RidCount + 1;
//     *RtlSubAuthoritySid(DstSid, RidCount) = Rid;
//
//     return DstSid;
// }
//
//
// static
// NTSTATUS
// BuildTokenUser( PTOKEN_USER User,
//                 PSID AccountDomainSid,
//                 ULONG RelativeId)
// {
//     User->User.Sid = AppendRidToSid(AccountDomainSid,
//                                     RelativeId);
//     if (User->User.Sid == NULL)
//     {
//         ERR("Could not create the user SID\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     User->User.Attributes = 0;
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// BuildTokenPrimaryGroup( PTOKEN_PRIMARY_GROUP PrimaryGroup,
//                         PSID AccountDomainSid,
//                         ULONG RelativeId)
// {
//     PrimaryGroup->PrimaryGroup = AppendRidToSid(AccountDomainSid,
//                                                 RelativeId);
//     if (PrimaryGroup->PrimaryGroup == NULL)
//     {
//         ERR("Could not create the primary group SID\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// BuildTokenGroups( PTOKEN_GROUPS *Groups,
//                   PSID AccountDomainSid,
//                   ULONG RelativeId,
//                   BOOL SpecialAccount)
// {
//     SID_IDENTIFIER_AUTHORITY SystemAuthority = {SECURITY_NT_AUTHORITY};
//     PTOKEN_GROUPS TokenGroups;
//     DWORD GroupCount = 0;
//     DWORD MaxGroups = 2;
//     PSID Sid;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (SpecialAccount)
//         MaxGroups++;
//
//     TokenGroups = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(TOKEN_GROUPS) +
//                                                 MaxGroups * sizeof(SID_AND_ATTRIBUTES));
//     if (TokenGroups == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (SpecialAccount)
//     {
//         /* Self */
//         Sid = AppendRidToSid(AccountDomainSid, RelativeId);
//         if (Sid == NULL)
//         {
//
//         }
//
//         TokenGroups->Groups[GroupCount].Sid = Sid;
//         TokenGroups->Groups[GroupCount].Attributes =
//             SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
//         GroupCount++;
//
//         /* Member of 'Users' alias */
//         RtlAllocateAndInitializeSid(&SystemAuthority,
//                                     2,
//                                     SECURITY_BUILTIN_DOMAIN_RID,
//                                     DOMAIN_ALIAS_RID_USERS,
//                                     SECURITY_NULL_RID,
//                                     SECURITY_NULL_RID,
//                                     SECURITY_NULL_RID,
//                                     SECURITY_NULL_RID,
//                                     SECURITY_NULL_RID,
//                                     SECURITY_NULL_RID,
//                                     &Sid);
//         TokenGroups->Groups[GroupCount].Sid = Sid;
//         TokenGroups->Groups[GroupCount].Attributes =
//             SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
//         GroupCount++;
//     }
//     else
//     {
//         /* Member of the domains users group */
//         Sid = AppendRidToSid(AccountDomainSid, DOMAIN_GROUP_RID_USERS);
//         if (Sid == NULL)
//         {
//
//         }
//
//         TokenGroups->Groups[GroupCount].Sid = Sid;
//         TokenGroups->Groups[GroupCount].Attributes =
//             SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
//         GroupCount++;
//     }
//
//     /* Member of 'Authenticated users' */
//     RtlAllocateAndInitializeSid(&SystemAuthority,
//                                 1,
//                                 SECURITY_AUTHENTICATED_USER_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 SECURITY_NULL_RID,
//                                 &Sid);
//     TokenGroups->Groups[GroupCount].Sid = Sid;
//     TokenGroups->Groups[GroupCount].Attributes =
//         SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
//     GroupCount++;
//
//     TokenGroups->GroupCount = GroupCount;
//     ASSERT(TokenGroups->GroupCount <= MaxGroups);
//
//     *Groups = TokenGroups;
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// BuildTokenInformationBuffer(PLSA_TOKEN_INFORMATION_V1 *TokenInformation,
//                             PRPC_SID AccountDomainSid,
//                             PSAMPR_USER_INFO_BUFFER UserInfo,
//                             BOOL SpecialAccount)
// {
//     PLSA_TOKEN_INFORMATION_V1 Buffer = NULL;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(LSA_TOKEN_INFORMATION_V1));
//     if (Buffer == NULL)
//     {
//         WARN("Failed to allocate the local buffer!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Buffer->ExpirationTime.LowPart = UserInfo->All.AccountExpires.LowPart;
//     Buffer->ExpirationTime.HighPart = UserInfo->All.AccountExpires.HighPart;
//
//     Status = BuildTokenUser(&Buffer->User,
//                             (PSID)AccountDomainSid,
//                             UserInfo->All.UserId);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("BuildTokenUser() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = BuildTokenPrimaryGroup(&Buffer->PrimaryGroup,
//                                     (PSID)AccountDomainSid,
//                                     UserInfo->All.PrimaryGroupId);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("BuildTokenPrimaryGroup() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = BuildTokenGroups(&Buffer->Groups,
//                               (PSID)AccountDomainSid,
//                               UserInfo->All.UserId,
//                               SpecialAccount);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("BuildTokenGroups() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *TokenInformation = Buffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (Buffer != NULL)
//         {
//             if (Buffer->User.User.Sid != NULL)
//                 DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer->User.User.Sid);
//
//             if (Buffer->Groups != NULL)
//             {
//                 for (i = 0; i < Buffer->Groups->GroupCount; i++)
//                 {
//                     if (Buffer->Groups->Groups[i].Sid != NULL)
//                         DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer->Groups->Groups[i].Sid);
//                 }
//
//                 DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer->Groups);
//             }
//
//             if (Buffer->PrimaryGroup.PrimaryGroup != NULL)
//                 DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer->PrimaryGroup.PrimaryGroup);
//
//             if (Buffer->DefaultDacl.DefaultDacl != NULL)
//                 DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer->DefaultDacl.DefaultDacl);
//
//             DispatchTable.FreeLsaHeap.unwrap_unchecked()(Buffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// MsvpChangePassword( PLSA_CLIENT_REQUEST ClientRequest,
//                     PVOID ProtocolSubmitBuffer,
//                     PVOID ClientBufferBase,
//                     ULONG SubmitBufferLength,
//                     PVOID *ProtocolReturnBuffer,
//                     PULONG ReturnBufferLength,
//                     PNTSTATUS ProtocolStatus)
// {
//     NTSTATUS Status;
//     PMSV1_0_CHANGEPASSWORD_REQUEST RequestBuffer;
//     ULONG_PTR PtrOffset;
//
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     SAMPR_HANDLE UserHandle = NULL;
//     PRPC_SID DomainSid = NULL;
//     RPC_UNICODE_STRING Names[1];
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//
//     ENCRYPTED_NT_OWF_PASSWORD OldNtPassword;
//     ENCRYPTED_NT_OWF_PASSWORD NewNtPassword;
//     ENCRYPTED_LM_OWF_PASSWORD OldLmPassword;
//     ENCRYPTED_LM_OWF_PASSWORD NewLmPassword;
//     OEM_STRING LmPwdString;
//     CHAR LmPwdBuffer[15];
//     BOOLEAN OldLmPasswordPresent = FALSE;
//     BOOLEAN NewLmPasswordPresent = FALSE;
//
//     ENCRYPTED_LM_OWF_PASSWORD OldLmEncryptedWithNewLm;
//     ENCRYPTED_LM_OWF_PASSWORD NewLmEncryptedWithOldLm;
//     ENCRYPTED_LM_OWF_PASSWORD OldNtEncryptedWithNewNt;
//     ENCRYPTED_LM_OWF_PASSWORD NewNtEncryptedWithOldNt;
//     PENCRYPTED_LM_OWF_PASSWORD pOldLmEncryptedWithNewLm = NULL;
//     PENCRYPTED_LM_OWF_PASSWORD pNewLmEncryptedWithOldLm = NULL;
//
//     TRACE("MsvpChangePassword()\n");
//
//     /* Parameters validation */
//
//     if (SubmitBufferLength < sizeof(MSV1_0_CHANGEPASSWORD_REQUEST))
//     {
//         ERR("Invalid SubmitBufferLength %lu\n", SubmitBufferLength);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RequestBuffer = (PMSV1_0_CHANGEPASSWORD_REQUEST)ProtocolSubmitBuffer;
//
//     /* Fix-up pointers in the request buffer info */
//     PtrOffset = (ULONG_PTR)ProtocolSubmitBuffer - (ULONG_PTR)ClientBufferBase;
//
//     Status = RtlValidateUnicodeString(0, &RequestBuffer->DomainName);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//     // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//     RequestBuffer->DomainName.Buffer = FIXUP_POINTER!(RequestBuffer->DomainName.Buffer, PtrOffset);
//     RequestBuffer->DomainName.MaximumLength = RequestBuffer->DomainName.Length;
//
//     Status = RtlValidateUnicodeString(0, &RequestBuffer->AccountName);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//     // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//     RequestBuffer->AccountName.Buffer = FIXUP_POINTER!(RequestBuffer->AccountName.Buffer, PtrOffset);
//     RequestBuffer->AccountName.MaximumLength = RequestBuffer->AccountName.Length;
//
//     Status = RtlValidateUnicodeString(0, &RequestBuffer->OldPassword);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//     // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//     RequestBuffer->OldPassword.Buffer = FIXUP_POINTER!(RequestBuffer->OldPassword.Buffer, PtrOffset);
//     RequestBuffer->OldPassword.MaximumLength = RequestBuffer->OldPassword.Length;
//
//     Status = RtlValidateUnicodeString(0, &RequestBuffer->NewPassword);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//     // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//     RequestBuffer->NewPassword.Buffer = FIXUP_POINTER!(RequestBuffer->NewPassword.Buffer, PtrOffset);
//     RequestBuffer->NewPassword.MaximumLength = RequestBuffer->NewPassword.Length;
//
//     TRACE("Domain: %S\n", RequestBuffer->DomainName.Buffer);
//     TRACE("Account: %S\n", RequestBuffer->AccountName.Buffer);
//     TRACE("Old Password: %S\n", RequestBuffer->OldPassword.Buffer);
//     TRACE("New Password: %S\n", RequestBuffer->NewPassword.Buffer);
//
//     /* Connect to the SAM server */
//     Status = SamIConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                          TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamIConnect() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get the domain SID */
//     Status = SamrLookupDomainInSamServer(ServerHandle,
//                                          (PRPC_UNICODE_STRING)&RequestBuffer->DomainName,
//                                          &DomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrLookupDomainInSamServer failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     /* Open the domain */
//     Status = SamrOpenDomain(ServerHandle,
//                             DOMAIN_LOOKUP,
//                             DomainSid,
//                             &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Names[0].Length = RequestBuffer->AccountName.Length;
//     Names[0].MaximumLength = RequestBuffer->AccountName.MaximumLength;
//     Names[0].Buffer = RequestBuffer->AccountName.Buffer;
//
//     /* Try to get the RID for the user name */
//     Status = SamrLookupNamesInDomain(DomainHandle,
//                                      1,
//                                      Names,
//                                      &RelativeIds,
//                                      &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrLookupNamesInDomain failed (Status %08lx)\n", Status);
//         Status = STATUS_NO_SUCH_USER;
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use.Element[0] != SidTypeUser)
//     {
//         TRACE("Account is not a user account!\n");
//         Status = STATUS_NO_SUCH_USER;
//         goto done;
//     }
//
//     /* Open the user object */
//     Status = SamrOpenUser(DomainHandle,
//                           USER_CHANGE_PASSWORD,
//                           RelativeIds.Element[0],
//                           &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrOpenUser failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//
//     /* Calculate the NT hash for the old password */
//     Status = SystemFunction007(&RequestBuffer->OldPassword,
//                                (LPBYTE)&OldNtPassword);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Calculate the NT hash for the new password */
//     Status = SystemFunction007(&RequestBuffer->NewPassword,
//                                (LPBYTE)&NewNtPassword);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction007 failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Calculate the LM password and hash for the old password */
//     LmPwdString.Length = 15;
//     LmPwdString.MaximumLength = 15;
//     LmPwdString.Buffer = LmPwdBuffer;
//     ZeroMemory!(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//     Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                &RequestBuffer->OldPassword,
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
//                                                &RequestBuffer->NewPassword,
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
//     /* Encrypt the old and new LM passwords, if they exist */
//     if (OldLmPasswordPresent && NewLmPasswordPresent)
//     {
//         /* Encrypt the old LM password */
//         Status = SystemFunction012((const BYTE *)&OldLmPassword,
//                                    (const BYTE *)&NewLmPassword,
//                                    (LPBYTE)&OldLmEncryptedWithNewLm);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         /* Encrypt the new LM password */
//         Status = SystemFunction012((const BYTE *)&NewLmPassword,
//                                    (const BYTE *)&OldLmPassword,
//                                    (LPBYTE)&NewLmEncryptedWithOldLm);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         pOldLmEncryptedWithNewLm = &OldLmEncryptedWithNewLm;
//         pNewLmEncryptedWithOldLm = &NewLmEncryptedWithOldLm;
//     }
//
//     /* Encrypt the old NT password */
//     Status = SystemFunction012((const BYTE *)&OldNtPassword,
//                                (const BYTE *)&NewNtPassword,
//                                (LPBYTE)&OldNtEncryptedWithNewNt);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Encrypt the new NT password */
//     Status = SystemFunction012((const BYTE *)&NewNtPassword,
//                                (const BYTE *)&OldNtPassword,
//                                (LPBYTE)&NewNtEncryptedWithOldNt);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SystemFunction012 failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Change the password */
//     Status = SamrChangePasswordUser(UserHandle,
//                                     OldLmPasswordPresent && NewLmPasswordPresent,
//                                     pOldLmEncryptedWithNewLm,
//                                     pNewLmEncryptedWithOldLm,
//                                     TRUE,
//                                     &OldNtEncryptedWithNewNt,
//                                     &NewNtEncryptedWithOldNt,
//                                     FALSE,
//                                     NULL,
//                                     FALSE,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrChangePasswordUser failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     if (UserHandle != NULL)
//         SamrCloseHandle(&UserHandle);
//
//     SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//     SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (DomainSid != NULL)
//         SamIFreeVoid(DomainSid);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// MsvpEnumerateUsers(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      PVOID ProtocolSubmitBuffer,
//      PVOID ClientBufferBase,
//      ULONG SubmitBufferLength,
//      PVOID *ProtocolReturnBuffer,
//      PULONG ReturnBufferLength,
//      PNTSTATUS ProtocolStatus)
// {
//     PMSV1_0_ENUMUSERS_RESPONSE LocalBuffer = NULL;
//     PVOID ClientBaseAddress = NULL;
//     ULONG BufferLength;
//     PLIST_ENTRY CurrentEntry;
//     PLOGON_LIST_ENTRY LogonEntry;
//     ULONG LogonCount = 0;
//     PLUID LuidPtr;
//     PULONG EnumPtr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("MsvpEnumerateUsers()\n");
//
//     if (SubmitBufferLength < sizeof(MSV1_0_ENUMUSERS_REQUEST))
//     {
//         ERR("Invalid SubmitBufferLength %lu\n", SubmitBufferLength);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RtlAcquireResourceShared(&LogonListResource, TRUE);
//
//     /* Count the currently logged-on users */
//     CurrentEntry = LogonListHead.Flink;
//     while (CurrentEntry != &LogonListHead)
//     {
//         LogonEntry = CONTAINING_RECORD(CurrentEntry,
//                                        LOGON_LIST_ENTRY,
//                                        ListEntry);
//
//         TRACE("Logon %lu: 0x%08lx\n", LogonCount, LogonEntry->LogonId.LowPart);
//         LogonCount++;
//
//         CurrentEntry = CurrentEntry->Flink;
//     }
//
//     TRACE("LogonCount %lu\n", LogonCount);
//
//     BufferLength = sizeof(MSV1_0_ENUMUSERS_RESPONSE) +
//                    (LogonCount * sizeof(LUID)) +
//                    (LogonCount * sizeof(ULONG));
//
//     LocalBuffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(BufferLength);
//     if (LocalBuffer == NULL)
//     {
//         ERR("Failed to allocate the local buffer!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = DispatchTable.AllocateClientBuffer.unwrap_unchecked()(ClientRequest,
//                                                 BufferLength,
//                                                 &ClientBaseAddress);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("DispatchTable.AllocateClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("ClientBaseAddress: %p\n", ClientBaseAddress);
//
//     /* Fill the local buffer */
//     LocalBuffer->MessageType = MsV1_0EnumerateUsers;
//     LocalBuffer->NumberOfLoggedOnUsers = LogonCount;
//
//     LuidPtr = (PLUID)((ULONG_PTR)LocalBuffer + sizeof(MSV1_0_ENUMUSERS_RESPONSE));
//     EnumPtr = (PULONG)((ULONG_PTR)LuidPtr + LogonCount * sizeof(LUID));
//
//     LocalBuffer->LogonIds = (PLUID)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)LuidPtr - (ULONG_PTR)LocalBuffer);
//     LocalBuffer->EnumHandles = (PULONG)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)EnumPtr - (ULONG_PTR)LocalBuffer);
//
//     /* Copy the LogonIds and EnumHandles into the local buffer */
//     CurrentEntry = LogonListHead.Flink;
//     while (CurrentEntry != &LogonListHead)
//     {
//         LogonEntry = CONTAINING_RECORD(CurrentEntry,
//                                        LOGON_LIST_ENTRY,
//                                        ListEntry);
//
//         TRACE("Logon: 0x%08lx  %lu\n", LogonEntry->LogonId.LowPart, LogonEntry->EnumHandle);
//         RtlCopyMemory!(LuidPtr, &LogonEntry->LogonId, sizeof(LUID));
//         LuidPtr++;
//
//         *EnumPtr = LogonEntry->EnumHandle;
//         EnumPtr++;
//
//         CurrentEntry = CurrentEntry->Flink;
//     }
//
//     Status = DispatchTable.CopyToClientBuffer.unwrap_unchecked()(ClientRequest,
//                                               BufferLength,
//                                               ClientBaseAddress,
//                                               LocalBuffer);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("DispatchTable.CopyToClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *ProtocolReturnBuffer = ClientBaseAddress;
//     *ReturnBufferLength = BufferLength;
//     *ProtocolStatus = STATUS_SUCCESS;
//
// done:
//     RtlReleaseResource(&LogonListResource);
//
//     if (LocalBuffer != NULL)
//         DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (ClientBaseAddress != NULL)
//             DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest,
//                                            ClientBaseAddress);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// MsvpGetUserInfo(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      PVOID ProtocolSubmitBuffer,
//      PVOID ClientBufferBase,
//      ULONG SubmitBufferLength,
//      PVOID *ProtocolReturnBuffer,
//      PULONG ReturnBufferLength,
//      PNTSTATUS ProtocolStatus)
// {
//     PMSV1_0_GETUSERINFO_REQUEST RequestBuffer;
//     PLOGON_LIST_ENTRY LogonEntry;
//     PMSV1_0_GETUSERINFO_RESPONSE LocalBuffer = NULL;
//     PVOID ClientBaseAddress = NULL;
//     ULONG BufferLength;
//     PWSTR BufferPtr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("MsvpGetUserInfo()\n");
//
//     if (SubmitBufferLength < sizeof(MSV1_0_GETUSERINFO_REQUEST))
//     {
//         ERR("Invalid SubmitBufferLength %lu\n", SubmitBufferLength);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RequestBuffer = (PMSV1_0_GETUSERINFO_REQUEST)ProtocolSubmitBuffer;
//
//     TRACE("LogonId: 0x%lx\n", RequestBuffer->LogonId.LowPart);
//
//     RtlAcquireResourceShared(&LogonListResource, TRUE);
//
//     LogonEntry = GetLogonByLogonId(&RequestBuffer->LogonId);
//     if (LogonEntry == NULL)
//     {
//         ERR("No logon found for LogonId %lx\n", RequestBuffer->LogonId.LowPart);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TRACE("UserName: %wZ\n", &LogonEntry->UserName);
//     TRACE("LogonDomain: %wZ\n", &LogonEntry->LogonDomainName);
//     TRACE("LogonServer: %wZ\n", &LogonEntry->LogonServer);
//
//     BufferLength = sizeof(MSV1_0_GETUSERINFO_RESPONSE) +
//                    LogonEntry->UserName.MaximumLength +
//                    LogonEntry->LogonDomainName.MaximumLength +
//                    LogonEntry->LogonServer.MaximumLength;
//
//     LocalBuffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(BufferLength);
//     if (LocalBuffer == NULL)
//     {
//         ERR("Failed to allocate the local buffer!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = DispatchTable.AllocateClientBuffer.unwrap_unchecked()(ClientRequest,
//                                                 BufferLength,
//                                                 &ClientBaseAddress);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("DispatchTable.AllocateClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("ClientBaseAddress: %p\n", ClientBaseAddress);
//
//     /* Fill the local buffer */
//     LocalBuffer->MessageType = MsV1_0GetUserInfo;
//
//     BufferPtr = (PWSTR)((ULONG_PTR)LocalBuffer + sizeof(MSV1_0_GETUSERINFO_RESPONSE));
//
//     /* UserName */
//     LocalBuffer->UserName.Length = LogonEntry->UserName.Length;
//     LocalBuffer->UserName.MaximumLength = LogonEntry->UserName.MaximumLength;
//     LocalBuffer->UserName.Buffer = (PWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)BufferPtr - (ULONG_PTR)LocalBuffer);
//
//     RtlCopyMemory!(BufferPtr, LogonEntry->UserName.Buffer, LogonEntry->UserName.MaximumLength);
//     BufferPtr = (PWSTR)((ULONG_PTR)BufferPtr + (ULONG_PTR)LocalBuffer->UserName.MaximumLength);
//
//     /* LogonDomainName */
//     LocalBuffer->LogonDomainName.Length = LogonEntry->LogonDomainName.Length;
//     LocalBuffer->LogonDomainName.MaximumLength = LogonEntry->LogonDomainName.MaximumLength;
//     LocalBuffer->LogonDomainName.Buffer = (PWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)BufferPtr - (ULONG_PTR)LocalBuffer);
//
//     RtlCopyMemory!(BufferPtr, LogonEntry->LogonDomainName.Buffer, LogonEntry->LogonDomainName.MaximumLength);
//     BufferPtr = (PWSTR)((ULONG_PTR)BufferPtr + (ULONG_PTR)LocalBuffer->LogonDomainName.MaximumLength);
//
//     /* LogonServer */
//     LocalBuffer->LogonServer.Length = LogonEntry->LogonServer.Length;
//     LocalBuffer->LogonServer.MaximumLength = LogonEntry->LogonServer.MaximumLength;
//     LocalBuffer->LogonServer.Buffer = (PWSTR)((ULONG_PTR)ClientBaseAddress + (ULONG_PTR)BufferPtr - (ULONG_PTR)LocalBuffer);
//
//     RtlCopyMemory!(BufferPtr, LogonEntry->LogonServer.Buffer, LogonEntry->LogonServer.MaximumLength);
//
//     /* Logon Type */
//     LocalBuffer->LogonType = LogonEntry->LogonType;
//
//     Status = DispatchTable.CopyToClientBuffer.unwrap_unchecked()(ClientRequest,
//                                               BufferLength,
//                                               ClientBaseAddress,
//                                               LocalBuffer);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("DispatchTable.CopyToClientBuffer failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *ProtocolReturnBuffer = ClientBaseAddress;
//     *ReturnBufferLength = BufferLength;
//     *ProtocolStatus = STATUS_SUCCESS;
//
// done:
//     RtlReleaseResource(&LogonListResource);
//
//     if (LocalBuffer != NULL)
//         DispatchTable.FreeLsaHeap.unwrap_unchecked()(LocalBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (ClientBaseAddress != NULL)
//             DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest,
//                                            ClientBaseAddress);
//     }
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// LsaApCallPackage( PLSA_CLIENT_REQUEST ClientRequest,
//                   PVOID ProtocolSubmitBuffer,
//                   PVOID ClientBufferBase,
//                   ULONG SubmitBufferLength,
//                   PVOID *ProtocolReturnBuffer,
//                   PULONG ReturnBufferLength,
//                   PNTSTATUS ProtocolStatus)
// {
//     NTSTATUS Status;
//     MSV1_0_PROTOCOL_MESSAGE_TYPE MessageType;
//
//     TRACE("LsaApCallPackage()\n");
//
//     if (SubmitBufferLength < sizeof(MSV1_0_PROTOCOL_MESSAGE_TYPE))
//         return STATUS_INVALID_PARAMETER;
//
//     MessageType = *((PMSV1_0_PROTOCOL_MESSAGE_TYPE)ProtocolSubmitBuffer);
//
//     *ProtocolReturnBuffer = NULL;
//     *ReturnBufferLength = 0;
//
//     switch (MessageType)
//     {
//         case MsV1_0Lm20ChallengeRequest:
//         case MsV1_0Lm20GetChallengeResponse:
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         case MsV1_0EnumerateUsers:
//              Status = MsvpEnumerateUsers(ClientRequest,
//                                          ProtocolSubmitBuffer,
//                                          ClientBufferBase,
//                                          SubmitBufferLength,
//                                          ProtocolReturnBuffer,
//                                          ReturnBufferLength,
//                                          ProtocolStatus);
//              break;
//
//         case MsV1_0GetUserInfo:
//              Status = MsvpGetUserInfo(ClientRequest,
//                                       ProtocolSubmitBuffer,
//                                       ClientBufferBase,
//                                       SubmitBufferLength,
//                                       ProtocolReturnBuffer,
//                                       ReturnBufferLength,
//                                       ProtocolStatus);
//              break;
//
//         case MsV1_0ReLogonUsers:
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//
//         case MsV1_0ChangePassword:
//             Status = MsvpChangePassword(ClientRequest,
//                                         ProtocolSubmitBuffer,
//                                         ClientBufferBase,
//                                         SubmitBufferLength,
//                                         ProtocolReturnBuffer,
//                                         ReturnBufferLength,
//                                         ProtocolStatus);
//             break;
//
//         case MsV1_0ChangeCachedPassword:
//         case MsV1_0GenericPassthrough:
//         case MsV1_0CacheLogon:
//         case MsV1_0SubAuth:
//         case MsV1_0DeriveCredential:
//         case MsV1_0CacheLookup:
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         default:
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// LsaApCallPackagePassthrough( PLSA_CLIENT_REQUEST ClientRequest,
//                              PVOID ProtocolSubmitBuffer,
//                              PVOID ClientBufferBase,
//                              ULONG SubmitBufferLength,
//                              PVOID *ProtocolReturnBuffer,
//                              PULONG ReturnBufferLength,
//                              PNTSTATUS ProtocolStatus)
// {
//     TRACE("LsaApCallPackagePassthrough()\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaApCallPackageUntrusted( PLSA_CLIENT_REQUEST ClientRequest,
//                            PVOID ProtocolSubmitBuffer,
//                            PVOID ClientBufferBase,
//                            ULONG SubmitBufferLength,
//                            PVOID *ProtocolReturnBuffer,
//                            PULONG ReturnBufferLength,
//                            PNTSTATUS ProtocolStatus)
// {
//     ULONG MessageType;
//     NTSTATUS Status;
//
//     TRACE("LsaApCallPackageUntrusted()\n");
//
//     if (SubmitBufferLength < sizeof(MSV1_0_PROTOCOL_MESSAGE_TYPE))
//         return STATUS_INVALID_PARAMETER;
//
//     MessageType = (ULONG)*((PMSV1_0_PROTOCOL_MESSAGE_TYPE)ProtocolSubmitBuffer);
//
//     *ProtocolReturnBuffer = NULL;
//     *ReturnBufferLength = 0;
//
//     if (MessageType == MsV1_0ChangePassword)
//         Status = MsvpChangePassword(ClientRequest,
//                                     ProtocolSubmitBuffer,
//                                     ClientBufferBase,
//                                     SubmitBufferLength,
//                                     ProtocolReturnBuffer,
//                                     ReturnBufferLength,
//                                     ProtocolStatus);
//     else
//         Status = STATUS_ACCESS_DENIED;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaApInitializePackage( ULONG AuthenticationPackageId,
//                         PLSA_DISPATCH_TABLE LsaDispatchTable,
//                         PLSA_STRING Database ,
//                         PLSA_STRING Confidentiality ,
//                         PLSA_STRING *AuthenticationPackageName)
// {
//     PANSI_STRING NameString;
//     PCHAR NameBuffer;
//
//     TRACE("LsaApInitializePackage(%lu %p %p %p %p)\n",
//           AuthenticationPackageId, LsaDispatchTable, Database,
//           Confidentiality, AuthenticationPackageName);
//
//     if (!PackageInitialized)
//     {
//         InitializeListHead(&LogonListHead);
//         RtlInitializeResource(&LogonListResource);
//         EnumCounter = 0;
//         PackageInitialized = TRUE;
//     }
//
//     /* Get the dispatch table entries */
//     DispatchTable.CreateLogonSession = LsaDispatchTable->CreateLogonSession;
//     DispatchTable.DeleteLogonSession = LsaDispatchTable->DeleteLogonSession;
//     DispatchTable.AddCredential = LsaDispatchTable->AddCredential;
//     DispatchTable.GetCredentials = LsaDispatchTable->GetCredentials;
//     DispatchTable.DeleteCredential = LsaDispatchTable->DeleteCredential;
//     DispatchTable.AllocateLsaHeap = LsaDispatchTable->AllocateLsaHeap;
//     DispatchTable.FreeLsaHeap = LsaDispatchTable->FreeLsaHeap;
//     DispatchTable.AllocateClientBuffer = LsaDispatchTable->AllocateClientBuffer;
//     DispatchTable.FreeClientBuffer = LsaDispatchTable->FreeClientBuffer;
//     DispatchTable.CopyToClientBuffer = LsaDispatchTable->CopyToClientBuffer;
//     DispatchTable.CopyFromClientBuffer = LsaDispatchTable->CopyFromClientBuffer;
//
//     /* Return the package name */
//     NameString = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(LSA_STRING));
//     if (NameString == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     NameBuffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(MSV1_0_PACKAGE_NAME));
//     if (NameBuffer == NULL)
//     {
//         DispatchTable.FreeLsaHeap.unwrap_unchecked()(NameString);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     strcpy(NameBuffer, MSV1_0_PACKAGE_NAME);
//
//     RtlInitAnsiString(NameString, NameBuffer);
//
//     *AuthenticationPackageName = (PLSA_STRING)NameString;
//
//     return STATUS_SUCCESS;
// }
//
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// LsaApLogonTerminated(
//      PLUID LogonId)
// {
//     PLOGON_LIST_ENTRY LogonEntry;
//
//     TRACE("LsaApLogonTerminated()\n");
//
//     /* Remove the given logon entry from the list */
//     LogonEntry = GetLogonByLogonId(LogonId);
//     if (LogonEntry != NULL)
//     {
//         RtlAcquireResourceExclusive(&LogonListResource, TRUE);
//         RemoveEntryList(&LogonEntry->ListEntry);
//         RtlReleaseResource(&LogonListResource);
//
//         if (LogonEntry->UserName.Buffer)
//             RtlFreeHeap(RtlGetProcessHeap!(), 0, LogonEntry->UserName.Buffer);
//
//         if (LogonEntry->LogonDomainName.Buffer)
//             RtlFreeHeap(RtlGetProcessHeap!(), 0, LogonEntry->LogonDomainName.Buffer);
//
//         if (LogonEntry->LogonServer.Buffer)
//             RtlFreeHeap(RtlGetProcessHeap!(), 0, LogonEntry->LogonServer.Buffer);
//
//         RtlFreeHeap(RtlGetProcessHeap!(), 0, LogonEntry);
//     }
// }
//
//
// /*
//  * Handle Network logon
//  */
// static
// NTSTATUS
// LsaApLogonUserEx2_Network(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      PVOID ProtocolSubmitBuffer,
//      PVOID ClientBufferBase,
//      ULONG SubmitBufferSize,
//      PUNICODE_STRING ComputerName,
//      PUNICODE_STRING* LogonUserRef,
//      PUNICODE_STRING* LogonDomainRef,
//      PLSA_SAM_PWD_DATA LogonPwdData,
//      SAMPR_HANDLE* UserHandlePtr,
//      PSAMPR_USER_INFO_BUFFER* UserInfoPtr,
//      PRPC_SID* AccountDomainSidPtr,
//      PBOOL SpecialAccount,
//      PMSV1_0_LM20_LOGON_PROFILE *LogonProfile,
//      PULONG LogonProfileSize,
//      PNTSTATUS SubStatus)
// {
//     NTSTATUS Status;
//     PMSV1_0_LM20_LOGON LogonInfo;
//     ULONG_PTR PtrOffset;
//
//     *LogonProfile = NULL;
//     *LogonProfileSize = 0;
//     *UserInfoPtr = NULL;
//     *AccountDomainSidPtr = NULL;
//     *SpecialAccount = FALSE;
//     LogonInfo = ProtocolSubmitBuffer;
//
//     if (SubmitBufferSize < sizeof(MSV1_0_LM20_LOGON))
//     {
//         ERR("Invalid SubmitBufferSize %lu\n", SubmitBufferSize);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Fix-up pointers in the authentication info */
//     PtrOffset = (ULONG_PTR)ProtocolSubmitBuffer - (ULONG_PTR)ClientBufferBase;
//
//     if ((!NtlmFixupAndValidateUStr(&LogonInfo->LogonDomainName, PtrOffset)) ||
//         (!NtlmFixupAndValidateUStr(&LogonInfo->UserName, PtrOffset)) ||
//         (!NtlmFixupAndValidateUStr(&LogonInfo->Workstation, PtrOffset)) ||
//         (!NtlmFixupAStr(&LogonInfo->CaseSensitiveChallengeResponse, PtrOffset)) ||
//         (!NtlmFixupAStr(&LogonInfo->CaseInsensitiveChallengeResponse, PtrOffset)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     LogonPwdData->IsNetwork = TRUE;
//     LogonPwdData->LogonInfo = LogonInfo;
//     LogonPwdData->ComputerName = ComputerName;
//     Status = SamValidateUser(Network,
//                              &LogonInfo->UserName,
//                              &LogonInfo->LogonDomainName,
//                              LogonPwdData,
//                              ComputerName,
//                              SpecialAccount,
//                              AccountDomainSidPtr,
//                              UserHandlePtr,
//                              UserInfoPtr,
//                              SubStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamValidateUser failed with 0x%lx\n", Status);
//         return Status;
//     }
//
//     if (LogonInfo->ParameterControl & MSV1_0_RETURN_PROFILE_PATH)
//     {
//         Status = BuildLm20LogonProfileBuffer(ClientRequest,
//                                              *UserInfoPtr,
//                                              LogonPwdData,
//                                              LogonProfile,
//                                              LogonProfileSize);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("BuildLm20LogonProfileBuffer failed with 0x%lx\n", Status);
//             return Status;
//         }
//     }
//
//     *LogonUserRef = &LogonInfo->UserName;
//     *LogonDomainRef = &LogonInfo->LogonDomainName;
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaApLogonUserEx2( PLSA_CLIENT_REQUEST ClientRequest,
//                    SECURITY_LOGON_TYPE LogonType,
//                    PVOID ProtocolSubmitBuffer,
//                    PVOID ClientBufferBase,
//                    ULONG SubmitBufferSize,
//                    PVOID *ProfileBuffer,
//                    PULONG ProfileBufferSize,
//                    PLUID LogonId,
//                    PNTSTATUS SubStatus,
//                    PLSA_TOKEN_INFORMATION_TYPE TokenInformationType,
//                    PVOID *TokenInformation,
//                    PUNICODE_STRING *AccountName,
//                    PUNICODE_STRING *AuthenticatingAuthority,
//                    PUNICODE_STRING *MachineName,
//                    PSECPKG_PRIMARY_CRED PrimaryCredentials, /* Not supported yet */
//                    PSECPKG_SUPPLEMENTAL_CRED_ARRAY *SupplementalCredentials) /* Not supported yet */
// {
//     NTSTATUS Status;
//     UNICODE_STRING ComputerName;
//     WCHAR ComputerNameData[MAX_COMPUTERNAME_LENGTH + 1];
//     PUNICODE_STRING LogonUserName = NULL;
//     LSA_SAM_PWD_DATA LogonPwdData = { FALSE, NULL };
//     PUNICODE_STRING LogonDomain = NULL;
//     SAMPR_HANDLE UserHandle = NULL;
//     PRPC_SID AccountDomainSid = NULL;
//     PSAMPR_USER_INFO_BUFFER UserInfo = NULL;
//     BOOLEAN SessionCreated = FALSE;
//     DWORD ComputerNameSize;
//     BOOL SpecialAccount = FALSE;
//     UCHAR LogonPassHash;
//     PUNICODE_STRING ErasePassword = NULL;
//     PLOGON_LIST_ENTRY LogonEntry = NULL;
//
//     TRACE("LsaApLogonUserEx2()\n");
//
//     TRACE("LogonType: %lu\n", LogonType);
//     TRACE("ProtocolSubmitBuffer: %p\n", ProtocolSubmitBuffer);
//     TRACE("SubmitBufferSize: %lu\n", SubmitBufferSize);
//
//     *ProfileBuffer = NULL;
//     *ProfileBufferSize = 0;
//     *SubStatus = STATUS_SUCCESS;
//     *AccountName = NULL;
//     *AuthenticatingAuthority = NULL;
//
//     /* Get the computer name */
//     ComputerNameSize = ARRAYSIZE!(ComputerNameData);
//     if (!GetComputerNameW(ComputerNameData, &ComputerNameSize))
//     {
//         ERR("Failed to get Computername.\n");
//         return STATUS_INTERNAL_ERROR;
//     }
//     RtlInitUnicodeString(&ComputerName, ComputerNameData);
//
//     /* Parameters validation */
//     if (LogonType == Interactive ||
//         LogonType == Batch ||
//         LogonType == Service)
//     {
//         PMSV1_0_INTERACTIVE_LOGON LogonInfo;
//         ULONG_PTR PtrOffset;
//
//         if (SubmitBufferSize < sizeof(MSV1_0_INTERACTIVE_LOGON))
//         {
//             ERR("Invalid SubmitBufferSize %lu\n", SubmitBufferSize);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         LogonInfo = (PMSV1_0_INTERACTIVE_LOGON)ProtocolSubmitBuffer;
//
//         if (LogonInfo->MessageType != MsV1_0InteractiveLogon &&
//             LogonInfo->MessageType != MsV1_0WorkstationUnlockLogon)
//         {
//             ERR("Invalid MessageType %lu\n", LogonInfo->MessageType);
//             return STATUS_BAD_VALIDATION_CLASS;
//         }
//
//
//         /* Fix-up pointers in the authentication info */
//         PtrOffset = (ULONG_PTR)ProtocolSubmitBuffer - (ULONG_PTR)ClientBufferBase;
//
//         /* LogonDomainName is optional and can be an empty string */
//         if (LogonInfo->LogonDomainName.Length)
//         {
//             // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//             LogonInfo->LogonDomainName.Buffer = FIXUP_POINTER!(LogonInfo->LogonDomainName.Buffer, PtrOffset);
//             LogonInfo->LogonDomainName.MaximumLength = LogonInfo->LogonDomainName.Length;
//         }
//         else
//         {
//             LogonInfo->LogonDomainName.Buffer = NULL;
//             LogonInfo->LogonDomainName.MaximumLength = 0;
//         }
//         Status = RtlValidateUnicodeString(0, &LogonInfo->LogonDomainName);
//         if (!NT_SUCCESS(Status))
//             return STATUS_INVALID_PARAMETER;
//
//         /* UserName is mandatory and cannot be an empty string */
//         // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//         LogonInfo->UserName.Buffer = FIXUP_POINTER!(LogonInfo->UserName.Buffer, PtrOffset);
//         LogonInfo->UserName.MaximumLength = LogonInfo->UserName.Length;
//
//         Status = RtlValidateUnicodeString(0, &LogonInfo->UserName);
//         if (!NT_SUCCESS(Status))
//             return STATUS_INVALID_PARAMETER;
//
//         /* MS docs says max length is 0xFF bytes. But thats not the full story:
//          *
//          * A Quote from https://groups.google.com/forum/#!topic/microsoft.public.win32.programmer.kernel/eFGcCo_ZObk:
//          * "... At least on my WinXP SP2. Domain and UserName are passed
//          * in clear text, but the Password is NOT. ..."
//          *
//          * If the higher byte of length != 0 we have to use RtlRunDecodeUnicodeString.
//          */
//         LogonPassHash = (LogonInfo->Password.Length >> 8) & 0xFF;
//         LogonInfo->Password.Length = LogonInfo->Password.Length & 0xFF;
//
//         /* Password is optional and can be an empty string */
//         if (LogonInfo->Password.Length)
//         {
//             // TODO: Check for Buffer limits wrt. ClientBufferBase and alignment.
//             LogonInfo->Password.Buffer = FIXUP_POINTER!(LogonInfo->Password.Buffer, PtrOffset);
//             LogonInfo->Password.MaximumLength = LogonInfo->Password.Length;
//         }
//         else
//         {
//             LogonInfo->Password.Buffer = NULL;
//             LogonInfo->Password.MaximumLength = 0;
//         }
//
//         /* Decode password */
//         if (LogonPassHash > 0)
//         {
//             RtlRunDecodeUnicodeString(LogonPassHash, &LogonInfo->Password);
//         }
//
//         /* ErasePassword will be "erased" before we return */
//         ErasePassword = &LogonInfo->Password;
//
//         Status = RtlValidateUnicodeString(0, &LogonInfo->Password);
//         if (!NT_SUCCESS(Status))
//             return STATUS_INVALID_PARAMETER;
//
//         LogonUserName = &LogonInfo->UserName;
//         LogonDomain = &LogonInfo->LogonDomainName;
//         LogonPwdData.IsNetwork = FALSE;
//         LogonPwdData.PlainPwd = &LogonInfo->Password;
//         LogonPwdData.ComputerName = &ComputerName;
//
//         TRACE("Domain: %wZ\n", &LogonInfo->LogonDomainName);
//         TRACE("User: %wZ\n", &LogonInfo->UserName);
//         TRACE("Password: %wZ\n", &LogonInfo->Password);
//
//         // TODO: If LogonType == Service, do some extra work using LogonInfo->Password.
//     }
//     else if (LogonType == Network)
//     {
//         Status = LsaApLogonUserEx2_Network(ClientRequest,
//                                            ProtocolSubmitBuffer,
//                                            ClientBufferBase,
//                                            SubmitBufferSize,
//                                            &ComputerName,
//                                            &LogonUserName,
//                                            &LogonDomain,
//                                            &LogonPwdData,
//                                            &UserHandle,
//                                            &UserInfo,
//                                            &AccountDomainSid,
//                                            &SpecialAccount,
//                                            (PMSV1_0_LM20_LOGON_PROFILE*)ProfileBuffer,
//                                            ProfileBufferSize,
//                                            SubStatus);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//     else
//     {
//         FIXME("LogonType %lu is not supported yet!\n", LogonType);
//         return STATUS_NOT_IMPLEMENTED;
//     }
//     // TODO: Add other LogonType validity checks.
//
//     Status = SamValidateUser(LogonType,
//                              LogonUserName,
//                              LogonDomain,
//                              &LogonPwdData,
//                              &ComputerName,
//                              &SpecialAccount,
//                              &AccountDomainSid,
//                              &UserHandle,
//                              &UserInfo,
//                              SubStatus);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Return logon information */
//
//     /* Create and return a new logon id */
//     Status = NtAllocateLocallyUniqueId(LogonId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtAllocateLocallyUniqueId failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     /* Create the logon session */
//     Status = DispatchTable.CreateLogonSession.unwrap_unchecked()(LogonId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("CreateLogonSession failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     SessionCreated = TRUE;
//
//     LogonEntry = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, sizeof(LOGON_LIST_ENTRY));
//     if (LogonEntry)
//     {
//         RtlCopyMemory!(&LogonEntry->LogonId, LogonId, sizeof(LUID));
//         LogonEntry->EnumHandle = EnumCounter;
//         EnumCounter++;
//
//         TRACE("Logon User: %wZ %wZ %lx\n", LogonUserName, LogonDomain, LogonId->LowPart);
//         LogonEntry->UserName.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, LogonUserName->MaximumLength);
//         if (LogonEntry->UserName.Buffer)
//         {
//             LogonEntry->UserName.MaximumLength = LogonUserName->MaximumLength;
//             RtlCopyUnicodeString(&LogonEntry->UserName, LogonUserName);
//         }
//
//         LogonEntry->LogonDomainName.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, LogonDomain->MaximumLength);
//         if (LogonEntry->LogonDomainName.Buffer)
//         {
//             LogonEntry->LogonDomainName.MaximumLength = LogonDomain->MaximumLength;
//             RtlCopyUnicodeString(&LogonEntry->LogonDomainName, LogonDomain);
//         }
//
//         LogonEntry->LogonServer.Buffer = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, ComputerName.MaximumLength);
//         if (LogonEntry->LogonServer.Buffer)
//         {
//             LogonEntry->LogonServer.MaximumLength = ComputerName.MaximumLength;
//             RtlCopyUnicodeString(&LogonEntry->LogonServer, &ComputerName);
//         }
//
//         LogonEntry->LogonType = LogonType;
//
//         RtlAcquireResourceExclusive(&LogonListResource, TRUE);
//         InsertTailList(&LogonListHead, &LogonEntry->ListEntry);
//         RtlReleaseResource(&LogonListResource);
//     }
//
//     if (LogonType == Interactive || LogonType == Batch || LogonType == Service)
//     {
//         /* Build and fill the interactive profile buffer */
//         Status = BuildInteractiveProfileBuffer(ClientRequest,
//                                                UserInfo,
//                                                ComputerName.Buffer,
//                                                (PMSV1_0_INTERACTIVE_PROFILE*)ProfileBuffer,
//                                                ProfileBufferSize);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("BuildInteractiveProfileBuffer failed (Status %08lx)\n", Status);
//             goto done;
//         }
//     }
//     else if (LogonType == Network)
//     {
//         //FIXME: no need to do anything, its already done ...
//     }
//
//     /* Return the token information type */
//     *TokenInformationType = LsaTokenInformationV1;
//
//     /* Build and fill the token information buffer */
//     Status = BuildTokenInformationBuffer((PLSA_TOKEN_INFORMATION_V1*)TokenInformation,
//                                          AccountDomainSid,
//                                          UserInfo,
//                                          SpecialAccount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("BuildTokenInformationBuffer failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     /* Erase password */
//     if (ErasePassword)
//     {
//         RtlEraseUnicodeString(ErasePassword);
//     }
//
//     /* Update the logon time/count or the bad password time/count */
//     if ((UserHandle != NULL) &&
//         (Status == STATUS_SUCCESS || Status == STATUS_WRONG_PASSWORD))
//     {
//         SAMPR_USER_INFO_BUFFER InternalInfo;
//
//         RtlZeroMemory!(&InternalInfo, sizeof(InternalInfo));
//
//         if (Status == STATUS_SUCCESS)
//             InternalInfo.Internal2.Flags = USER_LOGON_SUCCESS;
//         else
//             InternalInfo.Internal2.Flags = USER_LOGON_BAD_PASSWORD;
//
//         SamrSetInformationUser(UserHandle,
//                                UserInternal2Information,
//                                &InternalInfo);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the account name */
//         *AccountName = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(UNICODE_STRING));
//         if ((LogonUserName != NULL) &&
//             (*AccountName != NULL))
//         {
//             (*AccountName)->Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(LogonUserName->Length +
//                                                                    sizeof(UNICODE_NULL));
//             if ((*AccountName)->Buffer != NULL)
//             {
//                 (*AccountName)->MaximumLength = LogonUserName->Length +
//                                                 sizeof(UNICODE_NULL);
//                 RtlCopyUnicodeString(*AccountName, LogonUserName);
//             }
//         }
//
//         /* Return the authenticating authority */
//         *AuthenticatingAuthority = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(UNICODE_STRING));
//         if ((LogonDomain != NULL) &&
//             (*AuthenticatingAuthority != NULL))
//         {
//             (*AuthenticatingAuthority)->Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(LogonDomain->Length +
//                                                                                sizeof(UNICODE_NULL));
//             if ((*AuthenticatingAuthority)->Buffer != NULL)
//             {
//                 (*AuthenticatingAuthority)->MaximumLength = LogonDomain->Length +
//                                                             sizeof(UNICODE_NULL);
//                 RtlCopyUnicodeString(*AuthenticatingAuthority, LogonDomain);
//             }
//         }
//
//         /* Return the machine name */
//         *MachineName = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(sizeof(UNICODE_STRING));
//         if (*MachineName != NULL)
//         {
//             (*MachineName)->Buffer = DispatchTable.AllocateLsaHeap.unwrap_unchecked()(ComputerName.MaximumLength);
//             if ((*MachineName)->Buffer != NULL)
//             {
//                 (*MachineName)->MaximumLength = ComputerName.MaximumLength;
//                 (*MachineName)->Length = ComputerName.Length;
//                 RtlCopyMemory!((*MachineName)->Buffer,
//                               ComputerName.Buffer,
//                               ComputerName.MaximumLength);
//             }
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (SessionCreated != FALSE)
//             DispatchTable.DeleteLogonSession.unwrap_unchecked()(LogonId);
//
//         if (*ProfileBuffer != NULL)
//         {
//             DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest,
//                                            *ProfileBuffer);
//             *ProfileBuffer = NULL;
//         }
//     }
//
//     if (UserHandle != NULL)
//         SamrCloseHandle(&UserHandle);
//
//     SamIFree_SAMPR_USER_INFO_BUFFER(UserInfo,
//                                     UserAllInformation);
//
//     if (AccountDomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap!(), 0, AccountDomainSid);
//
//     if (Status == STATUS_NO_SUCH_USER ||
//         Status == STATUS_WRONG_PASSWORD)
//     {
//         *SubStatus = Status;
//         Status = STATUS_LOGON_FAILURE;
//     }
//
//     TRACE("LsaApLogonUserEx2 done (Status 0x%08lx, SubStatus 0x%08lx)\n", Status, *SubStatus);
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// SpLsaModeInitialize(
//      ULONG LsaVersion,
//      PULONG PackageVersion,
//      PSECPKG_FUNCTION_TABLE *ppTables,
//      PULONG pcTables)
// {
//     TRACE("SpLsaModeInitialize(0x%lx %p %p %p)\n",
//           LsaVersion, PackageVersion, ppTables, pcTables);
//
//     if (LsaVersion != SECPKG_INTERFACE_VERSION)
//         return STATUS_INVALID_PARAMETER;
//
//     *PackageVersion = SECPKG_INTERFACE_VERSION;
//
//     *ppTables = NtlmLsaFn.as_mut_ptr();
//     *pcTables = 1;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// SpUserModeInitialize(
//      ULONG LsaVersion,
//      PULONG PackageVersion,
//      PSECPKG_USER_FUNCTION_TABLE *ppTables,
//      PULONG pcTables)
// {
//     TRACE("SpUserModeInitialize(0x%lx %p %p %p)\n",
//           LsaVersion, PackageVersion, ppTables, pcTables);
//
//     if (LsaVersion != SECPKG_INTERFACE_VERSION)
//         return STATUS_INVALID_PARAMETER;
//
//     *PackageVersion = SECPKG_INTERFACE_VERSION;
//
//     *ppTables = NtlmUsrFn;
//     *pcTables = 1;
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
