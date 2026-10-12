// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_EFFBBBFADB50: [u16; 13] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x0073, 0x0072, 0x0076, 0x0073,
    0x0076, 0x0063, 0x0000,
];

use core::ffi::c_void;


#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS;

    Status = RpcServerUseProtseqEpW(RUSTCONV_WSTR_12A6BF85117F.as_ptr() as *mut u16, 20, RUSTCONV_WSTR_EFFBBBFADB50.as_ptr() as *mut u16, core::ptr::null_mut());
    if Status != RPC_S_OK {
        ERR!("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = RpcServerRegisterIf(srvsvc_v3_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
    if Status != RPC_S_OK {
        ERR!("RpcServerRegisterIf() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
    if Status != RPC_S_OK {
        ERR!("RpcServerListen() failed (Status %lx)\n", Status);
    }

    return 0;
}
#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut ::core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, len) as *mut ::core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut ::core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn SHARE_DEL_HANDLE_rundown(hClientHandle: SHARE_DEL_HANDLE) {}

#[no_mangle]
pub unsafe extern "system" fn Opnum0NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum1NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum2NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum3NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum4NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum5NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum6NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn Opnum7NotUsedOnWire() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NetrConnectionEnum(
    ServerName: SRVSVC_HANDLE,
    Qualifier: *mut WCHAR,
    InfoStruct: LPCONNECT_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrFileEnum(
    ServerName: SRVSVC_HANDLE,
    BasePath: *mut WCHAR,
    UserName: *mut WCHAR,
    InfoStruct: PFILE_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrFileGetInfo(
    ServerName: SRVSVC_HANDLE,
    FileId: DWORD,
    Level: DWORD,
    InfoStruct: LPFILE_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrFileClose(
    ServerName: SRVSVC_HANDLE,
    FileId: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrSessionEnum(
    ServerName: SRVSVC_HANDLE,
    ClientName: *mut WCHAR,
    UserName: *mut WCHAR,
    InfoStruct: PSESSION_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrSessionDel(
    ServerName: SRVSVC_HANDLE,
    ClientName: *mut WCHAR,
    UserName: *mut WCHAR,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareAdd(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    InfoStruct: LPSHARE_INFO,
    ParmErr: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareEnum(
    ServerName: SRVSVC_HANDLE,
    InfoStruct: LPSHARE_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareGetInfo(
    ServerName: SRVSVC_HANDLE,
    NetName: *mut WCHAR,
    Level: DWORD,
    InfoStruct: LPSHARE_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareSetInfo(
    ServerName: SRVSVC_HANDLE,
    NetName: *mut WCHAR,
    Level: DWORD,
    ShareInfo: LPSHARE_INFO,
    ParmErr: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareDel(
    ServerName: SRVSVC_HANDLE,
    NetName: *mut WCHAR,
    Reserved: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareDelSticky(
    ServerName: SRVSVC_HANDLE,
    NetName: *mut WCHAR,
    Reserved: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareCheck(
    ServerName: SRVSVC_HANDLE,
    Device: *mut WCHAR,
    Type: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NetrServerGetInfo(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    InfoStruct: *mut LPSERVER_INFO,
) -> DWORD {
    let mut szComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as _] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as _];
    let mut dwComputerNameLength: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pServerInfo: PSERVER_INFO = core::ptr::null_mut();
    let mut VersionInfo: OSVERSIONINFOW = core::mem::zeroed();

    dwComputerNameLength = (MAX_COMPUTERNAME_LENGTH + 1) as DWORD;
    GetComputerNameW(
        core::ptr::addr_of_mut!(szComputerName) as LPWSTR,
        core::ptr::addr_of_mut!(dwComputerNameLength),
    );
    dwComputerNameLength = dwComputerNameLength.wrapping_add(1); /* include NULL terminator */

    VersionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;
    GetVersionExW(core::ptr::addr_of_mut!(VersionInfo));

    match Level {
        100 => {
            dwSize = (core::mem::size_of::<SERVER_INFO_100>() as usize).wrapping_add(
                (dwComputerNameLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            ) as DWORD;
            pServerInfo = midl_user_allocate(dwSize as SIZE_T) as PSERVER_INFO;
            if pServerInfo.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            (*pServerInfo).ServerInfo100.sv100_platform_id = PLATFORM_ID_NT as DWORD;
            (*pServerInfo).ServerInfo100.sv100_name = (pServerInfo as usize)
                .wrapping_add(core::mem::size_of::<SERVER_INFO_100>() as usize)
                as LPWSTR;
            wcscpy(
                (*pServerInfo).ServerInfo100.sv100_name as *mut WCHAR,
                core::ptr::addr_of_mut!(szComputerName) as *mut WCHAR,
            );

            *InfoStruct = pServerInfo as LPSERVER_INFO;
        }

        101 => {
            dwSize = (core::mem::size_of::<SERVER_INFO_101>() as usize).wrapping_add(
                (dwComputerNameLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            ) as DWORD;
            pServerInfo = midl_user_allocate(dwSize as SIZE_T) as PSERVER_INFO;
            if pServerInfo.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            (*pServerInfo).ServerInfo101.sv101_platform_id = PLATFORM_ID_NT as DWORD;
            (*pServerInfo).ServerInfo101.sv101_name = (pServerInfo as usize)
                .wrapping_add(core::mem::size_of::<SERVER_INFO_101>() as usize)
                as LPWSTR;
            wcscpy(
                (*pServerInfo).ServerInfo101.sv101_name as *mut WCHAR,
                core::ptr::addr_of_mut!(szComputerName) as *mut WCHAR,
            );

            (*pServerInfo).ServerInfo101.sv101_version_major = VersionInfo.dwMajorVersion;
            (*pServerInfo).ServerInfo101.sv101_version_minor = VersionInfo.dwMinorVersion;
            (*pServerInfo).ServerInfo101.sv101_type =
                (dwServiceBits | SV_TYPE_NT as DWORD) as DWORD;
            (*pServerInfo).ServerInfo101.sv101_comment = core::ptr::null_mut::<WCHAR>(); /* FIXME */

            *InfoStruct = pServerInfo as LPSERVER_INFO;
        }

        102 => {
            dwSize = (core::mem::size_of::<SERVER_INFO_102>() as usize).wrapping_add(
                (dwComputerNameLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            ) as DWORD;
            pServerInfo = midl_user_allocate(dwSize as SIZE_T) as PSERVER_INFO;
            if pServerInfo.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            (*pServerInfo).ServerInfo102.sv102_platform_id = PLATFORM_ID_NT as DWORD;
            (*pServerInfo).ServerInfo102.sv102_name = (pServerInfo as usize)
                .wrapping_add(core::mem::size_of::<SERVER_INFO_102>() as usize)
                as LPWSTR;
            wcscpy(
                (*pServerInfo).ServerInfo102.sv102_name as *mut WCHAR,
                core::ptr::addr_of_mut!(szComputerName) as *mut WCHAR,
            );

            (*pServerInfo).ServerInfo102.sv102_version_major = VersionInfo.dwMajorVersion;
            (*pServerInfo).ServerInfo102.sv102_version_minor = VersionInfo.dwMinorVersion;
            (*pServerInfo).ServerInfo102.sv102_type =
                (dwServiceBits | SV_TYPE_NT as DWORD) as DWORD;
            (*pServerInfo).ServerInfo102.sv102_comment = core::ptr::null_mut::<WCHAR>(); /* FIXME */

            (*pServerInfo).ServerInfo102.sv102_users = 0 as DWORD; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_disc = 0 as LONG; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_hidden = SV_VISIBLE as BOOL; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_announce = 0 as DWORD; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_anndelta = 0 as DWORD; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_licenses = 0 as DWORD; /* FIXME */
            (*pServerInfo).ServerInfo102.sv102_userpath = core::ptr::null_mut::<WCHAR>(); /* FIXME */

            *InfoStruct = pServerInfo as LPSERVER_INFO;
        }

        _ => {
            FIXME!("level %d unimplemented\n", Level);
            return ERROR_INVALID_LEVEL as DWORD;
        }
    }

    return 0 as DWORD; // NERR_Success
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerSetInfo(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    ServerInfo: LPSERVER_INFO,
    ParmErr: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerDiskEnum(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    DiskInfoStruct: *mut DISK_ENUM_CONTAINER,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NetrServerStatisticsGet(
    ServerName: SRVSVC_HANDLE,
    Service: *mut WCHAR,
    Level: DWORD,
    Options: DWORD,
    InfoStruct: *mut LPSTAT_SERVER_0,
) -> NET_API_STATUS {
    let mut TimeOfDayInfo: SYSTEM_TIMEOFDAY_INFORMATION = core::mem::zeroed();
    let mut pStatBuffer: PSTAT_SERVER_0 = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        "NetrServerStatisticsGet(%p %p %lu 0x%lx %p)\n",
        ServerName,
        Service,
        Level,
        Options,
        InfoStruct
    );

    if Level != 0 {
        return ERROR_INVALID_LEVEL;
    }

    if Options != 0 {
        return ERROR_INVALID_PARAMETER;
    }

    pStatBuffer = midl_user_allocate(
        core::mem::size_of::<STAT_SERVER_0>() as SIZE_T
    ) as PSTAT_SERVER_0;
    if pStatBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    ZeroMemory!(
        pStatBuffer as *mut core::ffi::c_void,
        core::mem::size_of::<STAT_SERVER_0>(),
    );

    /* Query the boot time */
    Status = NtQuerySystemInformation(
        SystemTimeOfDayInformation,
        core::ptr::addr_of_mut!(TimeOfDayInfo) as PVOID,
        core::mem::size_of::<SYSTEM_TIMEOFDAY_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS!(Status) {
        let mut Seconds: ULONG = 0;
        if RtlTimeToSecondsSince1970(
            core::ptr::addr_of_mut!(TimeOfDayInfo.BootTime),
            core::ptr::addr_of_mut!(Seconds),
        ) != 0
        {
            (*pStatBuffer).sts0_start = Seconds as DWORD;
        }
    }

    // FIXME: Return the actual statistcs data!

    *InfoStruct = pStatBuffer as LPSTAT_SERVER_0;

    return NERR_Success;
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTransportAdd(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    Buffer: LPSERVER_TRANSPORT_INFO_0,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTransportEnum(
    ServerName: SRVSVC_HANDLE,
    InfoStruct: LPSERVER_XPORT_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTransportDel(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    Buffer: LPSERVER_TRANSPORT_INFO_0,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn NetrRemoteTOD(
    ServerName: SRVSVC_HANDLE,
    BufferPtr: *mut LPTIME_OF_DAY_INFO,
) -> DWORD {
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeZoneInfo: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut TimeZoneId: DWORD = 0;
    let mut lpTod: LPTIME_OF_DAY_INFO = core::ptr::null_mut();

    *BufferPtr = midl_user_allocate(core::mem::size_of::<TIME_OF_DAY_INFO>() as SIZE_T) as LPTIME_OF_DAY_INFO;
    if (*BufferPtr).is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    lpTod = *BufferPtr;

    /* Set the seconds since 1970 */
    NtQuerySystemTime(&raw mut Time);
    RtlTimeToSecondsSince1970(&raw mut Time, &raw mut (*lpTod).tod_elapsedt);

    /* Set the tick count */
    (*lpTod).tod_msecs = GetTickCount() as DWORD;

    /* Set the timezone */
    TimeZoneId = GetTimeZoneInformation(&raw mut TimeZoneInfo) as DWORD;

    if TimeZoneId == TIME_ZONE_ID_UNKNOWN as DWORD {
        (*lpTod).tod_timezone = TimeZoneInfo.Bias;
    } else if TimeZoneId == TIME_ZONE_ID_STANDARD as DWORD {
        (*lpTod).tod_timezone = TimeZoneInfo.Bias + TimeZoneInfo.StandardBias;
    } else if TimeZoneId == TIME_ZONE_ID_DAYLIGHT as DWORD {
        (*lpTod).tod_timezone = TimeZoneInfo.Bias + TimeZoneInfo.DaylightBias;
    } else {
        (*lpTod).tod_timezone = 0 as LONG;
    }

    /* Set the ??? */
    (*lpTod).tod_tinterval = 310 as DWORD;

    /* Set the date and time */
    GetSystemTime(&raw mut SystemTime);
    (*lpTod).tod_hours = SystemTime.wHour as DWORD;
    (*lpTod).tod_mins = SystemTime.wMinute as DWORD;
    (*lpTod).tod_secs = SystemTime.wSecond as DWORD;
    (*lpTod).tod_hunds = (SystemTime.wMilliseconds as DWORD) / (10 as DWORD);
    (*lpTod).tod_day = SystemTime.wDay as DWORD;
    (*lpTod).tod_month = SystemTime.wMonth as DWORD;
    (*lpTod).tod_year = SystemTime.wYear as DWORD;
    (*lpTod).tod_weekday = SystemTime.wDayOfWeek as DWORD;

    return 0; /* NERR_Success */
}
#[no_mangle]
pub unsafe extern "system" fn NetrServerSetServiceBits(mut ServerName: SRVSVC_HANDLE, mut Transport: *mut WCHAR, mut ServiceBits: DWORD, mut UpdateImmediately: DWORD) -> DWORD {
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"base/services/srvsvc/rpcserver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetrServerSetServiceBits\0" as *const u8 as *const ::core::ffi::c_char,
            639 as ::core::ffi::c_int,
            b"NetrServerSetServiceBits(%p %s %lx %lu)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            ServerName,
            debugstr_w(Transport),
            ServiceBits,
            UpdateImmediately,
        );
    }
    dwServiceBits = ServiceBits;
    return 0; // NERR_Success
}

#[no_mangle]
pub unsafe extern "system" fn NetprPathType(
    ServerName: SRVSVC_HANDLE,
    PathName: *mut WCHAR,
    PathType: *mut DWORD,
    Flags: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetprPathCanonicalize(
    ServerName: SRVSVC_HANDLE,
    PathName: *mut WCHAR,
    Outbuf: *mut u8,
    OutbufLen: DWORD,
    Prefix: *mut WCHAR,
    PathType: *mut DWORD,
    Flags: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetprPathCompare(
    ServerName: SRVSVC_HANDLE,
    PathName1: *mut WCHAR,
    PathName2: *mut WCHAR,
    PathType: DWORD,
    Flags: DWORD,
) -> i32 {
    // UNIMPLEMENTED;
    (ERROR_CALL_NOT_IMPLEMENTED as i32)
}

#[no_mangle]
pub unsafe extern "system" fn NetprNameValidate(
    ServerName: SRVSVC_HANDLE,
    Name: *mut WCHAR,
    NameType: DWORD,
    Flags: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetprNameCanonicalize(
    ServerName: SRVSVC_HANDLE,
    Name: *mut WCHAR,
    Outbuf: *mut WCHAR,
    OutbufLen: DWORD,
    NameType: DWORD,
    Flags: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetprNameCompare(
    ServerName: SRVSVC_HANDLE,
    Name1: *mut WCHAR,
    Name2: *mut WCHAR,
    NameType: DWORD,
    Flags: DWORD,
) -> i32 {
    // UNIMPLEMENTED;
    (ERROR_CALL_NOT_IMPLEMENTED as i32)
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareEnumSticky(
    ServerName: SRVSVC_HANDLE,
    InfoStruct: LPSHARE_ENUM_STRUCT,
    PreferedMaximumLength: DWORD,
    TotalEntries: *mut DWORD,
    ResumeHandle: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareDelStart(
    ServerName: SRVSVC_HANDLE,
    NetName: *mut WCHAR,
    Reserved: DWORD,
    ContextHandle: PSHARE_DEL_HANDLE,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrShareDelCommit(
    ContextHandle: PSHARE_DEL_HANDLE,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrpGetFileSecurity(
    ServerName: SRVSVC_HANDLE,
    ShareName: *mut WCHAR,
    lpFileName: *mut WCHAR,
    RequestedInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PADT_SECURITY_DESCRIPTOR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrpSetFileSecurity(
    ServerName: SRVSVC_HANDLE,
    ShareName: *mut WCHAR,
    lpFileName: *mut WCHAR,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PADT_SECURITY_DESCRIPTOR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTransportAddEx(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    Buffer: LPTRANSPORT_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerSetServiceBitsEx(
    ServerName: SRVSVC_HANDLE,
    EmulatedServer: *mut WCHAR,
    Transport: *mut WCHAR,
    ServiceBitsOfInterest: DWORD,
    ServiceBits: DWORD,
    UpdateImmediately: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsGetVersion(
    ServerName: SRVSVC_HANDLE,
    Version: *mut DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsCreateLocalPartition(
    ServerName: SRVSVC_HANDLE,
    ShareName: *mut WCHAR,
    EntryUid: *mut GUID,
    EntryPrefix: *mut WCHAR,
    ShortName: *mut WCHAR,
    RelationInfo: LPNET_DFS_ENTRY_ID_CONTAINER,
    Force: i32,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsDeleteLocalPartition(
    ServerName: SRVSVC_HANDLE,
    Uid: *mut GUID,
    Prefix: *mut WCHAR,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsSetLocalVolumeState(
    ServerName: SRVSVC_HANDLE,
    Uid: *mut GUID,
    Prefix: *mut WCHAR,
    State: u32,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsSetServerInfo() -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsCreateExitPoint(
    ServerName: SRVSVC_HANDLE,
    Uid: *mut GUID,
    Prefix: *mut WCHAR,
    Type: u32,
    ShortPrefixLen: DWORD,
    ShortPrefix: *mut WCHAR,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsDeleteExitPoint(
    ServerName: SRVSVC_HANDLE,
    Uid: *mut GUID,
    Prefix: *mut WCHAR,
    Type: u32,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsModifyPrefix(
    ServerName: SRVSVC_HANDLE,
    Uid: *mut GUID,
    Prefix: *mut WCHAR,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsFixLocalVolume(
    ServerName: SRVSVC_HANDLE,
    VolumeName: *mut WCHAR,
    EntryType: u32,
    ServiceType: u32,
    StgId: *mut WCHAR,
    EntryUid: *mut GUID,
    EntryPrefix: *mut WCHAR,
    RelationInfo: LPNET_DFS_ENTRY_ID_CONTAINER,
    CreateDisposition: u32,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrDfsManagerReportSiteInfo(
    ServerName: SRVSVC_HANDLE,
    ppSiteInfo: *mut LPDFS_SITELIST_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NetrServerTransportDelEx(
    ServerName: SRVSVC_HANDLE,
    Level: DWORD,
    Buffer: LPTRANSPORT_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
