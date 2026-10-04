use crate::prelude_browser::*;
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


// ---- BROWSER_IDENTIFY_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn BROWSER_IDENTIFY_HANDLE_bind(pszSystemName: BROWSER_IDENTIFY_HANDLE) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: *mut u16 = core::ptr::null_mut();
    let status: RPC_STATUS;

    let ncacn_np: [u16; 9] = [110, 99, 97, 99, 110, 95, 110, 112, 0]; // "ncacn_np\0"
    let pipe_browser: [u16; 14] = [0x5c, 0x70, 0x69, 0x70, 0x65, 0x5c, 0x62, 0x72, 0x6f, 0x77, 0x73, 0x65, 0x72, 0]; // "\\pipe\\browser\0"

    // RPC_WSTR is `*mut WCHAR` — the C hands wide literals straight to these parameters.
    let status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        pszSystemName as *mut u16,
        pipe_browser.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding as *mut *mut u16,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    let status = RpcBindingFromStringBindingW(
        pszStringBinding,
        &mut hBinding as *mut handle_t,
    );

    let status = RpcStringFreeW(&mut pszStringBinding as *mut *mut u16);

    hBinding
}

// ---- BROWSER_IDENTIFY_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn BROWSER_IDENTIFY_HANDLE_unbind(
    pszSystemName: BROWSER_IDENTIFY_HANDLE,
    mut hBinding: handle_t,
) {
    // RpcBindingFree NULLs the caller's handle through the pointer, so the local needs `mut`.
    let status = RpcBindingFree(&mut hBinding as *mut handle_t);
    if status != 0 {
    }
}

// ---- I_BrowserDebugTrace ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserDebugTrace(
    ServerName: LPWSTR,
    Buffer: PCHAR,
) -> NET_API_STATUS {
    rpc_try(|| I_BrowserrDebugTrace(ServerName, Buffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- I_BrowserQueryEmulatedDomains ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserQueryEmulatedDomains(
    ServerName: LPWSTR,
    EmulatedDomains: *mut PBROWSER_EMULATED_DOMAIN,
    EntriesRead: LPDWORD,
) -> NET_API_STATUS {
    let mut Container: BROWSER_EMULATED_DOMAIN_CONTAINER = core::mem::zeroed();
    *EmulatedDomains = core::ptr::null_mut();
    *EntriesRead = 0;

    let status = rpc_try(|| I_BrowserrQueryEmulatedDomains(ServerName, &mut Container)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == NERR_Success {
        *EmulatedDomains = Container.Buffer;
        *EntriesRead = Container.EntriesRead;
    }

    status
}

// ---- I_BrowserQueryOtherDomains ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserQueryOtherDomains(
    ServerName: LPCWSTR,
    // C is `LPBYTE *BufPtr` (browser.c:145) — an OUT pointer. The translation dropped a level.
    BufPtr: *mut LPBYTE,
    EntriesRead: LPDWORD,
    TotalEntries: LPDWORD,
) -> NET_API_STATUS {
    let mut Level100Container: _SERVER_INFO_100_CONTAINER = core::mem::zeroed();
    // Level100Container is zero-initialized; matches {0, NULL}
    let mut EnumStruct: _SERVER_ENUM_STRUCT = core::mem::zeroed();
    EnumStruct.Level = 100;
    // browser_c.h:372 names the union member `ServerInfo` — it is NOT anonymous, so bindgen
    // kept the name and there is no __bindgen_anon_1 here.
    EnumStruct.ServerInfo.Level100 = &mut Level100Container;

    // RpcTryExcept omitted per rules
    let status = rpc_try(|| I_BrowserrQueryOtherDomains(
        ServerName as PWSTR,
        &mut EnumStruct,
        TotalEntries,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == NERR_Success || status == ERROR_MORE_DATA {
        *BufPtr = (*(EnumStruct.ServerInfo.Level100)).Buffer as LPBYTE;
        *EntriesRead = (*(EnumStruct.ServerInfo.Level100)).EntriesRead;
    }
    status
}

// ---- I_BrowserServerEnum ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserServerEnum(
    ServerName: LPCWSTR,
    Transport: LPCWSTR,
    ClientName: LPCWSTR,
    Level: DWORD,
    BufPtr: *mut LPBYTE,
    PrefMaxLen: DWORD,
    EntriesRead: LPDWORD,
    TotalEntries: LPDWORD,
    ServerType: DWORD,
    Domain: LPCWSTR,
    ResumeHandle: LPDWORD,
) -> NET_API_STATUS {
    let mut Level100Container: _SERVER_INFO_100_CONTAINER = core::mem::zeroed();
    let mut EnumStruct: _SERVER_ENUM_STRUCT = core::mem::zeroed();

    EnumStruct.Level = 100;
    EnumStruct.ServerInfo.Level100 = &mut Level100Container as *mut _SERVER_INFO_100_CONTAINER;

    let status = rpc_try(|| I_BrowserrServerEnum(
        ServerName as PWSTR,
        Transport as PWSTR,
        ClientName as PWSTR,
        &mut EnumStruct,
        PrefMaxLen,
        TotalEntries,
        ServerType,
        Domain as PWSTR,
        ResumeHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == NERR_Success || status == ERROR_MORE_DATA {
        *BufPtr = (*EnumStruct.ServerInfo.Level100).Buffer as LPBYTE;
        *EntriesRead = (*EnumStruct.ServerInfo.Level100).EntriesRead;
    }

    status
}

// ---- I_BrowserQueryStatistics ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserQueryStatistics(
    ServerName: LPCWSTR,
    Statistics: *mut LPBROWSER_STATISTICS
) -> NET_API_STATUS {
    let status: NET_API_STATUS;
    status = rpc_try(|| I_BrowserrQueryStatistics(ServerName as PWSTR, Statistics)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- I_BrowserResetStatistics ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserResetStatistics(ServerName: LPCWSTR) -> NET_API_STATUS {
    let status = rpc_try(|| I_BrowserrResetStatistics(ServerName as PWSTR)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- I_BrowserResetNetlogonState ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserResetNetlogonState(ServerName: LPCWSTR) -> NET_API_STATUS {
    rpc_try(|| I_BrowserrResetNetlogonState(ServerName as PWSTR)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- I_BrowserSetNetlogonState ----
#[no_mangle]
pub unsafe extern "system" fn I_BrowserSetNetlogonState(
    ServerName: PWSTR,
    DomainName: PWSTR,
    EmulatedServerName: PWSTR,
    Role: DWORD,
) -> u32 {
    // TRY body executed directly, handler omitted per translation rules
    let status = rpc_try(|| I_BrowserrSetNetlogonState(ServerName, DomainName, EmulatedServerName, Role)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetBrowserStatisticsGet ----
#[no_mangle]
pub unsafe extern "system" fn NetBrowserStatisticsGet(
    ServerName: LPWSTR,
    Level: DWORD,
    // C is `LPBYTE *Buffer` (browser.c:345) — an OUT pointer. The translation dropped a level.
    Buffer: *mut LPBYTE,
) -> NET_API_STATUS {
    let mut Level100Container: BROWSER_STATISTICS_100_CONTAINER = core::mem::zeroed();
    let mut Level101Container: BROWSER_STATISTICS_101_CONTAINER = core::mem::zeroed();
    let mut StatisticsStruct: BROWSER_STATISTICS_STRUCT = core::mem::zeroed();

    if Level != 100 && Level != 101 {
        return ERROR_INVALID_LEVEL;
    }

    StatisticsStruct.Level = Level;
    match Level {
        100 => {
            StatisticsStruct.Statistics.Level100 = &mut Level100Container;
        }
        101 => {
            StatisticsStruct.Statistics.Level101 = &mut Level101Container;
        }
        // browser.c:359 is a `switch` with only the 100/101 cases and NO default — Level is
        // already validated above, so the C simply falls through. `unreachable!()` here would
        // have introduced a panic path the C never had.
        _ => {}
    }

    let status = rpc_try(|| NetrBrowserStatisticsGet(ServerName, Level, &mut StatisticsStruct)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    match Level {
        100 => {
            if !((*StatisticsStruct.Statistics.Level100).Buffer).is_null() {
                *Buffer = (*StatisticsStruct.Statistics.Level100).Buffer as LPBYTE;
            }
        }
        101 => {
            if !((*StatisticsStruct.Statistics.Level101).Buffer).is_null() {
                *Buffer = (*StatisticsStruct.Statistics.Level101).Buffer as LPBYTE;
            }
        }
        _ => {}
    }

    status
}

// ---- NetServerEnumEx ----
#[no_mangle]
pub unsafe extern "system" fn NetServerEnumEx(
    ServerName: LPCWSTR,
    Level: DWORD,
    Bufptr: *mut LPBYTE,
    PrefMaxlen: DWORD,
    EntriesRead: LPDWORD,
    totalentries: LPDWORD,
    servertype: DWORD,
    domain: LPCWSTR,
    FirstNameToReturn: LPCWSTR,
) -> DWORD {
    ERROR_NO_BROWSER_SERVERS_FOUND
}

// ---- NetServerEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetServerEnum(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    servertype: DWORD,
    domain: LPCWSTR,
    resume_handle: LPDWORD,
) -> DWORD {
    if !resume_handle.is_null() {
        *resume_handle = 0;
    }
    NetServerEnumEx(servername, level, bufptr, prefmaxlen, entriesread, totalentries, servertype, domain, core::ptr::null())
}
