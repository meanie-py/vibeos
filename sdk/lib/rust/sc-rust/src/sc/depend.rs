use crate::sc::print::PrintServiceStatus;
use crate::sc::sc::ReportLastError;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn EnumDepend(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut BufferSize: DWORD = 0;
    let mut EntriesRead: DWORD = 0;
    let mut pBuffer: LPENUM_SERVICE_STATUS = core::ptr::null_mut();
    let mut i: DWORD;

    'done: {
        hManager = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0043, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenSCManager FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, SERVICE_ENUMERATE_DEPENDENTS);
        if hService.is_null() {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenService FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if EnumDependentServices(
            hService,
            SERVICE_STATE_ALL,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(BufferSize),
            core::ptr::addr_of_mut!(EntriesRead),
        ) == 0
        {
            if BufferSize == 0 {
                _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0045, 0x006e, 0x0075, 0x006d, 0x0044, 0x0065, 0x0070, 0x0065, 0x006e, 0x0064, 0x0065, 0x006e, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] EnumDependentServices FAILED %lu:\n\n" */, GetLastError());
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            BufferSize as SIZE_T,
        ) as LPENUM_SERVICE_STATUS;
        if pBuffer.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0048, 0x0065, 0x0061, 0x0070, 0x0041, 0x006c, 0x006c, 0x006f, 0x0063, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] HeapAlloc FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if EnumDependentServices(
            hService,
            SERVICE_STATE_ALL,
            pBuffer,
            BufferSize,
            core::ptr::addr_of_mut!(BufferSize),
            core::ptr::addr_of_mut!(EntriesRead),
        ) == 0
        {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0045, 0x006e, 0x0075, 0x006d, 0x0044, 0x0065, 0x0070, 0x0065, 0x006e, 0x0064, 0x0065, 0x006e, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] EnumDependentServices FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf((&[0x0045, 0x006e, 0x0075, 0x006d, 0x003a, 0x0020, 0x0065, 0x006e, 0x0074, 0x0072, 0x0069, 0x0065, 0x0073, 0x0052, 0x0065, 0x0061, 0x0064, 0x0020, 0x003d, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "Enum: entriesRead = %lu\n" */, EntriesRead);

        i = 0;
        while i < EntriesRead {
            _tprintf((&[0x000a, 0x0000u16]).as_ptr() /* _T "\n" */);
            _tprintf((&[0x0053, 0x0045, 0x0052, 0x0056, 0x0049, 0x0043, 0x0045, 0x005f, 0x004e, 0x0041, 0x004d, 0x0045, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "SERVICE_NAME: %s\n" */, (*pBuffer.offset(i as isize)).lpServiceName);
            _tprintf((&[0x0044, 0x0049, 0x0053, 0x0050, 0x004c, 0x0041, 0x0059, 0x005f, 0x004e, 0x0041, 0x004d, 0x0045, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "DISPLAY_NAME: %s\n" */, (*pBuffer.offset(i as isize)).lpDisplayName);
            PrintServiceStatus(core::ptr::addr_of_mut!((*pBuffer.offset(i as isize)).ServiceStatus));
            i = i.wrapping_add(1);
        }
    }

    // done:
    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pBuffer as PVOID);
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
