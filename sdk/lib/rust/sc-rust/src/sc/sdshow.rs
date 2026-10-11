use crate::sc::sc::ReportLastError;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn SdShow(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut cbBytesNeeded: DWORD = 0;
    let mut pSecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut pStringBuffer: LPTSTR = core::ptr::null_mut();

    if false {
        _tprintf(
            [
                's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
                't' as u16, 'o' as u16, ' ' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16,
                's' as u16, 'd' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16,
                '\n' as u16, 0u16,
            ]
            .as_ptr(),
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
                [
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16,
                    'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16,
                    'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16,
                    'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16,
                    'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, READ_CONTROL);
        if hService.is_null() {
            _tprintf(
                [
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16,
                    'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16,
                    'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16,
                    'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16,
                    '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceObjectSecurity(
            hService,
            DACL_SECURITY_INFORMATION,
            &raw mut pSecurityDescriptor as PSECURITY_DESCRIPTOR,
            core::mem::size_of::<PSECURITY_DESCRIPTOR>() as DWORD,
            &raw mut cbBytesNeeded,
        ) == 0
        {
            if cbBytesNeeded == 0 {
                _tprintf(
                    [
                        '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16,
                        'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16,
                        'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'O' as u16,
                        'b' as u16, 'j' as u16, 'e' as u16, 'c' as u16, 't' as u16, 'S' as u16,
                        'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16,
                        'y' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16,
                        'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16,
                        ':' as u16, '\n' as u16, '\n' as u16, 0u16,
                    ]
                    .as_ptr(),
                    GetLastError(),
                );
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pSecurityDescriptor = HeapAlloc(
            GetProcessHeap(),
            0,
            cbBytesNeeded as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if pSecurityDescriptor.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf(
                [
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'H' as u16, 'e' as u16,
                    'a' as u16, 'p' as u16, 'A' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'c' as u16,
                    ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16,
                    ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16,
                    '\n' as u16, 0u16,
                ]
                .as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceObjectSecurity(
            hService,
            DACL_SECURITY_INFORMATION,
            pSecurityDescriptor,
            cbBytesNeeded,
            &raw mut cbBytesNeeded,
        ) == 0
        {
            _tprintf(
                [
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'Q' as u16,
                    'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'S' as u16, 'e' as u16,
                    'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'O' as u16,
                    'b' as u16, 'j' as u16, 'e' as u16, 'c' as u16, 't' as u16, 'S' as u16,
                    'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16,
                    'y' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16,
                    'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16,
                    ':' as u16, '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if ConvertSecurityDescriptorToStringSecurityDescriptor(
            pSecurityDescriptor,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &raw mut pStringBuffer,
            core::ptr::null_mut(),
        ) == 0
        {
            _tprintf(
                [
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'C' as u16,
                    'o' as u16, 'n' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 't' as u16,
                    'S' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16,
                    't' as u16, 'y' as u16, 'D' as u16, 'e' as u16, 's' as u16, 'c' as u16,
                    'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'o' as u16, 'r' as u16,
                    'T' as u16, 'o' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16,
                    'n' as u16, 'g' as u16, 'S' as u16, 'e' as u16, 'c' as u16, 'u' as u16,
                    'r' as u16, 'i' as u16, 't' as u16, 'y' as u16, 'D' as u16, 'e' as u16,
                    's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16,
                    'o' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16,
                    'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16,
                    'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr(),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf(
            [
                '\n' as u16, '%' as u16, 's' as u16, '\n' as u16, 0u16,
            ]
            .as_ptr(),
            pStringBuffer,
        );
    }

    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pStringBuffer.is_null() {
        LocalFree(pStringBuffer as HLOCAL);
    }

    if !pSecurityDescriptor.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pSecurityDescriptor as PVOID,
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
