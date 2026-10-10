use crate::sc::print::PrintService;
use crate::sc::query::QueryService;
use crate::sc::sc::ReportLastError;
use crate::sc::sc::hSCManager;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn Start(ServiceName: LPCTSTR, ServiceArgs: *mut LPCTSTR, ArgCount: INT) -> BOOL {
    let mut ServiceArgs = ServiceArgs;
    let mut hSCManagerLocal: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut pServiceInfo: LPSERVICE_STATUS_PROCESS = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;

    // SCDBG block (#ifdef SCDBG; compiled out in the default build)
    if false {
        let mut TmpArgs: *mut LPCTSTR = ServiceArgs;
        let mut TmpCnt: INT = ArgCount;
        _tprintf((&[0x0073, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0074, 0x006f, 0x0020, 0x0063, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "service to control - %s\n" */, ServiceName);
        _tprintf((&[0x0041, 0x0072, 0x0067, 0x0075, 0x006d, 0x0065, 0x006e, 0x0074, 0x0073, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "Arguments:\n" */);
        while TmpCnt != 0 {
            _tprintf((&[0x0020, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "  %s\n" */, *TmpArgs);
            TmpArgs = TmpArgs.add(1);
            TmpCnt -= 1;
        }
        _tprintf((&[0x000a, 0x0000u16]).as_ptr() /* _T "\n" */);
    }

    'done: {
        hSCManagerLocal = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hSCManagerLocal.is_null() {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0043, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenSCManager FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hSc = OpenService(
            hSCManagerLocal,
            ServiceName,
            SERVICE_START | SERVICE_QUERY_STATUS,
        );
        if hSc.is_null() {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenService FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if ArgCount == 0 {
            ServiceArgs = core::ptr::null_mut();
        }

        if StartService(hSc, ArgCount as DWORD, ServiceArgs) == 0 {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] StartService FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        pServiceInfo = QueryService(ServiceName);
        if !pServiceInfo.is_null() {
            PrintService(
                ServiceName,
                core::ptr::null_mut(),
                pServiceInfo,
                (TRUE) as BOOL,
            );

            HeapFree(GetProcessHeap(), 0, pServiceInfo as *mut core::ffi::c_void);
        }
    }

    // done:
    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !hSc.is_null() {
        CloseServiceHandle(hSc);
    }

    if !hSCManagerLocal.is_null() {
        CloseServiceHandle(hSCManagerLocal);
    }

    return bResult;
}
