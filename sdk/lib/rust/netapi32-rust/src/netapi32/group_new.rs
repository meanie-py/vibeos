use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
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

// _ENUM_PHASE was a file-local C `enum { BuiltinPhase, AccountPhase, DonePhase }` (int-sized).
// Modelled as an int alias + bare consts so the values work as assignments, comparisons, and
// `match` patterns (C-layout compatible: a C enum is `int`).
pub type _ENUM_PHASE = u32;
pub const BuiltinPhase: _ENUM_PHASE = 0;
pub const AccountPhase: _ENUM_PHASE = 1;
pub const DonePhase: _ENUM_PHASE = 2;
pub type ENUM_PHASE = _ENUM_PHASE;

#[repr(C)]
pub struct _GROUP_ENUM_CONTEXT {
    pub ServerHandle: SAM_HANDLE,
    pub DomainHandle: SAM_HANDLE,
    pub BuiltinDomainHandle: SAM_HANDLE,
    pub AccountDomainHandle: SAM_HANDLE,
    pub EnumerationContext: SAM_ENUMERATE_HANDLE,
    pub Buffer: PSAM_RID_ENUMERATION,
    pub Returned: ULONG,
    pub Index: ULONG,
    pub Phase: ENUM_PHASE,
}
pub type GROUP_ENUM_CONTEXT = _GROUP_ENUM_CONTEXT;
pub type PGROUP_ENUM_CONTEXT = *mut _GROUP_ENUM_CONTEXT;

#[repr(C)]
pub struct _USER_ENUM_CONTEXT {
    pub ServerHandle: SAM_HANDLE,
    pub DomainHandle: SAM_HANDLE,
    pub GroupHandle: SAM_HANDLE,
    pub MemberCount: ULONG,
    pub MemberIds: PULONG,
    pub Attributes: PULONG,
    pub Names: PUNICODE_STRING,
    pub Start: ULONG,
    pub Count: ULONG,
}
pub type USER_ENUM_CONTEXT = _USER_ENUM_CONTEXT;
pub type PUSER_ENUM_CONTEXT = *mut _USER_ENUM_CONTEXT;
unsafe extern "C" fn BuildGroupInfoBuffer(
    GroupInfo: *mut _GROUP_GENERAL_INFORMATION,
    Level: u32,
    GroupId: u32,
    Buffer: *mut *mut core::ffi::c_void,
) -> u32 {
    let mut GroupBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut GroupInfo0: *mut _GROUP_INFO_0;
    let mut GroupInfo1: *mut _GROUP_INFO_1;
    let mut GroupInfo2: *mut _GROUP_INFO_2;
    let mut GroupInfo3: *mut _GROUP_INFO_3;
    let mut Ptr: *mut u16;
    let mut Size: usize = 0;
    let mut ApiStatus: u32 = NERR_Success;

    *Buffer = core::ptr::null_mut();

    match Level {
        0 => {
            Size = core::mem::size_of::<_GROUP_INFO_0>()
                + (*GroupInfo).Name.Length as usize
                + core::mem::size_of::<u16>();
        }
        1 => {
            Size = core::mem::size_of::<_GROUP_INFO_1>()
                + (*GroupInfo).Name.Length as usize
                + core::mem::size_of::<u16>()
                + (*GroupInfo).AdminComment.Length as usize
                + core::mem::size_of::<u16>();
        }
        2 => {
            Size = core::mem::size_of::<_GROUP_INFO_2>()
                + (*GroupInfo).Name.Length as usize
                + core::mem::size_of::<u16>()
                + (*GroupInfo).AdminComment.Length as usize
                + core::mem::size_of::<u16>();
        }
        3 => {
            Size = core::mem::size_of::<_GROUP_INFO_3>()
                + (*GroupInfo).Name.Length as usize
                + core::mem::size_of::<u16>()
                + (*GroupInfo).AdminComment.Length as usize
                + core::mem::size_of::<u16>();
        }
        _ => {
            ApiStatus = ERROR_INVALID_LEVEL;
        }
    }

    if ApiStatus == NERR_Success {
        ApiStatus = NetApiBufferAllocate(Size as u32, &raw mut GroupBuffer);
        if ApiStatus == NERR_Success {
            ZeroMemory(GroupBuffer, Size);

            match Level {
                0 => {
                    GroupInfo0 = GroupBuffer as *mut _GROUP_INFO_0;
                    Ptr = (GroupInfo0 as usize)
                        .wrapping_add(core::mem::size_of::<LOCALGROUP_INFO_0>()) as *mut u16;
                    (*GroupInfo0).grpi0_name = Ptr;
                    memcpy(
                        (*GroupInfo0).grpi0_name as *mut core::ffi::c_void,
                        (*GroupInfo).Name.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).Name.Length as usize,
                    );
                    *(*GroupInfo0).grpi0_name
                        .add((*GroupInfo).Name.Length as usize / core::mem::size_of::<u16>()) = 0u16;
                }
                1 => {
                    GroupInfo1 = GroupBuffer as *mut _GROUP_INFO_1;
                    Ptr = (GroupInfo1 as usize)
                        .wrapping_add(core::mem::size_of::<_GROUP_INFO_1>()) as *mut u16;
                    (*GroupInfo1).grpi1_name = Ptr;
                    memcpy(
                        (*GroupInfo1).grpi1_name as *mut core::ffi::c_void,
                        (*GroupInfo).Name.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).Name.Length as usize,
                    );
                    *(*GroupInfo1).grpi1_name
                        .add((*GroupInfo).Name.Length as usize / core::mem::size_of::<u16>()) = 0u16;

                    Ptr = (Ptr as usize)
                        .wrapping_add((*GroupInfo).Name.Length as usize + core::mem::size_of::<u16>()) as *mut u16;
                    (*GroupInfo1).grpi1_comment = Ptr;
                    memcpy(
                        (*GroupInfo1).grpi1_comment as *mut core::ffi::c_void,
                        (*GroupInfo).AdminComment.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).AdminComment.Length as usize,
                    );
                    *(*GroupInfo1).grpi1_comment
                        .add((*GroupInfo).AdminComment.Length as usize / core::mem::size_of::<u16>()) = 0u16;
                }
                2 => {
                    GroupInfo2 = GroupBuffer as *mut _GROUP_INFO_2;
                    Ptr = (GroupInfo2 as usize)
                        .wrapping_add(core::mem::size_of::<_GROUP_INFO_2>()) as *mut u16;
                    (*GroupInfo2).grpi2_name = Ptr;
                    memcpy(
                        (*GroupInfo2).grpi2_name as *mut core::ffi::c_void,
                        (*GroupInfo).Name.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).Name.Length as usize,
                    );
                    *(*GroupInfo2).grpi2_name
                        .add((*GroupInfo).Name.Length as usize / core::mem::size_of::<u16>()) = 0u16;

                    Ptr = (Ptr as usize)
                        .wrapping_add((*GroupInfo).Name.Length as usize + core::mem::size_of::<u16>()) as *mut u16;
                    (*GroupInfo2).grpi2_comment = Ptr;
                    memcpy(
                        (*GroupInfo2).grpi2_comment as *mut core::ffi::c_void,
                        (*GroupInfo).AdminComment.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).AdminComment.Length as usize,
                    );
                    *(*GroupInfo2).grpi2_comment
                        .add((*GroupInfo).AdminComment.Length as usize / core::mem::size_of::<u16>()) = 0u16;

                    (*GroupInfo2).grpi2_group_id = GroupId;
                    (*GroupInfo2).grpi2_attributes = (*GroupInfo).Attributes as u32;
                }
                3 => {
                    GroupInfo3 = GroupBuffer as *mut _GROUP_INFO_3;
                    Ptr = (GroupInfo3 as usize)
                        .wrapping_add(core::mem::size_of::<_GROUP_INFO_3>()) as *mut u16;
                    (*GroupInfo3).grpi3_name = Ptr;
                    memcpy(
                        (*GroupInfo3).grpi3_name as *mut core::ffi::c_void,
                        (*GroupInfo).Name.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).Name.Length as usize,
                    );
                    *(*GroupInfo3).grpi3_name
                        .add((*GroupInfo).Name.Length as usize / core::mem::size_of::<u16>()) = 0u16;

                    Ptr = (Ptr as usize)
                        .wrapping_add((*GroupInfo).Name.Length as usize + core::mem::size_of::<u16>()) as *mut u16;
                    (*GroupInfo3).grpi3_comment = Ptr;
                    memcpy(
                        (*GroupInfo3).grpi3_comment as *mut core::ffi::c_void,
                        (*GroupInfo).AdminComment.Buffer as *const core::ffi::c_void,
                        (*GroupInfo).AdminComment.Length as usize,
                    );
                    *(*GroupInfo3).grpi3_comment
                        .add((*GroupInfo).AdminComment.Length as usize / core::mem::size_of::<u16>()) = 0u16;

                    (*GroupInfo3).grpi3_group_sid = core::ptr::null_mut();
                    (*GroupInfo3).grpi3_attributes = (*GroupInfo).Attributes as u32;
                }
                _ => {}
            }
        }
    }

    if ApiStatus == NERR_Success {
        *Buffer = GroupBuffer;
    } else if !GroupBuffer.is_null() {
        NetApiBufferFree(GroupBuffer);
    }

    ApiStatus
}
unsafe extern "C" fn FreeGroupInfo(GroupInfo: PGROUP_GENERAL_INFORMATION) {
    if !(*GroupInfo).Name.Buffer.is_null() {
        SamFreeMemory((*GroupInfo).Name.Buffer as PVOID);
    }
    if !(*GroupInfo).AdminComment.Buffer.is_null() {
        SamFreeMemory((*GroupInfo).AdminComment.Buffer as PVOID);
    }
    SamFreeMemory(GroupInfo as PVOID);
}
pub unsafe extern "C" fn OpenGroupByName(
    DomainHandle: SAM_HANDLE,
    GroupName: PUNICODE_STRING,
    DesiredAccess: ULONG,
    GroupHandle: PSAM_HANDLE,
    RelativeId: PULONG,
) -> DWORD {
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    Status = SamLookupNamesInDomain(DomainHandle, 1, GroupName, &mut RelativeIds, &mut Use);
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_WARN as u32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"OpenGroupByName\0" as *const u8 as *const core::ffi::c_char,
                244 as i32,
                b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0" as *const u8
                    as *const core::ffi::c_char,
                GroupName,
                Status,
            );
        }
        return NetpNtStatusToApiStatus(Status);
    }
    if *Use as u32 != SidTypeGroup as u32 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"OpenGroupByName\0" as *const u8 as *const core::ffi::c_char,
                251 as i32,
                b"Object is not a group!\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        ApiStatus = NERR_GroupNotFound as DWORD;
    } else {
        Status = SamOpenGroup(
            DomainHandle,
            DesiredAccess as ACCESS_MASK,
            *RelativeIds,
            GroupHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"OpenGroupByName\0" as *const u8 as *const core::ffi::c_char,
                    263 as i32,
                    b"SamOpenGroup failed (Status %08lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else if !RelativeId.is_null() {
            *RelativeId = *RelativeIds;
        }
    }
    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as PVOID);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as PVOID);
    }
    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupAdd(
    servername: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    let mut AdminComment = GROUP_ADM_COMMENT_INFORMATION {
        AdminComment: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut AttributeInfo = GROUP_ATTRIBUTE_INFORMATION { Attributes: 0 };
    let mut ServerName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut GroupName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut Name: PWSTR = core::ptr::null_mut();
    let mut Comment: PWSTR = core::ptr::null_mut();
    let mut Attributes: ULONG = 0;
    let mut RelativeId: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Entry trace log
    if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/group_new.c\0"
                as *const u8 as *const i8,
            b"NetGroupAdd\0" as *const u8 as *const i8,
            307,
            b"NetGroupAdd(%s, %d, %p, %p)\n\0" as *const u8 as *const i8,
            debugstr_w(servername as *const WCHAR),
            level as i32,
            buf,
            parm_err,
        );
    }

    if !parm_err.is_null() {
        *parm_err = PARM_ERROR_NONE as DWORD;
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
    }

    match level {
        0 => {
            Name = (*(buf as PGROUP_INFO_0)).grpi0_name;
            Comment = core::ptr::null_mut();
            Attributes = 0;
        }
        1 => {
            Name = (*(buf as PGROUP_INFO_1)).grpi1_name;
            Comment = (*(buf as PGROUP_INFO_1)).grpi1_comment;
            Attributes = 0;
        }
        2 => {
            Name = (*(buf as PGROUP_INFO_2)).grpi2_name;
            Comment = (*(buf as PGROUP_INFO_2)).grpi2_comment;
            Attributes = (*(buf as PGROUP_INFO_2)).grpi2_attributes;
        }
        3 => {
            Name = (*(buf as PGROUP_INFO_3)).grpi3_name;
            Comment = (*(buf as PGROUP_INFO_3)).grpi3_comment;
            Attributes = (*(buf as PGROUP_INFO_3)).grpi3_attributes;
        }
        _ => return ERROR_INVALID_LEVEL as DWORD,
    }

    RtlInitUnicodeString(&raw mut GroupName, Name as PCWSTR);

    'cleanup: {
        Status = SamConnect(
            if !servername.is_null() { &raw mut ServerName } else { core::ptr::null_mut() },
            &raw mut ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const i8,
                    b"NetGroupAdd\0" as *const u8 as *const i8,
                    356,
                    b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'cleanup;
        }

        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() { &raw mut ServerName } else { core::ptr::null_mut() },
            (DOMAIN_CREATE_GROUP | DOMAIN_LOOKUP) as ULONG,
            &raw mut DomainHandle,
        );
        if Status < 0 {
            if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const i8,
                    b"NetGroupAdd\0" as *const u8 as *const i8,
                    368,
                    b"SamOpenDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'cleanup;
        }

        ApiStatus = OpenGroupByName(
            DomainHandle,
            &raw mut GroupName,
            GROUP_READ_INFORMATION as ULONG,
            &raw mut GroupHandle,
            core::ptr::null_mut(),
        );
        if ApiStatus == NERR_Success as DWORD {
            if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const i8,
                    b"NetGroupAdd\0" as *const u8 as *const i8,
                    381,
                    b"OpenGroupByName: Group %wZ already exists!\n\0" as *const u8 as *const i8,
                    &raw mut GroupName,
                );
            }
            SamCloseHandle(GroupHandle);
            ApiStatus = ERROR_GROUP_EXISTS as DWORD;
            break 'cleanup;
        }

        ApiStatus = NERR_Success as DWORD;

        Status = SamCreateGroupInDomain(
            DomainHandle,
            &raw mut GroupName,
            (DELETE | GROUP_WRITE_ACCOUNT) as ACCESS_MASK,
            &raw mut GroupHandle,
            &raw mut RelativeId,
        );
        if Status < 0 {
            if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const i8,
                    b"NetGroupAdd\0" as *const u8 as *const i8,
                    398,
                    b"SamCreateGroupInDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'cleanup;
        }

        if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_TRACE as i32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const i8,
                b"NetGroupAdd\0" as *const u8 as *const i8,
                403,
                b"Created group \"%wZ\" (RID: %lu)\n\0" as *const u8 as *const i8,
                &raw mut GroupName,
                RelativeId,
            );
        }

        if level == 1 || level == 2 || level == 3 {
            RtlInitUnicodeString(&raw mut AdminComment.AdminComment, Comment as PCWSTR);
            Status = SamSetInformationGroup(
                GroupHandle,
                GroupAdminCommentInformation,
                &raw mut AdminComment as *mut core::ffi::c_void,
            );
            if Status < 0 {
                if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const i8,
                        b"NetGroupAdd\0" as *const u8 as *const i8,
                        415,
                        b"SamSetInformationAlias failed (Status %08lx)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                SamDeleteGroup(GroupHandle);
                break 'cleanup;
            }
        }

        if level == 2 || level == 3 {
            AttributeInfo.Attributes = Attributes;
            Status = SamSetInformationGroup(
                GroupHandle,
                GroupAttributeInformation,
                &raw mut AttributeInfo as *mut core::ffi::c_void,
            );
            if Status < 0 {
                if (*(__wine_dbch___default as *const __wine_debug_channel)).flags & (1 << __WINE_DBCL_ERR as i32) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const i8,
                        b"NetGroupAdd\0" as *const u8 as *const i8,
                        435,
                        b"SamSetInformationAlias failed (Status %08lx)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                SamDeleteGroup(GroupHandle);
                break 'cleanup;
            }
        }
    }

    if !GroupHandle.is_null() {
        if ApiStatus != NERR_Success as DWORD {
            SamDeleteGroup(GroupHandle);
        } else {
            SamCloseHandle(GroupHandle);
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
pub unsafe extern "system" fn NetGroupAddUser(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    username: LPCWSTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut UserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: PULONG = core::ptr::null_mut::<u32>();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut::<_SID_NAME_USE>();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // WINE_DEFAULT_DEBUG_CHANNEL(netapi32) macro expansions
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/group_new.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetGroupAddUser\0" as *const u8 as *const ::core::ffi::c_char,
            483 as ::core::ffi::c_int,
            b"NetGroupAddUser(%s, %s, %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
            debugstr_w(username as *const u16),
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as *const u16);
    }
    RtlInitUnicodeString(&raw mut GroupName, groupname as *const u16);
    RtlInitUnicodeString(&raw mut UserName, username as *const u16);

    Status = SamConnect(
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut::<_UNICODE_STRING>()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
    );

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
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"NetGroupAddUser\0" as *const u8 as *const ::core::ffi::c_char,
                499 as ::core::ffi::c_int,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() {
                &raw mut ServerName
            } else {
                core::ptr::null_mut::<_UNICODE_STRING>()
            },
            DOMAIN_LOOKUP as u32,
            &raw mut DomainHandle,
        );

        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"NetGroupAddUser\0" as *const u8 as *const ::core::ffi::c_char,
                    511 as ::core::ffi::c_int,
                    b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenGroupByName(
                DomainHandle,
                &raw mut GroupName,
                GROUP_ADD_MEMBER as u32,
                &raw mut GroupHandle,
                core::ptr::null_mut::<u32>(),
            );

            if ApiStatus != NERR_Success as u32 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"NetGroupAddUser\0" as *const u8 as *const ::core::ffi::c_char,
                        524 as ::core::ffi::c_int,
                        b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        &raw mut GroupName,
                        ApiStatus,
                    );
                }
                if ApiStatus == ERROR_NONE_MAPPED as u32 {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
            } else {
                Status = SamLookupNamesInDomain(
                    DomainHandle,
                    1 as u32,
                    &raw mut UserName,
                    &raw mut RelativeIds,
                    &raw mut Use,
                );

                if !NT_SUCCESS(Status) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_3,
                            __dbch_3,
                            b"dll/win32/netapi32/group_new.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetGroupAddUser\0" as *const u8
                                as *const ::core::ffi::c_char,
                            537 as ::core::ffi::c_int,
                            b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            &raw mut UserName,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                } else if *Use.offset(0isize) as u32
                    != SidTypeUser as u32
                {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_4,
                            __dbch_4,
                            b"dll/win32/netapi32/group_new.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"NetGroupAddUser\0" as *const u8
                                as *const ::core::ffi::c_char,
                            545 as ::core::ffi::c_int,
                            b"Object is not a user!\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                    ApiStatus = NERR_GroupNotFound as DWORD;
                } else {
                    Status = SamAddMemberToGroup(
                        GroupHandle,
                        *RelativeIds.offset(0isize),
                        0 as u32,
                    );

                    if !NT_SUCCESS(Status) {
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_5,
                                __dbch_5,
                                b"dll/win32/netapi32/group_new.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"NetGroupAddUser\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                555 as ::core::ffi::c_int,
                                b"SamAddMemberToGroup failed (Status %lu)\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                }
            }
        }
    }

    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as *mut core::ffi::c_void);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as *mut core::ffi::c_void);
    }
    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupDel(
    servername: LPCWSTR,
    groupname: LPCWSTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // TRACE: NetGroupDel(%s, %s)
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/group_new.c\0"
                .as_ptr() as *const i8,
            b"NetGroupDel\0".as_ptr() as *const i8,
            595,
            b"NetGroupDel(%s, %s)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ServerName), servername as PCWSTR);
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(GroupName), groupname as PCWSTR);

    'cleanup: loop {
        Status = SamConnect(
            if !servername.is_null() {
                core::ptr::addr_of_mut!(ServerName) as PUNICODE_STRING
            } else {
                core::ptr::null_mut()
            },
            core::ptr::addr_of_mut!(ServerHandle),
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
        );
        if !NT_SUCCESS(Status) {
            // ERR: SamConnect failed
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const i8,
                    b"NetGroupDel\0".as_ptr() as *const i8,
                    609,
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
                core::ptr::addr_of_mut!(ServerName) as PUNICODE_STRING
            } else {
                core::ptr::null_mut()
            },
            DOMAIN_LOOKUP,
            core::ptr::addr_of_mut!(DomainHandle),
        );
        if !NT_SUCCESS(Status) {
            // ERR: OpenAccountDomain failed
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const i8,
                    b"NetGroupDel\0".as_ptr() as *const i8,
                    621,
                    b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'cleanup;
        }

        ApiStatus = OpenGroupByName(
            DomainHandle,
            core::ptr::addr_of_mut!(GroupName),
            DELETE,
            core::ptr::addr_of_mut!(GroupHandle),
            core::ptr::null_mut(),
        );
        if ApiStatus != NERR_Success {
            // ERR: OpenGroupByName failed
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const i8,
                    b"NetGroupDel\0".as_ptr() as *const i8,
                    634,
                    b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                    core::ptr::addr_of_mut!(GroupName),
                    ApiStatus,
                );
            }
            if ApiStatus == ERROR_NONE_MAPPED {
                ApiStatus = NERR_GroupNotFound;
            }
            break 'cleanup;
        }

        Status = SamDeleteGroup(GroupHandle);
        if !NT_SUCCESS(Status) {
            // ERR: SamDeleteGroup failed
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const i8,
                    b"NetGroupDel\0".as_ptr() as *const i8,
                    644,
                    b"SamDeleteGroup failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'cleanup;
        }

        // success – exit loop normally
        break;
    }

    // cleanup
    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
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
pub unsafe extern "system" fn NetGroupDelUser(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    username: LPCWSTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
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
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: PULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/group_new.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
            682i32,
            b"NetGroupDelUser(%s, %s, %s)\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            debugstr_w(username as *const WCHAR),
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
    }
    RtlInitUnicodeString(&raw mut GroupName, groupname as PCWSTR);
    RtlInitUnicodeString(&raw mut UserName, username as PCWSTR);

    Status = SamConnect(
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut::<_UNICODE_STRING>()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                698i32,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const core::ffi::c_char,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() {
                &raw mut ServerName
            } else {
                core::ptr::null_mut::<_UNICODE_STRING>()
            },
            DOMAIN_LOOKUP as ULONG,
            &raw mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                    710i32,
                    b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenGroupByName(
                DomainHandle,
                &raw mut GroupName,
                GROUP_REMOVE_MEMBER as ULONG,
                &raw mut GroupHandle,
                core::ptr::null_mut(),
            );
            if ApiStatus != NERR_Success as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                        723i32,
                        b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8
                            as *const core::ffi::c_char,
                        &raw mut GroupName,
                        ApiStatus,
                    );
                }
                if ApiStatus == ERROR_NONE_MAPPED as DWORD {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
            } else {
                Status = SamLookupNamesInDomain(
                    DomainHandle,
                    1u32,
                    &raw mut UserName,
                    &raw mut RelativeIds,
                    &raw mut Use,
                );
                if !NT_SUCCESS(Status) {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0
                    {
                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_3,
                            __dbch_3,
                            b"dll/win32/netapi32/group_new.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                            736i32,
                            b"SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n\0"
                                as *const u8 as *const core::ffi::c_char,
                            &raw mut UserName,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                } else if *Use.offset(0) as u32 != SidTypeUser as u32 {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0
                    {
                        let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_4,
                            __dbch_4,
                            b"dll/win32/netapi32/group_new.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                            744i32,
                            b"Object is not a user!\n\0" as *const u8 as *const core::ffi::c_char,
                        );
                    }
                    ApiStatus = NERR_GroupNotFound as DWORD;
                } else {
                    Status =
                        SamRemoveMemberFromGroup(GroupHandle, *RelativeIds.offset(0));
                    if !NT_SUCCESS(Status) {
                        if (*__wine_dbch___default).flags as i32
                            & (1i32 << __WINE_DBCL_ERR as i32)
                            != 0
                        {
                            let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_5,
                                __dbch_5,
                                b"dll/win32/netapi32/group_new.c\0"
                                    as *const u8 as *const core::ffi::c_char,
                                b"NetGroupDelUser\0" as *const u8 as *const core::ffi::c_char,
                                753i32,
                                b"SamRemoveMemberFromGroup failed (Status %lu)\n\0"
                                    as *const u8 as *const core::ffi::c_char,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                }
            }
        }
    }

    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as PVOID);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as PVOID);
    }
    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
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
pub unsafe extern "system" fn NetGroupEnum(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resume_handle: PDWORD_PTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut CurrentGroup: PSAM_RID_ENUMERATION = core::ptr::null_mut();
    let mut EnumContext: PGROUP_ENUM_CONTEXT = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupInfo: PGROUP_GENERAL_INFORMATION = core::ptr::null_mut();
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut ApiStatus: u32 = 0; // NERR_Success
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Entry trace
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/group_new.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
            800i32,
            b"NetGroupEnum(%s, %d, %p, %d, %p, %p, %p)\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            level,
            bufptr,
            prefmaxlen,
            entriesread,
            totalentries,
            resume_handle,
        );
    }

    *entriesread = 0;
    *totalentries = 0;
    *bufptr = core::ptr::null_mut();

    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername as PCWSTR);
    }

    'processing: {
        if !resume_handle.is_null() && *resume_handle != 0 {
            EnumContext = *resume_handle as PGROUP_ENUM_CONTEXT;
        } else {
            ApiStatus = NetApiBufferAllocate(
                core::mem::size_of::<GROUP_ENUM_CONTEXT>() as u32,
                &mut EnumContext as *mut PGROUP_ENUM_CONTEXT as *mut PVOID,
            );
            if ApiStatus != 0 {
                break 'processing;
            }

            (*EnumContext).EnumerationContext = 0;
            (*EnumContext).Buffer = core::ptr::null_mut();
            (*EnumContext).Returned = 0;
            (*EnumContext).Index = 0;

            Status = SamConnect(
                if !servername.is_null() {
                    &mut ServerName
                } else {
                    core::ptr::null_mut()
                },
                &mut (*EnumContext).ServerHandle,
                (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
                core::ptr::null_mut(),
            );
            if Status < 0 {
                // Error trace
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                        830i32,
                        b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'processing;
            }

            Status = OpenAccountDomain(
                (*EnumContext).ServerHandle,
                if !servername.is_null() {
                    &mut ServerName
                } else {
                    core::ptr::null_mut()
                },
                (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as u32,
                &mut (*EnumContext).AccountDomainHandle,
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                        841i32,
                        b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'processing;
            }

            Status = OpenBuiltinDomain(
                (*EnumContext).ServerHandle,
                (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as u32,
                &mut (*EnumContext).BuiltinDomainHandle,
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                        851i32,
                        b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'processing;
            }

            (*EnumContext).Phase = AccountPhase;
            (*EnumContext).DomainHandle = (*EnumContext).AccountDomainHandle;
        }

        // trace Index
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                863i32,
                b"EnumContext->Index: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*EnumContext).Index,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                864i32,
                b"EnumContext->Returned: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*EnumContext).Returned,
            );
        }

        if (*EnumContext).Index >= (*EnumContext).Returned {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                    868i32,
                    b"Calling SamEnumerateGroupsInDomain\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            Status = SamEnumerateGroupsInDomain(
                (*EnumContext).DomainHandle,
                &mut (*EnumContext).EnumerationContext,
                &mut (*EnumContext).Buffer as *mut PSAM_RID_ENUMERATION as *mut PVOID,
                prefmaxlen as u32,
                &mut (*EnumContext).Returned,
            );
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                    876i32,
                    b"SamEnumerateGroupsInDomain returned (Status %08lx)\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    Status,
                );
            }
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                        879i32,
                        b"SamEnumerateAliasesInDomain failed (Status %08lx)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'processing;
            }
            if Status == STATUS_MORE_ENTRIES {
                ApiStatus = ERROR_MORE_DATA;
                break 'processing;
            }
        }

        // traces for EnumContext
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                891i32,
                b"EnumContext: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
                EnumContext as usize,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                892i32,
                b"EnumContext->Returned: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*EnumContext).Returned,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                893i32,
                b"EnumContext->Buffer: %p\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*EnumContext).Buffer,
            );
        }

        CurrentGroup = (*EnumContext).Buffer.offset((*EnumContext).Index as isize)
            as PSAM_RID_ENUMERATION;

        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                898i32,
                b"RID: %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*CurrentGroup).RelativeId,
            );
        }

        Status = SamOpenGroup(
            (*EnumContext).DomainHandle,
            GROUP_READ_INFORMATION as ACCESS_MASK,
            (*CurrentGroup).RelativeId,
            &mut GroupHandle,
        );
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                    906i32,
                    b"SamOpenGroup failed (Status %08lx)\n\0".as_ptr() as *const ::core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'processing;
        }

        Status = SamQueryInformationGroup(
            GroupHandle,
            GroupGeneralInformation,
            &mut GroupInfo as *mut PGROUP_GENERAL_INFORMATION as *mut PVOID,
        );
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                    916i32,
                    b"SamQueryInformationGroup failed (Status %08lx)\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'processing;
        }

        SamCloseHandle(GroupHandle);
        GroupHandle = core::ptr::null_mut();

        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                924i32,
                b"Name: %S\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*GroupInfo).Name.Buffer,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
                925i32,
                b"Comment: %S\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*GroupInfo).AdminComment.Buffer,
            );
        }

        ApiStatus = BuildGroupInfoBuffer(
            GroupInfo,
            level,
            (*CurrentGroup).RelativeId as u32,
            &mut Buffer,
        );
        if ApiStatus != 0 {
            break 'processing;
        }

        if !GroupInfo.is_null() {
            FreeGroupInfo(GroupInfo);
            GroupInfo = core::ptr::null_mut();
        }

        (*EnumContext).Index = (*EnumContext).Index.wrapping_add(1);
        *entriesread = (*entriesread).wrapping_add(1);

        if (*EnumContext).Index == (*EnumContext).Returned {
            match (*EnumContext).Phase {
                BuiltinPhase => {
                    (*EnumContext).Phase = AccountPhase;
                    (*EnumContext).DomainHandle = (*EnumContext).AccountDomainHandle;
                    (*EnumContext).EnumerationContext = 0;
                    (*EnumContext).Index = 0;
                    (*EnumContext).Returned = 0;

                    if !(*EnumContext).Buffer.is_null() {
                        i = 0;
                        while i < (*EnumContext).Returned {
                            SamFreeMemory(
                                (*(*EnumContext).Buffer.offset(i as isize)).Name.Buffer as PVOID,
                            );
                            i = i.wrapping_add(1);
                        }
                        SamFreeMemory((*EnumContext).Buffer as PVOID);
                        (*EnumContext).Buffer = core::ptr::null_mut();
                    }
                }
                AccountPhase | DonePhase => {
                    (*EnumContext).Phase = DonePhase;
                }
                _ => {}
            }
        }
    } // end 'processing

    // 'done' section
    if ApiStatus == 0 && !EnumContext.is_null() && (*EnumContext).Phase != DonePhase as u32 {
        ApiStatus = ERROR_MORE_DATA as u32;
    }
    if !EnumContext.is_null() {
        *totalentries = (*EnumContext).Returned as DWORD;
    }
    if resume_handle.is_null() || ApiStatus != ERROR_MORE_DATA as u32 {
        if !EnumContext.is_null() {
            if !(*EnumContext).BuiltinDomainHandle.is_null() {
                SamCloseHandle((*EnumContext).BuiltinDomainHandle);
            }
            if !(*EnumContext).AccountDomainHandle.is_null() {
                SamCloseHandle((*EnumContext).AccountDomainHandle);
            }
            if !(*EnumContext).ServerHandle.is_null() {
                SamCloseHandle((*EnumContext).ServerHandle);
            }
            if !(*EnumContext).Buffer.is_null() {
                i = 0;
                while i < (*EnumContext).Returned {
                    SamFreeMemory(
                        (*(*EnumContext).Buffer.offset(i as isize)).Name.Buffer as PVOID,
                    );
                    i = i.wrapping_add(1);
                }
                SamFreeMemory((*EnumContext).Buffer as PVOID);
            }
            NetApiBufferFree(EnumContext as PVOID);
            EnumContext = core::ptr::null_mut();
        }
    }

    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
    }
    if !GroupInfo.is_null() {
        FreeGroupInfo(GroupInfo);
    }
    if !resume_handle.is_null() {
        *resume_handle = EnumContext as DWORD_PTR;
    }
    *bufptr = Buffer as LPBYTE;

    // Return trace
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/group_new.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetGroupEnum\0".as_ptr() as *const ::core::ffi::c_char,
            1021i32,
            b"return %lu\n\0".as_ptr() as *const ::core::ffi::c_char,
            ApiStatus,
        );
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupGetInfo(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> NET_API_STATUS {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut GroupInfo: PGROUP_GENERAL_INFORMATION = core::ptr::null_mut::<_GROUP_GENERAL_INFORMATION>();
    let mut Buffer: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut RelativeId: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"group_new.c\0".as_ptr() as *const i8,
            b"NetGroupGetInfo\0".as_ptr() as *const i8,
            1047i32,
            b"NetGroupGetInfo(%s, %s, %d, %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            level,
            bufptr,
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
    }
    RtlInitUnicodeString(&raw mut GroupName, groupname as PCWSTR);

    Status = SamConnect(
        if !servername.is_null() {
            &raw mut ServerName
        } else {
            core::ptr::null_mut::<_UNICODE_STRING>()
        },
        &raw mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
    );
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"group_new.c\0".as_ptr() as *const i8,
                b"NetGroupGetInfo\0".as_ptr() as *const i8,
                1061i32,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() {
                &raw mut ServerName
            } else {
                core::ptr::null_mut::<_UNICODE_STRING>()
            },
            DOMAIN_LOOKUP as ULONG,
            &raw mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"group_new.c\0".as_ptr() as *const i8,
                    b"NetGroupGetInfo\0".as_ptr() as *const i8,
                    1073i32,
                    b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenGroupByName(
                DomainHandle,
                &raw mut GroupName,
                GROUP_READ_INFORMATION as ULONG,
                &raw mut GroupHandle,
                &raw mut RelativeId,
            );
            if ApiStatus != NERR_Success as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"group_new.c\0".as_ptr() as *const i8,
                        b"NetGroupGetInfo\0".as_ptr() as *const i8,
                        1086i32,
                        b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                        &raw mut GroupName,
                        ApiStatus,
                    );
                }
                if ApiStatus == ERROR_NONE_MAPPED as DWORD {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
            } else {
                Status = SamQueryInformationGroup(
                    GroupHandle,
                    GroupGeneralInformation,
                    &raw mut GroupInfo as *mut PVOID,
                );
                if !NT_SUCCESS(Status) {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"group_new.c\0".as_ptr() as *const i8,
                            b"NetGroupGetInfo\0".as_ptr() as *const i8,
                            1097i32,
                            b"SamQueryInformationGroup failed (Status %08lx)\n\0".as_ptr() as *const i8,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                } else {
                    ApiStatus = BuildGroupInfoBuffer(GroupInfo, level, RelativeId as DWORD, &raw mut Buffer);
                    // The original C does "if (ApiStatus != NERR_Success) goto done;" here,
                    // but since there's no more code before the cleanup, just fall through.
                    let _ = ApiStatus != NERR_Success as DWORD;
                }
            }
        }
    }

    if !GroupInfo.is_null() {
        FreeGroupInfo(GroupInfo);
    }
    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    *bufptr = Buffer as LPBYTE;
    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupGetUsers(
    servername: *const u16,
    groupname: *const u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: PDWORD_PTR,
) -> u32 {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut UserInfo0: *mut _GROUP_USERS_INFO_0 = core::ptr::null_mut::<_GROUP_USERS_INFO_0>();
    let mut UserInfo1: *mut _GROUP_USERS_INFO_1 = core::ptr::null_mut::<_GROUP_USERS_INFO_1>();
    let mut EnumContext: PUSER_ENUM_CONTEXT = core::ptr::null_mut::<_USER_ENUM_CONTEXT>();
    let mut Buffer: *mut core::ffi::c_void = core::ptr::null_mut::<core::ffi::c_void>();
    let mut i: u32 = 0;
    let mut idx: u32 = 0;
    let mut Size: u32 = 0;
    let mut Ptr: *mut u16 = core::ptr::null_mut::<u16>();
    let mut ApiStatus: u32 = 0; // init later
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/group_new.c\0"
                as *const u8 as *const i8,
            b"NetGroupGetUsers\0" as *const u8 as *const i8,
            1153i32,
            b"NetGroupGetUsers(%s, %s, %d, %p, %d, %p, %p, %p)\n\0" as *const u8 as *const i8,
            debugstr_w(servername),
            debugstr_w(groupname),
            level,
            bufptr,
            prefmaxlen,
            entriesread,
            totalentries,
            resume_handle,
        );
    }

    *entriesread = 0u32;
    *totalentries = 0u32;
    *bufptr = core::ptr::null_mut::<u8>();

    if !servername.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ServerName), servername);
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(GroupName), groupname);

    if !resume_handle.is_null() && *resume_handle != 0 {
        EnumContext = *resume_handle as PUSER_ENUM_CONTEXT;
    } else {
        ApiStatus = NetApiBufferAllocate(
            core::mem::size_of::<USER_ENUM_CONTEXT>() as u32,
            &mut EnumContext as *mut PUSER_ENUM_CONTEXT as *mut *mut core::ffi::c_void,
        );
        if ApiStatus != NERR_Success as u32 {
            // early failure, goto done equivalent
        } else {
            (*EnumContext).MemberCount = 0u32;
            (*EnumContext).MemberIds = core::ptr::null_mut::<u32>();
            (*EnumContext).Attributes = core::ptr::null_mut::<u32>();
            (*EnumContext).Names = core::ptr::null_mut::<_UNICODE_STRING>();
            (*EnumContext).Start = 0u32;
            (*EnumContext).Count = 0u32;

            Status = SamConnect(
                if !servername.is_null() {
                    core::ptr::addr_of_mut!(ServerName) as *mut _UNICODE_STRING
                } else {
                    core::ptr::null_mut::<_UNICODE_STRING>()
                },
                &mut (*EnumContext).ServerHandle,
                (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
                core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
            );
            if !NT_SUCCESS(Status) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const i8,
                        b"NetGroupGetUsers\0" as *const u8 as *const i8,
                        1188i32,
                        b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
            } else {
                Status = OpenAccountDomain(
                    (*EnumContext).ServerHandle,
                    if !servername.is_null() {
                        core::ptr::addr_of_mut!(ServerName) as *mut _UNICODE_STRING
                    } else {
                        core::ptr::null_mut::<_UNICODE_STRING>()
                    },
                    DOMAIN_LOOKUP as u32,
                    &mut (*EnumContext).DomainHandle,
                );
                if !NT_SUCCESS(Status) {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"dll/win32/netapi32/group_new.c\0"
                                as *const u8 as *const i8,
                            b"NetGroupGetUsers\0" as *const u8 as *const i8,
                            1200i32,
                            b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                } else {
                    ApiStatus = OpenGroupByName(
                        (*EnumContext).DomainHandle,
                        core::ptr::addr_of_mut!(GroupName),
                        GROUP_LIST_MEMBERS as u32,
                        &mut (*EnumContext).GroupHandle,
                        core::ptr::null_mut::<u32>(),
                    );
                    if ApiStatus != NERR_Success as u32 {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_ERR,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/group_new.c\0"
                                    as *const u8 as *const i8,
                                b"NetGroupGetUsers\0" as *const u8 as *const i8,
                                1213i32,
                                b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8
                                    as *const i8,
                                core::ptr::addr_of_mut!(GroupName),
                                ApiStatus,
                            );
                        }
                        if ApiStatus == ERROR_NONE_MAPPED as u32 {
                            ApiStatus = NERR_GroupNotFound as u32;
                        }
                    } else {
                        Status = SamGetMembersInGroup(
                            (*EnumContext).GroupHandle,
                            &mut (*EnumContext).MemberIds,
                            &mut (*EnumContext).Attributes,
                            &mut (*EnumContext).MemberCount,
                        );
                        if !NT_SUCCESS(Status) {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupGetUsers\0" as *const u8 as *const i8,
                                    1226i32,
                                    b"SamGetMembersInGroup failed (Status %08lx)\n\0" as *const u8
                                        as *const i8,
                                    Status,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else if (*EnumContext).MemberCount > 0 {
                            Status = SamLookupIdsInDomain(
                                (*EnumContext).DomainHandle,
                                (*EnumContext).MemberCount,
                                (*EnumContext).MemberIds,
                                &mut (*EnumContext).Names,
                                core::ptr::null_mut::<PSID_NAME_USE>(),
                            );
                            if !NT_SUCCESS(Status) {
                                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                    ros_dbg_log(
                                        __WINE_DBCL_ERR,
                                        __wine_dbch___default,
                                        b"dll/win32/netapi32/group_new.c\0"
                                            as *const u8 as *const i8,
                                        b"NetGroupGetUsers\0" as *const u8 as *const i8,
                                        1241i32,
                                        b"SamLookupIdsInDomain failed (Status %08lx)\n\0" as *const u8
                                            as *const i8,
                                        Status,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                                // ApiStatus set, will fall through to cleanup (goto done)
                            }
                            // else success, continue to buffer allocation
                        }
                    }
                }
            }
        }
    }

    // Note: If any of the above blocks set ApiStatus but not EnumContext or left early,
    // the flow continues here because we haven't jumped to cleanup/done yet.
    // But we need to handle the case where the early failures occurred.
    // In the C code, failures goto done, skipping the buffer size/allocation part.
    // So we must mimic that. We'll use a labeled block to break out of the buffer logic.

    'buffer_calc: {
        // In the C, any error in the init block above does `goto done`, skipping the
        // buffer size calculation and allocation. ApiStatus carries the error (it starts
        // at NERR_Success and is only set non-zero on failure), so skip on any error.
        if ApiStatus != NERR_Success as u32 {
            break 'buffer_calc;
        }

        Size = 0u32;
        if prefmaxlen == 0xFFFFFFFFu32 { // -1
            Size = ((*EnumContext).MemberCount as usize)
                .wrapping_mul(if level == 0 {
                    core::mem::size_of::<GROUP_USERS_INFO_0>()
                } else {
                    core::mem::size_of::<GROUP_USERS_INFO_1>()
                }) as u32;
            i = (*EnumContext).Start;
            while i < (*EnumContext).MemberCount {
                Size = (Size as u64).wrapping_add(
                    ((*(*EnumContext).Names.offset(i as isize)).Length as usize)
                        .wrapping_add(core::mem::size_of::<u16>()) as u64,
                ) as u32;
                i = i.wrapping_add(1);
            }
            (*EnumContext).Count = (*EnumContext).MemberCount;
        } else {
            i = (*EnumContext).Start;
            while i < (*EnumContext).MemberCount {
                Size = (Size as u64).wrapping_add(
                    (if level == 0 {
                        core::mem::size_of::<GROUP_USERS_INFO_0>()
                    } else {
                        core::mem::size_of::<GROUP_USERS_INFO_1>()
                    }) as u64,
                ) as u32;
                Size = (Size as u64).wrapping_add(
                    ((*(*EnumContext).Names.offset(i as isize)).Length as usize)
                        .wrapping_add(core::mem::size_of::<u16>()) as u64,
                ) as u32;
                (*EnumContext).Count = (*EnumContext).Count.wrapping_add(1);
                if Size >= prefmaxlen {
                    break;
                }
                i = i.wrapping_add(1);
            }
        }

        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const i8,
                b"NetGroupGetUsers\0" as *const u8 as *const i8,
                1273i32,
                b"Buffer size: %lu\n\0" as *const u8 as *const i8,
                Size,
            );
        }

        ApiStatus = NetApiBufferAllocate(Size as u32, &mut Buffer);
        if ApiStatus != NERR_Success as u32 {
            // goto done, but from buffer allocation failure, we need to go to cleanup.
            break 'buffer_calc;
        }

        core::ptr::write_bytes(Buffer as *mut u8, 0u8, Size as usize);

        if level == 0 {
            Ptr = (Buffer as usize)
                .wrapping_add(
                    ((*EnumContext).Count as usize)
                        .wrapping_mul(core::mem::size_of::<GROUP_USERS_INFO_0>()),
                ) as *mut u16;
        } else {
            Ptr = (Buffer as usize)
                .wrapping_add(
                    ((*EnumContext).Count as usize)
                        .wrapping_mul(core::mem::size_of::<GROUP_USERS_INFO_1>()),
                ) as *mut u16;
        }

        i = 0u32;
        while i < (*EnumContext).Count {
            idx = (*EnumContext).Start.wrapping_add(i);
            if level == 0 {
                UserInfo0 = Buffer as *mut _GROUP_USERS_INFO_0;
                let name_ptr_ref = &mut (*UserInfo0.offset(i as isize)).grui0_name;
                *name_ptr_ref = Ptr as *mut u16;
                core::ptr::copy_nonoverlapping(
                    (*(*EnumContext).Names.offset(idx as isize)).Buffer as *const u8,
                    (*UserInfo0.offset(i as isize)).grui0_name as *mut u8,
                    (*(*EnumContext).Names.offset(idx as isize)).Length as usize,
                );
                *(*UserInfo0.offset(i as isize)).grui0_name
                    .offset(
                        ((*(*EnumContext).Names.offset(idx as isize)).Length as usize)
                            .wrapping_div(core::mem::size_of::<u16>()) as isize,
                    ) = 0u16; // UNICODE_NULL
                Ptr = (Ptr as usize)
                    .wrapping_add((*(*EnumContext).Names.offset(idx as isize)).Length as usize)
                    .wrapping_add(core::mem::size_of::<u16>()) as *mut u16;
            } else {
                UserInfo1 = Buffer as *mut _GROUP_USERS_INFO_1;
                let name_ptr_ref1 = &mut (*UserInfo1.offset(i as isize)).grui1_name;
                *name_ptr_ref1 = Ptr as *mut u16;
                core::ptr::copy_nonoverlapping(
                    (*(*EnumContext).Names.offset(idx as isize)).Buffer as *const u8,
                    (*UserInfo1.offset(i as isize)).grui1_name as *mut u8,
                    (*(*EnumContext).Names.offset(idx as isize)).Length as usize,
                );
                *(*UserInfo1.offset(i as isize)).grui1_name
                    .offset(
                        ((*(*EnumContext).Names.offset(idx as isize)).Length as usize)
                            .wrapping_div(core::mem::size_of::<u16>()) as isize,
                    ) = 0u16;
                (*UserInfo1.offset(i as isize)).grui1_attributes =
                    *(*EnumContext).Attributes.offset(idx as isize) as u32;
                Ptr = (Ptr as usize)
                    .wrapping_add((*(*EnumContext).Names.offset(idx as isize)).Length as usize)
                    .wrapping_add(core::mem::size_of::<u16>()) as *mut u16;
            }
            i = i.wrapping_add(1);
        }

        (*EnumContext).Start = ((*EnumContext).Start as usize)
            .wrapping_add((*EnumContext).Count as usize) as u32;

        if (*EnumContext).MemberCount > (*EnumContext).Start {
            ApiStatus = ERROR_MORE_DATA as u32;
        } else {
            ApiStatus = NERR_Success as u32;
        }
    } // end buffer_calc block

    // cleanup (done label in C)
    if !EnumContext.is_null() {
        *entriesread = (*EnumContext).Count as u32;
        *totalentries = (*EnumContext).MemberCount as u32;
    }

    if resume_handle.is_null() || ApiStatus != ERROR_MORE_DATA as u32 {
        if !EnumContext.is_null() {
            if !(*EnumContext).Names.is_null() {
                i = 0u32;
                while i < (*EnumContext).MemberCount {
                    SamFreeMemory((*(*EnumContext).Names.offset(i as isize)).Buffer as *mut core::ffi::c_void);
                    i = i.wrapping_add(1);
                }
                SamFreeMemory((*EnumContext).Names as *mut core::ffi::c_void);
            }
            if !(*EnumContext).Attributes.is_null() {
                SamFreeMemory((*EnumContext).Attributes as *mut core::ffi::c_void);
            }
            if !(*EnumContext).MemberIds.is_null() {
                SamFreeMemory((*EnumContext).MemberIds as *mut core::ffi::c_void);
            }
            if !(*EnumContext).GroupHandle.is_null() {
                SamCloseHandle((*EnumContext).GroupHandle);
            }
            if !(*EnumContext).DomainHandle.is_null() {
                SamCloseHandle((*EnumContext).DomainHandle);
            }
            if !(*EnumContext).ServerHandle.is_null() {
                SamCloseHandle((*EnumContext).ServerHandle);
            }
            NetApiBufferFree(EnumContext as *mut core::ffi::c_void);
            EnumContext = core::ptr::null_mut::<_USER_ENUM_CONTEXT>();
        }
    }

    *bufptr = Buffer as *mut u8;
    if !resume_handle.is_null() {
        *resume_handle = EnumContext as DWORD_PTR;
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupSetInfo(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut GroupNameInfo: GROUP_NAME_INFORMATION = GROUP_NAME_INFORMATION {
        Name: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut AdminCommentInfo: GROUP_ADM_COMMENT_INFORMATION = GROUP_ADM_COMMENT_INFORMATION {
        AdminComment: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut AttributeInfo: GROUP_ATTRIBUTE_INFORMATION = GROUP_ATTRIBUTE_INFORMATION {
        Attributes: 0,
    };
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Debug log entry
    if !__wine_dbch___default.is_null()
        && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0)
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/group_new.c\0"
                as *const u8 as *const i8,
            b"NetGroupSetInfo\0" as *const u8 as *const i8,
            1401,
            b"NetGroupSetInfo(%s, %s, %d, %p, %p)\n\0" as *const u8 as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
            level,
            buf,
            parm_err,
        );
    }

    if !parm_err.is_null() {
        *parm_err = PARM_ERROR_NONE as DWORD;
    }

    if !servername.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ServerName), servername as PCWSTR);
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(GroupName), groupname as PCWSTR);

    Status = SamConnect(
        if !servername.is_null() {
            core::ptr::addr_of_mut!(ServerName)
        } else {
            core::ptr::null_mut()
        },
        core::ptr::addr_of_mut!(ServerHandle),
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
        core::ptr::null_mut(),
    );

    if (Status as i32) < 0 {
        // Error debug log
        if !__wine_dbch___default.is_null()
            && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/group_new.c\0"
                    as *const u8 as *const i8,
                b"NetGroupSetInfo\0" as *const u8 as *const i8,
                1418,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenAccountDomain(
            ServerHandle,
            if !servername.is_null() {
                core::ptr::addr_of_mut!(ServerName)
            } else {
                core::ptr::null_mut()
            },
            DOMAIN_LOOKUP as ULONG,
            core::ptr::addr_of_mut!(DomainHandle),
        );

        if (Status as i32) < 0 {
            // Error debug log
            if !__wine_dbch___default.is_null()
                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/group_new.c\0"
                        as *const u8 as *const i8,
                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                    1430,
                    b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenGroupByName(
                DomainHandle,
                core::ptr::addr_of_mut!(GroupName),
                GROUP_WRITE_ACCOUNT as ULONG,
                core::ptr::addr_of_mut!(GroupHandle),
                core::ptr::null_mut(),
            );
            if ApiStatus != (NERR_Success as DWORD) {
                // Warning debug log
                if !__wine_dbch___default.is_null()
                    && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_WARN as u32)) != 0)
                {
                    ros_dbg_log(
                        __WINE_DBCL_WARN,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/group_new.c\0"
                            as *const u8 as *const i8,
                        b"NetGroupSetInfo\0" as *const u8 as *const i8,
                        1443,
                        b"OpenGroupByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8
                            as *const i8,
                        core::ptr::addr_of_mut!(GroupName),
                        ApiStatus,
                    );
                }
                if ApiStatus == (ERROR_NONE_MAPPED as DWORD) {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
            } else {
                match level {
                    0 => {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(GroupNameInfo.Name),
                            (*(buf as *const _GROUP_INFO_0)).grpi0_name as PCWSTR,
                        );
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupNameInformation,
                            core::ptr::addr_of_mut!(GroupNameInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1461,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        }
                    }
                    1 => {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(GroupNameInfo.Name),
                            (*(buf as *const _GROUP_INFO_1)).grpi1_name as PCWSTR,
                        );
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupNameInformation,
                            core::ptr::addr_of_mut!(GroupNameInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1477,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else {
                            RtlInitUnicodeString(
                                core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                                (*(buf as *const _GROUP_INFO_1)).grpi1_comment as PCWSTR,
                            );
                            Status = SamSetInformationGroup(
                                GroupHandle,
                                GroupAdminCommentInformation,
                                core::ptr::addr_of_mut!(AdminCommentInfo) as PVOID,
                            );
                            if (Status as i32) < 0 {
                                // Error debug log
                                if !__wine_dbch___default.is_null()
                                    && ((*__wine_dbch___default).flags
                                        & (1 << __WINE_DBCL_ERR))
                                        != 0
                                {
                                    ros_dbg_log(
                                        __WINE_DBCL_ERR,
                                        __wine_dbch___default,
                                        b"dll/win32/netapi32/group_new.c\0"
                                            as *const u8 as *const i8,
                                        b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                        1491,
                                        b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                            as *const u8 as *const i8,
                                        ApiStatus,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                            }
                        }
                    }
                    2 => {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(GroupNameInfo.Name),
                            (*(buf as *const _GROUP_INFO_2)).grpi2_name as PCWSTR,
                        );
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupNameInformation,
                            core::ptr::addr_of_mut!(GroupNameInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1507,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else {
                            RtlInitUnicodeString(
                                core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                                (*(buf as *const _GROUP_INFO_2)).grpi2_comment as PCWSTR,
                            );
                            Status = SamSetInformationGroup(
                                GroupHandle,
                                GroupAdminCommentInformation,
                                core::ptr::addr_of_mut!(AdminCommentInfo) as PVOID,
                            );
                            if (Status as i32) < 0 {
                                // Error debug log
                                if !__wine_dbch___default.is_null()
                                    && ((*__wine_dbch___default).flags
                                        & (1 << __WINE_DBCL_ERR))
                                        != 0
                                {
                                    ros_dbg_log(
                                        __WINE_DBCL_ERR,
                                        __wine_dbch___default,
                                        b"dll/win32/netapi32/group_new.c\0"
                                            as *const u8 as *const i8,
                                        b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                        1521,
                                        b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                            as *const u8 as *const i8,
                                        ApiStatus,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                            } else {
                                AttributeInfo.Attributes =
                                    (*(buf as *const _GROUP_INFO_2)).grpi2_attributes as ULONG;
                                Status = SamSetInformationGroup(
                                    GroupHandle,
                                    GroupAttributeInformation,
                                    core::ptr::addr_of_mut!(AttributeInfo) as PVOID,
                                );
                                if (Status as i32) < 0 {
                                    // Error debug log
                                    if !__wine_dbch___default.is_null()
                                        && ((*__wine_dbch___default).flags
                                            & (1 << __WINE_DBCL_ERR))
                                            != 0
                                    {
                                        ros_dbg_log(
                                            __WINE_DBCL_ERR,
                                            __wine_dbch___default,
                                            b"dll/win32/netapi32/group_new.c\0"
                                                as *const u8 as *const i8,
                                            b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                            1534,
                                            b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                                as *const u8 as *const i8,
                                            ApiStatus,
                                        );
                                    }
                                    ApiStatus = NetpNtStatusToApiStatus(Status);
                                }
                            }
                        }
                    }
                    3 => {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(GroupNameInfo.Name),
                            (*(buf as *const _GROUP_INFO_3)).grpi3_name as PCWSTR,
                        );
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupNameInformation,
                            core::ptr::addr_of_mut!(GroupNameInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1550,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else {
                            RtlInitUnicodeString(
                                core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                                (*(buf as *const _GROUP_INFO_3)).grpi3_comment as PCWSTR,
                            );
                            Status = SamSetInformationGroup(
                                GroupHandle,
                                GroupAdminCommentInformation,
                                core::ptr::addr_of_mut!(AdminCommentInfo) as PVOID,
                            );
                            if (Status as i32) < 0 {
                                // Error debug log
                                if !__wine_dbch___default.is_null()
                                    && ((*__wine_dbch___default).flags
                                        & (1 << __WINE_DBCL_ERR))
                                        != 0
                                {
                                    ros_dbg_log(
                                        __WINE_DBCL_ERR,
                                        __wine_dbch___default,
                                        b"dll/win32/netapi32/group_new.c\0"
                                            as *const u8 as *const i8,
                                        b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                        1564,
                                        b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                            as *const u8 as *const i8,
                                        ApiStatus,
                                    );
                                }
                                ApiStatus = NetpNtStatusToApiStatus(Status);
                            } else {
                                AttributeInfo.Attributes =
                                    (*(buf as *const _GROUP_INFO_3)).grpi3_attributes as ULONG;
                                Status = SamSetInformationGroup(
                                    GroupHandle,
                                    GroupAttributeInformation,
                                    core::ptr::addr_of_mut!(AttributeInfo) as PVOID,
                                );
                                if (Status as i32) < 0 {
                                    // Error debug log
                                    if !__wine_dbch___default.is_null()
                                        && ((*__wine_dbch___default).flags
                                            & (1 << __WINE_DBCL_ERR))
                                            != 0
                                    {
                                        ros_dbg_log(
                                            __WINE_DBCL_ERR,
                                            __wine_dbch___default,
                                            b"dll/win32/netapi32/group_new.c\0"
                                                as *const u8 as *const i8,
                                            b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                            1577,
                                            b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                                as *const u8 as *const i8,
                                            ApiStatus,
                                        );
                                    }
                                    ApiStatus = NetpNtStatusToApiStatus(Status);
                                }
                            }
                        }
                    }
                    1002 => {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                            (*(buf as *const _GROUP_INFO_1002)).grpi1002_comment as PCWSTR,
                        );
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupAdminCommentInformation,
                            core::ptr::addr_of_mut!(AdminCommentInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1593,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        }
                    }
                    1005 => {
                        AttributeInfo.Attributes =
                            (*(buf as *const _GROUP_INFO_1005)).grpi1005_attributes as ULONG;
                        Status = SamSetInformationGroup(
                            GroupHandle,
                            GroupAttributeInformation,
                            core::ptr::addr_of_mut!(AttributeInfo) as PVOID,
                        );
                        if (Status as i32) < 0 {
                            // Error debug log
                            if !__wine_dbch___default.is_null()
                                && ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0)
                            {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/group_new.c\0"
                                        as *const u8 as *const i8,
                                    b"NetGroupSetInfo\0" as *const u8 as *const i8,
                                    1608,
                                    b"SamSetInformationGroup failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    ApiStatus,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        }
                    }
                    _ => {
                        ApiStatus = ERROR_INVALID_LEVEL as DWORD;
                    }
                }
            }
        }
    }

    // Cleanup
    if !GroupHandle.is_null() {
        SamCloseHandle(GroupHandle);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    return ApiStatus;
}

#[no_mangle]
pub unsafe extern "system" fn NetGroupSetUsers(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    totalentries: DWORD,
) -> DWORD {
    let mut api_status: DWORD = NERR_Success as DWORD;
    let mut server_name: _UNICODE_STRING = core::mem::zeroed();
    let mut group_name: _UNICODE_STRING = core::mem::zeroed();
    let mut server_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut domain_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut group_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut old_member_count: ULONG = 0;
    let mut old_member_ids: PULONG = core::ptr::null_mut();
    let mut old_attributes: PULONG = core::ptr::null_mut();
    let mut names_array: PUNICODE_STRING = core::ptr::null_mut();
    let mut new_member_ids: PULONG = core::ptr::null_mut();
    let mut new_member_use: PSID_NAME_USE = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;

    if !servername.is_null() {
        RtlInitUnicodeString(&mut server_name, servername);
    }
    RtlInitUnicodeString(&mut group_name, groupname);

    let result = 'done: {
        status = SamConnect(
            if !servername.is_null() { &mut server_name } else { core::ptr::null_mut() },
            &mut server_handle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            core::ptr::null_mut(),
        );
        if status < 0 {
            api_status = NetpNtStatusToApiStatus(status);
            break 'done api_status;
        }

        status = OpenAccountDomain(
            server_handle,
            if !servername.is_null() { &mut server_name } else { core::ptr::null_mut() },
            DOMAIN_LOOKUP as ULONG,
            &mut domain_handle,
        );
        if status < 0 {
            api_status = NetpNtStatusToApiStatus(status);
            break 'done api_status;
        }

        api_status = OpenGroupByName(
            domain_handle,
            &mut group_name,
            (GROUP_LIST_MEMBERS | GROUP_ADD_MEMBER | GROUP_REMOVE_MEMBER) as ULONG,
            &mut group_handle,
            core::ptr::null_mut(),
        );
        if api_status != NERR_Success as DWORD {
            if api_status == ERROR_NONE_MAPPED as DWORD {
                api_status = NERR_GroupNotFound as DWORD;
            }
            break 'done api_status;
        }

        status = SamGetMembersInGroup(
            group_handle,
            &mut old_member_ids,
            &mut old_attributes,
            &mut old_member_count,
        );
        if status < 0 {
            api_status = NetpNtStatusToApiStatus(status);
            break 'done api_status;
        }

        names_array = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            (totalentries as usize * core::mem::size_of::<_UNICODE_STRING>()) as SIZE_T,
        ) as PUNICODE_STRING;
        if names_array.is_null() {
            api_status = ERROR_OUTOFMEMORY as DWORD;
            break 'done api_status;
        }

        let user_info0 = buf as *mut _GROUP_USERS_INFO_0;
        let user_info1 = buf as *mut _GROUP_USERS_INFO_1;
        for i in 0..totalentries as usize {
            if level == 0 {
                RtlInitUnicodeString(
                    &mut *names_array.add(i),
                    (*user_info0.add(i)).grui0_name,
                );
            } else {
                RtlInitUnicodeString(
                    &mut *names_array.add(i),
                    (*user_info1.add(i)).grui1_name,
                );
            }
        }

        status = SamLookupNamesInDomain(
            domain_handle,
            totalentries,
            names_array,
            &mut new_member_ids,
            &mut new_member_use,
        );
        if status < 0 {
            if status == STATUS_NONE_MAPPED {
                api_status = NERR_UserNotFound as DWORD;
            } else {
                api_status = NetpNtStatusToApiStatus(status);
            }
            break 'done api_status;
        }

        for i in 0..totalentries as usize {
            let mut found: BOOL = FALSE;
            for j in 0..old_member_count as usize {
                if *new_member_ids.add(i) == *old_member_ids.add(j) {
                    if level == 1 {
                        status = SamSetMemberAttributesOfGroup(
                            group_handle,
                            *new_member_ids.add(i),
                            (*user_info1.add(i)).grui1_attributes as ULONG,
                        );
                        if status < 0 {
                            api_status = NetpNtStatusToApiStatus(status);
                            break 'done api_status;
                        }
                    }
                    found = TRUE;
                    break;
                }
            }
            if found == FALSE {
                if *new_member_use.add(i) as u32 != SidTypeUser as u32 {
                    api_status = NERR_GroupNotFound as DWORD;
                    break 'done api_status;
                }
                let attrs = if level == 0 { 0u32 } else { (*user_info1.add(i)).grui1_attributes as u32 };
                status = SamAddMemberToGroup(
                    group_handle,
                    *new_member_ids.add(i),
                    attrs,
                );
                if status < 0 {
                    api_status = NetpNtStatusToApiStatus(status);
                    break 'done api_status;
                }
            }
        }

        for i in 0..old_member_count as usize {
            let mut found: BOOL = FALSE;
            for j in 0..totalentries as usize {
                if *old_member_ids.add(i) == *new_member_ids.add(j) {
                    found = TRUE;
                    break;
                }
            }
            if found == FALSE {
                status = SamRemoveMemberFromGroup(
                    group_handle,
                    *old_member_ids.add(i),
                );
                if status < 0 {
                    api_status = NetpNtStatusToApiStatus(status);
                    break 'done api_status;
                }
            }
        }

        api_status = NERR_Success as DWORD;
        break 'done api_status;
    };

    if !new_member_use.is_null() {
        SamFreeMemory(new_member_use as *mut core::ffi::c_void);
    }
    if !new_member_ids.is_null() {
        SamFreeMemory(new_member_ids as *mut core::ffi::c_void);
    }
    if !names_array.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, names_array as *mut core::ffi::c_void);
    }
    if !old_member_ids.is_null() {
        SamFreeMemory(old_member_ids as *mut core::ffi::c_void);
    }
    if !group_handle.is_null() {
        SamCloseHandle(group_handle);
    }
    if !domain_handle.is_null() {
        SamCloseHandle(domain_handle);
    }
    if !server_handle.is_null() {
        SamCloseHandle(server_handle);
    }

    result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS NetAPI DLL
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     SAM service group interface code
//  * COPYRIGHT:   Copyright 2018 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "netapi32.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// typedef enum _ENUM_PHASE
// {
//     BuiltinPhase,
//     AccountPhase,
//     DonePhase
// } ENUM_PHASE;
//
// typedef struct _GROUP_ENUM_CONTEXT
// {
//     SAM_HANDLE ServerHandle;
//     SAM_HANDLE DomainHandle;
//     SAM_HANDLE BuiltinDomainHandle;
//     SAM_HANDLE AccountDomainHandle;
//
//     SAM_ENUMERATE_HANDLE EnumerationContext;
//     PSAM_RID_ENUMERATION Buffer;
//     ULONG Returned;
//     ULONG Index;
//     ENUM_PHASE Phase;
// } GROUP_ENUM_CONTEXT, *PGROUP_ENUM_CONTEXT;
//
// typedef struct _USER_ENUM_CONTEXT
// {
//     SAM_HANDLE ServerHandle;
//     SAM_HANDLE DomainHandle;
//     SAM_HANDLE GroupHandle;
//
//     ULONG MemberCount;
//     PULONG MemberIds;
//     PULONG Attributes;
//     PUNICODE_STRING Names;
//
//     ULONG Start;
//     ULONG Count;
// } USER_ENUM_CONTEXT, *PUSER_ENUM_CONTEXT;
//
//
// /* FUNCTIONS *****************************************************************/
//
// static
// NET_API_STATUS
// BuildGroupInfoBuffer(
//      PGROUP_GENERAL_INFORMATION GroupInfo,
//      DWORD Level,
//      DWORD GroupId,
//      LPVOID *Buffer)
// {
//     PVOID GroupBuffer = NULL;
//     PGROUP_INFO_0 GroupInfo0;
//     PGROUP_INFO_1 GroupInfo1;
//     PGROUP_INFO_2 GroupInfo2;
//     PGROUP_INFO_3 GroupInfo3;
//     PWSTR Ptr;
//     ULONG Size = 0;
//     NET_API_STATUS ApiStatus = NERR_Success;
//
//     *Buffer = NULL;
//
//     switch (Level)
//     {
//         case 0:
//             Size = sizeof(GROUP_INFO_0) +
//                    GroupInfo->Name.Length + sizeof(WCHAR);
//             break;
//
//         case 1:
//             Size = sizeof(GROUP_INFO_1) +
//                    GroupInfo->Name.Length + sizeof(WCHAR) +
//                    GroupInfo->AdminComment.Length + sizeof(WCHAR);
//             break;
//
//         case 2:
//             Size = sizeof(GROUP_INFO_2) +
//                    GroupInfo->Name.Length + sizeof(WCHAR) +
//                    GroupInfo->AdminComment.Length + sizeof(WCHAR);
//             break;
//
//         case 3:
//             Size = sizeof(GROUP_INFO_3) +
//                    GroupInfo->Name.Length + sizeof(WCHAR) +
//                    GroupInfo->AdminComment.Length + sizeof(WCHAR);
//                    /* FIXME: Sid size */
//             break;
//
//         default:
//             ApiStatus = ERROR_INVALID_LEVEL;
//             goto done;
//     }
//
//     ApiStatus = NetApiBufferAllocate(Size, &GroupBuffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     ZeroMemory(GroupBuffer, Size);
//
//     switch (Level)
//     {
//         case 0:
//             GroupInfo0 = (PGROUP_INFO_0)GroupBuffer;
//
//             Ptr = (PWSTR)((ULONG_PTR)GroupInfo0 + sizeof(LOCALGROUP_INFO_0));
//             GroupInfo0->grpi0_name = Ptr;
//
//             memcpy(GroupInfo0->grpi0_name,
//                    GroupInfo->Name.Buffer,
//                    GroupInfo->Name.Length);
//             GroupInfo0->grpi0_name[GroupInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 1:
//             GroupInfo1 = (PGROUP_INFO_1)GroupBuffer;
//
//             Ptr = (PWSTR)((ULONG_PTR)GroupInfo1 + sizeof(GROUP_INFO_1));
//             GroupInfo1->grpi1_name = Ptr;
//
//             memcpy(GroupInfo1->grpi1_name,
//                    GroupInfo->Name.Buffer,
//                    GroupInfo->Name.Length);
//             GroupInfo1->grpi1_name[GroupInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (PWSTR)((ULONG_PTR)Ptr + GroupInfo->Name.Length + sizeof(WCHAR));
//             GroupInfo1->grpi1_comment = Ptr;
//
//             memcpy(GroupInfo1->grpi1_comment,
//                    GroupInfo->AdminComment.Buffer,
//                    GroupInfo->AdminComment.Length);
//             GroupInfo1->grpi1_comment[GroupInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 2:
//             GroupInfo2 = (PGROUP_INFO_2)GroupBuffer;
//
//             Ptr = (PWSTR)((ULONG_PTR)GroupInfo2 + sizeof(GROUP_INFO_2));
//             GroupInfo2->grpi2_name = Ptr;
//
//             memcpy(GroupInfo2->grpi2_name,
//                    GroupInfo->Name.Buffer,
//                    GroupInfo->Name.Length);
//             GroupInfo2->grpi2_name[GroupInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (PWSTR)((ULONG_PTR)Ptr + GroupInfo->Name.Length + sizeof(WCHAR));
//             GroupInfo2->grpi2_comment = Ptr;
//
//             memcpy(GroupInfo2->grpi2_comment,
//                    GroupInfo->AdminComment.Buffer,
//                    GroupInfo->AdminComment.Length);
//             GroupInfo2->grpi2_comment[GroupInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             GroupInfo2->grpi2_group_id = GroupId;
//
//             GroupInfo2->grpi2_attributes= GroupInfo->Attributes;
//             break;
//
//         case 3:
//             GroupInfo3 = (PGROUP_INFO_3)GroupBuffer;
//
//             Ptr = (PWSTR)((ULONG_PTR)GroupInfo3 + sizeof(GROUP_INFO_3));
//             GroupInfo3->grpi3_name = Ptr;
//
//             memcpy(GroupInfo3->grpi3_name,
//                    GroupInfo->Name.Buffer,
//                    GroupInfo->Name.Length);
//             GroupInfo3->grpi3_name[GroupInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (PWSTR)((ULONG_PTR)Ptr + GroupInfo->Name.Length + sizeof(WCHAR));
//             GroupInfo3->grpi3_comment = Ptr;
//
//             memcpy(GroupInfo3->grpi3_comment,
//                    GroupInfo->AdminComment.Buffer,
//                    GroupInfo->AdminComment.Length);
//             GroupInfo3->grpi3_comment[GroupInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             GroupInfo3->grpi3_group_sid = NULL; /* FIXME */
//
//             GroupInfo3->grpi3_attributes= GroupInfo->Attributes;
//             break;
//     }
//
// done:
//     if (ApiStatus == NERR_Success)
//     {
//         *Buffer = GroupBuffer;
//     }
//     else
//     {
//         if (GroupBuffer != NULL)
//             NetApiBufferFree(GroupBuffer);
//     }
//
//     return ApiStatus;
// }
//
//
// static
// VOID
// FreeGroupInfo(
//      PGROUP_GENERAL_INFORMATION GroupInfo)
// {
//     if (GroupInfo->Name.Buffer != NULL)
//         SamFreeMemory(GroupInfo->Name.Buffer);
//
//     if (GroupInfo->AdminComment.Buffer != NULL)
//         SamFreeMemory(GroupInfo->AdminComment.Buffer);
//
//     SamFreeMemory(GroupInfo);
// }
//
//
// static
// NET_API_STATUS
// OpenGroupByName(
//      SAM_HANDLE DomainHandle,
//      PUNICODE_STRING GroupName,
//      ULONG DesiredAccess,
//      PSAM_HANDLE GroupHandle,
//      PULONG RelativeId)
// {
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     1,
//                                     GroupName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", GroupName, Status);
//         return NetpNtStatusToApiStatus(Status);
//     }
//
//     /* Fail, if it is not an alias account */
//     if (Use[0] != SidTypeGroup)
//     {
//         ERR("Object is not a group!\n");
//         ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     /* Open the alias account */
//     Status = SamOpenGroup(DomainHandle,
//                           DesiredAccess,
//                           RelativeIds[0],
//                           GroupHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenGroup failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     if (RelativeId != NULL)
//         *RelativeId = RelativeIds[0];
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
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NET_API_STATUS
// WINAPI
// NetGroupAdd(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE buf,
//      LPDWORD parm_err)
// {
//     GROUP_ADM_COMMENT_INFORMATION AdminComment;
//     GROUP_ATTRIBUTE_INFORMATION AttributeInfo;
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     PWSTR Name = NULL;
//     PWSTR Comment = NULL;
//     ULONG Attributes = 0;
//     ULONG RelativeId;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupAdd(%s, %d, %p, %p)\n",
//           debugstr_w(servername), level, buf, parm_err);
//
//     if (parm_err != NULL)
//         *parm_err = PARM_ERROR_NONE;
//
//     /* Initialize the Server name*/
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     /* Initialize the Alias name*/
//     switch (level)
//     {
//         case 0:
//             Name = ((PGROUP_INFO_0)buf)->grpi0_name;
//             Comment = NULL;
//             Attributes = 0;
//             break;
//
//         case 1:
//             Name = ((PGROUP_INFO_1)buf)->grpi1_name;
//             Comment = ((PGROUP_INFO_1)buf)->grpi1_comment;
//             Attributes = 0;
//             break;
//
//         case 2:
//             Name = ((PGROUP_INFO_2)buf)->grpi2_name;
//             Comment = ((PGROUP_INFO_2)buf)->grpi2_comment;
//             Attributes = ((PGROUP_INFO_2)buf)->grpi2_attributes;
//             break;
//
//         case 3:
//             Name = ((PGROUP_INFO_3)buf)->grpi3_name;
//             Comment = ((PGROUP_INFO_3)buf)->grpi3_comment;
//             Attributes = ((PGROUP_INFO_3)buf)->grpi3_attributes;
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     RtlInitUnicodeString(&GroupName, Name);
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
//     /* Open the account domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_CREATE_GROUP | DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Try to open the group account */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_READ_INFORMATION,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus == NERR_Success)
//     {
//         ERR("OpenGroupByName: Group %wZ already exists!\n", &GroupName);
//
//         SamCloseHandle(GroupHandle);
//         ApiStatus = ERROR_GROUP_EXISTS;
//         goto done;
//     }
//
//     ApiStatus = NERR_Success;
//
//     /* Create the group */
//     Status = SamCreateGroupInDomain(DomainHandle,
//                                     &GroupName,
//                                     DELETE | GROUP_WRITE_ACCOUNT,
//                                     &GroupHandle,
//                                     &RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamCreateGroupInDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     TRACE("Created group \"%wZ\" (RID: %lu)\n", &GroupName, RelativeId);
//
//     /* Set the admin comment */
//     if (level == 1 || level == 2 || level == 3)
//     {
//         RtlInitUnicodeString(&AdminComment.AdminComment, Comment);
//
//         Status = SamSetInformationGroup(GroupHandle,
//                                         GroupAdminCommentInformation,
//                                         &AdminComment);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamSetInformationAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//
//             /* Delete the Alias if the Comment could not be set */
//             SamDeleteGroup(GroupHandle);
//
//             goto done;
//         }
//     }
//
//     /* Set the attributes */
//     if (level == 2 || level == 3)
//     {
//         AttributeInfo.Attributes = Attributes;
//
//         Status = SamSetInformationGroup(GroupHandle,
//                                         GroupAttributeInformation,
//                                         &AttributeInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamSetInformationAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//
//             /* Delete the Alias if the Attributes could not be set */
//             SamDeleteGroup(GroupHandle);
//
//             goto done;
//         }
//     }
//
// done:
//     if (GroupHandle != NULL)
//     {
//         if (ApiStatus != NERR_Success)
//             SamDeleteGroup(GroupHandle);
//         else
//             SamCloseHandle(GroupHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupAddUser(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      LPCWSTR username)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupAddUser(%s, %s, %s)\n",
//           debugstr_w(servername), debugstr_w(groupname), debugstr_w(username));
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group account */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_ADD_MEMBER,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     1,
//                                     &UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", &UserName, Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("Object is not a user!\n");
//         ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     Status = SamAddMemberToGroup(GroupHandle,
//                                  RelativeIds[0],
//                                  0);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamAddMemberToGroup failed (Status %lu)\n", Status);
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
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupDel(
//      LPCWSTR servername,
//       LPCWSTR groupname)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupDel(%s, %s)\n",
//           debugstr_w(servername), debugstr_w(groupname));
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 DELETE,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     /* Delete the group */
//     Status = SamDeleteGroup(GroupHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamDeleteGroup failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
// done:
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupDelUser(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      LPCWSTR username)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     UNICODE_STRING UserName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupDelUser(%s, %s, %s)\n",
//           debugstr_w(servername), debugstr_w(groupname), debugstr_w(username));
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group account */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_REMOVE_MEMBER,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     1,
//                                     &UserName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", &UserName, Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use[0] != SidTypeUser)
//     {
//         ERR("Object is not a user!\n");
//         ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     Status = SamRemoveMemberFromGroup(GroupHandle,
//                                       RelativeIds[0]);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamRemoveMemberFromGroup failed (Status %lu)\n", Status);
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
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupEnum(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      PDWORD_PTR resume_handle)
// {
//     UNICODE_STRING ServerName;
//     PSAM_RID_ENUMERATION CurrentGroup;
//     PGROUP_ENUM_CONTEXT EnumContext = NULL;
//     ULONG i;
//     SAM_HANDLE GroupHandle = NULL;
//     PGROUP_GENERAL_INFORMATION GroupInfo = NULL;
//     PVOID Buffer = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupEnum(%s, %d, %p, %d, %p, %p, %p)\n", debugstr_w(servername),
//           level, bufptr, prefmaxlen, entriesread, totalentries, resume_handle);
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
//         EnumContext = (PGROUP_ENUM_CONTEXT)*resume_handle;
//     }
//     else
//     {
//         ApiStatus = NetApiBufferAllocate(sizeof(GROUP_ENUM_CONTEXT), (PVOID*)&EnumContext);
//         if (ApiStatus != NERR_Success)
//             goto done;
//
//         EnumContext->EnumerationContext = 0;
//         EnumContext->Buffer = NULL;
//         EnumContext->Returned = 0;
//         EnumContext->Index = 0;
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
//         Status = OpenAccountDomain(EnumContext->ServerHandle,
//                                    (servername != NULL) ? &ServerName : NULL,
//                                    DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP,
//                                    &EnumContext->AccountDomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         Status = OpenBuiltinDomain(EnumContext->ServerHandle,
//                                    DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP,
//                                    &EnumContext->BuiltinDomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("OpenBuiltinDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         EnumContext->Phase = AccountPhase; //BuiltinPhase;
//         EnumContext->DomainHandle = EnumContext->AccountDomainHandle; //BuiltinDomainHandle;
//     }
//
//
// //    while (TRUE)
// //    {
//         TRACE("EnumContext->Index: %lu\n", EnumContext->Index);
//         TRACE("EnumContext->Returned: %lu\n", EnumContext->Returned);
//
//         if (EnumContext->Index >= EnumContext->Returned)
//         {
//             TRACE("Calling SamEnumerateGroupsInDomain\n");
//
//             Status = SamEnumerateGroupsInDomain(EnumContext->DomainHandle,
//                                                 &EnumContext->EnumerationContext,
//                                                 (PVOID *)&EnumContext->Buffer,
//                                                 prefmaxlen,
//                                                 &EnumContext->Returned);
//
//             TRACE("SamEnumerateGroupsInDomain returned (Status %08lx)\n", Status);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamEnumerateAliasesInDomain failed (Status %08lx)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             if (Status == STATUS_MORE_ENTRIES)
//             {
//                 ApiStatus = NERR_BufTooSmall;
//                 goto done;
//             }
//         }
//
//         TRACE("EnumContext: %lu\n", EnumContext);
//         TRACE("EnumContext->Returned: %lu\n", EnumContext->Returned);
//         TRACE("EnumContext->Buffer: %p\n", EnumContext->Buffer);
//
//         /* Get a pointer to the current group */
//         CurrentGroup = &EnumContext->Buffer[EnumContext->Index];
//
//         TRACE("RID: %lu\n", CurrentGroup->RelativeId);
//
//         Status = SamOpenGroup(EnumContext->DomainHandle,
//                               GROUP_READ_INFORMATION,
//                               CurrentGroup->RelativeId,
//                               &GroupHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamOpenGroup failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         Status = SamQueryInformationGroup(GroupHandle,
//                                           GroupGeneralInformation,
//                                           (PVOID *)&GroupInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamQueryInformationGroup failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         SamCloseHandle(GroupHandle);
//         GroupHandle = NULL;
//
//         TRACE("Name: %S\n", GroupInfo->Name.Buffer);
//         TRACE("Comment: %S\n", GroupInfo->AdminComment.Buffer);
//
//         ApiStatus = BuildGroupInfoBuffer(GroupInfo,
//                                          level,
//                                          CurrentGroup->RelativeId,
//                                          &Buffer);
//         if (ApiStatus != NERR_Success)
//             goto done;
//
//         if (GroupInfo != NULL)
//         {
//             FreeGroupInfo(GroupInfo);
//             GroupInfo = NULL;
//         }
//
//         EnumContext->Index++;
//
//         (*entriesread)++;
//
//         if (EnumContext->Index == EnumContext->Returned)
//         {
//             switch (EnumContext->Phase)
//             {
//                 case BuiltinPhase:
//                     EnumContext->Phase = AccountPhase;
//                     EnumContext->DomainHandle = EnumContext->AccountDomainHandle;
//                     EnumContext->EnumerationContext = 0;
//                     EnumContext->Index = 0;
//                     EnumContext->Returned = 0;
//
//                     if (EnumContext->Buffer != NULL)
//                     {
//                         for (i = 0; i < EnumContext->Returned; i++)
//                         {
//                             SamFreeMemory(EnumContext->Buffer[i].Name.Buffer);
//                         }
//
//                         SamFreeMemory(EnumContext->Buffer);
//                         EnumContext->Buffer = NULL;
//                     }
//                     break;
//
//                 case AccountPhase:
//                 case DonePhase:
//                     EnumContext->Phase = DonePhase;
//                     break;
//             }
//         }
// //    }
//
// done:
//     if (ApiStatus == NERR_Success && EnumContext != NULL && EnumContext->Phase != DonePhase)
//         ApiStatus = ERROR_MORE_DATA;
//
//     if (EnumContext != NULL)
//         *totalentries = EnumContext->Returned;
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
//             if (EnumContext->ServerHandle != NULL)
//                 SamCloseHandle(EnumContext->ServerHandle);
//
//             if (EnumContext->Buffer != NULL)
//             {
//                 for (i = 0; i < EnumContext->Returned; i++)
//                 {
//                     SamFreeMemory(EnumContext->Buffer[i].Name.Buffer);
//                 }
//
//                 SamFreeMemory(EnumContext->Buffer);
//             }
//
//             NetApiBufferFree(EnumContext);
//             EnumContext = NULL;
//         }
//     }
//
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
//
//     if (GroupInfo != NULL)
//         FreeGroupInfo(GroupInfo);
//
//     if (resume_handle != NULL)
//         *resume_handle = (DWORD_PTR)EnumContext;
//
//     *bufptr = (LPBYTE)Buffer;
//
//     TRACE("return %lu\n", ApiStatus);
//
//     return ApiStatus;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetGroupGetInfo(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     PGROUP_GENERAL_INFORMATION GroupInfo = NULL;
//     PVOID Buffer = NULL;
//     ULONG RelativeId;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupGetInfo(%s, %s, %d, %p)\n",
//           debugstr_w(servername), debugstr_w(groupname), level, bufptr);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group account in the account domain */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_READ_INFORMATION,
//                                 &GroupHandle,
//                                 &RelativeId);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     Status = SamQueryInformationGroup(GroupHandle,
//                                       GroupGeneralInformation,
//                                       (PVOID *)&GroupInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamQueryInformationGroup failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     ApiStatus = BuildGroupInfoBuffer(GroupInfo,
//                                      level,
//                                      RelativeId,
//                                      &Buffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
// done:
//     if (GroupInfo != NULL)
//         FreeGroupInfo(GroupInfo);
//
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
//
//     if (DomainHandle != NULL)
//         SamCloseHandle(DomainHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupGetUsers(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      PDWORD_PTR resume_handle)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     PGROUP_USERS_INFO_0 UserInfo0;
//     PGROUP_USERS_INFO_1 UserInfo1;
//     PUSER_ENUM_CONTEXT EnumContext = NULL;
//     PVOID Buffer = NULL;
//     ULONG i, idx, Size;
//     PWSTR Ptr;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupGetUsers(%s, %s, %d, %p, %d, %p, %p, %p)\n",
//           debugstr_w(servername), debugstr_w(groupname), level, bufptr,
//           prefmaxlen, entriesread, totalentries, resume_handle);
//
//     *entriesread = 0;
//     *totalentries = 0;
//     *bufptr = NULL;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
//
//     if (resume_handle != NULL && *resume_handle != 0)
//     {
//         EnumContext = (PUSER_ENUM_CONTEXT)*resume_handle;
//     }
//     else
//     {
//         ApiStatus = NetApiBufferAllocate(sizeof(USER_ENUM_CONTEXT), (PVOID*)&EnumContext);
//         if (ApiStatus != NERR_Success)
//             goto done;
//
//         EnumContext->MemberCount = 0;
//         EnumContext->MemberIds = NULL;
//         EnumContext->Attributes = NULL;
//         EnumContext->Names = NULL;
//         EnumContext->Start = 0;
//         EnumContext->Count = 0;
//
//         /* Connect to the SAM Server */
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
//         /* Open the Acount Domain */
//         Status = OpenAccountDomain(EnumContext->ServerHandle,
//                                    (servername != NULL) ? &ServerName : NULL,
//                                    DOMAIN_LOOKUP,
//                                    &EnumContext->DomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Open the group account */
//         ApiStatus = OpenGroupByName(EnumContext->DomainHandle,
//                                     &GroupName,
//                                     GROUP_LIST_MEMBERS,
//                                     &EnumContext->GroupHandle,
//                                     NULL);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//
//         /* Get the group members */
//         Status = SamGetMembersInGroup(EnumContext->GroupHandle,
//                                       &EnumContext->MemberIds,
//                                       &EnumContext->Attributes,
//                                       &EnumContext->MemberCount);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamGetMembersInGroup failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         if (EnumContext->MemberCount > 0)
//         {
//             /* Get all member names */
//             Status = SamLookupIdsInDomain(EnumContext->DomainHandle,
//                                           EnumContext->MemberCount,
//                                           EnumContext->MemberIds,
//                                           &EnumContext->Names,
//                                           NULL);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamLookupIdsInDomain failed (Status %08lx)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//         }
//     }
//
//     /* Calculate the required buffer size */
//     Size = 0;
//     if (prefmaxlen == -1)
//     {
//         Size = EnumContext->MemberCount *
//                ((level == 0) ? sizeof(GROUP_USERS_INFO_0) : sizeof(GROUP_USERS_INFO_1));
//         for (i = EnumContext->Start; i < EnumContext->MemberCount; i++)
//             Size += EnumContext->Names[i].Length + sizeof(WCHAR);
//
//         EnumContext->Count = EnumContext->MemberCount;
//     }
//     else
//     {
//         for (i = EnumContext->Start; i < EnumContext->MemberCount; i++)
//         {
//             Size += (level == 0) ? sizeof(GROUP_USERS_INFO_0) : sizeof(GROUP_USERS_INFO_1);
//             Size += EnumContext->Names[i].Length + sizeof(WCHAR);
//
//             EnumContext->Count++;
//
//             if (Size >= prefmaxlen)
//                 break;
//         }
//     }
//
//     TRACE("Buffer size: %lu\n", Size);
//
//     /* Allocate and clear the buffer */
//     ApiStatus = NetApiBufferAllocate(Size, &Buffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     ZeroMemory(Buffer, Size);
//
//     /* Fill the buffer */
//     if (level == 0)
//         Ptr = (PWCHAR)((LONG_PTR)Buffer + EnumContext->Count * sizeof(GROUP_USERS_INFO_0));
//     else
//         Ptr = (PWCHAR)((LONG_PTR)Buffer + EnumContext->Count * sizeof(GROUP_USERS_INFO_1));
//
//     for (i = 0; i < EnumContext->Count; i++)
//     {
//         idx = EnumContext->Start + i;
//
//         if (level == 0)
//         {
//             UserInfo0 = (PGROUP_USERS_INFO_0)Buffer;
//
//             UserInfo0[i].grui0_name = Ptr;
//
//             memcpy(UserInfo0[i].grui0_name,
//                    EnumContext->Names[idx].Buffer,
//                    EnumContext->Names[idx].Length);
//             UserInfo0[i].grui0_name[EnumContext->Names[idx].Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (PWSTR)((ULONG_PTR)Ptr + EnumContext->Names[idx].Length + sizeof(WCHAR));
//         }
//         else
//         {
//             UserInfo1 = (PGROUP_USERS_INFO_1)Buffer;
//
//             UserInfo1[i].grui1_name = Ptr;
//
//             memcpy(UserInfo1[i].grui1_name,
//                    EnumContext->Names[idx].Buffer,
//                    EnumContext->Names[idx].Length);
//             UserInfo1[i].grui1_name[EnumContext->Names[idx].Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             UserInfo1[i].grui1_attributes = EnumContext->Attributes[idx];
//
//             Ptr = (PWSTR)((ULONG_PTR)Ptr + EnumContext->Names[idx].Length + sizeof(WCHAR));
//         }
//     }
//
//     /* Set the new start index */
//     EnumContext->Start += EnumContext->Count;
//
//     /* Only return ERROR_MORE_DATA if we are not done yet */
//     if (EnumContext->MemberCount > EnumContext->Start)
//         ApiStatus = ERROR_MORE_DATA;
//     else
//         ApiStatus = NERR_Success;
//
// done:
//     if (EnumContext != NULL)
//     {
//         *entriesread = EnumContext->Count;
//         *totalentries = EnumContext->MemberCount;
//     }
//
//     if (resume_handle == NULL || ApiStatus != ERROR_MORE_DATA)
//     {
//         if (EnumContext != NULL)
//         {
//             if (EnumContext->Names != NULL)
//             {
//                 for (i = 0; i < EnumContext->MemberCount; i++)
//                     SamFreeMemory(EnumContext->Names[i].Buffer);
//
//                 SamFreeMemory(EnumContext->Names);
//             }
//
//             if (EnumContext->Attributes != NULL)
//                 SamFreeMemory(EnumContext->Attributes);
//
//             if (EnumContext->MemberIds != NULL)
//                 SamFreeMemory(EnumContext->MemberIds);
//
//
//             if (EnumContext->GroupHandle != NULL)
//                 SamCloseHandle(EnumContext->GroupHandle);
//
//             if (EnumContext->DomainHandle != NULL)
//                 SamCloseHandle(EnumContext->DomainHandle);
//
//             if (EnumContext->ServerHandle != NULL)
//                 SamCloseHandle(EnumContext->ServerHandle);
//
//             NetApiBufferFree(EnumContext);
//             EnumContext = NULL;
//         }
//     }
//
//     *bufptr = (LPBYTE)Buffer;
//
//     if (resume_handle != NULL)
//         *resume_handle = (DWORD_PTR)EnumContext;
//
//     return ApiStatus;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetGroupSetInfo(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      DWORD level,
//      LPBYTE buf,
//      LPDWORD parm_err)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     GROUP_NAME_INFORMATION GroupNameInfo;
//     GROUP_ADM_COMMENT_INFORMATION AdminCommentInfo;
//     GROUP_ATTRIBUTE_INFORMATION AttributeInfo;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupSetInfo(%s, %s, %d, %p, %p)\n",
//           debugstr_w(servername), debugstr_w(groupname), level, buf, parm_err);
//
//     if (parm_err != NULL)
//         *parm_err = PARM_ERROR_NONE;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_WRITE_ACCOUNT,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus != NERR_Success)
//     {
//         WARN("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     switch (level)
//     {
//         case 0:
//             /* Set the group name */
//             RtlInitUnicodeString(&GroupNameInfo.Name,
//                                  ((PGROUP_INFO_0)buf)->grpi0_name);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupNameInformation,
//                                             &GroupNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 1:
//             /* Set the group name */
//             RtlInitUnicodeString(&GroupNameInfo.Name,
//                                  ((PGROUP_INFO_1)buf)->grpi1_name);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupNameInformation,
//                                             &GroupNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Set the admin comment */
//             RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                  ((PGROUP_INFO_1)buf)->grpi1_comment);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAdminCommentInformation,
//                                             &AdminCommentInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 2:
//             /* Set the group name */
//             RtlInitUnicodeString(&GroupNameInfo.Name,
//                                  ((PGROUP_INFO_2)buf)->grpi2_name);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupNameInformation,
//                                             &GroupNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Set the admin comment */
//             RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                  ((PGROUP_INFO_2)buf)->grpi2_comment);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAdminCommentInformation,
//                                             &AdminCommentInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Set the attributes */
//             AttributeInfo.Attributes = ((PGROUP_INFO_2)buf)->grpi2_attributes;
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAttributeInformation,
//                                             &AttributeInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 3:
//             /* Set the group name */
//             RtlInitUnicodeString(&GroupNameInfo.Name,
//                                  ((PGROUP_INFO_3)buf)->grpi3_name);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupNameInformation,
//                                             &GroupNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Set the admin comment */
//             RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                  ((PGROUP_INFO_3)buf)->grpi3_comment);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAdminCommentInformation,
//                                             &AdminCommentInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Set the attributes */
//             AttributeInfo.Attributes = ((PGROUP_INFO_3)buf)->grpi3_attributes;
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAttributeInformation,
//                                             &AttributeInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 1002:
//             /* Set the admin comment */
//             RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                  ((PGROUP_INFO_1002)buf)->grpi1002_comment);
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAdminCommentInformation,
//                                             &AdminCommentInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 1005:
//             /* Set the attributes */
//             AttributeInfo.Attributes = ((PGROUP_INFO_1005)buf)->grpi1005_attributes;
//
//             Status = SamSetInformationGroup(GroupHandle,
//                                             GroupAttributeInformation,
//                                             &AttributeInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamSetInformationGroup failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         default:
//             ApiStatus = ERROR_INVALID_LEVEL;
//             goto done;
//     }
//
// done:
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
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
// NET_API_STATUS
// WINAPI
// NetGroupSetUsers(
//      LPCWSTR servername,
//      LPCWSTR groupname,
//      DWORD level,
//      LPBYTE buf,
//      DWORD totalentries)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE GroupHandle = NULL;
//     ULONG OldMemberCount = 0;
//     PULONG OldMemberIDs = NULL;
//     PULONG OldAttributes = NULL;
//     PUNICODE_STRING NamesArray = NULL;
//     PGROUP_USERS_INFO_0 UserInfo0;
//     PGROUP_USERS_INFO_1 UserInfo1;
//     PULONG NewMemberIDs = NULL;
//     PSID_NAME_USE NewMemberUse = NULL;
//     ULONG i, j;
//     BOOL Found;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("NetGroupSetUsers(%s, %s, %d, %p, %d) stub!\n",
//           debugstr_w(servername), debugstr_w(groupname), level, buf, totalentries);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&GroupName, groupname);
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
//     /* Open the Acount Domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Open the group account in the account domain */
//     ApiStatus = OpenGroupByName(DomainHandle,
//                                 &GroupName,
//                                 GROUP_LIST_MEMBERS | GROUP_ADD_MEMBER | GROUP_REMOVE_MEMBER,
//                                 &GroupHandle,
//                                 NULL);
//     if (ApiStatus != NERR_Success)
//     {
//         ERR("OpenGroupByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         if (ApiStatus == ERROR_NONE_MAPPED)
//             ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     /* Get the group members */
//     Status = SamGetMembersInGroup(GroupHandle,
//                                   &OldMemberIDs,
//                                   &OldAttributes,
//                                   &OldMemberCount);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamGetMembersInGroup failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     NamesArray = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  totalentries * sizeof(UNICODE_STRING));
//     if (NamesArray == NULL)
//     {
//         ERR("RtlAllocateHeap failed\n");
//         ApiStatus = ERROR_OUTOFMEMORY;
//         goto done;
//     }
//
//     UserInfo0 = (PGROUP_USERS_INFO_0)buf;
//     UserInfo1 = (PGROUP_USERS_INFO_1)buf;
//     for (i = 0; i < totalentries; i++)
//     {
//         if (level == 0)
//             RtlInitUnicodeString(&NamesArray[i], UserInfo0[i].grui0_name);
//         else
//             RtlInitUnicodeString(&NamesArray[i], UserInfo1[i].grui1_name);
//     }
//
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     totalentries,
//                                     NamesArray,
//                                     &NewMemberIDs,
//                                     &NewMemberUse);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain failed (Status %08lx)\n", Status);
//
//         if (Status == STATUS_NONE_MAPPED)
//         {
//             ApiStatus = NERR_UserNotFound;
//             goto done;
//         }
//
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Add members and set attributes for existing members */
//     for (i = 0; i < totalentries; i++)
//     {
//         Found = FALSE;
//         for (j = 0; j < OldMemberCount; j++)
//         {
//             if (NewMemberIDs[i] == OldMemberIDs[j])
//             {
//                 if (level == 1)
//                 {
//                     Status = SamSetMemberAttributesOfGroup(GroupHandle,
//                                                            NewMemberIDs[i],
//                                                            UserInfo1[i].grui1_attributes);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         ERR("SamSetMemberAttributesOfGroup failed (Status %lu)\n", Status);
//                         ApiStatus = NetpNtStatusToApiStatus(Status);
//                         goto done;
//                     }
//                 }
//
//                 Found = TRUE;
//                 break;
//             }
//         }
//
//         if (Found == FALSE)
//         {
//             TRACE("Add member %lx\n", NewMemberIDs[i]);
//
//             if (NewMemberUse[i] != SidTypeUser)
//             {
//                 ERR("New member is not a user!\n");
//                 ApiStatus = NERR_GroupNotFound;
//                 goto done;
//             }
//
//             Status = SamAddMemberToGroup(GroupHandle,
//                                          NewMemberIDs[i],
//                                          (level == 0) ? 0 : UserInfo1[i].grui1_attributes);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamAddMemberToGroup failed (Status %lu)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//         }
//     }
//
//     /* Remove members */
//     for (i = 0; i < OldMemberCount; i++)
//     {
//         Found = FALSE;
//         for (j = 0; j < totalentries; j++)
//         {
//             if (OldMemberIDs[i] == NewMemberIDs[j])
//             {
//                 Found = TRUE;
//                 break;
//             }
//         }
//
//         if (Found == FALSE)
//         {
//             TRACE("Delete member %lx\n", OldMemberIDs[i]);
//
//             Status = SamRemoveMemberFromGroup(GroupHandle,
//                                               OldMemberIDs[i]);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("SamRemoveMemberFromGroup failed (Status %lu)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//         }
//     }
//
// done:
//     if (NewMemberUse != NULL)
//         SamFreeMemory(NewMemberUse);
//
//     if (NewMemberIDs != NULL)
//         SamFreeMemory(NewMemberIDs);
//
//     if (NamesArray != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NamesArray);
//
//     if (OldMemberIDs != NULL)
//         SamFreeMemory(OldMemberIDs);
//
//     if (GroupHandle != NULL)
//         SamCloseHandle(GroupHandle);
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
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
