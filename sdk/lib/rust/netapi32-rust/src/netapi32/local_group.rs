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

// _ENUM_PHASE was a file-local C `enum { BuiltinPhase, AccountPhase, DonePhase }` (int-sized).
// Modelled as an int alias + bare consts so the values work as assignments, comparisons, and
// `match` patterns (C-layout compatible: a C enum is `int`).
pub type _ENUM_PHASE = u32;
pub const BuiltinPhase: _ENUM_PHASE = 0;
pub const AccountPhase: _ENUM_PHASE = 1;
pub const DonePhase: _ENUM_PHASE = 2;
pub type ENUM_PHASE = _ENUM_PHASE;
#[repr(C)]
pub struct _ENUM_CONTEXT {
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
pub type ENUM_CONTEXT = _ENUM_CONTEXT;
pub type PENUM_CONTEXT = *mut _ENUM_CONTEXT;
#[repr(C)]
pub struct _MEMBER_ENUM_CONTEXT {
    pub ServerHandle: SAM_HANDLE,
    pub DomainHandle: SAM_HANDLE,
    pub AliasHandle: SAM_HANDLE,
    pub LsaHandle: LSA_HANDLE,
    pub Sids: *mut PSID,
    pub Count: ULONG,
    pub Domains: PLSA_REFERENCED_DOMAIN_LIST,
    pub Names: PLSA_TRANSLATED_NAME,
}
pub type MEMBER_ENUM_CONTEXT = _MEMBER_ENUM_CONTEXT;
pub type PMEMBER_ENUM_CONTEXT = *mut _MEMBER_ENUM_CONTEXT;
unsafe extern "C" fn BuildAliasInfoBuffer(
    AliasInfo: PALIAS_GENERAL_INFORMATION,
    level: DWORD,
    Buffer: *mut LPVOID,
) -> DWORD {
    let mut LocalBuffer: LPVOID = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success;

    *Buffer = core::ptr::null_mut();

    match level {
        0 => {
            Size = (core::mem::size_of::<LOCALGROUP_INFO_0>()
                + (*AliasInfo).Name.Length as usize
                + core::mem::size_of::<WCHAR>()) as ULONG;
        }
        1 => {
            Size = (core::mem::size_of::<LOCALGROUP_INFO_1>()
                + (*AliasInfo).Name.Length as usize
                + core::mem::size_of::<WCHAR>()
                + (*AliasInfo).AdminComment.Length as usize
                + core::mem::size_of::<WCHAR>()) as ULONG;
        }
        _ => {
            ApiStatus = ERROR_INVALID_LEVEL;
            // Go to done
        }
    }

    if ApiStatus == NERR_Success {
        ApiStatus = NetApiBufferAllocate(Size, &raw mut LocalBuffer);
        if ApiStatus == NERR_Success {
            // ZeroMemory(LocalBuffer, Size)
            core::ptr::write_bytes(LocalBuffer as *mut u8, 0u8, Size as usize);

            match level {
                0 => {
                    let LocalInfo0: PLOCALGROUP_INFO_0 = LocalBuffer as PLOCALGROUP_INFO_0;
                    let Ptr: LPWSTR = (LocalInfo0 as usize + core::mem::size_of::<LOCALGROUP_INFO_0>()) as LPWSTR;
                    (*LocalInfo0).lgrpi0_name = Ptr;

                    // Copy Name
                    core::ptr::copy_nonoverlapping(
                        (*AliasInfo).Name.Buffer,
                        Ptr,
                        ((*AliasInfo).Name.Length as usize) / 2,
                    );
                    *Ptr.add((*AliasInfo).Name.Length as usize / 2) = 0u16; // UNICODE_NULL
                }
                1 => {
                    let LocalInfo1: PLOCALGROUP_INFO_1 = LocalBuffer as PLOCALGROUP_INFO_1;
                    let mut Ptr: LPWSTR = (LocalInfo1 as usize + core::mem::size_of::<LOCALGROUP_INFO_1>()) as LPWSTR;
                    (*LocalInfo1).lgrpi1_name = Ptr;

                    // Copy Name
                    core::ptr::copy_nonoverlapping(
                        (*AliasInfo).Name.Buffer,
                        Ptr,
                        ((*AliasInfo).Name.Length as usize) / 2,
                    );
                    *Ptr.add((*AliasInfo).Name.Length as usize / 2) = 0u16;

                    Ptr = (Ptr as usize
                        + (*AliasInfo).Name.Length as usize
                        + core::mem::size_of::<WCHAR>()) as LPWSTR;
                    (*LocalInfo1).lgrpi1_comment = Ptr;

                    // Copy AdminComment
                    core::ptr::copy_nonoverlapping(
                        (*AliasInfo).AdminComment.Buffer,
                        Ptr,
                        ((*AliasInfo).AdminComment.Length as usize) / 2,
                    );
                    *Ptr.add((*AliasInfo).AdminComment.Length as usize / 2) = 0u16;
                }
                _ => {}
            }
        }
    }

    if ApiStatus == NERR_Success {
        *Buffer = LocalBuffer;
    } else if !LocalBuffer.is_null() {
        NetApiBufferFree(LocalBuffer as PVOID);
    }

    ApiStatus
}
unsafe extern "C" fn FreeAliasInfo(AliasInfo: PALIAS_GENERAL_INFORMATION) {
    if !(*AliasInfo).Name.Buffer.is_null() {
        SamFreeMemory((*AliasInfo).Name.Buffer as *mut core::ffi::c_void);
    }
    if !(*AliasInfo).AdminComment.Buffer.is_null() {
        SamFreeMemory((*AliasInfo).AdminComment.Buffer as *mut core::ffi::c_void);
    }
    SamFreeMemory(AliasInfo as *mut core::ffi::c_void);
}
unsafe extern "C" fn OpenAliasByName(
    DomainHandle: SAM_HANDLE,
    AliasName: PUNICODE_STRING,
    DesiredAccess: ULONG,
    AliasHandle: PSAM_HANDLE,
) -> u32 {
    let mut RelativeIds: *mut ULONG = core::ptr::null_mut();
    let mut Use: PSID_NAME_USE = core::ptr::null_mut();
    let mut ApiStatus: u32 = NERR_Success;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SamLookupNamesInDomain(
        DomainHandle,
        1,
        AliasName,
        &mut RelativeIds,
        &mut Use,
    );
    if !NT_SUCCESS(Status) {
        return NetpNtStatusToApiStatus(Status);
    }

    if (*Use.offset(0) as u32) != (SidTypeAlias as u32) {
        ApiStatus = NERR_GroupNotFound;
    } else {
        Status = SamOpenAlias(
            DomainHandle,
            DesiredAccess,
            *RelativeIds.offset(0),
            AliasHandle,
        );
        if !NT_SUCCESS(Status) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
        }
    }

    if !RelativeIds.is_null() {
        SamFreeMemory(RelativeIds as LPVOID);
    }
    if !Use.is_null() {
        SamFreeMemory(Use as LPVOID);
    }

    ApiStatus
}
unsafe extern "C" fn BuildSidListFromDomainAndName(
    ServerName: *mut _UNICODE_STRING,
    buf: *mut LOCALGROUP_MEMBERS_INFO_3,
    EntryCount: u32,
    MemberList: *mut *mut LOCALGROUP_MEMBERS_INFO_0,
) -> u32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LsaHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut NamesArray: *mut _UNICODE_STRING = core::ptr::null_mut();
    let mut i: u32;
    let mut Domains: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut Sids: PLSA_TRANSLATED_SID = core::ptr::null_mut();
    let mut MemberBuffer: *mut LOCALGROUP_MEMBERS_INFO_0 = core::ptr::null_mut();
    let mut ApiStatus: u32 = 0u32;
    let mut Status: i32 = STATUS_SUCCESS;

    ApiStatus = NetApiBufferAllocate(
        (core::mem::size_of::<_UNICODE_STRING>() as u32).wrapping_mul(EntryCount),
        &raw mut NamesArray as *mut LPVOID,
    );
    if ApiStatus != 0u32 {
        // goto done (outer)
    } else {
        i = 0u32;
        while i < EntryCount {
            RtlInitUnicodeString(
                NamesArray.add(i as usize),
                (*buf.add(i as usize)).lgrmi3_domainandname as LPCWSTR,
            );
            i = i.wrapping_add(1);
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0u32;
        ObjectAttributes.ObjectName = core::ptr::null_mut::<_UNICODE_STRING>();
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

        Status = LsaOpenPolicy(
            ServerName as PLSA_UNICODE_STRING,
            &raw mut ObjectAttributes as PLSA_OBJECT_ATTRIBUTES,
            POLICY_EXECUTE as u32,
            &raw mut LsaHandle,
        ) as i32;
        if !NT_SUCCESS(Status) {
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            Status = LsaLookupNames(
                LsaHandle,
                EntryCount,
                NamesArray as PLSA_UNICODE_STRING,
                &raw mut Domains,
                &raw mut Sids,
            ) as i32;
            if !NT_SUCCESS(Status) {
                ApiStatus = NetpNtStatusToApiStatus(Status);
            } else {
                ApiStatus = NetApiBufferAllocate(
                    (core::mem::size_of::<LOCALGROUP_MEMBERS_INFO_0>() as u32)
                        .wrapping_mul(EntryCount),
                    &raw mut MemberBuffer as *mut LPVOID,
                );
                if ApiStatus == 0u32 {
                    i = 0u32;
                    while i < EntryCount {
                        ApiStatus = BuildSidFromSidAndRid(
                            (*(*Domains)
                                .Domains
                                .add((*Sids.add(i as usize)).DomainIndex as usize))
                            .Sid,
                            (*Sids.add(i as usize)).RelativeId,
                            core::ptr::addr_of_mut!((*MemberBuffer.add(i as usize)).lgrmi0_sid),
                        );
                        if ApiStatus != 0u32 {
                            break;
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }

    if ApiStatus != 0u32 {
        if !MemberBuffer.is_null() {
            i = 0u32;
            while i < EntryCount {
                if !(*MemberBuffer.add(i as usize)).lgrmi0_sid.is_null() {
                    NetApiBufferFree((*MemberBuffer.add(i as usize)).lgrmi0_sid as LPVOID);
                }
                i = i.wrapping_add(1);
            }
            NetApiBufferFree(MemberBuffer as LPVOID);
            MemberBuffer = core::ptr::null_mut();
        }
    }

    if !Sids.is_null() {
        LsaFreeMemory(Sids as LPVOID);
    }
    if !Domains.is_null() {
        LsaFreeMemory(Domains as LPVOID);
    }
    if !LsaHandle.is_null() {
        LsaClose(LsaHandle);
    }
    if !NamesArray.is_null() {
        NetApiBufferFree(NamesArray as LPVOID);
    }

    *MemberList = MemberBuffer;
    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupAdd(
    servername: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> NET_API_STATUS {
    let mut admin_comment: ALIAS_ADM_COMMENT_INFORMATION = core::mem::zeroed();
    let mut server_name: _UNICODE_STRING = core::mem::zeroed();
    let mut alias_name: _UNICODE_STRING = core::mem::zeroed();
    let mut server_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut domain_handle: SAM_HANDLE = core::ptr::null_mut();
    let mut alias_handle: SAM_HANDLE = core::ptr::null_mut();
    let aliasname: LPWSTR;
    let aliascomment: LPWSTR;
    let mut relative_id: ULONG = 0;
    let mut status: NTSTATUS;

    if !parm_err.is_null() {
        *parm_err = 0; // PARM_ERROR_NONE
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut server_name, servername);
    }

    match level {
        0 => {
            aliasname = (*(buf as *const LOCALGROUP_INFO_0)).lgrpi0_name;
            aliascomment = core::ptr::null_mut();
        }
        1 => {
            let info = &*(buf as *const LOCALGROUP_INFO_1);
            aliasname = info.lgrpi1_name;
            aliascomment = info.lgrpi1_comment;
        }
        _ => return ERROR_INVALID_LEVEL as u32,
    }

    RtlInitUnicodeString(&raw mut alias_name, aliasname);

    let api_status = 'outer: {
        let mut api_status = NERR_Success as u32;

        status = SamConnect(
            if !servername.is_null() { &raw mut server_name } else { core::ptr::null_mut() },
            &raw mut server_handle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            api_status = NetpNtStatusToApiStatus(status);
            break 'outer api_status;
        }

        status = OpenBuiltinDomain(server_handle, DOMAIN_LOOKUP as u32, &raw mut domain_handle);
        if !NT_SUCCESS(status) {
            api_status = NetpNtStatusToApiStatus(status);
            break 'outer api_status;
        }

        api_status = OpenAliasByName(
            domain_handle,
            &raw mut alias_name,
            ALIAS_READ_INFORMATION as u32,
            &raw mut alias_handle,
        );
        if api_status == NERR_Success as u32 {
            SamCloseHandle(alias_handle);
            api_status = ERROR_ALIAS_EXISTS as u32;
            break 'outer api_status;
        }

        api_status = NERR_Success as u32;
        SamCloseHandle(domain_handle);
        domain_handle = core::ptr::null_mut();

        status = OpenAccountDomain(
            server_handle,
            if !servername.is_null() { &raw mut server_name } else { core::ptr::null_mut() },
            (DOMAIN_CREATE_ALIAS | DOMAIN_LOOKUP) as u32,
            &raw mut domain_handle,
        );
        if !NT_SUCCESS(status) {
            api_status = NetpNtStatusToApiStatus(status);
            break 'outer api_status;
        }

        status = SamCreateAliasInDomain(
            domain_handle,
            &raw mut alias_name,
            (DELETE | ALIAS_WRITE_ACCOUNT) as u32,
            &raw mut alias_handle,
            &raw mut relative_id,
        );
        if !NT_SUCCESS(status) {
            api_status = NetpNtStatusToApiStatus(status);
            break 'outer api_status;
        }

        if level == 1 {
            RtlInitUnicodeString(&raw mut admin_comment.AdminComment, aliascomment);
            status = SamSetInformationAlias(
                alias_handle,
                AliasAdminCommentInformation,
                &raw mut admin_comment as *mut core::ffi::c_void,
            );
            if !NT_SUCCESS(status) {
                api_status = NetpNtStatusToApiStatus(status);
                SamDeleteAlias(alias_handle);
                break 'outer api_status;
            }
        }

        api_status
    };

    if !alias_handle.is_null() {
        if api_status != NERR_Success as u32 {
            SamDeleteAlias(alias_handle);
        } else {
            SamCloseHandle(alias_handle);
        }
    }
    if !domain_handle.is_null() {
        SamCloseHandle(domain_handle);
    }
    if !server_handle.is_null() {
        SamCloseHandle(server_handle);
    }

    api_status
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupAddMember(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    membersid: PSID,
) -> DWORD {
    let mut Member: LOCALGROUP_MEMBERS_INFO_0 = LOCALGROUP_MEMBERS_INFO_0 {
        lgrmi0_sid: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/local_group.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetLocalGroupAddMember\0" as *const u8 as *const ::core::ffi::c_char,
            503 as ::core::ffi::c_int,
            b"(%s %s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            membersid,
        );
    }
    Member.lgrmi0_sid = membersid;
    return NetLocalGroupAddMembers(
        servername,
        groupname,
        0 as DWORD,
        &raw mut Member as LPBYTE,
        1 as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupAddMembers(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    totalentries: DWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut AliasName: _UNICODE_STRING = core::mem::zeroed();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut MemberList: PLOCALGROUP_MEMBERS_INFO_0 = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Wine debug channel initial check (from c2rust)
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch = __wine_dbch___default;
        let __dbcl = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
            b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
            538i32,
            b"(%s %s %d %p %d)\n\0" as *const u8 as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
            level,
            buf,
            totalentries,
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername);
    }
    RtlInitUnicodeString(&raw mut AliasName, groupname);

    'outer: loop {
        match level {
            0u32 => {
                MemberList = buf as PLOCALGROUP_MEMBERS_INFO_0;
            }
            3u32 => {
                // BuildSidListFromDomainAndName returns a Win32 NET_API_STATUS, not an
                // NTSTATUS. Casting it and testing NT_SUCCESS reads every positive
                // Win32 error (ERROR_NOT_ENOUGH_MEMORY = 8, ...) as SUCCESS, so the
                // failure path was skipped and the caller went on to use MemberList.
                // Take it as an ApiStatus and compare against NERR_Success (0).
                // Upstream-Commit: f2ef434107e (#9113)
                ApiStatus = BuildSidListFromDomainAndName(
                    if !servername.is_null() {
                        &raw mut ServerName
                    } else {
                        core::ptr::null_mut::<_UNICODE_STRING>()
                    },
                    buf as PLOCALGROUP_MEMBERS_INFO_3,
                    totalentries as u32,
                    &raw mut MemberList,
                );
                if ApiStatus != NERR_Success {
                    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32))
                        != 0
                    {
                        let __dbch = __wine_dbch___default;
                        let __dbcl = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                            b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                            558i32,
                            b"BuildSidListFromDomainAndName failed (Status %08lx)\n\0" as *const u8
                                as *const i8,
                            ApiStatus,
                        );
                    }
                    break 'outer;
                }
            }
            _ => {
                ApiStatus = ERROR_INVALID_LEVEL as DWORD;
                break 'outer;
            }
        }

        // Connect to the SAM Server
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
                let __dbch = __wine_dbch___default;
                let __dbcl = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                    b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                    576i32,
                    b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'outer;
        }

        // Open the Builtin Domain
        Status = OpenBuiltinDomain(ServerHandle, DOMAIN_LOOKUP as ULONG, &raw mut DomainHandle);
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch = __wine_dbch___default;
                let __dbcl = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                    b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                    587i32,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'outer;
        }

        // Open the alias in the builtin domain
        ApiStatus = OpenAliasByName(
            DomainHandle,
            &raw mut AliasName,
            ALIAS_ADD_MEMBER as ULONG,
            &raw mut AliasHandle,
        );
        if ApiStatus != NERR_Success as DWORD && ApiStatus != ERROR_NONE_MAPPED as DWORD {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                let __dbch = __wine_dbch___default;
                let __dbcl = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                    b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                    599i32,
                    b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8 as *const i8,
                    &raw mut AliasName,
                    ApiStatus,
                );
            }
            break 'outer;
        }

        if AliasHandle.is_null() {
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }

            // Open the Account Domain
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
                    let __dbch = __wine_dbch___default;
                    let __dbcl = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                        b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                        615i32,
                        b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }

            // Open the alias in the account domain
            ApiStatus = OpenAliasByName(
                DomainHandle,
                &raw mut AliasName,
                ALIAS_ADD_MEMBER as ULONG,
                &raw mut AliasHandle,
            );
            if ApiStatus != NERR_Success as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch = __wine_dbch___default;
                    let __dbcl = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                        b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                        627i32,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8 as *const i8,
                        &raw mut AliasName,
                        ApiStatus,
                    );
                }
                if ApiStatus == ERROR_NONE_MAPPED as DWORD {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
                break 'outer;
            }
        }

        // Add new members to the alias
        i = 0u32;
        while i < totalentries {
            Status = SamAddMemberToAlias(
                AliasHandle,
                (*MemberList.offset(i as isize)).lgrmi0_sid,
            );
            if !NT_SUCCESS(Status) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch = __wine_dbch___default;
                    let __dbcl = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0" as *const u8 as *const i8,
                        b"NetLocalGroupAddMembers\0" as *const u8 as *const i8,
                        641i32,
                        b"SamAddMemberToAlias failed (Status %lu)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }
            i = i.wrapping_add(1);
        }

        break 'outer; // Normal completion
    } // 'outer loop

    // Cleanup
    if level == 3u32 && !MemberList.is_null() {
        i = 0u32;
        while i < totalentries {
            if !(*MemberList.offset(i as isize)).lgrmi0_sid.is_null() {
                NetApiBufferFree((*MemberList.offset(i as isize)).lgrmi0_sid as LPVOID);
            }
            i = i.wrapping_add(1);
        }
        NetApiBufferFree(MemberList as LPVOID);
    }

    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
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
pub unsafe extern "system" fn NetLocalGroupDel(
    servername: LPCWSTR,
    groupname: LPCWSTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut GroupName: _UNICODE_STRING = core::mem::zeroed();
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Entry trace
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const i8,
            b"NetLocalGroupDel\0".as_ptr() as *const i8,
            689i32,
            b"(%s %s)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&mut ServerName, servername as PCWSTR);
    }
    RtlInitUnicodeString(&mut GroupName, groupname as PCWSTR);

    Status = SamConnect(
        if !servername.is_null() { &mut ServerName } else { core::ptr::null_mut() },
        &mut ServerHandle,
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupDel\0".as_ptr() as *const i8,
                703i32,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenBuiltinDomain(ServerHandle, DOMAIN_LOOKUP, &mut DomainHandle);
        if Status < 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupDel\0".as_ptr() as *const i8,
                    714i32,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenAliasByName(
                DomainHandle,
                &mut GroupName,
                DELETE as u32,
                &mut AliasHandle,
            );
            if ApiStatus != NERR_Success as DWORD && ApiStatus != ERROR_NONE_MAPPED as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupDel\0".as_ptr() as *const i8,
                        726i32,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                        &mut GroupName,
                        ApiStatus,
                    );
                }
            } else {
                if AliasHandle.is_null() {
                    if !DomainHandle.is_null() {
                        SamCloseHandle(DomainHandle);
                        DomainHandle = core::ptr::null_mut();
                    }
                    Status = OpenAccountDomain(
                        ServerHandle,
                        if !servername.is_null() { &mut ServerName } else { core::ptr::null_mut() },
                        DOMAIN_LOOKUP,
                        &mut DomainHandle,
                    );
                    if Status < 0 {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_ERR,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/local_group.c\0"
                                    .as_ptr() as *const i8,
                                b"NetLocalGroupDel\0".as_ptr() as *const i8,
                                745i32,
                                b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    } else {
                        ApiStatus = OpenAliasByName(
                            DomainHandle,
                            &mut GroupName,
                            DELETE as u32,
                            &mut AliasHandle,
                        );
                        if ApiStatus != NERR_Success as DWORD {
                            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                                ros_dbg_log(
                                    __WINE_DBCL_ERR,
                                    __wine_dbch___default,
                                    b"dll/win32/netapi32/local_group.c\0"
                                        .as_ptr() as *const i8,
                                    b"NetLocalGroupDel\0".as_ptr() as *const i8,
                                    757i32,
                                    b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                                    &mut GroupName,
                                    ApiStatus,
                                );
                            }
                            if ApiStatus == ERROR_NONE_MAPPED as DWORD {
                                ApiStatus = NERR_GroupNotFound as DWORD;
                            }
                        }
                    }
                }
                // If we still need to delete alias (AliasHandle non-null, or after successful open in account domain)
                if !AliasHandle.is_null() {
                    Status = SamDeleteAlias(AliasHandle);
                    if Status < 0 {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_ERR,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/local_group.c\0"
                                    .as_ptr() as *const i8,
                                b"NetLocalGroupDel\0".as_ptr() as *const i8,
                                768i32,
                                b"SamDeleteAlias failed (Status %08lx)\n\0".as_ptr() as *const i8,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                }
            }
        }
    }

    // cleanup
    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
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
pub unsafe extern "system" fn NetLocalGroupDelMember(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    membersid: PSID,
) -> DWORD {
    let mut Member: LOCALGROUP_MEMBERS_INFO_0 = LOCALGROUP_MEMBERS_INFO_0 {
        lgrmi0_sid: core::ptr::null_mut(),
    };
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/local_group.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetLocalGroupDelMember\0" as *const u8 as *const ::core::ffi::c_char,
            800i32,
            b"(%s %s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            membersid,
        );
    }
    Member.lgrmi0_sid = membersid;
    return NetLocalGroupDelMembers(
        servername,
        groupname,
        0u32,
        core::ptr::addr_of_mut!(Member).cast::<u8>(),
        1u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupDelMembers(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    totalentries: DWORD,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut AliasName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut MemberList: PLOCALGROUP_MEMBERS_INFO_0 = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/local_group.c\0"
                as *const u8 as *const i8,
            b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
            835,
            b"(%s %s %d %p %d)\n\0" as *const u8 as *const i8,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            level,
            buf,
            totalentries,
        );
    }

    'main: loop {
        if !servername.is_null() {
            RtlInitUnicodeString(&raw mut ServerName, servername as PCWSTR);
        }
        RtlInitUnicodeString(&raw mut AliasName, groupname as PCWSTR);

        match level {
            0 => {
                MemberList = buf as PLOCALGROUP_MEMBERS_INFO_0;
            }
            3 => {
                // BuildSidListFromDomainAndName returns a Win32 NET_API_STATUS, not an
                // NTSTATUS. Casting it and testing NT_SUCCESS reads every positive
                // Win32 error (ERROR_NOT_ENOUGH_MEMORY = 8, ...) as SUCCESS, so the
                // failure path was skipped and the caller went on to use MemberList.
                // Take it as an ApiStatus and compare against NERR_Success (0).
                // Upstream-Commit: f2ef434107e (#9113)
                ApiStatus = BuildSidListFromDomainAndName(
                    if !servername.is_null() {
                        &raw mut ServerName
                    } else {
                        core::ptr::null_mut::<_UNICODE_STRING>()
                    },
                    buf as PLOCALGROUP_MEMBERS_INFO_3,
                    totalentries as ULONG,
                    &raw mut MemberList,
                );
                if ApiStatus != NERR_Success {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"dll/win32/netapi32/local_group.c\0"
                                as *const u8 as *const i8,
                            b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                            855,
                            b"BuildSidListFromDomainAndName failed (Status %08lx)\n\0" as *const u8
                                as *const i8,
                            ApiStatus,
                        );
                    }
                    break 'main;
                }
            }
            _ => {
                ApiStatus = ERROR_INVALID_LEVEL as DWORD;
                break 'main;
            }
        }

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
        if !(Status >= 0) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        as *const u8 as *const i8,
                    b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                    873,
                    b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'main;
        }

        Status =
            OpenBuiltinDomain(ServerHandle, DOMAIN_LOOKUP as ULONG, &raw mut DomainHandle);
        if !(Status >= 0) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        as *const u8 as *const i8,
                    b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                    884,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'main;
        }

        ApiStatus = OpenAliasByName(
            DomainHandle,
            &raw mut AliasName,
            ALIAS_REMOVE_MEMBER as ULONG,
            &raw mut AliasHandle,
        );
        if ApiStatus != NERR_Success as DWORD && ApiStatus != ERROR_NONE_MAPPED as DWORD {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        as *const u8 as *const i8,
                    b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                    896,
                    b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8 as *const i8,
                    &raw mut AliasName,
                    ApiStatus,
                );
            }
            break 'main;
        }

        if AliasHandle.is_null() {
            if !DomainHandle.is_null() {
                SamCloseHandle(DomainHandle);
            }
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
            if !(Status >= 0) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            as *const u8 as *const i8,
                        b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                        912,
                        b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'main;
            }

            ApiStatus = OpenAliasByName(
                DomainHandle,
                &raw mut AliasName,
                ALIAS_REMOVE_MEMBER as ULONG,
                &raw mut AliasHandle,
            );
            if ApiStatus != NERR_Success as DWORD {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            as *const u8 as *const i8,
                        b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                        924,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8
                            as *const i8,
                        &raw mut AliasName,
                        ApiStatus,
                    );
                }
                if ApiStatus == ERROR_NONE_MAPPED as DWORD {
                    ApiStatus = NERR_GroupNotFound as DWORD;
                }
                break 'main;
            }
        }

        i = 0;
        while i < totalentries {
            Status = SamRemoveMemberFromAlias(
                AliasHandle,
                (*MemberList.offset(i as isize)).lgrmi0_sid,
            );
            if !(Status >= 0) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            as *const u8 as *const i8,
                        b"NetLocalGroupDelMembers\0" as *const u8 as *const i8,
                        938,
                        b"SamAddMemberToAlias failed (Status %lu)\n\0" as *const u8 as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'main;
            }
            i = i.wrapping_add(1);
        }
        break 'main;
    }

    if level == 3 && !MemberList.is_null() {
        i = 0;
        while i < totalentries {
            if !(*MemberList.offset(i as isize)).lgrmi0_sid.is_null() {
                NetApiBufferFree((*MemberList.offset(i as isize)).lgrmi0_sid as PVOID);
            }
            i = i.wrapping_add(1);
        }
        NetApiBufferFree(MemberList as PVOID);
    }

    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
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
pub unsafe extern "system" fn NetLocalGroupEnum(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resumehandle: PDWORD_PTR,
) -> DWORD {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut CurrentAlias: PSAM_RID_ENUMERATION;
    let mut EnumContext: PENUM_CONTEXT = core::ptr::null_mut();
    let mut i: ULONG;
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasInfo: PALIAS_GENERAL_INFORMATION = core::ptr::null_mut();
    let mut Buffer: LPVOID = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    'done: {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const i8,
            b"NetLocalGroupEnum\0".as_ptr() as *const i8,
            994i32,
            b"(%s %d %p %d %p %p %p) stub!\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const WCHAR),
            level,
            bufptr,
            prefmaxlen,
            entriesread,
            totalentries,
            resumehandle,
        );
    }

    *entriesread = 0u32;
    *totalentries = 0u32;
    *bufptr = core::ptr::null_mut();

    if !servername.is_null() {
        let unicode_ptr: PCWSTR = servername as *const WCHAR as *const u16;
        RtlInitUnicodeString(&raw mut ServerName, unicode_ptr);
    }

    if !resumehandle.is_null() && *resumehandle != 0 {
        EnumContext = *resumehandle as PENUM_CONTEXT;
    } else {
        ApiStatus = NetApiBufferAllocate(
            core::mem::size_of::<ENUM_CONTEXT>() as DWORD,
            &raw mut EnumContext as *mut LPVOID as *mut PVOID,
        );
        if ApiStatus != NERR_Success as DWORD {
            break 'done;
        }

        (*EnumContext).EnumerationContext = 0u64 as SAM_ENUMERATE_HANDLE;
        (*EnumContext).Buffer = core::ptr::null_mut();
        (*EnumContext).Returned = 0u32;
        (*EnumContext).Index = 0u32;

        Status = SamConnect(
            if !servername.is_null() {
                &raw mut ServerName as *mut _UNICODE_STRING
            } else {
                core::ptr::null_mut()
            },
            &raw mut (*EnumContext).ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1024i32,
                    b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        Status = OpenAccountDomain(
            (*EnumContext).ServerHandle,
            if !servername.is_null() {
                &raw mut ServerName as *mut _UNICODE_STRING
            } else {
                core::ptr::null_mut()
            },
            (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as ULONG,
            &raw mut (*EnumContext).AccountDomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1035i32,
                    b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        Status = OpenBuiltinDomain(
            (*EnumContext).ServerHandle,
            (DOMAIN_LIST_ACCOUNTS | DOMAIN_LOOKUP) as ULONG,
            &raw mut (*EnumContext).BuiltinDomainHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1045i32,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        (*EnumContext).Phase = BuiltinPhase;
        (*EnumContext).DomainHandle = (*EnumContext).BuiltinDomainHandle;
    }

    // The while(TRUE) loop body (executed once)
    {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1057i32,
                b"EnumContext->Index: %lu\n\0".as_ptr() as *const i8,
                (*EnumContext).Index,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1058i32,
                b"EnumContext->Returned: %lu\n\0".as_ptr() as *const i8,
                (*EnumContext).Returned,
            );
        }

        if (*EnumContext).Index >= (*EnumContext).Returned {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1062i32,
                    b"Calling SamEnumerateAliasesInDomain\n\0".as_ptr() as *const i8,
                );
            }

            Status = SamEnumerateAliasesInDomain(
                (*EnumContext).DomainHandle,
                &raw mut (*EnumContext).EnumerationContext,
                &raw mut (*EnumContext).Buffer as *mut PVOID,
                prefmaxlen as ULONG,
                &raw mut (*EnumContext).Returned,
            );

            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1070i32,
                    b"SamEnumerateAliasesInDomain returned (Status %08lx)\n\0".as_ptr()
                        as *const i8,
                    Status,
                );
            }

            if !NT_SUCCESS(Status) {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                        1073i32,
                        b"SamEnumerateAliasesInDomain failed (Status %08lx)\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'done;
            }

            if Status == STATUS_MORE_ENTRIES {
                ApiStatus = NERR_BufTooSmall as DWORD;
                break 'done;
            }
        }

        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1085i32,
                b"EnumContext: %lu\n\0".as_ptr() as *const i8,
                EnumContext,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1086i32,
                b"EnumContext->Returned: %lu\n\0".as_ptr() as *const i8,
                (*EnumContext).Returned,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1087i32,
                b"EnumContext->Buffer: %p\n\0".as_ptr() as *const i8,
                (*EnumContext).Buffer,
            );
        }

        CurrentAlias = (*EnumContext).Buffer.offset((*EnumContext).Index as isize);
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1092i32,
                b"RID: %lu\n\0".as_ptr() as *const i8,
                (*CurrentAlias).RelativeId,
            );
        }

        Status = SamOpenAlias(
            (*EnumContext).DomainHandle,
            ALIAS_READ_INFORMATION as ACCESS_MASK,
            (*CurrentAlias).RelativeId,
            &raw mut AliasHandle,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1100i32,
                    b"SamOpenAlias failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        Status = SamQueryInformationAlias(
            AliasHandle,
            AliasGeneralInformation,
            &raw mut AliasInfo as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/local_group.c\0"
                        .as_ptr() as *const i8,
                    b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                    1110i32,
                    b"SamQueryInformationAlias failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
            break 'done;
        }

        SamCloseHandle(AliasHandle);
        AliasHandle = core::ptr::null_mut();

        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1118i32,
                b"Name: %S\n\0".as_ptr() as *const i8,
                (*AliasInfo).Name.Buffer,
            );
        }
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupEnum\0".as_ptr() as *const i8,
                1119i32,
                b"Comment: %S\n\0".as_ptr() as *const i8,
                (*AliasInfo).AdminComment.Buffer,
            );
        }

        ApiStatus = BuildAliasInfoBuffer(AliasInfo, level, &raw mut Buffer);
        if ApiStatus != NERR_Success as DWORD {
            break 'done;
        }

        if !AliasInfo.is_null() {
            FreeAliasInfo(AliasInfo);
            AliasInfo = core::ptr::null_mut();
        }

        (*EnumContext).Index = (*EnumContext).Index.wrapping_add(1);
        *entriesread = (*entriesread).wrapping_add(1);

        if (*EnumContext).Index == (*EnumContext).Returned {
            match (*EnumContext).Phase {
                BuiltinPhase => {
                    (*EnumContext).Phase = AccountPhase;
                    (*EnumContext).DomainHandle = (*EnumContext).AccountDomainHandle;
                    (*EnumContext).EnumerationContext = 0u64 as SAM_ENUMERATE_HANDLE;
                    (*EnumContext).Index = 0u32;
                    (*EnumContext).Returned = 0u32;
                    if !(*EnumContext).Buffer.is_null() {
                        i = 0u32;
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
    }

    } // end 'done block

    if ApiStatus == NERR_Success as DWORD && !EnumContext.is_null()
        && (*EnumContext).Phase != DonePhase
    {
        ApiStatus = ERROR_MORE_DATA as DWORD;
    }

    if !EnumContext.is_null() {
        *totalentries = (*EnumContext).Returned as DWORD;
    }

    if resumehandle.is_null() || ApiStatus != ERROR_MORE_DATA as DWORD {
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
                i = 0u32;
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

    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
    }

    if !AliasInfo.is_null() {
        FreeAliasInfo(AliasInfo);
    }

    if !resumehandle.is_null() {
        *resumehandle = EnumContext as DWORD_PTR as ULONG_PTR;
    }

    *bufptr = Buffer as LPBYTE;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const i8,
            b"NetLocalGroupEnum\0".as_ptr() as *const i8,
            1214i32,
            b"return %lu\n\0".as_ptr() as *const i8,
            ApiStatus,
        );
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupGetInfo(
    servername: *const u16,
    groupname: *const u16,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    let mut current_block: u64;
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
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasInfo: PALIAS_GENERAL_INFORMATION = core::ptr::null_mut::<_ALIAS_GENERAL_INFORMATION>();
    let mut Buffer: LPVOID = core::ptr::null_mut();
    let mut ApiStatus: DWORD = NERR_Success as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/local_group.c\0"
                as *const u8 as *const i8,
            b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
            1242 as i32,
            b"(%s %s %d %p) stub!\n\0" as *const u8 as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
            level,
            bufptr,
        );
    }

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername);
    }
    RtlInitUnicodeString(&raw mut GroupName, groupname);

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

    if !(Status >= 0i32) {
        if (*__wine_dbch___default).flags as i32
            & (1 as i32) << __WINE_DBCL_ERR as i32
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/netapi32/local_group.c\0"
                    as *const u8 as *const i8,
                b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                1256 as i32,
                b"SamConnect failed (Status %08lx)\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        Status = OpenBuiltinDomain(ServerHandle, DOMAIN_LOOKUP as u32, &raw mut DomainHandle);
        if !(Status >= 0i32) {
            if (*__wine_dbch___default).flags as i32
                & (1 as i32) << __WINE_DBCL_ERR as i32
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/netapi32/local_group.c\0"
                        as *const u8 as *const i8,
                    b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                    1267 as i32,
                    b"OpenBuiltinDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                    Status,
                );
            }
            ApiStatus = NetpNtStatusToApiStatus(Status);
        } else {
            ApiStatus = OpenAliasByName(
                DomainHandle,
                &raw mut GroupName,
                ALIAS_READ_INFORMATION as u32,
                &raw mut AliasHandle,
            );
            if ApiStatus != NERR_Success as u32
                && ApiStatus != ERROR_NONE_MAPPED as u32
            {
                if (*__wine_dbch___default).flags as i32
                    & (1 as i32) << __WINE_DBCL_ERR as i32
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/netapi32/local_group.c\0"
                            as *const u8 as *const i8,
                        b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                        1279 as i32,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0" as *const u8 as *const i8,
                        &raw mut GroupName,
                        ApiStatus,
                    );
                }
                // Falls through to done? Actually this branch does not set current_block, it falls to the end where cleanup happens.
            } else {
                if AliasHandle.is_null() {
                    if !DomainHandle.is_null() {
                        SamCloseHandle(DomainHandle);
                    }
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
                    if !(Status >= 0i32) {
                        if (*__wine_dbch___default).flags as i32
                            & (1 as i32) << __WINE_DBCL_ERR as i32
                            != 0
                        {
                            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_3,
                                __dbch_3,
                                b"dll/win32/netapi32/local_group.c\0"
                                    as *const u8 as *const i8,
                                b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                                1295 as i32,
                                b"OpenAccountDomain failed (Status %08lx)\n\0" as *const u8 as *const i8,
                                Status,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                        current_block = 4645269894583499529;
                    } else {
                        ApiStatus = OpenAliasByName(
                            DomainHandle,
                            &raw mut GroupName,
                            ALIAS_READ_INFORMATION as u32,
                            &raw mut AliasHandle,
                        );
                        if ApiStatus != NERR_Success as u32 {
                            if (*__wine_dbch___default).flags as i32
                                & (1 as i32) << __WINE_DBCL_ERR as i32
                                != 0
                            {
                                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                                ros_dbg_log(
                                    __dbcl_4,
                                    __dbch_4,
                                    b"dll/win32/netapi32/local_group.c\0"
                                        as *const u8 as *const i8,
                                    b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                                    1307 as i32,
                                    b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0"
                                        as *const u8 as *const i8,
                                    &raw mut GroupName,
                                    ApiStatus,
                                );
                            }
                            if ApiStatus == ERROR_NONE_MAPPED as u32 {
                                ApiStatus = NERR_GroupNotFound as DWORD;
                            }
                            current_block = 4645269894583499529;
                        } else {
                            current_block = 17747245473264231573;
                        }
                    }
                } else {
                    current_block = 17747245473264231573;
                }
                match current_block {
                    4645269894583499529 => {}
                    _ => {
                        Status = SamQueryInformationAlias(
                            AliasHandle,
                            AliasGeneralInformation,
                            &raw mut AliasInfo as *mut PVOID,
                        );
                        if !(Status >= 0i32) {
                            if (*__wine_dbch___default).flags as i32
                                & (1 as i32) << __WINE_DBCL_ERR as i32
                                != 0
                            {
                                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                                ros_dbg_log(
                                    __dbcl_5,
                                    __dbch_5,
                                    b"dll/win32/netapi32/local_group.c\0"
                                        as *const u8 as *const i8,
                                    b"NetLocalGroupGetInfo\0" as *const u8 as *const i8,
                                    1319 as i32,
                                    b"SamQueryInformationAlias failed (Status %08lx)\n\0"
                                        as *const u8 as *const i8,
                                    Status,
                                );
                            }
                            ApiStatus = NetpNtStatusToApiStatus(Status);
                        } else {
                            ApiStatus = BuildAliasInfoBuffer(AliasInfo, level, &raw mut Buffer);
                        }
                    }
                }
            }
        }
    }

    if !AliasInfo.is_null() {
        FreeAliasInfo(AliasInfo);
    }
    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
    }
    if !DomainHandle.is_null() {
        SamCloseHandle(DomainHandle);
    }
    if !ServerHandle.is_null() {
        SamCloseHandle(ServerHandle);
    }

    *bufptr = Buffer as *mut u8;

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupGetMembers(
    servername: *const u16,
    localgroupname: *const u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resumehandle: PDWORD_PTR,
) -> u32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AliasName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __dbch,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const i8,
            b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
            1382i32,
            b"(%s %s %d %p %d %p %p %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(localgroupname as *const u16),
            level,
            bufptr,
            prefmaxlen,
            entriesread,
            totalentries,
            resumehandle,
        );
    }

    *entriesread = 0u32;
    *totalentries = 0u32;
    *bufptr = core::ptr::null_mut();

    if !servername.is_null() {
        RtlInitUnicodeString(&raw mut ServerName, servername as *const u16);
    }
    RtlInitUnicodeString(&raw mut AliasName, localgroupname as *const u16);

    let mut EnumContext: *mut MEMBER_ENUM_CONTEXT = core::ptr::null_mut();
    let mut Buffer: *mut u8 = core::ptr::null_mut();
    let mut ApiStatus: u32 = 0; // NERR_Success
    let mut Status: i32 = 0; // STATUS_SUCCESS

    'outer: {
        if !resumehandle.is_null() && *resumehandle != 0 {
            EnumContext = *resumehandle as *mut MEMBER_ENUM_CONTEXT;
        } else {
            ApiStatus = NetApiBufferAllocate(
                core::mem::size_of::<MEMBER_ENUM_CONTEXT>() as u32,
                &raw mut EnumContext as *mut *mut c_void,
            );
            if ApiStatus != 0 {
                // NERR_Success
                break 'outer;
            }

            Status = SamConnect(
                if !servername.is_null() {
                    &raw mut ServerName as *mut _UNICODE_STRING
                } else {
                    core::ptr::null_mut()
                },
                &raw mut (*EnumContext).ServerHandle,
                (0x10 | 0x20) as u32, // SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN
                core::ptr::null_mut(),
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                        1411i32,
                        b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }

            Status = OpenBuiltinDomain(
                (*EnumContext).ServerHandle,
                0x0001u32, // DOMAIN_LOOKUP
                &raw mut (*EnumContext).DomainHandle,
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                        1422i32,
                        b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }

            ApiStatus = OpenAliasByName(
                (*EnumContext).DomainHandle,
                &raw mut AliasName,
                0x00000004u32, // ALIAS_LIST_MEMBERS
                &raw mut (*EnumContext).AliasHandle,
            );
            if ApiStatus != 0 && ApiStatus != 1332 {
                // ERROR_NONE_MAPPED
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                        1434i32,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                        &raw mut AliasName,
                        ApiStatus,
                    );
                }
                break 'outer;
            }

            if (*EnumContext).AliasHandle.is_null() {
                if !(*EnumContext).DomainHandle.is_null() {
                    SamCloseHandle((*EnumContext).DomainHandle);
                }
                Status = OpenAccountDomain(
                    (*EnumContext).ServerHandle,
                    if !servername.is_null() {
                        &raw mut ServerName as *mut _UNICODE_STRING
                    } else {
                        core::ptr::null_mut()
                    },
                    0x0001u32, // DOMAIN_LOOKUP
                    &raw mut (*EnumContext).DomainHandle,
                );
                if Status < 0 {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __dbch,
                            b"dll/win32/netapi32/local_group.c\0"
                                .as_ptr() as *const i8,
                            b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                            1450i32,
                            b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    break 'outer;
                }
                ApiStatus = OpenAliasByName(
                    (*EnumContext).DomainHandle,
                    &raw mut AliasName,
                    0x00000004u32, // ALIAS_LIST_MEMBERS
                    &raw mut (*EnumContext).AliasHandle,
                );
                if ApiStatus != 0 {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __dbch,
                            b"dll/win32/netapi32/local_group.c\0"
                                .as_ptr() as *const i8,
                            b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                            1462i32,
                            b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr()
                                as *const i8,
                            &raw mut AliasName,
                            ApiStatus,
                        );
                    }
                    if ApiStatus == 1332 {
                        ApiStatus = 1202; // NERR_GroupNotFound
                    }
                    break 'outer;
                }
            }

            Status = SamGetMembersInAlias(
                (*EnumContext).AliasHandle,
                &raw mut (*EnumContext).Sids,
                &raw mut (*EnumContext).Count,
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                        1475i32,
                        b"SamGetMemberInAlias failed (Status %08lx)\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }
            if (*EnumContext).Count == 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __dbch,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                        1482i32,
                        b"No member found. We're done.\n\0".as_ptr() as *const i8,
                    );
                }
                ApiStatus = 0;
                break 'outer;
            }

            if level != 0 {
                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                ObjectAttributes.RootDirectory = core::ptr::null_mut();
                ObjectAttributes.Attributes = 0;
                ObjectAttributes.ObjectName = core::ptr::null_mut();
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = LsaOpenPolicy(
                    if !servername.is_null() {
                        &raw mut ServerName as PLSA_UNICODE_STRING
                    } else {
                        core::ptr::null_mut()
                    },
                    &raw mut ObjectAttributes as PLSA_OBJECT_ATTRIBUTES,
                    0x00000002u32, // POLICY_EXECUTE
                    &raw mut (*EnumContext).LsaHandle,
                );
                if Status < 0 {
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    break 'outer;
                }
                Status = LsaLookupSids(
                    (*EnumContext).LsaHandle,
                    (*EnumContext).Count,
                    (*EnumContext).Sids,
                    &raw mut (*EnumContext).Domains,
                    &raw mut (*EnumContext).Names,
                );
                if Status < 0 {
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    break 'outer;
                }
            }
        }

        // Calculate required buffer size
        let mut Size: u32 = 0;
        let mut i: usize = 0;
        while i < (*EnumContext).Count as usize {
            match level {
                0 => {
                    Size = Size.wrapping_add(
                        core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_0>() as u32
                            + RtlLengthSid(*(*EnumContext).Sids.add(i)) as u32,
                    );
                }
                1 => {
                    let name_len = (*(*EnumContext).Names.add(i)).Name.Length as u32;
                    Size = Size.wrapping_add(
                        core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_1>() as u32
                            + RtlLengthSid(*(*EnumContext).Sids.add(i)) as u32
                            + name_len
                            + core::mem::size_of::<u16>() as u32,
                    );
                }
                2 => {
                    let name_len = (*(*EnumContext).Names.add(i)).Name.Length as u32;
                    Size = Size.wrapping_add(
                        core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_2>() as u32
                            + RtlLengthSid(*(*EnumContext).Sids.add(i)) as u32
                            + name_len
                            + core::mem::size_of::<u16>() as u32,
                    );
                    if (*(*EnumContext).Names.add(i)).DomainIndex >= 0 {
                        let domain_idx = (*(*EnumContext).Names.add(i)).DomainIndex as usize;
                        let domain_len = (*(*(*EnumContext).Domains).Domains.add(domain_idx))
                            .Name.Length as u32;
                        Size = Size.wrapping_add(domain_len + core::mem::size_of::<u16>() as u32);
                    }
                }
                3 => {
                    let name_len = (*(*EnumContext).Names.add(i)).Name.Length as u32;
                    Size = Size.wrapping_add(
                        core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_3>() as u32
                            + name_len
                            + core::mem::size_of::<u16>() as u32,
                    );
                    if (*(*EnumContext).Names.add(i)).DomainIndex >= 0 {
                        let domain_idx = (*(*EnumContext).Names.add(i)).DomainIndex as usize;
                        let domain_len = (*(*(*EnumContext).Domains).Domains.add(domain_idx))
                            .Name.Length as u32;
                        Size = Size.wrapping_add(domain_len + core::mem::size_of::<u16>() as u32);
                    }
                }
                _ => {
                    ApiStatus = 124; // ERROR_INVALID_LEVEL
                    break 'outer;
                }
            }
            i = i.wrapping_add(1);
        }

        ApiStatus = NetApiBufferAllocate(Size, &raw mut Buffer as *mut *mut c_void);
        if ApiStatus != 0 {
            break 'outer;
        }
        core::ptr::write_bytes(Buffer, 0u8, Size as usize);

        let mut MembersInfo0: *mut _LOCALGROUP_MEMBERS_INFO_0 = core::ptr::null_mut();
        let mut MembersInfo1: *mut _LOCALGROUP_MEMBERS_INFO_1 = core::ptr::null_mut();
        let mut MembersInfo2: *mut _LOCALGROUP_MEMBERS_INFO_2 = core::ptr::null_mut();
        let mut MembersInfo3: *mut _LOCALGROUP_MEMBERS_INFO_3 = core::ptr::null_mut();
        let mut Ptr: *mut u16 = core::ptr::null_mut();

        match level {
            0 => {
                MembersInfo0 = Buffer as *mut _LOCALGROUP_MEMBERS_INFO_0;
                Ptr = (Buffer as usize
                    + core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_0>()
                        * (*EnumContext).Count as usize)
                    as *mut u16;
            }
            1 => {
                MembersInfo1 = Buffer as *mut _LOCALGROUP_MEMBERS_INFO_1;
                Ptr = (Buffer as usize
                    + core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_1>()
                        * (*EnumContext).Count as usize)
                    as *mut u16;
            }
            2 => {
                MembersInfo2 = Buffer as *mut _LOCALGROUP_MEMBERS_INFO_2;
                Ptr = (Buffer as usize
                    + core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_2>()
                        * (*EnumContext).Count as usize)
                    as *mut u16;
            }
            3 => {
                MembersInfo3 = Buffer as *mut _LOCALGROUP_MEMBERS_INFO_3;
                Ptr = (Buffer as usize
                    + core::mem::size_of::<_LOCALGROUP_MEMBERS_INFO_3>()
                        * (*EnumContext).Count as usize)
                    as *mut u16;
            }
            _ => {}
        }

        let mut i: usize = 0;
        while i < (*EnumContext).Count as usize {
            match level {
                0 => {
                    (*MembersInfo0).lgrmi0_sid = Ptr as *mut c_void;
                    let SidLength = RtlLengthSid(*(*EnumContext).Sids.add(i));
                    memcpy(
                        (*MembersInfo0).lgrmi0_sid as *mut core::ffi::c_void,
                        *(*EnumContext).Sids.add(i) as *const core::ffi::c_void,
                        SidLength as usize,
                    );
                    Ptr = (Ptr as usize + SidLength as usize) as *mut u16;
                    MembersInfo0 = MembersInfo0.add(1);
                }
                1 => {
                    (*MembersInfo1).lgrmi1_sid = Ptr as *mut c_void;
                    let SidLength = RtlLengthSid(*(*EnumContext).Sids.add(i));
                    memcpy(
                        (*MembersInfo1).lgrmi1_sid as *mut core::ffi::c_void,
                        *(*EnumContext).Sids.add(i) as *const core::ffi::c_void,
                        SidLength as usize,
                    );
                    Ptr = (Ptr as usize + SidLength as usize) as *mut u16;
                    (*MembersInfo1).lgrmi1_sidusage = (*(*EnumContext).Names.add(i)).Use;

                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __dbch,
                            b"dll/win32/netapi32/local_group.c\0"
                                .as_ptr() as *const i8,
                            b"NetLocalGroupGetMembers\0".as_ptr() as *const i8,
                            1614i32,
                            b"Name: %S\n\0".as_ptr() as *const i8,
                            (*(*EnumContext).Names.add(i)).Name.Buffer,
                        );
                    }

                    (*MembersInfo1).lgrmi1_name = Ptr;
                    memcpy(
                        (*MembersInfo1).lgrmi1_name as *mut core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Buffer
                            as *const core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Length as usize,
                    );
                    Ptr = (Ptr as usize
                        + (*(*EnumContext).Names.add(i)).Name.Length as usize
                        + core::mem::size_of::<u16>()) as *mut u16;
                    MembersInfo1 = MembersInfo1.add(1);
                }
                2 => {
                    (*MembersInfo2).lgrmi2_sid = Ptr as *mut c_void;
                    let SidLength = RtlLengthSid(*(*EnumContext).Sids.add(i));
                    memcpy(
                        (*MembersInfo2).lgrmi2_sid as *mut core::ffi::c_void,
                        *(*EnumContext).Sids.add(i) as *const core::ffi::c_void,
                        SidLength as usize,
                    );
                    Ptr = (Ptr as usize + SidLength as usize) as *mut u16;
                    (*MembersInfo2).lgrmi2_sidusage = (*(*EnumContext).Names.add(i)).Use;
                    (*MembersInfo2).lgrmi2_domainandname = Ptr;

                    let domain_idx = (*(*EnumContext).Names.add(i)).DomainIndex;
                    if domain_idx >= 0 {
                        let domain = &(*(*(*EnumContext).Domains).Domains.add(domain_idx as usize));
                        memcpy(
                            (*MembersInfo2).lgrmi2_domainandname as *mut core::ffi::c_void,
                            domain.Name.Buffer as *const core::ffi::c_void,
                            domain.Name.Length as usize,
                        );
                        Ptr = (Ptr as usize + domain.Name.Length as usize) as *mut u16;
                        *Ptr = '\\' as u16;
                        Ptr = (Ptr as usize + core::mem::size_of::<u16>()) as *mut u16;
                    }
                    memcpy(
                        Ptr as *mut core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Buffer
                            as *const core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Length as usize,
                    );
                    Ptr = (Ptr as usize
                        + (*(*EnumContext).Names.add(i)).Name.Length as usize
                        + core::mem::size_of::<u16>()) as *mut u16;
                    MembersInfo2 = MembersInfo2.add(1);
                }
                3 => {
                    (*MembersInfo3).lgrmi3_domainandname = Ptr;
                    let domain_idx = (*(*EnumContext).Names.add(i)).DomainIndex;
                    if domain_idx >= 0 {
                        let domain = &(*(*(*EnumContext).Domains).Domains.add(domain_idx as usize));
                        memcpy(
                            (*MembersInfo3).lgrmi3_domainandname as *mut core::ffi::c_void,
                            domain.Name.Buffer as *const core::ffi::c_void,
                            domain.Name.Length as usize,
                        );
                        Ptr = (Ptr as usize + domain.Name.Length as usize) as *mut u16;
                        *Ptr = '\\' as u16;
                        Ptr = (Ptr as usize + core::mem::size_of::<u16>()) as *mut u16;
                    }
                    memcpy(
                        Ptr as *mut core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Buffer
                            as *const core::ffi::c_void,
                        (*(*EnumContext).Names.add(i)).Name.Length as usize,
                    );
                    Ptr = (Ptr as usize
                        + (*(*EnumContext).Names.add(i)).Name.Length as usize
                        + core::mem::size_of::<u16>()) as *mut u16;
                    MembersInfo3 = MembersInfo3.add(1);
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }

        *entriesread = (*EnumContext).Count as u32;
        *bufptr = Buffer as *mut u8;
    }

    if !EnumContext.is_null() {
        *totalentries = (*EnumContext).Count as u32;
    }

    if resumehandle.is_null() || ApiStatus != 234 {
        // ERROR_MORE_DATA
        if !EnumContext.is_null() {
            if !(*EnumContext).LsaHandle.is_null() {
                LsaClose((*EnumContext).LsaHandle);
            }
            if !(*EnumContext).AliasHandle.is_null() {
                SamCloseHandle((*EnumContext).AliasHandle);
            }
            if !(*EnumContext).DomainHandle.is_null() {
                SamCloseHandle((*EnumContext).DomainHandle);
            }
            if !(*EnumContext).ServerHandle.is_null() {
                SamCloseHandle((*EnumContext).ServerHandle);
            }
            if !(*EnumContext).Sids.is_null() {
                SamFreeMemory((*EnumContext).Sids as *mut c_void);
            }
            if !(*EnumContext).Domains.is_null() {
                LsaFreeMemory((*EnumContext).Domains as *mut c_void);
            }
            if !(*EnumContext).Names.is_null() {
                LsaFreeMemory((*EnumContext).Names as *mut c_void);
            }
            NetApiBufferFree(EnumContext as *mut c_void);
        }
    }

    ApiStatus
}

#[no_mangle]
pub unsafe extern "system" fn NetLocalGroupSetInfo(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> u32 {
    let mut ServerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AliasName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServerHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasHandle: SAM_HANDLE = core::ptr::null_mut();
    let mut AliasNameInfo: ALIAS_NAME_INFORMATION = ALIAS_NAME_INFORMATION {
        Name: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut AdminCommentInfo: ALIAS_ADM_COMMENT_INFORMATION = ALIAS_ADM_COMMENT_INFORMATION {
        AdminComment: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut ApiStatus: u32 = NERR_Success;
    let mut Status: i32 = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const i8,
            b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
            1750i32,
            b"(%s %s %d %p %p)\n\0".as_ptr() as *const i8,
            debugstr_w(servername as *const u16),
            debugstr_w(groupname as *const u16),
            level,
            buf as *const core::ffi::c_void,
            parm_err as *const core::ffi::c_void,
        );
    }

    if !parm_err.is_null() {
        *parm_err = PARM_ERROR_NONE;
    }
    if !servername.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ServerName), servername);
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(AliasName), groupname);

    let server_name_ptr = if !servername.is_null() {
        core::ptr::addr_of_mut!(ServerName)
    } else {
        core::ptr::null_mut::<_UNICODE_STRING>()
    };
    Status = SamConnect(
        server_name_ptr,
        core::ptr::addr_of_mut!(ServerHandle),
        (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as u32,
        core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
    );
    if Status < 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/netapi32/local_group.c\0"
                    .as_ptr() as *const i8,
                b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                1767i32,
                b"SamConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        ApiStatus = NetpNtStatusToApiStatus(Status);
    } else {
        'outer: loop {
            Status = OpenBuiltinDomain(ServerHandle, DOMAIN_LOOKUP, core::ptr::addr_of_mut!(DomainHandle));
            if Status < 0 {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                        1778i32,
                        b"OpenBuiltinDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
                ApiStatus = NetpNtStatusToApiStatus(Status);
                break 'outer;
            }

            ApiStatus = OpenAliasByName(
                DomainHandle,
                core::ptr::addr_of_mut!(AliasName),
                ALIAS_WRITE_ACCOUNT,
                core::ptr::addr_of_mut!(AliasHandle),
            );
            if ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/local_group.c\0"
                            .as_ptr() as *const i8,
                        b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                        1790i32,
                        b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                        core::ptr::addr_of_mut!(AliasName) as *const core::ffi::c_void,
                        ApiStatus,
                    );
                }
                break 'outer;
            }

            if AliasHandle.is_null() {
                if !DomainHandle.is_null() {
                    SamCloseHandle(DomainHandle);
                }
                let account_server_name = if !servername.is_null() {
                    core::ptr::addr_of_mut!(ServerName)
                } else {
                    core::ptr::null_mut::<_UNICODE_STRING>()
                };
                Status = OpenAccountDomain(
                    ServerHandle,
                    account_server_name,
                    DOMAIN_LOOKUP,
                    core::ptr::addr_of_mut!(DomainHandle),
                );
                if Status < 0 {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"dll/win32/netapi32/local_group.c\0"
                                .as_ptr() as *const i8,
                            b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                            1806i32,
                            b"OpenAccountDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                            Status,
                        );
                    }
                    ApiStatus = NetpNtStatusToApiStatus(Status);
                    break 'outer;
                }

                ApiStatus = OpenAliasByName(
                    DomainHandle,
                    core::ptr::addr_of_mut!(AliasName),
                    ALIAS_WRITE_ACCOUNT,
                    core::ptr::addr_of_mut!(AliasHandle),
                );
                if ApiStatus != NERR_Success {
                    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"dll/win32/netapi32/local_group.c\0"
                                .as_ptr() as *const i8,
                            b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                            1818i32,
                            b"OpenAliasByName(%wZ) failed (ApiStatus %lu)\n\0".as_ptr() as *const i8,
                            core::ptr::addr_of_mut!(AliasName) as *const core::ffi::c_void,
                            ApiStatus,
                        );
                    }
                    if ApiStatus == ERROR_NONE_MAPPED {
                        ApiStatus = NERR_GroupNotFound;
                    }
                    break 'outer;
                }
            }

            match level {
                0 => {
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(AliasNameInfo.Name),
                        (*(buf as PLOCALGROUP_INFO_0)).lgrpi0_name,
                    );
                    Status = SamSetInformationAlias(
                        AliasHandle,
                        AliasNameInformation,
                        core::ptr::addr_of_mut!(AliasNameInfo) as *mut core::ffi::c_void,
                    );
                    if Status < 0 {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_TRACE,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/local_group.c\0"
                                    .as_ptr() as *const i8,
                                b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                                1837i32,
                                b"SamSetInformationAlias failed (ApiStatus %lu)\n\0".as_ptr()
                                    as *const i8,
                                ApiStatus,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                }
                1 | 1002 => {
                    if level == 1 {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                            (*(buf as PLOCALGROUP_INFO_1)).lgrpi1_comment,
                        );
                    } else {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(AdminCommentInfo.AdminComment),
                            (*(buf as PLOCALGROUP_INFO_1002)).lgrpi1002_comment,
                        );
                    }
                    Status = SamSetInformationAlias(
                        AliasHandle,
                        AliasAdminCommentInformation,
                        core::ptr::addr_of_mut!(AdminCommentInfo) as *mut core::ffi::c_void,
                    );
                    if Status < 0 {
                        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_TRACE,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/local_group.c\0"
                                    .as_ptr() as *const i8,
                                b"NetLocalGroupSetInfo\0".as_ptr() as *const i8,
                                1858i32,
                                b"SamSetInformationAlias failed (ApiStatus %lu)\n\0".as_ptr()
                                    as *const i8,
                                ApiStatus,
                            );
                        }
                        ApiStatus = NetpNtStatusToApiStatus(Status);
                    }
                }
                _ => {
                    ApiStatus = ERROR_INVALID_LEVEL;
                    break 'outer;
                }
            }
            break 'outer;
        }
    }

    if !AliasHandle.is_null() {
        SamCloseHandle(AliasHandle);
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
pub unsafe extern "system" fn NetLocalGroupSetMembers(
    servername: LPCWSTR,
    groupname: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    totalentries: DWORD,
) -> DWORD {
    // WINE_DBCL_FIXME check and logging
    if (*(__wine_dbch___default as *const __wine_debug_channel)).flags as u32
        & (1u32 << __WINE_DBCL_FIXME as u32)
        != 0
    {
        let dbcl = __WINE_DBCL_FIXME;
        let dbch = __wine_dbch___default;
        ros_dbg_log(
            dbcl,
            dbch,
            b"dll/win32/netapi32/local_group.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetLocalGroupSetMembers\0".as_ptr() as *const core::ffi::c_char,
            1896i32,
            b"(%s %s %d %p %d) stub!\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_w(servername as *const WCHAR),
            debugstr_w(groupname as *const WCHAR),
            level,
            buf,
            totalentries,
        );
    }
    0u32 // NERR_Success
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2006 Robert Reif
//  *
//  * netapi32 local group functions
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
// typedef struct _ENUM_CONTEXT
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
//
// } ENUM_CONTEXT, *PENUM_CONTEXT;
//
// typedef struct _MEMBER_ENUM_CONTEXT
// {
//     SAM_HANDLE ServerHandle;
//     SAM_HANDLE DomainHandle;
//     SAM_HANDLE AliasHandle;
//     LSA_HANDLE LsaHandle;
//
//     PSID *Sids;
//     ULONG Count;
//     PLSA_REFERENCED_DOMAIN_LIST Domains;
//     PLSA_TRANSLATED_NAME Names;
//
// } MEMBER_ENUM_CONTEXT, *PMEMBER_ENUM_CONTEXT;
//
//
// static
// NET_API_STATUS
// BuildAliasInfoBuffer(PALIAS_GENERAL_INFORMATION AliasInfo,
//                      DWORD level,
//                      LPVOID *Buffer)
// {
//     LPVOID LocalBuffer = NULL;
//     PLOCALGROUP_INFO_0 LocalInfo0;
//     PLOCALGROUP_INFO_1 LocalInfo1;
//     LPWSTR Ptr;
//     ULONG Size = 0;
//     NET_API_STATUS ApiStatus = NERR_Success;
//
//     *Buffer = NULL;
//
//     switch (level)
//     {
//         case 0:
//             Size = sizeof(LOCALGROUP_INFO_0) +
//                    AliasInfo->Name.Length + sizeof(WCHAR);
//             break;
//
//         case 1:
//             Size = sizeof(LOCALGROUP_INFO_1) +
//                    AliasInfo->Name.Length + sizeof(WCHAR) +
//                    AliasInfo->AdminComment.Length + sizeof(WCHAR);
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
//             LocalInfo0 = (PLOCALGROUP_INFO_0)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)LocalInfo0 + sizeof(LOCALGROUP_INFO_0));
//             LocalInfo0->lgrpi0_name = Ptr;
//
//             memcpy(LocalInfo0->lgrpi0_name,
//                    AliasInfo->Name.Buffer,
//                    AliasInfo->Name.Length);
//             LocalInfo0->lgrpi0_name[AliasInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//
//         case 1:
//             LocalInfo1 = (PLOCALGROUP_INFO_1)LocalBuffer;
//
//             Ptr = (LPWSTR)((ULONG_PTR)LocalInfo1 + sizeof(LOCALGROUP_INFO_1));
//             LocalInfo1->lgrpi1_name = Ptr;
//
//             memcpy(LocalInfo1->lgrpi1_name,
//                    AliasInfo->Name.Buffer,
//                    AliasInfo->Name.Length);
//             LocalInfo1->lgrpi1_name[AliasInfo->Name.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             Ptr = (LPWSTR)((ULONG_PTR)Ptr + AliasInfo->Name.Length + sizeof(WCHAR));
//             LocalInfo1->lgrpi1_comment = Ptr;
//
//             memcpy(LocalInfo1->lgrpi1_comment,
//                    AliasInfo->AdminComment.Buffer,
//                    AliasInfo->AdminComment.Length);
//             LocalInfo1->lgrpi1_comment[AliasInfo->AdminComment.Length / sizeof(WCHAR)] = UNICODE_NULL;
//             break;
//     }
//
// done:
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
// VOID
// FreeAliasInfo(PALIAS_GENERAL_INFORMATION AliasInfo)
// {
//     if (AliasInfo->Name.Buffer != NULL)
//         SamFreeMemory(AliasInfo->Name.Buffer);
//
//     if (AliasInfo->AdminComment.Buffer != NULL)
//         SamFreeMemory(AliasInfo->AdminComment.Buffer);
//
//     SamFreeMemory(AliasInfo);
// }
//
//
// static
// NET_API_STATUS
// OpenAliasByName(SAM_HANDLE DomainHandle,
//                 PUNICODE_STRING AliasName,
//                 ULONG DesiredAccess,
//                 PSAM_HANDLE AliasHandle)
// {
//     PULONG RelativeIds = NULL;
//     PSID_NAME_USE Use = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Get the RID for the given user name */
//     Status = SamLookupNamesInDomain(DomainHandle,
//                                     1,
//                                     AliasName,
//                                     &RelativeIds,
//                                     &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamLookupNamesInDomain(%wZ) failed (Status %08lx)\n", AliasName, Status);
//         return NetpNtStatusToApiStatus(Status);
//     }
//
//     /* Fail, if it is not an alias account */
//     if (Use[0] != SidTypeAlias)
//     {
//         ERR("Object is not an Alias!\n");
//         ApiStatus = NERR_GroupNotFound;
//         goto done;
//     }
//
//     /* Open the alias account */
//     Status = SamOpenAlias(DomainHandle,
//                           DesiredAccess,
//                           RelativeIds[0],
//                           AliasHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
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
// static
// NET_API_STATUS
// BuildSidListFromDomainAndName( PUNICODE_STRING ServerName,
//                                PLOCALGROUP_MEMBERS_INFO_3 buf,
//                                ULONG EntryCount,
//                                PLOCALGROUP_MEMBERS_INFO_0 *MemberList)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE LsaHandle = NULL;
//     PUNICODE_STRING NamesArray = NULL;
//     ULONG i;
//     PLSA_REFERENCED_DOMAIN_LIST Domains = NULL;
//     PLSA_TRANSLATED_SID Sids = NULL;
//     PLOCALGROUP_MEMBERS_INFO_0 MemberBuffer = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     ApiStatus = NetApiBufferAllocate(sizeof(UNICODE_STRING) * EntryCount,
//                                      (LPVOID*)&NamesArray);
//     if (ApiStatus != NERR_Success)
//     {
//         goto done;
//     }
//
//     for (i = 0; i < EntryCount; i++)
//     {
//         RtlInitUnicodeString(&NamesArray[i],
//                              buf[i].lgrmi3_domainandname);
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                0,
//                                NULL);
//
//     Status = LsaOpenPolicy(ServerName,
//                            (PLSA_OBJECT_ATTRIBUTES)&ObjectAttributes,
//                            POLICY_EXECUTE,
//                            &LsaHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     Status = LsaLookupNames(LsaHandle,
//                             EntryCount,
//                             NamesArray,
//                             &Domains,
//                             &Sids);
//     if (!NT_SUCCESS(Status))
//     {
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     ApiStatus = NetApiBufferAllocate(sizeof(LOCALGROUP_MEMBERS_INFO_0) * EntryCount,
//                                      (LPVOID*)&MemberBuffer);
//     if (ApiStatus != NERR_Success)
//     {
//         goto done;
//     }
//
//     for (i = 0; i < EntryCount; i++)
//     {
//         ApiStatus = BuildSidFromSidAndRid(Domains->Domains[Sids[i].DomainIndex].Sid,
//                                           Sids[i].RelativeId,
//                                           &MemberBuffer[i].lgrmi0_sid);
//         if (ApiStatus != NERR_Success)
//         {
//             goto done;
//         }
//     }
//
// done:
//     if (ApiStatus != NERR_Success)
//     {
//         if (MemberBuffer != NULL)
//         {
//             for (i = 0; i < EntryCount; i++)
//             {
//                 if (MemberBuffer[i].lgrmi0_sid != NULL)
//                     NetApiBufferFree(MemberBuffer[i].lgrmi0_sid);
//             }
//
//             NetApiBufferFree(MemberBuffer);
//             MemberBuffer = NULL;
//         }
//     }
//
//     if (Sids != NULL)
//         LsaFreeMemory(Sids);
//
//     if (Domains != NULL)
//         LsaFreeMemory(Domains);
//
//     if (LsaHandle != NULL)
//         LsaClose(LsaHandle);
//
//     if (NamesArray != NULL)
//         NetApiBufferFree(NamesArray);
//
//     *MemberList = MemberBuffer;
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetLocalGroupAdd  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupAdd(
//     LPCWSTR servername,
//     DWORD level,
//     LPBYTE buf,
//     LPDWORD parm_err)
// {
//     ALIAS_ADM_COMMENT_INFORMATION AdminComment;
//     UNICODE_STRING ServerName;
//     UNICODE_STRING AliasName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     LPWSTR aliasname = NULL;
//     LPWSTR aliascomment = NULL;
//     ULONG RelativeId;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %d %p %p) stub!\n", debugstr_w(servername), level, buf,
//           parm_err);
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
//             aliasname = ((PLOCALGROUP_INFO_0)buf)->lgrpi0_name;
//             aliascomment = NULL;
//             break;
//
//         case 1:
//             aliasname = ((PLOCALGROUP_INFO_1)buf)->lgrpi1_name;
//             aliascomment = ((PLOCALGROUP_INFO_1)buf)->lgrpi1_comment;
//             break;
//
//         default:
//             return ERROR_INVALID_LEVEL;
//     }
//
//     RtlInitUnicodeString(&AliasName, aliasname);
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
//     /* Try to open the Alias Account in the Builtin Domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &AliasName,
//                                 ALIAS_READ_INFORMATION,
//                                 &AliasHandle);
//     if (ApiStatus == NERR_Success)
//     {
//         ERR("OpenAliasByName: alias %wZ already exists!\n", &AliasName);
//
//         SamCloseHandle(AliasHandle);
//         ApiStatus = ERROR_ALIAS_EXISTS;
//         goto done;
//     }
//
//     ApiStatus = NERR_Success;
//
//     /* Close the Builtin Domain */
//     SamCloseHandle(DomainHandle);
//     DomainHandle = NULL;
//
//     /* Open the account domain */
//     Status = OpenAccountDomain(ServerHandle,
//                                (servername != NULL) ? &ServerName : NULL,
//                                DOMAIN_CREATE_ALIAS | DOMAIN_LOOKUP,
//                                &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamOpenDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     /* Create the alias */
//     Status = SamCreateAliasInDomain(DomainHandle,
//                                     &AliasName,
//                                     DELETE | ALIAS_WRITE_ACCOUNT,
//                                     &AliasHandle,
//                                     &RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamCreateAliasInDomain failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     TRACE("Created alias \"%wZ\" (RID: %lu)\n", &AliasName, RelativeId);
//
//     /* Set the admin comment */
//     if (level == 1)
//     {
//         RtlInitUnicodeString(&AdminComment.AdminComment, aliascomment);
//
//         Status = SamSetInformationAlias(AliasHandle,
//                                         AliasAdminCommentInformation,
//                                         &AdminComment);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamSetInformationAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//
//             /* Delete the Alias if the Comment could not be set */
//             SamDeleteAlias(AliasHandle);
//
//             goto done;
//         }
//     }
//
// done:
//     if (AliasHandle != NULL)
//     {
//         if (ApiStatus != NERR_Success)
//             SamDeleteAlias(AliasHandle);
//         else
//             SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupAddMember  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupAddMember(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     PSID membersid)
// {
//     LOCALGROUP_MEMBERS_INFO_0 Member;
//
//     TRACE("(%s %s %p)\n", debugstr_w(servername),
//           debugstr_w(groupname), membersid);
//
//     Member.lgrmi0_sid = membersid;
//
//     return NetLocalGroupAddMembers(servername,
//                                    groupname,
//                                    0,
//                                    (LPBYTE)&Member,
//                                    1);
// }
//
//
// /************************************************************
//  *                NetLocalGroupAddMembers  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupAddMembers(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     DWORD level,
//     LPBYTE buf,
//     DWORD totalentries)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING AliasName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     PLOCALGROUP_MEMBERS_INFO_0 MemberList = NULL;
//     ULONG i;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %d %p %d)\n", debugstr_w(servername),
//           debugstr_w(groupname), level, buf, totalentries);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&AliasName, groupname);
//
//     switch (level)
//     {
//         case 0:
//             MemberList = (PLOCALGROUP_MEMBERS_INFO_0)buf;
//             break;
//
//         case 3:
//             Status = BuildSidListFromDomainAndName((servername != NULL) ? &ServerName : NULL,
//                                                    (PLOCALGROUP_MEMBERS_INFO_3)buf,
//                                                    totalentries,
//                                                    &MemberList);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("BuildSidListFromDomainAndName failed (Status %08lx)\n", Status);
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
//     /* Open the alias account in the builtin domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &AliasName,
//                                 ALIAS_ADD_MEMBER,
//                                 &AliasHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//         goto done;
//     }
//
//     if (AliasHandle == NULL)
//     {
//         if (DomainHandle != NULL)
//             SamCloseHandle(DomainHandle);
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
//         /* Open the alias account in the account domain */
//         ApiStatus = OpenAliasByName(DomainHandle,
//                                     &AliasName,
//                                     ALIAS_ADD_MEMBER,
//                                     &AliasHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//     }
//
//     /* Add new members to the alias */
//     for (i = 0; i < totalentries; i++)
//     {
//         Status = SamAddMemberToAlias(AliasHandle,
//                                      MemberList[i].lgrmi0_sid);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamAddMemberToAlias failed (Status %lu)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//     }
//
// done:
//     if (level == 3 && MemberList != NULL)
//     {
//         for (i = 0; i < totalentries; i++)
//         {
//             if (MemberList[i].lgrmi0_sid != NULL)
//                 NetApiBufferFree(MemberList[i].lgrmi0_sid);
//         }
//
//         NetApiBufferFree(MemberList);
//     }
//
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupDel  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupDel(
//     LPCWSTR servername,
//     LPCWSTR groupname)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s)\n", debugstr_w(servername), debugstr_w(groupname));
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
//     /* Open the alias account in the builtin domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &GroupName,
//                                 DELETE,
//                                 &AliasHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         TRACE("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         goto done;
//     }
//
//     if (AliasHandle == NULL)
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
//         /* Open the alias account in the account domain */
//         ApiStatus = OpenAliasByName(DomainHandle,
//                                     &GroupName,
//                                     DELETE,
//                                     &AliasHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//     }
//
//     /* Delete the alias */
//     Status = SamDeleteAlias(AliasHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamDeleteAlias failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
// done:
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupDelMember  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupDelMember(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     PSID membersid)
// {
//     LOCALGROUP_MEMBERS_INFO_0 Member;
//
//     TRACE("(%s %s %p)\n", debugstr_w(servername),
//           debugstr_w(groupname), membersid);
//
//     Member.lgrmi0_sid = membersid;
//
//     return NetLocalGroupDelMembers(servername,
//                                    groupname,
//                                    0,
//                                    (LPBYTE)&Member,
//                                    1);
// }
//
//
// /************************************************************
//  *                NetLocalGroupDelMembers  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupDelMembers(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     DWORD level,
//     LPBYTE buf,
//     DWORD totalentries)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING AliasName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     PLOCALGROUP_MEMBERS_INFO_0 MemberList = NULL;
//     ULONG i;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %d %p %d)\n", debugstr_w(servername),
//           debugstr_w(groupname), level, buf, totalentries);
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&AliasName, groupname);
//
//     switch (level)
//     {
//         case 0:
//             MemberList = (PLOCALGROUP_MEMBERS_INFO_0)buf;
//             break;
//
//         case 3:
//             Status = BuildSidListFromDomainAndName((servername != NULL) ? &ServerName : NULL,
//                                                    (PLOCALGROUP_MEMBERS_INFO_3)buf,
//                                                    totalentries,
//                                                    &MemberList);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("BuildSidListFromDomainAndName failed (Status %08lx)\n", Status);
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
//     /* Open the alias account in the builtin domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &AliasName,
//                                 ALIAS_REMOVE_MEMBER,
//                                 &AliasHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//         goto done;
//     }
//
//     if (AliasHandle == NULL)
//     {
//         if (DomainHandle != NULL)
//             SamCloseHandle(DomainHandle);
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
//         /* Open the alias account in the account domain */
//         ApiStatus = OpenAliasByName(DomainHandle,
//                                     &AliasName,
//                                     ALIAS_REMOVE_MEMBER,
//                                     &AliasHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//     }
//
//     /* Remove members from the alias */
//     for (i = 0; i < totalentries; i++)
//     {
//         Status = SamRemoveMemberFromAlias(AliasHandle,
//                                           MemberList[i].lgrmi0_sid);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamAddMemberToAlias failed (Status %lu)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//     }
//
// done:
//     if (level == 3 && MemberList != NULL)
//     {
//         for (i = 0; i < totalentries; i++)
//         {
//             if (MemberList[i].lgrmi0_sid != NULL)
//                 NetApiBufferFree(MemberList[i].lgrmi0_sid);
//         }
//
//         NetApiBufferFree(MemberList);
//     }
//
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupEnum  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupEnum(
//     LPCWSTR servername,
//     DWORD level,
//     LPBYTE* bufptr,
//     DWORD prefmaxlen,
//     LPDWORD entriesread,
//     LPDWORD totalentries,
//     PDWORD_PTR resumehandle)
// {
//     UNICODE_STRING ServerName;
//     PSAM_RID_ENUMERATION CurrentAlias;
//     PENUM_CONTEXT EnumContext = NULL;
//     ULONG i;
//     SAM_HANDLE AliasHandle = NULL;
//     PALIAS_GENERAL_INFORMATION AliasInfo = NULL;
//     LPVOID Buffer = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %d %p %d %p %p %p) stub!\n", debugstr_w(servername),
//           level, bufptr, prefmaxlen, entriesread, totalentries, resumehandle);
//
//     *entriesread = 0;
//     *totalentries = 0;
//     *bufptr = NULL;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     if (resumehandle != NULL && *resumehandle != 0)
//     {
//         EnumContext = (PENUM_CONTEXT)*resumehandle;
//     }
//     else
//     {
//         ApiStatus = NetApiBufferAllocate(sizeof(ENUM_CONTEXT), (PVOID*)&EnumContext);
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
//         EnumContext->Phase = BuiltinPhase;
//         EnumContext->DomainHandle = EnumContext->BuiltinDomainHandle;
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
//             TRACE("Calling SamEnumerateAliasesInDomain\n");
//
//             Status = SamEnumerateAliasesInDomain(EnumContext->DomainHandle,
//                                                  &EnumContext->EnumerationContext,
//                                                  (PVOID *)&EnumContext->Buffer,
//                                                  prefmaxlen,
//                                                  &EnumContext->Returned);
//
//             TRACE("SamEnumerateAliasesInDomain returned (Status %08lx)\n", Status);
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
//         /* Get a pointer to the current alias */
//         CurrentAlias = &EnumContext->Buffer[EnumContext->Index];
//
//         TRACE("RID: %lu\n", CurrentAlias->RelativeId);
//
//         Status = SamOpenAlias(EnumContext->DomainHandle,
//                               ALIAS_READ_INFORMATION,
//                               CurrentAlias->RelativeId,
//                               &AliasHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamOpenAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         Status = SamQueryInformationAlias(AliasHandle,
//                                           AliasGeneralInformation,
//                                           (PVOID *)&AliasInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamQueryInformationAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         SamCloseHandle(AliasHandle);
//         AliasHandle = NULL;
//
//         TRACE("Name: %S\n", AliasInfo->Name.Buffer);
//         TRACE("Comment: %S\n", AliasInfo->AdminComment.Buffer);
//
//         ApiStatus = BuildAliasInfoBuffer(AliasInfo,
//                                          level,
//                                          &Buffer);
//         if (ApiStatus != NERR_Success)
//             goto done;
//
//         if (AliasInfo != NULL)
//         {
//             FreeAliasInfo(AliasInfo);
//             AliasInfo = NULL;
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
//     if (resumehandle == NULL || ApiStatus != ERROR_MORE_DATA)
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
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
//
//     if (AliasInfo != NULL)
//         FreeAliasInfo(AliasInfo);
//
//     if (resumehandle != NULL)
//         *resumehandle = (DWORD_PTR)EnumContext;
//
//     *bufptr = (LPBYTE)Buffer;
//
//     TRACE ("return %lu\n", ApiStatus);
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  * NetLocalGroupGetInfo  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupGetInfo(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     DWORD level,
//     LPBYTE* bufptr)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING GroupName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     PALIAS_GENERAL_INFORMATION AliasInfo = NULL;
//     LPVOID Buffer = NULL;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %d %p) stub!\n", debugstr_w(servername),
//           debugstr_w(groupname), level, bufptr);
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
//     /* Open the alias account in the builtin domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &GroupName,
//                                 ALIAS_READ_INFORMATION,
//                                 &AliasHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//         goto done;
//     }
//
//     if (AliasHandle == NULL)
//     {
//         if (DomainHandle != NULL)
//             SamCloseHandle(DomainHandle);
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
//         /* Open the alias account in the account domain */
//         ApiStatus = OpenAliasByName(DomainHandle,
//                                     &GroupName,
//                                     ALIAS_READ_INFORMATION,
//                                     &AliasHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &GroupName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//     }
//
//     Status = SamQueryInformationAlias(AliasHandle,
//                                       AliasGeneralInformation,
//                                       (PVOID *)&AliasInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamQueryInformationAlias failed (Status %08lx)\n", Status);
//         ApiStatus = NetpNtStatusToApiStatus(Status);
//         goto done;
//     }
//
//     ApiStatus = BuildAliasInfoBuffer(AliasInfo,
//                                      level,
//                                      &Buffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
// done:
//     if (AliasInfo != NULL)
//         FreeAliasInfo(AliasInfo);
//
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupGetMembers  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupGetMembers(
//     LPCWSTR servername,
//     LPCWSTR localgroupname,
//     DWORD level,
//     LPBYTE* bufptr,
//     DWORD prefmaxlen,
//     LPDWORD entriesread,
//     LPDWORD totalentries,
//     PDWORD_PTR resumehandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING ServerName;
//     UNICODE_STRING AliasName;
//     PMEMBER_ENUM_CONTEXT EnumContext = NULL;
//     LPVOID Buffer = NULL;
//     PLOCALGROUP_MEMBERS_INFO_0 MembersInfo0;
//     PLOCALGROUP_MEMBERS_INFO_1 MembersInfo1;
//     PLOCALGROUP_MEMBERS_INFO_2 MembersInfo2;
//     PLOCALGROUP_MEMBERS_INFO_3 MembersInfo3;
//     LPWSTR Ptr;
//     ULONG Size = 0;
//     ULONG SidLength;
//     ULONG i;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %d %p %d %p %p %p)\n", debugstr_w(servername),
//           debugstr_w(localgroupname), level, bufptr, prefmaxlen, entriesread,
//           totalentries, resumehandle);
//
//     *entriesread = 0;
//     *totalentries = 0;
//     *bufptr = NULL;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&AliasName, localgroupname);
//
//     if (resumehandle != NULL && *resumehandle != 0)
//     {
//         EnumContext = (PMEMBER_ENUM_CONTEXT)*resumehandle;
//     }
//     else
//     {
//         /* Allocate the enumeration context */
//         ApiStatus = NetApiBufferAllocate(sizeof(MEMBER_ENUM_CONTEXT), (PVOID*)&EnumContext);
//         if (ApiStatus != NERR_Success)
//             goto done;
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
//         /* Open the Builtin Domain */
//         Status = OpenBuiltinDomain(EnumContext->ServerHandle,
//                                    DOMAIN_LOOKUP,
//                                    &EnumContext->DomainHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("OpenBuiltinDomain failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         /* Open the alias account in the builtin domain */
//         ApiStatus = OpenAliasByName(EnumContext->DomainHandle,
//                                     &AliasName,
//                                     ALIAS_LIST_MEMBERS,
//                                     &EnumContext->AliasHandle);
//         if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//             goto done;
//         }
//
//         if (EnumContext->AliasHandle == NULL)
//         {
//             if (EnumContext->DomainHandle != NULL)
//                 SamCloseHandle(EnumContext->DomainHandle);
//
//             /* Open the Acount Domain */
//             Status = OpenAccountDomain(EnumContext->ServerHandle,
//                                        (servername != NULL) ? &ServerName : NULL,
//                                        DOMAIN_LOOKUP,
//                                        &EnumContext->DomainHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("OpenAccountDomain failed (Status %08lx)\n", Status);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             /* Open the alias account in the account domain */
//             ApiStatus = OpenAliasByName(EnumContext->DomainHandle,
//                                         &AliasName,
//                                         ALIAS_LIST_MEMBERS,
//                                         &EnumContext->AliasHandle);
//             if (ApiStatus != NERR_Success)
//             {
//                 ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//                 if (ApiStatus == ERROR_NONE_MAPPED)
//                     ApiStatus = NERR_GroupNotFound;
//                 goto done;
//             }
//         }
//
//         /* Get the member list */
//         Status = SamGetMembersInAlias(EnumContext->AliasHandle,
//                                       &EnumContext->Sids,
//                                       &EnumContext->Count);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamGetMemberInAlias failed (Status %08lx)\n", Status);
//             ApiStatus = NetpNtStatusToApiStatus(Status);
//             goto done;
//         }
//
//         if (EnumContext->Count == 0)
//         {
//             TRACE("No member found. We're done.\n");
//             ApiStatus = NERR_Success;
//             goto done;
//         }
//
//         /* Get name and domain information for all members */
//         if (level != 0)
//         {
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        NULL,
//                                        0,
//                                        0,
//                                        NULL);
//
//             Status = LsaOpenPolicy((servername != NULL) ? &ServerName : NULL,
//                                    (PLSA_OBJECT_ATTRIBUTES)&ObjectAttributes,
//                                    POLICY_EXECUTE,
//                                    &EnumContext->LsaHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//
//             Status = LsaLookupSids(EnumContext->LsaHandle,
//                                    EnumContext->Count,
//                                    EnumContext->Sids,
//                                    &EnumContext->Domains,
//                                    &EnumContext->Names);
//             if (!NT_SUCCESS(Status))
//             {
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//         }
//     }
//
//     /* Calculate the required buffer size */
//     for (i = 0; i < EnumContext->Count; i++)
//     {
//         switch (level)
//         {
//             case 0:
//                 Size += sizeof(LOCALGROUP_MEMBERS_INFO_0) +
//                         RtlLengthSid(EnumContext->Sids[i]);
//                 break;
//
//             case 1:
//                 Size += sizeof(LOCALGROUP_MEMBERS_INFO_1) +
//                         RtlLengthSid(EnumContext->Sids[i]) +
//                         EnumContext->Names[i].Name.Length + sizeof(WCHAR);
//                 break;
//
//             case 2:
//                 Size += sizeof(LOCALGROUP_MEMBERS_INFO_2) +
//                         RtlLengthSid(EnumContext->Sids[i]) +
//                         EnumContext->Names[i].Name.Length + sizeof(WCHAR);
//                 if (EnumContext->Names[i].DomainIndex >= 0)
//                     Size += EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length + sizeof(WCHAR);
//                 break;
//
//             case 3:
//                 Size += sizeof(LOCALGROUP_MEMBERS_INFO_3) +
//                         EnumContext->Names[i].Name.Length + sizeof(WCHAR);
//                 if (EnumContext->Names[i].DomainIndex >= 0)
//                     Size += EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length + sizeof(WCHAR);
//                 break;
//
//             default:
//                 ApiStatus = ERROR_INVALID_LEVEL;
//                 goto done;
//         }
//     }
//
//     /* Allocate the member buffer */
//     ApiStatus = NetApiBufferAllocate(Size, &Buffer);
//     if (ApiStatus != NERR_Success)
//         goto done;
//
//     ZeroMemory(Buffer, Size);
//
//     /* Fill the member buffer */
//     switch (level)
//     {
//         case 0:
//             MembersInfo0 = (PLOCALGROUP_MEMBERS_INFO_0)Buffer;
//             Ptr = (PVOID)((ULONG_PTR)Buffer + sizeof(LOCALGROUP_MEMBERS_INFO_0) * EnumContext->Count);
//             break;
//
//         case 1:
//             MembersInfo1 = (PLOCALGROUP_MEMBERS_INFO_1)Buffer;
//             Ptr = (PVOID)((ULONG_PTR)Buffer + sizeof(LOCALGROUP_MEMBERS_INFO_1) * EnumContext->Count);
//             break;
//
//         case 2:
//             MembersInfo2 = (PLOCALGROUP_MEMBERS_INFO_2)Buffer;
//             Ptr = (PVOID)((ULONG_PTR)Buffer + sizeof(LOCALGROUP_MEMBERS_INFO_2) * EnumContext->Count);
//             break;
//
//         case 3:
//             MembersInfo3 = (PLOCALGROUP_MEMBERS_INFO_3)Buffer;
//             Ptr = (PVOID)((ULONG_PTR)Buffer + sizeof(LOCALGROUP_MEMBERS_INFO_3) * EnumContext->Count);
//             break;
//     }
//
//     for (i = 0; i < EnumContext->Count; i++)
//     {
//         switch (level)
//         {
//             case 0:
//                 MembersInfo0->lgrmi0_sid = (PSID)Ptr;
//
//                 SidLength = RtlLengthSid(EnumContext->Sids[i]);
//                 memcpy(MembersInfo0->lgrmi0_sid,
//                        EnumContext->Sids[i],
//                        SidLength);
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + SidLength);
//                 MembersInfo0++;
//                 break;
//
//             case 1:
//                 MembersInfo1->lgrmi1_sid = (PSID)Ptr;
//
//                 SidLength = RtlLengthSid(EnumContext->Sids[i]);
//                 memcpy(MembersInfo1->lgrmi1_sid,
//                        EnumContext->Sids[i],
//                        SidLength);
//
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + SidLength);
//
//                 MembersInfo1->lgrmi1_sidusage = EnumContext->Names[i].Use;
//
//                 TRACE("Name: %S\n", EnumContext->Names[i].Name.Buffer);
//
//                 MembersInfo1->lgrmi1_name = Ptr;
//
//                 memcpy(MembersInfo1->lgrmi1_name,
//                        EnumContext->Names[i].Name.Buffer,
//                        EnumContext->Names[i].Name.Length);
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + EnumContext->Names[i].Name.Length + sizeof(WCHAR));
//                 MembersInfo1++;
//                 break;
//
//             case 2:
//                 MembersInfo2->lgrmi2_sid = (PSID)Ptr;
//
//                 SidLength = RtlLengthSid(EnumContext->Sids[i]);
//                 memcpy(MembersInfo2->lgrmi2_sid,
//                        EnumContext->Sids[i],
//                        SidLength);
//
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + SidLength);
//
//                 MembersInfo2->lgrmi2_sidusage = EnumContext->Names[i].Use;
//
//                 MembersInfo2->lgrmi2_domainandname = Ptr;
//
//                 if (EnumContext->Names[i].DomainIndex >= 0)
//                 {
//                     memcpy(MembersInfo2->lgrmi2_domainandname,
//                            EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Buffer,
//                            EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length);
//
//                     Ptr = (PVOID)((ULONG_PTR)Ptr + EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length);
//
//                     *((LPWSTR)Ptr) = L'\\';
//
//                     Ptr = (PVOID)((ULONG_PTR)Ptr + sizeof(WCHAR));
//                 }
//
//                 memcpy(Ptr,
//                        EnumContext->Names[i].Name.Buffer,
//                        EnumContext->Names[i].Name.Length);
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + EnumContext->Names[i].Name.Length + sizeof(WCHAR));
//                 MembersInfo2++;
//                 break;
//
//             case 3:
//                 MembersInfo3->lgrmi3_domainandname = Ptr;
//
//                 if (EnumContext->Names[i].DomainIndex >= 0)
//                 {
//                     memcpy(MembersInfo3->lgrmi3_domainandname,
//                            EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Buffer,
//                            EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length);
//
//                     Ptr = (PVOID)((ULONG_PTR)Ptr + EnumContext->Domains->Domains[EnumContext->Names[i].DomainIndex].Name.Length);
//
//                     *((LPWSTR)Ptr) = L'\\';
//
//                     Ptr = (PVOID)((ULONG_PTR)Ptr + sizeof(WCHAR));
//                 }
//
//                 memcpy(Ptr,
//                        EnumContext->Names[i].Name.Buffer,
//                        EnumContext->Names[i].Name.Length);
//                 Ptr = (PVOID)((ULONG_PTR)Ptr + EnumContext->Names[i].Name.Length + sizeof(WCHAR));
//                 MembersInfo3++;
//                 break;
//         }
//     }
//
//     *entriesread = EnumContext->Count;
//
//     *bufptr = (LPBYTE)Buffer;
//
// done:
//     if (EnumContext != NULL)
//         *totalentries = EnumContext->Count;
//
//     if (resumehandle == NULL || ApiStatus != ERROR_MORE_DATA)
//     {
//         /* Release the enumeration context */
//         if (EnumContext != NULL)
//         {
//             if (EnumContext->LsaHandle != NULL)
//                 LsaClose(EnumContext->LsaHandle);
//
//             if (EnumContext->AliasHandle != NULL)
//                 SamCloseHandle(EnumContext->AliasHandle);
//
//             if (EnumContext->DomainHandle != NULL)
//                 SamCloseHandle(EnumContext->DomainHandle);
//
//             if (EnumContext->ServerHandle != NULL)
//                 SamCloseHandle(EnumContext->ServerHandle);
//
//             if (EnumContext->Sids != NULL)
//                 SamFreeMemory(EnumContext->Sids);
//
//             if (EnumContext->Domains != NULL)
//                 LsaFreeMemory(EnumContext->Domains);
//
//             if (EnumContext->Names != NULL)
//                 LsaFreeMemory(EnumContext->Names);
//
//             NetApiBufferFree(EnumContext);
//             EnumContext = NULL;
//         }
//     }
//
//     return ApiStatus;
// }
//
//
// /************************************************************
//  *                NetLocalGroupSetInfo  (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupSetInfo(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     DWORD level,
//     LPBYTE buf,
//     LPDWORD parm_err)
// {
//     UNICODE_STRING ServerName;
//     UNICODE_STRING AliasName;
//     SAM_HANDLE ServerHandle = NULL;
//     SAM_HANDLE DomainHandle = NULL;
//     SAM_HANDLE AliasHandle = NULL;
//     ALIAS_NAME_INFORMATION AliasNameInfo;
//     ALIAS_ADM_COMMENT_INFORMATION AdminCommentInfo;
//     NET_API_STATUS ApiStatus = NERR_Success;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%s %s %d %p %p)\n", debugstr_w(servername),
//           debugstr_w(groupname), level, buf, parm_err);
//
//     if (parm_err != NULL)
//         *parm_err = PARM_ERROR_NONE;
//
//     if (servername != NULL)
//         RtlInitUnicodeString(&ServerName, servername);
//
//     RtlInitUnicodeString(&AliasName, groupname);
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
//     /* Open the alias account in the builtin domain */
//     ApiStatus = OpenAliasByName(DomainHandle,
//                                 &AliasName,
//                                 ALIAS_WRITE_ACCOUNT,
//                                 &AliasHandle);
//     if (ApiStatus != NERR_Success && ApiStatus != ERROR_NONE_MAPPED)
//     {
//         ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//         goto done;
//     }
//
//     if (AliasHandle == NULL)
//     {
//         if (DomainHandle != NULL)
//             SamCloseHandle(DomainHandle);
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
//         /* Open the alias account in the account domain */
//         ApiStatus = OpenAliasByName(DomainHandle,
//                                     &AliasName,
//                                     ALIAS_WRITE_ACCOUNT,
//                                     &AliasHandle);
//         if (ApiStatus != NERR_Success)
//         {
//             ERR("OpenAliasByName(%wZ) failed (ApiStatus %lu)\n", &AliasName, ApiStatus);
//             if (ApiStatus == ERROR_NONE_MAPPED)
//                 ApiStatus = NERR_GroupNotFound;
//             goto done;
//         }
//     }
//
//     switch (level)
//     {
//         case 0:
//             /* Set the alias name */
//             RtlInitUnicodeString(&AliasNameInfo.Name,
//                                  ((PLOCALGROUP_INFO_0)buf)->lgrpi0_name);
//
//             Status = SamSetInformationAlias(AliasHandle,
//                                             AliasNameInformation,
//                                             &AliasNameInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 TRACE("SamSetInformationAlias failed (ApiStatus %lu)\n", ApiStatus);
//                 ApiStatus = NetpNtStatusToApiStatus(Status);
//                 goto done;
//             }
//             break;
//
//         case 1:
//         case 1002:
//             /* Set the alias admin comment */
//             if (level == 1)
//                 RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                      ((PLOCALGROUP_INFO_1)buf)->lgrpi1_comment);
//             else
//                 RtlInitUnicodeString(&AdminCommentInfo.AdminComment,
//                                      ((PLOCALGROUP_INFO_1002)buf)->lgrpi1002_comment);
//
//             Status = SamSetInformationAlias(AliasHandle,
//                                             AliasAdminCommentInformation,
//                                             &AdminCommentInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 TRACE("SamSetInformationAlias failed (ApiStatus %lu)\n", ApiStatus);
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
//     if (AliasHandle != NULL)
//         SamCloseHandle(AliasHandle);
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
// /************************************************************
//  *                NetLocalGroupSetMember (NETAPI32.@)
//  */
// NET_API_STATUS
// WINAPI
// NetLocalGroupSetMembers(
//     LPCWSTR servername,
//     LPCWSTR groupname,
//     DWORD level,
//     LPBYTE buf,
//     DWORD totalentries)
// {
//     FIXME("(%s %s %d %p %d) stub!\n", debugstr_w(servername),
//             debugstr_w(groupname), level, buf, totalentries);
//     return NERR_Success;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
