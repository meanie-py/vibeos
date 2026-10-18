use crate::winspool::marshalling::MarshallUpStructuresArray;
use ntdll_sys::seh::rpc_try;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetLastError() -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn SetLastError(dwErrCode: DWORD);
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut BOOL) -> i32;
}

#[no_mangle]
pub unsafe extern "system" fn AddMonitorA(pName: PSTR, Level: DWORD, pMonitors: PBYTE) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut len: INT;
    let mut res: BOOL;
    let mut mi2a: LPMONITOR_INFO_2A;
    let mut mi2w: MONITOR_INFO_2W = core::mem::zeroed();

    mi2a = pMonitors as LPMONITOR_INFO_2A;

    if Level != 2 {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    /* XP: unchanged, win9x: ERROR_INVALID_ENVIRONMENT */
    if mi2a.is_null() {
        return FALSE as BOOL;
    }

    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
        nameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len);
    }

    memset(core::ptr::addr_of_mut!(mi2w) as LPVOID, 0, core::mem::size_of::<MONITOR_INFO_2W>());
    if !(*mi2a).pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*mi2a).pName, -1, core::ptr::null_mut(), 0);
        mi2w.pName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*mi2a).pName, -1, mi2w.pName, len);
    }
    if !(*mi2a).pEnvironment.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*mi2a).pEnvironment, -1, core::ptr::null_mut(), 0);
        mi2w.pEnvironment = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*mi2a).pEnvironment, -1, mi2w.pEnvironment, len);
    }
    if !(*mi2a).pDLLName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*mi2a).pDLLName, -1, core::ptr::null_mut(), 0);
        mi2w.pDLLName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*mi2a).pDLLName, -1, mi2w.pDLLName, len);
    }

    res = AddMonitorW(nameW, Level, core::ptr::addr_of_mut!(mi2w) as LPBYTE);

    if !mi2w.pName.is_null() {
        HeapFree(GetProcessHeap(), 0, mi2w.pName as LPVOID);
    }
    if !mi2w.pEnvironment.is_null() {
        HeapFree(GetProcessHeap(), 0, mi2w.pEnvironment as LPVOID);
    }
    if !mi2w.pDLLName.is_null() {
        HeapFree(GetProcessHeap(), 0, mi2w.pDLLName as LPVOID);
    }
    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as LPVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn AddMonitorW(pName: PWSTR, Level: DWORD, pMonitors: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut MonitorInfoContainer: WINSPOOL_MONITOR_CONTAINER = core::mem::zeroed();

    if Level != 2 {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    (*core::ptr::addr_of_mut!(MonitorInfoContainer)).MonitorInfo.pMonitorInfo2 = pMonitors as *mut WINSPOOL_MONITOR_INFO_2;
    (*core::ptr::addr_of_mut!(MonitorInfoContainer)).Level = Level;

    // Do the RPC call
    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcAddMonitor(pName, core::ptr::addr_of_mut!(MonitorInfoContainer))).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcExcept handler: dead under this tree's USE_DUMMY_PSEH build (the C handler never runs either), and stable Rust has no SEH. Original: dwErrorCode = RpcExceptionCode();
    // RpcEndExcept;

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteMonitorA(pName: PSTR, pEnvironment: PSTR, pMonitorName: PSTR) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut EnvironmentW: LPWSTR = core::ptr::null_mut();
    let mut MonitorNameW: LPWSTR = core::ptr::null_mut();
    let mut res: BOOL;
    let mut len: INT;

    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
        nameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len);
    }

    if !pEnvironment.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pEnvironment, -1, core::ptr::null_mut(), 0);
        EnvironmentW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pEnvironment, -1, EnvironmentW, len);
    }
    if !pMonitorName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pMonitorName, -1, core::ptr::null_mut(), 0);
        MonitorNameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pMonitorName, -1, MonitorNameW, len);
    }

    res = DeleteMonitorW(nameW, EnvironmentW, MonitorNameW);

    if !MonitorNameW.is_null() { HeapFree(GetProcessHeap(), 0, MonitorNameW as LPVOID); }
    if !EnvironmentW.is_null() { HeapFree(GetProcessHeap(), 0, EnvironmentW as LPVOID); }
    if !nameW.is_null() { HeapFree(GetProcessHeap(), 0, nameW as LPVOID); }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteMonitorW(pName: PWSTR, pEnvironment: PWSTR, pMonitorName: PWSTR) -> BOOL {
    let mut dwErrorCode: DWORD;

    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcDeleteMonitor(pName, pEnvironment, pMonitorName)).unwrap_or_else(|code| code as _);


    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumMonitorsA(pName: PSTR, Level: DWORD, pMonitors: PBYTE, mut cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> WINBOOL {
    let mut res: BOOL;
    let mut bufferW: LPBYTE = core::ptr::null_mut();
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut numentries: DWORD = 0;
    let mut len: INT = 0;

    if Level < 1 || Level > 2
    {
        SetLastError( ERROR_INVALID_LEVEL );
        return FALSE as WINBOOL;
    }

    /* convert servername to unicode */
    if !pName.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
        nameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len);
    }
    /* alloc (userbuffersize*sizeof(WCHAR) and try to enum the monitors */
    needed = cbBuf.wrapping_mul(core::mem::size_of::<u16>() as DWORD);
    if needed != 0 {
        bufferW = HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as LPBYTE;
    }
    res = EnumMonitorsW(nameW, Level, bufferW, needed, pcbNeeded, pcReturned);

    if res == 0 && (GetLastError() as DWORD == ERROR_INSUFFICIENT_BUFFER)
    {
        if !pcbNeeded.is_null() {
            needed = *pcbNeeded;
        }
        /* HeapReAlloc return NULL, when bufferW was NULL */
        bufferW = if !bufferW.is_null() {
            HeapReAlloc(GetProcessHeap(), 0, bufferW as LPVOID, needed as SIZE_T) as LPBYTE
        } else {
            HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as LPBYTE
        };

        /* Try again with the large Buffer */
        res = EnumMonitorsW(nameW, Level, bufferW, needed, pcbNeeded, pcReturned);
    }
    numentries = if !pcReturned.is_null() { *pcReturned } else { 0 };
    needed = 0;
    /*
       W2k require the buffersize from EnumMonitorsW also for EnumMonitorsA.
       We use the smaller Ansi-Size to avoid conflicts with fixed Buffers of old Apps.
     */
    'emA_cleanup: {
        if res != 0
        {
            /* EnumMonitorsW collected all Data. Parse them to calculate ANSI-Size */
            let mut entrysize: DWORD = 0;
            let mut index: DWORD;
            let mut ptr: LPSTR;
            let mut mi2w: *mut MONITOR_INFO_2W;
            let mut mi2a: *mut MONITOR_INFO_2A;

            /* MONITOR_INFO_*W and MONITOR_INFO_*A have the same size */
            entrysize = if Level == 1 {
                core::mem::size_of::<MONITOR_INFO_1A>() as DWORD
            } else {
                core::mem::size_of::<MONITOR_INFO_2A>() as DWORD
            };

            /* First pass: calculate the size for all Entries */
            mi2w = bufferW as *mut MONITOR_INFO_2W;
            mi2a = pMonitors as *mut MONITOR_INFO_2A;
            index = 0;
            while index < numentries
            {
                index += 1;
                needed = needed.wrapping_add(entrysize);    /* MONITOR_INFO_?A */

                needed = needed.wrapping_add(WideCharToMultiByte(CP_ACP, 0, (*mi2w).pName, -1,
                                                core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD);
                if Level > 1
                {
                    needed = needed.wrapping_add(WideCharToMultiByte(CP_ACP, 0, (*mi2w).pEnvironment, -1,
                                                    core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD);
                    needed = needed.wrapping_add(WideCharToMultiByte(CP_ACP, 0, (*mi2w).pDLLName, -1,
                                                    core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD);
                }
                /* use LPBYTE with entrysize to avoid double code (MONITOR_INFO_1 + MONITOR_INFO_2) */
                mi2w = ((mi2w as LPBYTE).add(entrysize as usize)) as *mut MONITOR_INFO_2W;
                mi2a = ((mi2a as LPBYTE).add(entrysize as usize)) as *mut MONITOR_INFO_2A;
            }

            /* check for errors and quit on failure */
            if cbBuf < needed
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                res = FALSE as BOOL;
                break 'emA_cleanup;
            }
            len = entrysize.wrapping_mul(numentries) as INT;       /* room for all MONITOR_INFO_?A */
            ptr = pMonitors.add(len as usize) as LPSTR;            /* room for strings */
            cbBuf = cbBuf.wrapping_sub(len as DWORD);              /* free Bytes in the user-Buffer */
            mi2w = bufferW as *mut MONITOR_INFO_2W;
            mi2a = pMonitors as *mut MONITOR_INFO_2A;
            index = 0;
            /* Second Pass: Fill the User Buffer (if we have one) */
            while index < numentries && !pMonitors.is_null()
            {
                index += 1;

                (*mi2a).pName = ptr;
                len = WideCharToMultiByte(CP_ACP, 0, (*mi2w).pName, -1,
                                                ptr, cbBuf as i32, core::ptr::null(), core::ptr::null_mut());
                ptr = ptr.offset(len as isize);
                cbBuf = cbBuf.wrapping_sub(len as DWORD);
                if Level > 1
                {
                    (*mi2a).pEnvironment = ptr;
                    len = WideCharToMultiByte(CP_ACP, 0, (*mi2w).pEnvironment, -1,
                                                ptr, cbBuf as i32, core::ptr::null(), core::ptr::null_mut());
                    ptr = ptr.offset(len as isize);
                    cbBuf = cbBuf.wrapping_sub(len as DWORD);

                    (*mi2a).pDLLName = ptr;
                    len = WideCharToMultiByte(CP_ACP, 0, (*mi2w).pDLLName, -1,
                                                ptr, cbBuf as i32, core::ptr::null(), core::ptr::null_mut());
                    ptr = ptr.offset(len as isize);
                    cbBuf = cbBuf.wrapping_sub(len as DWORD);
                }
                /* use LPBYTE with entrysize to avoid double code (MONITOR_INFO_1 + MONITOR_INFO_2) */
                mi2w = ((mi2w as LPBYTE).add(entrysize as usize)) as *mut MONITOR_INFO_2W;
                mi2a = ((mi2a as LPBYTE).add(entrysize as usize)) as *mut MONITOR_INFO_2A;
            }
        }
    }
    if !pcbNeeded.is_null() {
        *pcbNeeded = needed;
    }
    if !pcReturned.is_null() {
        *pcReturned = if res != 0 { numentries } else { 0 };
    }

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as LPVOID);
    }
    if !bufferW.is_null() {
        HeapFree(GetProcessHeap(), 0, bufferW as LPVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn EnumMonitorsW(pName: PWSTR, Level: DWORD, pMonitors: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;

    if Level < 1 || Level > 2
    {
        SetLastError( ERROR_INVALID_LEVEL );
        return FALSE as WINBOOL;
    }

    // Do the RPC call
    // RpcTryExcept
    {
        dwErrorCode = rpc_try(|| _RpcEnumMonitors(pName, Level, pMonitors, cbBuf, pcbNeeded, pcReturned)).unwrap_or_else(|code| code as _);
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler: reached only on a structured exception,
    // which plain Rust cannot model here; kept literally but never taken on this path.
    #[allow(unreachable_code)]
    // RpcEndExcept;

    if dwErrorCode == ERROR_SUCCESS
    {
        // Replace relative offset addresses in the output by absolute pointers.
        MarshallUpStructuresArray(cbBuf, pMonitors as PVOID, *pcReturned, core::ptr::addr_of!((*pMonitorInfoMarshalling[Level as usize]).pInfo) as *const MARSHALLING_INFO, (*pMonitorInfoMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
