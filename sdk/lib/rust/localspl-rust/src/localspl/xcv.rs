use crate::localspl::ports::CreatePortEntry;
use crate::localspl::ports::FindPort;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_611F4BFBD191: [u16; 8] = [0x0041u16, 0x0064, 0x0064, 0x0050, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_9A80C6E935E1: [u16; 11] = [
    0x0044u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0050, 0x006F, 0x0072, 0x0074,
    0x0000,
];

use core::ffi::c_void;

unsafe extern "C" fn _HandleAddPort(
    hXcv: HANDLE,
    pInputData: PBYTE,
    pcbOutputNeeded: PDWORD,
    pdwStatus: *mut DWORD,
) -> DWORD {
    let mut res: DWORD = 0;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pXcv: PLOCAL_XCV_HANDLE = core::ptr::null_mut();
    let pHandle: PLOCAL_HANDLE = hXcv as PLOCAL_HANDLE;
    let pPortName: PWSTR = pInputData as PWSTR;

    'cleanup: {
        // Check if this is a printer handle.
        if (*pHandle).HandleType as DWORD != HandleType_Xcv as DWORD {
            res = ERROR_INVALID_HANDLE as DWORD;
            break 'cleanup;
        }

        pXcv = (*pHandle).pSpecificHandle as PLOCAL_XCV_HANDLE;

        pPrintMonitor = (*pXcv).pPrintMonitor;
        if pPrintMonitor.is_null() {
            res = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        // Sanity checks
        if pInputData.is_null() || pcbOutputNeeded.is_null() {
            res = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_add(1);
        if (*pPrintMonitor).bIsLevel2 != 0
            && (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvDataPort
                .is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                RUSTCONV_WSTR_611F4BFBD191.as_ptr(),
                pInputData,
                0 as DWORD,
                core::ptr::null_mut::<BYTE>(),
                0 as DWORD,
                pcbOutputNeeded,
            );
        } else if (*pPrintMonitor).bIsLevel2 == 0
            && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                RUSTCONV_WSTR_611F4BFBD191.as_ptr(),
                pInputData,
                0 as DWORD,
                core::ptr::null_mut::<BYTE>(),
                0 as DWORD,
                pcbOutputNeeded,
            );
        }
        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_sub(1);

        if res == ERROR_SUCCESS as DWORD {
            CreatePortEntry(pPortName as PCWSTR, pPrintMonitor);
        }
    }

    // Cleanup:
    if !pdwStatus.is_null() {
        *pdwStatus = res;
    }
    return res;
}

unsafe extern "C" fn _HandleDeletePort(
    mut hXcv: HANDLE,
    mut pInputData: PBYTE,
    mut pcbOutputNeeded: PDWORD,
    mut pdwStatus: *mut DWORD,
) -> DWORD {
    let mut res: DWORD = 0;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = ::core::ptr::null_mut::<_LOCAL_PRINT_MONITOR>();
    let mut pXcv: PLOCAL_XCV_HANDLE = ::core::ptr::null_mut::<_LOCAL_XCV_HANDLE>();
    let mut pPort: PLOCAL_PORT = ::core::ptr::null_mut::<_LOCAL_PORT>();
    let mut pHandle: PLOCAL_HANDLE = hXcv as PLOCAL_HANDLE;
    let mut pPortName: PWSTR = pInputData as PWSTR;
    'cleanup: {
        // Check if this is a printer handle.
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Xcv as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            res = ERROR_INVALID_HANDLE as DWORD;
            break 'cleanup;
        }

        pXcv = (*pHandle).pSpecificHandle as PLOCAL_XCV_HANDLE;

        pPrintMonitor = (*pXcv).pPrintMonitor;
        if pPrintMonitor.is_null() {
            res = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        // Sanity checks
        if pInputData.is_null() || pcbOutputNeeded.is_null() {
            res = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_add(1);
        //
        //  Call back to monitor, update the Registry and Registry List.
        //
        if (*pPrintMonitor).bIsLevel2 != 0
            && (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvDataPort
                .is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                RUSTCONV_WSTR_9A80C6E935E1.as_ptr(),
                pInputData,
                0 as DWORD,
                ::core::ptr::null_mut::<BYTE>(),
                0 as DWORD,
                pcbOutputNeeded,
            );
        } else if (*pPrintMonitor).bIsLevel2 == 0
            && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                RUSTCONV_WSTR_9A80C6E935E1.as_ptr(),
                pInputData,
                0 as DWORD,
                ::core::ptr::null_mut::<BYTE>(),
                0 as DWORD,
                pcbOutputNeeded,
            );
        }
        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_sub(1);
        //
        //  Now find and remove Local Port data.
        //
        if res == ERROR_SUCCESS as DWORD {
            pPort = FindPort(pPortName as PCWSTR);
            if !pPort.is_null() {
                RemoveEntryList(&raw mut (*pPort).Entry);

                DllFreeSplMem(pPort as PVOID);
            }
        }
    }
    if !pdwStatus.is_null() {
        *pdwStatus = res;
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn LocalXcvData(hXcv: HANDLE, pszDataName: *const WCHAR, pInputData: *mut BYTE, cbInputData: DWORD, pOutputData: *mut BYTE, cbOutputData: DWORD, pcbOutputNeeded: *mut DWORD, pdwStatus: *mut DWORD) -> BOOL {
    let mut res: DWORD = 0;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pXcv: PLOCAL_XCV_HANDLE = core::ptr::null_mut();
    let pHandle: PLOCAL_HANDLE = hXcv as PLOCAL_HANDLE;

    'cleanup: {
        // Sanity checks
        if pszDataName.is_null() {
            res = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        // Call the appropriate handler for the requested data name.
        if wcscmp(pszDataName, RUSTCONV_WSTR_611F4BFBD191.as_ptr()) == 0 {
            return _HandleAddPort(
                hXcv,
                pInputData as PBYTE,
                pcbOutputNeeded as PDWORD,
                pdwStatus,
            ) as BOOL;
        }

        if wcscmp(pszDataName, RUSTCONV_WSTR_9A80C6E935E1.as_ptr()) == 0 {
            return _HandleDeletePort(
                hXcv,
                pInputData as PBYTE,
                pcbOutputNeeded as PDWORD,
                pdwStatus,
            ) as BOOL;
        }

        //
        // After the two Intercept Handlers, defer call back to Monitor.
        //

        // Check if this is a printer handle.
        if (*pHandle).HandleType != HandleType_Xcv {
            res = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        pXcv = (*pHandle).pSpecificHandle as PLOCAL_XCV_HANDLE;

        pPrintMonitor = (*pXcv).pPrintMonitor;
        if pPrintMonitor.is_null() {
            res = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_add(1);
        if (*pPrintMonitor).bIsLevel2 != 0
            && (*((*pPrintMonitor).pMonitor as PMONITOR2)).pfnXcvDataPort.is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                pszDataName as LPCWSTR,
                pInputData as PBYTE,
                cbInputData,
                pOutputData as PBYTE,
                cbOutputData,
                pcbOutputNeeded as PDWORD,
            );
        } else if (*pPrintMonitor).bIsLevel2 == 0
            && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .is_some()
        {
            res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvDataPort
                .unwrap_unchecked()(
                (*pXcv).hXcv,
                pszDataName as LPCWSTR,
                pInputData as PBYTE,
                cbInputData,
                pOutputData as PBYTE,
                cbOutputData,
                pcbOutputNeeded as PDWORD,
            );
        }
        (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_sub(1);
    }

    // Cleanup:
    SetLastError(res);
    if !pdwStatus.is_null() {
        *pdwStatus = res;
    }
    return res as BOOL;
}
