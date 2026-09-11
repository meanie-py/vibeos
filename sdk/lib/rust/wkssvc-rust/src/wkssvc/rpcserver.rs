use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(
    lpParameter: *mut core::ffi::c_void,
) -> u32 {
    let mut status: u32;
    let ncacn_np = &[0x6Eu16, 0x63, 0x61, 0x63, 0x6E, 0x5F, 0x6E, 0x70, 0x0000];
    let pipe_name = &[
        0x5Cu16, 0x70, 0x69, 0x70, 0x65, 0x5C, 0x77, 0x6B, 0x73, 0x73, 0x76, 0x63, 0x0000,
    ];
    status = (RpcServerUseProtseqEpW(
        ncacn_np.as_ptr() as *mut c_ushort,
        20,
        pipe_name.as_ptr() as *mut c_ushort,
        core::ptr::null_mut(),
    )) as u32;
    if status != (RPC_S_OK) as u32 {
        return 0;
    }
    status = (RpcServerRegisterIf(
        wkssvc_v1_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if status != (RPC_S_OK) as u32 {
        return 0;
    }
    status = (RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, (0u8) as u32)) as u32;
    if status != (RPC_S_OK) as u32 {
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len) as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
unsafe extern "C" fn NetpGetClientLogonId(LogonId: PLUID) -> u32 {
    let mut thread_token: HANDLE = core::ptr::null_mut();
    let mut statistics: TOKEN_STATISTICS = core::mem::zeroed();
    let mut length: u32 = 0;
    let mut nt_status: i32;
    let mut api_status: u32 = 0; // NERR_Success

    api_status = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if api_status != 0 {
        return api_status;
    }

    nt_status = NtOpenThreadToken(
        (-2isize) as HANDLE,
        TOKEN_QUERY,
        1u8,
        &mut thread_token,
    );
    if nt_status < 0 {
        api_status = RtlNtStatusToDosError(nt_status);
        // goto done
        if !thread_token.is_null() {
            NtClose(thread_token);
        }
        RpcRevertToSelf();
        return api_status;
    }

    nt_status = NtQueryInformationToken(
        thread_token,
        TokenStatistics,
        &mut statistics as *mut _ as *mut core::ffi::c_void,
        (core::mem::size_of::<TOKEN_STATISTICS>()) as u32,
        &mut length,
    );
    if nt_status < 0 {
        api_status = RtlNtStatusToDosError(nt_status);
        // goto done
        if !thread_token.is_null() {
            NtClose(thread_token);
        }
        RpcRevertToSelf();
        return api_status;
    }

    RtlCopyLuid(LogonId, core::ptr::addr_of!(statistics.AuthenticationId) as *mut LUID);

    // done:
    if !thread_token.is_null() {
        NtClose(thread_token);
    }
    RpcRevertToSelf();
    api_status
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaGetInfo(
    ServerName: WKSSVC_IDENTIFY_HANDLE,
    Level: u32,
    WkstaInfo: *mut WKSTA_INFO,
) -> u32 {
    let mut szComputerName: [wchar_t; (MAX_COMPUTERNAME_LENGTH + 1) as usize] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
    let mut dwComputerNameLength: u32 = (MAX_COMPUTERNAME_LENGTH + 1) as u32;
    let pszLanRoot: *const wchar_t = &[0u16] as *const u16 as *const wchar_t;
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut DomainInfo: *mut POLICY_PRIMARY_DOMAIN_INFO = core::ptr::null_mut();
    let mut LoggedOnUsers: u32 = 0;
    let mut ObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();

    // Get computer name
    GetComputerNameW(szComputerName.as_mut_ptr(), &mut dwComputerNameLength);
    dwComputerNameLength += 1; // include NULL terminator

    // Zero and open policy
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut u8,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    let NtStatus: NTSTATUS = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        POLICY_VIEW_LOCAL_INFORMATION,
        &mut PolicyHandle,
    );
    if NtStatus != STATUS_SUCCESS {
        return LsaNtStatusToWinError(NtStatus);
    }

    // Query primary domain information
    let NtStatus = LsaQueryInformationPolicy(
        PolicyHandle,
        PolicyPrimaryDomainInformation,
        &mut DomainInfo as *mut *mut POLICY_PRIMARY_DOMAIN_INFO as *mut *mut core::ffi::c_void,
    );
    LsaClose(PolicyHandle);
    if NtStatus != STATUS_SUCCESS {
        return LsaNtStatusToWinError(NtStatus);
    }

    // This block can break to 'done: label
    let result: u32 = 'done: loop {
        let mut dwResult: u32 = NERR_Success;

        // Level 102 needs logged-on users enumeration
        if Level == 102 {
            let mut EnumRequest: MSV1_0_ENUMUSERS_REQUEST = core::mem::zeroed();
            EnumRequest.MessageType = MsV1_0EnumerateUsers;
            let mut EnumResponseBuffer: *mut MSV1_0_ENUMUSERS_RESPONSE = core::ptr::null_mut();
            let mut EnumResponseBufferSize: u32 = 0;
            let mut ProtocolStatus: NTSTATUS = 0;
            let NtStatus = LsaCallAuthenticationPackage(
                LsaHandle,
                LsaAuthenticationPackage,
                &mut EnumRequest as *mut _ as *mut core::ffi::c_void,
                (core::mem::size_of::<MSV1_0_ENUMUSERS_REQUEST>()) as u32,
                &mut EnumResponseBuffer as *mut *mut MSV1_0_ENUMUSERS_RESPONSE as *mut *mut core::ffi::c_void,
                &mut EnumResponseBufferSize,
                &mut ProtocolStatus,
            );
            if !NT_SUCCESS(NtStatus) {
                dwResult = RtlNtStatusToDosError(NtStatus);
                break 'done dwResult;
            }
            LoggedOnUsers = (*EnumResponseBuffer).NumberOfLoggedOnUsers;
            LsaFreeReturnBuffer(EnumResponseBuffer as *mut core::ffi::c_void);
        }

        match Level {
            100 => {
                let pWkstaInfo100: *mut WKSTA_INFO_100 = midl_user_allocate((core::mem::size_of::<WKSTA_INFO_100>()) as SIZE_T) as *mut WKSTA_INFO_100;
                if pWkstaInfo100.is_null() {
                    dwResult = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    (*pWkstaInfo100).wki100_platform_id = PLATFORM_ID_NT;
                    let comp_len = dwComputerNameLength as usize * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo100).wki100_computername = midl_user_allocate((comp_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo100).wki100_computername.is_null() {
                        wcscpy((*pWkstaInfo100).wki100_computername, szComputerName.as_ptr());
                    }
                    let dom_name_len = (wcslen((*DomainInfo).Name.Buffer) as usize + 1) * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo100).wki100_langroup = midl_user_allocate((dom_name_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo100).wki100_langroup.is_null() {
                        wcscpy((*pWkstaInfo100).wki100_langroup, (*DomainInfo).Name.Buffer);
                    }
                    (*pWkstaInfo100).wki100_ver_major = VersionInfo.dwMajorVersion;
                    (*pWkstaInfo100).wki100_ver_minor = VersionInfo.dwMinorVersion;
                    (*WkstaInfo).WkstaInfo100 = pWkstaInfo100;
                }
            }
            101 => {
                let pWkstaInfo101: *mut WKSTA_INFO_101 = midl_user_allocate((core::mem::size_of::<WKSTA_INFO_101>()) as SIZE_T) as *mut WKSTA_INFO_101;
                if pWkstaInfo101.is_null() {
                    dwResult = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    (*pWkstaInfo101).wki101_platform_id = PLATFORM_ID_NT;
                    let comp_len = dwComputerNameLength as usize * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo101).wki101_computername = midl_user_allocate((comp_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo101).wki101_computername.is_null() {
                        wcscpy((*pWkstaInfo101).wki101_computername, szComputerName.as_ptr());
                    }
                    let dom_name_len = (wcslen((*DomainInfo).Name.Buffer) as usize + 1) * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo101).wki101_langroup = midl_user_allocate((dom_name_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo101).wki101_langroup.is_null() {
                        wcscpy((*pWkstaInfo101).wki101_langroup, (*DomainInfo).Name.Buffer);
                    }
                    (*pWkstaInfo101).wki101_ver_major = VersionInfo.dwMajorVersion;
                    (*pWkstaInfo101).wki101_ver_minor = VersionInfo.dwMinorVersion;
                    let lan_len = (wcslen(pszLanRoot) as usize + 1) * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo101).wki101_lanroot = midl_user_allocate((lan_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo101).wki101_lanroot.is_null() {
                        wcscpy((*pWkstaInfo101).wki101_lanroot, pszLanRoot);
                    }
                    (*WkstaInfo).WkstaInfo101 = pWkstaInfo101;
                }
            }
            102 => {
                let pWkstaInfo102: *mut WKSTA_INFO_102 = midl_user_allocate((core::mem::size_of::<WKSTA_INFO_102>()) as SIZE_T) as *mut WKSTA_INFO_102;
                if pWkstaInfo102.is_null() {
                    dwResult = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    (*pWkstaInfo102).wki102_platform_id = PLATFORM_ID_NT;
                    let comp_len = dwComputerNameLength as usize * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo102).wki102_computername = midl_user_allocate((comp_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo102).wki102_computername.is_null() {
                        wcscpy((*pWkstaInfo102).wki102_computername, szComputerName.as_ptr());
                    }
                    let dom_name_len = (wcslen((*DomainInfo).Name.Buffer) as usize + 1) * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo102).wki102_langroup = midl_user_allocate((dom_name_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo102).wki102_langroup.is_null() {
                        wcscpy((*pWkstaInfo102).wki102_langroup, (*DomainInfo).Name.Buffer);
                    }
                    (*pWkstaInfo102).wki102_ver_major = VersionInfo.dwMajorVersion;
                    (*pWkstaInfo102).wki102_ver_minor = VersionInfo.dwMinorVersion;
                    let lan_len = (wcslen(pszLanRoot) as usize + 1) * core::mem::size_of::<wchar_t>();
                    (*pWkstaInfo102).wki102_lanroot = midl_user_allocate((lan_len) as SIZE_T) as *mut wchar_t;
                    if !(*pWkstaInfo102).wki102_lanroot.is_null() {
                        wcscpy((*pWkstaInfo102).wki102_lanroot, pszLanRoot);
                    }
                    (*pWkstaInfo102).wki102_logged_on_users = LoggedOnUsers;
                    (*WkstaInfo).WkstaInfo102 = pWkstaInfo102;
                }
            }
            502 => {
                let pWkstaInfo502: *mut WKSTA_INFO_502 = midl_user_allocate((core::mem::size_of::<WKSTA_INFO_502>()) as SIZE_T) as *mut WKSTA_INFO_502;
                if pWkstaInfo502.is_null() {
                    dwResult = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    CopyMemory(
                        pWkstaInfo502 as *mut core::ffi::c_void,
                        &WkstaInfo502 as *const _ as *const core::ffi::c_void,
                        core::mem::size_of::<WKSTA_INFO_502>(),
                    );
                    (*WkstaInfo).WkstaInfo502 = pWkstaInfo502;
                }
            }
            _ => {
                // FIXME("Level %lu unimplemented\n", Level);
                dwResult = ERROR_INVALID_LEVEL;
            }
        }
        break 'done dwResult;
    };

    // cleanup: done label logic
    if !DomainInfo.is_null() {
        LsaFreeMemory(DomainInfo as *mut core::ffi::c_void);
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaSetInfo(
    ServerName: isize,
    Level: u32,
    WkstaInfo: *const WKSTA_INFO,
    ErrorParameter: *mut u32,
) -> u32 {
    let mut dw_err_param: u32 = 0;
    let mut dw_result: u32 = 0;

    match Level {
        502 => {
            let keep_conn = (*(*WkstaInfo).WkstaInfo502).wki502_keep_conn;
            if keep_conn >= 1 && keep_conn <= 65535 {
                WkstaInfo502.wki502_keep_conn = keep_conn;
            } else {
                dw_err_param = WKSTA_KEEPCONN_PARMNUM;
                dw_result = ERROR_INVALID_PARAMETER;
            }

            if dw_result == 0 {
                let max_cmds = (*(*WkstaInfo).WkstaInfo502).wki502_max_cmds;
                if max_cmds >= 50 && max_cmds <= 65535 {
                    WkstaInfo502.wki502_max_cmds = max_cmds;
                } else {
                    dw_err_param = WKSTA_MAXCMDS_PARMNUM;
                    dw_result = ERROR_INVALID_PARAMETER;
                }
            }

            if dw_result == 0 {
                let sess_timeout = (*(*WkstaInfo).WkstaInfo502).wki502_sess_timeout;
                if sess_timeout >= 60 && sess_timeout <= 65535 {
                    WkstaInfo502.wki502_sess_timeout = sess_timeout;
                } else {
                    dw_err_param = WKSTA_SESSTIMEOUT_PARMNUM;
                    dw_result = ERROR_INVALID_PARAMETER;
                }
            }

            if dw_result == 0 {
                let dormant_file_limit = (*(*WkstaInfo).WkstaInfo502).wki502_dormant_file_limit;
                if dormant_file_limit != 0 {
                    WkstaInfo502.wki502_dormant_file_limit = dormant_file_limit;
                } else {
                    dw_err_param = WKSTA_DORMANTFILELIMIT_PARMNUM;
                    dw_result = ERROR_INVALID_PARAMETER;
                }
            }
        }
        1013 => {
            let keep_conn = (*(*WkstaInfo).WkstaInfo1013).wki1013_keep_conn;
            if keep_conn >= 1 && keep_conn <= 65535 {
                WkstaInfo502.wki502_keep_conn = keep_conn;
            } else {
                dw_err_param = WKSTA_KEEPCONN_PARMNUM;
                dw_result = ERROR_INVALID_PARAMETER;
            }
        }
        1018 => {
            let sess_timeout = (*(*WkstaInfo).WkstaInfo1018).wki1018_sess_timeout;
            if sess_timeout >= 60 && sess_timeout <= 65535 {
                WkstaInfo502.wki502_sess_timeout = sess_timeout;
            } else {
                dw_err_param = WKSTA_SESSTIMEOUT_PARMNUM;
                dw_result = ERROR_INVALID_PARAMETER;
            }
        }
        1046 => {
            let dormant_file_limit = (*(*WkstaInfo).WkstaInfo1046).wki1046_dormant_file_limit;
            if dormant_file_limit != 0 {
                WkstaInfo502.wki502_dormant_file_limit = dormant_file_limit;
            } else {
                dw_err_param = WKSTA_DORMANTFILELIMIT_PARMNUM;
                dw_result = ERROR_INVALID_PARAMETER;
            }
        }
        _ => {
            ERR!("Invalid Level %lu\n", Level);
            dw_result = ERROR_INVALID_LEVEL;
        }
    }

    if dw_result == 0 {
        SaveWorkstationInfo(Level);
    }

    if (dw_result == ERROR_INVALID_PARAMETER) && !ErrorParameter.is_null() {
        *ErrorParameter = dw_err_param;
    }

    dw_result
}
#[no_mangle]
pub unsafe extern "system" fn NetrWkstaUserEnum(
    _server_name: WKSSVC_IDENTIFY_HANDLE,
    user_info: LPWKSTA_USER_ENUM_STRUCT,
    preferred_maximum_length: u32,
    total_entries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    let mut enum_request: _MSV1_0_ENUMUSERS_REQUEST = core::mem::zeroed();
    let mut enum_response_buffer: *mut _MSV1_0_ENUMUSERS_RESPONSE = core::ptr::null_mut();
    let mut user_info_request: _MSV1_0_GETUSERINFO_REQUEST = core::mem::zeroed();
    let mut user_info_response_buffer: *mut _MSV1_0_GETUSERINFO_RESPONSE = core::ptr::null_mut();
    let mut user_info_array: *mut *mut _MSV1_0_GETUSERINFO_RESPONSE = core::ptr::null_mut();
    let mut enum_response_buffer_size: u32 = 0;
    let mut user_info_response_buffer_size: u32 = 0;
    let mut status: NTSTATUS;
    let mut protocol_status: NTSTATUS = 0;
    let mut i: u32;
    let mut start: u32;
    let mut count: u32;
    let mut p_logon_id: PLUID;
    let mut p_enum_handle: *mut u32;
    let mut dw_result: u32 = NERR_Success;

    let mut p_user_info0: *mut _WKSTA_USER_INFO_0 = core::ptr::null_mut();
    let mut p_user_info1: *mut _WKSTA_USER_INFO_1 = core::ptr::null_mut();

    if (*user_info).Level > 1 {
        return ERROR_INVALID_LEVEL;
    }

    /* Enumerate all currently logged-on users */
    enum_request.MessageType = MsV1_0EnumerateUsers;
    status = LsaCallAuthenticationPackage(
        LsaHandle,
        LsaAuthenticationPackage,
        core::ptr::addr_of_mut!(enum_request) as *mut core::ffi::c_void,
        core::mem::size_of::<_MSV1_0_ENUMUSERS_REQUEST>() as ULONG,
        &mut enum_response_buffer as *mut *mut _MSV1_0_ENUMUSERS_RESPONSE as *mut *mut core::ffi::c_void,
        &mut enum_response_buffer_size,
        &mut protocol_status,
    );

    if !NT_SUCCESS(status) {
        dw_result = RtlNtStatusToDosError(status);
        return dw_result;
    }

    if (*enum_response_buffer).NumberOfLoggedOnUsers > 0 {
        p_logon_id = (*enum_response_buffer).LogonIds;
        p_enum_handle = (*enum_response_buffer).EnumHandles;

        user_info_array = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*enum_response_buffer).NumberOfLoggedOnUsers as usize
                * core::mem::size_of::<*mut _MSV1_0_GETUSERINFO_RESPONSE>()) as SIZE_T,
        ) as *mut *mut _MSV1_0_GETUSERINFO_RESPONSE;
        if user_info_array.is_null() {
            dw_result = ERROR_NOT_ENOUGH_MEMORY;
            return dw_result;
        }

        for i in 0..(*enum_response_buffer).NumberOfLoggedOnUsers {
            user_info_request.MessageType = MsV1_0GetUserInfo;
            user_info_request.LogonId = *p_logon_id;
            status = LsaCallAuthenticationPackage(
                LsaHandle,
                LsaAuthenticationPackage,
                core::ptr::addr_of_mut!(user_info_request) as *mut core::ffi::c_void,
                core::mem::size_of::<_MSV1_0_GETUSERINFO_REQUEST>() as ULONG,
                &mut user_info_response_buffer as *mut *mut _MSV1_0_GETUSERINFO_RESPONSE
                    as *mut *mut core::ffi::c_void,
                &mut user_info_response_buffer_size,
                &mut protocol_status,
            );
            if !NT_SUCCESS(status) {
                dw_result = RtlNtStatusToDosError(status);
                return dw_result;
            }

            *user_info_array.add(i as usize) = user_info_response_buffer;

            p_logon_id = p_logon_id.add(1);
            p_enum_handle = p_enum_handle.add(1);
        }

        if preferred_maximum_length == MAX_PREFERRED_LENGTH {
            start = 0;
            count = (*enum_response_buffer).NumberOfLoggedOnUsers;
        } else {
            /* Not implemented */
            dw_result = ERROR_CALL_NOT_IMPLEMENTED;
            return dw_result;
        }

        match (*user_info).Level {
            0 => {
                p_user_info0 = midl_user_allocate(
                    ((count as usize) * core::mem::size_of::<_WKSTA_USER_INFO_0>()) as SIZE_T,
                ) as *mut _WKSTA_USER_INFO_0;
                if p_user_info0.is_null() {
                    dw_result = ERROR_NOT_ENOUGH_MEMORY;
                    return dw_result;
                }

                ZeroMemory(
                    p_user_info0 as *mut core::ffi::c_void,
                    (count as usize) * core::mem::size_of::<_WKSTA_USER_INFO_0>(),
                );

                for i in 0..count {
                    let user = *user_info_array.add((start + i) as usize);
                    let name_len = (*user).UserName.Length as usize;
                    let buffer = midl_user_allocate((name_len + core::mem::size_of::<wchar_t>()) as SIZE_T) as *mut wchar_t;
                    if buffer.is_null() {
                        dw_result = ERROR_NOT_ENOUGH_MEMORY;
                        return dw_result;
                    }

                    ZeroMemory(buffer as *mut core::ffi::c_void, name_len + core::mem::size_of::<wchar_t>());
                    CopyMemory(
                        buffer as *mut core::ffi::c_void,
                        (*user).UserName.Buffer as *const core::ffi::c_void,
                        name_len,
                    );

                    (*p_user_info0.add(i as usize)).wkui0_username = buffer;
                }

                (*user_info).WkstaUserInfo.Level0.as_mut().EntriesRead = count;
                (*user_info).WkstaUserInfo.Level0.as_mut().Buffer = p_user_info0;
                *total_entries = (*enum_response_buffer).NumberOfLoggedOnUsers;
                *resume_handle = 0;
            }
            1 => {
                p_user_info1 = midl_user_allocate(
                    ((count as usize) * core::mem::size_of::<_WKSTA_USER_INFO_1>()) as SIZE_T,
                ) as *mut _WKSTA_USER_INFO_1;
                if p_user_info1.is_null() {
                    dw_result = ERROR_NOT_ENOUGH_MEMORY;
                    return dw_result;
                }

                ZeroMemory(
                    p_user_info1 as *mut core::ffi::c_void,
                    (count as usize) * core::mem::size_of::<_WKSTA_USER_INFO_1>(),
                );

                for i in 0..count {
                    let user = *user_info_array.add((start + i) as usize);

                    let username = midl_user_allocate(
                        ((*user).UserName.Length as usize + core::mem::size_of::<wchar_t>()) as SIZE_T,
                    ) as *mut wchar_t;
                    if username.is_null() {
                        dw_result = ERROR_NOT_ENOUGH_MEMORY;
                        return dw_result;
                    }
                    ZeroMemory(
                        username as *mut core::ffi::c_void,
                        (*user).UserName.Length as usize + core::mem::size_of::<wchar_t>(),
                    );
                    CopyMemory(
                        username as *mut core::ffi::c_void,
                        (*user).UserName.Buffer as *const core::ffi::c_void,
                        (*user).UserName.Length as usize,
                    );
                    (*p_user_info1.add(i as usize)).wkui1_username = username;

                    let logon_domain = midl_user_allocate(
                        ((*user).LogonDomainName.Length as usize + core::mem::size_of::<wchar_t>()) as SIZE_T,
                    ) as *mut wchar_t;
                    if logon_domain.is_null() {
                        dw_result = ERROR_NOT_ENOUGH_MEMORY;
                        return dw_result;
                    }
                    ZeroMemory(
                        logon_domain as *mut core::ffi::c_void,
                        (*user).LogonDomainName.Length as usize + core::mem::size_of::<wchar_t>(),
                    );
                    CopyMemory(
                        logon_domain as *mut core::ffi::c_void,
                        (*user).LogonDomainName.Buffer as *const core::ffi::c_void,
                        (*user).LogonDomainName.Length as usize,
                    );
                    (*p_user_info1.add(i as usize)).wkui1_logon_domain = logon_domain;

                    // FIXME: wkui1_oth_domains

                    let logon_server = midl_user_allocate(
                        ((*user).LogonServer.Length as usize + core::mem::size_of::<wchar_t>()) as SIZE_T,
                    ) as *mut wchar_t;
                    if logon_server.is_null() {
                        dw_result = ERROR_NOT_ENOUGH_MEMORY;
                        return dw_result;
                    }
                    ZeroMemory(
                        logon_server as *mut core::ffi::c_void,
                        (*user).LogonServer.Length as usize + core::mem::size_of::<wchar_t>(),
                    );
                    CopyMemory(
                        logon_server as *mut core::ffi::c_void,
                        (*user).LogonServer.Buffer as *const core::ffi::c_void,
                        (*user).LogonServer.Length as usize,
                    );
                    (*p_user_info1.add(i as usize)).wkui1_logon_server = logon_server;
                }

                (*user_info).WkstaUserInfo.Level1.as_mut().EntriesRead = count;
                (*user_info).WkstaUserInfo.Level1.as_mut().Buffer = p_user_info1;
                *total_entries = (*enum_response_buffer).NumberOfLoggedOnUsers;
                *resume_handle = 0;
            }
            _ => {}
        }
    } else {
        if (*user_info).Level == 0 {
            (*user_info).WkstaUserInfo.Level0.as_mut().Buffer = core::ptr::null_mut();
            (*user_info).WkstaUserInfo.Level0.as_mut().EntriesRead = 0;
        } else {
            (*user_info).WkstaUserInfo.Level1.as_mut().Buffer = core::ptr::null_mut();
            (*user_info).WkstaUserInfo.Level1.as_mut().EntriesRead = 0;
        }
        *total_entries = 0;
        dw_result = NERR_Success;
    }

    // cleanup
    if !user_info_array.is_null() {
        for i in 0..(*enum_response_buffer).NumberOfLoggedOnUsers {
            let user = *user_info_array.add(i as usize);
            if !(*user).UserName.Buffer.is_null() {
                LsaFreeReturnBuffer((*user).UserName.Buffer as *mut core::ffi::c_void);
            }
            if !(*user).LogonDomainName.Buffer.is_null() {
                LsaFreeReturnBuffer((*user).LogonDomainName.Buffer as *mut core::ffi::c_void);
            }
            if !(*user).LogonServer.Buffer.is_null() {
                LsaFreeReturnBuffer((*user).LogonServer.Buffer as *mut core::ffi::c_void);
            }
            LsaFreeReturnBuffer(user as *mut core::ffi::c_void);
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, user_info_array as *mut core::ffi::c_void);
    }

    if !enum_response_buffer.is_null() {
        LsaFreeReturnBuffer(enum_response_buffer as *mut core::ffi::c_void);
    }

    dw_result
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaUserGetInfo(
    Unused: *mut core::ffi::c_void,
    Level: u32,
    UserInfo: *mut _WKSTA_USER_INFO,
) -> u32 {
    let mut user_info_request: MSV1_0_GETUSERINFO_REQUEST = core::mem::zeroed();
    let mut user_info_response_buffer: PMSV1_0_GETUSERINFO_RESPONSE = core::ptr::null_mut();
    let mut user_info_response_buffer_size: u32 = 0;
    let mut protocol_status: NTSTATUS = 0;
    let mut logon_id: LUID = core::mem::zeroed();
    let mut p_user_info_0: PWKSTA_USER_INFO_0 = core::ptr::null_mut();
    let mut p_user_info_1: PWKSTA_USER_INFO_1 = core::ptr::null_mut();
    let mut p_user_info_1101: PWKSTA_USER_INFO_1101 = core::ptr::null_mut();
    let mut dw_result: u32 = 0; // NERR_Success

    if !Unused.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if Level > 1 && Level != 1101 {
        return ERROR_INVALID_LEVEL;
    }

    if Level != 1101 {
        dw_result = NetpGetClientLogonId(&mut logon_id);
        if dw_result != 0 {
            return dw_result;
        }

        user_info_request.MessageType = MsV1_0GetUserInfo;
        user_info_request.LogonId = logon_id;

        let status = LsaCallAuthenticationPackage(
            LsaHandle,
            LsaAuthenticationPackage,
            &mut user_info_request as *mut _ as *mut core::ffi::c_void,
            (core::mem::size_of::<MSV1_0_GETUSERINFO_REQUEST>()) as u32,
            &mut user_info_response_buffer as *mut _ as *mut *mut core::ffi::c_void,
            &mut user_info_response_buffer_size,
            &mut protocol_status,
        );

        if !NT_SUCCESS(status) {
            return RtlNtStatusToDosError(status);
        }
    }

    match Level {
        0 => {
            p_user_info_0 = midl_user_allocate((core::mem::size_of::<WKSTA_USER_INFO_0>()) as SIZE_T) as PWKSTA_USER_INFO_0;
            if p_user_info_0.is_null() {
                dw_result = ERROR_NOT_ENOUGH_MEMORY;
            } else {
                core::ptr::write_bytes(p_user_info_0, 0, 1); // ONE element (count is elements, not bytes)

                let len_with_null = (*user_info_response_buffer).UserName.Length as usize + 2; // + sizeof(WCHAR)
                (*p_user_info_0).wkui0_username = midl_user_allocate((len_with_null) as SIZE_T) as *mut u16;
                if (*p_user_info_0).wkui0_username.is_null() {
                    midl_user_free(p_user_info_0 as *mut core::ffi::c_void);
                    dw_result = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    core::ptr::write_bytes((*p_user_info_0).wkui0_username as *mut u8, 0, len_with_null); // len_with_null is BYTES
                    core::ptr::copy_nonoverlapping(
                        (*user_info_response_buffer).UserName.Buffer as *const u8,
                        (*p_user_info_0).wkui0_username as *mut u8,
                        (*user_info_response_buffer).UserName.Length as usize,
                    );
                    (*UserInfo).UserInfo0 = p_user_info_0;
                }
            }
        }
        1 => {
            p_user_info_1 = midl_user_allocate((core::mem::size_of::<WKSTA_USER_INFO_1>()) as SIZE_T) as PWKSTA_USER_INFO_1;
            if p_user_info_1.is_null() {
                dw_result = ERROR_NOT_ENOUGH_MEMORY;
            } else {
                core::ptr::write_bytes(p_user_info_1, 0, 1); // ONE element (count is elements, not bytes)

                // User Name
                let len_with_null = (*user_info_response_buffer).UserName.Length as usize + 2;
                (*p_user_info_1).wkui1_username = midl_user_allocate((len_with_null) as SIZE_T) as *mut u16;
                if (*p_user_info_1).wkui1_username.is_null() {
                    midl_user_free(p_user_info_1 as *mut core::ffi::c_void);
                    dw_result = ERROR_NOT_ENOUGH_MEMORY;
                } else {
                    core::ptr::write_bytes((*p_user_info_1).wkui1_username as *mut u8, 0, len_with_null); // len_with_null is BYTES
                    core::ptr::copy_nonoverlapping(
                        (*user_info_response_buffer).UserName.Buffer as *const u8,
                        (*p_user_info_1).wkui1_username as *mut u8,
                        (*user_info_response_buffer).UserName.Length as usize,
                    );

                    // Logon Domain Name
                    let domain_len_with_null = (*user_info_response_buffer).LogonDomainName.Length as usize + 2;
                    (*p_user_info_1).wkui1_logon_domain = midl_user_allocate((domain_len_with_null) as SIZE_T) as *mut u16;
                    if (*p_user_info_1).wkui1_logon_domain.is_null() {
                        midl_user_free((*p_user_info_1).wkui1_username as *mut core::ffi::c_void);
                        midl_user_free(p_user_info_1 as *mut core::ffi::c_void);
                        dw_result = ERROR_NOT_ENOUGH_MEMORY;
                    } else {
                        core::ptr::write_bytes((*p_user_info_1).wkui1_logon_domain as *mut u8, 0, domain_len_with_null); // domain_len_with_null is BYTES
                        core::ptr::copy_nonoverlapping(
                            (*user_info_response_buffer).LogonDomainName.Buffer as *const u8,
                            (*p_user_info_1).wkui1_logon_domain as *mut u8,
                            (*user_info_response_buffer).LogonDomainName.Length as usize,
                        );

                        // Logon Server
                        let server_len_with_null = (*user_info_response_buffer).LogonServer.Length as usize + 2;
                        (*p_user_info_1).wkui1_logon_server = midl_user_allocate((server_len_with_null) as SIZE_T) as *mut u16;
                        if (*p_user_info_1).wkui1_logon_server.is_null() {
                            midl_user_free((*p_user_info_1).wkui1_username as *mut core::ffi::c_void);
                            midl_user_free((*p_user_info_1).wkui1_logon_domain as *mut core::ffi::c_void);
                            midl_user_free(p_user_info_1 as *mut core::ffi::c_void);
                            dw_result = ERROR_NOT_ENOUGH_MEMORY;
                        } else {
                            core::ptr::write_bytes((*p_user_info_1).wkui1_logon_server as *mut u8, 0, server_len_with_null); // server_len_with_null is BYTES
                            core::ptr::copy_nonoverlapping(
                                (*user_info_response_buffer).LogonServer.Buffer as *const u8,
                                (*p_user_info_1).wkui1_logon_server as *mut u8,
                                (*user_info_response_buffer).LogonServer.Length as usize,
                            );
                            (*UserInfo).UserInfo1 = p_user_info_1;
                        }
                    }
                }
            }
        }
        1101 => {
            p_user_info_1101 = midl_user_allocate((core::mem::size_of::<WKSTA_USER_INFO_1101>()) as SIZE_T) as PWKSTA_USER_INFO_1101;
            if p_user_info_1101.is_null() {
                dw_result = ERROR_NOT_ENOUGH_MEMORY;
            } else {
                core::ptr::write_bytes(p_user_info_1101, 0, 1); // ONE element (count is elements, not bytes)
                // FIXME: wkui1101_oth_domains
                (*UserInfo).UserInfo1101 = p_user_info_1101;
            }
        }
        _ => {
            ERR!("\n");
            dw_result = ERROR_INVALID_LEVEL;
        }
    }

    if !user_info_response_buffer.is_null() {
        LsaFreeReturnBuffer(user_info_response_buffer as *mut core::ffi::c_void);
    }

    dw_result
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaUserSetInfo(
    Unused: WKSSVC_IDENTIFY_HANDLE,
    Level: u32,
    UserInfo: *mut WKSTA_USER_INFO,
    ErrorParameter: *mut u32,
) -> u32 {
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn NetrWkstaTransportEnum(
    ServerName: LPVOID,
    TransportInfo: LPVOID,
    PreferredMaximumLength: u32,
    TotalEntries: *mut u32,
    ResumeHandle: *mut u32,
) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaTransportAdd(
    ServerName: WKSSVC_IDENTIFY_HANDLE,
    Level: u32,
    TransportInfo: *mut WKSTA_TRANSPORT_INFO_0,
    ErrorParameter: *mut u32,
) -> u32 {
    // UNIMPLEMENTED (FIXME debug stub)
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrWkstaTransportDel(
    _: WKSSVC_IDENTIFY_HANDLE,
    _: *mut wchar_t,
    _: u32,
) -> u32 {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrUseAdd(
    ServerName: LPVOID,
    Level: u32,
    InfoStruct: LPVOID,
    ErrorParameter: *mut u32,
) -> u32 {
    // UNIMPLEMENTED: stub
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrUseGetInfo(
    ServerName: *mut core::ffi::c_void,
    UseName: *mut wchar_t,
    Level: u32,
    InfoStruct: *mut core::ffi::c_void,
) -> u32 {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetrUseDel(
    ServerName: *mut core::ffi::c_void,
    UseName: *mut u16,
    ForceLevel: u32,
) -> u32 {
    // UNIMPLEMENTED
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetrUseEnum(
    ServerName: LPVOID,
    InfoStruct: LPVOID,
    PreferredMaximumLength: u32,
    TotalEntries: *mut u32,
    ResumeHandle: *mut u32,
) -> u32 {
    // UNIMPLEMENTED;
    UNIMPLEMENTED!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrMessageBufferSend() -> u32 {
    ERROR_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn NetrWorkstationStatisticsGet(
    ServerName: WKSSVC_IDENTIFY_HANDLE,
    ServiceName: *mut wchar_t,
    Level: u32,
    Options: u32,
    Buffer: *mut *mut STAT_WORKSTATION_0,
) -> u32 {
    let mut TimeOfDayInfo: SYSTEM_TIMEOFDAY_INFORMATION = core::mem::zeroed();
    let pStatBuffer: *mut STAT_WORKSTATION_0;
    let Status: NTSTATUS;

    if Level != 0 {
        return ERROR_INVALID_LEVEL;
    }

    if Options != 0 {
        return ERROR_INVALID_PARAMETER;
    }

    pStatBuffer = midl_user_allocate((core::mem::size_of::<STAT_WORKSTATION_0>()) as SIZE_T) as *mut STAT_WORKSTATION_0;
    if pStatBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    ZeroMemory(pStatBuffer as *mut core::ffi::c_void, core::mem::size_of::<STAT_WORKSTATION_0>());

    Status = NtQuerySystemInformation(
        SystemTimeOfDayInformation,
        &raw mut TimeOfDayInfo as *mut core::ffi::c_void,
        (core::mem::size_of::<SYSTEM_TIMEOFDAY_INFORMATION>()) as u32,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) {
        let mut Seconds: u32 = 0;
        if RtlTimeToSecondsSince1970(
            core::ptr::addr_of!(TimeOfDayInfo.BootTime) as PLARGE_INTEGER,
            &raw mut Seconds,
        ) != 0
        {
            (*pStatBuffer).StatisticsStartTime.u.HighPart = 0;
            (*pStatBuffer).StatisticsStartTime.u.LowPart = Seconds;
        }
    }

    // FIXME: Return the actual statistcs data!

    *Buffer = pStatBuffer;

    0 // NERR_Success
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonDomainNameAdd(
    _DomainName: *mut core::ffi::c_void,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrLogonDomainNameDel(_DomainName: WKSSVC_IDENTIFY_HANDLE) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrJoinDomain() -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrUnjoinDomain() -> u32 {
    return ERROR_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn NetrValidateName() -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrRenameMachineInDomain() -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetJoinInformation(
    _ServerName: *mut core::ffi::c_void,
    NameBuffer: *mut *mut u16,
    BufferType: *mut u32,
) -> u32 {
    if NameBuffer.is_null() {
        return ERROR_INVALID_PARAMETER as u32;
    }
    NetpGetJoinInformation(NameBuffer, BufferType)
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetJoinableOUs() -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrJoinDomain2(
    RpcBindingHandle: *mut core::ffi::c_void,
    ServerName: *mut u16,
    DomainNameParam: *mut u16,
    MachineAccountOU: *mut u16,
    AccountName: *mut u16,
    Password: *mut core::ffi::c_void,
    Options: u32,
) -> u32 {
    if DomainNameParam.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if (Options & NETSETUP_JOIN_DOMAIN) != 0 {
        ERROR_CALL_NOT_IMPLEMENTED
    } else {
        NetpJoinWorkgroup(DomainNameParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn NetrUnjoinDomain2(
    _RpcBindingHandle: *mut core::ffi::c_void,
    _ServerName: *mut u16,
    _AccountName: *mut u16,
    _Password: *mut core::ffi::c_void,
    _Options: u32,
) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrRenameMachineInDomain2(
    _RpcBindingHandle: *mut core::ffi::c_void,
    _ServerName: *mut wchar_t,
    _MachineName: *mut wchar_t,
    _AccountName: *mut wchar_t,
    _Password: *mut JOINPR_ENCRYPTED_USER_PASSWORD,
    _Options: u32,
) -> u32 {
    // UNIMPLEMENTED
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetrValidateName2(
    RpcBindingHandle: *mut core::ffi::c_void,
    ServerName: *mut u16,
    NameToValidate: *mut u16,
    AccountName: *mut u16,
    Password: *mut core::ffi::c_void,
    NameType: u32,
) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrGetJoinableOUs2(
    RpcBindingHandle: *mut core::ffi::c_void,
    ServerName: *mut u16,
    DomainNameParam: *mut u16,
    AccountName: *mut u16,
    Password: *mut core::ffi::c_void,
    OUCount: *mut u32,
    OUs: *mut *mut *mut u16,
) -> u32 {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrAddAlternateComputerName(
    RpcBindingHandle: *mut core::ffi::c_void,
    ServerName: LPCWSTR,
    AlternateName: LPCWSTR,
    DomainAccount: LPCWSTR,
    EncryptedPassword: *mut core::ffi::c_void,
    Reserved: u32,
) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrRemoveAlternateComputerName(
    _RpcBindingHandle: *mut core::ffi::c_void,
    _ServerName: LPCWSTR,
    _AlternateName: LPCWSTR,
    _DomainAccount: LPCWSTR,
    _EncryptedPassword: *mut core::ffi::c_void,
    _Reserved: u32,
) -> u32 {
    // UNIMPLEMENTED
    UNIMPLEMENTED!();
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetrSetPrimaryComputerName(
    RpcBindingHandle: *mut core::ffi::c_void,
    ServerName: *mut u16,
    PrimaryName: *mut u16,
    DomainAccount: *mut u16,
    EncryptedPassword: *mut core::ffi::c_void,
    Reserved: u32,
) -> u32 {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrEnumerateComputerNames(
    ServerName: WKSSVC_IMPERSONATE_HANDLE,
    NameType: NET_COMPUTER_NAME_TYPE,
    Reserved: u32,
    ComputerNames: *mut *mut NET_COMPUTER_NAME_ARRAY,
) -> u32 {
    // UNIMPLEMENTED stub
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS Services
//  *  Copyright (C) 2015 ReactOS Team
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
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Services
//  * FILE:             base/services/wkssvc/rpcserver.c
//  * PURPOSE:          Workstation service
//  * PROGRAMMER:       Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #include "lmerr.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wkssvc);
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD
// WINAPI
// RpcThreadRoutine(
//     LPVOID lpParameter)
// {
//     RPC_STATUS Status;
//
//     Status = RpcServerUseProtseqEpW(L"ncacn_np", 20, L"\\pipe\\wkssvc", NULL);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerRegisterIf(wkssvc_v1_0_s_ifspec, NULL, NULL);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerListen() failed (Status %lx)\n", Status);
//     }
//
//     return 0;
// }
//
//
// void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// static
// NET_API_STATUS
// NetpGetClientLogonId(
//      PLUID LogonId)
// {
//     HANDLE ThreadToken = NULL;
//     TOKEN_STATISTICS Statistics;
//     ULONG Length;
//     NTSTATUS NtStatus;
//     NET_API_STATUS ApiStatus = NERR_Success;
//
//     ApiStatus = RpcImpersonateClient(NULL);
//     if (ApiStatus != NERR_Success)
//         return ApiStatus;
//
//     NtStatus = NtOpenThreadToken(NtCurrentThread(),
//                                  TOKEN_QUERY,
//                                  TRUE,
//                                  &ThreadToken);
//     if (!NT_SUCCESS(NtStatus))
//     {
//         ApiStatus = RtlNtStatusToDosError(NtStatus);
//         goto done;
//     }
//
//     NtStatus = NtQueryInformationToken(ThreadToken,
//                                        TokenStatistics,
//                                        (PVOID)&Statistics,
//                                        sizeof(Statistics),
//                                        &Length);
//     if (!NT_SUCCESS(NtStatus))
//     {
//         ApiStatus = RtlNtStatusToDosError(NtStatus);
//         goto done;
//     }
//
//     TRACE("Client LUID: %lx\n", Statistics.AuthenticationId.LowPart);
//     RtlCopyLuid(LogonId, &Statistics.AuthenticationId);
//
// done:
//     if (ThreadToken != NULL)
//         NtClose(ThreadToken);
//
//     RpcRevertToSelf();
//
//     return ApiStatus;
// }
//
//
// /* Function 0 */
// unsigned long
// __stdcall
// NetrWkstaGetInfo(
//      WKSSVC_IDENTIFY_HANDLE ServerName,
//      unsigned long Level,
//      LPWKSTA_INFO WkstaInfo)
// {
//     WCHAR szComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//     DWORD dwComputerNameLength;
//     LPCWSTR pszLanRoot = L"";
//     PWKSTA_INFO_100 pWkstaInfo100 = NULL;
//     PWKSTA_INFO_101 pWkstaInfo101 = NULL;
//     PWKSTA_INFO_102 pWkstaInfo102 = NULL;
//     PWKSTA_INFO_502 pWkstaInfo502 = NULL;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes;
//     LSA_HANDLE PolicyHandle;
//     PPOLICY_PRIMARY_DOMAIN_INFO DomainInfo = NULL;
//     ULONG LoggedOnUsers;
//     NTSTATUS NtStatus;
//     DWORD dwResult = NERR_Success;
//
//     TRACE("NetrWkstaGetInfo level %lu\n", Level);
//
//     dwComputerNameLength = MAX_COMPUTERNAME_LENGTH + 1;
//     GetComputerNameW(szComputerName, &dwComputerNameLength);
//     dwComputerNameLength++; /* include NULL terminator */
//
//     ZeroMemory(&ObjectAttributes, sizeof(ObjectAttributes));
//     NtStatus = LsaOpenPolicy(NULL,
//                              &ObjectAttributes,
//                              POLICY_VIEW_LOCAL_INFORMATION,
//                              &PolicyHandle);
//     if (NtStatus != STATUS_SUCCESS)
//     {
//         WARN("LsaOpenPolicy() failed (Status 0x%08lx)\n", NtStatus);
//         return LsaNtStatusToWinError(NtStatus);
//     }
//
//     NtStatus = LsaQueryInformationPolicy(PolicyHandle,
//                                          PolicyPrimaryDomainInformation,
//                                          (PVOID*)&DomainInfo);
//
//     LsaClose(PolicyHandle);
//
//     if (NtStatus != STATUS_SUCCESS)
//     {
//         WARN("LsaQueryInformationPolicy() failed (Status 0x%08lx)\n", NtStatus);
//         return LsaNtStatusToWinError(NtStatus);
//     }
//
//     if (Level == 102)
//     {
//         MSV1_0_ENUMUSERS_REQUEST EnumRequest;
//         PMSV1_0_ENUMUSERS_RESPONSE EnumResponseBuffer = NULL;
//         DWORD EnumResponseBufferSize = 0;
//         NTSTATUS ProtocolStatus;
//
//         /* enumerate all currently logged-on users */
//         EnumRequest.MessageType = MsV1_0EnumerateUsers;
//         NtStatus = LsaCallAuthenticationPackage(LsaHandle,
//                                                 LsaAuthenticationPackage,
//                                                 &EnumRequest,
//                                                 sizeof(EnumRequest),
//                                                 (PVOID*)&EnumResponseBuffer,
//                                                 &EnumResponseBufferSize,
//                                                 &ProtocolStatus);
//         if (!NT_SUCCESS(NtStatus))
//         {
//             dwResult = RtlNtStatusToDosError(NtStatus);
//             goto done;
//         }
//
//         LoggedOnUsers = EnumResponseBuffer->NumberOfLoggedOnUsers;
//
//         LsaFreeReturnBuffer(EnumResponseBuffer);
//     }
//
//     switch (Level)
//     {
//         case 100:
//             pWkstaInfo100 = midl_user_allocate(sizeof(WKSTA_INFO_100));
//             if (pWkstaInfo100 == NULL)
//             {
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             pWkstaInfo100->wki100_platform_id = PLATFORM_ID_NT;
//
//             pWkstaInfo100->wki100_computername = midl_user_allocate(dwComputerNameLength * sizeof(WCHAR));
//             if (pWkstaInfo100->wki100_computername != NULL)
//                 wcscpy(pWkstaInfo100->wki100_computername, szComputerName);
//
//             pWkstaInfo100->wki100_langroup = midl_user_allocate((wcslen(DomainInfo->Name.Buffer) + 1) * sizeof(WCHAR));
//             if (pWkstaInfo100->wki100_langroup != NULL)
//                 wcscpy(pWkstaInfo100->wki100_langroup, DomainInfo->Name.Buffer);
//
//             pWkstaInfo100->wki100_ver_major = VersionInfo.dwMajorVersion;
//             pWkstaInfo100->wki100_ver_minor = VersionInfo.dwMinorVersion;
//
//             WkstaInfo->WkstaInfo100 = pWkstaInfo100;
//             break;
//
//         case 101:
//             pWkstaInfo101 = midl_user_allocate(sizeof(WKSTA_INFO_101));
//             if (pWkstaInfo101 == NULL)
//             {
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             pWkstaInfo101->wki101_platform_id = PLATFORM_ID_NT;
//
//             pWkstaInfo101->wki101_computername = midl_user_allocate(dwComputerNameLength * sizeof(WCHAR));
//             if (pWkstaInfo101->wki101_computername != NULL)
//                 wcscpy(pWkstaInfo101->wki101_computername, szComputerName);
//
//             pWkstaInfo101->wki101_langroup = midl_user_allocate((wcslen(DomainInfo->Name.Buffer) + 1) * sizeof(WCHAR));
//             if (pWkstaInfo101->wki101_langroup != NULL)
//                 wcscpy(pWkstaInfo101->wki101_langroup, DomainInfo->Name.Buffer);
//
//             pWkstaInfo101->wki101_ver_major = VersionInfo.dwMajorVersion;
//             pWkstaInfo101->wki101_ver_minor = VersionInfo.dwMinorVersion;
//
//             pWkstaInfo101->wki101_lanroot = midl_user_allocate((wcslen(pszLanRoot) + 1) * sizeof(WCHAR));
//             if (pWkstaInfo101->wki101_lanroot != NULL)
//                 wcscpy(pWkstaInfo101->wki101_lanroot, pszLanRoot);
//
//             WkstaInfo->WkstaInfo101 = pWkstaInfo101;
//             break;
//
//         case 102:
//             pWkstaInfo102 = midl_user_allocate(sizeof(WKSTA_INFO_102));
//             if (pWkstaInfo102 == NULL)
//             {
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             pWkstaInfo102->wki102_platform_id = PLATFORM_ID_NT;
//
//             pWkstaInfo102->wki102_computername = midl_user_allocate(dwComputerNameLength * sizeof(WCHAR));
//             if (pWkstaInfo102->wki102_computername != NULL)
//                 wcscpy(pWkstaInfo102->wki102_computername, szComputerName);
//
//             pWkstaInfo102->wki102_langroup = midl_user_allocate((wcslen(DomainInfo->Name.Buffer) + 1) * sizeof(WCHAR));
//             if (pWkstaInfo102->wki102_langroup != NULL)
//                 wcscpy(pWkstaInfo102->wki102_langroup, DomainInfo->Name.Buffer);
//
//             pWkstaInfo102->wki102_ver_major = VersionInfo.dwMajorVersion;
//             pWkstaInfo102->wki102_ver_minor = VersionInfo.dwMinorVersion;
//
//             pWkstaInfo102->wki102_lanroot = midl_user_allocate((wcslen(pszLanRoot) + 1) * sizeof(WCHAR));
//             if (pWkstaInfo102->wki102_lanroot != NULL)
//                 wcscpy(pWkstaInfo102->wki102_lanroot, pszLanRoot);
//
//             pWkstaInfo102->wki102_logged_on_users = LoggedOnUsers;
//
//             WkstaInfo->WkstaInfo102 = pWkstaInfo102;
//             break;
//
//         case 502:
//             pWkstaInfo502 = midl_user_allocate(sizeof(WKSTA_INFO_502));
//             if (pWkstaInfo502 == NULL)
//             {
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             CopyMemory(pWkstaInfo502, &WkstaInfo502, sizeof(WKSTA_INFO_502));
//
//             WkstaInfo->WkstaInfo502 = pWkstaInfo502;
//             break;
//
//         default:
//             FIXME("Level %lu unimplemented\n", Level);
//             dwResult = ERROR_INVALID_LEVEL;
//             break;
//     }
//
// done:
//     if (DomainInfo != NULL)
//         LsaFreeMemory(DomainInfo);
//
//     return dwResult;
// }
//
//
// /* Function 1 */
// unsigned long
// __stdcall
// NetrWkstaSetInfo(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     unsigned long Level,
//     LPWKSTA_INFO WkstaInfo,
//     unsigned long *ErrorParameter)
// {
//     DWORD dwErrParam = 0;
//     DWORD dwResult = NERR_Success;
//
//     TRACE("NetrWkstaSetInfo(%lu %p %p)\n",
//           Level, WkstaInfo, ErrorParameter);
//
//     switch (Level)
//     {
//         case 502:
//             if (WkstaInfo->WkstaInfo502->wki502_keep_conn >= 1 && WkstaInfo->WkstaInfo502->wki502_keep_conn <= 65535)
//             {
//                 WkstaInfo502.wki502_keep_conn = WkstaInfo->WkstaInfo502->wki502_keep_conn;
//             }
//             else
//             {
//                 dwErrParam = WKSTA_KEEPCONN_PARMNUM;
//                 dwResult = ERROR_INVALID_PARAMETER;
//             }
//
//             if (dwResult == NERR_Success)
//             {
//                 if (WkstaInfo->WkstaInfo502->wki502_max_cmds >= 50 && WkstaInfo->WkstaInfo502->wki502_max_cmds <= 65535)
//                 {
//                     WkstaInfo502.wki502_max_cmds = WkstaInfo->WkstaInfo502->wki502_max_cmds;
//                 }
//                 else
//                 {
//                     dwErrParam = WKSTA_MAXCMDS_PARMNUM;
//                     dwResult = ERROR_INVALID_PARAMETER;
//                 }
//             }
//
//             if (dwResult == NERR_Success)
//             {
//                 if (WkstaInfo->WkstaInfo502->wki502_sess_timeout >= 60 && WkstaInfo->WkstaInfo502->wki502_sess_timeout <= 65535)
//                 {
//                     WkstaInfo502.wki502_sess_timeout = WkstaInfo->WkstaInfo502->wki502_sess_timeout;
//                 }
//                 else
//                 {
//                     dwErrParam = WKSTA_SESSTIMEOUT_PARMNUM;
//                     dwResult = ERROR_INVALID_PARAMETER;
//                 }
//             }
//
//             if (dwResult == NERR_Success)
//             {
//                 if (WkstaInfo->WkstaInfo502->wki502_dormant_file_limit != 0)
//                 {
//                     WkstaInfo502.wki502_dormant_file_limit = WkstaInfo->WkstaInfo502->wki502_dormant_file_limit;
//                 }
//                 else
//                 {
//                     dwErrParam = WKSTA_DORMANTFILELIMIT_PARMNUM;
//                     dwResult = ERROR_INVALID_PARAMETER;
//                 }
//             }
//             break;
//
//         case 1013:
//             if (WkstaInfo->WkstaInfo1013->wki1013_keep_conn >= 1 && WkstaInfo->WkstaInfo1013->wki1013_keep_conn <= 65535)
//             {
//                 WkstaInfo502.wki502_keep_conn = WkstaInfo->WkstaInfo1013->wki1013_keep_conn;
//             }
//             else
//             {
//                 dwErrParam = WKSTA_KEEPCONN_PARMNUM;
//                 dwResult = ERROR_INVALID_PARAMETER;
//             }
//             break;
//
//         case 1018:
//             if (WkstaInfo->WkstaInfo1018->wki1018_sess_timeout >= 60 && WkstaInfo->WkstaInfo1018->wki1018_sess_timeout <= 65535)
//             {
//                 WkstaInfo502.wki502_sess_timeout = WkstaInfo->WkstaInfo1018->wki1018_sess_timeout;
//             }
//             else
//             {
//                 dwErrParam = WKSTA_SESSTIMEOUT_PARMNUM;
//                 dwResult = ERROR_INVALID_PARAMETER;
//             }
//             break;
//
//         case 1046:
//             if (WkstaInfo->WkstaInfo1046->wki1046_dormant_file_limit != 0)
//             {
//                 WkstaInfo502.wki502_dormant_file_limit = WkstaInfo->WkstaInfo1046->wki1046_dormant_file_limit;
//             }
//             else
//             {
//                 dwErrParam = WKSTA_DORMANTFILELIMIT_PARMNUM;
//                 dwResult = ERROR_INVALID_PARAMETER;
//             }
//             break;
//
//         default:
//             ERR("Invalid Level %lu\n", Level);
//             dwResult = ERROR_INVALID_LEVEL;
//             break;
//     }
//
//     /* Save the workstation in the registry */
//     if (dwResult == NERR_Success)
//     {
//         SaveWorkstationInfo(Level);
//
//         /* FIXME: Notify the redirector */
//     }
//
//     if ((dwResult == ERROR_INVALID_PARAMETER) && (ErrorParameter != NULL))
//         *ErrorParameter = dwErrParam;
//
//     return dwResult;
// }
//
//
// /* Function 2 */
// unsigned long
// __stdcall
// NetrWkstaUserEnum(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     LPWKSTA_USER_ENUM_STRUCT UserInfo,
//     unsigned long PreferredMaximumLength,
//     unsigned long *TotalEntries,
//     unsigned long *ResumeHandle)
// {
//     MSV1_0_ENUMUSERS_REQUEST EnumRequest;
//     PMSV1_0_ENUMUSERS_RESPONSE EnumResponseBuffer = NULL;
//     MSV1_0_GETUSERINFO_REQUEST UserInfoRequest;
//     PMSV1_0_GETUSERINFO_RESPONSE UserInfoResponseBuffer = NULL;
//     PMSV1_0_GETUSERINFO_RESPONSE *UserInfoArray = NULL;
//     DWORD EnumResponseBufferSize = 0;
//     DWORD UserInfoResponseBufferSize = 0;
//     NTSTATUS Status, ProtocolStatus;
//     ULONG i, start, count;
//     PLUID pLogonId;
//     PULONG pEnumHandle;
//     DWORD dwResult = NERR_Success;
//
//     PWKSTA_USER_INFO_0 pUserInfo0 = NULL;
//     PWKSTA_USER_INFO_1 pUserInfo1 = NULL;
//
//     TRACE("NetrWkstaUserEnum(%p %p 0x%lx %p %p)\n",
//           ServerName, UserInfo, PreferredMaximumLength, TotalEntries, ResumeHandle);
//
//     if (UserInfo->Level > 1)
//     {
//         ERR("Invalid Level %lu\n", UserInfo->Level);
//         return ERROR_INVALID_LEVEL;
//     }
//
//     /* Enumerate all currently logged-on users */
//     EnumRequest.MessageType = MsV1_0EnumerateUsers;
//     Status = LsaCallAuthenticationPackage(LsaHandle,
//                                           LsaAuthenticationPackage,
//                                           &EnumRequest,
//                                           sizeof(EnumRequest),
//                                           (PVOID*)&EnumResponseBuffer,
//                                           &EnumResponseBufferSize,
//                                           &ProtocolStatus);
//
//     TRACE("LsaCallAuthenticationPackage Status 0x%08lx ResponseBufferSize %lu\n", Status, EnumResponseBufferSize);
//     if (!NT_SUCCESS(Status))
//     {
//         dwResult = RtlNtStatusToDosError(Status);
//         goto done;
//     }
//
//     TRACE("LoggedOnUsers: %lu\n", EnumResponseBuffer->NumberOfLoggedOnUsers);
//     TRACE("ResponseBuffer: 0x%p\n", EnumResponseBuffer);
//     TRACE("LogonIds: 0x%p\n", EnumResponseBuffer->LogonIds);
//     TRACE("EnumHandles: 0x%p\n", EnumResponseBuffer->EnumHandles);
//     if (EnumResponseBuffer->NumberOfLoggedOnUsers > 0)
//     {
//         pLogonId = EnumResponseBuffer->LogonIds;
//         pEnumHandle = EnumResponseBuffer->EnumHandles;
//         TRACE("pLogonId: 0x%p\n", pLogonId);
//         TRACE("pEnumHandle: 0x%p\n", pEnumHandle);
//
//         UserInfoArray = RtlAllocateHeap(RtlGetProcessHeap(),
//                                         HEAP_ZERO_MEMORY,
//                                         EnumResponseBuffer->NumberOfLoggedOnUsers * sizeof(PMSV1_0_GETUSERINFO_RESPONSE));
//         if (UserInfoArray == NULL)
//         {
//             dwResult = ERROR_NOT_ENOUGH_MEMORY;
//             goto done;
//         }
//
//         for (i = 0; i < EnumResponseBuffer->NumberOfLoggedOnUsers; i++)
//         {
//             TRACE("Logon %lu: 0x%08lx  %lu\n", i, pLogonId->LowPart, *pEnumHandle);
//
//             UserInfoRequest.MessageType = MsV1_0GetUserInfo;
//             UserInfoRequest.LogonId = *pLogonId;
//             Status = LsaCallAuthenticationPackage(LsaHandle,
//                                                   LsaAuthenticationPackage,
//                                                   &UserInfoRequest,
//                                                   sizeof(UserInfoRequest),
//                                                   (PVOID*)&UserInfoResponseBuffer,
//                                                   &UserInfoResponseBufferSize,
//                                                   &ProtocolStatus);
//             TRACE("LsaCallAuthenticationPackage:MsV1_0GetUserInfo Status 0x%08lx ResponseBufferSize %lu\n", Status, UserInfoResponseBufferSize);
//             if (!NT_SUCCESS(Status))
//             {
//                 dwResult = RtlNtStatusToDosError(Status);
//                 goto done;
//             }
//
//             UserInfoArray[i] = UserInfoResponseBuffer;
//
//             TRACE("UserName: %wZ\n", &UserInfoArray[i]->UserName);
//             TRACE("LogonDomain: %wZ\n", &UserInfoArray[i]->LogonDomainName);
//             TRACE("LogonServer: %wZ\n", &UserInfoArray[i]->LogonServer);
//
//             pLogonId++;
//             pEnumHandle++;
//         }
//
//         if (PreferredMaximumLength == MAX_PREFERRED_LENGTH)
//         {
//             start = 0;
//             count = EnumResponseBuffer->NumberOfLoggedOnUsers;
//         }
//         else
//         {
//             FIXME("Calculate the start index and the number of matching array entries!");
//             dwResult = ERROR_CALL_NOT_IMPLEMENTED;
//             goto done;
//         }
//
//         switch (UserInfo->Level)
//         {
//             case 0:
//                 pUserInfo0 = midl_user_allocate(count * sizeof(WKSTA_USER_INFO_0));
//                 if (pUserInfo0 == NULL)
//                 {
//                     ERR("\n");
//                     dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                     break;
//                 }
//
//                 ZeroMemory(pUserInfo0, count * sizeof(WKSTA_USER_INFO_0));
//
//                 for (i = 0; i < 0 + count; i++) 
//                 {
//                     pUserInfo0[i].wkui0_username = midl_user_allocate(UserInfoArray[start + i]->UserName.Length + sizeof(WCHAR));
//                     if (pUserInfo0[i].wkui0_username == NULL)
//                     {
//                         ERR("\n");
//                         dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                         break;
//                     }
//
//                     ZeroMemory(pUserInfo0[i].wkui0_username, UserInfoArray[start + i]->UserName.Length + sizeof(WCHAR));
//                     CopyMemory(pUserInfo0[i].wkui0_username, UserInfoArray[start + i]->UserName.Buffer, UserInfoArray[start + i]->UserName.Length);
//                 }
//
//                 UserInfo->WkstaUserInfo.Level0.as_mut().EntriesRead = count;
//                 UserInfo->WkstaUserInfo.Level0.as_mut().Buffer = pUserInfo0;
//                 *TotalEntries = EnumResponseBuffer->NumberOfLoggedOnUsers;
//                 *ResumeHandle = 0;
//                 break;
//
//             case 1:
//                 pUserInfo1 = midl_user_allocate(count * sizeof(WKSTA_USER_INFO_1));
//                 if (pUserInfo1 == NULL)
//                 {
//                     ERR("\n");
//                     dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                     break;
//                 }
//
//                 ZeroMemory(pUserInfo1, count * sizeof(WKSTA_USER_INFO_1));
//
//                 for (i = 0; i < 0 + count; i++) 
//                 {
//                     pUserInfo1[i].wkui1_username = midl_user_allocate(UserInfoArray[start + i]->UserName.Length + sizeof(WCHAR));
//                     if (pUserInfo1[i].wkui1_username == NULL)
//                     {
//                         ERR("\n");
//                         dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                         break;
//                     }
//
//                     ZeroMemory(pUserInfo1[i].wkui1_username, UserInfoArray[start + i]->UserName.Length + sizeof(WCHAR));
//                     CopyMemory(pUserInfo1[i].wkui1_username, UserInfoArray[start + i]->UserName.Buffer, UserInfoArray[start + i]->UserName.Length);
//
//                     pUserInfo1[i].wkui1_logon_domain = midl_user_allocate(UserInfoArray[start + i]->LogonDomainName.Length + sizeof(WCHAR));
//                     if (pUserInfo1[i].wkui1_logon_domain == NULL)
//                     {
//                         ERR("\n");
//                         dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                         break;
//                     }
//
//                     ZeroMemory(pUserInfo1[i].wkui1_logon_domain, UserInfoArray[start + i]->LogonDomainName.Length + sizeof(WCHAR));
//                     CopyMemory(pUserInfo1[i].wkui1_logon_domain, UserInfoArray[start + i]->LogonDomainName.Buffer, UserInfoArray[start + i]->LogonDomainName.Length);
//
//                     // FIXME: wkui1_oth_domains
//
//                     pUserInfo1[i].wkui1_logon_server = midl_user_allocate(UserInfoArray[start + i]->LogonServer.Length + sizeof(WCHAR));
//                     if (pUserInfo1[i].wkui1_logon_server == NULL)
//                     {
//                         ERR("\n");
//                         dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                         break;
//                     }
//
//                     ZeroMemory(pUserInfo1[i].wkui1_logon_server, UserInfoArray[start + i]->LogonServer.Length + sizeof(WCHAR));
//                     CopyMemory(pUserInfo1[i].wkui1_logon_server, UserInfoArray[start + i]->LogonServer.Buffer, UserInfoArray[start + i]->LogonServer.Length);
//                 }
//
//                 UserInfo->WkstaUserInfo.Level1.as_mut().EntriesRead = count;
//                 UserInfo->WkstaUserInfo.Level1.as_mut().Buffer = pUserInfo1;
//                 *TotalEntries = EnumResponseBuffer->NumberOfLoggedOnUsers;
//                 *ResumeHandle = 0;
//                 break;
//
//                 break;
//         }
//     }
//     else
//     {
//         if (UserInfo->Level == 0)
//         {
//             UserInfo->WkstaUserInfo.Level0.as_mut().Buffer = NULL;
//             UserInfo->WkstaUserInfo.Level0.as_mut().EntriesRead = 0;
//         }
//         else
//         {
//             UserInfo->WkstaUserInfo.Level1.as_mut().Buffer = NULL;
//             UserInfo->WkstaUserInfo.Level1.as_mut().EntriesRead = 0;
//         }
//
//         *TotalEntries = 0;
//         dwResult = NERR_Success;
//     }
//
// done:
//     if (UserInfoArray !=NULL)
//     {
//
//         for (i = 0; i < EnumResponseBuffer->NumberOfLoggedOnUsers; i++)
//         {
//             if (UserInfoArray[i]->UserName.Buffer != NULL)
//                 LsaFreeReturnBuffer(UserInfoArray[i]->UserName.Buffer);
//
//             if (UserInfoArray[i]->LogonDomainName.Buffer != NULL)
//                 LsaFreeReturnBuffer(UserInfoArray[i]->LogonDomainName.Buffer);
//
//             if (UserInfoArray[i]->LogonServer.Buffer != NULL)
//                 LsaFreeReturnBuffer(UserInfoArray[i]->LogonServer.Buffer);
//
//             LsaFreeReturnBuffer(UserInfoArray[i]);
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, UserInfoArray);
//     }
//
//     if (EnumResponseBuffer != NULL)
//         LsaFreeReturnBuffer(EnumResponseBuffer);
//
//     return dwResult;
// }
//
//
// /* Function 3 */
// unsigned long
// __stdcall
// NetrWkstaUserGetInfo(
//     WKSSVC_IDENTIFY_HANDLE Unused,
//     unsigned long Level,
//     LPWKSTA_USER_INFO UserInfo)
// {
//     MSV1_0_GETUSERINFO_REQUEST UserInfoRequest;
//     PMSV1_0_GETUSERINFO_RESPONSE UserInfoResponseBuffer = NULL;
//     DWORD UserInfoResponseBufferSize = 0;
//     NTSTATUS Status, ProtocolStatus;
//     LUID LogonId;
//     PWKSTA_USER_INFO_0 pUserInfo0 = NULL;
//     PWKSTA_USER_INFO_1 pUserInfo1 = NULL;
//     PWKSTA_USER_INFO_1101 pUserInfo1101 = NULL;
//     DWORD dwResult = NERR_Success;
//
//     TRACE("NetrWkstaUserGetInfo(%s, %d, %p)\n", debugstr_w(Unused), Level, UserInfo);
//
//     if (Unused != NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     if (Level > 1 && Level != 1101)
//         return ERROR_INVALID_LEVEL;
//
//     if (Level != 1101)
//     {
//         dwResult = NetpGetClientLogonId(&LogonId);
//         if (dwResult != NERR_Success)
//         {
//             ERR("NetpGetClientLogonId() failed (%u)\n", dwResult);
//             return dwResult;
//         }
//
//         TRACE("LogonId: 0x%08lx\n", LogonId.LowPart);
//
//         UserInfoRequest.MessageType = MsV1_0GetUserInfo;
//         UserInfoRequest.LogonId = LogonId;
//         Status = LsaCallAuthenticationPackage(LsaHandle,
//                                               LsaAuthenticationPackage,
//                                               &UserInfoRequest,
//                                               sizeof(UserInfoRequest),
//                                               (PVOID*)&UserInfoResponseBuffer,
//                                               &UserInfoResponseBufferSize,
//                                               &ProtocolStatus);
//         TRACE("LsaCallAuthenticationPackage:MsV1_0GetUserInfo Status 0x%08lx ResponseBufferSize %lu\n", Status, UserInfoResponseBufferSize);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("\n");
//             return RtlNtStatusToDosError(Status);
//         }
//
//         TRACE("UserName: %wZ\n", &UserInfoResponseBuffer->UserName);
//         TRACE("LogonDomain: %wZ\n", &UserInfoResponseBuffer->LogonDomainName);
//         TRACE("LogonServer: %wZ\n", &UserInfoResponseBuffer->LogonServer);
//     }
//
//     switch (Level)
//     {
//         case 0:
//             pUserInfo0 = midl_user_allocate(sizeof(WKSTA_USER_INFO_0));
//             if (pUserInfo0 == NULL)
//             {
//                 ERR("\n");
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo0, sizeof(WKSTA_USER_INFO_0));
//
//             /* User Name */
//             pUserInfo0->wkui0_username =
//                 midl_user_allocate(UserInfoResponseBuffer->UserName.Length + sizeof(WCHAR));
//             if (pUserInfo0->wkui0_username == NULL)
//             {
//                 ERR("\n");
//                 midl_user_free(pUserInfo0);
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo0->wkui0_username,
//                        UserInfoResponseBuffer->UserName.Length + sizeof(WCHAR));
//             CopyMemory(pUserInfo0->wkui0_username,
//                        UserInfoResponseBuffer->UserName.Buffer,
//                        UserInfoResponseBuffer->UserName.Length);
//
//             UserInfo->UserInfo0 = pUserInfo0;
//             break;
//
//         case 1:
//             pUserInfo1 = midl_user_allocate(sizeof(WKSTA_USER_INFO_1));
//             if (pUserInfo1 == NULL)
//             {
//                 ERR("\n");
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo1, sizeof(WKSTA_USER_INFO_1));
//
//             /* User Name */
//             pUserInfo1->wkui1_username =
//                 midl_user_allocate(UserInfoResponseBuffer->UserName.Length + sizeof(WCHAR));
//             if (pUserInfo1->wkui1_username == NULL)
//             {
//                 ERR("\n");
//                 midl_user_free(pUserInfo1);
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo1->wkui1_username,
//                        UserInfoResponseBuffer->UserName.Length + sizeof(WCHAR));
//             CopyMemory(pUserInfo1->wkui1_username,
//                        UserInfoResponseBuffer->UserName.Buffer,
//                        UserInfoResponseBuffer->UserName.Length);
//
//             /* Logon Domain Name */
//             pUserInfo1->wkui1_logon_domain =
//                 midl_user_allocate(UserInfoResponseBuffer->LogonDomainName.Length + sizeof(WCHAR));
//             if (pUserInfo1->wkui1_logon_domain == NULL)
//             {
//                 ERR("\n");
//                 midl_user_free(pUserInfo1->wkui1_username);
//                 midl_user_free(pUserInfo1);
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo1->wkui1_logon_domain,
//                        UserInfoResponseBuffer->LogonDomainName.Length + sizeof(WCHAR));
//             CopyMemory(pUserInfo1->wkui1_logon_domain,
//                        UserInfoResponseBuffer->LogonDomainName.Buffer,
//                        UserInfoResponseBuffer->LogonDomainName.Length);
//
//             /* FIXME: wkui1_oth_domains */
//
//             /* Logon Server */
//             pUserInfo1->wkui1_logon_server =
//                 midl_user_allocate(UserInfoResponseBuffer->LogonServer.Length + sizeof(WCHAR));
//             if (pUserInfo1->wkui1_logon_server == NULL)
//             {
//                 ERR("\n");
//                 midl_user_free(pUserInfo1->wkui1_username);
//                 midl_user_free(pUserInfo1->wkui1_logon_domain);
//                 midl_user_free(pUserInfo1);
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo1->wkui1_logon_server,
//                        UserInfoResponseBuffer->LogonServer.Length + sizeof(WCHAR));
//             CopyMemory(pUserInfo1->wkui1_logon_server,
//                        UserInfoResponseBuffer->LogonServer.Buffer,
//                        UserInfoResponseBuffer->LogonServer.Length);
//
//             UserInfo->UserInfo1 = pUserInfo1;
//             break;
//
//         case 1101:
//             pUserInfo1101 = midl_user_allocate(sizeof(WKSTA_USER_INFO_1101));
//             if (pUserInfo1101 == NULL)
//             {
//                 ERR("Failed to allocate WKSTA_USER_INFO_1101\n");
//                 dwResult = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             ZeroMemory(pUserInfo1101, sizeof(WKSTA_USER_INFO_1101));
//
//             /* FIXME: wkui1101_oth_domains */
//
//             UserInfo->UserInfo1101 = pUserInfo1101;
//             break;
//
//         default:
//             ERR("\n");
//             dwResult = ERROR_INVALID_LEVEL;
//             break;
//     }
//
//     if (UserInfoResponseBuffer)
//         LsaFreeReturnBuffer(UserInfoResponseBuffer);
//
//     return dwResult;
// }
//
//
// /* Function 4 */
// unsigned long
// __stdcall
// NetrWkstaUserSetInfo(
//     WKSSVC_IDENTIFY_HANDLE Unused,
//     unsigned long Level,
//     LPWKSTA_USER_INFO UserInfo,
//     unsigned long *ErrorParameter)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 5 */
// unsigned long
// __stdcall
// NetrWkstaTransportEnum(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     LPWKSTA_TRANSPORT_ENUM_STRUCT TransportInfo,
//     unsigned long PreferredMaximumLength,
//     unsigned long* TotalEntries,
//     unsigned long *ResumeHandle)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 6 */
// unsigned long
// __stdcall
// NetrWkstaTransportAdd(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     unsigned long Level,
//     LPWKSTA_TRANSPORT_INFO_0 TransportInfo,
//     unsigned long *ErrorParameter)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 7 */
// unsigned long
// __stdcall
// NetrWkstaTransportDel(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     wchar_t *TransportName,
//     unsigned long ForceLevel)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 8 */
// unsigned long
// __stdcall
// NetrUseAdd(
//     WKSSVC_IMPERSONATE_HANDLE ServerName,
//     unsigned long Level,
//     LPUSE_INFO InfoStruct,
//     unsigned long *ErrorParameter)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 9 */
// unsigned long
// __stdcall
// NetrUseGetInfo(
//     WKSSVC_IMPERSONATE_HANDLE ServerName,
//     wchar_t *UseName,
//     unsigned long Level,
//     LPUSE_INFO InfoStruct)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 10 */
// unsigned long
// __stdcall
// NetrUseDel(
//     WKSSVC_IMPERSONATE_HANDLE ServerName,
//     wchar_t *UseName,
//     unsigned long ForceLevel)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 11 */
// unsigned long
// __stdcall
// NetrUseEnum(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     LPUSE_ENUM_STRUCT InfoStruct,
//     unsigned long PreferredMaximumLength,
//     unsigned long *TotalEntries,
//     unsigned long *ResumeHandle)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 12 - Not used on wire */
// unsigned long
// __stdcall
// NetrMessageBufferSend(void)
// {
//     TRACE("NetrMessageBufferSend()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 13 */
// unsigned long
// __stdcall
// NetrWorkstationStatisticsGet(
//     WKSSVC_IDENTIFY_HANDLE ServerName,
//     wchar_t *ServiceName,
//     unsigned long Level,
//     unsigned long Options,
//     LPSTAT_WORKSTATION_0 *Buffer)
// {
//     SYSTEM_TIMEOFDAY_INFORMATION TimeOfDayInfo;
//     PSTAT_WORKSTATION_0 pStatBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("NetrWorkstationStatisticsGet(%p %p %lu 0x%lx %p)\n",
//           ServerName, ServiceName, Level, Options, Buffer);
//
//     if (Level != 0)
//         return ERROR_INVALID_LEVEL;
//
//     if (Options != 0)
//         return ERROR_INVALID_PARAMETER;
//
//     pStatBuffer = midl_user_allocate(sizeof(STAT_WORKSTATION_0));
//     if (pStatBuffer == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     ZeroMemory(pStatBuffer, sizeof(STAT_WORKSTATION_0));
//
//     /* Query the boot time */
//     Status = NtQuerySystemInformation(SystemTimeOfDayInformation,
//                                       &TimeOfDayInfo,
//                                       sizeof(TimeOfDayInfo),
//                                       NULL);
//     if (NT_SUCCESS(Status))
//     {
//         ULONG Seconds = 0;
//         if (RtlTimeToSecondsSince1970(&TimeOfDayInfo.BootTime, &Seconds))
//         {
//             pStatBuffer->StatisticsStartTime.u.HighPart = 0;
//             pStatBuffer->StatisticsStartTime.u.LowPart = Seconds;
//         }
//     }
//
//     // FIXME: Return the actual statistcs data!
//
//     *Buffer = pStatBuffer;
//
//     return NERR_Success;
// }
//
//
// /* Function 14 - Not used on wire */
// unsigned long
// __stdcall
// NetrLogonDomainNameAdd(
//     WKSSVC_IDENTIFY_HANDLE DomainName)
// {
//     TRACE("NetrLogonDomainNameAdd(%s)\n",
//           debugstr_w(DomainName));
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 15 - Not used on wire */
// unsigned long
// __stdcall
// NetrLogonDomainNameDel(
//     WKSSVC_IDENTIFY_HANDLE DomainName)
// {
//     TRACE("NetrLogonDomainNameDel(%s)\n",
//           debugstr_w(DomainName));
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 16 - Not used on wire */
// unsigned long
// __stdcall
// NetrJoinDomain(void)
// {
//     TRACE("NetrJoinDomain()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 17 - Not used on wire */
// unsigned long
// __stdcall
// NetrUnjoinDomain(void)
// {
//     TRACE("NetrUnjoinDomain()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 18 - Not used on wire */
// unsigned long
// __stdcall
// NetrValidateName(void)
// {
//     TRACE("NetrValidateName()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 19 - Not used on wire */
// unsigned long
// __stdcall
// NetrRenameMachineInDomain(void)
// {
//     TRACE("NetrRenameMachineInDomain()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 20 */
// unsigned long
// __stdcall
// NetrGetJoinInformation(
//     WKSSVC_IMPERSONATE_HANDLE ServerName,
//     wchar_t **NameBuffer,
//     PNETSETUP_JOIN_STATUS BufferType)
// {
//     TRACE("NetrGetJoinInformation(%p %p %p)\n",
//           ServerName, NameBuffer, BufferType);
//
//     if (NameBuffer == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     return NetpGetJoinInformation(NameBuffer,
//                                   BufferType);
// }
//
//
// /* Function 21 - Not used on wire */
// unsigned long
// __stdcall
// NetrGetJoinableOUs(void)
// {
//     TRACE("NetrGetJoinableOUs()\n");
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// /* Function 22 */
// unsigned long
// __stdcall
// NetrJoinDomain2(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *DomainNameParam,
//     wchar_t *MachineAccountOU,
//     wchar_t *AccountName,
//     PJOINPR_ENCRYPTED_USER_PASSWORD Password,
//     unsigned long Options)
// {
//     NET_API_STATUS status;
//
//     FIXME("NetrJoinDomain2(%p %S %S %S %S %p 0x%lx)\n",
//           RpcBindingHandle, ServerName, DomainNameParam, MachineAccountOU,
//           AccountName, Password, Options);
//
//     if (DomainNameParam == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     if (Options & NETSETUP_JOIN_DOMAIN)
//     {
//         FIXME("NetrJoinDomain2: NETSETUP_JOIN_DOMAIN is not supported yet!\n");
//         status = ERROR_CALL_NOT_IMPLEMENTED;
//     }
//     else
//     {
//         status = NetpJoinWorkgroup(DomainNameParam);
//     }
//
//     return status;
// }
//
//
// /* Function 23 */
// unsigned long
// __stdcall
// NetrUnjoinDomain2(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *AccountName,
//     PJOINPR_ENCRYPTED_USER_PASSWORD Password,
//     unsigned long Options)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 24 */
// unsigned long
// __stdcall
// NetrRenameMachineInDomain2(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *MachineName,
//     wchar_t *AccountName,
//     PJOINPR_ENCRYPTED_USER_PASSWORD Password,
//     unsigned long Options)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 25 */
// unsigned long
// __stdcall
// NetrValidateName2(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *NameToValidate,
//     wchar_t *AccountName,
//     PJOINPR_ENCRYPTED_USER_PASSWORD Password,
//     NETSETUP_NAME_TYPE NameType)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 26 */
// unsigned long
// __stdcall
// NetrGetJoinableOUs2(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *DomainNameParam,
//     wchar_t *AccountName,
//     PJOINPR_ENCRYPTED_USER_PASSWORD Password,
//     unsigned long* OUCount,
//     wchar_t ***OUs)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 27 */
// unsigned long
// __stdcall
// NetrAddAlternateComputerName(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *AlternateName,
//     wchar_t *DomainAccount,
//     PJOINPR_ENCRYPTED_USER_PASSWORD EncryptedPassword,
//     unsigned long Reserved)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 28 */
// unsigned long
// __stdcall
// NetrRemoveAlternateComputerName(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *AlternateName,
//     wchar_t *DomainAccount,
//     PJOINPR_ENCRYPTED_USER_PASSWORD EncryptedPassword,
//     unsigned long Reserved)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 29 */
// unsigned long
// __stdcall
// NetrSetPrimaryComputerName(
//     handle_t RpcBindingHandle,
//     wchar_t *ServerName,
//     wchar_t *PrimaryName,
//     wchar_t *DomainAccount,
//     PJOINPR_ENCRYPTED_USER_PASSWORD EncryptedPassword,
//     unsigned long Reserved)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 30 */
// unsigned long
// __stdcall
// NetrEnumerateComputerNames(
//     WKSSVC_IMPERSONATE_HANDLE ServerName,
//     NET_COMPUTER_NAME_TYPE NameType,
//     unsigned long Reserved,
//     PNET_COMPUTER_NAME_ARRAY *ComputerNames)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rpcndr.h:94 `#define midl_user_allocate MIDL_user_allocate` — the C source spells these
// lowercase but the preprocessor emits the CAPITALISED symbol, which is what the generated
// wkssvc_s.c marshaller binds to. The #[no_mangle] definitions above therefore carry the
// capitalised name; these aliases keep the C's source spelling working for in-crate callers.
pub use self::{MIDL_user_allocate as midl_user_allocate, MIDL_user_free as midl_user_free};
