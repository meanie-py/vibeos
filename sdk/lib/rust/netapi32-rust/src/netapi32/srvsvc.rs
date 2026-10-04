use crate::prelude_rpc::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (standard netapi32 wine-debug channel; TRACE/FIXME reference it) ----
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


// ---- SRVSVC_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn SRVSVC_HANDLE_bind(pszSystemName: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    let mut hBinding: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pszStringBinding: *mut u16 = core::ptr::null_mut();
    let mut status: i32;

    let protocol_seq: [u16; 9] = [
        0x006E, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000,
    ];
    let endpoint: [u16; 13] = [
        0x005C, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C,
        0x0073, 0x0072, 0x0076, 0x0073, 0x0076, 0x0063, 0x0000,
    ];

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        protocol_seq.as_ptr() as *mut u16,
        pszSystemName as *mut u16,
        endpoint.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(
        pszStringBinding,
        &mut hBinding as *mut *mut core::ffi::c_void,
    );

    status = RpcStringFreeW(&mut pszStringBinding);

    hBinding
}

// ---- SRVSVC_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn SRVSVC_HANDLE_unbind(
    pszSystemName: SRVSVC_HANDLE,
    hBinding: handle_t,
) {
    let _ = pszSystemName;
    let mut binding = hBinding;
    let status = RpcBindingFree(&mut binding);
    if status != 0 {
    }
}

// ---- NetConnectionEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetConnectionEnum(
    servername: *mut u16,
    qualifier: *mut u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    if level > 1 {
        return ERROR_INVALID_LEVEL;
    }

    let mut EnumStruct: _CONNECT_ENUM_STRUCT = core::mem::zeroed();
    let mut Level0Container: _CONNECT_INFO_0_CONTAINER = core::mem::zeroed();
    let mut Level1Container: _CONNECT_INFO_1_CONTAINER = core::mem::zeroed();

    EnumStruct.Level = level;
    match level {
        0 => {
            EnumStruct.ConnectInfo.Level0 = &mut Level0Container as *mut _CONNECT_INFO_0_CONTAINER;
        }
        1 => {
            EnumStruct.ConnectInfo.Level1 = &mut Level1Container as *mut _CONNECT_INFO_1_CONTAINER;
        }
        _ => {}
    }

    let status = rpc_try(|| NetrConnectionEnum(
        servername,
        qualifier,
        &mut EnumStruct as *mut _CONNECT_ENUM_STRUCT,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            if !(*EnumStruct.ConnectInfo.Level0).Buffer.is_null() {
                *bufptr = (*EnumStruct.ConnectInfo.Level0).Buffer as *mut u8;
                *entriesread = (*EnumStruct.ConnectInfo.Level0).EntriesRead;
            }
        }
        1 => {
            if !(*EnumStruct.ConnectInfo.Level1).Buffer.is_null() {
                *bufptr = (*EnumStruct.ConnectInfo.Level1).Buffer as *mut u8;
                *entriesread = (*EnumStruct.ConnectInfo.Level1).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetFileClose ----
#[no_mangle]
pub unsafe extern "system" fn NetFileClose(
    servername: *mut u16,
    fileid: u32,
) -> u32 {
    let status = rpc_try(|| NetrFileClose(servername, fileid)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetFileEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetFileEnum(
    // srvsvc.c:176-178 declares these LMSTR (LPWSTR), not const.
    servername: LMSTR,
    basepath: LMSTR,
    username: LMSTR,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut usize,
) -> u32 {
    if level != 2 && level != 3 {
        // srvsvc.c:196 returns ERROR_INVALID_LEVEL (124). The translation emitted the literal
        // 1213 under an ERROR_INVALID_LEVEL comment — 1213 is ERROR_INVALID_SERVICENAME, an
        // unrelated code that would have been handed back to every caller passing a bad level.
        return ERROR_INVALID_LEVEL;
    }

    let mut enum_struct: FILE_ENUM_STRUCT = core::mem::zeroed();
    let mut level2_container: FILE_INFO_2_CONTAINER = core::mem::zeroed();
    let mut level3_container: FILE_INFO_3_CONTAINER = core::mem::zeroed();

    enum_struct.Level = level;
    match level {
        2 => {
            enum_struct.FileInfo.Level2 = &mut level2_container as *mut FILE_INFO_2_CONTAINER;
        }
        3 => {
            enum_struct.FileInfo.Level3 = &mut level3_container as *mut FILE_INFO_3_CONTAINER;
        }
        _ => {}
    }

    // RpcTryExcept body (no SEH)
    let status = rpc_try(|| NetrFileEnum(
        servername,
        basepath,
        username,
        &mut enum_struct,
        prefmaxlen,
        totalentries,
        resume_handle as *mut u32,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        2 => {
            if !(*enum_struct.FileInfo.Level2).Buffer.is_null() {
                *bufptr = (*enum_struct.FileInfo.Level2).Buffer as *mut u8;
                *entriesread = (*enum_struct.FileInfo.Level2).EntriesRead;
            }
        }
        3 => {
            if !(*enum_struct.FileInfo.Level3).Buffer.is_null() {
                *bufptr = (*enum_struct.FileInfo.Level3).Buffer as *mut u8;
                *entriesread = (*enum_struct.FileInfo.Level3).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetFileGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetFileGetInfo(
    servername: *mut u16,
    fileid: u32,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    let status;
    *bufptr = core::ptr::null_mut();
    // srvsvc.c:269 — `(LPFILE_INFO)bufptr`
    status = rpc_try(|| NetrFileGetInfo(servername, fileid, level, bufptr as LPFILE_INFO)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetRemoteTOD ----
#[no_mangle]
pub unsafe extern "system" fn NetRemoteTOD(
    UncServerName: LPCWSTR,
    BufferPtr: *mut LPBYTE,
) -> NET_API_STATUS {
    *BufferPtr = core::ptr::null_mut();
    rpc_try(|| NetrRemoteTOD(
        UncServerName as SRVSVC_HANDLE,
        BufferPtr as *mut LPTIME_OF_DAY_INFO,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetServerDiskEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetServerDiskEnum(
    servername: *mut u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    // A local `struct DISK_ENUM_CONTAINER { EntriesRead: u32, Buffer: *mut u8 }` shadowed the
    // real wire type here; srvsvc_c.h declares Buffer as LPDISK_INFO. Use the island type.
    let mut EnumContainer = DISK_ENUM_CONTAINER {
        EntriesRead: 0,
        Buffer: core::ptr::null_mut(),
    };

    let status: u32 = rpc_try(|| NetrServerDiskEnum(
        servername,
        level,
        &mut EnumContainer,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !EnumContainer.Buffer.is_null() {
        // srvsvc.c:341 — `*bufptr = (LPBYTE)EnumContainer.Buffer;`
        *bufptr = EnumContainer.Buffer as LPBYTE;
    } else {
        *bufptr = core::ptr::null_mut();
    }

    if EnumContainer.EntriesRead > 0 {
        *entriesread = EnumContainer.EntriesRead - 1;
    } else {
        *entriesread = 0;
    }

    status
}

// ---- NetServerGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetServerGetInfo(
    servername: LPWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE
) -> NET_API_STATUS {
    *bufptr = core::ptr::null_mut();
    // srvsvc.c:385 — `(LPSERVER_INFO *)bufptr`
    let status = rpc_try(|| NetrServerGetInfo(servername, level, bufptr as *mut LPSERVER_INFO)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetServerSetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetServerSetInfo(
    servername: LPWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> NET_API_STATUS {
    // srvsvc.c:414 — `(LPSERVER_INFO)&buf`
    let status = rpc_try(|| NetrServerSetInfo(
        servername,
        level,
        core::ptr::addr_of!(buf) as LPSERVER_INFO,
        parm_err,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- I_NetServerSetServiceBits ----
#[no_mangle]
pub unsafe extern "system" fn I_NetServerSetServiceBits(
    servername: LPWSTR,
    transport: LPWSTR,
    servicebits: DWORD,
    updateimmediately: DWORD,
) -> NET_API_STATUS {
    rpc_try(|| NetrServerSetServiceBits(servername, transport, servicebits, updateimmediately)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetServerTransportAdd ----
#[no_mangle]
pub unsafe extern "system" fn NetServerTransportAdd(
    servername: LPWSTR,
    level: DWORD,
    bufptr: LPBYTE,
) -> NET_API_STATUS {
    rpc_try(|| NetrServerTransportAdd(servername, level, bufptr as *mut _)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetServerTransportAddEx ----
#[no_mangle]
pub unsafe extern "system" fn NetServerTransportAddEx(
    servername: LPWSTR,
    level: DWORD,
    bufptr: LPBYTE,
) -> NET_API_STATUS {
    let status = rpc_try(|| NetrServerTransportAddEx(servername, level, bufptr as *mut _)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetServerTransportDel ----
#[no_mangle]
pub unsafe extern "system" fn NetServerTransportDel(
    servername: LPWSTR,
    level: DWORD,
    bufptr: LPBYTE,
) -> NET_API_STATUS {
    let status = rpc_try(|| NetrServerTransportDel(servername, level, bufptr as *mut _)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetServerTransportEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetServerTransportEnum(
    servername: LPWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resume_handle: LPDWORD,
) -> NET_API_STATUS {
    let mut EnumStruct: SERVER_XPORT_ENUM_STRUCT = core::mem::zeroed();
    let mut Level0Container: SERVER_XPORT_INFO_0_CONTAINER = core::mem::zeroed();
    let mut Level1Container: SERVER_XPORT_INFO_1_CONTAINER = core::mem::zeroed();

    EnumStruct.Level = level;
    match level {
        0 => {
            EnumStruct.XportInfo.Level0 = &mut Level0Container;
        }
        1 => {
            EnumStruct.XportInfo.Level1 = &mut Level1Container;
        }
        _ => {}
    }

    // RpcTryExcept – executed directly, handler omitted as dead code
    let status = rpc_try(|| NetrServerTransportEnum(
        servername,
        &mut EnumStruct,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            let container = EnumStruct.XportInfo.Level0;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as LPBYTE;
                *entriesread = (*container).EntriesRead;
            }
        }
        1 => {
            let container = EnumStruct.XportInfo.Level1;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as LPBYTE;
                *entriesread = (*container).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetSessionDel ----
#[no_mangle]
pub unsafe extern "system" fn NetSessionDel(
    servername: *mut u16,
    UncClientName: *mut u16,
    username: *mut u16,
) -> NET_API_STATUS {
    rpc_try(|| NetrSessionDel(servername, UncClientName, username)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetSessionEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetSessionEnum(
    servername: *mut u16,
    UncClientName: *mut u16,
    username: *mut u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    let mut EnumStruct: SESSION_ENUM_STRUCT = core::mem::zeroed();
    let mut Level0Container: SESSION_INFO_0_CONTAINER = core::mem::zeroed();
    let mut Level1Container: SESSION_INFO_1_CONTAINER = core::mem::zeroed();
    let mut Level2Container: SESSION_INFO_2_CONTAINER = core::mem::zeroed();
    let mut Level10Container: SESSION_INFO_10_CONTAINER = core::mem::zeroed();
    let mut Level502Container: SESSION_INFO_502_CONTAINER = core::mem::zeroed();
    let mut status: u32;

    if level > 2 && level != 10 && level != 502 {
        return ERROR_INVALID_LEVEL;
    }

    *bufptr = core::ptr::null_mut();
    *entriesread = 0;

    EnumStruct.Level = level;
    match level {
        0 => EnumStruct.SessionInfo.Level0 = &mut Level0Container as *mut _,
        1 => EnumStruct.SessionInfo.Level1 = &mut Level1Container as *mut _,
        2 => EnumStruct.SessionInfo.Level2 = &mut Level2Container as *mut _,
        10 => EnumStruct.SessionInfo.Level10 = &mut Level10Container as *mut _,
        502 => EnumStruct.SessionInfo.Level502 = &mut Level502Container as *mut _,
        _ => {}
    }

    // RpcTryExcept omitted
    status = rpc_try(|| NetrSessionEnum(
        servername,
        UncClientName,
        username,
        &mut EnumStruct,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            if !(*EnumStruct.SessionInfo.Level0).Buffer.is_null() {
                *bufptr = (*EnumStruct.SessionInfo.Level0).Buffer as *mut u8;
                *entriesread = (*EnumStruct.SessionInfo.Level0).EntriesRead;
            }
        }
        1 => {
            if !(*EnumStruct.SessionInfo.Level1).Buffer.is_null() {
                *bufptr = (*EnumStruct.SessionInfo.Level1).Buffer as *mut u8;
                *entriesread = (*EnumStruct.SessionInfo.Level1).EntriesRead;
            }
        }
        2 => {
            if !(*EnumStruct.SessionInfo.Level2).Buffer.is_null() {
                *bufptr = (*EnumStruct.SessionInfo.Level2).Buffer as *mut u8;
                *entriesread = (*EnumStruct.SessionInfo.Level2).EntriesRead;
            }
        }
        10 => {
            if !(*EnumStruct.SessionInfo.Level10).Buffer.is_null() {
                *bufptr = (*EnumStruct.SessionInfo.Level10).Buffer as *mut u8;
                *entriesread = (*EnumStruct.SessionInfo.Level10).EntriesRead;
            }
        }
        502 => {
            if !(*EnumStruct.SessionInfo.Level502).Buffer.is_null() {
                *bufptr = (*EnumStruct.SessionInfo.Level502).Buffer as *mut u8;
                *entriesread = (*EnumStruct.SessionInfo.Level502).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetSessionGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetSessionGetInfo(
    servername: LPWSTR,
    UncClientName: LPWSTR,
    username: LPWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> DWORD {
    let mut Level0Container: SESSION_INFO_0_CONTAINER = core::mem::zeroed();
    let mut Level1Container: SESSION_INFO_1_CONTAINER = core::mem::zeroed();
    let mut Level2Container: SESSION_INFO_2_CONTAINER = core::mem::zeroed();
    let mut Level10Container: SESSION_INFO_10_CONTAINER = core::mem::zeroed();
    let mut dwTotalEntries: DWORD = 0;
    let mut EnumStruct: SESSION_ENUM_STRUCT = core::mem::zeroed();
    let mut status: DWORD;

    if level > 2 && level != 10 {
        return ERROR_INVALID_LEVEL;
    }

    if UncClientName.is_null() || username.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *bufptr = core::ptr::null_mut();

    EnumStruct.Level = level;
    match level {
        0 => EnumStruct.SessionInfo.Level0 = &mut Level0Container,
        1 => EnumStruct.SessionInfo.Level1 = &mut Level1Container,
        2 => EnumStruct.SessionInfo.Level2 = &mut Level2Container,
        10 => EnumStruct.SessionInfo.Level10 = &mut Level10Container,
        _ => {}
    }

    status = rpc_try(|| NetrSessionEnum(
        servername,
        UncClientName,
        username,
        &mut EnumStruct,
        0xFFFFFFFFu32, // MAX_PREFERRED_LENGTH
        &mut dwTotalEntries,
        core::ptr::null_mut(),
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            let level0 = EnumStruct.SessionInfo.Level0;
            if !(*level0).Buffer.is_null() {
                *bufptr = (*level0).Buffer as *mut u8;
            }
        }
        1 => {
            let level1 = EnumStruct.SessionInfo.Level1;
            if !(*level1).Buffer.is_null() {
                *bufptr = (*level1).Buffer as *mut u8;
            }
        }
        2 => {
            let level2 = EnumStruct.SessionInfo.Level2;
            if !(*level2).Buffer.is_null() {
                *bufptr = (*level2).Buffer as *mut u8;
            }
        }
        10 => {
            let level10 = EnumStruct.SessionInfo.Level10;
            if !(*level10).Buffer.is_null() {
                *bufptr = (*level10).Buffer as *mut u8;
            }
        }
        _ => {}
    }

    status
}

// ---- NetShareAdd ----
#[no_mangle]
pub unsafe extern "system" fn NetShareAdd(
    // srvsvc.c:851 declares this LMSTR (LPWSTR), not const.
    servername: LMSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> NET_API_STATUS {
    if level != 2 && level != 502 && level != 503 {
        return ERROR_INVALID_LEVEL;
    }

    let status = rpc_try(|| NetrShareAdd(
        servername,
        level,
        &buf as *const *mut u8 as LPSHARE_INFO,
        parm_err,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    status
}

// ---- NetShareCheck ----
#[no_mangle]
pub unsafe extern "system" fn NetShareCheck(
    servername: LPWSTR,
    device: LPWSTR,
    type_: LPDWORD,
) -> NET_API_STATUS {
    rpc_try(|| NetrShareCheck(servername, device, type_)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetShareDel ----
#[no_mangle]
pub unsafe extern "system" fn NetShareDel(
    servername: LMSTR,
    netname: LMSTR,
    reserved: DWORD,
) -> NET_API_STATUS {
    if netname.is_null() || *netname == 0 || reserved != 0 {
        return ERROR_INVALID_PARAMETER;
    }

    let status = rpc_try(|| NetrShareDel(servername, netname, reserved)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetShareDelSticky ----
#[no_mangle]
pub unsafe extern "system" fn NetShareDelSticky(
    servername: *mut u16,
    netname: *mut u16,
    reserved: u32,
) -> u32 {
    let mut status: u32;
    if netname.is_null() || (*netname == 0u16) || reserved != 0 {
        return ERROR_INVALID_PARAMETER;
    }
    status = rpc_try(|| NetrShareDelSticky(servername, netname, reserved)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    return status;
}

// ---- NetShareEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetShareEnum(
    servername: LMSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resume_handle: LPDWORD,
) -> NET_API_STATUS {
    let mut enum_struct: SHARE_ENUM_STRUCT = core::mem::zeroed();
    let mut level0_container: SHARE_INFO_0_CONTAINER = core::mem::zeroed();
    let mut level1_container: SHARE_INFO_1_CONTAINER = core::mem::zeroed();
    let mut level2_container: SHARE_INFO_2_CONTAINER = core::mem::zeroed();
    let mut level502_container: SHARE_INFO_502_CONTAINER = core::mem::zeroed();

    if level > 2 && level != 502 {
        return ERROR_INVALID_LEVEL;
    }

    *bufptr = core::ptr::null_mut();
    *entriesread = 0;
    *totalentries = 0;

    enum_struct.Level = level;
    match level {
        0 => {
            enum_struct.ShareInfo.Level0 = &mut level0_container;
        }
        1 => {
            enum_struct.ShareInfo.Level1 = &mut level1_container;
        }
        2 => {
            enum_struct.ShareInfo.Level2 = &mut level2_container;
        }
        502 => {
            enum_struct.ShareInfo.Level502 = &mut level502_container;
        }
        _ => {}
    }

    let status = rpc_try(|| NetrShareEnum(
        servername,
        &mut enum_struct,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            if !(*enum_struct.ShareInfo.Level0).Buffer.is_null() {
                *bufptr = (*enum_struct.ShareInfo.Level0).Buffer as LPBYTE;
                *entriesread = (*enum_struct.ShareInfo.Level0).EntriesRead;
            }
        }
        1 => {
            if !(*enum_struct.ShareInfo.Level1).Buffer.is_null() {
                *bufptr = (*enum_struct.ShareInfo.Level1).Buffer as LPBYTE;
                *entriesread = (*enum_struct.ShareInfo.Level1).EntriesRead;
            }
        }
        2 => {
            if !(*enum_struct.ShareInfo.Level2).Buffer.is_null() {
                *bufptr = (*enum_struct.ShareInfo.Level2).Buffer as LPBYTE;
                *entriesread = (*enum_struct.ShareInfo.Level2).EntriesRead;
            }
        }
        502 => {
            if !(*enum_struct.ShareInfo.Level502).Buffer.is_null() {
                *bufptr = (*enum_struct.ShareInfo.Level502).Buffer as LPBYTE;
                *entriesread = (*enum_struct.ShareInfo.Level502).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetShareEnumSticky ----
#[no_mangle]
pub unsafe extern "system" fn NetShareEnumSticky(
    servername: *mut u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    let mut enum_struct: SHARE_ENUM_STRUCT = core::mem::zeroed();
    let mut level0_container: SHARE_INFO_0_CONTAINER = core::mem::zeroed();
    let mut level1_container: SHARE_INFO_1_CONTAINER = core::mem::zeroed();
    let mut level2_container: SHARE_INFO_2_CONTAINER = core::mem::zeroed();
    let mut level502_container: SHARE_INFO_502_CONTAINER = core::mem::zeroed();

    if level > 2 && level != 502 {
        return ERROR_INVALID_LEVEL;
    }

    *bufptr = core::ptr::null_mut();
    *entriesread = 0;
    *totalentries = 0;

    enum_struct.Level = level;
    match level {
        0 => {
            enum_struct.ShareInfo.Level0 = core::ptr::addr_of_mut!(level0_container);
        }
        1 => {
            enum_struct.ShareInfo.Level1 = core::ptr::addr_of_mut!(level1_container);
        }
        2 => {
            enum_struct.ShareInfo.Level2 = core::ptr::addr_of_mut!(level2_container);
        }
        502 => {
            enum_struct.ShareInfo.Level502 = core::ptr::addr_of_mut!(level502_container);
        }
        _ => {}
    }

    let status = rpc_try(|| NetrShareEnum(
        servername,
        &mut enum_struct as *mut SHARE_ENUM_STRUCT as LPSHARE_ENUM_STRUCT,
        prefmaxlen,
        totalentries,
        resume_handle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match level {
        0 => {
            let container = enum_struct.ShareInfo.Level0;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as *mut u8;
                *entriesread = (*container).EntriesRead;
            }
        }
        1 => {
            let container = enum_struct.ShareInfo.Level1;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as *mut u8;
                *entriesread = (*container).EntriesRead;
            }
        }
        2 => {
            let container = enum_struct.ShareInfo.Level2;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as *mut u8;
                *entriesread = (*container).EntriesRead;
            }
        }
        502 => {
            let container = enum_struct.ShareInfo.Level502;
            if !(*container).Buffer.is_null() {
                *bufptr = (*container).Buffer as *mut u8;
                *entriesread = (*container).EntriesRead;
            }
        }
        _ => {}
    }

    status
}

// ---- NetShareGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetShareGetInfo(
    // srvsvc.c:1178-1179 declares these LMSTR (LPWSTR), not const.
    servername: LMSTR,
    netname: LMSTR,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    if level > 2 && level != 502 && level != 1005 {
        return ERROR_INVALID_LEVEL;
    }
    if netname.is_null() || *netname == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    *bufptr = core::ptr::null_mut();

    // srvsvc.c:1201 — `(LPSHARE_INFO)bufptr`
    let status = rpc_try(|| NetrShareGetInfo(servername, netname, level, bufptr as LPSHARE_INFO)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetShareSetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetShareSetInfo(
    servername: LPWSTR,
    netname: LPWSTR,
    level: DWORD,
    // `&buf` is taken below (matching the C's `(LPSHARE_INFO)&buf`), so this binding needs `mut`.
    mut buf: LPBYTE,
    parm_err: LPDWORD,
) -> NET_API_STATUS {
    if level != 2 && level != 502 && level != 503 && level != 1004 &&
        level != 1005 && level != 1006 && level != 1501 {
        return ERROR_INVALID_LEVEL;
    }

    let status = rpc_try(|| NetrShareSetInfo(
        servername,
        netname,
        level,
        // srvsvc.c:1236 — `(LPSHARE_INFO)&buf`
        core::ptr::addr_of_mut!(buf) as LPSHARE_INFO,
        parm_err,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}
