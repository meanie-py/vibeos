use crate::sc::sc::ReportLastError;
use crate::sc::sc::hSCManager;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn Delete(ServiceName: LPCTSTR) -> BOOL {
    let mut hSCManager_: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (TRUE) as i32;

    if false {
        let fmt0 = ['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, '\n' as u16, 0];
        _tprintf(fmt0.as_ptr(), ServiceName);
    }

    'done: {
        hSCManager_ = OpenSCManager(
            core::ptr::null(),
            core::ptr::null(),
            SC_MANAGER_CONNECT as DWORD,
        );
        if hSCManager_.is_null() {
            let fmt1 = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
            _tprintf(fmt1.as_ptr(), GetLastError());
            bRet = (FALSE) as i32;
            break 'done;
        }

        hSc = OpenService(hSCManager_, ServiceName as LPCWSTR, DELETE as DWORD);
        if hSc.is_null() {
            let fmt2 = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
            _tprintf(fmt2.as_ptr(), GetLastError());
            bRet = (FALSE) as i32;
            break 'done;
        }

        if ((DeleteService(hSc)) == 0) {
            let fmt3 = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'D' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
            _tprintf(fmt3.as_ptr(), GetLastError());
            bRet = (FALSE) as i32;
            break 'done;
        }

        let fmt4 = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'D' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16, '\n' as u16, '\n' as u16, 0];
        _tprintf(fmt4.as_ptr());
    }

    if (bRet) as u32 == FALSE {
        ReportLastError();
    }

    if !hSc.is_null() {
        CloseServiceHandle(hSc);
    }

    if !hSCManager_.is_null() {
        CloseServiceHandle(hSCManager_);
    }

    return bRet;
}
