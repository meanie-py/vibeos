use crate::sc::print::PrintService;
use crate::sc::sc::ReportLastError;
use crate::sc::sc::hSCManager;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn Control(Control: DWORD, ServiceName: LPCTSTR, Args: *mut LPCTSTR, ArgCount: INT) -> BOOL {
    static FMT_SVC: [u16; 25] = ['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
    static FMT_CMD: [u16; 15] = ['c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, '\n' as u16, 0];
    static FMT_ARGS: [u16; 12] = ['A' as u16, 'r' as u16, 'g' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'n' as u16, 't' as u16, 's' as u16, ':' as u16, '\n' as u16, 0];
    static FMT_ARG: [u16; 6] = [' ' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
    static FMT_NL: [u16; 2] = ['\n' as u16, 0];
    static FMT_F: [u16; 33] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    static FMT_G: [u16; 31] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];
    static FMT_H: [u16; 34] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0];

    let mut hSCManagerLocal: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut Status: SERVICE_STATUS = core::mem::zeroed();
    let mut dwDesiredAccess: DWORD = 0;
    let mut bResult: BOOL = (TRUE) as i32;
    let mut StatusEx: SERVICE_STATUS_PROCESS = core::mem::zeroed();

    if false {
        let mut TmpArgs: *mut LPCTSTR = Args;
        let mut TmpCnt: INT = ArgCount;
        _tprintf(FMT_SVC.as_ptr(), ServiceName);
        _tprintf(FMT_CMD.as_ptr(), Control);
        _tprintf(FMT_ARGS.as_ptr());
        while TmpCnt != 0 {
            _tprintf(FMT_ARG.as_ptr(), *TmpArgs);
            TmpArgs = TmpArgs.add(1);
            TmpCnt -= 1;
        }
        _tprintf(FMT_NL.as_ptr());
    }

    if Control == SERVICE_CONTROL_STOP {
        dwDesiredAccess = SERVICE_STOP;
    } else if Control == SERVICE_CONTROL_PAUSE
        || Control == SERVICE_CONTROL_CONTINUE
        || Control == SERVICE_CONTROL_PARAMCHANGE
        || Control == SERVICE_CONTROL_NETBINDADD
        || Control == SERVICE_CONTROL_NETBINDREMOVE
        || Control == SERVICE_CONTROL_NETBINDENABLE
        || Control == SERVICE_CONTROL_NETBINDDISABLE
    {
        dwDesiredAccess = SERVICE_PAUSE_CONTINUE;
    } else if Control == SERVICE_CONTROL_INTERROGATE {
        dwDesiredAccess = SERVICE_INTERROGATE;
    } else {
        if Control >= 128 && Control <= 255 {
            dwDesiredAccess = SERVICE_USER_DEFINED_CONTROL;
        } else {
            dwDesiredAccess = 0;
        }
    }

    'done: {
        hSCManagerLocal = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hSCManagerLocal.is_null() {
            _tprintf(FMT_F.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hSCManagerLocal, ServiceName, dwDesiredAccess);
        if hService.is_null() {
            _tprintf(FMT_G.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if ControlService(hService, Control, core::ptr::addr_of_mut!(Status)) == 0 {
            _tprintf(FMT_H.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        /* FIXME: lazy hack ;) */
        CopyMemory!(
            core::ptr::addr_of_mut!(StatusEx) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(Status) as *mut core::ffi::c_void,
            core::mem::size_of::<SERVICE_STATUS>(),
        );
        StatusEx.dwProcessId = 0;
        StatusEx.dwServiceFlags = 0;

        PrintService(
            ServiceName,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(StatusEx),
            (FALSE) as BOOL,
        );
    }

    if bResult == 0 {
        ReportLastError();
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hSCManagerLocal.is_null() {
        CloseServiceHandle(hSCManagerLocal);
    }

    return bResult;
}
