use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::wcslen;
// The crate's own functions: the definition is the declaration.
use crate::samsrv::registry::SampRegOpenKey;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    fn SampCreateAliasSD(Sd: *mut PSECURITY_DESCRIPTOR, SdSize: *mut ULONG) -> NTSTATUS;
    fn SampRegCloseKey(KeyHandle: *mut HANDLE) -> NTSTATUS;
    fn SampRegCreateKey( hParentKey: HANDLE, lpKeyName: LPCWSTR, samDesired: ULONG, phKeyHandle: *mut HANDLE, ) -> NTSTATUS;
    fn SampRegDeleteKey(hParentKey: HANDLE, lpKeyName: LPCWSTR) -> NTSTATUS;
    fn SampRegSetValue( hKey: HANDLE, lpValueName: LPCWSTR, dwType: ULONG, lpData: LPVOID, cbData: ULONG, ) -> NTSTATUS;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn SampCreateGroupSD(Sd: *mut PSECURITY_DESCRIPTOR, SdSize: *mut ULONG) -> NTSTATUS;
    fn SampCreateServerSD(Sd: *mut PSECURITY_DESCRIPTOR, SdSize: *mut ULONG) -> NTSTATUS;
}
// WINAPI (stdcall on i386) — was mis-emitted as extern "C"; corrected so the
// i386 decoration/stack-cleanup matches advapi32/kernel32 imports.
extern "system" {
    fn ConvertSidToStringSidW(Sid: PSID, StringSid: *mut LPWSTR) -> BOOL;
    fn LocalFree(hMem: LPVOID) -> LPVOID;
    fn RegCloseKey(hKey: HKEY) -> LONG;
    fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: DWORD, lpSecurityAttributes: LPVOID, phkResult: *mut HKEY, lpdwDisposition: *mut DWORD, ) -> LONG;
    #[link_name = "RegSetValueExW"]
    fn RegSetValueEx( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: LPVOID, cbData: DWORD, ) -> LONG;
}

// ---- file-scope preamble (translated) ----
pub const TICKS_PER_SECOND: i64 = 10000000;

pub static mut SecurityNtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};

unsafe fn SampSetupAddMemberToAlias(
    hDomainKey: HKEY,
    AliasId: ULONG,
    MemberSid: PSID,
) -> BOOL {

    let mut dwDisposition: DWORD = 0;
    let mut MemberSidString: LPWSTR = core::ptr::null_mut();
    let mut szKeyName: [u16; 256] = [0; 256];
    let mut hMembersKey: HKEY = core::ptr::null_mut();

    ConvertSidToStringSidW(MemberSid, &mut MemberSidString);

    let fmt1: [u16; 19] = [
        b'A' as u16, b'l' as u16, b'i' as u16, b'a' as u16, b's' as u16, b'e' as u16, b's' as u16,
        b'\\' as u16, b'%' as u16, b'0' as u16, b'8' as u16, b'l' as u16, b'X' as u16,
        b'\\' as u16, b'M' as u16, b'e' as u16, b'm' as u16, b'b' as u16, 0,
    ];
    // "Aliases\\%08lX\\Members"
    let fmt_members: [u16; 22] = [
        b'A' as u16, b'l' as u16, b'i' as u16, b'a' as u16, b's' as u16, b'e' as u16, b's' as u16,
        b'\\' as u16, b'%' as u16, b'0' as u16, b'8' as u16, b'l' as u16, b'X' as u16,
        b'\\' as u16, b'M' as u16, b'e' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16,
        b's' as u16, 0,
    ];
    let _ = fmt1;

    swprintf(szKeyName.as_mut_ptr(), fmt_members.as_ptr(), AliasId);

    if RegCreateKeyExW(
        hDomainKey,
        szKeyName.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hMembersKey,
        &mut dwDisposition,
    ) == 0
    {
        RegSetValueEx(
            hMembersKey,
            MemberSidString,
            0,
            REG_BINARY,
            MemberSid as LPVOID,
            RtlLengthSid(MemberSid),
        );

        RegCloseKey(hMembersKey);
    }

    // "Aliases\\Members\\%s"
    let fmt_amembers: [u16; 19] = [
        b'A' as u16, b'l' as u16, b'i' as u16, b'a' as u16, b's' as u16, b'e' as u16, b's' as u16,
        b'\\' as u16, b'M' as u16, b'e' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16,
        b's' as u16, b'\\' as u16, b'%' as u16, b's' as u16, 0,
    ];

    swprintf(szKeyName.as_mut_ptr(), fmt_amembers.as_ptr(), MemberSidString);

    if RegCreateKeyExW(
        hDomainKey,
        szKeyName.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hMembersKey,
        &mut dwDisposition,
    ) == 0
    {
        // "%08lX"
        let fmt_id: [u16; 6] = [
            b'%' as u16, b'0' as u16, b'8' as u16, b'l' as u16, b'X' as u16, 0,
        ];

        swprintf(szKeyName.as_mut_ptr(), fmt_id.as_ptr(), AliasId);

        RegSetValueEx(
            hMembersKey,
            szKeyName.as_ptr(),
            0,
            REG_BINARY,
            MemberSid as LPVOID,
            RtlLengthSid(MemberSid),
        );

        RegCloseKey(hMembersKey);
    }

    if !MemberSidString.is_null() {
        LocalFree(MemberSidString as LPVOID);
    }

    1
}

unsafe fn SampSetupCreateAliasAccount(
    hDomainKey: HANDLE,
    lpAccountName: LPCWSTR,
    lpDescription: LPCWSTR,
    ulRelativeId: ULONG,
) -> NTSTATUS {

    let mut szAccountKeyName: [u16; 32] = [0; 32];
    let mut hAccountKey: HANDLE = core::ptr::null_mut();
    let mut hNamesKey: HANDLE = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    let fmt: &[u16] = &[
        0x41, 0x6C, 0x69, 0x61, 0x73, 0x65, 0x73, 0x5C, 0x25, 0x30, 0x38, 0x6C, 0x58, 0x00,
    ];
    swprintf(szAccountKeyName.as_mut_ptr(), fmt.as_ptr(), ulRelativeId);

    let name_w: &[u16] = &[0x4E, 0x61, 0x6D, 0x65, 0x00];
    let desc_w: &[u16] = &[
        0x44, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x00,
    ];
    let secdesc_w: &[u16] = &[0x53, 0x65, 0x63, 0x44, 0x65, 0x73, 0x63, 0x00];
    let aliases_names_w: &[u16] = &[
        0x41, 0x6C, 0x69, 0x61, 0x73, 0x65, 0x73, 0x5C, 0x4E, 0x61, 0x6D, 0x65, 0x73, 0x00,
    ];

    Status = SampRegCreateKey(
        hDomainKey,
        szAccountKeyName.as_ptr(),
        KEY_ALL_ACCESS,
        &mut hAccountKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = SampRegSetValue(
            hAccountKey,
            name_w.as_ptr(),
            REG_SZ,
            lpAccountName as LPVOID,
            ((wcslen(lpAccountName) + 1) * core::mem::size_of::<u16>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            desc_w.as_ptr(),
            REG_SZ,
            lpDescription as LPVOID,
            ((wcslen(lpDescription) + 1) * core::mem::size_of::<u16>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateAliasSD(&mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(hAccountKey, secdesc_w.as_ptr(), REG_BINARY, Sd, SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            hDomainKey,
            aliases_names_w.as_ptr(),
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hNamesKey,
            lpAccountName,
            REG_DWORD,
            &ulRelativeId as *const ULONG as LPVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    SampRegCloseKey(&mut hNamesKey);

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    if !hAccountKey.is_null() {
        SampRegCloseKey(&mut hAccountKey);

        if !NT_SUCCESS(Status) {
            SampRegDeleteKey(hDomainKey, szAccountKeyName.as_ptr());
        }
    }

    Status
}

unsafe fn SampSetupAddMemberToGroup(
    hDomainKey: HANDLE,
    GroupId: ULONG,
    MemberId: ULONG,
) -> NTSTATUS {
    let mut szKeyName: [WCHAR; 256] = [0; 256];
    let mut hGroupKey: HANDLE = core::ptr::null_mut();
    let mut MembersBuffer: PULONG = core::ptr::null_mut();
    let mut MembersCount: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    swprintf(
        szKeyName.as_mut_ptr(),
        w!("Groups\\%08lX").as_ptr() as *const u16,
        GroupId,
    );

    Status = SampRegOpenKey(
        hDomainKey,
        szKeyName.as_mut_ptr(),
        KEY_ALL_ACCESS,
        &mut hGroupKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        Status = SampRegQueryValue(
            hGroupKey,
            w!("Members").as_ptr() as *const u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        MembersBuffer = midl_user_allocate(
            (Length as usize + core::mem::size_of::<ULONG>()) as _,
        ) as PULONG;
        if MembersBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        if Status != STATUS_OBJECT_NAME_NOT_FOUND {
            Status = SampRegQueryValue(
                hGroupKey,
                w!("Members").as_ptr() as *const u16,
                core::ptr::null_mut(),
                MembersBuffer as _,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            MembersCount = Length / core::mem::size_of::<ULONG>() as ULONG;
        }

        i = 0;
        while i < MembersCount {
            if *MembersBuffer.offset(i as isize) == MemberId {
                Status = STATUS_MEMBER_IN_GROUP;
                break 'done;
            }
            i += 1;
        }

        *MembersBuffer.offset(MembersCount as isize) = MemberId;
        Length += core::mem::size_of::<ULONG>() as ULONG;

        Status = SampRegSetValue(
            hGroupKey,
            w!("Members").as_ptr() as *const u16,
            REG_BINARY,
            MembersBuffer as _,
            Length,
        );

        break 'done;
    }

    if !MembersBuffer.is_null() {
        midl_user_free(MembersBuffer as _);
    }

    SampRegCloseKey(&mut hGroupKey);

    Status
}

unsafe fn SampSetupCreateGroupAccount(
    hDomainKey: HANDLE,
    lpAccountName: LPCWSTR,
    lpComment: LPCWSTR,
    ulRelativeId: ULONG,
) -> NTSTATUS {

    let mut FixedGroupData: SAM_GROUP_FIXED_DATA = core::mem::zeroed();
    let mut szAccountKeyName: [u16; 32] = [0; 32];
    let mut hAccountKey: HANDLE = core::ptr::null_mut();
    let mut hNamesKey: HANDLE = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    FixedGroupData.Version = 1;
    FixedGroupData.Reserved = 0;
    FixedGroupData.GroupId = ulRelativeId;
    FixedGroupData.Attributes = 0;

    let fmt: &[u16] = &[
        b'G' as u16, b'r' as u16, b'o' as u16, b'u' as u16, b'p' as u16, b's' as u16,
        b'\\' as u16, b'%' as u16, b'0' as u16, b'8' as u16, b'l' as u16, b'X' as u16, 0,
    ];
    swprintf(szAccountKeyName.as_mut_ptr(), fmt.as_ptr(), ulRelativeId);

    let name_f: &[u16] = &[b'F' as u16, 0];
    let name_name: &[u16] = &[b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];
    let name_admincomment: &[u16] = &[
        b'A' as u16, b'd' as u16, b'm' as u16, b'i' as u16, b'n' as u16, b'C' as u16,
        b'o' as u16, b'm' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0,
    ];
    let name_secdesc: &[u16] = &[
        b'S' as u16, b'e' as u16, b'c' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, 0,
    ];
    let name_groups_names: &[u16] = &[
        b'G' as u16, b'r' as u16, b'o' as u16, b'u' as u16, b'p' as u16, b's' as u16,
        b'\\' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b's' as u16, 0,
    ];

    Status = SampRegCreateKey(
        hDomainKey,
        szAccountKeyName.as_ptr(),
        KEY_ALL_ACCESS,
        &mut hAccountKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        Status = SampRegSetValue(
            hAccountKey,
            name_f.as_ptr(),
            REG_BINARY,
            &mut FixedGroupData as *mut _ as LPVOID,
            core::mem::size_of::<SAM_GROUP_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            name_name.as_ptr(),
            REG_SZ,
            lpAccountName as LPVOID,
            ((wcslen(lpAccountName) + 1) * core::mem::size_of::<u16>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            name_admincomment.as_ptr(),
            REG_SZ,
            lpComment as LPVOID,
            ((wcslen(lpComment) + 1) * core::mem::size_of::<u16>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateGroupSD(&mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            name_secdesc.as_ptr(),
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            hDomainKey,
            name_groups_names.as_ptr(),
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hNamesKey,
            lpAccountName,
            REG_DWORD,
            &mut (ulRelativeId as ULONG) as *mut _ as LPVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );

        break 'done;
    }

    SampRegCloseKey(&mut hNamesKey);

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    if !hAccountKey.is_null() {
        SampRegCloseKey(&mut hAccountKey);

        if !NT_SUCCESS(Status) {
            SampRegDeleteKey(hDomainKey, szAccountKeyName.as_ptr());
        }
    }

    Status
}

unsafe fn SampSetupCreateUserAccount(
    hDomainKey: HANDLE,
    lpAccountName: LPCWSTR,
    lpComment: LPCWSTR,
    lpDomainSid: PSID,
    ulRelativeId: ULONG,
    UserAccountControl: ULONG,
) -> NTSTATUS {
    let mut FixedUserData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut GroupMembership: GROUP_MEMBERSHIP = core::mem::zeroed();
    let mut LogonHours: [UCHAR; 23] = [0; 23];
    let lpEmptyString: LPWSTR = wide!("").as_ptr() as LPWSTR;
    let mut szAccountKeyName: [WCHAR; 32] = [0; 32];
    let mut hAccountKey: HANDLE = core::ptr::null_mut();
    let mut hNamesKey: HANDLE = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let UserSid: PSID;
    let mut Status: NTSTATUS;

    UserSid = AppendRidToSid(lpDomainSid, ulRelativeId);

    Status = SampCreateUserSD(UserSid, &mut Sd, &mut SdSize);
    if !NT_SUCCESS(Status) {
        SampRegCloseKey(&mut hNamesKey);
        if !Sd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
        }
        if !UserSid.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
        }
        if !hAccountKey.is_null() {
            SampRegCloseKey(&mut hAccountKey);
            if !NT_SUCCESS(Status) {
                SampRegDeleteKey(hDomainKey, szAccountKeyName.as_mut_ptr());
            }
        }
        return Status;
    }

    FixedUserData.Version = 1;
    FixedUserData.Reserved = 0;
    FixedUserData.LastLogon.QuadPart = 0;
    FixedUserData.LastLogoff.QuadPart = 0;
    FixedUserData.PasswordLastSet.QuadPart = 0;
    FixedUserData.AccountExpires.QuadPart = MAXLONGLONG as _;
    FixedUserData.LastBadPasswordTime.QuadPart = 0;
    FixedUserData.UserId = ulRelativeId;
    FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
    FixedUserData.UserAccountControl = UserAccountControl;
    FixedUserData.CountryCode = 0;
    FixedUserData.CodePage = 0;
    FixedUserData.BadPasswordCount = 0;
    FixedUserData.LogonCount = 0;
    FixedUserData.AdminCount = 0;
    FixedUserData.OperatorCount = 0;

    swprintf(
        szAccountKeyName.as_mut_ptr(),
        wide!("Users\\%08lX").as_ptr(),
        ulRelativeId,
    );

    Status = SampRegCreateKey(
        hDomainKey,
        szAccountKeyName.as_mut_ptr(),
        KEY_ALL_ACCESS,
        &mut hAccountKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        Status = SampRegSetValue(
            hAccountKey,
            wide!("F").as_ptr(),
            REG_BINARY,
            &mut FixedUserData as *mut _ as LPVOID,
            core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("Name").as_ptr(),
            REG_SZ,
            lpAccountName as LPVOID,
            ((wcslen(lpAccountName) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("FullName").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("HomeDirectory").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("HomeDirectoryDrive").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("ScriptPath").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("ProfilePath").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("AdminComment").as_ptr(),
            REG_SZ,
            lpComment as LPVOID,
            ((wcslen(lpComment) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("UserComment").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("WorkStations").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("Parameters").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (LogonHours.as_mut_ptr() as PUSHORT).write_unaligned(168);
        memset(
            &mut LogonHours[2] as *mut _ as LPVOID,
            0xff,
            21,
        );

        Status = SampRegSetValue(
            hAccountKey,
            wide!("LogonHours").as_ptr(),
            REG_BINARY,
            LogonHours.as_mut_ptr() as LPVOID,
            core::mem::size_of::<[UCHAR; 23]>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
        GroupMembership.Attributes = SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT;

        Status = SampRegSetValue(
            hAccountKey,
            wide!("Groups").as_ptr(),
            REG_BINARY,
            &mut GroupMembership as *mut _ as LPVOID,
            core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("LMPwd").as_ptr(),
            REG_BINARY,
            core::ptr::addr_of_mut!(EmptyLmHash) as LPVOID,
            core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("NTPwd").as_ptr(),
            REG_BINARY,
            core::ptr::addr_of_mut!(EmptyNtHash) as LPVOID,
            core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("LMPwdHistory").as_ptr(),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("NTPwdHistory").as_ptr(),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("PrivateData").as_ptr(),
            REG_SZ,
            lpEmptyString as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hAccountKey,
            wide!("SecDesc").as_ptr(),
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            hDomainKey,
            wide!("Users\\Names").as_ptr(),
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hNamesKey,
            lpAccountName,
            REG_DWORD,
            &ulRelativeId as *const _ as LPVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );

        break 'done;
    }

    SampRegCloseKey(&mut hNamesKey);

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    if !UserSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
    }

    if !hAccountKey.is_null() {
        SampRegCloseKey(&mut hAccountKey);

        if !NT_SUCCESS(Status) {
            SampRegDeleteKey(hDomainKey, szAccountKeyName.as_mut_ptr());
        }
    }

    Status
}

unsafe fn SampSetupCreateDomain(
    hServerKey: HANDLE,
    lpKeyName: LPCWSTR,
    lpDomainName: LPCWSTR,
    lpDomainSid: PSID,
    bBuiltinDomain: BOOLEAN,
    lpDomainKey: *mut HANDLE,
) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut szDomainKeyName: [WCHAR; 32] = [0; 32];
    let mut lpEmptyString: [WCHAR; 1] = [0];
    let mut hDomainKey: HANDLE = core::ptr::null_mut();
    let mut hAliasesKey: HANDLE = core::ptr::null_mut();
    let mut hGroupsKey: HANDLE = core::ptr::null_mut();
    let mut hUsersKey: HANDLE = core::ptr::null_mut();
    let mut hNamesKey: HANDLE = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    if !lpDomainKey.is_null() {
        *lpDomainKey = core::ptr::null_mut();
    }

    memset(
        core::ptr::addr_of_mut!(FixedData) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>(),
    );
    FixedData.Version = 1;
    NtQuerySystemTime(core::ptr::addr_of_mut!(FixedData.CreationTime));
    FixedData.DomainModifiedCount.QuadPart = 0;
    FixedData.MaxPasswordAge.QuadPart = -(6i64 * 7 * 24 * 60 * 60 * TICKS_PER_SECOND);
    FixedData.MinPasswordAge.QuadPart = 0;
    FixedData.ForceLogoff.QuadPart = MAXLONGLONG as i64;
    FixedData.LockoutDuration.QuadPart = -(30i64 * 60 * TICKS_PER_SECOND);
    FixedData.LockoutObservationWindow.QuadPart = -(30i64 * 60 * TICKS_PER_SECOND);
    FixedData.ModifiedCountAtLastPromotion.QuadPart = 0;
    FixedData.NextRid = 1000;
    FixedData.PasswordProperties = 0;
    FixedData.MinPasswordLength = 0;
    FixedData.PasswordHistoryLength = 0;
    FixedData.LockoutThreshold = 0;
    FixedData.DomainServerState = DomainServerEnabled;
    FixedData.DomainServerRole = DomainServerRolePrimary;
    FixedData.UasCompatibilityRequired = 1u8;

    wcscpy(
        szDomainKeyName.as_mut_ptr(),
        w!("Domains\\").as_ptr() as LPCWSTR,
    );
    wcscat(szDomainKeyName.as_mut_ptr(), lpKeyName);

    'done: {
        Status = SampRegCreateKey(
            hServerKey,
            szDomainKeyName.as_ptr(),
            KEY_ALL_ACCESS,
            &mut hDomainKey,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = SampRegSetValue(
            hDomainKey,
            w!("F").as_ptr() as LPCWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as LPVOID,
            core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if !lpDomainSid.is_null() {
            Status = SampRegSetValue(
                hDomainKey,
                w!("Name").as_ptr() as LPCWSTR,
                REG_SZ,
                lpDomainName as LPVOID,
                ((wcslen(lpDomainName) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SampRegSetValue(
                hDomainKey,
                w!("SID").as_ptr() as LPCWSTR,
                REG_BINARY,
                lpDomainSid as LPVOID,
                RtlLengthSid(lpDomainSid),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = SampRegSetValue(
            hDomainKey,
            w!("OemInformation").as_ptr() as LPCWSTR,
            REG_SZ,
            lpEmptyString.as_mut_ptr() as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hDomainKey,
            w!("ReplicaSourceNodeName").as_ptr()
                as LPCWSTR,
            REG_SZ,
            lpEmptyString.as_mut_ptr() as LPVOID,
            core::mem::size_of::<WCHAR>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegCreateKey(
            hDomainKey,
            w!("Aliases").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hAliasesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegCreateKey(
            hAliasesKey,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SampRegCloseKey(&mut hNamesKey);

        Status = SampRegCreateKey(
            hDomainKey,
            w!("Groups").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hGroupsKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegCreateKey(
            hGroupsKey,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SampRegCloseKey(&mut hNamesKey);

        Status = SampRegCreateKey(
            hDomainKey,
            w!("Users").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hUsersKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegCreateKey(
            hUsersKey,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_ALL_ACCESS,
            &mut hNamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if bBuiltinDomain != 0 {
            Status = SampCreateBuiltinDomainSD(&mut Sd, &mut SdSize);
        } else {
            Status = SampCreateAccountDomainSD(&mut Sd, &mut SdSize);
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            hServerKey,
            w!("SecDesc").as_ptr() as LPCWSTR,
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SampRegCloseKey(&mut hNamesKey);

        if !lpDomainKey.is_null() {
            *lpDomainKey = hDomainKey;
        }
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    SampRegCloseKey(&mut hAliasesKey);
    SampRegCloseKey(&mut hGroupsKey);
    SampRegCloseKey(&mut hUsersKey);

    if !NT_SUCCESS(Status) {
        SampRegCloseKey(&mut hDomainKey);
    }

    Status
}

unsafe fn SampSetupCreateServer(hSamKey: HANDLE, lpServerKey: *mut HANDLE) -> NTSTATUS {
    let mut hServerKey: HANDLE = core::ptr::null_mut();
    let mut hDomainsKey: HANDLE = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = SampRegCreateKey(hSamKey,
                              wide!("SAM").as_ptr(),
                              KEY_ALL_ACCESS,
                              &mut hServerKey);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = SampRegCreateKey(hServerKey,
                                  wide!("Domains").as_ptr(),
                                  KEY_ALL_ACCESS,
                                  &mut hDomainsKey);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateServerSD(&mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(hServerKey,
                                 wide!("SecDesc").as_ptr(),
                                 REG_BINARY,
                                 Sd,
                                 SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SampRegCloseKey(&mut hDomainsKey);

        *lpServerKey = hServerKey;
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampGetAccountDomainInfo(AccountDomainInfo: *mut PPOLICY_ACCOUNT_DOMAIN_INFO) -> NTSTATUS {
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"SampGetAccountDomainInfo\n\0".as_ptr());

    memset(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ObjectAttributes),
        POLICY_VIEW_LOCAL_INFORMATION,
        core::ptr::addr_of_mut!(PolicyHandle),
    );
    if Status != STATUS_SUCCESS {
        ERR!(
            "LsaOpenPolicy failed (Status: 0x%08lx)\n",
            Status,
        );
        return Status;
    }

    Status = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyAccountDomainInformation,
        AccountDomainInfo as *mut PVOID,
    );

    LsaClose(PolicyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampInitializeSAM() -> BOOL {
    let mut AccountDomainInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let mut hSamKey: HANDLE = core::ptr::null_mut();
    let mut hServerKey: HANDLE = core::ptr::null_mut();
    let mut hBuiltinDomainKey: HANDLE = core::ptr::null_mut();
    let mut hAccountDomainKey: HANDLE = core::ptr::null_mut();
    let mut pBuiltinSid: PSID = core::ptr::null_mut();
    let mut pInteractiveSid: PSID = core::ptr::null_mut();
    let mut pAuthenticatedUserSid: PSID = core::ptr::null_mut();
    let mut bResult: BOOL = 1;
    let mut pSid: PSID;
    let hInstance: HINSTANCE;
    let mut szComment: [u16; 256] = [0; 256];
    let mut szName: [u16; 80] = [0; 80];
    let mut Status: NTSTATUS;

    TRACE!(b"SampInitializeSAM() called\n\0".as_ptr());

    hInstance = GetModuleHandleW(
        ['s' as u16, 'a' as u16, 'm' as u16, 's' as u16, 'r' as u16, 'v' as u16, '.' as u16,
         'd' as u16, 'l' as u16, 'l' as u16, 0].as_ptr(),
    );

    'done: {
        Status = SampRegOpenKey(
            core::ptr::null_mut(),
            ['\\' as u16, 'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16,
             'r' as u16, 'y' as u16, '\\' as u16, 'M' as u16, 'a' as u16, 'c' as u16, 'h' as u16,
             'i' as u16, 'n' as u16, 'e' as u16, '\\' as u16, 'S' as u16, 'A' as u16, 'M' as u16, 0]
                .as_ptr(),
            (KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS) as ULONG,
            &mut hSamKey,
        );
        if !NT_SUCCESS(Status) {
            ERR!("Failed to open the SAM key (Status: 0x%08lx)\n", Status);
            return 0;
        }

        Status = SampSetupCreateServer(hSamKey, &mut hServerKey);
        if !NT_SUCCESS(Status) {
            bResult = 0;
            break 'done;
        }

        pBuiltinSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1) as SIZE_T);
        if pBuiltinSid.is_null() {
            ERR!("Failed to allocate the Builtin Domain SID\n");
            bResult = 0;
            break 'done;
        }

        RtlInitializeSid(pBuiltinSid, &raw mut SecurityNtAuthority, 1);
        *RtlSubAuthoritySid(pBuiltinSid, 0) = SECURITY_BUILTIN_DOMAIN_RID as ULONG;

        pInteractiveSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1) as SIZE_T);
        if pInteractiveSid.is_null() {
            ERR!("Failed to allocate the Interactive SID\n");
            bResult = 0;
            break 'done;
        }

        RtlInitializeSid(pInteractiveSid, &raw mut SecurityNtAuthority, 1);
        *RtlSubAuthoritySid(pInteractiveSid, 0) = SECURITY_INTERACTIVE_RID as ULONG;

        pAuthenticatedUserSid =
            RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1) as SIZE_T);
        if pAuthenticatedUserSid.is_null() {
            ERR!("Failed to allocate the Authenticated User SID\n");
            bResult = 0;
            break 'done;
        }

        RtlInitializeSid(pAuthenticatedUserSid, &raw mut SecurityNtAuthority, 1);
        *RtlSubAuthoritySid(pAuthenticatedUserSid, 0) = SECURITY_AUTHENTICATED_USER_RID as ULONG;

        Status = SampGetAccountDomainInfo(&mut AccountDomainInfo);
        if !NT_SUCCESS(Status) {
            ERR!("SampGetAccountDomainInfo failed (Status %08lx)\n", Status);
            bResult = 0;
            break 'done;
        }

        SampLoadString(hInstance, IDS_DOMAIN_BUILTIN_NAME, szName.as_mut_ptr(), 80);

        Status = SampSetupCreateDomain(
            hServerKey,
            ['B' as u16, 'u' as u16, 'i' as u16, 'l' as u16, 't' as u16, 'i' as u16, 'n' as u16, 0]
                .as_ptr(),
            szName.as_ptr(),
            pBuiltinSid,
            1u8,
            &mut hBuiltinDomainKey,
        );
        if !NT_SUCCESS(Status) {
            bResult = 0;
            break 'done;
        }

        SampLoadString(hInstance, IDS_ALIAS_ADMINISTRATORS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_ADMINISTRATORS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_USERS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_USERS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_USERS as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_GUESTS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_GUESTS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_GUESTS as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_POWER_USERS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_POWER_USERS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_POWER_USERS as ULONG,
        );

        if SampProductType != NtProductWinNt {
            SampLoadString(hInstance, IDS_ALIAS_PRINT_OPS_NAME, szName.as_mut_ptr(), 80);
            SampLoadString(hInstance, IDS_ALIAS_PRINT_OPS_COMMENT, szComment.as_mut_ptr(), 256);
            SampSetupCreateAliasAccount(
                hBuiltinDomainKey,
                szName.as_ptr(),
                szComment.as_ptr(),
                DOMAIN_ALIAS_RID_PRINT_OPS as ULONG,
            );
        }

        SampLoadString(hInstance, IDS_ALIAS_BACKUP_OPS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_BACKUP_OPS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_BACKUP_OPS as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_REPLICATOR_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_REPLICATOR_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_REPLICATOR as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_REMOTE_DESKTOP_USERS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_REMOTE_DESKTOP_USERS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS as ULONG,
        );

        SampLoadString(hInstance, IDS_ALIAS_NETWORK_CONFIGURATION_OPS_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_ALIAS_NETWORK_CONFIGURATION_OPS_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateAliasAccount(
            hBuiltinDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS as ULONG,
        );

        if SampProductType != NtProductWinNt {
            SampLoadString(hInstance, IDS_ALIAS_MONITORING_USERS_NAME, szName.as_mut_ptr(), 80);
            SampLoadString(hInstance, IDS_ALIAS_MONITORING_USERS_COMMENT, szComment.as_mut_ptr(), 256);
            SampSetupCreateAliasAccount(
                hBuiltinDomainKey,
                szName.as_ptr(),
                szComment.as_ptr(),
                DOMAIN_ALIAS_RID_MONITORING_USERS as ULONG,
            );

            SampLoadString(hInstance, IDS_ALIAS_LOGGING_USERS_NAME, szName.as_mut_ptr(), 80);
            SampLoadString(hInstance, IDS_ALIAS_LOGGING_USERS_COMMENT, szComment.as_mut_ptr(), 256);
            SampSetupCreateAliasAccount(
                hBuiltinDomainKey,
                szName.as_ptr(),
                szComment.as_ptr(),
                DOMAIN_ALIAS_RID_LOGGING_USERS as ULONG,
            );
        }

        pSid = AppendRidToSid((*AccountDomainInfo).DomainSid, DOMAIN_USER_RID_ADMIN as ULONG);
        if !pSid.is_null() {
            SampSetupAddMemberToAlias(hBuiltinDomainKey, DOMAIN_ALIAS_RID_ADMINS as ULONG, pSid);
            RtlFreeHeap(RtlGetProcessHeap(), 0, pSid);
        }

        pSid = AppendRidToSid((*AccountDomainInfo).DomainSid, DOMAIN_USER_RID_GUEST as ULONG);
        if !pSid.is_null() {
            SampSetupAddMemberToAlias(hBuiltinDomainKey, DOMAIN_ALIAS_RID_GUESTS as ULONG, pSid);
            RtlFreeHeap(RtlGetProcessHeap(), 0, pSid);
        }

        SampSetupAddMemberToAlias(hBuiltinDomainKey, DOMAIN_ALIAS_RID_USERS as ULONG, pInteractiveSid);
        SampSetupAddMemberToAlias(hBuiltinDomainKey, DOMAIN_ALIAS_RID_USERS as ULONG, pAuthenticatedUserSid);

        Status = SampSetupCreateDomain(
            hServerKey,
            ['A' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 0]
                .as_ptr(),
            [0u16].as_ptr(),
            (*AccountDomainInfo).DomainSid,
            0u8,
            &mut hAccountDomainKey,
        );
        if !NT_SUCCESS(Status) {
            bResult = 0;
            break 'done;
        }

        SampLoadString(hInstance, IDS_GROUP_NONE_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_GROUP_NONE_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateGroupAccount(
            hAccountDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            DOMAIN_GROUP_RID_USERS as ULONG,
        );

        SampLoadString(hInstance, IDS_USER_ADMINISTRATOR_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_USER_ADMINISTRATOR_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateUserAccount(
            hAccountDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            (*AccountDomainInfo).DomainSid,
            DOMAIN_USER_RID_ADMIN as ULONG,
            (USER_DONT_EXPIRE_PASSWORD | USER_NORMAL_ACCOUNT) as ULONG,
        );

        SampSetupAddMemberToGroup(
            hAccountDomainKey,
            DOMAIN_GROUP_RID_USERS as ULONG,
            DOMAIN_USER_RID_ADMIN as ULONG,
        );

        SampLoadString(hInstance, IDS_USER_GUEST_NAME, szName.as_mut_ptr(), 80);
        SampLoadString(hInstance, IDS_USER_GUEST_COMMENT, szComment.as_mut_ptr(), 256);
        SampSetupCreateUserAccount(
            hAccountDomainKey,
            szName.as_ptr(),
            szComment.as_ptr(),
            (*AccountDomainInfo).DomainSid,
            DOMAIN_USER_RID_GUEST as ULONG,
            (USER_ACCOUNT_DISABLED | USER_DONT_EXPIRE_PASSWORD | USER_NORMAL_ACCOUNT) as ULONG,
        );

        SampSetupAddMemberToGroup(
            hAccountDomainKey,
            DOMAIN_GROUP_RID_USERS as ULONG,
            DOMAIN_USER_RID_GUEST as ULONG,
        );
    }

    if !AccountDomainInfo.is_null() {
        LsaFreeMemory(AccountDomainInfo as LPVOID);
    }

    if !pAuthenticatedUserSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pAuthenticatedUserSid);
    }

    if !pInteractiveSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pInteractiveSid);
    }

    if !pBuiltinSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pBuiltinSid);
    }

    SampRegCloseKey(&mut hAccountDomainKey);
    SampRegCloseKey(&mut hBuiltinDomainKey);
    SampRegCloseKey(&mut hServerKey);
    SampRegCloseKey(&mut hSamKey);

    TRACE!(b"SampInitializeSAM() done\n\0".as_ptr());

    bResult
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/setup.c
//  * PURPOSE:         Registry setup routines
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// #include <ntsecapi.h>
//
// #include "resources.h"
//
// /* GLOBALS *****************************************************************/
//
// #define TICKS_PER_SECOND 10000000LL
//
// SID_IDENTIFIER_AUTHORITY SecurityNtAuthority = {SECURITY_NT_AUTHORITY};
//
// /* FUNCTIONS ***************************************************************/
//
// static BOOL
// SampSetupAddMemberToAlias(HKEY hDomainKey,
//                           ULONG AliasId,
//                           PSID MemberSid)
// {
//     DWORD dwDisposition;
//     LPWSTR MemberSidString = NULL;
//     WCHAR szKeyName[256];
//     HKEY hMembersKey;
//
//     ConvertSidToStringSidW(MemberSid, &MemberSidString);
//
//     swprintf(szKeyName, L"Aliases\\%08lX\\Members", AliasId);
//
//     if (!RegCreateKeyExW(hDomainKey,
//                          szKeyName,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          KEY_ALL_ACCESS,
//                          NULL,
//                          &hMembersKey,
//                          &dwDisposition))
//     {
//         RegSetValueEx(hMembersKey,
//                       MemberSidString,
//                       0,
//                       REG_BINARY,
//                       (LPVOID)MemberSid,
//                       RtlLengthSid(MemberSid));
//
//         RegCloseKey(hMembersKey);
//     }
//
//     swprintf(szKeyName, L"Aliases\\Members\\%s", MemberSidString);
//
//     if (!RegCreateKeyExW(hDomainKey,
//                          szKeyName,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          KEY_ALL_ACCESS,
//                          NULL,
//                          &hMembersKey,
//                          &dwDisposition))
//     {
//         swprintf(szKeyName, L"%08lX", AliasId);
//
//         RegSetValueEx(hMembersKey,
//                       szKeyName,
//                       0,
//                       REG_BINARY,
//                       (LPVOID)MemberSid,
//                       RtlLengthSid(MemberSid));
//
//         RegCloseKey(hMembersKey);
//     }
//
//     if (MemberSidString != NULL)
//         LocalFree(MemberSidString);
//
//     return TRUE;
// }
//
//
// static
// NTSTATUS
// SampSetupCreateAliasAccount(HANDLE hDomainKey,
//                             LPCWSTR lpAccountName,
//                             LPCWSTR lpDescription,
//                             ULONG ulRelativeId)
// {
//     WCHAR szAccountKeyName[32];
//     HANDLE hAccountKey = NULL;
//     HANDLE hNamesKey = NULL;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     NTSTATUS Status;
//
//     swprintf(szAccountKeyName, L"Aliases\\%08lX", ulRelativeId);
//
//     Status = SampRegCreateKey(hDomainKey,
//                               szAccountKeyName,
//                               KEY_ALL_ACCESS,
//                               &hAccountKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Name",
//                              REG_SZ,
//                              (LPVOID)lpAccountName,
//                              (wcslen(lpAccountName) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Description",
//                              REG_SZ,
//                              (LPVOID)lpDescription,
//                              (wcslen(lpDescription) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the server SD */
//     Status = SampCreateAliasSD(&Sd,
//                                &SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set SecDesc attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"SecDesc",
//                              REG_BINARY,
//                              Sd,
//                              SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(hDomainKey,
//                             L"Aliases\\Names",
//                             KEY_ALL_ACCESS,
//                             &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hNamesKey,
//                             lpAccountName,
//                             REG_DWORD,
//                             (LPVOID)&ulRelativeId,
//                             sizeof(ULONG));
//
// done:
//     SampRegCloseKey(&hNamesKey);
//
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     if (hAccountKey != NULL)
//     {
//         SampRegCloseKey(&hAccountKey);
//
//         if (!NT_SUCCESS(Status))
//             SampRegDeleteKey(hDomainKey,
//                              szAccountKeyName);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampSetupAddMemberToGroup(IN HANDLE hDomainKey,
//                           IN ULONG GroupId,
//                           IN ULONG MemberId)
// {
//     WCHAR szKeyName[256];
//     HANDLE hGroupKey = NULL;
//     PULONG MembersBuffer = NULL;
//     ULONG MembersCount = 0;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     swprintf(szKeyName, L"Groups\\%08lX", GroupId);
//
//     Status = SampRegOpenKey(hDomainKey,
//                             szKeyName,
//                             KEY_ALL_ACCESS,
//                             &hGroupKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegQueryValue(hGroupKey,
//                                L"Members",
//                                NULL,
//                                NULL,
//                                &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//         goto done;
//
//     MembersBuffer = midl_user_allocate(Length + sizeof(ULONG));
//     if (MembersBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         Status = SampRegQueryValue(hGroupKey,
//                                    L"Members",
//                                    NULL,
//                                    MembersBuffer,
//                                    &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         MembersCount = Length / sizeof(ULONG);
//     }
//
//     for (i = 0; i < MembersCount; i++)
//     {
//         if (MembersBuffer[i] == MemberId)
//         {
//             Status = STATUS_MEMBER_IN_GROUP;
//             goto done;
//         }
//     }
//
//     MembersBuffer[MembersCount] = MemberId;
//     Length += sizeof(ULONG);
//
//     Status = SampRegSetValue(hGroupKey,
//                              L"Members",
//                              REG_BINARY,
//                              MembersBuffer,
//                              Length);
//
// done:
//     if (MembersBuffer != NULL)
//         midl_user_free(MembersBuffer);
//
//     SampRegCloseKey(&hGroupKey);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampSetupCreateGroupAccount(HANDLE hDomainKey,
//                             LPCWSTR lpAccountName,
//                             LPCWSTR lpComment,
//                             ULONG ulRelativeId)
// {
//     SAM_GROUP_FIXED_DATA FixedGroupData;
//     WCHAR szAccountKeyName[32];
//     HANDLE hAccountKey = NULL;
//     HANDLE hNamesKey = NULL;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     NTSTATUS Status;
//
//     /* Initialize fixed group data */
//     FixedGroupData.Version = 1;
//     FixedGroupData.Reserved = 0;
//     FixedGroupData.GroupId = ulRelativeId;
//     FixedGroupData.Attributes = 0;
//
//     swprintf(szAccountKeyName, L"Groups\\%08lX", ulRelativeId);
//
//     Status = SampRegCreateKey(hDomainKey,
//                               szAccountKeyName,
//                               KEY_ALL_ACCESS,
//                               &hAccountKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"F",
//                              REG_BINARY,
//                              (LPVOID)&FixedGroupData,
//                              sizeof(SAM_GROUP_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Name",
//                              REG_SZ,
//                              (LPVOID)lpAccountName,
//                              (wcslen(lpAccountName) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"AdminComment",
//                              REG_SZ,
//                              (LPVOID)lpComment,
//                              (wcslen(lpComment) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the security descriptor */
//     Status = SampCreateGroupSD(&Sd,
//                                &SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SecDesc attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"SecDesc",
//                              REG_BINARY,
//                              Sd,
//                              SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(hDomainKey,
//                             L"Groups\\Names",
//                             KEY_ALL_ACCESS,
//                             &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hNamesKey,
//                             lpAccountName,
//                             REG_DWORD,
//                             (LPVOID)&ulRelativeId,
//                             sizeof(ULONG));
//
// done:
//     SampRegCloseKey(&hNamesKey);
//
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     if (hAccountKey != NULL)
//     {
//         SampRegCloseKey(&hAccountKey);
//
//         if (!NT_SUCCESS(Status))
//             SampRegDeleteKey(hDomainKey,
//                              szAccountKeyName);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampSetupCreateUserAccount(HANDLE hDomainKey,
//                            LPCWSTR lpAccountName,
//                            LPCWSTR lpComment,
//                            PSID lpDomainSid,
//                            ULONG ulRelativeId,
//                            ULONG UserAccountControl)
// {
//     SAM_USER_FIXED_DATA FixedUserData;
//     GROUP_MEMBERSHIP GroupMembership;
//     UCHAR LogonHours[23];
//     LPWSTR lpEmptyString = L"";
//     WCHAR szAccountKeyName[32];
//     HANDLE hAccountKey = NULL;
//     HANDLE hNamesKey = NULL;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     PSID UserSid = NULL;
//     NTSTATUS Status;
//
//     UserSid = AppendRidToSid(lpDomainSid,
//                              ulRelativeId);
//
//     /* Create the security descriptor */
//     Status = SampCreateUserSD(UserSid,
//                               &Sd,
//                               &SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Initialize fixed user data */
//     FixedUserData.Version = 1;
//     FixedUserData.Reserved = 0;
//     FixedUserData.LastLogon.QuadPart = 0;
//     FixedUserData.LastLogoff.QuadPart = 0;
//     FixedUserData.PasswordLastSet.QuadPart = 0;
//     FixedUserData.AccountExpires.QuadPart = MAXLONGLONG;
//     FixedUserData.LastBadPasswordTime.QuadPart = 0;
//     FixedUserData.UserId = ulRelativeId;
//     FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
//     FixedUserData.UserAccountControl = UserAccountControl;
//     FixedUserData.CountryCode = 0;
//     FixedUserData.CodePage = 0;
//     FixedUserData.BadPasswordCount = 0;
//     FixedUserData.LogonCount = 0;
//     FixedUserData.AdminCount = 0;
//     FixedUserData.OperatorCount = 0;
//
//     swprintf(szAccountKeyName, L"Users\\%08lX", ulRelativeId);
//
//     Status = SampRegCreateKey(hDomainKey,
//                               szAccountKeyName,
//                               KEY_ALL_ACCESS,
//                               &hAccountKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"F",
//                              REG_BINARY,
//                              (LPVOID)&FixedUserData,
//                              sizeof(SAM_USER_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Name",
//                              REG_SZ,
//                              (LPVOID)lpAccountName,
//                              (wcslen(lpAccountName) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"FullName",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"HomeDirectory",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"HomeDirectoryDrive",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"ScriptPath",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"ProfilePath",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"AdminComment",
//                              REG_SZ,
//                              (LPVOID)lpComment,
//                              (wcslen(lpComment) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"UserComment",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"WorkStations",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Parameters",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set LogonHours attribute*/
//     *((PUSHORT)LogonHours) = 168;
//     memset(&(LogonHours[2]), 0xff, 21);
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"LogonHours",
//                              REG_BINARY,
//                              (LPVOID)LogonHours,
//                              sizeof(LogonHours));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set Groups attribute*/
//     GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
//     GroupMembership.Attributes = SE_GROUP_MANDATORY |
//                                  SE_GROUP_ENABLED |
//                                  SE_GROUP_ENABLED_BY_DEFAULT;
//
//     Status = SampRegSetValue(hAccountKey,
//                              L"Groups",
//                              REG_BINARY,
//                              (LPVOID)&GroupMembership,
//                              sizeof(GROUP_MEMBERSHIP));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set LMPwd attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"LMPwd",
//                              REG_BINARY,
//                              (LPVOID)&EmptyLmHash,
//                              sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set NTPwd attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"NTPwd",
//                              REG_BINARY,
//                              (LPVOID)&EmptyNtHash,
//                              sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set LMPwdHistory attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"LMPwdHistory",
//                              REG_BINARY,
//                              NULL,
//                              0);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set NTPwdHistory attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"NTPwdHistory",
//                              REG_BINARY,
//                              NULL,
//                              0);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set PrivateData attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"PrivateData",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the SecDesc attribute*/
//     Status = SampRegSetValue(hAccountKey,
//                              L"SecDesc",
//                              REG_BINARY,
//                              Sd,
//                              SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(hDomainKey,
//                             L"Users\\Names",
//                             KEY_ALL_ACCESS,
//                             &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hNamesKey,
//                             lpAccountName,
//                             REG_DWORD,
//                             (LPVOID)&ulRelativeId,
//                             sizeof(ULONG));
//
// done:
//     SampRegCloseKey(&hNamesKey);
//
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     if (UserSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
//
//     if (hAccountKey != NULL)
//     {
//         SampRegCloseKey(&hAccountKey);
//
//         if (!NT_SUCCESS(Status))
//             SampRegDeleteKey(hDomainKey,
//                              szAccountKeyName);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampSetupCreateDomain(IN HANDLE hServerKey,
//                       IN LPCWSTR lpKeyName,
//                       IN LPCWSTR lpDomainName,
//                       IN PSID lpDomainSid,
//                       IN BOOLEAN bBuiltinDomain,
//                       OUT HANDLE *lpDomainKey)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     WCHAR szDomainKeyName[32];
//     LPWSTR lpEmptyString = L"";
//     HANDLE hDomainKey = NULL;
//     HANDLE hAliasesKey = NULL;
//     HANDLE hGroupsKey = NULL;
//     HANDLE hUsersKey = NULL;
//     HANDLE hNamesKey = NULL;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     NTSTATUS Status;
//
//     if (lpDomainKey != NULL)
//         *lpDomainKey = NULL;
//
//     /* Initialize the fixed domain data */
//     memset(&FixedData, 0, sizeof(SAM_DOMAIN_FIXED_DATA));
//     FixedData.Version = 1;
//     NtQuerySystemTime(&FixedData.CreationTime);
//     FixedData.DomainModifiedCount.QuadPart = 0;
//     FixedData.MaxPasswordAge.QuadPart = -(6LL * 7LL * 24LL * 60LL * 60LL * TICKS_PER_SECOND); /* 6 weeks */
//     FixedData.MinPasswordAge.QuadPart = 0;                                                    /* right now */
//     FixedData.ForceLogoff.QuadPart = LLONG_MAX;                                               /* very far in the future aka never */
//     FixedData.LockoutDuration.QuadPart = -(30LL * 60LL * TICKS_PER_SECOND);                   /* 30 minutes */
//     FixedData.LockoutObservationWindow.QuadPart = -(30LL * 60LL * TICKS_PER_SECOND);          /* 30 minutes */
//     FixedData.ModifiedCountAtLastPromotion.QuadPart = 0;
//     FixedData.NextRid = 1000;
//     FixedData.PasswordProperties = 0;
//     FixedData.MinPasswordLength = 0;
//     FixedData.PasswordHistoryLength = 0;
//     FixedData.LockoutThreshold = 0;
//     FixedData.DomainServerState = DomainServerEnabled;
//     FixedData.DomainServerRole = DomainServerRolePrimary;
//     FixedData.UasCompatibilityRequired = TRUE;
//
//     wcscpy(szDomainKeyName, L"Domains\\");
//     wcscat(szDomainKeyName, lpKeyName);
//
//     Status = SampRegCreateKey(hServerKey,
//                               szDomainKeyName,
//                               KEY_ALL_ACCESS,
//                               &hDomainKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Set the fixed data value */
//     Status = SampRegSetValue(hDomainKey,
//                              L"F",
//                              REG_BINARY,
//                              (LPVOID)&FixedData,
//                              sizeof(SAM_DOMAIN_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (lpDomainSid != NULL)
//     {
//         Status = SampRegSetValue(hDomainKey,
//                                  L"Name",
//                                  REG_SZ,
//                                  (LPVOID)lpDomainName,
//                                  (wcslen(lpDomainName) + 1) * sizeof(WCHAR));
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         Status = SampRegSetValue(hDomainKey,
//                                  L"SID",
//                                  REG_BINARY,
//                                  (LPVOID)lpDomainSid,
//                                  RtlLengthSid(lpDomainSid));
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     Status = SampRegSetValue(hDomainKey,
//                              L"OemInformation",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegSetValue(hDomainKey,
//                              L"ReplicaSourceNodeName",
//                              REG_SZ,
//                              (LPVOID)lpEmptyString,
//                              sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Alias container */
//     Status = SampRegCreateKey(hDomainKey,
//                               L"Aliases",
//                               KEY_ALL_ACCESS,
//                               &hAliasesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegCreateKey(hAliasesKey,
//                               L"Names",
//                               KEY_ALL_ACCESS,
//                               &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     SampRegCloseKey(&hNamesKey);
//
//     /* Create the Groups container */
//     Status = SampRegCreateKey(hDomainKey,
//                               L"Groups",
//                               KEY_ALL_ACCESS,
//                               &hGroupsKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegCreateKey(hGroupsKey,
//                               L"Names",
//                               KEY_ALL_ACCESS,
//                               &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     SampRegCloseKey(&hNamesKey);
//
//     /* Create the Users container */
//     Status = SampRegCreateKey(hDomainKey,
//                               L"Users",
//                               KEY_ALL_ACCESS,
//                               &hUsersKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegCreateKey(hUsersKey,
//                               L"Names",
//                               KEY_ALL_ACCESS,
//                               &hNamesKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the server SD */
//     if (bBuiltinDomain != FALSE)
//         Status = SampCreateBuiltinDomainSD(&Sd,
//                                            &SdSize);
//     else
//         Status = SampCreateAccountDomainSD(&Sd,
//                                            &SdSize);
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set SecDesc attribute*/
//     Status = SampRegSetValue(hServerKey,
//                              L"SecDesc",
//                              REG_BINARY,
//                              Sd,
//                              SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     SampRegCloseKey(&hNamesKey);
//
//     if (lpDomainKey != NULL)
//         *lpDomainKey = hDomainKey;
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     SampRegCloseKey(&hAliasesKey);
//     SampRegCloseKey(&hGroupsKey);
//     SampRegCloseKey(&hUsersKey);
//
//     if (!NT_SUCCESS(Status))
//         SampRegCloseKey(&hDomainKey);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampSetupCreateServer(IN HANDLE hSamKey,
//                       OUT HANDLE *lpServerKey)
// {
//     HANDLE hServerKey = NULL;
//     HANDLE hDomainsKey = NULL;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     NTSTATUS Status;
//
//     Status = SampRegCreateKey(hSamKey,
//                               L"SAM",
//                               KEY_ALL_ACCESS,
//                               &hServerKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegCreateKey(hServerKey,
//                               L"Domains",
//                               KEY_ALL_ACCESS,
//                               &hDomainsKey);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the server SD */
//     Status = SampCreateServerSD(&Sd,
//                                 &SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set SecDesc attribute*/
//     Status = SampRegSetValue(hServerKey,
//                              L"SecDesc",
//                              REG_BINARY,
//                              Sd,
//                              SdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     SampRegCloseKey(&hDomainsKey);
//
//     *lpServerKey = hServerKey;
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampGetAccountDomainInfo(PPOLICY_ACCOUNT_DOMAIN_INFO *AccountDomainInfo)
// {
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle;
//     NTSTATUS Status;
//
//     TRACE("SampGetAccountDomainInfo\n");
//
//     memset(&ObjectAttributes, 0, sizeof(LSA_OBJECT_ATTRIBUTES));
//     ObjectAttributes.Length = sizeof(LSA_OBJECT_ATTRIBUTES);
//
//     Status = LsaOpenPolicy(NULL,
//                            &ObjectAttributes,
//                            POLICY_VIEW_LOCAL_INFORMATION,
//                            &PolicyHandle);
//     if (Status != STATUS_SUCCESS)
//     {
//         ERR("LsaOpenPolicy failed (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsaQueryInformationPolicy(PolicyHandle,
//                                        PolicyAccountDomainInformation,
//                                        (PVOID *)AccountDomainInfo);
//
//     LsaClose(PolicyHandle);
//
//     return Status;
// }
//
//
// BOOL
// SampInitializeSAM(VOID)
// {
//     PPOLICY_ACCOUNT_DOMAIN_INFO AccountDomainInfo = NULL;
//     HANDLE hSamKey = NULL;
//     HANDLE hServerKey = NULL;
//     HANDLE hBuiltinDomainKey = NULL;
//     HANDLE hAccountDomainKey = NULL;
//     PSID pBuiltinSid = NULL;
//     PSID pInteractiveSid = NULL;
//     PSID pAuthenticatedUserSid = NULL;
//     BOOL bResult = TRUE;
//     PSID pSid;
//     HINSTANCE hInstance;
//     WCHAR szComment[256];
//     WCHAR szName[80];
//     NTSTATUS Status;
//
//     TRACE("SampInitializeSAM() called\n");
//
//     hInstance = GetModuleHandleW(L"samsrv.dll");
//
//     /* Open the SAM key */
//     Status = SampRegOpenKey(NULL,
//                             L"\\Registry\\Machine\\SAM",
//                             KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS,
//                             &hSamKey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to open the SAM key (Status: 0x%08lx)\n", Status);
//         return FALSE;
//     }
//
//     /* Create the SAM Server object */
//     Status = SampSetupCreateServer(hSamKey,
//                                    &hServerKey);
//     if (!NT_SUCCESS(Status))
//     {
//         bResult = FALSE;
//         goto done;
//     }
//
//     /* Create and initialize the Builtin Domain SID */
//     pBuiltinSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1));
//     if (pBuiltinSid == NULL)
//     {
//         ERR("Failed to allocate the Builtin Domain SID\n");
//         bResult = FALSE;
//         goto done;
//     }
//
//     RtlInitializeSid(pBuiltinSid, &SecurityNtAuthority, 1);
//     *(RtlSubAuthoritySid(pBuiltinSid, 0)) = SECURITY_BUILTIN_DOMAIN_RID;
//
//     /* Create and initialize the Interactive SID */
//     pInteractiveSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1));
//     if (pInteractiveSid == NULL)
//     {
//         ERR("Failed to allocate the Interactive SID\n");
//         bResult = FALSE;
//         goto done;
//     }
//
//     RtlInitializeSid(pInteractiveSid, &SecurityNtAuthority, 1);
//     *(RtlSubAuthoritySid(pInteractiveSid, 0)) = SECURITY_INTERACTIVE_RID;
//
//     /* Create and initialize the Authenticated User SID */
//     pAuthenticatedUserSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, RtlLengthRequiredSid(1));
//     if (pAuthenticatedUserSid == NULL)
//     {
//         ERR("Failed to allocate the Authenticated User SID\n");
//         bResult = FALSE;
//         goto done;
//     }
//
//     RtlInitializeSid(pAuthenticatedUserSid, &SecurityNtAuthority, 1);
//     *(RtlSubAuthoritySid(pAuthenticatedUserSid, 0)) = SECURITY_AUTHENTICATED_USER_RID;
//
//     /* Get account domain information */
//     Status = SampGetAccountDomainInfo(&AccountDomainInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampGetAccountDomainInfo failed (Status %08lx)\n", Status);
//         bResult = FALSE;
//         goto done;
//     }
//
//     SampLoadString(hInstance, IDS_DOMAIN_BUILTIN_NAME, szName, 80);
//
//     /* Create the Builtin domain */
//     Status = SampSetupCreateDomain(hServerKey,
//                                    L"Builtin",
//                                    szName,
//                                    pBuiltinSid,
//                                    TRUE,
//                                    &hBuiltinDomainKey);
//     if (!NT_SUCCESS(Status))
//     {
//         bResult = FALSE;
//         goto done;
//     }
//
//     /* Create the Administrators alias */
//     SampLoadString(hInstance, IDS_ALIAS_ADMINISTRATORS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_ADMINISTRATORS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_ADMINS);
//
//     /* Create the Users alias */
//     SampLoadString(hInstance, IDS_ALIAS_USERS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_USERS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_USERS);
//
//     /* Create the Guests alias */
//     SampLoadString(hInstance, IDS_ALIAS_GUESTS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_GUESTS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_GUESTS);
//
//     /* Create the Power Users alias */
//     SampLoadString(hInstance, IDS_ALIAS_POWER_USERS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_POWER_USERS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_POWER_USERS);
//
//     if (SampProductType != NtProductWinNt)
//     {
//         /* Create the Print Operators alias (Server only) */
//         SampLoadString(hInstance, IDS_ALIAS_PRINT_OPS_NAME, szName, 80);
//         SampLoadString(hInstance, IDS_ALIAS_PRINT_OPS_COMMENT, szComment, 256);
//
//         SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                     szName,
//                                     szComment,
//                                     DOMAIN_ALIAS_RID_PRINT_OPS);
//     }
//
//     /* Create the Backup Operators alias */
//     SampLoadString(hInstance, IDS_ALIAS_BACKUP_OPS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_BACKUP_OPS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_BACKUP_OPS);
//
//     /* Create the Replicator alias */
//     SampLoadString(hInstance, IDS_ALIAS_REPLICATOR_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_REPLICATOR_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_REPLICATOR);
//
//     /* Create the Remote Desktop Users alias */
//     SampLoadString(hInstance, IDS_ALIAS_REMOTE_DESKTOP_USERS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_REMOTE_DESKTOP_USERS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_REMOTE_DESKTOP_USERS);
//
//     /* Create the Network Configuration Operators alias */
//     SampLoadString(hInstance, IDS_ALIAS_NETWORK_CONFIGURATION_OPS_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_ALIAS_NETWORK_CONFIGURATION_OPS_COMMENT, szComment, 256);
//
//     SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_ALIAS_RID_NETWORK_CONFIGURATION_OPS);
//
//     if (SampProductType != NtProductWinNt)
//     {
//         /* Create the Performance Monitor Users alias (Server only) */
//         SampLoadString(hInstance, IDS_ALIAS_MONITORING_USERS_NAME, szName, 80);
//         SampLoadString(hInstance, IDS_ALIAS_MONITORING_USERS_COMMENT, szComment, 256);
//
//         SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                     szName,
//                                     szComment,
//                                     DOMAIN_ALIAS_RID_MONITORING_USERS);
//
//         /* Create the Performance Log Users alias (Server only) */
//         SampLoadString(hInstance, IDS_ALIAS_LOGGING_USERS_NAME, szName, 80);
//         SampLoadString(hInstance, IDS_ALIAS_LOGGING_USERS_COMMENT, szComment, 256);
//
//         SampSetupCreateAliasAccount(hBuiltinDomainKey,
//                                     szName,
//                                     szComment,
//                                     DOMAIN_ALIAS_RID_LOGGING_USERS);
//     }
//
//     /* Add the Administrator user to the Administrators alias */
//     pSid = AppendRidToSid(AccountDomainInfo->DomainSid,
//                           DOMAIN_USER_RID_ADMIN);
//     if (pSid != NULL)
//     {
//         SampSetupAddMemberToAlias(hBuiltinDomainKey,
//                                   DOMAIN_ALIAS_RID_ADMINS,
//                                   pSid);
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pSid);
//     }
//
//     /* Add the Guest user to the Guests alias */
//     pSid = AppendRidToSid(AccountDomainInfo->DomainSid,
//                           DOMAIN_USER_RID_GUEST);
//     if (pSid != NULL)
//     {
//         SampSetupAddMemberToAlias(hBuiltinDomainKey,
//                                   DOMAIN_ALIAS_RID_GUESTS,
//                                   pSid);
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pSid);
//     }
//
//     /* Add the Interactive SID to the Users alias */
//     SampSetupAddMemberToAlias(hBuiltinDomainKey,
//                               DOMAIN_ALIAS_RID_USERS,
//                               pInteractiveSid);
//
//     /* Add the Authenticated User SID to the Users alias */
//     SampSetupAddMemberToAlias(hBuiltinDomainKey,
//                               DOMAIN_ALIAS_RID_USERS,
//                               pAuthenticatedUserSid);
//
//     /* Create the Account domain */
//     Status = SampSetupCreateDomain(hServerKey,
//                                    L"Account",
//                                    L"",
//                                    AccountDomainInfo->DomainSid,
//                                    FALSE,
//                                    &hAccountDomainKey);
//     if (!NT_SUCCESS(Status))
//     {
//         bResult = FALSE;
//         goto done;
//     }
//
//     /* Create the None group */
//     SampLoadString(hInstance, IDS_GROUP_NONE_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_GROUP_NONE_COMMENT, szComment, 256);
//
//     SampSetupCreateGroupAccount(hAccountDomainKey,
//                                 szName,
//                                 szComment,
//                                 DOMAIN_GROUP_RID_USERS);
//
//     /* Create the Administrator user */
//     SampLoadString(hInstance, IDS_USER_ADMINISTRATOR_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_USER_ADMINISTRATOR_COMMENT, szComment, 256);
//
//     SampSetupCreateUserAccount(hAccountDomainKey,
//                                szName,
//                                szComment,
//                                AccountDomainInfo->DomainSid,
//                                DOMAIN_USER_RID_ADMIN,
//                                USER_DONT_EXPIRE_PASSWORD | USER_NORMAL_ACCOUNT);
//
//     /* Add the Adminitrator user to the Administrators alias */
//     SampSetupAddMemberToGroup(hAccountDomainKey,
//                               DOMAIN_GROUP_RID_USERS,
//                               DOMAIN_USER_RID_ADMIN);
//
//     /* Create the Guest user */
//     SampLoadString(hInstance, IDS_USER_GUEST_NAME, szName, 80);
//     SampLoadString(hInstance, IDS_USER_GUEST_COMMENT, szComment, 256);
//
//     SampSetupCreateUserAccount(hAccountDomainKey,
//                                szName,
//                                szComment,
//                                AccountDomainInfo->DomainSid,
//                                DOMAIN_USER_RID_GUEST,
//                                USER_ACCOUNT_DISABLED | USER_DONT_EXPIRE_PASSWORD | USER_NORMAL_ACCOUNT);
//
//     /* Add the Guest user to the Guests alias */
//     SampSetupAddMemberToGroup(hAccountDomainKey,
//                               DOMAIN_GROUP_RID_USERS,
//                               DOMAIN_USER_RID_GUEST);
//
// done:
//     if (AccountDomainInfo)
//         LsaFreeMemory(AccountDomainInfo);
//
//     if (pAuthenticatedUserSid)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pAuthenticatedUserSid);
//
//     if (pInteractiveSid)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pInteractiveSid);
//
//     if (pBuiltinSid)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pBuiltinSid);
//
//     SampRegCloseKey(&hAccountDomainKey);
//     SampRegCloseKey(&hBuiltinDomainKey);
//     SampRegCloseKey(&hServerKey);
//     SampRegCloseKey(&hSamKey);
//
//     TRACE("SampInitializeSAM() done\n");
//
//     return bResult;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
