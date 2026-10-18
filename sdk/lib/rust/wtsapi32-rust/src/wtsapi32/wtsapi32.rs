use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `#define WIN32_NO_STATUS` is a flag with no value, so it has no Rust constant to emit.
// `WINE_DEFAULT_DEBUG_CHANNEL(wtsapi);` only declares the debug channel for the crate's logging macros.
// `(!(3 as ULONG_PTR) as HANDLE)` is function-like, so it is skipped and inlined at its use sites as `((HANDLE)~(ULONG_PTR)3)`.

#[no_mangle]
pub unsafe extern "system" fn WTSCloseServer(hServer: HANDLE) {
    // FIXME("Stub %p\n", hServer);
}

#[no_mangle]
pub unsafe extern "system" fn WTSConnectSessionA(
    LogonId: ULONG,
    TargetLogonId: ULONG,
    pPassword: PSTR,
    bWait: BOOL,
) -> BOOL {
    // FIXME("Stub %d %d (%s) %d\n", LogonId, TargetLogonId, debugstr_a(pPassword), bWait);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSConnectSessionW(
    LogonId: ULONG,
    TargetLogonId: ULONG,
    pPassword: PWSTR,
    bWait: BOOL,
) -> BOOL {
    // FIXME("Stub %d %d (%s) %d\n", LogonId, TargetLogonId, debugstr_w(pPassword), bWait);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSDisconnectSession(
    hServer: HANDLE,
    SessionId_local: DWORD,
    bWait: BOOL,
) -> BOOL {
    // FIXME("Stub %p 0x%08x %d\n", hServer, SessionId_local, bWait);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnableChildSessions(enable: BOOL) -> BOOL {
    // FIXME("Stub %d\n", enable);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateProcessesExW(server: HANDLE, level: *mut DWORD, session_id: DWORD, info: *mut *mut WCHAR, count: *mut DWORD) -> BOOL {
    if !count.is_null() {
        *count = 0;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateProcessesExA(server: HANDLE, level: *mut DWORD, session_id: DWORD, info: *mut *mut i8, count: *mut DWORD) -> BOOL {
    if !count.is_null() {
        *count = 0;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateProcessesA(
    hServer: HANDLE,
    Reserved: DWORD,
    Version: DWORD,
    ppProcessInfo: *mut PWTS_PROCESS_INFOA,
    pCount: *mut DWORD,
) -> BOOL {
    if ppProcessInfo.is_null() || pCount.is_null() {
        return (FALSE) as BOOL;
    }

    *pCount = 0;
    *ppProcessInfo = core::ptr::null_mut();

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateProcessesW(
    mut hServer: HANDLE,
    mut Reserved: DWORD,
    mut Version: DWORD,
    mut ppProcessInfo: *mut PWTS_PROCESS_INFOW,
    mut pCount: *mut DWORD,
) -> BOOL {
    let mut processInfo: *mut WTS_PROCESS_INFOW = ::core::ptr::null_mut::<WTS_PROCESS_INFOW>();
    let mut spi: *mut SYSTEM_PROCESS_INFORMATION =
        ::core::ptr::null_mut::<SYSTEM_PROCESS_INFORMATION>();
    let mut size: ULONG = 0x4000 as ULONG;
    let mut buf: *mut ::core::ffi::c_void = NULL;
    let mut status: NTSTATUS = 0;
    let mut count: DWORD = 0;
    let mut name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    if ppProcessInfo.is_null()
        || pCount.is_null()
        || Reserved != 0 as DWORD
        || Version != 1 as DWORD
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }
    if hServer != (::core::ptr::null_mut::<::core::ffi::c_void>() as HANDLE) as HANDLE {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return FALSE as BOOL;
    }
    loop {
        size = size.wrapping_mul(2 as ULONG);
        HeapFree(GetProcessHeap(), 0 as DWORD, buf as PVOID);
        buf = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T)
            as *mut ::core::ffi::c_void;
        if buf.is_null() {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            return FALSE as BOOL;
        }
        status = NtQuerySystemInformation(
            SystemProcessInformation,
            buf as PVOID,
            size,
            ::core::ptr::null_mut::<ULONG>(),
        );
        if !(status == STATUS_INFO_LENGTH_MISMATCH as NTSTATUS) {
            break;
        }
    }
    if status != STATUS_SUCCESS as NTSTATUS {
        HeapFree(GetProcessHeap(), 0 as DWORD, buf as PVOID);
        SetLastError(RtlNtStatusToDosError(status) as DWORD);
        return FALSE as BOOL;
    }
    spi = buf as *mut SYSTEM_PROCESS_INFORMATION;
    size = 0 as ULONG;
    count = size as DWORD;
    loop {
        size = (size as ::core::ffi::c_ulonglong).wrapping_add(
            (::core::mem::size_of::<WTS_PROCESS_INFOW>() as usize)
                .wrapping_add((*spi).ImageName.Length as usize)
                .wrapping_add(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_ulonglong,
        ) as ULONG as ULONG;
        count = count.wrapping_add(1);
        if (*spi).NextEntryOffset == 0 as ULONG {
            break;
        }
        spi = (spi as PCHAR).offset((*spi).NextEntryOffset as isize)
            as *mut SYSTEM_PROCESS_INFORMATION;
    }
    processInfo = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T)
        as *mut WTS_PROCESS_INFOW;
    if processInfo.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, buf as PVOID);
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return FALSE as BOOL;
    }
    name = processInfo.offset(count as isize) as *mut WTS_PROCESS_INFOW as *mut WCHAR;
    *ppProcessInfo = processInfo as PWTS_PROCESS_INFOW;
    *pCount = count;
    spi = buf as *mut SYSTEM_PROCESS_INFORMATION;
    loop {
        let fresh15 = count;
        count = count.wrapping_sub(1);
        if !(fresh15 != 0) {
            break;
        }
        (*processInfo).SessionId = 0 as DWORD;
        (*processInfo).ProcessId =
            HandleToUlong!((*spi).UniqueProcessId as *const ::core::ffi::c_void) as DWORD;
        (*processInfo).pProcessName = name as LPWSTR;
        (*processInfo).pUserSid = NULL as PSID;
        memcpy(
            name as *mut ::core::ffi::c_void,
            (*spi).ImageName.Buffer as *const ::core::ffi::c_void,
            (*spi).ImageName.Length as size_t,
        );
        *name.offset(
            ((*spi).ImageName.Length as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as isize,
        ) = 0 as WCHAR;
        processInfo = processInfo.offset(1);
        name = name.offset(
            ((*spi).ImageName.Length as usize)
                .wrapping_add(::core::mem::size_of::<WCHAR>() as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as isize,
        );
        spi = (spi as PCHAR).offset((*spi).NextEntryOffset as isize)
            as *mut SYSTEM_PROCESS_INFORMATION;
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, buf as PVOID);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateServersA(
    pDomainName: LPSTR,
    Reserved: DWORD,
    Version: DWORD,
    ppServerInfo: *mut PWTS_SERVER_INFOA,
    pCount: *mut DWORD,
) -> BOOL {
    // FIXME("Stub %s 0x%08x 0x%08x %p %p\n", debugstr_a(pDomainName), Reserved, Version, ppServerInfo, pCount);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateServersW(
    pDomainName: LPWSTR,
    Reserved: DWORD,
    Version: DWORD,
    ppServerInfo: *mut PWTS_SERVER_INFOW,
    pCount: *mut DWORD,
) -> BOOL {
    // FIXME("Stub %s 0x%08x 0x%08x %p %p\n", debugstr_w(pDomainName), Reserved, Version, ppServerInfo, pCount);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateSessionsExW(
    server: HANDLE,
    level: *mut DWORD,
    filter: DWORD,
    info: *mut WTS_SESSION_INFO_1W,
    count: *mut DWORD,
) -> BOOL {
    DPRINT1!("Stub %p %p %d %p %p\n", server, level, filter, info, count);
    if !count.is_null() {
        *count = 0;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateSessionsExA(server: HANDLE, level: *mut DWORD, filter: DWORD, info: *mut WTS_SESSION_INFO_1A, count: *mut DWORD) -> BOOL {
    if !count.is_null() {
        *count = 0;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateSessionsA(hServer: HANDLE, Reserved: DWORD, Version: DWORD, ppSessionInfo: *mut PWTS_SESSION_INFOA, pCount: *mut DWORD) -> BOOL {
    static mut once: i32 = 0;

    let fresh16 = once;
    once = once + 1;
    if fresh16 == 0 {
        FIXME!("Stub %p 0x%08x 0x%08x %p %p\n", hServer, Reserved, Version, ppSessionInfo, pCount);
    }

    if ppSessionInfo.is_null() || pCount.is_null() {
        return (FALSE) as BOOL;
    }

    *pCount = 0;
    *ppSessionInfo = core::ptr::null_mut();

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSEnumerateSessionsW(
    hServer: HANDLE,
    Reserved: DWORD,
    Version: DWORD,
    ppSessionInfo: *mut PWTS_SESSION_INFOW,
    pCount: *mut DWORD,
) -> BOOL {
    if ppSessionInfo.is_null() || pCount.is_null() {
        return (FALSE) as BOOL;
    }

    *pCount = 0;
    *ppSessionInfo = core::ptr::null_mut();

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSFreeMemory(pMemory: PVOID) {
    heap_free(pMemory);
}

#[no_mangle]
pub unsafe extern "system" fn WTSFreeMemoryExA(
    mut type_0: WTS_TYPE_CLASS,
    mut ptr: *mut core::ffi::c_void,
    mut nmemb: ULONG,
) -> BOOL {
    heap_free(ptr);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSFreeMemoryExW(type_0: WTS_TYPE_CLASS, ptr: *mut core::ffi::c_void, nmemb: ULONG) -> BOOL {
    heap_free(ptr);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSLogoffSession(hserver: HANDLE, session_id: DWORD, bwait: BOOL) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WTSOpenServerExW(server_name: *mut WCHAR) -> HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn WTSOpenServerExA(mut server_name: LPSTR) -> HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn WTSOpenServerA(pServerName: LPSTR) -> HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn WTSOpenServerW(mut pServerName: LPWSTR) -> HANDLE {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wtsapi32/wtsapi32.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"WTSOpenServerW\0" as *const u8 as *const ::core::ffi::c_char,
            355 as ::core::ffi::c_int,
            b"(%s) stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pServerName as *const WCHAR),
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn WTSQuerySessionInformationA(
    hServer: HANDLE,
    SessionId_local: DWORD,
    WTSInfoClass: WTS_INFO_CLASS,
    Buffer: *mut LPSTR,
    BytesReturned: *mut DWORD,
) -> BOOL {
    let wcsErrorCode: size_t = -(1 as ::core::ffi::c_int) as size_t;
    let mut buffer: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ansiBuffer: LPSTR = ::core::ptr::null_mut::<CHAR>();
    let mut bytesReturned: DWORD = 0 as DWORD;
    let mut result: BOOL = FALSE as BOOL;
    let mut len: size_t = 0;

    if BytesReturned.is_null() || Buffer.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return FALSE as BOOL;
    }

    if WTSQuerySessionInformationW(
        hServer,
        SessionId_local,
        WTSInfoClass,
        core::ptr::addr_of_mut!(buffer),
        core::ptr::addr_of_mut!(bytesReturned),
    ) == 0
    {
        ansiBuffer = buffer as LPSTR;
        *Buffer = ansiBuffer;
        *BytesReturned = bytesReturned;
        return FALSE as BOOL;
    }

    match WTSInfoClass as ::core::ffi::c_uint {
        WTSInitialProgram
        | WTSApplicationName
        | WTSWorkingDirectory
        | WTSOEMId
        | WTSUserName
        | WTSWinStationName
        | WTSDomainName
        | WTSClientName
        | WTSClientDirectory => {
            len = wcstombs(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                buffer as *const WCHAR,
                0 as size_t,
            );
            if len != wcsErrorCode {
                len = len.wrapping_add(1);
                ansiBuffer = heap_alloc_zero((len as SIZE_T) as _) as LPSTR;
                if !ansiBuffer.is_null()
                    && wcstombs(
                        ansiBuffer as *mut ::core::ffi::c_char,
                        buffer as *const WCHAR,
                        len,
                    ) != wcsErrorCode
                {
                    result = TRUE as BOOL;
                    bytesReturned = len as DWORD;
                }
            }
            WTSFreeMemory(buffer as PVOID);
        }

        _ => {
            result = TRUE as BOOL;
            ansiBuffer = buffer as LPSTR;
        }
    }

    *Buffer = ansiBuffer;
    *BytesReturned = bytesReturned;

    return result;
}
#[no_mangle]
pub unsafe extern "system" fn WTSQuerySessionInformationW(
    mut hServer: HANDLE,
    mut SessionId_local: DWORD,
    mut WTSInfoClass: WTS_INFO_CLASS,
    mut Buffer: *mut LPWSTR,
    mut BytesReturned: *mut DWORD,
) -> BOOL {
    {
        if BytesReturned.is_null() || Buffer.is_null() {
            SetLastError(ERROR_INVALID_USER_BUFFER);
            return (FALSE) as BOOL;
        }

        if (WTSInfoClass as ::core::ffi::c_uint) > (WTSClientProtocolType as ::core::ffi::c_uint) {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }

        match WTSInfoClass as ::core::ffi::c_uint {
            WTSSessionId => {
                let size: DWORD = ::core::mem::size_of::<ULONG>() as DWORD;
                let mut output: *mut ULONG =
                    heap_alloc_zero((size as SIZE_T) as _) as *mut ULONG;
                if output.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    return (FALSE) as BOOL;
                }

                *output = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).SessionId;
                *Buffer = output as LPWSTR;
                *BytesReturned = size;
                return (TRUE) as BOOL;
            }

            WTSUserName => {
                let mut username: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
                let mut count: DWORD = 0;

                GetUserNameW(
                    ::core::ptr::null_mut::<WCHAR>(),
                    ::core::ptr::addr_of_mut!(count),
                );
                if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                    return (FALSE) as BOOL;
                }
                username = heap_alloc(
                    ((count as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
                ) as *mut WCHAR;
                if username.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    return (FALSE) as BOOL;
                }

                GetUserNameW(username as LPWSTR, ::core::ptr::addr_of_mut!(count));
                *Buffer = username as LPWSTR;
                *BytesReturned =
                    (count as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD;
                return TRUE as BOOL;
            }

            WTSConnectState => {
                let size_0: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;
                let mut output_0: *mut WCHAR =
                    heap_alloc_zero((size_0 as SIZE_T) as _) as *mut WCHAR;
                if output_0.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    return FALSE as BOOL;
                }

                *Buffer = output_0 as LPWSTR;
                *BytesReturned = size_0;
                return TRUE as BOOL;
            }

            WTSClientProtocolType => {
                let size_1: DWORD = ::core::mem::size_of::<WORD>() as DWORD;
                let mut output_1: *mut WCHAR =
                    heap_alloc_zero((size_1 as SIZE_T) as _) as *mut WCHAR;
                if output_1.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    return FALSE as BOOL;
                }

                *Buffer = output_1 as LPWSTR;
                *BytesReturned = size_1;
                return TRUE as BOOL;
            }

            _ => {
                if !BytesReturned.is_null() {
                    *BytesReturned = 0;
                }
            }
        }

        /* FIXME: Forward request to winsta.dll::WinStationQueryInformationW */

        return FALSE as BOOL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn WTSQueryUserToken(session_id: ULONG, token: PHANDLE) -> BOOL {
    if token.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    return DuplicateHandle(GetCurrentProcess(), (!(3 as ULONG_PTR) as HANDLE),
                           GetCurrentProcess(), token,
                           0, (FALSE) as BOOL, DUPLICATE_SAME_ACCESS);
}

#[no_mangle]
pub unsafe extern "system" fn WTSQueryUserConfigA(
    pServerName: LPSTR,
    pUserName: LPSTR,
    WTSConfigClass: WTS_CONFIG_CLASS,
    ppBuffer: *mut LPSTR,
    pBytesReturned: *mut DWORD,
) -> BOOL {
    // FIXME("Stub (%s) (%s) 0x%08x %p %p\n", debugstr_a(pServerName), debugstr_a(pUserName), WTSConfigClass, ppBuffer, pBytesReturned);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSQueryUserConfigW(
    pServerName: LPWSTR,
    pUserName: LPWSTR,
    WTSConfigClass: WTS_CONFIG_CLASS,
    ppBuffer: *mut LPWSTR,
    pBytesReturned: *mut DWORD,
) -> BOOL {
    // FIXME("Stub (%s) (%s) 0x%08x %p %p\n", debugstr_w(pServerName), debugstr_w(pUserName), WTSConfigClass, ppBuffer, pBytesReturned);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSRegisterSessionNotification(
    hWnd: HWND,
    dwFlags: DWORD,
) -> BOOL {
    // FIXME("Stub %p 0x%08x\n", hWnd, dwFlags);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSRegisterSessionNotificationEx(
    hServer: HANDLE,
    hWnd: HWND,
    dwFlags: DWORD,
) -> BOOL {
    // FIXME("Stub %p %p 0x%08x\n", hServer, hWnd, dwFlags);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSSendMessageA(
    hServer: HANDLE,
    SessionId_local: DWORD,
    pTitle: LPSTR,
    TitleLength: DWORD,
    pMessage: LPSTR,
    MessageLength: DWORD,
    Style: DWORD,
    Timeout: DWORD,
    pResponse: *mut DWORD,
    bWait: BOOL,
) -> BOOL {
    // FIXME("Stub %p 0x%08x (%s) %d (%s) %d 0x%08x %d %p %d\n", hServer, SessionId_local, debugstr_a(pTitle), TitleLength, debugstr_a(pMessage), MessageLength, Style, Timeout, pResponse, bWait);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSSendMessageW(
    hServer: HANDLE,
    SessionId_local: DWORD,
    pTitle: LPWSTR,
    TitleLength: DWORD,
    pMessage: LPWSTR,
    MessageLength: DWORD,
    Style: DWORD,
    Timeout: DWORD,
    pResponse: *mut DWORD,
    bWait: BOOL,
) -> BOOL {
    // FIXME("Stub %p 0x%08x (%s) %d (%s) %d 0x%08x %d %p %d\n", hServer, SessionId_local, debugstr_w(pTitle), TitleLength, debugstr_w(pMessage), MessageLength, Style, Timeout, pResponse, bWait);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSSetUserConfigA(
    pServerName: LPSTR,
    pUserName: LPSTR,
    WTSConfigClass: WTS_CONFIG_CLASS,
    pBuffer: LPSTR,
    DataLength: DWORD,
) -> BOOL {
    // FIXME("Stub (%s) (%s) 0x%08x %p %d\n", debugstr_a(pServerName), debugstr_a(pUserName), WTSConfigClass,pBuffer, DataLength);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSSetUserConfigW(
    pServerName: LPWSTR,
    pUserName: LPWSTR,
    WTSConfigClass: WTS_CONFIG_CLASS,
    pBuffer: LPWSTR,
    DataLength: DWORD,
) -> BOOL {
    // FIXME("Stub (%s) (%s) 0x%08x %p %d\n", debugstr_w(pServerName), debugstr_w(pUserName), WTSConfigClass,pBuffer, DataLength);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSShutdownSystem(hServer: HANDLE, ShutdownFlag: DWORD) -> BOOL {
    // FIXME("Stub %p 0x%08x\n", hServer,ShutdownFlag);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSStartRemoteControlSessionA(
    pTargetServerName: LPSTR,
    TargetLogonId: ULONG,
    HotkeyVk: BYTE,
    HotkeyModifiers: USHORT,
) -> BOOL {
    // FIXME("Stub (%s) %d %d %d\n", debugstr_a(pTargetServerName), TargetLogonId, HotkeyVk, HotkeyModifiers);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSStartRemoteControlSessionW(
    pTargetServerName: LPWSTR,
    TargetLogonId: ULONG,
    HotkeyVk: BYTE,
    HotkeyModifiers: USHORT,
) -> BOOL {
    // FIXME("Stub (%s) %d %d %d\n", debugstr_w(pTargetServerName), TargetLogonId, HotkeyVk, HotkeyModifiers);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSStopRemoteControlSession(LogonId: ULONG) -> BOOL {
    // FIXME("Stub %d\n", LogonId);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSTerminateProcess(
    hServer: HANDLE,
    ProcessId: DWORD,
    ExitCode: DWORD,
) -> BOOL {
    // FIXME("Stub %p %d %d\n", hServer, ProcessId, ExitCode);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSUnRegisterSessionNotification(hWnd: HWND) -> BOOL {
    // FIXME("Stub %p\n", hWnd);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSUnRegisterSessionNotificationEx(
    hServer: HANDLE,
    hWnd: HWND,
) -> BOOL {
    // FIXME("Stub %p %p\n", hServer, hWnd);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelClose(hChannelHandle: HANDLE) -> BOOL {
    // FIXME("Stub %p\n", hChannelHandle);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelOpen(
    hServer: HANDLE,
    SessionId_local: DWORD,
    pVirtualName: LPSTR,
) -> HANDLE {
    // FIXME("Stub %p %d (%s)\n", hServer, SessionId_local, debugstr_a(pVirtualName));
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelOpenEx(
    SessionId_local: DWORD,
    pVirtualName: LPSTR,
    flags: DWORD,
) -> HANDLE {
    // FIXME("Stub %d (%s) %d\n", SessionId, debugstr_a(pVirtualName), flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelPurgeInput(hChannelHandle: HANDLE) -> BOOL {
    // FIXME("Stub %p\n", hChannelHandle);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelPurgeOutput(hChannelHandle: HANDLE) -> BOOL {
    // FIXME("Stub %p\n", hChannelHandle);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelQuery(
    hChannelHandle: HANDLE,
    WtsVirtualClass: WTS_VIRTUAL_CLASS,
    ppBuffer: *mut PVOID,
    pBytesReturned: *mut DWORD,
) -> BOOL {
    // FIXME("Stub %p %d %p %p\n", hChannelHandle, WtsVirtualClass, ppBuffer, pBytesReturned);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelRead(
    hChannelHandle: HANDLE,
    TimeOut: ULONG,
    Buffer: PCHAR,
    BufferSize: ULONG,
    pBytesRead: PULONG,
) -> BOOL {
    // FIXME("Stub %p %d %p %d %p\n", hChannelHandle, TimeOut, Buffer, BufferSize, pBytesRead);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSVirtualChannelWrite(
    hChannelHandle: HANDLE,
    Buffer: PCHAR,
    Length: ULONG,
    pBytesWritten: PULONG,
) -> BOOL {
    // FIXME("Stub %p %p %d %p\n", hChannelHandle, Buffer, Length, pBytesWritten);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn WTSWaitSystemEvent(
    hServer: HANDLE,
    Mask: DWORD,
    Flags: *mut DWORD,
) -> BOOL {
    // FIXME("Stub %p 0x%08x %p\n", hServer, Mask, Flags);
    (FALSE as i32)
}
