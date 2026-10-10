use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcGetSpoolFileInfo(hPrinter: WINSPOOL_PRINTER_HANDLE, hProcessHandle: WINSPOOL_HANDLE, Level: DWORD, pFileInfo: *mut WINSPOOL_FILE_INFO_1, dwSize: DWORD, dwNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS as DWORD {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/printing/base/spoolsv/spoolfile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"_RpcGetSpoolFileInfo\0" as *const u8 as *const ::core::ffi::c_char,
                18 as ::core::ffi::c_int,
                b"RpcImpersonateClient failed with error %lu!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                dwErrorCode,
            );
        }
        return dwErrorCode;
    }
    if SplGetSpoolFileInfo(
        hPrinter as HANDLE,
        hProcessHandle as HANDLE,
        Level,
        pFileInfo as *mut FILE_INFO_1,
        dwSize,
        dwNeeded,
    ) == 0
    {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcCommitSpoolData(hPrinter: WINSPOOL_PRINTER_HANDLE, hProcessHandle: WINSPOOL_HANDLE, cbCommit: DWORD, Level: DWORD, pFileInfo: *mut WINSPOOL_FILE_INFO_1, dwSize: DWORD, dwNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if SplCommitSpoolData(hPrinter as HANDLE, hProcessHandle as HANDLE, cbCommit, Level, pFileInfo as *mut FILE_INFO_1, dwSize, dwNeeded) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetSpoolFileInfo2(
    mut hPrinter: WINSPOOL_PRINTER_HANDLE,
    mut dwProcessId: DWORD,
    mut Level: DWORD,
    mut pFileInfoContainer: *mut WINSPOOL_FILE_INFO_CONTAINER,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut dwNeeded: DWORD = 0;
    let mut hProcessHandle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    hProcessHandle = OpenProcess(PROCESS_DUP_HANDLE, FALSE as BOOL, dwProcessId);

    if SplGetSpoolFileInfo(
        hPrinter as HANDLE,
        hProcessHandle,
        Level,
        (*pFileInfoContainer).FileInfo.pFileInfo1 as *mut FILE_INFO_1,
        core::mem::size_of::<FILE_INFO_1>() as DWORD,
        &raw mut dwNeeded,
    ) == 0
    {
        dwErrorCode = GetLastError();
    }

    if !hProcessHandle.is_null() {
        CloseHandle(hProcessHandle);
    }

    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcCommitSpoolData2(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    dwProcessId: DWORD,
    cbCommit: DWORD,
    Level: DWORD,
    pFileInfoContainer: *mut WINSPOOL_FILE_INFO_CONTAINER,
) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut dwNeeded: DWORD = 0;
    let mut hProcessHandle: HANDLE;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    hProcessHandle = OpenProcess(PROCESS_DUP_HANDLE, FALSE as BOOL, dwProcessId);

    if SplCommitSpoolData(
        hPrinter as HANDLE,
        hProcessHandle,
        cbCommit,
        Level,
        (*pFileInfoContainer).FileInfo.pFileInfo1 as *mut FILE_INFO_1,
        core::mem::size_of::<FILE_INFO_1>() as DWORD,
        core::ptr::addr_of_mut!(dwNeeded),
    ) == 0
    {
        dwErrorCode = GetLastError();
    }

    if !hProcessHandle.is_null() {
        CloseHandle(hProcessHandle);
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcCloseSpoolFileHandle(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if SplCloseSpoolFileHandle(hPrinter as HANDLE) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
