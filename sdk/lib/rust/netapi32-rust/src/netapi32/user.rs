use crate::prelude::*;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MSV1_0_PACKAGE_NAME: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"MICROSOFT_AUTHENTICATION_PACKAGE_V1_0\0",
    )
};

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

pub static mut g_EnumContextListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub static mut g_EnumContextListLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub static mut g_EnumContextHandle: LONG = 0 as LONG;

pub type SAM_HANDLE = LPVOID;
pub type SAM_ENUMERATE_HANDLE = u32;

// _SAM_RID_ENUMERATION / PSAM_RID_ENUMERATION come from the bindgen `types` module (with the real
// { RelativeId, Name } fields). The prior local `pub struct _SAM_RID_ENUMERATION;` was an empty
// shadow that hid those fields — removed so the layout-exact bindgen type is used.

#[repr(C)]
pub struct _ENUM_CONTEXT {
    pub ListLink: LIST_ENTRY,
    pub EnumHandle: u32,
    pub ServerHandle: SAM_HANDLE,
    pub BuiltinDomainHandle: SAM_HANDLE,
    pub AccountDomainHandle: SAM_HANDLE,
    pub BuiltinDomainSid: PSID,
    pub AccountDomainSid: PSID,
    pub EnumerationContext: SAM_ENUMERATE_HANDLE,
    pub Buffer: PSAM_RID_ENUMERATION,
    pub Count: u32,
    pub Index: u32,
    pub BuiltinDone: u8,
}
pub type PENUM_CONTEXT = *mut _ENUM_CONTEXT;
pub type ENUM_CONTEXT = _ENUM_CONTEXT;
pub unsafe extern "C" fn DeltaTimeToSeconds(DeltaTime: LARGE_INTEGER) -> ULONG {
    if DeltaTime.QuadPart == 0 {
        return 0;
    }

    let seconds_quad: LONGLONG = -(DeltaTime.QuadPart as LONGLONG) / 10000000;
    let mut Seconds: LARGE_INTEGER = core::mem::zeroed();
    Seconds.QuadPart = seconds_quad;

    if Seconds.__bindgen_anon_1.HighPart != 0 {
        return TIMEQ_FOREVER;
    }

    Seconds.__bindgen_anon_1.LowPart as ULONG
}
unsafe extern "C" fn GetAllowedWorldAce(
    Acl: PACL,
    Ace: *mut PACCESS_ALLOWED_ACE,
) -> NTSTATUS {
    let mut WorldSid: [ULONG; 18] = [0u32; 18];
    let mut AclSize: ACL_SIZE_INFORMATION = core::mem::zeroed();
    let mut LocalAce: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    *Ace = core::ptr::null_mut::<_ACCESS_ALLOWED_ACE>();

    let mut nt_authority = SID_IDENTIFIER_AUTHORITY {
        Value: [0u8, 0u8, 0u8, 0u8, 0u8, 1u8],
    };
    RtlInitializeSid(
        core::ptr::addr_of_mut!(WorldSid) as *mut ULONG as PSID,
        core::ptr::addr_of_mut!(nt_authority),
        1u8,
    );
    *RtlSubAuthoritySid(core::ptr::addr_of_mut!(WorldSid) as *mut ULONG as PSID, 0u32) =
        SECURITY_WORLD_RID as ULONG;

    Status = RtlQueryInformationAcl(
        Acl,
        &mut AclSize as *mut ACL_SIZE_INFORMATION as PVOID,
        core::mem::size_of::<ACL_SIZE_INFORMATION>() as ULONG,
        AclSizeInformation,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    for i in 0..AclSize.AceCount {
        Status = RtlGetAce(Acl, i, core::ptr::addr_of_mut!(LocalAce));
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if (*(LocalAce as PACE_HEADER)).AceType as u32 == ACCESS_ALLOWED_ACE_TYPE
            && RtlEqualSid(
                core::ptr::addr_of_mut!(WorldSid) as *mut ULONG as PSID,
                core::ptr::addr_of_mut!((*(LocalAce as PACCESS_ALLOWED_ACE)).SidStart) as PSID,
            ) != 0
        {
            *Ace = LocalAce as PACCESS_ALLOWED_ACE;
            return STATUS_SUCCESS;
        }
    }

    STATUS_SUCCESS
}
unsafe extern "C" fn GetAccountFlags(AccountControl: ULONG, Dacl: PACL) -> ULONG {
    let mut Ace: PACCESS_ALLOWED_ACE = core::ptr::null_mut();
    let mut Flags: ULONG = UF_SCRIPT;

    if !Dacl.is_null() {
        let Status = GetAllowedWorldAce(Dacl, &mut Ace);
        if NT_SUCCESS(Status) {
            if Ace.is_null() {
                Flags |= UF_PASSWD_CANT_CHANGE;
            } else if ((*Ace).Mask & USER_CHANGE_PASSWORD) == 0 {
                Flags |= UF_PASSWD_CANT_CHANGE;
            }
        }
    }

    if (AccountControl & USER_ACCOUNT_DISABLED) != 0 {
        Flags |= UF_ACCOUNTDISABLE;
    }
    if (AccountControl & USER_HOME_DIRECTORY_REQUIRED) != 0 {
        Flags |= UF_HOMEDIR_REQUIRED;
    }
    if (AccountControl & USER_PASSWORD_NOT_REQUIRED) != 0 {
        Flags |= UF_PASSWD_NOTREQD;
    }
    if (AccountControl & USER_ACCOUNT_AUTO_LOCKED) != 0 {
        Flags |= UF_LOCKOUT;
    }
    if (AccountControl & USER_DONT_EXPIRE_PASSWORD) != 0 {
        Flags |= UF_DONT_EXPIRE_PASSWD;
    }

    if (AccountControl & USER_TEMP_DUPLICATE_ACCOUNT) != 0 {
        Flags |= UF_TEMP_DUPLICATE_ACCOUNT;
    } else if (AccountControl & USER_NORMAL_ACCOUNT) != 0 {
        Flags |= UF_NORMAL_ACCOUNT;
    } else if (AccountControl & USER_INTERDOMAIN_TRUST_ACCOUNT) != 0 {
        Flags |= UF_INTERDOMAIN_TRUST_ACCOUNT;
    } else if (AccountControl & USER_WORKSTATION_TRUST_ACCOUNT) != 0 {
        Flags |= UF_WORKSTATION_TRUST_ACCOUNT;
    } else if (AccountControl & USER_SERVER_TRUST_ACCOUNT) != 0 {
        Flags |= UF_SERVER_TRUST_ACCOUNT;
    }

    Flags
}
unsafe extern "C" fn GetAccountControl(Flags: ULONG) -> ULONG {
    let mut AccountControl: ULONG = 0;
    if Flags & UF_ACCOUNTDISABLE != 0 {
        AccountControl |= USER_ACCOUNT_DISABLED;
    }
    if Flags & UF_HOMEDIR_REQUIRED != 0 {
        AccountControl |= USER_HOME_DIRECTORY_REQUIRED;
    }
    if Flags & UF_PASSWD_NOTREQD != 0 {
        AccountControl |= USER_PASSWORD_NOT_REQUIRED;
    }
    if Flags & UF_LOCKOUT != 0 {
        AccountControl |= USER_ACCOUNT_AUTO_LOCKED;
    }
    if Flags & UF_DONT_EXPIRE_PASSWD != 0 {
        AccountControl |= USER_DONT_EXPIRE_PASSWORD;
    }
    if Flags & UF_TEMP_DUPLICATE_ACCOUNT != 0 {
        AccountControl |= USER_TEMP_DUPLICATE_ACCOUNT;
    } else if Flags & UF_NORMAL_ACCOUNT != 0 {
        AccountControl |= USER_NORMAL_ACCOUNT;
    } else if Flags & UF_INTERDOMAIN_TRUST_ACCOUNT != 0 {
        AccountControl |= USER_INTERDOMAIN_TRUST_ACCOUNT;
    } else if Flags & UF_WORKSTATION_TRUST_ACCOUNT != 0 {
        AccountControl |= USER_WORKSTATION_TRUST_ACCOUNT;
    } else if Flags & UF_SERVER_TRUST_ACCOUNT != 0 {
        AccountControl |= USER_SERVER_TRUST_ACCOUNT;
    }
    AccountControl
}
unsafe extern "C" fn GetPasswordAge(PasswordLastSet: PLARGE_INTEGER) -> u32 {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut SystemSecondsSince1970: u32 = 0;
    let mut PasswordSecondsSince1970: u32 = 0;
    let Status = NtQuerySystemTime(&mut SystemTime);
    if !NT_SUCCESS(Status) {
        return 0;
    }
    RtlTimeToSecondsSince1970(&mut SystemTime, &mut SystemSecondsSince1970);
    RtlTimeToSecondsSince1970(PasswordLastSet, &mut PasswordSecondsSince1970);
    SystemSecondsSince1970.wrapping_sub(PasswordSecondsSince1970)
}
unsafe extern "C" fn ChangeUserDacl(Dacl: PACL, Flags: u32) {
    let mut Ace: PACCESS_ALLOWED_ACE = core::ptr::null_mut();
    if Dacl.is_null() {
        return;
    }
    let Status = GetAllowedWorldAce(Dacl, &raw mut Ace);
    if Status < 0 {
        return;
    }
    if (Flags & UF_PASSWD_CANT_CHANGE) != 0 {
        (*Ace).Mask &= !USER_CHANGE_PASSWORD;
    } else {
        (*Ace).Mask |= USER_CHANGE_PASSWORD;
    }
}
unsafe extern "C" fn GetUserDacl(UserHandle: SAM_HANDLE, Dacl: *mut PACL) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
            b"GetUserDacl\0".as_ptr() as *const i8,
            297i32,
            b"(%p %p)\n\0".as_ptr() as *const i8,
            UserHandle,
            Dacl,
        );
    }
    *Dacl = core::ptr::null_mut();
    let mut SecurityDescriptor: *mut core::ffi::c_void = core::ptr::null_mut();
    let result = 'main: {
        let Status = SamQuerySecurityObject(
            UserHandle,
            DACL_SECURITY_INFORMATION as u32,
            &raw mut SecurityDescriptor,
        );
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
                    b"GetUserDacl\0".as_ptr() as *const i8,
                    306i32,
                    b"SamQuerySecurityObject() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            break 'main NetpNtStatusToApiStatus(Status);
        }
        let mut Present: u8 = 0;
        let mut Defaulted: u8 = 0;
        let mut SamDacl: PACL = core::ptr::null_mut();
        let Status = RtlGetDaclSecurityDescriptor(
            SecurityDescriptor,
            &raw mut Present,
            &raw mut SamDacl,
            &raw mut Defaulted,
        );
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
                    b"GetUserDacl\0".as_ptr() as *const i8,
                    317i32,
                    b"RtlGetDaclSecurityDescriptor() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            break 'main NERR_InternalError as u32;
        }
        if Present == 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
                    b"GetUserDacl\0".as_ptr() as *const i8,
                    324i32,
                    b"No DACL present\n\0".as_ptr() as *const i8,
                );
            }
            break 'main NERR_Success as u32;
        }
        let mut AclSize: ACL_SIZE_INFORMATION = core::mem::zeroed();
        let Status = RtlQueryInformationAcl(
            SamDacl,
            &mut AclSize as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        );
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
                    b"GetUserDacl\0".as_ptr() as *const i8,
                    335i32,
                    b"RtlQueryInformationAcl() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            break 'main NERR_InternalError as u32;
        }
        let LocalDacl = HeapAlloc(
            GetProcessHeap(),
            0u32,
            AclSize.AclBytesInUse as SIZE_T,
        ) as PACL;
        if LocalDacl.is_null() {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
                    b"GetUserDacl\0".as_ptr() as *const i8,
                    343i32,
                    b"Memory allocation failed\n\0".as_ptr() as *const i8,
                );
            }
            break 'main ERROR_NOT_ENOUGH_MEMORY as u32;
        }
        RtlCopyMemory(
            LocalDacl as *mut core::ffi::c_void,
            SamDacl as *const core::ffi::c_void,
            AclSize.AclBytesInUse as usize,
        );
        *Dacl = LocalDacl;
        break 'main NERR_Success as u32;
    };
    if !SecurityDescriptor.is_null() {
        SamFreeMemory(SecurityDescriptor);
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/user.c\0".as_ptr() as *const i8,
            b"GetUserDacl\0".as_ptr() as *const i8,
            358i32,
            b"done (ApiStatus: 0x%08lx)\n\0".as_ptr() as *const i8,
            result,
        );
    }
    result
}
unsafe extern "C" fn FreeUserInfo(UserInfo: PUSER_ALL_INFORMATION) {
    if !(*UserInfo).UserName.Buffer.is_null() {
        SamFreeMemory((*UserInfo).UserName.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).FullName.Buffer.is_null() {
        SamFreeMemory((*UserInfo).FullName.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).HomeDirectory.Buffer.is_null() {
        SamFreeMemory((*UserInfo).HomeDirectory.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).HomeDirectoryDrive.Buffer.is_null() {
        SamFreeMemory((*UserInfo).HomeDirectoryDrive.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).ScriptPath.Buffer.is_null() {
        SamFreeMemory((*UserInfo).ScriptPath.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).ProfilePath.Buffer.is_null() {
        SamFreeMemory((*UserInfo).ProfilePath.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).AdminComment.Buffer.is_null() {
        SamFreeMemory((*UserInfo).AdminComment.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).WorkStations.Buffer.is_null() {
        SamFreeMemory((*UserInfo).WorkStations.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).UserComment.Buffer.is_null() {
        SamFreeMemory((*UserInfo).UserComment.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).Parameters.Buffer.is_null() {
        SamFreeMemory((*UserInfo).Parameters.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).PrivateData.Buffer.is_null() {
        SamFreeMemory((*UserInfo).PrivateData.Buffer as *mut core::ffi::c_void);
    }
    if !(*UserInfo).LogonHours.LogonHours.is_null() {
        SamFreeMemory((*UserInfo).LogonHours.LogonHours as *mut core::ffi::c_void);
    }
    SamFreeMemory(UserInfo as *mut core::ffi::c_void);
}
unsafe extern "C" fn GetUserPrivileges(
    builtin_domain_handle: SAM_HANDLE,
    user_handle: SAM_HANDLE,
    account_domain_sid: PSID,
    relative_id: ULONG,
    priv_: PDWORD,
    auth_flags: PDWORD,
) -> NET_API_STATUS {
    let mut group_membership: PGROUP_MEMBERSHIP = core::ptr::null_mut();
    let mut group_count: ULONG = 0;
    let mut sid_count: ULONG = 0;
    let mut alias_count: ULONG = 0;
    let mut sid_array: *mut PSID = core::ptr::null_mut();
    let mut alias_array: PULONG = core::ptr::null_mut();
    let mut b_admin: BOOL = 0i32;
    let mut b_user: BOOL = 0i32;
    let mut api_status: NET_API_STATUS = NERR_Success;
    let mut status: NTSTATUS;

    // TRACE
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr()
                .cast::<::core::ffi::c_char>(),
            b"GetUserPrivileges\0"
                .as_ptr()
                .cast::<::core::ffi::c_char>(),
            426,
            b"GetUserPrivileges(%p)\n\0"
                .as_ptr()
                .cast::<::core::ffi::c_char>(),
            user_handle,
        );
    }

    'done: {
        status = SamGetGroupsForUser(user_handle, &mut group_membership, &mut group_count);
        if !NT_SUCCESS(status) {
            // ERR
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    b"GetUserPrivileges\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    434,
                    b"SamGetGroupsForUser() failed (Status 0x%08lx)\n\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    status,
                );
            }
            api_status = NetpNtStatusToApiStatus(status);
            break 'done;
        }

        api_status = NetApiBufferAllocate(
            ((group_count + 1) * ::core::mem::size_of::<PSID>() as ULONG) as DWORD,
            (&mut sid_array as *mut *mut PSID).cast::<PVOID>(),
        );
        if api_status != NERR_Success {
            break 'done;
        }

        sid_count = 0;
        api_status = BuildSidFromSidAndRid(account_domain_sid, relative_id, sid_array);
        if api_status != NERR_Success {
            break 'done;
        }
        sid_count += 1;

        let mut i: ULONG = 0;
        while i < group_count {
            api_status = BuildSidFromSidAndRid(
                account_domain_sid,
                (*group_membership.add(i as usize)).RelativeId,
                sid_array.add(i as usize + 1),
            );
            if api_status != NERR_Success {
                break 'done;
            }
            sid_count += 1;
            i += 1;
        }

        status = SamGetAliasMembership(
            builtin_domain_handle,
            sid_count,
            sid_array,
            &mut alias_count,
            &mut alias_array,
        );
        if !NT_SUCCESS(status) {
            // ERR
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/user.c\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    b"GetUserPrivileges\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    481,
                    b"SamGetAliasMembership() failed (Status 0x%08lx)\n\0"
                        .as_ptr()
                        .cast::<::core::ffi::c_char>(),
                    status,
                );
            }
            api_status = NetpNtStatusToApiStatus(status);
            break 'done;
        }

        *auth_flags = 0;
        let mut i: ULONG = 0;
        while i < alias_count {
            match *alias_array.add(i as usize) {
                544 => {
                    b_admin = 1i32;
                }
                545 => {
                    b_user = 1i32;
                }
                548 => {
                    *auth_flags |= AF_OP_ACCOUNTS;
                }
                549 => {
                    *auth_flags |= AF_OP_SERVER;
                }
                550 => {
                    *auth_flags |= AF_OP_PRINT;
                }
                _ => {}
            }
            i += 1;
        }

        if b_admin != 0 {
            *priv_ = USER_PRIV_ADMIN;
        } else if b_user != 0 {
            *priv_ = USER_PRIV_USER;
        } else {
            *priv_ = USER_PRIV_GUEST;
        }
    }

    // cleanup
    if !alias_array.is_null() {
        SamFreeMemory(alias_array as PVOID);
    }
    if !sid_array.is_null() {
        let mut j: ULONG = 0;
        while j < sid_count {
            NetApiBufferFree(*sid_array.add(j as usize) as PVOID);
            j += 1;
        }
        NetApiBufferFree(sid_array as PVOID);
    }
    if !group_membership.is_null() {
        SamFreeMemory(group_membership as PVOID);
    }

    api_status
}
unsafe extern "C" fn BuildUserInfoBuffer(
    BuiltinDomainHandle: SAM_HANDLE,
    UserHandle: SAM_HANDLE,
    AccountDomainSid: PSID,
    RelativeId: ULONG,
    level: DWORD,
    Buffer: *mut LPVOID,
) -> NET_API_STATUS {

    static LOGON_SERVER_BYTES: [u8; 8] = *b"\\\0\\\0*\0\0\0";
    let mut logon_server = _UNICODE_STRING {
        Length: (size_of::<[u16; 4]>() - size_of::<u16>()) as u16,
        MaximumLength: size_of::<[u16; 4]>() as u16,
        Buffer: &raw const LOGON_SERVER_BYTES as *const u8 as *mut u16,
    };

    let mut UserInfo: PUSER_ALL_INFORMATION = ptr::null_mut();
    let mut LocalBuffer: LPVOID = ptr::null_mut();
    let mut Dacl: PACL = ptr::null_mut();
    let mut Priv: DWORD = 0;
    let mut AuthFlags: DWORD = 0;
    let mut Size: ULONG = 0;

    *Buffer = ptr::null_mut();

    let mut ApiStatus = NERR_Success as DWORD;

    let final_status = 'done: {
        let status = SamQueryInformationUser(UserHandle, UserAllInformation, &mut UserInfo as *mut _ as *mut PVOID);
        if !NT_SUCCESS(status) {
            ApiStatus = NetpNtStatusToApiStatus(status);
            break 'done ApiStatus;
        }

        if level == 1 || level == 2 || level == 3 || level == 4 || level == 20 || level == 23 {
            ApiStatus = GetUserDacl(UserHandle, &mut Dacl);
            if ApiStatus != NERR_Success as DWORD {
                break 'done ApiStatus;
            }
        }

        if level == 1 || level == 2 || level == 3 || level == 4 || level == 11 {
            ApiStatus = GetUserPrivileges(BuiltinDomainHandle, UserHandle, AccountDomainSid, RelativeId, &mut Priv, &mut AuthFlags);
            if ApiStatus != NERR_Success as DWORD {
                break 'done ApiStatus;
            }
        }

        match level {
            0 => {
                Size = (size_of::<USER_INFO_0>() + (*UserInfo).UserName.Length as usize + size_of::<u16>()) as ULONG;
            }
            1 => {
                Size = (size_of::<USER_INFO_1>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectory.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).ScriptPath.Length as usize + size_of::<u16>()) as ULONG;
            }
            2 => {
                Size = (size_of::<USER_INFO_2>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectory.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).ScriptPath.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).UserComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).Parameters.Length as usize + size_of::<u16>()
                    + (*UserInfo).WorkStations.Length as usize + size_of::<u16>()
                    + logon_server.Length as usize + size_of::<u16>()) as ULONG;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    Size += (((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8;
                }
            }
            3 => {
                Size = (size_of::<USER_INFO_3>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectory.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).ScriptPath.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).UserComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).Parameters.Length as usize + size_of::<u16>()
                    + (*UserInfo).WorkStations.Length as usize + size_of::<u16>()
                    + logon_server.Length as usize + size_of::<u16>()
                    + (*UserInfo).ProfilePath.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectoryDrive.Length as usize + size_of::<u16>()) as ULONG;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    Size += (((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8;
                }
            }
            4 => {
                Size = (size_of::<USER_INFO_4>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectory.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).ScriptPath.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).UserComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).Parameters.Length as usize + size_of::<u16>()
                    + (*UserInfo).WorkStations.Length as usize + size_of::<u16>()
                    + logon_server.Length as usize + size_of::<u16>()
                    + (*UserInfo).ProfilePath.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectoryDrive.Length as usize + size_of::<u16>()) as ULONG;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    Size += (((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8;
                }
                Size += (RtlLengthSid(AccountDomainSid) as ULONG) + (size_of::<ULONG>() as ULONG);
            }
            10 => {
                Size = (size_of::<USER_INFO_10>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).UserComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()) as ULONG;
            }
            11 => {
                Size = (size_of::<USER_INFO_11>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).UserComment.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).HomeDirectory.Length as usize + size_of::<u16>()
                    + (*UserInfo).Parameters.Length as usize + size_of::<u16>()
                    + logon_server.Length as usize + size_of::<u16>()
                    + (*UserInfo).WorkStations.Length as usize + size_of::<u16>()) as ULONG;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    Size += (((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8;
                }
            }
            20 => {
                Size = (size_of::<USER_INFO_20>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()) as ULONG;
            }
            23 => {
                Size = (size_of::<USER_INFO_23>()
                    + (*UserInfo).UserName.Length as usize + size_of::<u16>()
                    + (*UserInfo).FullName.Length as usize + size_of::<u16>()
                    + (*UserInfo).AdminComment.Length as usize + size_of::<u16>()) as ULONG;
                Size += (RtlLengthSid(AccountDomainSid) as ULONG) + (size_of::<ULONG>() as ULONG);
            }
            _ => {
                ApiStatus = ERROR_INVALID_LEVEL as DWORD;
                break 'done ApiStatus;
            }
        }

        ApiStatus = NetApiBufferAllocate(Size as DWORD, &mut LocalBuffer) as DWORD;
        if ApiStatus != NERR_Success as DWORD {
            break 'done ApiStatus;
        }

        ptr::write_bytes(LocalBuffer as *mut u8, 0, Size as usize);

        let mut Ptr: *mut u16;
        match level {
            0 => {
                let ui0 = LocalBuffer as *mut USER_INFO_0;
                Ptr = (ui0 as usize + size_of::<USER_INFO_0>()) as *mut u16;
                (*ui0).usri0_name = Ptr;
                let byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
            }
            1 => {
                let ui1 = LocalBuffer as *mut USER_INFO_1;
                Ptr = (ui1 as usize + size_of::<USER_INFO_1>()) as *mut u16;
                (*ui1).usri1_name = Ptr;
                let mut byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui1).usri1_password = ptr::null_mut();
                (*ui1).usri1_password_age = GetPasswordAge(ptr::addr_of!((*UserInfo).PasswordLastSet).cast_mut());
                (*ui1).usri1_priv = Priv;
                (*ui1).usri1_home_dir = Ptr;
                byte_len = (*UserInfo).HomeDirectory.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectory.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui1).usri1_comment = Ptr;
                byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui1).usri1_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui1).usri1_script_path = Ptr;
                byte_len = (*UserInfo).ScriptPath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ScriptPath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
            }
            2 => {
                let ui2 = LocalBuffer as *mut USER_INFO_2;
                Ptr = (ui2 as usize + size_of::<USER_INFO_2>()) as *mut u16;
                (*ui2).usri2_name = Ptr;
                let mut byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_password = ptr::null_mut();
                (*ui2).usri2_password_age = GetPasswordAge(ptr::addr_of!((*UserInfo).PasswordLastSet).cast_mut());
                (*ui2).usri2_priv = Priv;
                (*ui2).usri2_home_dir = Ptr;
                byte_len = (*UserInfo).HomeDirectory.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectory.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_comment = Ptr;
                byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui2).usri2_script_path = Ptr;
                byte_len = (*UserInfo).ScriptPath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ScriptPath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_auth_flags = AuthFlags;
                (*ui2).usri2_full_name = Ptr;
                byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_usr_comment = Ptr;
                byte_len = (*UserInfo).UserComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_parms = Ptr;
                byte_len = (*UserInfo).Parameters.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).Parameters.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_workstations = Ptr;
                byte_len = (*UserInfo).WorkStations.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).WorkStations.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                if (*UserInfo).LastLogon.QuadPart == 0 {
                    (*ui2).usri2_last_logon = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogon).cast_mut(), &mut (*ui2).usri2_last_logon);
                }
                if (*UserInfo).LastLogoff.QuadPart == 0 {
                    (*ui2).usri2_last_logoff = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogoff).cast_mut(), &mut (*ui2).usri2_last_logoff);
                }
                if (*UserInfo).AccountExpires.QuadPart == MAXLONGLONG as i64 {
                    (*ui2).usri2_acct_expires = TIMEQ_FOREVER as DWORD;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).AccountExpires).cast_mut(), &mut (*ui2).usri2_acct_expires);
                }
                (*ui2).usri2_max_storage = USER_MAXSTORAGE_UNLIMITED as DWORD;
                (*ui2).usri2_units_per_week = (*UserInfo).LogonHours.UnitsPerWeek as DWORD;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    (*ui2).usri2_logon_hours = Ptr as *mut u8;
                    let logon_len = ((((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8) as usize;
                    ptr::copy_nonoverlapping((*UserInfo).LogonHours.LogonHours as *const u8, (*ui2).usri2_logon_hours, logon_len);
                    Ptr = (Ptr as usize + logon_len) as *mut u16;
                }
                (*ui2).usri2_bad_pw_count = (*UserInfo).BadPasswordCount as DWORD;
                (*ui2).usri2_num_logons = (*UserInfo).LogonCount as DWORD;
                (*ui2).usri2_logon_server = Ptr;
                byte_len = logon_server.Length as usize;
                ptr::copy_nonoverlapping(logon_server.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui2).usri2_country_code = (*UserInfo).CountryCode as DWORD;
                (*ui2).usri2_code_page = (*UserInfo).CodePage as DWORD;
            }
            3 => {
                let ui3 = LocalBuffer as *mut USER_INFO_3;
                Ptr = (ui3 as usize + size_of::<USER_INFO_3>()) as *mut u16;
                (*ui3).usri3_name = Ptr;
                let mut byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_password = ptr::null_mut();
                (*ui3).usri3_password_age = GetPasswordAge(ptr::addr_of!((*UserInfo).PasswordLastSet).cast_mut());
                (*ui3).usri3_priv = Priv;
                (*ui3).usri3_home_dir = Ptr;
                byte_len = (*UserInfo).HomeDirectory.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectory.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_comment = Ptr;
                byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui3).usri3_script_path = Ptr;
                byte_len = (*UserInfo).ScriptPath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ScriptPath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_auth_flags = AuthFlags;
                (*ui3).usri3_full_name = Ptr;
                byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_usr_comment = Ptr;
                byte_len = (*UserInfo).UserComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_parms = Ptr;
                byte_len = (*UserInfo).Parameters.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).Parameters.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_workstations = Ptr;
                byte_len = (*UserInfo).WorkStations.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).WorkStations.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                if (*UserInfo).LastLogon.QuadPart == 0 {
                    (*ui3).usri3_last_logon = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogon).cast_mut(), &mut (*ui3).usri3_last_logon);
                }
                if (*UserInfo).LastLogoff.QuadPart == 0 {
                    (*ui3).usri3_last_logoff = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogoff).cast_mut(), &mut (*ui3).usri3_last_logoff);
                }
                if (*UserInfo).AccountExpires.QuadPart == MAXLONGLONG as i64 {
                    (*ui3).usri3_acct_expires = TIMEQ_FOREVER as DWORD;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).AccountExpires).cast_mut(), &mut (*ui3).usri3_acct_expires);
                }
                (*ui3).usri3_max_storage = USER_MAXSTORAGE_UNLIMITED as DWORD;
                (*ui3).usri3_units_per_week = (*UserInfo).LogonHours.UnitsPerWeek as DWORD;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    (*ui3).usri3_logon_hours = Ptr as *mut u8;
                    let logon_len = ((((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8) as usize;
                    ptr::copy_nonoverlapping((*UserInfo).LogonHours.LogonHours as *const u8, (*ui3).usri3_logon_hours, logon_len);
                    Ptr = (Ptr as usize + logon_len) as *mut u16;
                }
                (*ui3).usri3_bad_pw_count = (*UserInfo).BadPasswordCount as DWORD;
                (*ui3).usri3_num_logons = (*UserInfo).LogonCount as DWORD;
                (*ui3).usri3_logon_server = Ptr;
                byte_len = logon_server.Length as usize;
                ptr::copy_nonoverlapping(logon_server.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_country_code = (*UserInfo).CountryCode as DWORD;
                (*ui3).usri3_code_page = (*UserInfo).CodePage as DWORD;
                (*ui3).usri3_user_id = RelativeId as DWORD;
                (*ui3).usri3_primary_group_id = (*UserInfo).PrimaryGroupId as DWORD;
                (*ui3).usri3_profile = Ptr;
                byte_len = (*UserInfo).ProfilePath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ProfilePath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_home_dir_drive = Ptr;
                byte_len = (*UserInfo).HomeDirectoryDrive.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectoryDrive.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui3).usri3_password_expired = ((*UserInfo).UserAccountControl & USER_PASSWORD_EXPIRED) as DWORD;
            }
            4 => {
                let ui4 = LocalBuffer as *mut USER_INFO_4;
                Ptr = (ui4 as usize + size_of::<USER_INFO_4>()) as *mut u16;
                (*ui4).usri4_name = Ptr;
                let mut byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_password = ptr::null_mut();
                (*ui4).usri4_password_age = GetPasswordAge(ptr::addr_of!((*UserInfo).PasswordLastSet).cast_mut());
                (*ui4).usri4_priv = Priv;
                (*ui4).usri4_home_dir = Ptr;
                byte_len = (*UserInfo).HomeDirectory.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectory.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_comment = Ptr;
                byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui4).usri4_script_path = Ptr;
                byte_len = (*UserInfo).ScriptPath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ScriptPath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_auth_flags = AuthFlags;
                (*ui4).usri4_full_name = Ptr;
                byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_usr_comment = Ptr;
                byte_len = (*UserInfo).UserComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_parms = Ptr;
                byte_len = (*UserInfo).Parameters.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).Parameters.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_workstations = Ptr;
                byte_len = (*UserInfo).WorkStations.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).WorkStations.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                if (*UserInfo).LastLogon.QuadPart == 0 {
                    (*ui4).usri4_last_logon = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogon).cast_mut(), &mut (*ui4).usri4_last_logon);
                }
                if (*UserInfo).LastLogoff.QuadPart == 0 {
                    (*ui4).usri4_last_logoff = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogoff).cast_mut(), &mut (*ui4).usri4_last_logoff);
                }
                if (*UserInfo).AccountExpires.QuadPart == MAXLONGLONG as i64 {
                    (*ui4).usri4_acct_expires = TIMEQ_FOREVER as DWORD;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).AccountExpires).cast_mut(), &mut (*ui4).usri4_acct_expires);
                }
                (*ui4).usri4_max_storage = USER_MAXSTORAGE_UNLIMITED as DWORD;
                (*ui4).usri4_units_per_week = (*UserInfo).LogonHours.UnitsPerWeek as DWORD;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    (*ui4).usri4_logon_hours = Ptr as *mut u8;
                    let logon_len = ((((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8) as usize;
                    ptr::copy_nonoverlapping((*UserInfo).LogonHours.LogonHours as *const u8, (*ui4).usri4_logon_hours, logon_len);
                    Ptr = (Ptr as usize + logon_len) as *mut u16;
                }
                (*ui4).usri4_bad_pw_count = (*UserInfo).BadPasswordCount as DWORD;
                (*ui4).usri4_num_logons = (*UserInfo).LogonCount as DWORD;
                (*ui4).usri4_logon_server = Ptr;
                byte_len = logon_server.Length as usize;
                ptr::copy_nonoverlapping(logon_server.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_country_code = (*UserInfo).CountryCode as DWORD;
                (*ui4).usri4_code_page = (*UserInfo).CodePage as DWORD;
                (*ui4).usri4_user_sid = Ptr as PSID;
                CopySidFromSidAndRid((*ui4).usri4_user_sid, AccountDomainSid, RelativeId);
                Ptr = (Ptr as usize + RtlLengthSid(AccountDomainSid) as usize + size_of::<ULONG>()) as *mut u16;
                (*ui4).usri4_primary_group_id = (*UserInfo).PrimaryGroupId as DWORD;
                (*ui4).usri4_profile = Ptr;
                byte_len = (*UserInfo).ProfilePath.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).ProfilePath.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_home_dir_drive = Ptr;
                byte_len = (*UserInfo).HomeDirectoryDrive.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectoryDrive.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui4).usri4_password_expired = ((*UserInfo).UserAccountControl & USER_PASSWORD_EXPIRED) as DWORD;
            }
            10 => {
                let ui10 = LocalBuffer as *mut USER_INFO_10;
                Ptr = (ui10 as usize + size_of::<USER_INFO_10>()) as *mut u16;
                (*ui10).usri10_name = Ptr;
                let byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui10).usri10_comment = Ptr;
                let byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui10).usri10_usr_comment = Ptr;
                let byte_len = (*UserInfo).UserComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui10).usri10_full_name = Ptr;
                let byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
            }
            11 => {
                let ui11 = LocalBuffer as *mut USER_INFO_11;
                Ptr = (ui11 as usize + size_of::<USER_INFO_11>()) as *mut u16;
                (*ui11).usri11_name = Ptr;
                let mut byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_comment = Ptr;
                byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_usr_comment = Ptr;
                byte_len = (*UserInfo).UserComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_full_name = Ptr;
                byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_priv = Priv;
                (*ui11).usri11_auth_flags = AuthFlags;
                (*ui11).usri11_password_age = GetPasswordAge(ptr::addr_of!((*UserInfo).PasswordLastSet).cast_mut());
                (*ui11).usri11_home_dir = Ptr;
                byte_len = (*UserInfo).HomeDirectory.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).HomeDirectory.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_parms = Ptr;
                byte_len = (*UserInfo).Parameters.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).Parameters.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                if (*UserInfo).LastLogon.QuadPart == 0 {
                    (*ui11).usri11_last_logon = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogon).cast_mut(), &mut (*ui11).usri11_last_logon);
                }
                if (*UserInfo).LastLogoff.QuadPart == 0 {
                    (*ui11).usri11_last_logoff = 0;
                } else {
                    RtlTimeToSecondsSince1970(ptr::addr_of!((*UserInfo).LastLogoff).cast_mut(), &mut (*ui11).usri11_last_logoff);
                }
                (*ui11).usri11_bad_pw_count = (*UserInfo).BadPasswordCount as DWORD;
                (*ui11).usri11_num_logons = (*UserInfo).LogonCount as DWORD;
                (*ui11).usri11_logon_server = Ptr;
                byte_len = logon_server.Length as usize;
                ptr::copy_nonoverlapping(logon_server.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_country_code = (*UserInfo).CountryCode as DWORD;
                (*ui11).usri11_workstations = Ptr;
                byte_len = (*UserInfo).WorkStations.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).WorkStations.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui11).usri11_max_storage = USER_MAXSTORAGE_UNLIMITED as DWORD;
                (*ui11).usri11_units_per_week = (*UserInfo).LogonHours.UnitsPerWeek as DWORD;
                if (*UserInfo).LogonHours.UnitsPerWeek as i32 > 0 {
                    (*ui11).usri11_logon_hours = Ptr as *mut u8;
                    let logon_len = ((((*UserInfo).LogonHours.UnitsPerWeek as ULONG) + 7) / 8) as usize;
                    ptr::copy_nonoverlapping((*UserInfo).LogonHours.LogonHours as *const u8, (*ui11).usri11_logon_hours, logon_len);
                    Ptr = (Ptr as usize + logon_len) as *mut u16;
                }
                (*ui11).usri11_code_page = (*UserInfo).CodePage as DWORD;
            }
            20 => {
                let ui20 = LocalBuffer as *mut USER_INFO_20;
                Ptr = (ui20 as usize + size_of::<USER_INFO_20>()) as *mut u16;
                (*ui20).usri20_name = Ptr;
                let byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui20).usri20_full_name = Ptr;
                let byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui20).usri20_comment = Ptr;
                let byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui20).usri20_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui20).usri20_user_id = RelativeId as DWORD;
            }
            23 => {
                let ui23 = LocalBuffer as *mut USER_INFO_23;
                Ptr = (ui23 as usize + size_of::<USER_INFO_23>()) as *mut u16;
                (*ui23).usri23_name = Ptr;
                let byte_len = (*UserInfo).UserName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).UserName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui23).usri23_full_name = Ptr;
                let byte_len = (*UserInfo).FullName.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).FullName.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui23).usri23_comment = Ptr;
                let byte_len = (*UserInfo).AdminComment.Length as usize;
                ptr::copy_nonoverlapping((*UserInfo).AdminComment.Buffer, Ptr, byte_len / size_of::<u16>());
                *Ptr.add(byte_len / size_of::<u16>()) = 0u16;
                Ptr = (Ptr as usize + byte_len + size_of::<u16>()) as *mut u16;
                (*ui23).usri23_flags = GetAccountFlags((*UserInfo).UserAccountControl, Dacl) as DWORD;
                (*ui23).usri23_user_sid = Ptr as PSID;
                CopySidFromSidAndRid((*ui23).usri23_user_sid, AccountDomainSid, RelativeId);
                Ptr = (Ptr as usize + RtlLengthSid(AccountDomainSid) as usize + size_of::<ULONG>()) as *mut u16;
            }
            _ => {}
        }

        break 'done ApiStatus;
    };

    if !UserInfo.is_null() {
        FreeUserInfo(UserInfo);
    }
    if !Dacl.is_null() {
        HeapFree(GetProcessHeap(), 0, Dacl as PVOID);
    }
    if final_status == NERR_Success as DWORD {
        *Buffer = LocalBuffer;
    } else if !LocalBuffer.is_null() {
        NetApiBufferFree(LocalBuffer as PVOID);
    }
    final_status
}
unsafe extern "C" fn SetUserInfo(
    UserHandle: SAM_HANDLE,
    UserInfo: LPBYTE,
    Level: DWORD,
    parm_err: PDWORD,
) -> DWORD {
    let mut user_all_info: USER_ALL_INFORMATION = core::mem::zeroed();
    let mut dacl: PACL = core::ptr::null_mut();
    let mut api_status: DWORD = NERR_Success;

    // Levels that require a pre-fetched DACL
    if Level == 1 || Level == 2 || Level == 3 || Level == 4 || Level == 22 || Level == 1008 {
        api_status = GetUserDacl(UserHandle, &mut dacl);
        if api_status != NERR_Success {
            // jump to cleanup
            return 'done: {
                if !dacl.is_null() {
                    HeapFree(GetProcessHeap(), 0, dacl as LPVOID);
                }
                api_status
            };
        }
    }

    // Dispatch by information level. Wrapped in a labeled block so the C `break;` that
    // exits the switch (skipping the rest of a case) maps to `break 'validate`.
    'validate: { match Level {
        0 => {
            let user_info0 = UserInfo as *mut USER_INFO_0;
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(user_all_info.UserName),
                (*user_info0).usri0_name as *const u16,
            );
            user_all_info.WhichFields |= USER_ALL_USERNAME;
        }
        1 => {
            let user_info1 = UserInfo as *mut USER_INFO_1;
            if !(*user_info1).usri1_password.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.NtPassword),
                    (*user_info1).usri1_password as *const u16,
                );
                user_all_info.NtPasswordPresent = 1u8;
                user_all_info.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
            }
            if !(*user_info1).usri1_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info1).usri1_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
            if !(*user_info1).usri1_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info1).usri1_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
            ChangeUserDacl(dacl, (*user_info1).usri1_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info1).usri1_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
            if !(*user_info1).usri1_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info1).usri1_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
        }
        2 => {
            let user_info2 = UserInfo as *mut USER_INFO_2;
            if !(*user_info2).usri2_password.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.NtPassword),
                    (*user_info2).usri2_password as *const u16,
                );
                user_all_info.NtPasswordPresent = 1u8;
                user_all_info.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
            }
            if !(*user_info2).usri2_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info2).usri2_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
            if !(*user_info2).usri2_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info2).usri2_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
            ChangeUserDacl(dacl, (*user_info2).usri2_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info2).usri2_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
            if !(*user_info2).usri2_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info2).usri2_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
            if !(*user_info2).usri2_full_name.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.FullName),
                    (*user_info2).usri2_full_name as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_FULLNAME;
            }
            if !(*user_info2).usri2_usr_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.UserComment),
                    (*user_info2).usri2_usr_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_USERCOMMENT;
            }
            if !(*user_info2).usri2_parms.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.Parameters),
                    (*user_info2).usri2_parms as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PARAMETERS;
            }
            if !(*user_info2).usri2_workstations.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.WorkStations),
                    (*user_info2).usri2_workstations as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_WORKSTATIONS;
            }
            if (*user_info2).usri2_acct_expires == TIMEQ_FOREVER {
                (*core::ptr::addr_of_mut!(user_all_info.AccountExpires)).QuadPart = MAXLONGLONG as i64;
            } else {
                RtlSecondsSince1970ToTime(
                    (*user_info2).usri2_acct_expires as ULONG,
                    core::ptr::addr_of_mut!(user_all_info.AccountExpires),
                );
            }
            user_all_info.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
            if !(*user_info2).usri2_logon_hours.is_null() {
                if (*user_info2).usri2_units_per_week > USHRT_MAX as u32 {
                    if !parm_err.is_null() {
                        *parm_err = USER_UNITS_PER_WEEK_PARMNUM as DWORD;
                    }
                    api_status = ERROR_INVALID_PARAMETER as DWORD;
                    break 'validate;
                }
                user_all_info.LogonHours.UnitsPerWeek = (*user_info2).usri2_units_per_week as u16;
                user_all_info.LogonHours.LogonHours = (*user_info2).usri2_logon_hours as *mut u8;
                user_all_info.WhichFields |= USER_ALL_LOGONHOURS;
            }
            user_all_info.CountryCode = (*user_info2).usri2_country_code as u16;
            user_all_info.WhichFields |= USER_ALL_COUNTRYCODE;
            user_all_info.CodePage = (*user_info2).usri2_code_page as u16;
            user_all_info.WhichFields |= USER_ALL_CODEPAGE;
        }
        3 => {
            let user_info3 = UserInfo as *mut USER_INFO_3;
            if !(*user_info3).usri3_password.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.NtPassword),
                    (*user_info3).usri3_password as *const u16,
                );
                user_all_info.NtPasswordPresent = 1u8;
                user_all_info.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
            }
            if !(*user_info3).usri3_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info3).usri3_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
            if !(*user_info3).usri3_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info3).usri3_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
            ChangeUserDacl(dacl, (*user_info3).usri3_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info3).usri3_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
            if !(*user_info3).usri3_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info3).usri3_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
            if !(*user_info3).usri3_full_name.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.FullName),
                    (*user_info3).usri3_full_name as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_FULLNAME;
            }
            if !(*user_info3).usri3_usr_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.UserComment),
                    (*user_info3).usri3_usr_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_USERCOMMENT;
            }
            if !(*user_info3).usri3_parms.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.Parameters),
                    (*user_info3).usri3_parms as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PARAMETERS;
            }
            if !(*user_info3).usri3_workstations.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.WorkStations),
                    (*user_info3).usri3_workstations as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_WORKSTATIONS;
            }
            if (*user_info3).usri3_acct_expires == TIMEQ_FOREVER {
                (*core::ptr::addr_of_mut!(user_all_info.AccountExpires)).QuadPart = MAXLONGLONG as i64;
            } else {
                RtlSecondsSince1970ToTime(
                    (*user_info3).usri3_acct_expires as ULONG,
                    core::ptr::addr_of_mut!(user_all_info.AccountExpires),
                );
            }
            user_all_info.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
            if !(*user_info3).usri3_logon_hours.is_null() {
                if (*user_info3).usri3_units_per_week > USHRT_MAX as u32 {
                    if !parm_err.is_null() {
                        *parm_err = USER_UNITS_PER_WEEK_PARMNUM as DWORD;
                    }
                    api_status = ERROR_INVALID_PARAMETER as DWORD;
                    break 'validate;
                }
                user_all_info.LogonHours.UnitsPerWeek = (*user_info3).usri3_units_per_week as u16;
                user_all_info.LogonHours.LogonHours = (*user_info3).usri3_logon_hours as *mut u8;
                user_all_info.WhichFields |= USER_ALL_LOGONHOURS;
            }
            user_all_info.CountryCode = (*user_info3).usri3_country_code as u16;
            user_all_info.WhichFields |= USER_ALL_COUNTRYCODE;
            user_all_info.CodePage = (*user_info3).usri3_code_page as u16;
            user_all_info.WhichFields |= USER_ALL_CODEPAGE;
            user_all_info.PrimaryGroupId = (*user_info3).usri3_primary_group_id as ULONG;
            user_all_info.WhichFields |= USER_ALL_PRIMARYGROUPID;
            if !(*user_info3).usri3_profile.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ProfilePath),
                    (*user_info3).usri3_profile as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PROFILEPATH;
            }
            if !(*user_info3).usri3_home_dir_drive.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectoryDrive),
                    (*user_info3).usri3_home_dir_drive as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
            }
            user_all_info.PasswordExpired = if (*user_info3).usri3_password_expired != 0 { 1u8 } else { 0u8 };
            user_all_info.WhichFields |= USER_ALL_PASSWORDEXPIRED;
        }
        4 => {
            let user_info4 = UserInfo as *mut USER_INFO_4;
            if !(*user_info4).usri4_password.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.NtPassword),
                    (*user_info4).usri4_password as *const u16,
                );
                user_all_info.NtPasswordPresent = 1u8;
                user_all_info.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
            }
            if !(*user_info4).usri4_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info4).usri4_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
            if !(*user_info4).usri4_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info4).usri4_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
            ChangeUserDacl(dacl, (*user_info4).usri4_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info4).usri4_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
            if !(*user_info4).usri4_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info4).usri4_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
            if !(*user_info4).usri4_full_name.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.FullName),
                    (*user_info4).usri4_full_name as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_FULLNAME;
            }
            if !(*user_info4).usri4_usr_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.UserComment),
                    (*user_info4).usri4_usr_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_USERCOMMENT;
            }
            if !(*user_info4).usri4_parms.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.Parameters),
                    (*user_info4).usri4_parms as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PARAMETERS;
            }
            if !(*user_info4).usri4_workstations.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.WorkStations),
                    (*user_info4).usri4_workstations as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_WORKSTATIONS;
            }
            if (*user_info4).usri4_acct_expires == TIMEQ_FOREVER {
                (*core::ptr::addr_of_mut!(user_all_info.AccountExpires)).QuadPart = MAXLONGLONG as i64;
            } else {
                RtlSecondsSince1970ToTime(
                    (*user_info4).usri4_acct_expires as ULONG,
                    core::ptr::addr_of_mut!(user_all_info.AccountExpires),
                );
            }
            user_all_info.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
            if !(*user_info4).usri4_logon_hours.is_null() {
                if (*user_info4).usri4_units_per_week > USHRT_MAX as u32 {
                    if !parm_err.is_null() {
                        *parm_err = USER_UNITS_PER_WEEK_PARMNUM as DWORD;
                    }
                    api_status = ERROR_INVALID_PARAMETER as DWORD;
                    break 'validate;
                }
                user_all_info.LogonHours.UnitsPerWeek = (*user_info4).usri4_units_per_week as u16;
                user_all_info.LogonHours.LogonHours = (*user_info4).usri4_logon_hours as *mut u8;
                user_all_info.WhichFields |= USER_ALL_LOGONHOURS;
            }
            user_all_info.CountryCode = (*user_info4).usri4_country_code as u16;
            user_all_info.WhichFields |= USER_ALL_COUNTRYCODE;
            user_all_info.CodePage = (*user_info4).usri4_code_page as u16;
            user_all_info.WhichFields |= USER_ALL_CODEPAGE;
            user_all_info.PrimaryGroupId = (*user_info4).usri4_primary_group_id as ULONG;
            user_all_info.WhichFields |= USER_ALL_PRIMARYGROUPID;
            if !(*user_info4).usri4_profile.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ProfilePath),
                    (*user_info4).usri4_profile as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PROFILEPATH;
            }
            if !(*user_info4).usri4_home_dir_drive.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectoryDrive),
                    (*user_info4).usri4_home_dir_drive as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
            }
            user_all_info.PasswordExpired = if (*user_info4).usri4_password_expired != 0 { 1u8 } else { 0u8 };
            user_all_info.WhichFields |= USER_ALL_PASSWORDEXPIRED;
        }
        22 => {
            let user_info22 = UserInfo as *mut USER_INFO_22;
            if !(*user_info22).usri22_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info22).usri22_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
            if !(*user_info22).usri22_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info22).usri22_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
            ChangeUserDacl(dacl, (*user_info22).usri22_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info22).usri22_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
            if !(*user_info22).usri22_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info22).usri22_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
            if !(*user_info22).usri22_full_name.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.FullName),
                    (*user_info22).usri22_full_name as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_FULLNAME;
            }
            if !(*user_info22).usri22_usr_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.UserComment),
                    (*user_info22).usri22_usr_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_USERCOMMENT;
            }
            if !(*user_info22).usri22_parms.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.Parameters),
                    (*user_info22).usri22_parms as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PARAMETERS;
            }
            if !(*user_info22).usri22_workstations.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.WorkStations),
                    (*user_info22).usri22_workstations as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_WORKSTATIONS;
            }
            if (*user_info22).usri22_acct_expires == TIMEQ_FOREVER {
                (*core::ptr::addr_of_mut!(user_all_info.AccountExpires)).QuadPart = MAXLONGLONG as i64;
            } else {
                RtlSecondsSince1970ToTime(
                    (*user_info22).usri22_acct_expires as ULONG,
                    core::ptr::addr_of_mut!(user_all_info.AccountExpires),
                );
            }
            user_all_info.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
            if !(*user_info22).usri22_logon_hours.is_null() {
                if (*user_info22).usri22_units_per_week > USHRT_MAX as u32 {
                    if !parm_err.is_null() {
                        *parm_err = USER_UNITS_PER_WEEK_PARMNUM as DWORD;
                    }
                    api_status = ERROR_INVALID_PARAMETER as DWORD;
                    break 'validate;
                }
                user_all_info.LogonHours.UnitsPerWeek = (*user_info22).usri22_units_per_week as u16;
                user_all_info.LogonHours.LogonHours = (*user_info22).usri22_logon_hours as *mut u8;
                user_all_info.WhichFields |= USER_ALL_LOGONHOURS;
            }
            user_all_info.CountryCode = (*user_info22).usri22_country_code as u16;
            user_all_info.WhichFields |= USER_ALL_COUNTRYCODE;
            user_all_info.CodePage = (*user_info22).usri22_code_page as u16;
            user_all_info.WhichFields |= USER_ALL_CODEPAGE;
        }
        1003 => {
            let user_info1003 = UserInfo as *mut USER_INFO_1003;
            if !(*user_info1003).usri1003_password.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.NtPassword),
                    (*user_info1003).usri1003_password as *const u16,
                );
                user_all_info.NtPasswordPresent = 1u8;
                user_all_info.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
            }
        }
        1006 => {
            let user_info1006 = UserInfo as *mut USER_INFO_1006;
            if !(*user_info1006).usri1006_home_dir.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectory),
                    (*user_info1006).usri1006_home_dir as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORY;
            }
        }
        1007 => {
            let user_info1007 = UserInfo as *mut USER_INFO_1007;
            if !(*user_info1007).usri1007_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.AdminComment),
                    (*user_info1007).usri1007_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_ADMINCOMMENT;
            }
        }
        1008 => {
            let user_info1008 = UserInfo as *mut USER_INFO_1008;
            ChangeUserDacl(dacl, (*user_info1008).usri1008_flags as ULONG);
            user_all_info.UserAccountControl = GetAccountControl((*user_info1008).usri1008_flags as ULONG);
            user_all_info.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
        }
        1009 => {
            let user_info1009 = UserInfo as *mut USER_INFO_1009;
            if !(*user_info1009).usri1009_script_path.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ScriptPath),
                    (*user_info1009).usri1009_script_path as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_SCRIPTPATH;
            }
        }
        1011 => {
            let user_info1011 = UserInfo as *mut USER_INFO_1011;
            if !(*user_info1011).usri1011_full_name.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.FullName),
                    (*user_info1011).usri1011_full_name as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_FULLNAME;
            }
        }
        1012 => {
            let user_info1012 = UserInfo as *mut USER_INFO_1012;
            if !(*user_info1012).usri1012_usr_comment.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.UserComment),
                    (*user_info1012).usri1012_usr_comment as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_USERCOMMENT;
            }
        }
        1013 => {
            let user_info1013 = UserInfo as *mut USER_INFO_1013;
            if !(*user_info1013).usri1013_parms.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.Parameters),
                    (*user_info1013).usri1013_parms as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PARAMETERS;
            }
        }
        1014 => {
            let user_info1014 = UserInfo as *mut USER_INFO_1014;
            if !(*user_info1014).usri1014_workstations.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.WorkStations),
                    (*user_info1014).usri1014_workstations as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_WORKSTATIONS;
            }
        }
        1017 => {
            let user_info1017 = UserInfo as *mut USER_INFO_1017;
            if (*user_info1017).usri1017_acct_expires == TIMEQ_FOREVER {
                (*core::ptr::addr_of_mut!(user_all_info.AccountExpires)).QuadPart = MAXLONGLONG as i64;
            } else {
                RtlSecondsSince1970ToTime(
                    (*user_info1017).usri1017_acct_expires as ULONG,
                    core::ptr::addr_of_mut!(user_all_info.AccountExpires),
                );
            }
            user_all_info.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
        }
        1020 => {
            let user_info1020 = UserInfo as *mut USER_INFO_1020;
            if !(*user_info1020).usri1020_logon_hours.is_null() {
                if (*user_info1020).usri1020_units_per_week > USHRT_MAX as u32 {
                    if !parm_err.is_null() {
                        *parm_err = USER_UNITS_PER_WEEK_PARMNUM as DWORD;
                    }
                    api_status = ERROR_INVALID_PARAMETER as DWORD;
                    break 'validate;
                }
                user_all_info.LogonHours.UnitsPerWeek = (*user_info1020).usri1020_units_per_week as u16;
                user_all_info.LogonHours.LogonHours = (*user_info1020).usri1020_logon_hours as *mut u8;
                user_all_info.WhichFields |= USER_ALL_LOGONHOURS;
            }
        }
        1024 => {
            let user_info1024 = UserInfo as *mut USER_INFO_1024;
            user_all_info.CountryCode = (*user_info1024).usri1024_country_code as u16;
            user_all_info.WhichFields |= USER_ALL_COUNTRYCODE;
        }
        1025 => {
            let user_info1025 = UserInfo as *mut USER_INFO_1025;
            user_all_info.CodePage = (*user_info1025).usri1025_code_page as u16;
            user_all_info.WhichFields |= USER_ALL_CODEPAGE;
        }
        1051 => {
            let user_info1051 = UserInfo as *mut USER_INFO_1051;
            user_all_info.PrimaryGroupId = (*user_info1051).usri1051_primary_group_id as ULONG;
            user_all_info.WhichFields |= USER_ALL_PRIMARYGROUPID;
        }
        1052 => {
            let user_info1052 = UserInfo as *mut USER_INFO_1052;
            if !(*user_info1052).usri1052_profile.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.ProfilePath),
                    (*user_info1052).usri1052_profile as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_PROFILEPATH;
            }
        }
        1053 => {
            let user_info1053 = UserInfo as *mut USER_INFO_1053;
            if !(*user_info1053).usri1053_home_dir_drive.is_null() {
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(user_all_info.HomeDirectoryDrive),
                    (*user_info1053).usri1053_home_dir_drive as *const u16,
                );
                user_all_info.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
            }
        }
        1018 | _ => {}
    } } // end 'validate block

    if api_status == NERR_Success {
        let status = SamSetInformationUser(UserHandle, UserAllInformation, &mut user_all_info as *mut _ as PVOID);
        if status < 0 {
            api_status = NetpNtStatusToApiStatus(status);
        }
    }

    if !dacl.is_null() {
        HeapFree(GetProcessHeap(), 0, dacl as LPVOID);
    }

    api_status
}
unsafe extern "C" fn OpenUserByName(
    DomainHandle: SAM_HANDLE,
    UserName: PUNICODE_STRING,
    DesiredAccess: ULONG,
    UserHandle: PSAM_HANDLE,
) -> DWORD {
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SamLookupNamesInDomain(
        DomainHandle,
        1 as ULONG,
        UserName,
        core::ptr::addr_of_mut!(RelativeIds),
        core::ptr::addr_of_mut!(Use),
    );
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/netapi32/user.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"OpenUserByName\0" as *const u8 as *const core::ffi::c_char,
                2233 as core::ffi::c_int,
                b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0" as *const u8
                    as *const core::ffi::c_char,
                UserName,
                Status,
            );
        }
        return NetpNtStatusToApiStatus(Status);
    }
    if *Use.offset(0) as core::ffi::c_uint
        != SidTypeUser as core::ffi::c_int as core::ffi::c_uint
    {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/netapi32/user.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"OpenUserByName\0" as *const u8 as *const core::ffi::c_char,
                2240 as core::ffi::c_int,
                b"Object is not a user!\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        ApiStatus = NERR_GroupNotFound as DWORD;
    } else {
        Status = SamOpenUser(
            DomainHandle,
            DesiredAccess as ACCESS_MASK,
            *RelativeIds.offset(0),
            UserHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/user.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"OpenUserByName\0" as *const u8 as *const core::ffi::c_char,
                    2252 as core::ffi::c_int,
                    b"SamOpenUser failed (Status %08lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        }
    }
    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as PVOID);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as PVOID);
    }
    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetUserAdd(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    const NERR_Success: DWORD = 0;
    const PARM_ERROR_NONE: DWORD = 0;

    let mut ServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut UserName: _UNICODE_STRING = core::mem::zeroed();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GrantedAccess: ULONG = 0;
    let mut RelativeId: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if !parm_err.is_null() {
        *parm_err = PARM_ERROR_NONE;
    }

    match level {
        1 | 2 | 3 | 4 => {}
        _ => return ERROR_INVALID_LEVEL,
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername);
    }

    'done: loop {
        Status = SamConnect(
            if !servername.is_null() {
                &mut ServerName
            } else {
                core::ptr::null_mut()
            },
            &mut ServerHandle,
            SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() {
                &mut ServerName
            } else {
                core::ptr::null_mut()
            },
            DOMAIN_CREATE_USER | DOMAIN_LOOKUP | DOMAIN_READ_PASSWORD_PARAMETERS,
            &mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        RtlInitUnicodeString(
            &mut UserName,
            (*(bufptr as PUSER_INFO_1)).usri1_name,
        );
        Status = SamCreateUser2InDomain(
            DomainHandle,
            &mut UserName,
            USER_NORMAL_ACCOUNT,
            USER_ALL_ACCESS | DELETE | WRITE_DAC,
            &mut UserHandle,
            &mut GrantedAccess,
            &mut RelativeId,
        );
        if !NT_SUCCESS(Status) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        ApiStatus = SetUserInfo(UserHandle, bufptr, level, parm_err as PDWORD);
        if ApiStatus != NERR_Success {
            break 'done;
        }
        break;
    }

    if !UserHandle.is_null() {
        if ApiStatus != NERR_Success {
            SamDeleteUser(UserHandle);
        } else {
            SamCloseHandle(UserHandle);
        }
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetUserChangePassword(
    domainname: LPCWSTR,
    username: LPCWSTR,
    oldpassword: LPCWSTR,
    newpassword: LPCWSTR,
) -> DWORD {
    let mut RequestBuffer: PMSV1_0_CHANGEPASSWORD_REQUEST = core::ptr::null_mut();
    let mut ResponseBuffer: PMSV1_0_CHANGEPASSWORD_RESPONSE = core::ptr::null_mut();
    let mut RequestBufferSize: ULONG;
    let mut ResponseBufferSize: ULONG = 0u32;
    let mut Ptr: LPWSTR;
    let mut PackageName: ANSI_STRING = core::mem::zeroed();
    let mut AuthenticationPackage: ULONG = 0u32;
    let mut LsaHandle: HANDLE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ProtocolStatus: NTSTATUS = STATUS_SUCCESS;

    'done: {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetUserChangePassword\0".as_ptr() as *const ::core::ffi::c_char,
            2416i32,
            b"(%s, %s, ..., ...)\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_w(domainname),
            debugstr_w(username),
        );
    }

    if oldpassword.is_null() || newpassword.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    Status = LsaConnectUntrusted(&raw mut LsaHandle) as NTSTATUS;
    if !NT_SUCCESS(Status) {
        return NetpNtStatusToApiStatus(Status);
    }

    RtlInitAnsiString(
        &raw mut PackageName,
        MSV1_0_PACKAGE_NAME.as_ptr() as *const ::core::ffi::c_char,
    );

    Status = LsaLookupAuthenticationPackage(
        LsaHandle,
        &raw mut PackageName as PLSA_STRING,
        &raw mut AuthenticationPackage,
    ) as NTSTATUS;

    if !NT_SUCCESS(Status) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    RequestBufferSize = (core::mem::size_of::<MSV1_0_CHANGEPASSWORD_REQUEST>()
        + (wcslen(domainname) + 1) * core::mem::size_of::<WCHAR>()
        + (wcslen(username) + 1) * core::mem::size_of::<WCHAR>()
        + (wcslen(oldpassword) + 1) * core::mem::size_of::<WCHAR>()
        + (wcslen(newpassword) + 1) * core::mem::size_of::<WCHAR>()) as ULONG;

    ApiStatus = NetApiBufferAllocate(RequestBufferSize, &raw mut RequestBuffer as *mut PVOID);
    if ApiStatus != NERR_Success {
        break 'done;
    }

    (*RequestBuffer).MessageType = MsV1_0ChangePassword;
    (*RequestBuffer).Impersonating = 1u8; // TRUE

    Ptr = (RequestBuffer as usize)
        .wrapping_add(core::mem::size_of::<MSV1_0_CHANGEPASSWORD_REQUEST>()) as LPWSTR;

    (*RequestBuffer).DomainName.Length =
        (wcslen(domainname) * core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).DomainName.MaximumLength =
        ((*RequestBuffer).DomainName.Length as usize + core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).DomainName.Buffer = Ptr;

    core::ptr::copy_nonoverlapping(
        domainname as *const u8,
        Ptr as *mut u8,
        (*RequestBuffer).DomainName.MaximumLength as usize,
    );

    Ptr = (Ptr as usize)
        .wrapping_add((*RequestBuffer).DomainName.MaximumLength as usize) as LPWSTR;

    (*RequestBuffer).AccountName.Length =
        (wcslen(username) * core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).AccountName.MaximumLength =
        ((*RequestBuffer).AccountName.Length as usize + core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).AccountName.Buffer = Ptr;

    core::ptr::copy_nonoverlapping(
        username as *const u8,
        Ptr as *mut u8,
        (*RequestBuffer).AccountName.MaximumLength as usize,
    );

    Ptr = (Ptr as usize)
        .wrapping_add((*RequestBuffer).AccountName.MaximumLength as usize) as LPWSTR;

    (*RequestBuffer).OldPassword.Length =
        (wcslen(oldpassword) * core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).OldPassword.MaximumLength =
        ((*RequestBuffer).OldPassword.Length as usize + core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).OldPassword.Buffer = Ptr;

    core::ptr::copy_nonoverlapping(
        oldpassword as *const u8,
        Ptr as *mut u8,
        (*RequestBuffer).OldPassword.MaximumLength as usize,
    );

    Ptr = (Ptr as usize)
        .wrapping_add((*RequestBuffer).OldPassword.MaximumLength as usize) as LPWSTR;

    (*RequestBuffer).NewPassword.Length =
        (wcslen(newpassword) * core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).NewPassword.MaximumLength =
        ((*RequestBuffer).NewPassword.Length as usize + core::mem::size_of::<WCHAR>()) as USHORT;
    (*RequestBuffer).NewPassword.Buffer = Ptr;

    core::ptr::copy_nonoverlapping(
        newpassword as *const u8,
        Ptr as *mut u8,
        (*RequestBuffer).NewPassword.MaximumLength as usize,
    );

    Status = LsaCallAuthenticationPackage(
        LsaHandle,
        AuthenticationPackage,
        RequestBuffer as PVOID,
        RequestBufferSize,
        &raw mut ResponseBuffer as *mut PVOID,
        &raw mut ResponseBufferSize,
        &raw mut ProtocolStatus,
    ) as NTSTATUS;

    if !NT_SUCCESS(Status) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else if !NT_SUCCESS(ProtocolStatus) {
        ApiStatus = NetpNtStatusToApiStatus(ProtocolStatus);
    }

    } // end 'done block
    if !RequestBuffer.is_null() {
        NetApiBufferFree(RequestBuffer as PVOID);
    }
    if !ResponseBuffer.is_null() {
        LsaFreeReturnBuffer(ResponseBuffer as PVOID);
    }
    if !LsaHandle.is_null() {
        NtClose(LsaHandle);
    }

    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetUserDel(
    servername: *const u16,
    username: *const u16,
) -> DWORD {
    let mut server_name = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut user_name = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut server_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut domain_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut user_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut api_status: DWORD = NERR_Success as DWORD;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"user.c\0".as_ptr() as *const i8,
            b"NetUserDel\0".as_ptr() as *const i8,
            2554,
            b"(%s, %s)\n\0".as_ptr() as *const i8,
            debugstr_w(servername),
            debugstr_w(username),
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut server_name, servername);
    }
    RtlInitUnicodeString(&raw mut user_name, username);

    loop {
        status = SamConnect(
            if !servername.is_null() { &raw mut server_name } else { core::ptr::null_mut::<_UNICODE_STRING>() },
            &raw mut server_handle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
        );
        if !NT_SUCCESS(status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"user.c\0".as_ptr() as *const i8,
                    b"NetUserDel\0".as_ptr() as *const i8,
                    2568,
                    b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    status,
                );
            }
            api_status = NetpNtStatusToApiStatus(status);
            break;
        }

        status = OpenBuiltinDomain(server_handle, DOMAIN_LOOKUP as ULONG, &raw mut domain_handle);
        if !NT_SUCCESS(status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"user.c\0".as_ptr() as *const i8,
                    b"NetUserDel\0".as_ptr() as *const i8,
                    2579,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    status,
                );
            }
            api_status = NetpNtStatusToApiStatus(status);
            break;
        }

        api_status = OpenUserByName(
            domain_handle,
            &raw mut user_name,
            DELETE as ULONG,
            &raw mut user_handle,
        );
        if api_status != NERR_Success as DWORD && api_status != ERROR_NONE_MAPPED as DWORD {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"user.c\0".as_ptr() as *const i8,
                    b"NetUserDel\0".as_ptr() as *const i8,
                    2591,
                    b"OpenUserByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                    &raw mut user_name,
                    api_status,
                );
            }
            break;
        }

        if user_handle.is_null() {
            if !domain_handle.is_null() {
                SamCloseHandle(domain_handle);
                domain_handle = core::ptr::null_mut();
            }

            status = OpenAccountDomain(
                server_handle,
                if !servername.is_null() { &raw mut server_name } else { core::ptr::null_mut::<_UNICODE_STRING>() },
                DOMAIN_LOOKUP as ULONG,
                &raw mut domain_handle,
            );
            if !NT_SUCCESS(status) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"user.c\0".as_ptr() as *const i8,
                        b"NetUserDel\0".as_ptr() as *const i8,
                        2610,
                        b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        status,
                    );
                }
                api_status = NetpNtStatusToApiStatus(status);
                break;
            }

            api_status = OpenUserByName(
                domain_handle,
                &raw mut user_name,
                DELETE as ULONG,
                &raw mut user_handle,
            );
            if api_status != NERR_Success as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"user.c\0".as_ptr() as *const i8,
                        b"NetUserDel\0".as_ptr() as *const i8,
                        2622,
                        b"OpenUserByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                        &raw mut user_name,
                        api_status,
                    );
                }
                if api_status == ERROR_NONE_MAPPED as DWORD {
                    api_status = NERR_UserNotFound as DWORD;
                }
                break;
            }
        }

        status = SamDeleteUser(user_handle);
        if !NT_SUCCESS(status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"user.c\0".as_ptr() as *const i8,
                    b"NetUserDel\0".as_ptr() as *const i8,
                    2633,
                    b"SamDeleteUser failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    status,
                );
            }
            api_status = NetpNtStatusToApiStatus(status);
            break;
        }
        user_handle = core::ptr::null_mut();
        break;
    }

    if !user_handle.is_null() {
        SamCloseHandle(user_handle);
    }
    if !domain_handle.is_null() {
        SamCloseHandle(domain_handle);
    }
    if !server_handle.is_null() {
        SamCloseHandle(server_handle);
    }

    api_status
}
unsafe extern "C" fn AllocateEnumContext(AllocatedEnumContext: *mut PENUM_CONTEXT) -> DWORD {
    let mut ApiStatus: DWORD;
    let mut EnumContext: PENUM_CONTEXT = core::ptr::null_mut();

    ApiStatus = NetApiBufferAllocate(
        core::mem::size_of::<ENUM_CONTEXT>() as DWORD,
        &raw mut EnumContext as *mut LPVOID,
    );
    if ApiStatus != NERR_Success as DWORD {
        return ApiStatus;
    }

    (*EnumContext).EnumerationContext = 0;
    (*EnumContext).Buffer = core::ptr::null_mut();
    (*EnumContext).Count = 0;
    (*EnumContext).Index = 0;
    (*EnumContext).BuiltinDone = 0u8; // BOOLEAN FALSE

    (*EnumContext).EnumHandle = InterlockedIncrement(&raw mut g_EnumContextHandle) as ULONG;
    if (*EnumContext).EnumHandle == 0 {
        (*EnumContext).EnumHandle = InterlockedIncrement(&raw mut g_EnumContextHandle) as ULONG;
    }

    EnterCriticalSection(&raw mut g_EnumContextListLock);
    InsertTailList(
        &raw mut g_EnumContextListHead,
        &raw mut (*EnumContext).ListLink,
    );
    LeaveCriticalSection(&raw mut g_EnumContextListLock);

    *AllocatedEnumContext = EnumContext;
    return NERR_Success as DWORD;
}
unsafe extern "C" fn FreeEnumContext(EnumContext: PENUM_CONTEXT) {
    EnterCriticalSection(core::ptr::addr_of_mut!(g_EnumContextListLock));
    RemoveEntryList(core::ptr::addr_of_mut!((*EnumContext).ListLink));
    LeaveCriticalSection(core::ptr::addr_of_mut!(g_EnumContextListLock));
    NetApiBufferFree(EnumContext as LPVOID);
}
unsafe extern "C" fn LookupEnumContext(EnumerationHandle: SAM_ENUMERATE_HANDLE) -> PENUM_CONTEXT {
    let mut FoundEnumContext: PENUM_CONTEXT = core::ptr::null_mut();
    EnterCriticalSection(core::ptr::addr_of_mut!(g_EnumContextListLock));
    let mut ListEntry = g_EnumContextListHead.Flink;
    while ListEntry as *mut _ != core::ptr::addr_of_mut!(g_EnumContextListHead) {
        let EnumContext = (ListEntry as usize)
            .wrapping_sub(core::mem::offset_of!(_ENUM_CONTEXT, ListLink))
            as PENUM_CONTEXT;
        if (*EnumContext).EnumHandle == EnumerationHandle {
            FoundEnumContext = EnumContext;
            break;
        }
        ListEntry = (*ListEntry).Flink;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(g_EnumContextListLock));
    FoundEnumContext
}

#[no_mangle]
pub unsafe extern "system" fn NetUserEnum(
    servername: LPCWSTR,
    level: DWORD,
    filter: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resume_handle: LPDWORD,
) -> DWORD {
    let mut current_block: u64;
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut CurrentUser: PSAM_RID_ENUMERATION = core::ptr::null_mut::<_SAM_RID_ENUMERATION>();
    let mut EnumContext: PENUM_CONTEXT = core::ptr::null_mut::<_ENUM_CONTEXT>();
    let mut Buffer: LPVOID = NULL;
    let mut i: ULONG = 0;
    let mut UserHandle: SAM_HANDLE = NULL;
    let mut DesiredAccess: ACCESS_MASK = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/user.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
            2760 as ::core::ffi::c_int,
            b"(%s %d 0x%d %p %d %p %p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            level,
            filter,
            bufptr,
            prefmaxlen,
            entriesread,
            totalentries,
            resume_handle,
        );
    }

    *entriesread = 0 as DWORD;
    *totalentries = 0 as DWORD;
    *bufptr = core::ptr::null_mut::<BYTE>();

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
    }

    if !resume_handle.is_null() && *resume_handle != 0 as ::core::ffi::c_ulong {
        EnumContext = LookupEnumContext(*resume_handle);
        current_block = 6528285054092551010;
    } else {
        ApiStatus = AllocateEnumContext(&raw mut EnumContext);
        if ApiStatus != NERR_Success as ::core::ffi::c_ulong {
            current_block = 10801318626883441134;
        } else {
            Status = SamConnect(
                if !servername.is_null() {
                    &raw mut ServerName
                } else {
                    core::ptr::null_mut::<_UNICODE_STRING>()
                },
                &raw mut (*EnumContext).ServerHandle,
                (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
                core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
            );
            if !(Status >= 0 as ::core::ffi::c_long) {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/netapi32/user.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                        2785 as ::core::ffi::c_int,
                        b"SamConnect failed (Status %08lx)\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                current_block = 10801318626883441134;
            } else {
                Status = GetAccountDomainSid(
                    if !servername.is_null() {
                        &raw mut ServerName
                    } else {
                        core::ptr::null_mut::<_UNICODE_STRING>()
                    },
                    &raw mut (*EnumContext).AccountDomainSid,
                );
                if !(Status >= 0 as ::core::ffi::c_long) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_1,
                            __dbch_1,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2795 as ::core::ffi::c_int,
                            b"GetAccountDomainSid failed (Status %08lx)\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    current_block = 10801318626883441134;
                } else {
                    Status = SamOpenDomain(
                        (*EnumContext).ServerHandle,
                        (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as ACCESS_MASK,
                        (*EnumContext).AccountDomainSid,
                        &raw mut (*EnumContext).AccountDomainHandle,
                    );
                    if !(Status >= 0 as ::core::ffi::c_long) {
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_2,
                                __dbch_2,
                                b"dll/win32/netapi32/user.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                                2807 as ::core::ffi::c_int,
                                b"SamOpenDomain failed (Status %08lx)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                        current_block = 10801318626883441134;
                    } else {
                        Status = GetBuiltinDomainSid(&raw mut (*EnumContext).BuiltinDomainSid);
                        if !(Status >= 0 as ::core::ffi::c_long) {
                            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                                != 0
                            {
                                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                                ros_dbg_log(
                                    __dbcl_3,
                                    __dbch_3,
                                    b"dll/win32/netapi32/user.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                                    2816 as ::core::ffi::c_int,
                                    b"GetBuiltinDomainSid failed (Status %08lx)\n\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    Status,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                            current_block = 10801318626883441134;
                        } else {
                            DesiredAccess = (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as ACCESS_MASK;
                            if level == 1 as ::core::ffi::c_ulong
                                || level == 2 as ::core::ffi::c_ulong
                                || level == 3 as ::core::ffi::c_ulong
                                || level == 4 as ::core::ffi::c_ulong
                                || level == 11 as ::core::ffi::c_ulong
                            {
                                DesiredAccess |=
                                    DOMAIN_GET_ALIAS_MEMBERSHIP as ::core::ffi::c_ulong;
                            }
                            Status = SamOpenDomain(
                                (*EnumContext).ServerHandle,
                                DesiredAccess,
                                (*EnumContext).BuiltinDomainSid,
                                &raw mut (*EnumContext).BuiltinDomainHandle,
                            );
                            if !(Status >= 0 as ::core::ffi::c_long) {
                                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                                    & (1 as ::core::ffi::c_int)
                                        << __WINE_DBCL_ERR as ::core::ffi::c_int
                                    != 0
                                {
                                    let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                                    ros_dbg_log(
                                        __dbcl_4,
                                        __dbch_4,
                                        b"dll/win32/netapi32/user.c\0"
                                            as *const u8 as *const ::core::ffi::c_char,
                                        b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                                        2832 as ::core::ffi::c_int,
                                        b"SamOpenDomain failed (Status %08lx)\n\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        Status,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                                current_block = 10801318626883441134;
                            } else {
                                current_block = 6528285054092551010;
                            }
                        }
                    }
                }
            }
        }
    }

    match current_block {
        6528285054092551010 => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_5,
                    __dbch_5,
                    b"dll/win32/netapi32/user.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                    2840 as ::core::ffi::c_int,
                    b"EnumContext->Index: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*EnumContext).Index,
                );
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_6: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_6,
                    __dbch_6,
                    b"dll/win32/netapi32/user.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                    2841 as ::core::ffi::c_int,
                    b"EnumContext->Count: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*EnumContext).Count,
                );
            }
            if (*EnumContext).Index >= (*EnumContext).Count {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_7: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_7,
                        __dbch_7,
                        b"dll/win32/netapi32/user.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                        2851 as ::core::ffi::c_int,
                        b"Calling SamEnumerateUsersInDomain\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                Status = SamEnumerateUsersInDomain(
                    (*EnumContext).AccountDomainHandle,
                    &raw mut (*EnumContext).EnumerationContext,
                    0 as ULONG,
                    &raw mut (*EnumContext).Buffer as *mut PVOID,
                    prefmaxlen as ULONG,
                    &raw mut (*EnumContext).Count,
                );
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_8: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_8: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_8,
                        __dbch_8,
                        b"dll/win32/netapi32/user.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                        2859 as ::core::ffi::c_int,
                        b"SamEnumerateUsersInDomain returned (Status %08lx)\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                if !(Status >= 0 as ::core::ffi::c_long) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_9: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_9: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_9,
                            __dbch_9,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2862 as ::core::ffi::c_int,
                            b"SamEnumerateUsersInDomain failed (Status %08lx)\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    current_block = 10801318626883441134;
                } else if Status == STATUS_MORE_ENTRIES {
                    ApiStatus = NERR_BufTooSmall as DWORD;
                    current_block = 10801318626883441134;
                } else {
                    (*EnumContext).BuiltinDone = TRUE as BOOLEAN;
                    current_block = 10887629115603254199;
                }
            } else {
                current_block = 10887629115603254199;
            }
            match current_block {
                10801318626883441134 => {}
                _ => {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_10: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_10: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_10,
                            __dbch_10,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2878 as ::core::ffi::c_int,
                            b"EnumContext: %lu\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            EnumContext,
                        );
                    }
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_11: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_11: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_11,
                            __dbch_11,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2879 as ::core::ffi::c_int,
                            b"EnumContext->Count: %lu\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*EnumContext).Count,
                        );
                    }
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_12: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_12: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_12,
                            __dbch_12,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2880 as ::core::ffi::c_int,
                            b"EnumContext->Buffer: %p\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*EnumContext).Buffer,
                        );
                    }
                    CurrentUser = (*EnumContext).Buffer.offset((*EnumContext).Index as isize)
                        as *mut _SAM_RID_ENUMERATION
                        as PSAM_RID_ENUMERATION;
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_13: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_13: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_13,
                            __dbch_13,
                            b"dll/win32/netapi32/user.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                            2885 as ::core::ffi::c_int,
                            b"RID: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
                            (*CurrentUser).RelativeId,
                        );
                    }
                    DesiredAccess = (READ_CONTROL
                        | USER_READ_GENERAL as u32
                        | USER_READ_PREFERENCES as u32
                        | USER_READ_LOGON as u32
                        | USER_READ_ACCOUNT as u32)
                        as ACCESS_MASK;
                    if level == 1 as ::core::ffi::c_ulong
                        || level == 2 as ::core::ffi::c_ulong
                        || level == 3 as ::core::ffi::c_ulong
                        || level == 4 as ::core::ffi::c_ulong
                        || level == 11 as ::core::ffi::c_ulong
                    {
                        DesiredAccess |= USER_LIST_GROUPS as ::core::ffi::c_ulong;
                    }
                    Status = SamOpenUser(
                        (*EnumContext).AccountDomainHandle,
                        DesiredAccess,
                        (*CurrentUser).RelativeId,
                        &raw mut UserHandle,
                    );
                    if !(Status >= 0 as ::core::ffi::c_long) {
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_14: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_14: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_14,
                                __dbch_14,
                                b"dll/win32/netapi32/user.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                                2897 as ::core::ffi::c_int,
                                b"SamOpenUser failed (Status %08lx)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    } else {
                        ApiStatus = BuildUserInfoBuffer(
                            (*EnumContext).BuiltinDomainHandle,
                            UserHandle,
                            (*EnumContext).AccountDomainSid,
                            (*CurrentUser).RelativeId,
                            level,
                            &raw mut Buffer,
                        );
                        if ApiStatus != NERR_Success as ::core::ffi::c_ulong {
                            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                                != 0
                            {
                                let __dbch_15: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_15: __wine_debug_class = __WINE_DBCL_ERR;
                                ros_dbg_log(
                                    __dbcl_15,
                                    __dbch_15,
                                    b"dll/win32/netapi32/user.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
                                    2910 as ::core::ffi::c_int,
                                    b"BuildUserInfoBuffer failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    ApiStatus,
                                );
                            }
                        } else {
                            SamCloseHandle(UserHandle);
                            UserHandle = NULL as SAM_HANDLE;
                            (*EnumContext).Index = (*EnumContext).Index.wrapping_add(1);
                            *entriesread = (*entriesread).wrapping_add(1);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    if ApiStatus == NERR_Success as ::core::ffi::c_ulong
        && !EnumContext.is_null()
        && (*EnumContext).Index < (*EnumContext).Count
    {
        ApiStatus = ERROR_MORE_DATA as DWORD;
    }

    if !EnumContext.is_null() {
        *totalentries = (*EnumContext).Count as DWORD;
    }

    if resume_handle.is_null() || ApiStatus != ERROR_MORE_DATA as ::core::ffi::c_ulong {
        if !EnumContext.is_null() {
            if !(*EnumContext).BuiltinDomainHandle.is_null() {
                SamCloseHandle((*EnumContext).BuiltinDomainHandle);
            }
            if !(*EnumContext).AccountDomainHandle.is_null() {
                SamCloseHandle((*EnumContext).AccountDomainHandle);
            }
            if !(*EnumContext).BuiltinDomainSid.is_null() {
                RtlFreeHeap(
                    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap as HANDLE,
                    0 as ULONG,
                    (*EnumContext).BuiltinDomainSid as PVOID,
                );
            }
            if !(*EnumContext).AccountDomainSid.is_null() {
                RtlFreeHeap(
                    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap as HANDLE,
                    0 as ULONG,
                    (*EnumContext).AccountDomainSid as PVOID,
                );
            }
            if !(*EnumContext).ServerHandle.is_null() {
                SamCloseHandle((*EnumContext).ServerHandle);
            }
            if !(*EnumContext).Buffer.is_null() {
                i = 0 as ULONG;
                while i < (*EnumContext).Count {
                    SamFreeMemory((*(*EnumContext).Buffer.offset(i as isize)).Name.Buffer as PVOID);
                    i = i.wrapping_add(1);
                }
                SamFreeMemory((*EnumContext).Buffer as PVOID);
            }
            FreeEnumContext(EnumContext);
            EnumContext = core::ptr::null_mut::<_ENUM_CONTEXT>();
        }
    }

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }

    if !resume_handle.is_null() {
        *resume_handle = (if !EnumContext.is_null() {
            (*EnumContext).EnumHandle as ::core::ffi::c_ulong
        } else {
            0 as ::core::ffi::c_ulong
        }) as DWORD;
    }

    *bufptr = Buffer as LPBYTE;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_16: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_16: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_16,
            __dbch_16,
            b"dll/win32/netapi32/user.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetUserEnum\0" as *const u8 as *const ::core::ffi::c_char,
            2971 as ::core::ffi::c_int,
            b"return %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
            ApiStatus,
        );
    }

    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetUserGetGroups(
    servername: LPCWSTR,
    username: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefixmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainSid: PSID = core::ptr::null_mut();
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut GroupMembership: PGROUP_MEMBERSHIP = core::ptr::null_mut();
    let mut GroupCount: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr() as *const i8,
            b"NetUserGetGroups\0".as_ptr() as *const i8,
            3006,
            b"%s %s %d %p %d %p %p stub\n\0".as_ptr() as *const i8,
            debugstr_w(servername),
            debugstr_w(username),
            level,
            bufptr,
            prefixmaxlen,
            entriesread,
            totalentries,
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
    }
    RtlInitUnicodeString(&raw mut UserName, username as PCWSTR);

    Status = SamConnect(
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut(),
    );

    if !(Status >= 0) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/netapi32/user.c\0"
                    .as_ptr() as *const i8,
                b"NetUserGetGroups\0".as_ptr() as *const i8,
                3020,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = GetAccountDomainSid(
            if !servername.is_null() {
                &raw mut ServerName
            } else {
                core::ptr::null_mut()
            },
            &raw mut AccountDomainSid,
        );
        if !(Status >= 0) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/netapi32/user.c\0"
                        .as_ptr() as *const i8,
                    b"NetUserGetGroups\0".as_ptr() as *const i8,
                    3030,
                    b"GetAccountDomainSid failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            Status = SamOpenDomain(
                ServerHandle,
                (DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP) as ACCESS_MASK,
                AccountDomainSid,
                &raw mut AccountDomainHandle,
            );
            if !(Status >= 0) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/netapi32/user.c\0"
                            .as_ptr() as *const i8,
                        b"NetUserGetGroups\0".as_ptr() as *const i8,
                        3042,
                        b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
            } else {
                Status = SamLookupNamesInDomain(
                    AccountDomainHandle,
                    1,
                    &raw mut UserName,
                    &raw mut RelativeIds,
                    &raw mut Use,
                );
                if !(Status >= 0) {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_3,
                            __dbch_3,
                            b"dll/win32/netapi32/user.c\0"
                                .as_ptr() as *const i8,
                            b"NetUserGetGroups\0".as_ptr() as *const i8,
                            3055,
                            b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0"
                                .as_ptr() as *const i8,
                            &raw mut UserName,
                            Status,
                        );
                    }
                    if Status == STATUS_NONE_MAPPED {
                        ApiStatus = NERR_UserNotFound as DWORD;
                    } else {
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                } else if *Use.offset(0) as u32 != SidTypeUser as i32 as u32 {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_4,
                            __dbch_4,
                            b"dll/win32/netapi32/user.c\0"
                                .as_ptr() as *const i8,
                            b"NetUserGetGroups\0".as_ptr() as *const i8,
                            3066,
                            b"Account is not a User!\n\0".as_ptr() as *const i8,
                        );
                    }
                    ApiStatus = NERR_UserNotFound as DWORD;
                } else {
                    Status = SamOpenUser(
                        AccountDomainHandle,
                        USER_LIST_GROUPS as ACCESS_MASK,
                        *RelativeIds.offset(0),
                        &raw mut UserHandle,
                    );
                    if !(Status >= 0) {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                            let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_5,
                                __dbch_5,
                                b"dll/win32/netapi32/user.c\0"
                                    .as_ptr() as *const i8,
                                b"NetUserGetGroups\0".as_ptr() as *const i8,
                                3078,
                                b"SamOpenUser failed (Status %08lx)\n\0".as_ptr() as *const i8,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    } else {
                        Status = SamGetGroupsForUser(
                            UserHandle,
                            &raw mut GroupMembership,
                            &raw mut GroupCount,
                        );
                        if !(Status >= 0) {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_6: __wine_debug_class = __WINE_DBCL_ERR;
                                ros_dbg_log(
                                    __dbcl_6,
                                    __dbch_6,
                                    b"dll/win32/netapi32/user.c\0"
                                        .as_ptr() as *const i8,
                                    b"NetUserGetGroups\0".as_ptr() as *const i8,
                                    3089,
                                    b"SamGetGroupsForUser failed (Status %08lx)\n\0"
                                        .as_ptr() as *const i8,
                                    Status,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else if GroupCount == 0 {
                            ApiStatus = NERR_Success as DWORD;
                        }
                    }
                }
            }
        }
    }

    if !GroupMembership.is_null() {
        SamFreeMemory(GroupMembership as PVOID);
    }
    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }
    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as PVOID);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as PVOID);
    }
    if !AccountDomainSid.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            AccountDomainSid as PVOID,
        );
    }
    if !AccountDomainHandle.is_null() {
        SamCloseHandle(AccountDomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    if ApiStatus != NERR_Success as DWORD && ApiStatus != ERROR_MORE_DATA as DWORD {
        *entriesread = 0;
        *totalentries = 0;
    } else {
        // *entriesread = Count;
        // *totalentries = Count;
    }
    // *bufptr = (LPBYTE)Buffer;

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetUserGetInfo(
    servername: LPCWSTR,
    username: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> NET_API_STATUS {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut BuiltinDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut Buffer: LPVOID = core::ptr::null_mut();
    let mut AccountDomainSid: PSID = core::ptr::null_mut();
    let mut BuiltinDomainSid: PSID = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK;
    let mut ApiStatus: NET_API_STATUS = 0; // NERR_Success
    let mut Status: NTSTATUS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"user.c\0".as_ptr() as *const i8,
            b"NetUserGetInfo\0".as_ptr() as *const i8,
            3168i32,
            b"(%s, %s, %d, %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername),
            debugstr_w(username),
            level,
            bufptr,
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername);
    }
    RtlInitUnicodeString(&mut UserName, username);

    Status = SamConnect(
        if !servername.is_null() { &mut ServerName } else { core::ptr::null_mut() },
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN),
        core::ptr::null_mut(),
    );

    if !(Status >= 0) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"user.c\0".as_ptr() as *const i8,
                b"NetUserGetInfo\0".as_ptr() as *const i8,
                3182i32,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = GetBuiltinDomainSid(&mut BuiltinDomainSid);
        if !(Status >= 0) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"user.c\0".as_ptr() as *const i8,
                    b"NetUserGetInfo\0".as_ptr() as *const i8,
                    3191i32,
                    b"GetBuiltinDomainSid failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            DesiredAccess = DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP;
            if level == 1 || level == 2 || level == 3 || level == 4 || level == 11 {
                DesiredAccess |= DOMAIN_GET_ALIAS_MEMBERSHIP;
            }
            Status = SamOpenDomain(
                ServerHandle,
                DesiredAccess,
                BuiltinDomainSid,
                &mut BuiltinDomainHandle,
            );
            if !(Status >= 0) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"user.c\0".as_ptr() as *const i8,
                        b"NetUserGetInfo\0".as_ptr() as *const i8,
                        3207i32,
                        b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
            } else {
                Status = GetAccountDomainSid(
                    if !servername.is_null() { &mut ServerName } else { core::ptr::null_mut() },
                    &mut AccountDomainSid,
                );
                if !(Status >= 0) {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"user.c\0".as_ptr() as *const i8,
                            b"NetUserGetInfo\0".as_ptr() as *const i8,
                            3217i32,
                            b"GetAccountDomainSid failed (Status %08lx)\n\0".as_ptr() as *const i8,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                } else {
                    Status = SamOpenDomain(
                        ServerHandle,
                        DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP,
                        AccountDomainSid,
                        &mut AccountDomainHandle,
                    );
                    if !(Status >= 0) {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_ERR,
                                __wine_dbch___default,
                                b"user.c\0".as_ptr() as *const i8,
                                b"NetUserGetInfo\0".as_ptr() as *const i8,
                                3229i32,
                                b"SamOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    } else {
                        Status = SamLookupNamesInDomain(
                            AccountDomainHandle,
                            1u32,
                            &mut UserName,
                            &mut RelativeIds,
                            &mut Use,
                        );
                        if !(Status >= 0) {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"user.c\0".as_ptr() as *const i8,
                                    b"NetUserGetInfo\0".as_ptr() as *const i8,
                                    3242i32,
                                    b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0"
                                        .as_ptr() as *const i8,
                                    &mut UserName,
                                    Status,
                                );
                            }
                            if Status == STATUS_NONE_MAPPED {
                                ApiStatus = NERR_UserNotFound;
                            } else {
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                            }
                        } else if *Use.offset(0) != SidTypeUser {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"user.c\0".as_ptr() as *const i8,
                                    b"NetUserGetInfo\0".as_ptr() as *const i8,
                                    3253i32,
                                    b"No user found!\n\0".as_ptr() as *const i8,
                                );
                            }
                            ApiStatus = NERR_UserNotFound;
                        } else {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                                ros_dbg_log(
                                    __WINE_DBCL_TRACE,
                                    __wine_dbch___default,
                                    b"user.c\0".as_ptr() as *const i8,
                                    b"NetUserGetInfo\0".as_ptr() as *const i8,
                                    3258i32,
                                    b"RID: %lu\n\0".as_ptr() as *const i8,
                                    *RelativeIds.offset(0),
                                );
                            }
                            DesiredAccess = READ_CONTROL
                                | USER_READ_GENERAL
                                | USER_READ_PREFERENCES
                                | USER_READ_LOGON
                                | USER_READ_ACCOUNT;
                            if level == 1 || level == 2 || level == 3 || level == 4 || level == 11 {
                                DesiredAccess |= USER_LIST_GROUPS;
                            }
                            Status = SamOpenUser(
                                AccountDomainHandle,
                                DesiredAccess,
                                *RelativeIds.offset(0),
                                &mut UserHandle,
                            );
                            if !(Status >= 0) {
                                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                    ros_dbg_log(
                                        __WINE_DBCL_ERR,
                                        __wine_dbch___default,
                                        b"user.c\0".as_ptr() as *const i8,
                                        b"NetUserGetInfo\0".as_ptr() as *const i8,
                                        3271i32,
                                        b"SamOpenUser failed (Status %08lx)\n\0".as_ptr() as *const i8,
                                        Status,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                            } else {
                                ApiStatus = BuildUserInfoBuffer(
                                    BuiltinDomainHandle,
                                    UserHandle,
                                    AccountDomainSid,
                                    *RelativeIds.offset(0),
                                    level,
                                    &mut Buffer,
                                );
                                if ApiStatus != NERR_Success {
                                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                        ros_dbg_log(
                                            __WINE_DBCL_ERR,
                                            __wine_dbch___default,
                                            b"user.c\0".as_ptr() as *const i8,
                                            b"NetUserGetInfo\0".as_ptr() as *const i8,
                                            3284i32,
                                            b"BuildUserInfoBuffer failed (ApiStatus %08lu)\n\0"
                                                .as_ptr() as *const i8,
                                            ApiStatus,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }
    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as *mut c_void);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as *mut c_void);
    }
    if !AccountDomainHandle.is_null() {
        SamCloseHandle(AccountDomainHandle);
    }
    if !AccountDomainSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid as *mut c_void);
    }
    if !BuiltinDomainHandle.is_null() {
        SamCloseHandle(BuiltinDomainHandle);
    }
    if !BuiltinDomainSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, BuiltinDomainSid as *mut c_void);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    *bufptr = Buffer as LPBYTE;
    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetUserGetLocalGroups(
    servername: LPCWSTR,
    username: LPCWSTR,
    level: DWORD,
    flags: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut BuiltinDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainSid: PSID = core::ptr::null_mut();
    let mut UserSid: PSID = core::ptr::null_mut();
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut BuiltinMemberCount: ULONG = 0u32;
    let mut AccountMemberCount: ULONG = 0u32;
    let mut BuiltinAliases: PULONG = core::ptr::null_mut();
    let mut AccountAliases: PULONG = core::ptr::null_mut();
    let mut BuiltinNames: PUNICODE_STRING = core::ptr::null_mut();
    let mut AccountNames: PUNICODE_STRING = core::ptr::null_mut();
    let mut Buffer: PLOCALGROUP_USERS_INFO_0 = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Count: ULONG = 0u32;
    let mut Index: ULONG = 0;
    let mut StrPtr: LPWSTR = core::ptr::null_mut();
    let mut ApiStatus: DWORD = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if level != 0u32 {
        return ERROR_INVALID_LEVEL as DWORD;
    }

    if (flags as u32) & !(LG_INCLUDE_INDIRECT as u32) != 0 {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    // flags & LG_INCLUDE_INDIRECT is not supported yet, just ignoring

    'done: {
    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername);
    }
    RtlInitUnicodeString(&mut UserName, username);

    Status = SamConnect(
        if !servername.is_null() {
            &mut ServerName
        } else {
            core::ptr::null_mut()
        },
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut(),
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    Status = OpenBuiltinDomain(
        ServerHandle,
        (DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP) as u32,
        &mut BuiltinDomainHandle,
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    Status = GetAccountDomainSid(
        if !servername.is_null() {
            &mut ServerName
        } else {
            core::ptr::null_mut()
        },
        &mut AccountDomainSid,
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    Status = SamOpenDomain(
        ServerHandle,
        (DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP) as u32,
        AccountDomainSid,
        &mut AccountDomainHandle,
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    Status = SamLookupNamesInDomain(
        AccountDomainHandle,
        1u32,
        &mut UserName,
        &mut RelativeIds,
        &mut Use,
    );
    if !(Status >= 0) {
        if Status == STATUS_NONE_MAPPED {
            ApiStatus = NERR_UserNotFound as DWORD;
        } else {
            ApiStatus = NetpNtStatusToApiStatus(Status);
        }
        break 'done;
    }

    if *Use.offset(0isize) as u32 != SidTypeUser as u32 {
        ApiStatus = NERR_UserNotFound as DWORD;
        break 'done;
    }

    ApiStatus = BuildSidFromSidAndRid(
        AccountDomainSid,
        *RelativeIds.offset(0isize),
        &mut UserSid,
    );
    if ApiStatus != 0u32 {
        break 'done;
    }

    Status = SamGetAliasMembership(
        BuiltinDomainHandle,
        1u32,
        &mut UserSid,
        &mut BuiltinMemberCount,
        &mut BuiltinAliases,
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    if BuiltinMemberCount > 0 {
        Status = SamLookupIdsInDomain(
            BuiltinDomainHandle,
            BuiltinMemberCount,
            BuiltinAliases,
            &mut BuiltinNames,
            core::ptr::null_mut(),
        );
        if !(Status >= 0) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }
    }

    Status = SamGetAliasMembership(
        AccountDomainHandle,
        1u32,
        &mut UserSid,
        &mut AccountMemberCount,
        &mut AccountAliases,
    );
    if !(Status >= 0) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'done;
    }

    if AccountMemberCount > 0 {
        Status = SamLookupIdsInDomain(
            AccountDomainHandle,
            AccountMemberCount,
            AccountAliases,
            &mut AccountNames,
            core::ptr::null_mut(),
        );
        if !(Status >= 0) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }
    }

    // Calculate required buffer size
    Size = 0;
    for i in 0..BuiltinMemberCount as usize {
        if (*BuiltinNames.add(i)).Length > 0 {
            Size += (core::mem::size_of::<LOCALGROUP_USERS_INFO_0>()
                + (*BuiltinNames.add(i)).Length as usize
                + core::mem::size_of::<u16>()) as ULONG;
            Count = Count.wrapping_add(1);
        }
    }
    for i in 0..AccountMemberCount as usize {
        if (*AccountNames.add(i)).Length > 0 {
            Size += (core::mem::size_of::<LOCALGROUP_USERS_INFO_0>()
                + (*AccountNames.add(i)).Length as usize
                + core::mem::size_of::<u16>()) as ULONG;
            Count = Count.wrapping_add(1);
        }
    }

    if Size == 0 {
        ApiStatus = 0u32; // NERR_Success
        break 'done;
    }

    ApiStatus = NetApiBufferAllocate(Size, &mut Buffer as *mut _ as *mut *mut core::ffi::c_void);
    if ApiStatus != 0u32 {
        break 'done;
    }

    ZeroMemory(Buffer as *mut core::ffi::c_void, Size as usize);

    // Compute starting string position: after array of LOCALGROUP_USERS_INFO_0
    StrPtr = (Buffer as *mut u8).add(Count as usize * core::mem::size_of::<LOCALGROUP_USERS_INFO_0>())
        as LPWSTR;

    Index = 0;
    for i in 0..BuiltinMemberCount as usize {
        if (*BuiltinNames.add(i)).Length > 0 {
            CopyMemory(
                StrPtr as *mut core::ffi::c_void,
                (*BuiltinNames.add(i)).Buffer as *const core::ffi::c_void,
                (*BuiltinNames.add(i)).Length as usize,
            );
            (*Buffer.add(Index as usize)).lgrui0_name = StrPtr;
            StrPtr = (StrPtr as *mut u8)
                .add((*BuiltinNames.add(i)).Length as usize + core::mem::size_of::<u16>())
                as LPWSTR;
            Index += 1;
        }
    }

    for i in 0..AccountMemberCount as usize {
        if (*AccountNames.add(i)).Length > 0 {
            CopyMemory(
                StrPtr as *mut core::ffi::c_void,
                (*AccountNames.add(i)).Buffer as *const core::ffi::c_void,
                (*AccountNames.add(i)).Length as usize,
            );
            (*Buffer.add(Index as usize)).lgrui0_name = StrPtr;
            StrPtr = (StrPtr as *mut u8)
                .add((*AccountNames.add(i)).Length as usize + core::mem::size_of::<u16>())
                as LPWSTR;
            Index += 1;
        }
    }

    } // end 'done block

    // cleanup (was the `done:` target; ran on every exit via `break 'done`)
    {
        if !AccountNames.is_null() {
            SamFreeMemory(AccountNames as *mut core::ffi::c_void);
        }
        if !BuiltinNames.is_null() {
            SamFreeMemory(BuiltinNames as *mut core::ffi::c_void);
        }
        if !AccountAliases.is_null() {
            SamFreeMemory(AccountAliases as *mut core::ffi::c_void);
        }
        if !BuiltinAliases.is_null() {
            SamFreeMemory(BuiltinAliases as *mut core::ffi::c_void);
        }
        if !RelativeIds.is_null() {
            SamFreeMemory(RelativeIds as *mut core::ffi::c_void);
        }
        if !Use.is_null() {
            SamFreeMemory(Use as *mut core::ffi::c_void);
        }
        if !UserSid.is_null() {
            NetApiBufferFree(UserSid as *mut core::ffi::c_void);
        }
        if !AccountDomainSid.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid as *mut core::ffi::c_void);
        }
        if !AccountDomainHandle.is_null() {
            SamCloseHandle(AccountDomainHandle);
        }
        if !BuiltinDomainHandle.is_null() {
            SamCloseHandle(BuiltinDomainHandle);
        }
        if !ServerHandle.is_null() {
            SamCloseHandle(ServerHandle);
        }

        if ApiStatus != 0u32 && ApiStatus != ERROR_MORE_DATA as DWORD {
            *entriesread = 0;
            *totalentries = 0;
        } else {
            *entriesread = Count as DWORD;
            *totalentries = Count as DWORD;
        }
        *bufptr = Buffer as LPBYTE;
    }

    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetUserModalsGet(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> DWORD {

    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = ptr::null_mut();
    let mut DomainSid: PSID = ptr::null_mut();
    let mut PasswordInfo: PDOMAIN_PASSWORD_INFORMATION = ptr::null_mut();
    let mut LogoffInfo: PDOMAIN_LOGOFF_INFORMATION = ptr::null_mut();
    let mut ServerRoleInfo: PDOMAIN_SERVER_ROLE_INFORMATION = ptr::null_mut();
    let mut ReplicationInfo: PDOMAIN_REPLICATION_INFORMATION = ptr::null_mut();
    let mut NameInfo: PDOMAIN_NAME_INFORMATION = ptr::null_mut();
    let mut LockoutInfo: PDOMAIN_LOCKOUT_INFORMATION = ptr::null_mut();
    let mut DesiredAccess: ULONG = 0;
    let mut BufferSize: ULONG = 0;
    let mut umi0: PUSER_MODALS_INFO_0 = ptr::null_mut();
    let mut umi1: PUSER_MODALS_INFO_1 = ptr::null_mut();
    let mut umi2: PUSER_MODALS_INFO_2 = ptr::null_mut();
    let mut umi3: PUSER_MODALS_INFO_3 = ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    *bufptr = ptr::null_mut::<u8>();

    'cleanup: {
    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername as PCWSTR);
    }

    Status = SamConnect(
        if servername.is_null() {
            ptr::null_mut()
        } else {
            &mut ServerName
        },
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'cleanup;
    }

    Status = GetAccountDomainSid(
        if servername.is_null() {
            ptr::null_mut()
        } else {
            &mut ServerName
        },
        &mut DomainSid,
    );
    if !NT_SUCCESS(Status) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'cleanup;
    }

    match level {
        0 => DesiredAccess = (DOMAIN_READ_OTHER_PARAMETERS | DOMAIN_READ_PASSWORD_PARAMETERS) as ULONG,
        1 => DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS as ULONG,
        2 => DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS as ULONG,
        3 => DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS as ULONG,
        _ => {
            ApiStatus = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }
    }

    Status = SamOpenDomain(ServerHandle, DesiredAccess as ACCESS_MASK, DomainSid, &mut DomainHandle);
    if !NT_SUCCESS(Status) {
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'cleanup;
    }

    match level {
        0 => {
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainPasswordInformation,
                &mut PasswordInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainLogoffInformation,
                &mut LogoffInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            BufferSize = size_of::<USER_MODALS_INFO_0>() as ULONG;
        }
        1 => {
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainServerRoleInformation,
                &mut ServerRoleInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainReplicationInformation,
                &mut ReplicationInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            BufferSize = (size_of::<USER_MODALS_INFO_1>()
                + (*ReplicationInfo).ReplicaSourceNodeName.Length as usize
                + size_of::<WCHAR>()) as ULONG;
        }
        2 => {
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainNameInformation,
                &mut NameInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            BufferSize = (size_of::<USER_MODALS_INFO_2>()
                + (*NameInfo).DomainName.Length as usize
                + size_of::<WCHAR>()
                + RtlLengthSid(DomainSid) as usize) as ULONG;
        }
        3 => {
            Status = SamQueryInformationDomain(
                DomainHandle,
                DomainLockoutInformation,
                &mut LockoutInfo as *mut _ as *mut PVOID,
            );
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'cleanup;
            }
            BufferSize = size_of::<USER_MODALS_INFO_3>() as ULONG;
        }
        _ => unreachable!(),
    }

    ApiStatus = NetApiBufferAllocate(BufferSize, bufptr as *mut PVOID);
    if ApiStatus != NERR_Success {
        break 'cleanup;
    }

    match level {
        0 => {
            umi0 = *bufptr as PUSER_MODALS_INFO_0;
            (*umi0).usrmod0_min_passwd_len = (*PasswordInfo).MinPasswordLength as DWORD;
            (*umi0).usrmod0_max_passwd_age =
                (-(*PasswordInfo).MaxPasswordAge.QuadPart / 10_000_000) as ULONG as DWORD;
            (*umi0).usrmod0_min_passwd_age = DeltaTimeToSeconds((*PasswordInfo).MinPasswordAge);
            (*umi0).usrmod0_force_logoff = DeltaTimeToSeconds((*LogoffInfo).ForceLogoff);
            (*umi0).usrmod0_password_hist_len = (*PasswordInfo).PasswordHistoryLength as DWORD;
        }
        1 => {
            umi1 = *bufptr as PUSER_MODALS_INFO_1;
            match (*ServerRoleInfo).DomainServerRole {
                3 => (*umi1).usrmod1_role = UAS_ROLE_PRIMARY as DWORD,
                2 => (*umi1).usrmod1_role = UAS_ROLE_BACKUP as DWORD,
                _ => {
                    ApiStatus = NERR_InternalError;
                    break 'cleanup;
                }
            }
            let dst = (*bufptr).add(size_of::<USER_MODALS_INFO_1>()) as LPWSTR;
            (*umi1).usrmod1_primary = dst;
            let len = (*ReplicationInfo).ReplicaSourceNodeName.Length as usize;
            RtlCopyMemory(dst as *mut _, (*ReplicationInfo).ReplicaSourceNodeName.Buffer as *const _, len);
            *dst.add(len / size_of::<WCHAR>()) = 0u16;
        }
        2 => {
            umi2 = *bufptr as PUSER_MODALS_INFO_2;
            let dst = (*bufptr).add(size_of::<USER_MODALS_INFO_2>()) as LPWSTR;
            (*umi2).usrmod2_domain_name = dst;
            let len = (*NameInfo).DomainName.Length as usize;
            RtlCopyMemory(dst as *mut _, (*NameInfo).DomainName.Buffer as *const _, len);
            *dst.add(len / size_of::<WCHAR>()) = 0u16;
            let sid_dst = (*bufptr)
                .add(size_of::<USER_MODALS_INFO_2>())
                .add(len)
                .add(size_of::<WCHAR>()) as PSID;
            (*umi2).usrmod2_domain_id = sid_dst;
            RtlCopyMemory(sid_dst as *mut _, DomainSid as *const _, RtlLengthSid(DomainSid) as usize);
        }
        3 => {
            umi3 = *bufptr as PUSER_MODALS_INFO_3;
            (*umi3).usrmod3_lockout_duration = DeltaTimeToSeconds((*LockoutInfo).LockoutDuration);
            (*umi3).usrmod3_lockout_observation_window =
                DeltaTimeToSeconds((*LockoutInfo).LockoutObservationWindow);
            (*umi3).usrmod3_lockout_threshold = (*LockoutInfo).LockoutThreshold as DWORD;
        }
        _ => unreachable!(),
    }
    } // end 'cleanup block

    if !LockoutInfo.is_null() {
        SamFreeMemory(LockoutInfo as PVOID);
    }
    if !NameInfo.is_null() {
        SamFreeMemory(NameInfo as PVOID);
    }
    if !ReplicationInfo.is_null() {
        SamFreeMemory(ReplicationInfo as PVOID);
    }
    if !ServerRoleInfo.is_null() {
        SamFreeMemory(ServerRoleInfo as PVOID);
    }
    if !LogoffInfo.is_null() {
        SamFreeMemory(LogoffInfo as PVOID);
    }
    if !PasswordInfo.is_null() {
        SamFreeMemory(PasswordInfo as PVOID);
    }
    if !DomainSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid as PVOID);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetUserModalsSet(
    servername: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr() as *const i8,
            b"NetUserModalsSet\0".as_ptr() as *const i8,
            3948i32,
            b"(%s %d %p %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const u16),
            level,
            buf,
            parm_err,
        );
    }
    return ERROR_ACCESS_DENIED as u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetUserSetGroups(
    servername: *const u16,
    username: *const u16,
    level: u32,
    buf: *mut u8,
    num_entries: u32,
) -> u32 {
    if (*(__wine_dbch___default)).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            dbcl,
            dbch,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr() as *const i8,
            b"NetUserSetGroups\0".as_ptr() as *const i8,
            3965i32,
            b"(%s %s %lu %p %lu)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(username as *const u16),
            level,
            buf as *mut core::ffi::c_void,
            num_entries,
        );
    }
    ERROR_ACCESS_DENIED
}

#[no_mangle]
pub unsafe extern "system" fn NetUserSetInfo(
    servername: LPCWSTR,
    username: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AccountDomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut UserHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = 0; // NERR_Success
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // trace debug log
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/user.c\0"
                .as_ptr() as *const i8,
            b"NetUserSetInfo\0".as_ptr() as *const i8,
            3990,
            b"(%s %s %lu %p %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername),
            debugstr_w(username),
            level,
            buf,
            parm_err,
        );
    }

    if !parm_err.is_null() {
        *parm_err = 0; // PARM_ERROR_NONE
    }

    match level {
        1003 => {}
        1009 => {}
        0 | 1 | 2 | 3 | 4 | 22 | 1006 | 1007 | 1008 | 1011 | 1012 | 1013 | 1014 | 1017 | 1018
        | 1020 | 1024 | 1025 | 1051 | 1052 | 1053 => {}
        _ => return ERROR_INVALID_LEVEL as DWORD,
    }

    'cleanup: {
    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as *const u16);
    }
    RtlInitUnicodeString(&raw mut UserName, username as *const u16);

    Status = SamConnect(
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut(),
    );

    if Status < 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/user.c\0"
                    .as_ptr() as *const i8,
                b"NetUserSetInfo\0".as_ptr() as *const i8,
                4042,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'cleanup;
    }

    Status = OpenAccountDomain(
        ServerHandle,
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut()
        },
        (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP | DOMAIN_READ_PASSWORD_PARAMETERS) as ULONG,
        &raw mut AccountDomainHandle,
    );

    if Status < 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/user.c\0"
                    .as_ptr() as *const i8,
                b"NetUserSetInfo\0".as_ptr() as *const i8,
                4054,
                b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
        break 'cleanup;
    }

    ApiStatus = OpenUserByName(
        AccountDomainHandle,
        &raw mut UserName,
        USER_ALL_ACCESS as ULONG,
        &raw mut UserHandle,
    );

    if ApiStatus != 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/user.c\0"
                    .as_ptr() as *const i8,
                b"NetUserSetInfo\0".as_ptr() as *const i8,
                4066,
                b"OpenUserByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                &raw mut UserName,
                ApiStatus,
            );
        }
        break 'cleanup;
    }

    ApiStatus = SetUserInfo(UserHandle, buf, level, parm_err);

    if ApiStatus != 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/user.c\0"
                    .as_ptr() as *const i8,
                b"NetUserSetInfo\0".as_ptr() as *const i8,
                4077,
                b"SetUserInfo failed (Status %lu)\n\0".as_ptr() as *const i8,
                ApiStatus,
            );
        }
    }

    } // end 'cleanup block

    if !UserHandle.is_null() {
        SamCloseHandle(UserHandle);
    }
    if !AccountDomainHandle.is_null() {
        SamCloseHandle(AccountDomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    ApiStatus
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2002 Andriy Palamarchuk
//  *
//  * netapi32 user functions
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
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// /*
//  *  TODO:
//  *    Implement NetUserGetGroups (WIP)
//  *    Implement NetUserSetGroups
//  *    NetUserGetLocalGroups does not support LG_INCLUDE_INDIRECT yet.
//  *    Add missing information levels.
//  *    ...
//  */
//
// #include "netapi32.h"
//
// #include <ndk/kefuncs.h>
// #include <ndk/obfuncs.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// typedef struct _ENUM_CONTEXT
// {
//     LIST_ENTRY ListLink;
//     ULONG EnumHandle;
//
//     SAM_HANDLE ServerHandle;
//     SAM_HANDLE BuiltinDomainHandle;
//     SAM_HANDLE AccountDomainHandle;
//     PSID BuiltinDomainSid;
//     PSID AccountDomainSid;
//
//     SAM_ENUMERATE_HANDLE EnumerationContext;
//     PSAM_RID_ENUMERATION Buffer;
//     ULONG Count;
//     ULONG Index;
//     BOOLEAN BuiltinDone;
//
// } ENUM_CONTEXT, *PENUM_CONTEXT;
//
// LIST_ENTRY g_EnumContextListHead;
// CRITICAL_SECTION g_EnumContextListLock;
// LONG g_EnumContextHandle = 0;
//
// static
// ULONG
// DeltaTimeToSeconds(LARGE_INTEGER DeltaTime)
// {
//     LARGE_INTEGER Seconds;
//
//     if (DeltaTime.QuadPart == 0)
//         return 0;
//
//     Seconds.QuadPart = -DeltaTime.QuadPart / 10000000;
//
//     if (Seconds.HighPart != 0)
//         return TIMEQ_FOREVER;
//
//     return Seconds.LowPart;
// }
//
//
// static
// NTSTATUS
// GetAllowedWorldAce( PACL Acl,
//                     PACCESS_ALLOWED_ACE *Ace)
// {
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     ULONG WorldSid[sizeof(SID) / sizeof(ULONG) + SID_MAX_SUB_AUTHORITIES];
//     ACL_SIZE_INFORMATION AclSize;
//     PVOID LocalAce = NULL;
//     ULONG i;
//     NTSTATUS Status;
//
//     *Ace = NULL;
//
//     RtlInitializeSid((PSID)WorldSid,
//                      &WorldAuthority,
//                      1);
//     *(RtlSubAuthoritySid((PSID)WorldSid, 0)) = SECURITY_WORLD_RID;
//
//     Status = RtlQueryInformationAcl(Acl,
//                                     &AclSize,
//                                     sizeof(AclSize),
//                                     AclSizeInformation);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     for (i = 0; i < AclSize.AceCount; i++)
//     {
//         Status = RtlGetAce(Acl, i, &LocalAce);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         if (((PACE_HEADER)LocalAce)->AceType != ACCESS_ALLOWED_ACE_TYPE)
//             continue;
//
//         if (RtlEqualSid((PSID)WorldSid,
//                         (PSID)&((PACCESS_ALLOWED_ACE)LocalAce)->SidStart))
//         {
//             *Ace = (PACCESS_ALLOWED_ACE)LocalAce;
//             return STATUS_SUCCESS;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// ULONG
// GetAccountFlags(ULONG AccountControl,
//                 PACL Dacl)
// {
//     PACCESS_ALLOWED_ACE Ace = NULL;
//     ULONG Flags = UF_SCRIPT;
//     NTSTATUS Status;
//
//     if (Dacl != NULL)
//     {
//         Status = GetAllowedWorldAce(Dacl, &Ace);
//         if (NT_SUCCESS(Status))
//         {
//             if (Ace == NULL)
//             {
//                 Flags |= UF_PASSWD_CANT_CHANGE;
//             }
//             else if ((Ace->Mask & USER_CHANGE_PASSWORD) == 0)
//             {
//                 Flags |= UF_PASSWD_CANT_CHANGE;
//             }
//         }
//     }
//
//     if (AccountControl & USER_ACCOUNT_DISABLED)
//         Flags |= UF_ACCOUNTDISABLE;
//
//     if (AccountControl & USER_HOME_DIRECTORY_REQUIRED)
//         Flags |= UF_HOMEDIR_REQUIRED;
//
//     if (AccountControl & USER_PASSWORD_NOT_REQUIRED)
//         Flags |= UF_PASSWD_NOTREQD;
//
//     if (AccountControl & USER_ACCOUNT_AUTO_LOCKED)
//         Flags |= UF_LOCKOUT;
//
//     if (AccountControl & USER_DONT_EXPIRE_PASSWORD)
//         Flags |= UF_DONT_EXPIRE_PASSWD;
//
// /*
//     if (AccountControl & USER_ENCRYPTED_TEXT_PASSWORD_ALLOWED)
//         Flags |= UF_ENCRYPTED_TEXT_PASSWORD_ALLOWED;
//
//     if (AccountControl & USER_SMARTCARD_REQUIRED)
//         Flags |= UF_SMARTCARD_REQUIRED;
//
//     if (AccountControl & USER_TRUSTED_FOR_DELEGATION)
//         Flags |= UF_TRUSTED_FOR_DELEGATION;
//
//     if (AccountControl & USER_NOT_DELEGATED)
//         Flags |= UF_NOT_DELEGATED;
//
//     if (AccountControl & USER_USE_DES_KEY_ONLY)
//         Flags |= UF_USE_DES_KEY_ONLY;
//
//     if (AccountControl & USER_DONT_REQUIRE_PREAUTH)
//         Flags |= UF_DONT_REQUIRE_PREAUTH;
//
//     if (AccountControl & USER_PASSWORD_EXPIRED)
//         Flags |= UF_PASSWORD_EXPIRED;
// */
//
//     /* Set account type flags */
//     if (AccountControl & USER_TEMP_DUPLICATE_ACCOUNT)
//         Flags |= UF_TEMP_DUPLICATE_ACCOUNT;
//     else if (AccountControl & USER_NORMAL_ACCOUNT)
//         Flags |= UF_NORMAL_ACCOUNT;
//     else if (AccountControl & USER_INTERDOMAIN_TRUST_ACCOUNT)
//         Flags |= UF_INTERDOMAIN_TRUST_ACCOUNT;
//     else if (AccountControl & USER_WORKSTATION_TRUST_ACCOUNT)
//         Flags |= UF_WORKSTATION_TRUST_ACCOUNT;
//     else if (AccountControl & USER_SERVER_TRUST_ACCOUNT)
//         Flags |= UF_SERVER_TRUST_ACCOUNT;
//
//     return Flags;
// }
//
//
// static
// ULONG
// GetAccountControl(ULONG Flags)
// {
//     ULONG AccountControl = 0;
//
//     if (Flags & UF_ACCOUNTDISABLE)
//         AccountControl |= USER_ACCOUNT_DISABLED;
//
//     if (Flags & UF_HOMEDIR_REQUIRED)
//         AccountControl |= USER_HOME_DIRECTORY_REQUIRED;
//
//     if (Flags & UF_PASSWD_NOTREQD)
//         AccountControl |= USER_PASSWORD_NOT_REQUIRED;
//
//     if (Flags & UF_LOCKOUT)
//         AccountControl |= USER_ACCOUNT_AUTO_LOCKED;
//
//     if (Flags & UF_DONT_EXPIRE_PASSWD)
//         AccountControl |= USER_DONT_EXPIRE_PASSWORD;
//
//     /* Set account type flags */
//     if (Flags & UF_TEMP_DUPLICATE_ACCOUNT)
//         AccountControl |= USER_TEMP_DUPLICATE_ACCOUNT;
//     else if (Flags & UF_NORMAL_ACCOUNT)
//         AccountControl |= USER_NORMAL_ACCOUNT;
//     else if (Flags & UF_INTERDOMAIN_TRUST_ACCOUNT)
//         AccountControl |= USER_INTERDOMAIN_TRUST_ACCOUNT;
//     else if (Flags & UF_WORKSTATION_TRUST_ACCOUNT)
//         AccountControl |= USER_WORKSTATION_TRUST_ACCOUNT;
//     else if (Flags & UF_SERVER_TRUST_ACCOUNT)
//         AccountControl |= USER_SERVER_TRUST_ACCOUNT;
//
//     return AccountControl;
// }
//
//
// static
// DWORD
// GetPasswordAge( PLARGE_INTEGER PasswordLastSet)
// {
//     LARGE_INTEGER SystemTime;
//     ULONG SystemSecondsSince1970;
//     ULONG PasswordSecondsSince1970;
//     NTSTATUS Status;
//
//     Status = NtQuerySystemTime(&SystemTime);
//     if (!NT_SUCCESS(Status))
//         return 0;
//
//     RtlTimeToSecondsSince1970(&SystemTime, &SystemSecondsSince1970);
//     RtlTimeToSecondsSince1970(PasswordLastSet, &PasswordSecondsSince1970);
//
//     return SystemSecondsSince1970 - PasswordSecondsSince1970;
// }
//
//
// static
// VOID
// ChangeUserDacl( PACL Dacl,
//                 ULONG Flags)
// {
//     PACCESS_ALLOWED_ACE Ace = NULL;
//     NTSTATUS Status;
//
//     if (Dacl == NULL)
//         return;
//
//     Status = GetAllowedWorldAce(Dacl, &Ace);
//     if (!NT_SUCCESS(Status))
//         return;
//
//     if (Flags & UF_PASSWD_CANT_CHANGE)
//         Ace->Mask &= ~USER_CHANGE_PASSWORD;
//     else
//         Ace->Mask |= USER_CHANGE_PASSWORD;
// }
//
//
// static
// NET_API_STATUS
// GetUserDacl( SAM_HANDLE UserHandle,
//              PACL *Dacl)
// {
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     PACL SamDacl;
//     PACL LocalDacl;
//     BOOLEAN Defaulted;
//     BOOLEAN Present;
//     ACL_SIZE_INFORMATION AclSize;
//     NET_API_STATUS ApiStatus;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n", UserHandle, Dacl);
//
//     *Dacl = NULL;
//
//     Status = SamQuerySecurityObject(UserHandle,
//                                     DACL_SECURITY_INFORMATION,
//                                     &SecurityDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamQuerySecurityObject() failed (Status 0x%08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     Status = RtlGetDaclSecurityDescriptor(SecurityDescriptor,
//                                           &Present,
//                                           &SamDacl,
//                                           &Defaulted);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("RtlGetDaclSecurityDescriptor() failed (Status 0x%08lx)\n", Status);
//         ApiStatus = NERR_InternalError;
//         goto done;
//     }
//
//     if (Present == FALSE)
//     {
//         TRACE("No DACL present\n");
//         ApiStatus = NERR_Success;
//         goto done;
//     }
//
//     Status = RtlQueryInformationAcl(SamDacl,
//                                     &AclSize,
//                                     sizeof(AclSize),
//                                     AclSizeInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("RtlQueryInformationAcl() failed (Status 0x%08lx)\n", Status);
//         ApiStatus = NERR_InternalError;
//         goto done;
//     }
//
//     LocalDacl = HeapAlloc(GetProcessHeap(), 0, AclSize.AclBytesInUse);
//     if (LocalDacl == NULL)
//     {
//         TRACE("Memory allocation failed\n");
//         ApiStatus = ERROR_NOT_ENOUGH_MEMORY;
//         goto done;
//     }
//
//     RtlCopyMemory(LocalDacl, SamDacl, AclSize.AclBytesInUse);
//
//     *Dacl = LocalDacl;
//
//     ApiStatus = NERR_Success;
//
// done:
//     if (SecurityDescriptor != NULL)
//         SamFreeMemory(SecurityDescriptor);
//
//     TRACE("done (ApiStatus: 0x%08lx)\n", ApiStatus);
//
//     return ApiStatus;
// }
//
//
// static
// VOID
// FreeUserInfo(PUSER_ALL_INFORMATION UserInfo)
// {
//     if (UserInfo->UserName.Buffer != NULL)
//         SamFreeMemory(UserInfo->UserName.Buffer);
//
//     if (UserInfo->FullName.Buffer != NULL)
//         SamFreeMemory(UserInfo->FullName.Buffer);
//
//     if (UserInfo->HomeDirectory.Buffer != NULL)
//         SamFreeMemory(UserInfo->HomeDirectory.Buffer);
//
//     if (UserInfo->HomeDirectoryDrive.Buffer != NULL)
//         SamFreeMemory(UserInfo->HomeDirectoryDrive.Buffer);
//
//     if (UserInfo->ScriptPath.Buffer != NULL)
//         SamFreeMemory(UserInfo->ScriptPath.Buffer);
//
//     if (UserInfo->ProfilePath.Buffer != NULL)
//         SamFreeMemory(UserInfo->ProfilePath.Buffer);
//
//     if (UserInfo->AdminComment.Buffer != NULL)
//         SamFreeMemory(UserInfo->AdminComment.Buffer);
//
//     if (UserInfo->WorkStations.Buffer != NULL)
//         SamFreeMemory(UserInfo->WorkStations.Buffer);
//
//     if (UserInfo->UserComment.Buffer != NULL)
//         SamFreeMemory(UserInfo->UserComment.Buffer);
//
//     if (UserInfo->Parameters.Buffer != NULL)
//         SamFreeMemory(UserInfo->Parameters.Buffer);
//
//     if (UserInfo->PrivateData.Buffer != NULL)
//         SamFreeMemory(UserInfo->PrivateData.Buffer);
//
//     if (UserInfo->LogonHours.LogonHours != NULL)
//         SamFreeMemory(UserInfo->LogonHours.LogonHours);
//
//     SamFreeMemory(UserInfo);
// }
//
//
// static
// NET_API_STATUS
// GetUserPrivileges(
//      SAM_HANDLE BuiltinDomainHandle,
//      SAM_HANDLE UserHandle,
//      PSID AccountDomainSid,
//      ULONG RelativeId,
//      PDWORD Priv,
//      PDWORD AuthFlags)
// {
//     PGROUP_MEMBERSHIP GroupMembership = NULL;
//     ULONG GroupCount, SidCount, AliasCount, i;
//     PSID *SidArray = NULL;
//     PULONG AliasArray = NULL;
//     BOOL bAdmin = FALSE, bUser = FALSE;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status;
//
//     FIXME("GetUserPrivileges(%p)\n", UserHandle);
//
//     /* Get the users group memberships */
//     Status = SamGetGroupsForUser(UserHandle,
//                                  &GroupMembership,
//                                  &GroupCount);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetGroupsForUser() failed (Status 0x%08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Allocate the SID array */
//     ApiStatus = NetApiBufferAllocate((GroupCount + 1) * sizeof(PSID),
//                                      (PVOID*)&SidArray);
//     if (ApiStatus != NERR_Success)
//     {
//         goto done;
//     }
//
//     /* Add the user to the SID array */
//     SidCount = 0;
//     ApiStatus = BuildSidFromSidAndRid(AccountDomainSid,
//                                       RelativeId,
//                                       &SidArray[0]);
//     if (ApiStatus != NERR_Success)
//     {
//         goto done;
//     }
//
//     SidCount++;
//
//     /* Add the groups to the SID array */
//     for (i = 0; i < GroupCount; i++)
//     {
//         ApiStatus = BuildSidFromSidAndRid(AccountDomainSid,
//                                           GroupMembership[i].RelativeId,
//                                           &SidArray[i + 1]);
//         if (ApiStatus != NERR_Success)
//         {
//             goto done;
//         }
//
//         SidCount++;
//     }
//
//     /* Get aliases for the user and his groups */
//     Status = SamGetAliasMembership(BuiltinDomainHandle,
//                                    SidCount,
//                                    SidArray,
//                                    &AliasCount,
//                                    &AliasArray);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetAliasMembership() failed (Status 0x%08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     *AuthFlags = 0;
//
//     /* Set the AuthFlags */
//     for (i = 0; i < AliasCount; i++)
//     {
//         switch (AliasArray[i])
//         {
//             case DOMAIN_ALIAS_RID_ADMINS:
//                 bAdmin = TRUE;
//                 break;
//
//             case DOMAIN_ALIAS_RID_USERS:
//                 bUser = TRUE;
//                 break;
//
//             case DOMAIN_ALIAS_RID_ACCOUNT_OPS:
//                 *AuthFlags |= AF_OP_ACCOUNTS;
//                 break;
//
//             case DOMAIN_ALIAS_RID_SYSTEM_OPS:
//                 *AuthFlags |= AF_OP_SERVER;
//                 break;
//
//             case DOMAIN_ALIAS_RID_PRINT_OPS:
//                 *AuthFlags |= AF_OP_PRINT;
//                 break;
//         }
//     }
//
//     /* Set the prvileges */
//     if (bAdmin)
//     {
//         *Priv = USER_PRIV_ADMIN;
//     }
//     else if (bUser)
//     {
//         *Priv = USER_PRIV_USER;
//     }
//     else
//     {
//         *Priv = USER_PRIV_GUEST;
//     }
//
// done:
//     if (AliasArray != NULL)
//         SamFreeMemory(AliasArray);
//
//     if (SidArray != NULL)
//     {
//         for (i = 0; i < SidCount; i++)
//             NetApiBufferFree(SidArray[i]);
//
//         NetApiBufferFree(SidArray);
//     }
//
//     if (GroupMembership != NULL)
//         SamFreeMemory(GroupMembership);
//
//     return ApiStatus;
// }
//
//
// static
// NET_API_STATUS
// BuildUserInfoBuffer(
//      SAM_HANDLE BuiltinDomainHandle,
//      SAM_HANDLE UserHandle,
//      PSID AccountDomainSid,
//      ULONG RelativeId,
//      DWORD level,
//      LPVOID *Buffer)
// {
//     UNICODE_STRING LogonServer = RTL_CONSTANT_STRING(L"\\\\*");
//     PUSER_ALL_INFORMATION UserInfo = NULL;
//     LPVOID LocalBuffer = NULL;
//     PACL Dacl = NULL;
//     DWORD Priv = 0, AuthFlags = 0;
//     PUSER_INFO_0 UserInfo0;
//     PUSER_INFO_1 UserInfo1;
//     PUSER_INFO_2 UserInfo2;
//     PUSER_INFO_3 UserInfo3;
//     PUSER_INFO_4 UserInfo4;
//     PUSER_INFO_10 UserInfo10;
//     PUSER_INFO_11 UserInfo11;
//     PUSER_INFO_20 UserInfo20;
//     PUSER_INFO_23 UserInfo23;
//     LPWSTR Ptr;
//     ULONG Size = 0;
//     NTSTATUS Status;
//     NET_API_STATUS ApiStatus = NERR_Success;
//
//     *Buffer = NULL;
//
//     Status = SamQueryInformationUser(UserHandle,
//                                      UserAllInformation,
//                                      (PVOID *)&UserInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamQueryInformationUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     if ((level == 1) || (level == 2) || (level == 3) ||
//         (level == 4) || (level == 20) || (level == 23))
//     {
//         ApiStatus = GetUserDacl(UserHandle, &Dacl);
//         if (ApiStatus != NERR_Success)
//             goto done;
//     }
//
//     if ((level == 1) || (level == 2) || (level == 3) ||
//         (level == 4) || (level == 11))
//     {
//         ApiStatus = GetUserPrivileges(BuiltinDomainHandle,
//                                       UserHandle,
//                                       AccountDomainSid,
//                                       RelativeId,
//                                       &Priv,
//                                       &AuthFlags);
//         if (ApiStatus != NERR_Success)
//             goto done;
//     }
//
//     switch (level)
//     {
//         case 0:
//             Size = sizeof(USER_INFO_0) +
//                    UserInfo->UserName.Length + sizeof(WCHAR);
//             break;
//
//         case 1:
//             Size = sizeof(USER_INFO_1) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->ScriptPath.Length + sizeof(WCHAR);
//             break;
//
//         case 2:
//             Size = sizeof(USER_INFO_2) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->ScriptPath.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->UserComment.Length + sizeof(WCHAR) +
//                    UserInfo->Parameters.Length + sizeof(WCHAR) +
//                    UserInfo->WorkStations.Length + sizeof(WCHAR) +
//                    LogonServer.Length + sizeof(WCHAR);
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//                 Size += (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8;
//             break;
//
//         case 3:
//             Size = sizeof(USER_INFO_3) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->ScriptPath.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->UserComment.Length + sizeof(WCHAR) +
//                    UserInfo->Parameters.Length + sizeof(WCHAR) +
//                    UserInfo->WorkStations.Length + sizeof(WCHAR) +
//                    LogonServer.Length + sizeof(WCHAR) +
//                    UserInfo->ProfilePath.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectoryDrive.Length + sizeof(WCHAR);
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//                 Size += (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8;
//             break;
//
//         case 4:
//             Size = sizeof(USER_INFO_4) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->ScriptPath.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->UserComment.Length + sizeof(WCHAR) +
//                    UserInfo->Parameters.Length + sizeof(WCHAR) +
//                    UserInfo->WorkStations.Length + sizeof(WCHAR) +
//                    LogonServer.Length + sizeof(WCHAR) +
//                    UserInfo->ProfilePath.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectoryDrive.Length + sizeof(WCHAR);
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//                 Size += (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8;
//
//             Size += RtlLengthSid(AccountDomainSid) + sizeof(ULONG);
//             break;
//
//         case 10:
//             Size = sizeof(USER_INFO_10) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->UserComment.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR);
//             break;
//
//         case 11:
//             Size = sizeof(USER_INFO_11) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR) +
//                    UserInfo->UserComment.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->HomeDirectory.Length + sizeof(WCHAR) +
//                    UserInfo->Parameters.Length + sizeof(WCHAR) +
//                    LogonServer.Length + sizeof(WCHAR) +
//                    UserInfo->WorkStations.Length + sizeof(WCHAR);
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//                 Size += (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8;
//             break;
//
//         case 20:
//             Size = sizeof(USER_INFO_20) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR);
//             break;
//
//         case 23:
//             Size = sizeof(USER_INFO_23) +
//                    UserInfo->UserName.Length + sizeof(WCHAR) +
//                    UserInfo->FullName.Length + sizeof(WCHAR) +
//                    UserInfo->AdminComment.Length + sizeof(WCHAR);
//
//             Size += RtlLengthSid(AccountDomainSid) + sizeof(ULONG);
//             break;
//
//         default:
//             ApiStatus = ERROR_INVALID_LEVEL;
//             goto done;
//     }
//
//     ApiStatus = NetApiBufferAllocate(Size, &LocalBuffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     ZeroMemory(LocalBuffer, Size);
//
//     switch (level)
//     {
//         case 0:
//             UserInfo0 = (PUSER_INFO_0)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo0 + sizeof(USER_INFO_0));
//
//             UserInfo0->usri0_name = Ptr;
//
//             memcpy(UserInfo0->usri0_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo0->usri0_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 1:
//             UserInfo1 = (PUSER_INFO_1)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo1 + sizeof(USER_INFO_1));
//
//             UserInfo1->usri1_name = Ptr;
//
//             memcpy(UserInfo1->usri1_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo1->usri1_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo1->usri1_password = NULL;
//             UserInfo1->usri1_password_age = GetPasswordAge(&UserInfo->PasswordLastSet);
//
//             UserInfo1->usri1_priv = Priv;
//
//             UserInfo1->usri1_home_dir = Ptr;
//             memcpy(UserInfo1->usri1_home_dir,
//                    UserInfo->HomeDirectory.Buffer,
//                    UserInfo->HomeDirectory.Length);
//             UserInfo1->usri1_home_dir[UserInfo->HomeDirectory.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectory.Length + sizeof(WCHAR));
//
//             UserInfo1->usri1_comment = Ptr;
//             memcpy(UserInfo1->usri1_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo1->usri1_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo1->usri1_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                      Dacl);
//
//             UserInfo1->usri1_script_path = Ptr;
//             memcpy(UserInfo1->usri1_script_path,
//                    UserInfo->ScriptPath.Buffer,
//                    UserInfo->ScriptPath.Length);
//             UserInfo1->usri1_script_path[UserInfo->ScriptPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 2:
//             UserInfo2 = (PUSER_INFO_2)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo2 + sizeof(USER_INFO_2));
//
//             UserInfo2->usri2_name = Ptr;
//
//             memcpy(UserInfo2->usri2_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo2->usri2_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_password = NULL;
//             UserInfo2->usri2_password_age = GetPasswordAge(&UserInfo->PasswordLastSet);
//
//             UserInfo2->usri2_priv = Priv;
//
//             UserInfo2->usri2_home_dir = Ptr;
//             memcpy(UserInfo2->usri2_home_dir,
//                    UserInfo->HomeDirectory.Buffer,
//                    UserInfo->HomeDirectory.Length);
//             UserInfo2->usri2_home_dir[UserInfo->HomeDirectory.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectory.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_comment = Ptr;
//             memcpy(UserInfo2->usri2_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo2->usri2_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                      Dacl);
//
//             UserInfo2->usri2_script_path = Ptr;
//             memcpy(UserInfo2->usri2_script_path,
//                    UserInfo->ScriptPath.Buffer,
//                    UserInfo->ScriptPath.Length);
//             UserInfo2->usri2_script_path[UserInfo->ScriptPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->ScriptPath.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_auth_flags = AuthFlags;
//
//             UserInfo2->usri2_full_name = Ptr;
//             memcpy(UserInfo2->usri2_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo2->usri2_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_usr_comment = Ptr;
//             memcpy(UserInfo2->usri2_usr_comment,
//                    UserInfo->UserComment.Buffer,
//                    UserInfo->UserComment.Length);
//             UserInfo2->usri2_usr_comment[UserInfo->UserComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserComment.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_parms = Ptr;
//             memcpy(UserInfo2->usri2_parms,
//                    UserInfo->Parameters.Buffer,
//                    UserInfo->Parameters.Length);
//             UserInfo2->usri2_parms[UserInfo->Parameters.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->Parameters.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_workstations = Ptr;
//             memcpy(UserInfo2->usri2_workstations,
//                    UserInfo->WorkStations.Buffer,
//                    UserInfo->WorkStations.Length);
//             UserInfo2->usri2_workstations[UserInfo->WorkStations.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->WorkStations.Length + sizeof(WCHAR));
//
//             if (UserInfo->LastLogon.QuadPart == 0)
//                 UserInfo2->usri2_last_logon = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogon,
//                                           &UserInfo2->usri2_last_logon);
//
//             if (UserInfo->LastLogoff.QuadPart == 0)
//                 UserInfo2->usri2_last_logoff = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogoff,
//                                           &UserInfo2->usri2_last_logoff);
//
//             if (UserInfo->AccountExpires.QuadPart == MAXLONGLONG)
//                 UserInfo2->usri2_acct_expires = TIMEQ_FOREVER;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->AccountExpires,
//                                           &UserInfo2->usri2_acct_expires);
//
//             UserInfo2->usri2_max_storage = USER_MAXSTORAGE_UNLIMITED;
//             UserInfo2->usri2_units_per_week = UserInfo->LogonHours.UnitsPerWeek;
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//             {
//                 UserInfo2->usri2_logon_hours = (PVOID)Ptr;
//
//                 memcpy(UserInfo2->usri2_logon_hours,
//                        UserInfo->LogonHours.LogonHours,
//                        (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//
//                 Ptr = (LPWSTR)((ULONG_PTR)Ptr + (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//             }
//
//             UserInfo2->usri2_bad_pw_count = UserInfo->BadPasswordCount;
//             UserInfo2->usri2_num_logons = UserInfo->LogonCount;
//
//             UserInfo2->usri2_logon_server = Ptr;
//             memcpy(UserInfo2->usri2_logon_server,
//                    LogonServer.Buffer,
//                    LogonServer.Length);
//             UserInfo2->usri2_logon_server[LogonServer.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + LogonServer.Length + sizeof(WCHAR));
//
//             UserInfo2->usri2_country_code = UserInfo->CountryCode;
//             UserInfo2->usri2_code_page = UserInfo->CodePage;
//             break;
//
//         case 3:
//             UserInfo3 = (PUSER_INFO_3)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo3 + sizeof(USER_INFO_3));
//
//             UserInfo3->usri3_name = Ptr;
//
//             memcpy(UserInfo3->usri3_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo3->usri3_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_password = NULL;
//             UserInfo3->usri3_password_age = GetPasswordAge(&UserInfo->PasswordLastSet);
//
//             UserInfo3->usri3_priv = Priv;
//
//             UserInfo3->usri3_home_dir = Ptr;
//             memcpy(UserInfo3->usri3_home_dir,
//                    UserInfo->HomeDirectory.Buffer,
//                    UserInfo->HomeDirectory.Length);
//             UserInfo3->usri3_home_dir[UserInfo->HomeDirectory.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectory.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_comment = Ptr;
//             memcpy(UserInfo3->usri3_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo3->usri3_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                      Dacl);
//
//             UserInfo3->usri3_script_path = Ptr;
//             memcpy(UserInfo3->usri3_script_path,
//                    UserInfo->ScriptPath.Buffer,
//                    UserInfo->ScriptPath.Length);
//             UserInfo3->usri3_script_path[UserInfo->ScriptPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->ScriptPath.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_auth_flags = AuthFlags;
//
//             UserInfo3->usri3_full_name = Ptr;
//             memcpy(UserInfo3->usri3_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo3->usri3_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_usr_comment = Ptr;
//             memcpy(UserInfo3->usri3_usr_comment,
//                    UserInfo->UserComment.Buffer,
//                    UserInfo->UserComment.Length);
//             UserInfo3->usri3_usr_comment[UserInfo->UserComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserComment.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_parms = Ptr;
//             memcpy(UserInfo3->usri3_parms,
//                    UserInfo->Parameters.Buffer,
//                    UserInfo->Parameters.Length);
//             UserInfo3->usri3_parms[UserInfo->Parameters.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->Parameters.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_workstations = Ptr;
//             memcpy(UserInfo3->usri3_workstations,
//                    UserInfo->WorkStations.Buffer,
//                    UserInfo->WorkStations.Length);
//             UserInfo3->usri3_workstations[UserInfo->WorkStations.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->WorkStations.Length + sizeof(WCHAR));
//
//             if (UserInfo->LastLogon.QuadPart == 0)
//                 UserInfo3->usri3_last_logon = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogon,
//                                           &UserInfo3->usri3_last_logon);
//
//             if (UserInfo->LastLogoff.QuadPart == 0)
//                 UserInfo3->usri3_last_logoff = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogoff,
//                                           &UserInfo3->usri3_last_logoff);
//
//             if (UserInfo->AccountExpires.QuadPart == MAXLONGLONG)
//                 UserInfo3->usri3_acct_expires = TIMEQ_FOREVER;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->AccountExpires,
//                                           &UserInfo3->usri3_acct_expires);
//
//             UserInfo3->usri3_max_storage = USER_MAXSTORAGE_UNLIMITED;
//             UserInfo3->usri3_units_per_week = UserInfo->LogonHours.UnitsPerWeek;
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//             {
//                 UserInfo3->usri3_logon_hours = (PVOID)Ptr;
//
//                 memcpy(UserInfo3->usri3_logon_hours,
//                        UserInfo->LogonHours.LogonHours,
//                        (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//
//                 Ptr = (LPWSTR)((ULONG_PTR)Ptr + (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//             }
//
//             UserInfo3->usri3_bad_pw_count = UserInfo->BadPasswordCount;
//             UserInfo3->usri3_num_logons = UserInfo->LogonCount;
//
//             UserInfo3->usri3_logon_server = Ptr;
//             memcpy(UserInfo3->usri3_logon_server,
//                    LogonServer.Buffer,
//                    LogonServer.Length);
//             UserInfo3->usri3_logon_server[LogonServer.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + LogonServer.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_country_code = UserInfo->CountryCode;
//             UserInfo3->usri3_code_page = UserInfo->CodePage;
//             UserInfo3->usri3_user_id = RelativeId;
//             UserInfo3->usri3_primary_group_id = UserInfo->PrimaryGroupId;
//
//             UserInfo3->usri3_profile = Ptr;
//             memcpy(UserInfo3->usri3_profile,
//                    UserInfo->ProfilePath.Buffer,
//                    UserInfo->ProfilePath.Length);
//             UserInfo3->usri3_profile[UserInfo->ProfilePath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->ProfilePath.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_home_dir_drive = Ptr;
//             memcpy(UserInfo3->usri3_home_dir_drive,
//                    UserInfo->HomeDirectoryDrive.Buffer,
//                    UserInfo->HomeDirectoryDrive.Length);
//             UserInfo3->usri3_home_dir_drive[UserInfo->HomeDirectoryDrive.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectoryDrive.Length + sizeof(WCHAR));
//
//             UserInfo3->usri3_password_expired = (UserInfo->UserAccountControl & USER_PASSWORD_EXPIRED);
//             break;
//
//         case 4:
//             UserInfo4 = (PUSER_INFO_4)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo4 + sizeof(USER_INFO_4));
//
//             UserInfo4->usri4_name = Ptr;
//
//             memcpy(UserInfo4->usri4_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo4->usri4_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_password = NULL;
//             UserInfo4->usri4_password_age = GetPasswordAge(&UserInfo->PasswordLastSet);
//
//             UserInfo4->usri4_priv = Priv;
//
//             UserInfo4->usri4_home_dir = Ptr;
//             memcpy(UserInfo4->usri4_home_dir,
//                    UserInfo->HomeDirectory.Buffer,
//                    UserInfo->HomeDirectory.Length);
//             UserInfo4->usri4_home_dir[UserInfo->HomeDirectory.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectory.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_comment = Ptr;
//             memcpy(UserInfo4->usri4_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo4->usri4_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                      Dacl);
//
//             UserInfo4->usri4_script_path = Ptr;
//             memcpy(UserInfo4->usri4_script_path,
//                    UserInfo->ScriptPath.Buffer,
//                    UserInfo->ScriptPath.Length);
//             UserInfo4->usri4_script_path[UserInfo->ScriptPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->ScriptPath.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_auth_flags = AuthFlags;
//
//             UserInfo4->usri4_full_name = Ptr;
//             memcpy(UserInfo4->usri4_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo4->usri4_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_usr_comment = Ptr;
//             memcpy(UserInfo4->usri4_usr_comment,
//                    UserInfo->UserComment.Buffer,
//                    UserInfo->UserComment.Length);
//             UserInfo4->usri4_usr_comment[UserInfo->UserComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserComment.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_parms = Ptr;
//             memcpy(UserInfo4->usri4_parms,
//                    UserInfo->Parameters.Buffer,
//                    UserInfo->Parameters.Length);
//             UserInfo4->usri4_parms[UserInfo->Parameters.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->Parameters.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_workstations = Ptr;
//             memcpy(UserInfo4->usri4_workstations,
//                    UserInfo->WorkStations.Buffer,
//                    UserInfo->WorkStations.Length);
//             UserInfo4->usri4_workstations[UserInfo->WorkStations.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->WorkStations.Length + sizeof(WCHAR));
//
//             if (UserInfo->LastLogon.QuadPart == 0)
//                 UserInfo4->usri4_last_logon = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogon,
//                                           &UserInfo4->usri4_last_logon);
//
//             if (UserInfo->LastLogoff.QuadPart == 0)
//                 UserInfo4->usri4_last_logoff = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogoff,
//                                           &UserInfo4->usri4_last_logoff);
//
//             if (UserInfo->AccountExpires.QuadPart == MAXLONGLONG)
//                 UserInfo4->usri4_acct_expires = TIMEQ_FOREVER;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->AccountExpires,
//                                           &UserInfo4->usri4_acct_expires);
//
//             UserInfo4->usri4_max_storage = USER_MAXSTORAGE_UNLIMITED;
//             UserInfo4->usri4_units_per_week = UserInfo->LogonHours.UnitsPerWeek;
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//             {
//                 UserInfo4->usri4_logon_hours = (PVOID)Ptr;
//
//                 memcpy(UserInfo4->usri4_logon_hours,
//                        UserInfo->LogonHours.LogonHours,
//                        (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//
//                 Ptr = (LPWSTR)((ULONG_PTR)Ptr + (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//             }
//
//             UserInfo4->usri4_bad_pw_count = UserInfo->BadPasswordCount;
//             UserInfo4->usri4_num_logons = UserInfo->LogonCount;
//
//             UserInfo4->usri4_logon_server = Ptr;
//             memcpy(UserInfo4->usri4_logon_server,
//                    LogonServer.Buffer,
//                    LogonServer.Length);
//             UserInfo4->usri4_logon_server[LogonServer.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + LogonServer.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_country_code = UserInfo->CountryCode;
//             UserInfo4->usri4_code_page = UserInfo->CodePage;
//
//             UserInfo4->usri4_user_sid = (PVOID)Ptr;
//             CopySidFromSidAndRid(UserInfo4->usri4_user_sid, AccountDomainSid, RelativeId);
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + RtlLengthSid(AccountDomainSid) + sizeof(ULONG));
//
//             UserInfo4->usri4_primary_group_id = UserInfo->PrimaryGroupId;
//
//             UserInfo4->usri4_profile = Ptr;
//             memcpy(UserInfo4->usri4_profile,
//                    UserInfo->ProfilePath.Buffer,
//                    UserInfo->ProfilePath.Length);
//             UserInfo4->usri4_profile[UserInfo->ProfilePath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->ProfilePath.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_home_dir_drive = Ptr;
//             memcpy(UserInfo4->usri4_home_dir_drive,
//                    UserInfo->HomeDirectoryDrive.Buffer,
//                    UserInfo->HomeDirectoryDrive.Length);
//             UserInfo4->usri4_home_dir_drive[UserInfo->HomeDirectoryDrive.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectoryDrive.Length + sizeof(WCHAR));
//
//             UserInfo4->usri4_password_expired = (UserInfo->UserAccountControl & USER_PASSWORD_EXPIRED);
//             break;
//
//         case 10:
//             UserInfo10 = (PUSER_INFO_10)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo10 + sizeof(USER_INFO_10));
//
//             UserInfo10->usri10_name = Ptr;
//
//             memcpy(UserInfo10->usri10_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo10->usri10_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo10->usri10_comment = Ptr;
//             memcpy(UserInfo10->usri10_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo10->usri10_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo10->usri10_usr_comment = Ptr;
//             memcpy(UserInfo10->usri10_usr_comment,
//                    UserInfo->UserComment.Buffer,
//                    UserInfo->UserComment.Length);
//             UserInfo10->usri10_usr_comment[UserInfo->UserComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserComment.Length + sizeof(WCHAR));
//
//             UserInfo10->usri10_full_name = Ptr;
//             memcpy(UserInfo10->usri10_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo10->usri10_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 11:
//             UserInfo11 = (PUSER_INFO_11)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo11 + sizeof(USER_INFO_11));
//
//             UserInfo11->usri11_name = Ptr;
//
//             memcpy(UserInfo11->usri11_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo11->usri11_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_comment = Ptr;
//             memcpy(UserInfo11->usri11_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo11->usri11_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_usr_comment = Ptr;
//             memcpy(UserInfo11->usri11_usr_comment,
//                    UserInfo->UserComment.Buffer,
//                    UserInfo->UserComment.Length);
//             UserInfo11->usri11_usr_comment[UserInfo->UserComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserComment.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_full_name = Ptr;
//             memcpy(UserInfo11->usri11_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo11->usri11_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_priv = Priv;
//             UserInfo11->usri11_auth_flags = AuthFlags;
//
//             UserInfo11->usri11_password_age = GetPasswordAge(&UserInfo->PasswordLastSet);
//
//             UserInfo11->usri11_home_dir = Ptr;
//             memcpy(UserInfo11->usri11_home_dir,
//                    UserInfo->HomeDirectory.Buffer,
//                    UserInfo->HomeDirectory.Length);
//             UserInfo11->usri11_home_dir[UserInfo->HomeDirectory.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->HomeDirectory.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_parms = Ptr;
//             memcpy(UserInfo11->usri11_parms,
//                    UserInfo->Parameters.Buffer,
//                    UserInfo->Parameters.Length);
//             UserInfo11->usri11_parms[UserInfo->Parameters.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->Parameters.Length + sizeof(WCHAR));
//
//             if (UserInfo->LastLogon.QuadPart == 0)
//                 UserInfo11->usri11_last_logon = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogon,
//                                           &UserInfo11->usri11_last_logon);
//
//             if (UserInfo->LastLogoff.QuadPart == 0)
//                 UserInfo11->usri11_last_logoff = 0;
//             else
//                 RtlTimeToSecondsSince1970(&UserInfo->LastLogoff,
//                                           &UserInfo11->usri11_last_logoff);
//
//             UserInfo11->usri11_bad_pw_count = UserInfo->BadPasswordCount;
//             UserInfo11->usri11_num_logons = UserInfo->LogonCount;
//
//             UserInfo11->usri11_logon_server = Ptr;
//             memcpy(UserInfo11->usri11_logon_server,
//                    LogonServer.Buffer,
//                    LogonServer.Length);
//             UserInfo11->usri11_logon_server[LogonServer.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + LogonServer.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_country_code = UserInfo->CountryCode;
//
//             UserInfo11->usri11_workstations = Ptr;
//             memcpy(UserInfo11->usri11_workstations,
//                    UserInfo->WorkStations.Buffer,
//                    UserInfo->WorkStations.Length);
//             UserInfo11->usri11_workstations[UserInfo->WorkStations.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->WorkStations.Length + sizeof(WCHAR));
//
//             UserInfo11->usri11_max_storage = USER_MAXSTORAGE_UNLIMITED;
//             UserInfo11->usri11_units_per_week = UserInfo->LogonHours.UnitsPerWeek;
//
//             if (UserInfo->LogonHours.UnitsPerWeek > 0)
//             {
//                 UserInfo11->usri11_logon_hours = (PVOID)Ptr;
//
//                 memcpy(UserInfo11->usri11_logon_hours,
//                        UserInfo->LogonHours.LogonHours,
//                        (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//
//                 Ptr = (LPWSTR)((ULONG_PTR)Ptr + (((ULONG)UserInfo->LogonHours.UnitsPerWeek) + 7) / 8);
//             }
//
//             UserInfo11->usri11_code_page = UserInfo->CodePage;
//             break;
//
//         case 20:
//             UserInfo20 = (PUSER_INFO_20)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo20 + sizeof(USER_INFO_20));
//
//             UserInfo20->usri20_name = Ptr;
//
//             memcpy(UserInfo20->usri20_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo20->usri20_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo20->usri20_full_name = Ptr;
//             memcpy(UserInfo20->usri20_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo20->usri20_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo20->usri20_comment = Ptr;
//             memcpy(UserInfo20->usri20_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo20->usri20_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo20->usri20_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                        Dacl);
//
//             UserInfo20->usri20_user_id = RelativeId;
//             break;
//
//         case 23:
//             UserInfo23 = (PUSER_INFO_23)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)UserInfo23 + sizeof(USER_INFO_23));
//
//             UserInfo23->usri23_name = Ptr;
//
//             memcpy(UserInfo23->usri23_name,
//                    UserInfo->UserName.Buffer,
//                    UserInfo->UserName.Length);
//             UserInfo23->usri23_name[UserInfo->UserName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->UserName.Length + sizeof(WCHAR));
//
//             UserInfo23->usri23_full_name = Ptr;
//             memcpy(UserInfo23->usri23_full_name,
//                    UserInfo->FullName.Buffer,
//                    UserInfo->FullName.Length);
//             UserInfo23->usri23_full_name[UserInfo->FullName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->FullName.Length + sizeof(WCHAR));
//
//             UserInfo23->usri23_comment = Ptr;
//             memcpy(UserInfo23->usri23_comment,
//                    UserInfo->AdminComment.Buffer,
//                    UserInfo->AdminComment.Length);
//             UserInfo23->usri23_comment[UserInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + UserInfo->AdminComment.Length + sizeof(WCHAR));
//
//             UserInfo23->usri23_flags = GetAccountFlags(UserInfo->UserAccountControl,
//                                                        Dacl);
//
//             UserInfo23->usri23_user_sid = (PVOID)Ptr;
//             CopySidFromSidAndRid(UserInfo23->usri23_user_sid, AccountDomainSid, RelativeId);
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + RtlLengthSid(AccountDomainSid) + sizeof(ULONG));
//             break;
//     }
//
// done:
//     if (UserInfo != NULL)
//         FreeUserInfo(UserInfo);
//
//     if (Dacl != NULL)
//         HeapFree(GetProcessHeap(), 0, Dacl);
//
//     if (ApiStatus == NERR_Success)
//     {
//         *Buffer = LocalBuffer;
//     }
//     else
//     {
//         if (LocalBuffer != NULL)
//             NetApiBufferFree(LocalBuffer);
//     }
//
//     return ApiStatus;
// }
//
//
// static
// NET_API_STATUS
// SetUserInfo(SAM_HANDLE UserHandle,
//             LPBYTE UserInfo,
//             DWORD Level,
//             PDWORD parm_err)
// {
//     USER_ALL_INFORMATION UserAllInfo;
//     PUSER_INFO_0 UserInfo0;
//     PUSER_INFO_1 UserInfo1;
//     PUSER_INFO_2 UserInfo2;
//     PUSER_INFO_3 UserInfo3;
//     PUSER_INFO_4 UserInfo4;
//     PUSER_INFO_22 UserInfo22;
//     PUSER_INFO_1003 UserInfo1003;
//     PUSER_INFO_1006 UserInfo1006;
//     PUSER_INFO_1007 UserInfo1007;
//     PUSER_INFO_1008 UserInfo1008;
//     PUSER_INFO_1009 UserInfo1009;
//     PUSER_INFO_1011 UserInfo1011;
//     PUSER_INFO_1012 UserInfo1012;
//     PUSER_INFO_1013 UserInfo1013;
//     PUSER_INFO_1014 UserInfo1014;
//     PUSER_INFO_1017 UserInfo1017;
//     PUSER_INFO_1020 UserInfo1020;
//     PUSER_INFO_1024 UserInfo1024;
//     PUSER_INFO_1025 UserInfo1025;
//     PUSER_INFO_1051 UserInfo1051;
//     PUSER_INFO_1052 UserInfo1052;
//     PUSER_INFO_1053 UserInfo1053;
//     PACL Dacl = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     ZeroMemory(&UserAllInfo, sizeof(USER_ALL_INFORMATION));
//
//     if ((Level == 1) || (Level == 2) || (Level == 3) ||
//         (Level == 4) || (Level == 22) || (Level == 1008))
//     {
//         ApiStatus = GetUserDacl(UserHandle, &Dacl);
//         if (ApiStatus != NERR_Success)
//             goto done;
//     }
//
//     switch (Level)
//     {
//         case 0:
//             UserInfo0 = (PUSER_INFO_0)UserInfo;
//
//             RtlInitUnicodeString(&UserAllInfo.UserName,
//                                  UserInfo0->usri0_name);
//
//             UserAllInfo.WhichFields |= USER_ALL_USERNAME;
//             break;
//
//         case 1:
//             UserInfo1 = (PUSER_INFO_1)UserInfo;
//
//             // usri1_name ignored
//
//             if (UserInfo1->usri1_password != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.NtPassword,
//                                      UserInfo1->usri1_password);
//                 UserAllInfo.NtPasswordPresent = TRUE;
//                 UserAllInfo.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
//             }
//
//             // usri1_password_age ignored
//
// //          UserInfo1->usri1_priv
//
//             if (UserInfo1->usri1_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo1->usri1_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//
//             if (UserInfo1->usri1_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo1->usri1_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//
//             ChangeUserDacl(Dacl, UserInfo1->usri1_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo1->usri1_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//
//             if (UserInfo1->usri1_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo1->usri1_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//             break;
//
//         case 2:
//             UserInfo2 = (PUSER_INFO_2)UserInfo;
//
//             // usri2_name ignored
//
//             if (UserInfo2->usri2_password != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.NtPassword,
//                                      UserInfo2->usri2_password);
//                 UserAllInfo.NtPasswordPresent = TRUE;
//                 UserAllInfo.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
//             }
//
//             // usri2_password_age ignored
//
// //          UserInfo2->usri2_priv;
//
//             if (UserInfo2->usri2_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo2->usri2_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//
//             if (UserInfo2->usri2_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo2->usri2_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//
//             ChangeUserDacl(Dacl, UserInfo2->usri2_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo2->usri2_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//
//             if (UserInfo2->usri2_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo2->usri2_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//
// //          UserInfo2->usri2_auth_flags;
//
//             if (UserInfo2->usri2_full_name != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.FullName,
//                                      UserInfo2->usri2_full_name);
//                 UserAllInfo.WhichFields |= USER_ALL_FULLNAME;
//             }
//
//             if (UserInfo2->usri2_usr_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.UserComment,
//                                      UserInfo2->usri2_usr_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_USERCOMMENT;
//             }
//
//             if (UserInfo2->usri2_parms != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.Parameters,
//                                      UserInfo2->usri2_parms);
//                 UserAllInfo.WhichFields |= USER_ALL_PARAMETERS;
//             }
//
//             if (UserInfo2->usri2_workstations != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.WorkStations,
//                                      UserInfo2->usri2_workstations);
//                 UserAllInfo.WhichFields |= USER_ALL_WORKSTATIONS;
//             }
//
//             // usri2_last_logon ignored
//             // usri2_last_logoff ignored
//
//             if (UserInfo2->usri2_acct_expires == TIMEQ_FOREVER)
//             {
//                 UserAllInfo.AccountExpires.QuadPart = MAXLONGLONG;
//             }
//             else
//             {
//                 RtlSecondsSince1970ToTime(UserInfo2->usri2_acct_expires,
//                                           &UserAllInfo.AccountExpires);
//             }
//             UserAllInfo.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
//
//             // usri2_max_storage ignored
//
//             if (UserInfo2->usri2_logon_hours != NULL)
//             {
//                 if (UserInfo2->usri2_units_per_week > USHRT_MAX)
//                 {
//                     if (parm_err != NULL)
//                         *parm_err = USER_UNITS_PER_WEEK_PARMNUM;
//                     ApiStatus = ERROR_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 UserAllInfo.LogonHours.UnitsPerWeek = UserInfo2->usri2_units_per_week;
//                 UserAllInfo.LogonHours.LogonHours = UserInfo2->usri2_logon_hours;
//                 UserAllInfo.WhichFields |= USER_ALL_LOGONHOURS;
//             }
//
//             // usri2_bad_pw_count ignored
//             // usri2_num_logons ignored
//             // usri2_logon_server ignored
//
//             UserAllInfo.CountryCode = UserInfo2->usri2_country_code;
//             UserAllInfo.WhichFields |= USER_ALL_COUNTRYCODE;
//
//             UserAllInfo.CodePage = UserInfo2->usri2_code_page;
//             UserAllInfo.WhichFields |= USER_ALL_CODEPAGE;
//             break;
//
//         case 3:
//             UserInfo3 = (PUSER_INFO_3)UserInfo;
//
//             // usri3_name ignored
//
//             if (UserInfo3->usri3_password != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.NtPassword,
//                                      UserInfo3->usri3_password);
//                 UserAllInfo.NtPasswordPresent = TRUE;
//                 UserAllInfo.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
//             }
//
//             // usri3_password_age ignored
//
// //          UserInfo3->usri3_priv;
//
//             if (UserInfo3->usri3_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo3->usri3_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//
//             if (UserInfo3->usri3_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo3->usri3_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//
//             ChangeUserDacl(Dacl, UserInfo3->usri3_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo3->usri3_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//
//             if (UserInfo3->usri3_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo3->usri3_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//
// //          UserInfo3->usri3_auth_flags;
//
//             if (UserInfo3->usri3_full_name != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.FullName,
//                                      UserInfo3->usri3_full_name);
//                 UserAllInfo.WhichFields |= USER_ALL_FULLNAME;
//             }
//
//             if (UserInfo3->usri3_usr_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.UserComment,
//                                      UserInfo3->usri3_usr_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_USERCOMMENT;
//             }
//
//             if (UserInfo3->usri3_parms != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.Parameters,
//                                      UserInfo3->usri3_parms);
//                 UserAllInfo.WhichFields |= USER_ALL_PARAMETERS;
//             }
//
//             if (UserInfo3->usri3_workstations != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.WorkStations,
//                                      UserInfo3->usri3_workstations);
//                 UserAllInfo.WhichFields |= USER_ALL_WORKSTATIONS;
//             }
//
//             // usri3_last_logon ignored
//             // usri3_last_logoff ignored
//
//             if (UserInfo3->usri3_acct_expires == TIMEQ_FOREVER)
//             {
//                 UserAllInfo.AccountExpires.QuadPart = MAXLONGLONG;
//             }
//             else
//             {
//                 RtlSecondsSince1970ToTime(UserInfo3->usri3_acct_expires,
//                                           &UserAllInfo.AccountExpires);
//             }
//             UserAllInfo.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
//
//             // usri3_max_storage ignored
//
//             if (UserInfo3->usri3_logon_hours != NULL)
//             {
//                 if (UserInfo3->usri3_units_per_week > USHRT_MAX)
//                 {
//                     if (parm_err != NULL)
//                         *parm_err = USER_UNITS_PER_WEEK_PARMNUM;
//                     ApiStatus = ERROR_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 UserAllInfo.LogonHours.UnitsPerWeek = UserInfo3->usri3_units_per_week;
//                 UserAllInfo.LogonHours.LogonHours = UserInfo3->usri3_logon_hours;
//                 UserAllInfo.WhichFields |= USER_ALL_LOGONHOURS;
//             }
//
//             // usri3_bad_pw_count ignored
//             // usri3_num_logons ignored
//             // usri3_logon_server ignored
//
//             UserAllInfo.CountryCode = UserInfo3->usri3_country_code;
//             UserAllInfo.WhichFields |= USER_ALL_COUNTRYCODE;
//
//             UserAllInfo.CodePage = UserInfo3->usri3_code_page;
//             UserAllInfo.WhichFields |= USER_ALL_CODEPAGE;
//
//             // usri3_user_id ignored
//
//             UserAllInfo.PrimaryGroupId = UserInfo3->usri3_primary_group_id;
//             UserAllInfo.WhichFields |= USER_ALL_PRIMARYGROUPID;
//
//             if (UserInfo3->usri3_profile != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ProfilePath,
//                                      UserInfo3->usri3_profile);
//                 UserAllInfo.WhichFields |= USER_ALL_PROFILEPATH;
//             }
//
//             if (UserInfo3->usri3_home_dir_drive != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectoryDrive,
//                                      UserInfo3->usri3_home_dir_drive);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
//             }
//
//             UserAllInfo.PasswordExpired = (UserInfo3->usri3_password_expired != 0);
//             UserAllInfo.WhichFields |= USER_ALL_PASSWORDEXPIRED;
//             break;
//
//         case 4:
//             UserInfo4 = (PUSER_INFO_4)UserInfo;
//
//             // usri4_name ignored
//
//             if (UserInfo4->usri4_password != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.NtPassword,
//                                      UserInfo4->usri4_password);
//                 UserAllInfo.NtPasswordPresent = TRUE;
//                 UserAllInfo.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
//             }
//
//             // usri4_password_age ignored
//
// //          UserInfo3->usri4_priv;
//
//             if (UserInfo4->usri4_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo4->usri4_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//
//             if (UserInfo4->usri4_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo4->usri4_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//
//             ChangeUserDacl(Dacl, UserInfo4->usri4_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo4->usri4_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//
//             if (UserInfo4->usri4_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo4->usri4_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//
// //          UserInfo4->usri4_auth_flags;
//
//             if (UserInfo4->usri4_full_name != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.FullName,
//                                      UserInfo4->usri4_full_name);
//                 UserAllInfo.WhichFields |= USER_ALL_FULLNAME;
//             }
//
//             if (UserInfo4->usri4_usr_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.UserComment,
//                                      UserInfo4->usri4_usr_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_USERCOMMENT;
//             }
//
//             if (UserInfo4->usri4_parms != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.Parameters,
//                                      UserInfo4->usri4_parms);
//                 UserAllInfo.WhichFields |= USER_ALL_PARAMETERS;
//             }
//
//             if (UserInfo4->usri4_workstations != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.WorkStations,
//                                      UserInfo4->usri4_workstations);
//                 UserAllInfo.WhichFields |= USER_ALL_WORKSTATIONS;
//             }
//
//             // usri4_last_logon ignored
//             // usri4_last_logoff ignored
//
//             if (UserInfo4->usri4_acct_expires == TIMEQ_FOREVER)
//             {
//                 UserAllInfo.AccountExpires.QuadPart = MAXLONGLONG;
//             }
//             else
//             {
//                 RtlSecondsSince1970ToTime(UserInfo4->usri4_acct_expires,
//                                           &UserAllInfo.AccountExpires);
//             }
//             UserAllInfo.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
//
//             // usri4_max_storage ignored
//
//             if (UserInfo4->usri4_logon_hours != NULL)
//             {
//                 if (UserInfo4->usri4_units_per_week > USHRT_MAX)
//                 {
//                     if (parm_err != NULL)
//                         *parm_err = USER_UNITS_PER_WEEK_PARMNUM;
//                     ApiStatus = ERROR_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 UserAllInfo.LogonHours.UnitsPerWeek = UserInfo4->usri4_units_per_week;
//                 UserAllInfo.LogonHours.LogonHours = UserInfo4->usri4_logon_hours;
//                 UserAllInfo.WhichFields |= USER_ALL_LOGONHOURS;
//             }
//
//             // usri4_bad_pw_count ignored
//             // usri4_num_logons ignored
//             // usri4_logon_server ignored
//
//             UserAllInfo.CountryCode = UserInfo4->usri4_country_code;
//             UserAllInfo.WhichFields |= USER_ALL_COUNTRYCODE;
//
//             UserAllInfo.CodePage = UserInfo4->usri4_code_page;
//             UserAllInfo.WhichFields |= USER_ALL_CODEPAGE;
//
//             // usri4_user_sid ignored
//
//             UserAllInfo.PrimaryGroupId = UserInfo4->usri4_primary_group_id;
//             UserAllInfo.WhichFields |= USER_ALL_PRIMARYGROUPID;
//
//             if (UserInfo4->usri4_profile != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ProfilePath,
//                                      UserInfo4->usri4_profile);
//                 UserAllInfo.WhichFields |= USER_ALL_PROFILEPATH;
//             }
//
//             if (UserInfo4->usri4_home_dir_drive != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectoryDrive,
//                                      UserInfo4->usri4_home_dir_drive);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
//             }
//
//             UserAllInfo.PasswordExpired = (UserInfo4->usri4_password_expired != 0);
//             UserAllInfo.WhichFields |= USER_ALL_PASSWORDEXPIRED;
//             break;
//
// //        case 21:
// //            break;
//
//         case 22:
//             UserInfo22 = (PUSER_INFO_22)UserInfo;
//
//             // usri22_name ignored
//
// //          UserInfo22->usri22_password[ENCRYPTED_PWLEN];
//
//             // usri22_password_age ignored
//
// //          UserInfo3->usri3_priv;
//
//             if (UserInfo22->usri22_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo22->usri22_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//
//             if (UserInfo22->usri22_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo22->usri22_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//
//             ChangeUserDacl(Dacl, UserInfo22->usri22_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo22->usri22_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//
//             if (UserInfo22->usri22_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo22->usri22_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//
// //          UserInfo22->usri22_auth_flags;
//
//             if (UserInfo22->usri22_full_name != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.FullName,
//                                      UserInfo22->usri22_full_name);
//                 UserAllInfo.WhichFields |= USER_ALL_FULLNAME;
//             }
//
//             if (UserInfo22->usri22_usr_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.UserComment,
//                                      UserInfo22->usri22_usr_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_USERCOMMENT;
//             }
//
//             if (UserInfo22->usri22_parms != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.Parameters,
//                                      UserInfo22->usri22_parms);
//                 UserAllInfo.WhichFields |= USER_ALL_PARAMETERS;
//             }
//
//             if (UserInfo22->usri22_workstations != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.WorkStations,
//                                      UserInfo22->usri22_workstations);
//                 UserAllInfo.WhichFields |= USER_ALL_WORKSTATIONS;
//             }
//
//             // usri22_last_logon ignored
//             // usri22_last_logoff ignored
//
//             if (UserInfo22->usri22_acct_expires == TIMEQ_FOREVER)
//             {
//                 UserAllInfo.AccountExpires.QuadPart = MAXLONGLONG;
//             }
//             else
//             {
//                 RtlSecondsSince1970ToTime(UserInfo22->usri22_acct_expires,
//                                           &UserAllInfo.AccountExpires);
//             }
//             UserAllInfo.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
//
//             // usri22_max_storage ignored
//
//             if (UserInfo22->usri22_logon_hours != NULL)
//             {
//                 if (UserInfo22->usri22_units_per_week > USHRT_MAX)
//                 {
//                     if (parm_err != NULL)
//                         *parm_err = USER_UNITS_PER_WEEK_PARMNUM;
//                     ApiStatus = ERROR_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 UserAllInfo.LogonHours.UnitsPerWeek = UserInfo22->usri22_units_per_week;
//                 UserAllInfo.LogonHours.LogonHours = UserInfo22->usri22_logon_hours;
//                 UserAllInfo.WhichFields |= USER_ALL_LOGONHOURS;
//             }
//
//             // usri22_bad_pw_count ignored
//             // usri22_num_logons ignored
//             // usri22_logon_server ignored
//
//             UserAllInfo.CountryCode = UserInfo22->usri22_country_code;
//             UserAllInfo.WhichFields |= USER_ALL_COUNTRYCODE;
//
//             UserAllInfo.CodePage = UserInfo22->usri22_code_page;
//             UserAllInfo.WhichFields |= USER_ALL_CODEPAGE;
//             break;
//
//         case 1003:
//             UserInfo1003 = (PUSER_INFO_1003)UserInfo;
//
//             if (UserInfo1003->usri1003_password != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.NtPassword,
//                                      UserInfo1003->usri1003_password);
//                 UserAllInfo.NtPasswordPresent = TRUE;
//                 UserAllInfo.WhichFields |= USER_ALL_NTPASSWORDPRESENT;
//             }
//             break;
//
// //        case 1005:
// //            break;
//
//         case 1006:
//             UserInfo1006 = (PUSER_INFO_1006)UserInfo;
//
//             if (UserInfo1006->usri1006_home_dir != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectory,
//                                      UserInfo1006->usri1006_home_dir);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORY;
//             }
//             break;
//
//         case 1007:
//             UserInfo1007 = (PUSER_INFO_1007)UserInfo;
//
//             if (UserInfo1007->usri1007_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.AdminComment,
//                                      UserInfo1007->usri1007_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_ADMINCOMMENT;
//             }
//             break;
//
//         case 1008:
//             UserInfo1008 = (PUSER_INFO_1008)UserInfo;
//             ChangeUserDacl(Dacl, UserInfo1008->usri1008_flags);
//             UserAllInfo.UserAccountControl = GetAccountControl(UserInfo1008->usri1008_flags);
//             UserAllInfo.WhichFields |= USER_ALL_USERACCOUNTCONTROL;
//             break;
//
//         case 1009:
//             UserInfo1009 = (PUSER_INFO_1009)UserInfo;
//
//             if (UserInfo1009->usri1009_script_path != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ScriptPath,
//                                      UserInfo1009->usri1009_script_path);
//                 UserAllInfo.WhichFields |= USER_ALL_SCRIPTPATH;
//             }
//             break;
//
// //        case 1010:
// //            break;
//
//         case 1011:
//             UserInfo1011 = (PUSER_INFO_1011)UserInfo;
//
//             if (UserInfo1011->usri1011_full_name != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.FullName,
//                                      UserInfo1011->usri1011_full_name);
//                 UserAllInfo.WhichFields |= USER_ALL_FULLNAME;
//             }
//             break;
//
//         case 1012:
//             UserInfo1012 = (PUSER_INFO_1012)UserInfo;
//
//             if (UserInfo1012->usri1012_usr_comment != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.UserComment,
//                                      UserInfo1012->usri1012_usr_comment);
//                 UserAllInfo.WhichFields |= USER_ALL_USERCOMMENT;
//             }
//             break;
//
//         case 1013:
//             UserInfo1013 = (PUSER_INFO_1013)UserInfo;
//
//             if (UserInfo1013->usri1013_parms != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.Parameters,
//                                      UserInfo1013->usri1013_parms);
//                 UserAllInfo.WhichFields |= USER_ALL_PARAMETERS;
//             }
//             break;
//
//         case 1014:
//             UserInfo1014 = (PUSER_INFO_1014)UserInfo;
//
//             if (UserInfo1014->usri1014_workstations != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.WorkStations,
//                                      UserInfo1014->usri1014_workstations);
//                 UserAllInfo.WhichFields |= USER_ALL_WORKSTATIONS;
//             }
//             break;
//
//         case 1017:
//             UserInfo1017 = (PUSER_INFO_1017)UserInfo;
//
//             if (UserInfo1017->usri1017_acct_expires == TIMEQ_FOREVER)
//             {
//                 UserAllInfo.AccountExpires.QuadPart = MAXLONGLONG;
//             }
//             else
//             {
//                 RtlSecondsSince1970ToTime(UserInfo1017->usri1017_acct_expires,
//                                           &UserAllInfo.AccountExpires);
//             }
//             UserAllInfo.WhichFields |= USER_ALL_ACCOUNTEXPIRES;
//             break;
//
//         case 1018:
//             // usri1018_max_storage ignored
//             break;
//
//         case 1020:
//             UserInfo1020 = (PUSER_INFO_1020)UserInfo;
//
//             if (UserInfo1020->usri1020_logon_hours != NULL)
//             {
//                 if (UserInfo1020->usri1020_units_per_week > USHRT_MAX)
//                 {
//                     if (parm_err != NULL)
//                         *parm_err = USER_UNITS_PER_WEEK_PARMNUM;
//                     ApiStatus = ERROR_INVALID_PARAMETER;
//                     break;
//                 }
//
//                 UserAllInfo.LogonHours.UnitsPerWeek = UserInfo1020->usri1020_units_per_week;
//                 UserAllInfo.LogonHours.LogonHours = UserInfo1020->usri1020_logon_hours;
//                 UserAllInfo.WhichFields |= USER_ALL_LOGONHOURS;
//             }
//             break;
//
//         case 1024:
//             UserInfo1024 = (PUSER_INFO_1024)UserInfo;
//
//             UserAllInfo.CountryCode = UserInfo1024->usri1024_country_code;
//             UserAllInfo.WhichFields |= USER_ALL_COUNTRYCODE;
//             break;
//
//         case 1025:
//             UserInfo1025 = (PUSER_INFO_1025)UserInfo;
//
//             UserAllInfo.CodePage = UserInfo1025->usri1025_code_page;
//             UserAllInfo.WhichFields |= USER_ALL_CODEPAGE;
//             break;
//
//         case 1051:
//             UserInfo1051 = (PUSER_INFO_1051)UserInfo;
//
//             UserAllInfo.PrimaryGroupId = UserInfo1051->usri1051_primary_group_id;
//             UserAllInfo.WhichFields |= USER_ALL_PRIMARYGROUPID;
//             break;
//
//         case 1052:
//             UserInfo1052 = (PUSER_INFO_1052)UserInfo;
//
//             if (UserInfo1052->usri1052_profile != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.ProfilePath,
//                                      UserInfo1052->usri1052_profile);
//                 UserAllInfo.WhichFields |= USER_ALL_PROFILEPATH;
//             }
//             break;
//
//         case 1053:
//             UserInfo1053 = (PUSER_INFO_1053)UserInfo;
//
//             if (UserInfo1053->usri1053_home_dir_drive != NULL)
//             {
//                 RtlInitUnicodeString(&UserAllInfo.HomeDirectoryDrive,
//                                      UserInfo1053->usri1053_home_dir_drive);
//                 UserAllInfo.WhichFields |= USER_ALL_HOMEDIRECTORYDRIVE;
//             }
//             break;
//     }
//
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     Status = SamSetInformationUser(UserHandle,
//                                    UserAllInformation,
//                                    &UserAllInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamSetInformationUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
// done:
//     if (Dacl != NULL)
//         HeapFree(GetProcessHeap(), 0, Dacl);
//
//     return ApiStatus;
// }
//
//
// static
// NET_API_STATUS
// OpenUserByName(SAM_HANDLE DomainHandle,
//                PUNICODE_STRING UserName,
//                ULONG DesiredAccess,
//                PSAM_HANDLE UserHandle)
// {
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     1,
//                                     UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", UserName, Status);
//         return NetpNtStatusToApiStatus(Status);
//     }
//
//     /* Fail, if it is not an alias account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("Object is not a user!\n");
//         ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     /* Open the alias account */
//     Status = SamOpenUser(DomainHandle,
//                          DesiredAccess,
//                          RelativeIds[0],
//                          UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
// done:
//     if (RelativeIds != NULL)
//         SamFreeMemory(RelativeIds);
//
//     if (Use != NULL)
//         SamFreeMemory(Use);
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetUserAdd (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserAdd(LPCWSTR servername,
//            DWORD level,
//            LPBYTE bufptr,
//            LPDWORD parm_err)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     ULONG GrantedAccess;
//     ULONG RelativeId;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s, %d, %p, %p)\n", debugstr_w(servername), level, bufptr, parm_err);
//
//     if (parm_err != NULL)
//         *parm_err = PARM_ERROR_NONE;
//
//     /* Check the info level */
//     switch (level)
//     {
//         case 1:
//         case 2:
//         case 3:
//         case 4:
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_CREATE_USER | DOMAIN_LOOKUP | DOMAIN_READ_PASSWORD_PARAMETERS,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Initialize the user name string */
//     RtlInitUnicodeString(&UserName,
//                          ((PUSER_INFO_1)bufptr)->usri1_name);
//
//     /* Create the user account */
//     Status = SamCreateUser2InDomain(DomainHandle,
//                                     &UserName,
//                                     USER_NORMAL_ACCOUNT,
//                                     USER_ALL_ACCESS | DELETE | WRITE_DAC,
//                                     &UserHandle,
//                                     &GrantedAccess,
//                                     &RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamCreateUser2InDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Set user information */
//     ApiStatus = SetUserInfo(UserHandle,
//                             bufptr,
//                             level,
//                             parm_err);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("SetUserInfo failed (Status %lu)\n", ApiStatus);
//         goto done;
//     }
//
// done:
//     if (UserHandle != NULL)
//     {
//         if (ApiStatus != NERR_Success)
//             SamDeleteUser(UserHandle);
//         else
//             SamCloseHandle(UserHandle);
//     }
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
//
// /******************************************************************************
//  *                NetUserChangePassword  (NETAPI32.@)
//  * PARAMS
//  *  domainname  [I] Optional. Domain on which the user resides or the logon
//  *                  domain of the current user if NULL.
//  *  username    [I] Optional. Username to change the password for or the name
//  *                  of the current user if NULL.
//  *  oldpassword [I] The user's current password.
//  *  newpassword [I] The password that the user will be changed to using.
//  *
//  * RETURNS
//  *  Success: NERR_Success.
//  *  Failure: NERR_* failure code or win error code.
//  *
//  */
// NET_API_STATUS
// WINAPI
// NetUserChangePassword(LPCWSTR domainname,
//                       LPCWSTR username,
//                       LPCWSTR oldpassword,
//                       LPCWSTR newpassword)
// {
//     PMSV1_0_CHANGEPASSWORD_REQUEST RequestBuffer = NULL;
//     PMSV1_0_CHANGEPASSWORD_RESPONSE ResponseBuffer = NULL;
//     ULONG RequestBufferSize;
//     ULONG ResponseBufferSize = 0;
//     LPWSTR Ptr;
//     ANSI_STRING PackageName;
//     ULONG AuthenticationPackage = 0;
//     HANDLE LsaHandle = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//     NTSTATUS ProtocolStatus;
//
//     TRACE("(%s, %s, ..., ...)\n", debugstr_w(domainname), debugstr_w(username));
//
//     /* FIXME: handle null domain or user name */
//
//     /* Check the parameters */
//     if ((oldpassword == NULL) ||
//         (newpassword == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     /* Connect to the LSA server */
//     Status = LsaConnectUntrusted(&LsaHandle);
//     if (!NT_SUCCESS(Status))
//         return NetpNtStatusToApiStatus(Status);
//
//     /* Get the authentication package ID */
//     RtlInitAnsiString(&PackageName,
//                       MSV1_0_PACKAGE_NAME);
//
//     Status = LsaLookupAuthenticationPackage(LsaHandle,
//                                             &PackageName,
//                                             &AuthenticationPackage);
//     if (!NT_SUCCESS(Status))
//     {
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Calculate the request buffer size */
//     RequestBufferSize = sizeof(MSV1_0_CHANGEPASSWORD_REQUEST) +
//                         ((wcslen(domainname) + 1) * sizeof(WCHAR)) +
//                         ((wcslen(username) + 1) * sizeof(WCHAR)) +
//                         ((wcslen(oldpassword) + 1) * sizeof(WCHAR)) +
//                         ((wcslen(newpassword) + 1) * sizeof(WCHAR));
//
//     /* Allocate the request buffer */
//     ApiStatus = NetApiBufferAllocate(RequestBufferSize,
//                                      (PVOID*)&RequestBuffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     /* Initialize the request buffer */
//     RequestBuffer->MessageType = MsV1_0ChangePassword;
//     RequestBuffer->Impersonating = TRUE;
//
//     Ptr = (LPWSTR)((ULONG_PTR)RequestBuffer + sizeof(MSV1_0_CHANGEPASSWORD_REQUEST));
//
//     /* Pack the domain name */
//     RequestBuffer->DomainName.Length = wcslen(domainname) * sizeof(WCHAR);
//     RequestBuffer->DomainName.MaximumLength = RequestBuffer->DomainName.Length + sizeof(WCHAR);
//     RequestBuffer->DomainName.Buffer = Ptr;
//
//     RtlCopyMemory(RequestBuffer->DomainName.Buffer,
//                   domainname,
//                   RequestBuffer->DomainName.MaximumLength);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + RequestBuffer->DomainName.MaximumLength);
//
//     /* Pack the user name */
//     RequestBuffer->AccountName.Length = wcslen(username) * sizeof(WCHAR);
//     RequestBuffer->AccountName.MaximumLength = RequestBuffer->AccountName.Length + sizeof(WCHAR);
//     RequestBuffer->AccountName.Buffer = Ptr;
//
//     RtlCopyMemory(RequestBuffer->AccountName.Buffer,
//                   username,
//                   RequestBuffer->AccountName.MaximumLength);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + RequestBuffer->AccountName.MaximumLength);
//
//     /* Pack the old password */
//     RequestBuffer->OldPassword.Length = wcslen(oldpassword) * sizeof(WCHAR);
//     RequestBuffer->OldPassword.MaximumLength = RequestBuffer->OldPassword.Length + sizeof(WCHAR);
//     RequestBuffer->OldPassword.Buffer = Ptr;
//
//     RtlCopyMemory(RequestBuffer->OldPassword.Buffer,
//                   oldpassword,
//                   RequestBuffer->OldPassword.MaximumLength);
//
//     Ptr = (LPWSTR)((ULONG_PTR)Ptr + RequestBuffer->OldPassword.MaximumLength);
//
//     /* Pack the new password */
//     RequestBuffer->NewPassword.Length = wcslen(newpassword) * sizeof(WCHAR);
//     RequestBuffer->NewPassword.MaximumLength = RequestBuffer->NewPassword.Length + sizeof(WCHAR);
//     RequestBuffer->NewPassword.Buffer = Ptr;
//
//     RtlCopyMemory(RequestBuffer->NewPassword.Buffer,
//                   newpassword,
//                   RequestBuffer->NewPassword.MaximumLength);
//
//     /* Call the authentication package */
//     Status = LsaCallAuthenticationPackage(LsaHandle,
//                                           AuthenticationPackage,
//                                           RequestBuffer,
//                                           RequestBufferSize,
//                                           (PVOID*)&ResponseBuffer,
//                                           &ResponseBufferSize,
//                                           &ProtocolStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     if (!NT_SUCCESS(ProtocolStatus))
//     {
//         ApiStatus = NetpNtStatusToApiStatus(ProtocolStatus);
//         goto done;
//     }
//
// done:
//     if (RequestBuffer != NULL)
//         NetApiBufferFree(RequestBuffer);
//
//     if (ResponseBuffer != NULL)
//         LsaFreeReturnBuffer(ResponseBuffer);
//
//     if (LsaHandle != NULL)
//         NtClose(LsaHandle);
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetUserDel  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserDel(LPCWSTR servername,
//            LPCWSTR username)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s, %s)\n", debugstr_w(servername), debugstr_w(username));
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&UserName, username);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Builtin Domain */
//     Status = OpenBuiltinDomain(ServerHandle,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenBuiltinDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the user account in the builtin domain */
//     ApiStatus = OpenUserByName(DomainHandle,
//                                &UserName,
//                                DELETE,
//                                &UserHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         TRACE("OpenUserByName(%wZ) failed (ApiStatus %lu)\n", &UserName, ApiStatus);
//         goto done;
//     }
//
//     if (UserHandle == NULL)
//     {
//         if (DomainHandle != NULL)
//         {
//             SamCloseHandle(DomainHandle);
//             DomainHandle = NULL;
//         }
//
//         /* Open the Acount Domain */
//         Status = OpenAccountDomain(ServerHandle,
//                                    (servername != NULL) ? &ServerName : NULL,
//                                    DOMAIN_LOOKUP,
//                                    &DomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Open the user account in the account domain */
//         ApiStatus = OpenUserByName(DomainHandle,
//                                    &UserName,
//                                    DELETE,
//                                    &UserHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenUserByName(%wZ) failed (ApiStatus %lu)\n", &UserName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_UserNotFound;
//             goto done;
//         }
//     }
//
//     /* Delete the user */
//     Status = SamDeleteUser(UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamDeleteUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* A successful delete invalidates the handle */
//     UserHandle = NULL;
//
// done:
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
// static
// NET_API_STATUS
// AllocateEnumContext(
//     PENUM_CONTEXT *AllocatedEnumContext)
// {
//     NET_API_STATUS ApiStatus;
//     PENUM_CONTEXT EnumContext;
//
//     /* Allocate the context structure */
//     ApiStatus = NetApiBufferAllocate(sizeof(ENUM_CONTEXT), (PVOID*)&EnumContext);
//     if (ApiStatus != NERR_Success)
//         return ApiStatus;
//
//     /* Initialize the fields */
//     EnumContext->EnumerationContext = 0;
//     EnumContext->Buffer = NULL;
//     EnumContext->Count = 0;
//     EnumContext->Index = 0;
//     EnumContext->BuiltinDone = FALSE;
//
//     /* Set a "unique" handle */
//     EnumContext->EnumHandle = InterlockedIncrement(&g_EnumContextHandle);
//     if (EnumContext->EnumHandle == 0)
//     {
//         EnumContext->EnumHandle = InterlockedIncrement(&g_EnumContextHandle);
//     }
//
//     /* Insert the context in the list */
//     EnterCriticalSection(&g_EnumContextListLock);
//     InsertTailList(&g_EnumContextListHead, &EnumContext->ListLink);
//     LeaveCriticalSection(&g_EnumContextListLock);
//
//     *AllocatedEnumContext = EnumContext;
//     return NERR_Success;
// }
//
// static
// VOID
// FreeEnumContext(
//     PENUM_CONTEXT EnumContext)
//
// {
//     /* Remove the context from the list */
//     EnterCriticalSection(&g_EnumContextListLock);
//     RemoveEntryList(&EnumContext->ListLink);
//     LeaveCriticalSection(&g_EnumContextListLock);
//
//     /* Free it */
//     NetApiBufferFree(EnumContext);
// }
//
// static
// PENUM_CONTEXT
// LookupEnumContext(
//     SAM_ENUMERATE_HANDLE EnumerationHandle)
// {
//     PENUM_CONTEXT FoundEnumContext = NULL;
//     PLIST_ENTRY ListEntry;
//
//     /* Acquire the list lock */
//     EnterCriticalSection(&g_EnumContextListLock);
//
//     /* Search the list for the handle */
//     for (ListEntry = g_EnumContextListHead.Flink;
//          ListEntry != &g_EnumContextListHead;
//          ListEntry = ListEntry->Flink)
//     {
//         PENUM_CONTEXT EnumContext = CONTAINING_RECORD(ListEntry, ENUM_CONTEXT, ListLink);
//         if (EnumContext->EnumHandle == EnumerationHandle)
//         {
//             FoundEnumContext = EnumContext;
//             break;
//         }
//     }
//
//     /* Release the list lock */
//     LeaveCriticalSection(&g_EnumContextListLock);
//
//     return FoundEnumContext;
// }
//
// /************************************************************
//  * NetUserEnum  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserEnum(LPCWSTR servername,
//             DWORD level,
//             DWORD filter,
//             LPBYTE* bufptr,
//             DWORD prefmaxlen,
//             LPDWORD entriesread,
//             LPDWORD totalentries,
//             LPDWORD resume_handle)
// {
//     UNICODE_STRING ServerName;
//     PSAM_RID_ENUMERATION CurrentUser;
//     PENUM_CONTEXT EnumContext = NULL;
//     LPVOID Buffer = NULL;
//     ULONG i;
//     SAM_HANDLE UserHandle = NULL;
//     ACCESS_MASK DesiredAccess;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %d 0x%d %p %d %p %p %p)\n", debugstr_w(servername), level,
//           filter, bufptr, prefmaxlen, entriesread, totalentries, resume_handle);
//
//     *entriesread = 0;
//     *totalentries = 0;
//     *bufptr = NULL;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     if (resume_handle != NULL && *resume_handle != 0)
//     {
//         EnumContext = LookupEnumContext(*resume_handle);
//     }
//     else
//     {
//         ApiStatus = AllocateEnumContext(&EnumContext);
//         if (ApiStatus != NERR_Success)
//             goto done;
//
//         Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                             &EnumContext->ServerHandle,
//                             SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                             NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamConnect failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Get the Account Domain SID */
//         Status = GetAccountDomainSid((servername != NULL) ? &ServerName : NULL,
//                                      &EnumContext->AccountDomainSid);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Open the Account Domain */
//         Status = SamOpenDomain(EnumContext->ServerHandle,
//                                DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP,
//                                EnumContext->AccountDomainSid,
//                                &EnumContext->AccountDomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Get the Builtin Domain SID */
//         Status = GetBuiltinDomainSid(&EnumContext->BuiltinDomainSid);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("GetBuiltinDomainSid failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         DesiredAccess = DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP;
//         if ((level == 1) || (level == 2) || (level == 3) || (level == 4) || (level == 11))
//             DesiredAccess |= DOMAIN_GET_ALIAS_MEMBERSHIP;
//
//         /* Open the Builtin Domain */
//         Status = SamOpenDomain(EnumContext->ServerHandle,
//                                DesiredAccess,
//                                EnumContext->BuiltinDomainSid,
//                                &EnumContext->BuiltinDomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//     }
//
// //    while (TRUE)
// //    {
//         TRACE("EnumContext->Index: %lu\n", EnumContext->Index);
//         TRACE("EnumContext->Count: %lu\n", EnumContext->Count);
//
//         if (EnumContext->Index >= EnumContext->Count)
//         {
// //            if (EnumContext->BuiltinDone != FALSE)
// //            {
// //                ApiStatus = NERR_Success;
// //                goto done;
// //            }
//
//             TRACE("Calling SamEnumerateUsersInDomain\n");
//             Status = SamEnumerateUsersInDomain(EnumContext->AccountDomainHandle, //BuiltinDomainHandle,
//                                                &EnumContext->EnumerationContext,
//                                                0,
//                                                (PVOID *)&EnumContext->Buffer,
//                                                prefmaxlen,
//                                                &EnumContext->Count);
//
//             TRACE("SamEnumerateUsersInDomain returned (Status %08lx)\n", Status);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamEnumerateUsersInDomain failed (Status %08lx)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             if (Status == STATUS_MORE_ENTRIES)
//             {
//                 ApiStatus = NERR_BufTooSmall;
//                 goto done;
//             }
//             else
//             {
//                 EnumContext->BuiltinDone = TRUE;
//             }
//         }
//
//         TRACE("EnumContext: %lu\n", EnumContext);
//         TRACE("EnumContext->Count: %lu\n", EnumContext->Count);
//         TRACE("EnumContext->Buffer: %p\n", EnumContext->Buffer);
//
//         /* Get a pointer to the current user */
//         CurrentUser = &EnumContext->Buffer[EnumContext->Index];
//
//         TRACE("RID: %lu\n", CurrentUser->RelativeId);
//
//         DesiredAccess = READ_CONTROL | USER_READ_GENERAL | USER_READ_PREFERENCES | USER_READ_LOGON | USER_READ_ACCOUNT;
//         if ((level == 1) || (level == 2) || (level == 3) || (level == 4) || (level == 11))
//             DesiredAccess |= USER_LIST_GROUPS;
//
//         Status = SamOpenUser(EnumContext->AccountDomainHandle, //BuiltinDomainHandle,
//                              DesiredAccess,
//                              CurrentUser->RelativeId,
//                              &UserHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamOpenUser failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         ApiStatus = BuildUserInfoBuffer(EnumContext->BuiltinDomainHandle,
//                                         UserHandle,
//                                         EnumContext->AccountDomainSid,
//                                         CurrentUser->RelativeId,
//                                         level,
//                                         &Buffer);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("BuildUserInfoBuffer failed (ApiStatus %lu)\n", ApiStatus);
//             goto done;
//         }
//
//         SamCloseHandle(UserHandle);
//         UserHandle = NULL;
//
//         EnumContext->Index++;
//
//         (*entriesread)++;
// //    }
//
// done:
//     if (ApiStatus == NERR_Success && EnumContext != NULL && EnumContext->Index < EnumContext->Count)
//         ApiStatus = ERROR_MORE_DATA;
//
//     if (EnumContext != NULL)
//         *totalentries = EnumContext->Count;
//
//     if (resume_handle == NULL || ApiStatus != ERROR_MORE_DATA)
//     {
//         if (EnumContext != NULL)
//         {
//             if (EnumContext->BuiltinDomainHandle != NULL)
//                 SamCloseHandle(EnumContext->BuiltinDomainHandle);
//
//             if (EnumContext->AccountDomainHandle != NULL)
//                 SamCloseHandle(EnumContext->AccountDomainHandle);
//
//             if (EnumContext->BuiltinDomainSid != NULL)
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, EnumContext->BuiltinDomainSid);
//
//             if (EnumContext->AccountDomainSid != NULL)
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, EnumContext->AccountDomainSid);
//
//             if (EnumContext->ServerHandle != NULL)
//                 SamCloseHandle(EnumContext->ServerHandle);
//
//             if (EnumContext->Buffer != NULL)
//             {
//                 for (i = 0; i < EnumContext->Count; i++)
//                 {
//                     SamFreeMemory(EnumContext->Buffer[i].Name.Buffer);
//                 }
//
//                 SamFreeMemory(EnumContext->Buffer);
//             }
//
//             FreeEnumContext(EnumContext);
//             EnumContext = NULL;
//         }
//     }
//
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (resume_handle != NULL)
//         *resume_handle = EnumContext ? EnumContext->EnumHandle : 0;
//
//     *bufptr = (LPBYTE)Buffer;
//
//     TRACE("return %lu\n", ApiStatus);
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetUserGetGroups (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserGetGroups(LPCWSTR servername,
//                  LPCWSTR username,
//                  DWORD level,
//                  LPBYTE *bufptr,
//                  DWORD prefixmaxlen,
//                  LPDWORD entriesread,
//                  LPDWORD totalentries)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE AccountDomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     PSID AccountDomainSid = NULL;
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     PGROUP_MEMBERSHIP GroupMembership = NULL;
//     ULONG GroupCount;
//
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("%s %s %d %p %d %p %p stub\n", debugstr_w(servername),
//           debugstr_w(username), level, bufptr, prefixmaxlen, entriesread,
//           totalentries);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&UserName, username);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the Account Domain SID */
//     Status = GetAccountDomainSid((servername != NULL) ? &ServerName : NULL,
//                                  &AccountDomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP,
//                            AccountDomainSid,
//                            &AccountDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(AccountDomainHandle,
//                                     1,
//                                     &UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", &UserName, Status);
//         if (Status == STATUS_NONE_MAPPED)
//             ApiStatus = NERR_UserNotFound;
//         else
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("Account is not a User!\n");
//         ApiStatus = NERR_UserNotFound;
//         goto done;
//     }
//
//     /* Open the user object */
//     Status = SamOpenUser(AccountDomainHandle,
//                          USER_LIST_GROUPS,
//                          RelativeIds[0],
//                          &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the group memberships of this user */
//     Status = SamGetGroupsForUser(UserHandle,
//                                  &GroupMembership,
//                                  &GroupCount);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetGroupsForUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* If there is no group membership, we're done */
//     if (GroupCount == 0)
//     {
//         ApiStatus = NERR_Success;
//         goto done;
//     }
//
//
// done:
//
//     if (GroupMembership != NULL)
//         SamFreeMemory(GroupMembership);
//
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (RelativeIds != NULL)
//         SamFreeMemory(RelativeIds);
//
//     if (Use != NULL)
//         SamFreeMemory(Use);
//
//     if (AccountDomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid);
//
//     if (AccountDomainHandle != NULL)
//         SamCloseHandle(AccountDomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_MORE_DATA)
//     {
//         *entriesread = 0;
//         *totalentries = 0;
//     }
//     else
//     {
// //        *entriesread = Count;
// //        *totalentries = Count;
//     }
//
// //    *bufptr = (LPBYTE)Buffer;
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetUserGetInfo  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserGetInfo(LPCWSTR servername,
//                LPCWSTR username,
//                DWORD level,
//                LPBYTE* bufptr)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE AccountDomainHandle = NULL;
//     SAM_HANDLE BuiltinDomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     LPVOID Buffer = NULL;
//     PSID AccountDomainSid = NULL;
//     PSID BuiltinDomainSid = NULL;
//     ACCESS_MASK DesiredAccess;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s, %s, %d, %p)\n", debugstr_w(servername),
//           debugstr_w(username), level, bufptr);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&UserName, username);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the Builtin Domain SID */
//     Status = GetBuiltinDomainSid(&BuiltinDomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetBuiltinDomainSid failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     DesiredAccess = DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP;
//     if ((level == 1) || (level == 2) || (level == 3) || (level == 4) || (level == 11))
//         DesiredAccess |= DOMAIN_GET_ALIAS_MEMBERSHIP;
//
//     /* Open the Builtin Domain */
//     Status = SamOpenDomain(ServerHandle,
//                            DesiredAccess,
//                            BuiltinDomainSid,
//                            &BuiltinDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the Account Domain SID */
//     Status = GetAccountDomainSid((servername != NULL) ? &ServerName : NULL,
//                                  &AccountDomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP,
//                            AccountDomainSid,
//                            &AccountDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(AccountDomainHandle,
//                                     1,
//                                     &UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", &UserName, Status);
//         if (Status == STATUS_NONE_MAPPED)
//             ApiStatus = NERR_UserNotFound;
//         else
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Check if the account is a user account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("No user found!\n");
//         ApiStatus = NERR_UserNotFound;
//         goto done;
//     }
//
//     TRACE("RID: %lu\n", RelativeIds[0]);
//
//     DesiredAccess = READ_CONTROL | USER_READ_GENERAL | USER_READ_PREFERENCES | USER_READ_LOGON | USER_READ_ACCOUNT;
//     if ((level == 1) || (level == 2) || (level == 3) || (level == 4) || (level == 11))
//         DesiredAccess |= USER_LIST_GROUPS;
//
//     /* Open the user object */
//     Status = SamOpenUser(AccountDomainHandle,
//                          DesiredAccess,
//                          RelativeIds[0],
//                          &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenUser failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     ApiStatus = BuildUserInfoBuffer(BuiltinDomainHandle,
//                                     UserHandle,
//                                     AccountDomainSid,
//                                     RelativeIds[0],
//                                     level,
//                                     &Buffer);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("BuildUserInfoBuffer failed (ApiStatus %08lu)\n", ApiStatus);
//         goto done;
//     }
//
// done:
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (RelativeIds != NULL)
//         SamFreeMemory(RelativeIds);
//
//     if (Use != NULL)
//         SamFreeMemory(Use);
//
//     if (AccountDomainHandle != NULL)
//         SamCloseHandle(AccountDomainHandle);
//
//     if (AccountDomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid);
//
//     if (BuiltinDomainHandle != NULL)
//         SamCloseHandle(BuiltinDomainHandle);
//
//     if (BuiltinDomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, BuiltinDomainSid);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     *bufptr = (LPBYTE)Buffer;
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetUserGetLocalGroups  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserGetLocalGroups(LPCWSTR servername,
//                       LPCWSTR username,
//                       DWORD level,
//                       DWORD flags,
//                       LPBYTE* bufptr,
//                       DWORD prefmaxlen,
//                       LPDWORD entriesread,
//                       LPDWORD totalentries)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE BuiltinDomainHandle = NULL;
//     SAM_HANDLE AccountDomainHandle = NULL;
//     PSID AccountDomainSid = NULL;
//     PSID UserSid = NULL;
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     ULONG BuiltinMemberCount = 0;
//     ULONG AccountMemberCount = 0;
//     PULONG BuiltinAliases = NULL;
//     PULONG AccountAliases = NULL;
//     PUNICODE_STRING BuiltinNames = NULL;
//     PUNICODE_STRING AccountNames = NULL;
//     PLOCALGROUP_USERS_INFO_0 Buffer = NULL;
//     ULONG Size;
//     ULONG Count = 0;
//     ULONG Index;
//     ULONG i;
//     LPWSTR StrPtr;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s, %s, %d, %08x, %p %d, %p, %p) stub!\n",
//           debugstr_w(servername), debugstr_w(username), level, flags, bufptr,
//           prefmaxlen, entriesread, totalentries);
//
//     if (level != 0)
//         return ERROR_INVALID_LEVEL;
//
//     if (flags & ~LG_INCLUDE_INDIRECT)
//         return ERROR_INVALID_PARAMETER;
//
//     if (flags & LG_INCLUDE_INDIRECT)
//     {
//         WARN("The flag LG_INCLUDE_INDIRECT is not supported yet!\n");
//     }
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&UserName, username);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Builtin Domain */
//     Status = OpenBuiltinDomain(ServerHandle,
//                                DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP,
//                                &BuiltinDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenBuiltinDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the Account Domain SID */
//     Status = GetAccountDomainSid((servername != NULL) ? &ServerName : NULL,
//                                  &AccountDomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = SamOpenDomain(ServerHandle,
//                            DOMAIN_LOOKUP | DOMAIN_GET_ALIAS_MEMBERSHIP,
//                            AccountDomainSid,
//                            &AccountDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(AccountDomainHandle,
//                                     1,
//                                     &UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", &UserName, Status);
//         if (Status == STATUS_NONE_MAPPED)
//             ApiStatus = NERR_UserNotFound;
//         else
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("Account is not a User!\n");
//         ApiStatus = NERR_UserNotFound;
//         goto done;
//     }
//
//     /* Build the User SID from the Account Domain SID and the users RID */
//     ApiStatus = BuildSidFromSidAndRid(AccountDomainSid,
//                                       RelativeIds[0],
//                                       &UserSid);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("BuildSidFromSidAndRid failed!\n");
//         goto done;
//     }
//
//     /* Get alias memberships in the Builtin Domain */
//     Status = SamGetAliasMembership(BuiltinDomainHandle,
//                                    1,
//                                    &UserSid,
//                                    &BuiltinMemberCount,
//                                    &BuiltinAliases);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetAliasMembership failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     if (BuiltinMemberCount > 0)
//     {
//         /* Get the Names of the builtin alias members */
//         Status = SamLookupIdsInDomain(BuiltinDomainHandle,
//                                       BuiltinMemberCount,
//                                       BuiltinAliases,
//                                       &BuiltinNames,
//                                       NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamLookupIdsInDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//     }
//
//     /* Get alias memberships in the Account Domain */
//     Status = SamGetAliasMembership(AccountDomainHandle,
//                                    1,
//                                    &UserSid,
//                                    &AccountMemberCount,
//                                    &AccountAliases);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetAliasMembership failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     if (AccountMemberCount > 0)
//     {
//         /* Get the Names of the builtin alias members */
//         Status = SamLookupIdsInDomain(AccountDomainHandle,
//                                       AccountMemberCount,
//                                       AccountAliases,
//                                       &AccountNames,
//                                       NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamLookupIdsInDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//     }
//
//     /* Calculate the required buffer size */
//     Size = 0;
//
//     for (i = 0; i < BuiltinMemberCount; i++)
//     {
//         if (BuiltinNames[i].Length > 0)
//         {
//             Size += (sizeof(LOCALGROUP_USERS_INFO_0) + BuiltinNames[i].Length + sizeof(UNICODE_NULL));
//             Count++;
//         }
//     }
//
//     for (i = 0; i < AccountMemberCount; i++)
//     {
//         if (AccountNames[i].Length > 0)
//         {
//             Size += (sizeof(LOCALGROUP_USERS_INFO_0) + AccountNames[i].Length + sizeof(UNICODE_NULL));
//             Count++;
//         }
//     }
//
//     if (Size == 0)
//     {
//         ApiStatus = NERR_Success;
//         goto done;
//     }
//
//     /* Allocate buffer */
//     ApiStatus = NetApiBufferAllocate(Size, (LPVOID*)&Buffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     ZeroMemory(Buffer, Size);
//
//     StrPtr = (LPWSTR)((INT_PTR)Buffer + Count * sizeof(LOCALGROUP_USERS_INFO_0));
//
//     /* Copy data to the allocated buffer */
//     Index = 0;
//     for (i = 0; i < BuiltinMemberCount; i++)
//     {
//         if (BuiltinNames[i].Length > 0)
//         {
//             CopyMemory(StrPtr,
//                        BuiltinNames[i].Buffer,
//                        BuiltinNames[i].Length);
//             Buffer[Index].lgrui0_name = StrPtr;
//
//             StrPtr = (LPWSTR)((INT_PTR)StrPtr + BuiltinNames[i].Length + sizeof(UNICODE_NULL));
//             Index++;
//         }
//     }
//
//     for (i = 0; i < AccountMemberCount; i++)
//     {
//         if (AccountNames[i].Length > 0)
//         {
//             CopyMemory(StrPtr,
//                        AccountNames[i].Buffer,
//                        AccountNames[i].Length);
//             Buffer[Index].lgrui0_name = StrPtr;
//
//             StrPtr = (LPWSTR)((INT_PTR)StrPtr + AccountNames[i].Length + sizeof(UNICODE_NULL));
//             Index++;
//         }
//     }
//
// done:
//     if (AccountNames != NULL)
//         SamFreeMemory(AccountNames);
//
//     if (BuiltinNames != NULL)
//         SamFreeMemory(BuiltinNames);
//
//     if (AccountAliases != NULL)
//         SamFreeMemory(AccountAliases);
//
//     if (BuiltinAliases != NULL)
//         SamFreeMemory(BuiltinAliases);
//
//     if (RelativeIds != NULL)
//         SamFreeMemory(RelativeIds);
//
//     if (Use != NULL)
//         SamFreeMemory(Use);
//
//     if (UserSid != NULL)
//         NetApiBufferFree(UserSid);
//
//     if (AccountDomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid);
//
//     if (AccountDomainHandle != NULL)
//         SamCloseHandle(AccountDomainHandle);
//
//     if (BuiltinDomainHandle != NULL)
//         SamCloseHandle(BuiltinDomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_MORE_DATA)
//     {
//         *entriesread = 0;
//         *totalentries = 0;
//     }
//     else
//     {
//         *entriesread = Count;
//         *totalentries = Count;
//     }
//
//     *bufptr = (LPBYTE)Buffer;
//
//     return ApiStatus;
// }
//
//
// /******************************************************************************
//  * NetUserModalsGet  (NETAPI32.@)
//  *
//  * Retrieves global information for all users and global groups in the security
//  * database.
//  *
//  * PARAMS
//  *  servername [I] Specifies the DNS or the NetBIOS name of the remote server
//  *                 on which the function is to execute.
//  *  level      [I] Information level of the data.
//  *     0   Return global passwords parameters. bufptr points to a
//  *         USER_MODALS_INFO_0 struct.
//  *     1   Return logon server and domain controller information. bufptr
//  *         points to a USER_MODALS_INFO_1 struct.
//  *     2   Return domain name and identifier. bufptr points to a 
//  *         USER_MODALS_INFO_2 struct.
//  *     3   Return lockout information. bufptr points to a USER_MODALS_INFO_3
//  *         struct.
//  *  bufptr     [O] Buffer that receives the data.
//  *
//  * RETURNS
//  *  Success: NERR_Success.
//  *  Failure: 
//  *     ERROR_ACCESS_DENIED - the user does not have access to the info.
//  *     NERR_InvalidComputer - computer name is invalid.
//  */
// NET_API_STATUS
// WINAPI
// NetUserModalsGet(LPCWSTR servername,
//                  DWORD level,
//                  LPBYTE *bufptr)
// {
//     UNICODE_STRING ServerName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     PSID DomainSid = NULL;
//     PDOMAIN_PASSWORD_INFORMATION PasswordInfo = NULL;
//     PDOMAIN_LOGOFF_INFORMATION LogoffInfo = NULL;
//     PDOMAIN_SERVER_ROLE_INFORMATION ServerRoleInfo = NULL;
//     PDOMAIN_REPLICATION_INFORMATION ReplicationInfo = NULL;
//     PDOMAIN_NAME_INFORMATION NameInfo = NULL;
//     PDOMAIN_LOCKOUT_INFORMATION LockoutInfo = NULL;
//     ULONG DesiredAccess;
//     ULONG BufferSize;
//     PUSER_MODALS_INFO_0 umi0;
//     PUSER_MODALS_INFO_1 umi1;
//     PUSER_MODALS_INFO_2 umi2;
//     PUSER_MODALS_INFO_3 umi3;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %d %p)\n", debugstr_w(servername), level, bufptr);
//
//     *bufptr = NULL;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Get the Account Domain SID */
//     Status = GetAccountDomainSid((servername != NULL) ? &ServerName : NULL,
//                                  &DomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     switch (level)
//     {
//         case 0:
//             DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS | DOMAIN_READ_PASSWORD_PARAMETERS;
//             break;
//
//         case 1:
//             DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS;
//             break;
//
//         case 2:
//             DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS;
//             break;
//
//         case 3:
//             DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS;
//             break;
//
//         default:
//             ApiStatus = ERROR_INVALID_LEVEL;
//             goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = SamOpenDomain(ServerHandle,
//                            DesiredAccess,
//                            DomainSid,
//                            &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     switch (level)
//     {
//         case 0:
//             /* return global passwords parameters */
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainPasswordInformation,
//                                                (PVOID*)&PasswordInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainLogoffInformation,
//                                                (PVOID*)&LogoffInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             BufferSize = sizeof(USER_MODALS_INFO_0);
//             break;
//
//         case 1:
//             /* return logon server and domain controller info */
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainServerRoleInformation,
//                                                (PVOID*)&ServerRoleInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainReplicationInformation,
//                                                (PVOID*)&ReplicationInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             BufferSize = sizeof(USER_MODALS_INFO_1) +
//                          ReplicationInfo->ReplicaSourceNodeName.Length + sizeof(WCHAR);
//             break;
//
//         case 2:
//             /* return domain name and identifier */
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainNameInformation,
//                                                (PVOID*)&NameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             BufferSize = sizeof( USER_MODALS_INFO_2 ) +
//                          NameInfo->DomainName.Length + sizeof(WCHAR) +
//                          RtlLengthSid(DomainSid);
//             break;
//
//         case 3:
//             /* return lockout information */
//             Status = SamQueryInformationDomain(DomainHandle,
//                                                DomainLockoutInformation,
//                                                (PVOID*)&LockoutInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             BufferSize = sizeof(USER_MODALS_INFO_3);
//             break;
//
//         default:
//             TRACE("Invalid level %d is specified\n", level);
//             ApiStatus = ERROR_INVALID_LEVEL;
//             goto done;
//     }
//
//
//     ApiStatus = NetApiBufferAllocate(BufferSize,
//                                      (LPVOID *)bufptr);
//     if (ApiStatus != NERR_Success)
//     {
//         WARN("NetApiBufferAllocate() failed\n");
//         goto done;
//     }
//
//     switch (level)
//     {
//         case 0:
//             umi0 = (PUSER_MODALS_INFO_0)*bufptr;
//
//             umi0->usrmod0_min_passwd_len = PasswordInfo->MinPasswordLength;
//             umi0->usrmod0_max_passwd_age = (ULONG)(-PasswordInfo->MaxPasswordAge.QuadPart / 10000000);
//             umi0->usrmod0_min_passwd_age =
//                 DeltaTimeToSeconds(PasswordInfo->MinPasswordAge);
//             umi0->usrmod0_force_logoff =
//                 DeltaTimeToSeconds(LogoffInfo->ForceLogoff);
//             umi0->usrmod0_password_hist_len = PasswordInfo->PasswordHistoryLength;
//             break;
//
//         case 1:
//             umi1 = (PUSER_MODALS_INFO_1)*bufptr;
//
//             switch (ServerRoleInfo->DomainServerRole)
//             {
//                 case DomainServerRolePrimary:
//                     umi1->usrmod1_role = UAS_ROLE_PRIMARY;
//                     break;
//
//                 case DomainServerRoleBackup:
//                     umi1->usrmod1_role = UAS_ROLE_BACKUP;
//                     break;
//
//                 default:
//                     ApiStatus = NERR_InternalError;
//                     goto done;
//             }
//
//             umi1->usrmod1_primary = (LPWSTR)(*bufptr + sizeof(USER_MODALS_INFO_1));
//             RtlCopyMemory(umi1->usrmod1_primary,
//                           ReplicationInfo->ReplicaSourceNodeName.Buffer,
//                           ReplicationInfo->ReplicaSourceNodeName.Length);
//             umi1->usrmod1_primary[ReplicationInfo->ReplicaSourceNodeName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 2:
//             umi2 = (PUSER_MODALS_INFO_2)*bufptr;
//
//             umi2->usrmod2_domain_name = (LPWSTR)(*bufptr + sizeof(USER_MODALS_INFO_2));
//             RtlCopyMemory(umi2->usrmod2_domain_name,
//                           NameInfo->DomainName.Buffer,
//                           NameInfo->DomainName.Length);
//             umi2->usrmod2_domain_name[NameInfo->DomainName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             umi2->usrmod2_domain_id = *bufptr +
//                                       sizeof(USER_MODALS_INFO_2) +
//                                       NameInfo->DomainName.Length + sizeof(WCHAR);
//             RtlCopyMemory(umi2->usrmod2_domain_id,
//                           DomainSid,
//                           RtlLengthSid(DomainSid));
//             break;
//
//         case 3:
//             umi3 = (PUSER_MODALS_INFO_3)*bufptr;
//             umi3->usrmod3_lockout_duration =
//                 DeltaTimeToSeconds(LockoutInfo->LockoutDuration);
//             umi3->usrmod3_lockout_observation_window =
//                 DeltaTimeToSeconds(LockoutInfo->LockoutObservationWindow );
//             umi3->usrmod3_lockout_threshold = LockoutInfo->LockoutThreshold;
//             break;
//     }
//
// done:
//     if (LockoutInfo != NULL)
//         SamFreeMemory(LockoutInfo);
//
//     if (NameInfo != NULL)
//         SamFreeMemory(NameInfo);
//
//     if (ReplicationInfo != NULL)
//         SamFreeMemory(ReplicationInfo);
//
//     if (ServerRoleInfo != NULL)
//         SamFreeMemory(ServerRoleInfo);
//
//     if (LogoffInfo != NULL)
//         SamFreeMemory(LogoffInfo);
//
//     if (PasswordInfo != NULL)
//         SamFreeMemory(PasswordInfo);
//
//     if (DomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
//
// /******************************************************************************
//  * NetUserModalsSet  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserModalsSet( LPCWSTR servername,
//                   DWORD level,
//                   LPBYTE buf,
//                   LPDWORD parm_err)
// {
//     FIXME("(%s %d %p %p)\n", debugstr_w(servername), level, buf, parm_err);
//     return ERROR_ACCESS_DENIED;
// }
//
//
// /******************************************************************************
//  * NetUserSetGroups  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserSetGroups(LPCWSTR servername,
//                  LPCWSTR username,
//                  DWORD level,
//                  LPBYTE buf,
//                  DWORD num_entries)
// {
//     FIXME("(%s %s %lu %p %lu)\n",
//           debugstr_w(servername), debugstr_w(username), level, buf, num_entries);
//     return ERROR_ACCESS_DENIED;
// }
//
//
// /******************************************************************************
//  * NetUserSetInfo  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetUserSetInfo(LPCWSTR servername,
//                LPCWSTR username,
//                DWORD level,
//                LPBYTE buf,
//                LPDWORD parm_err)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE AccountDomainHandle = NULL;
//     SAM_HANDLE UserHandle = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %lu %p %p)\n",
//           debugstr_w(servername), debugstr_w(username), level, buf, parm_err);
//
//     if (parm_err != NULL)
//         *parm_err = PARM_ERROR_NONE;
//
//     /* Check the info level */
//     switch (level)
//     {
//         case 0:
//         case 1:
//         case 2:
//         case 3:
//         case 4:
// //        case 21:
//         case 22:
//         case 1003:
// //        case 1005:
//         case 1006:
//         case 1007:
//         case 1008:
//         case 1009:
// //        case 1010:
//         case 1011:
//         case 1012:
//         case 1013:
//         case 1014:
//         case 1017:
//         case 1018:
//         case 1020:
//         case 1024:
//         case 1025:
//         case 1051:
//         case 1052:
//         case 1053:
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&UserName, username);
//
//     /* Connect to the SAM Server */
//     Status = SamConnect((servername != NULL) ? &ServerName : NULL,
//                         &ServerHandle,
//                         SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
//                         NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamConnect failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the Account Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP | DOMAIN_READ_PASSWORD_PARAMETERS,
//                                &AccountDomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the User Account */
//     ApiStatus = OpenUserByName(AccountDomainHandle,
//                                &UserName,
//                                USER_ALL_ACCESS,
//                                &UserHandle);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenUserByName(%wZ) failed (ApiStatus %lu)\n", &UserName, ApiStatus);
//         goto done;
//     }
//
//     /* Set user information */
//     ApiStatus = SetUserInfo(UserHandle,
//                             buf,
//                             level,
//                             parm_err);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("SetUserInfo failed (Status %lu)\n", ApiStatus);
//     }
//
// done:
//     if (UserHandle != NULL)
//         SamCloseHandle(UserHandle);
//
//     if (AccountDomainHandle != NULL)
//         SamCloseHandle(AccountDomainHandle);
//
//     if (ServerHandle != NULL)
//         SamCloseHandle(ServerHandle);
//
//     return ApiStatus;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
