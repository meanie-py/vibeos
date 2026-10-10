use crate::sc::misc::ParseCreateConfigArguments;
use crate::sc::sc::ReportLastError;
use crate::sc::usage::SetConfigUsage;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn QueryConfig(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut cbBytesNeeded: DWORD = 0;
    let mut pServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut lpPtr: LPWSTR = core::ptr::null_mut();
    let mut nLen: SSIZE_T = 0;
    let mut i: SSIZE_T = 0;

    if false {
        _tprintf(
            ['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 'u' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
            ServiceName,
        );
    }

    'done: {
        hManager = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            _tprintf(
                ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, SERVICE_QUERY_CONFIG);
        if hService.is_null() {
            _tprintf(
                ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig(
            hService,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(cbBytesNeeded),
        ) == 0
        {
            if cbBytesNeeded == 0 {
                _tprintf(
                    ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
                    GetLastError(),
                );
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pServiceConfig = HeapAlloc(
            GetProcessHeap(),
            0,
            cbBytesNeeded as SIZE_T,
        ) as LPQUERY_SERVICE_CONFIG;
        if pServiceConfig.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf(
                ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'H' as u16, 'e' as u16, 'a' as u16, 'p' as u16, 'A' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'c' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig(
            hService,
            pServiceConfig,
            cbBytesNeeded,
            core::ptr::addr_of_mut!(cbBytesNeeded),
        ) == 0
        {
            _tprintf(
                ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf(
            ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, 'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr(),
        );

        _tprintf(
            ['S' as u16, 'E' as u16, 'R' as u16, 'V' as u16, 'I' as u16, 'C' as u16, 'E' as u16, '_' as u16, 'N' as u16, 'A' as u16, 'M' as u16, 'E' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16].as_ptr(),
            ServiceName,
        );
        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'T' as u16, 'Y' as u16, 'P' as u16, 'E' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, '-' as u16, '3' as u16, 'l' as u16, 'x' as u16, ' ' as u16, 0u16].as_ptr(),
            (*pServiceConfig).dwServiceType,
        );
        match (*pServiceConfig).dwServiceType {
            SERVICE_KERNEL_DRIVER => {
                _tprintf(
                    ['K' as u16, 'E' as u16, 'R' as u16, 'N' as u16, 'E' as u16, 'L' as u16, '_' as u16, 'D' as u16, 'R' as u16, 'I' as u16, 'V' as u16, 'E' as u16, 'R' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_FILE_SYSTEM_DRIVER => {
                _tprintf(
                    ['F' as u16, 'I' as u16, 'L' as u16, 'E' as u16, '_' as u16, 'S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16, '_' as u16, 'D' as u16, 'R' as u16, 'I' as u16, 'V' as u16, 'E' as u16, 'R' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_WIN32_OWN_PROCESS => {
                _tprintf(
                    ['W' as u16, 'I' as u16, 'N' as u16, '3' as u16, '2' as u16, '_' as u16, 'O' as u16, 'W' as u16, 'N' as u16, '_' as u16, 'P' as u16, 'R' as u16, 'O' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_WIN32_SHARE_PROCESS => {
                _tprintf(
                    ['W' as u16, 'I' as u16, 'N' as u16, '3' as u16, '2' as u16, '_' as u16, 'S' as u16, 'H' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '_' as u16, 'P' as u16, 'R' as u16, 'O' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            _ if (*pServiceConfig).dwServiceType == SERVICE_WIN32_OWN_PROCESS + SERVICE_INTERACTIVE_PROCESS => {
                _tprintf(
                    ['W' as u16, 'I' as u16, 'N' as u16, '3' as u16, '2' as u16, '_' as u16, 'O' as u16, 'W' as u16, 'N' as u16, '_' as u16, 'P' as u16, 'R' as u16, 'O' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, ' ' as u16, '(' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ')' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            _ if (*pServiceConfig).dwServiceType == SERVICE_WIN32_SHARE_PROCESS + SERVICE_INTERACTIVE_PROCESS => {
                _tprintf(
                    ['W' as u16, 'I' as u16, 'N' as u16, '3' as u16, '2' as u16, '_' as u16, 'S' as u16, 'H' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '_' as u16, 'P' as u16, 'R' as u16, 'O' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, ' ' as u16, '(' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ')' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            _ => {
                _tprintf(['\n' as u16, 0u16].as_ptr());
            }
        }

        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '_' as u16, 'T' as u16, 'Y' as u16, 'P' as u16, 'E' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, '-' as u16, '3' as u16, 'l' as u16, 'x' as u16, ' ' as u16, 0u16].as_ptr(),
            (*pServiceConfig).dwStartType,
        );
        match (*pServiceConfig).dwStartType {
            SERVICE_BOOT_START => {
                _tprintf(
                    ['B' as u16, 'O' as u16, 'O' as u16, 'T' as u16, '_' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_SYSTEM_START => {
                _tprintf(
                    ['S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16, '_' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_AUTO_START => {
                _tprintf(
                    ['A' as u16, 'U' as u16, 'T' as u16, 'O' as u16, '_' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_DEMAND_START => {
                _tprintf(
                    ['D' as u16, 'E' as u16, 'M' as u16, 'A' as u16, 'N' as u16, 'D' as u16, '_' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_DISABLED => {
                _tprintf(
                    ['D' as u16, 'I' as u16, 'S' as u16, 'A' as u16, 'B' as u16, 'L' as u16, 'E' as u16, 'D' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            _ => {
                _tprintf(['\n' as u16, 0u16].as_ptr());
            }
        }

        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, '_' as u16, 'C' as u16, 'O' as u16, 'N' as u16, 'T' as u16, 'R' as u16, 'O' as u16, 'L' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, '-' as u16, '3' as u16, 'l' as u16, 'x' as u16, ' ' as u16, 0u16].as_ptr(),
            (*pServiceConfig).dwErrorControl,
        );
        match (*pServiceConfig).dwErrorControl {
            SERVICE_ERROR_IGNORE => {
                _tprintf(
                    ['I' as u16, 'G' as u16, 'N' as u16, 'O' as u16, 'R' as u16, 'E' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_ERROR_NORMAL => {
                _tprintf(
                    ['N' as u16, 'O' as u16, 'R' as u16, 'M' as u16, 'A' as u16, 'L' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_ERROR_SEVERE => {
                _tprintf(
                    ['S' as u16, 'E' as u16, 'V' as u16, 'E' as u16, 'R' as u16, 'E' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            SERVICE_ERROR_CRITICAL => {
                _tprintf(
                    ['C' as u16, 'R' as u16, 'I' as u16, 'T' as u16, 'I' as u16, 'C' as u16, 'A' as u16, 'L' as u16, '\n' as u16, 0u16].as_ptr(),
                );
            }
            _ => {
                _tprintf(['\n' as u16, 0u16].as_ptr());
            }
        }

        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'B' as u16, 'I' as u16, 'N' as u16, 'A' as u16, 'R' as u16, 'Y' as u16, '_' as u16, 'P' as u16, 'A' as u16, 'T' as u16, 'H' as u16, '_' as u16, 'N' as u16, 'A' as u16, 'M' as u16, 'E' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16].as_ptr(),
            (*pServiceConfig).lpBinaryPathName,
        );
        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'L' as u16, 'O' as u16, 'A' as u16, 'D' as u16, '_' as u16, 'O' as u16, 'R' as u16, 'D' as u16, 'E' as u16, 'R' as u16, '_' as u16, 'G' as u16, 'R' as u16, 'O' as u16, 'U' as u16, 'P' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16].as_ptr(),
            (*pServiceConfig).lpLoadOrderGroup,
        );
        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'T' as u16, 'A' as u16, 'G' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0u16].as_ptr(),
            (*pServiceConfig).dwTagId,
        );
        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'D' as u16, 'I' as u16, 'S' as u16, 'P' as u16, 'L' as u16, 'A' as u16, 'Y' as u16, '_' as u16, 'N' as u16, 'A' as u16, 'M' as u16, 'E' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16].as_ptr(),
            (*pServiceConfig).lpDisplayName,
        );
        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'D' as u16, 'E' as u16, 'P' as u16, 'E' as u16, 'N' as u16, 'D' as u16, 'E' as u16, 'N' as u16, 'C' as u16, 'I' as u16, 'E' as u16, 'S' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 0u16].as_ptr(),
        );
        lpPtr = (*pServiceConfig).lpDependencies;
        i = 0 as SSIZE_T;
        while *lpPtr != 0u16 {
            nLen = _tcslen(lpPtr) as SSIZE_T;
            if i != 0 as SSIZE_T {
                _tprintf(
                    ['\n' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 0u16].as_ptr(),
                );
            }
            _tprintf(['%' as u16, 's' as u16, 0u16].as_ptr(), lpPtr);
            lpPtr = lpPtr.offset(nLen as isize + 1 as isize);
            i += 1;
        }
        _tprintf(['\n' as u16, 0u16].as_ptr());

        _tprintf(
            [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'S' as u16, 'E' as u16, 'R' as u16, 'V' as u16, 'I' as u16, 'C' as u16, 'E' as u16, '_' as u16, 'S' as u16, 'T' as u16, 'A' as u16, 'R' as u16, 'T' as u16, '_' as u16, 'N' as u16, 'A' as u16, 'M' as u16, 'E' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16].as_ptr(),
            (*pServiceConfig).lpServiceStartName,
        );
    }

    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pServiceConfig.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pServiceConfig as PVOID,
        );
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn SetConfig(ServiceArgs: *mut LPCTSTR, ArgCount: INT) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut ServiceInfo: SERVICE_CREATE_INFO = core::mem::zeroed::<SERVICE_CREATE_INFO>();

    if ParseCreateConfigArguments(ServiceArgs, ArgCount, (TRUE) as BOOL, core::ptr::addr_of_mut!(ServiceInfo)) == 0 {
        SetConfigUsage();
        return (FALSE) as BOOL;
    }

    // SCDBG debug block (compiled out in C; kept as dead code so tokens stay live)
    if false {
        _tprintf((['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpServiceName);
        _tprintf((['d' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpDisplayName);
        _tprintf((['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.dwServiceType);
        _tprintf((['s' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.dwStartType);
        _tprintf((['e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.dwErrorControl);
        _tprintf((['B' as u16, 'i' as u16, 'n' as u16, 'a' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'p' as u16, 'a' as u16, 't' as u16, 'h' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpBinaryPathName);
        _tprintf((['l' as u16, 'o' as u16, 'a' as u16, 'd' as u16, ' ' as u16, 'o' as u16, 'r' as u16, 'd' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpLoadOrderGroup);
        _tprintf((['t' as u16, 'a' as u16, 'g' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.dwTagId);
        _tprintf((['d' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 'c' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpDependencies);
        _tprintf((['a' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpServiceStartName);
        _tprintf((['a' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, ' ' as u16, 'p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0].as_ptr()), ServiceInfo.lpPassword);
    }

    'done: {
        hManager = OpenSCManager(core::ptr::null_mut(),
                                 core::ptr::null_mut(),
                                 SC_MANAGER_CONNECT);
        if hManager.is_null() {
            _tprintf((['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0].as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager,
                               ServiceInfo.lpServiceName,
                               SERVICE_CHANGE_CONFIG);
        if hService.is_null() {
            _tprintf((['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0].as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if ChangeServiceConfig(hService,
                                 ServiceInfo.dwServiceType,
                                 ServiceInfo.dwStartType,
                                 ServiceInfo.dwErrorControl,
                                 ServiceInfo.lpBinaryPathName,
                                 ServiceInfo.lpLoadOrderGroup,
                                 if ServiceInfo.bTagId != 0 {
                                     core::ptr::addr_of_mut!(ServiceInfo.dwTagId)
                                 } else {
                                     core::ptr::null_mut()
                                 },
                                 ServiceInfo.lpDependencies,
                                 ServiceInfo.lpServiceStartName,
                                 ServiceInfo.lpPassword,
                                 ServiceInfo.lpDisplayName) == 0
        {
            _tprintf((['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0].as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf((['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, 'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, '\n' as u16, 0].as_ptr()));
    }

    // done:
    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
