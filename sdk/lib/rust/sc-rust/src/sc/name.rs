use crate::sc::sc::ReportLastError;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetDisplayName(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut BufferSize: DWORD = 0;
    let mut pBuffer: LPTSTR = core::ptr::null_mut();
    'done: {
        hManager = OpenSCManager(
            core::ptr::null(),
            core::ptr::null(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0043, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenSCManager FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        if GetServiceDisplayName(
            hManager,
            ServiceName,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(BufferSize),
        ) == 0
        {
            if BufferSize == 0 {
                _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0047, 0x0065, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x004e, 0x0061, 0x006d, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] GetServiceDisplayName FAILED %lu:\n\n" */, GetLastError());
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (BufferSize.wrapping_add(1 as DWORD) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPTSTR;
        if pBuffer.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0048, 0x0065, 0x0061, 0x0070, 0x0041, 0x006c, 0x006c, 0x006f, 0x0063, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] HeapAlloc FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        BufferSize = BufferSize.wrapping_add(1);
        if GetServiceDisplayName(
            hManager,
            ServiceName,
            pBuffer,
            core::ptr::addr_of_mut!(BufferSize),
        ) == 0
        {
            _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0047, 0x0065, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x004e, 0x0061, 0x006d, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] GetServiceDisplayName FAILED %lu:\n\n" */, GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0047, 0x0065, 0x0074, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x004e, 0x0061, 0x006d, 0x0065, 0x0020, 0x0053, 0x0055, 0x0043, 0x0043, 0x0045, 0x0053, 0x0053, 0x0020, 0x0020, 0x004e, 0x0061, 0x006d, 0x0065, 0x0020, 0x003d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] GetServiceDisplayName SUCCESS  Name = %s\n" */, pBuffer);
    }

    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pBuffer as PVOID);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn GetKeyName(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut BufferSize: DWORD = 0;
    let mut pBuffer: LPTSTR = core::ptr::null_mut();

    'done: {
        hManager = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            _tprintf(
                ([
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                    'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16,
                    'C' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16,
                    'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'F' as u16,
                    'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16,
                    ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16,
                    '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr()),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        if GetServiceKeyName(
            hManager,
            ServiceName,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(BufferSize),
        ) == 0
        {
            if BufferSize == 0 {
                _tprintf(
                    ([
                        '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                        'G' as u16, 'e' as u16, 't' as u16, 'S' as u16, 'e' as u16,
                        'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
                        'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16,
                        'm' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16,
                        'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                        '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16,
                        '\n' as u16, 0u16,
                    ]
                    .as_ptr()),
                    GetLastError(),
                );
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            ((BufferSize + 1) as SIZE_T)
                * (core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPTSTR;
        if pBuffer.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            _tprintf(
                ([
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                    'H' as u16, 'e' as u16, 'a' as u16, 'p' as u16, 'A' as u16,
                    'l' as u16, 'l' as u16, 'o' as u16, 'c' as u16, ' ' as u16,
                    'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16,
                    'D' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16,
                    ':' as u16, '\n' as u16, '\n' as u16, 0u16,
                ]
                .as_ptr()),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        BufferSize += 1;
        if GetServiceKeyName(
            hManager,
            ServiceName,
            pBuffer,
            core::ptr::addr_of_mut!(BufferSize),
        ) == 0
        {
            _tprintf(
                ([
                    '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                    'G' as u16, 'e' as u16, 't' as u16, 'S' as u16, 'e' as u16,
                    'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
                    'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16,
                    'm' as u16, 'e' as u16, ' ' as u16, 'F' as u16, 'A' as u16,
                    'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                    '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16,
                    '\n' as u16, 0u16,
                ]
                .as_ptr()),
                GetLastError(),
            );
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf(
            ([
                '[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'G' as u16, 'e' as u16, 't' as u16, 'S' as u16, 'e' as u16,
                'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
                'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16,
                'm' as u16, 'e' as u16, ' ' as u16, 'S' as u16, 'U' as u16,
                'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16,
                ' ' as u16, ' ' as u16, 'N' as u16, 'a' as u16, 'm' as u16,
                'e' as u16, ' ' as u16, '=' as u16, ' ' as u16, '%' as u16,
                's' as u16, '\n' as u16, 0u16,
            ]
            .as_ptr()),
            pBuffer,
        );
    }

    // done:
    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pBuffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pBuffer as PVOID,
        );
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
