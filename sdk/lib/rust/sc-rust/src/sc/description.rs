use crate::sc::sc::ReportLastError;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn QueryDescription(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut cbBytesNeeded: DWORD = 0;
    let mut pServiceDescription: LPSERVICE_DESCRIPTION = core::ptr::null_mut();

    let fmt_scdbg = ['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_open_scm = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_open_svc = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_qsc2 = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, '2' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_heap = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'H' as u16, 'e' as u16, 'a' as u16, 'p' as u16, 'A' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'c' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_success = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, '2' as u16, ' ' as u16, 'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, '\n' as u16, 0];
    let fmt_svc_name = ['S' as u16, 'E' as u16, 'R' as u16, 'V' as u16, 'I' as u16, 'C' as u16, 'E' as u16, '_' as u16, 'N' as u16, 'A' as u16, 'M' as u16, 'E' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
    let fmt_desc = [' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
    let empty_str = ['\0' as u16];

    if false {
        _tprintf(fmt_scdbg.as_ptr(), ServiceName);
    }

    'done: {
        hManager = OpenSCManager(core::ptr::null_mut(),
                                 core::ptr::null_mut(),
                                 SC_MANAGER_CONNECT);
        if hManager.is_null() {
            _tprintf(fmt_open_scm.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, SERVICE_QUERY_CONFIG);
        if hService.is_null() {
            _tprintf(fmt_open_svc.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(hService,
                                 SERVICE_CONFIG_DESCRIPTION,
                                 core::ptr::null_mut(),
                                 0,
                                 core::ptr::addr_of_mut!(cbBytesNeeded)) == 0
        {
            if cbBytesNeeded == 0 {
                _tprintf(fmt_qsc2.as_ptr(), GetLastError());
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pServiceDescription = HeapAlloc(GetProcessHeap(), 0, cbBytesNeeded as SIZE_T) as LPSERVICE_DESCRIPTION;
        if pServiceDescription.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf(fmt_heap.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(hService,
                                 SERVICE_CONFIG_DESCRIPTION,
                                 pServiceDescription as LPBYTE,
                                 cbBytesNeeded,
                                 core::ptr::addr_of_mut!(cbBytesNeeded)) == 0
        {
            _tprintf(fmt_qsc2.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf(fmt_success.as_ptr());

        _tprintf(fmt_svc_name.as_ptr(), ServiceName);
        _tprintf(fmt_desc.as_ptr(),
                 if !(*pServiceDescription).lpDescription.is_null() {
                     (*pServiceDescription).lpDescription as *const WCHAR
                 } else {
                     empty_str.as_ptr() as *const WCHAR
                 });
    }

    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if pServiceDescription != core::ptr::null_mut() {
        HeapFree(GetProcessHeap(), 0, pServiceDescription as PVOID);
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
pub unsafe extern "C" fn SetDescription(ServiceName: LPCTSTR, Description: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut ServiceDescription: SERVICE_DESCRIPTION = core::mem::zeroed();

    if false {
        let FmtScdbg = [
            's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
            't' as u16, 'o' as u16, ' ' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16,
            'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16,
            'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, '-' as u16, ' ' as u16,
            '%' as u16, 's' as u16, '\n' as u16, '\n' as u16, 0u16,
        ];
        _tprintf((FmtScdbg.as_ptr()), ServiceName);
    }

    hManager = OpenSCManager(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_CONNECT,
    );
    'done: {
        if hManager.is_null() {
            let FmtOpenSCM = [
                '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16,
                'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16,
            ];
            _tprintf((FmtOpenSCM.as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, SERVICE_CHANGE_CONFIG);
        if hService.is_null() {
            let FmtOpenService = [
                '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16,
                'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16,
            ];
            _tprintf((FmtOpenService.as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        ServiceDescription.lpDescription = Description as LPTSTR;

        if ChangeServiceConfig2(
            hService,
            SERVICE_CONFIG_DESCRIPTION,
            (core::ptr::addr_of_mut!(ServiceDescription) as LPBYTE).cast(),
        ) == 0
        {
            let FmtChangeFail = [
                '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16,
                'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
                'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, '2' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16,
            ];
            _tprintf((FmtChangeFail.as_ptr()), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        let FmtChangeOk = [
            '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
            'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16,
            'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
            'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, '2' as u16, ' ' as u16,
            'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16,
            '\n' as u16, '\n' as u16, 0u16,
        ];
        _tprintf((FmtChangeOk.as_ptr()));
    }

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
