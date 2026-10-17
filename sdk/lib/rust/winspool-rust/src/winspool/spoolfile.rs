use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn GetSpoolFileHandle(mut hPrinter: HANDLE) -> HANDLE {
    let mut dwErrorCode: DWORD = 0;
    let mut cpid: DWORD = 0;
    let mut FileInfoContainer: WINSPOOL_FILE_INFO_CONTAINER = _WINSPOOL_FILE_INFO_CONTAINER {
        Level: 0,
        FileInfo: _WINSPOOL_FILE_INFO_CONTAINER__bindgen_ty_1 {
            pFileInfo1: core::ptr::null_mut::<WINSPOOL_FILE_INFO_1>(),
        },
    };
    let mut wsplfi: WINSPOOL_FILE_INFO_1 = _WINSPOOL_FILE_INFO_1 {
        bInheritHandle: 0,
        hSpoolFileHandle: core::ptr::null_mut::<WCHAR>(),
        dwOptions: 0,
    };
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut hHandle: HANDLE = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/spoolfile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetSpoolFileHandle\0" as *const u8 as *const ::core::ffi::c_char,
            20 as ::core::ffi::c_int,
            b"GetSpoolFileHandle(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hPrinter,
        );
    }
    if IntProtectHandle(hPrinter, FALSE) != 0 {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    } else {
        if (*pHandle).hSpoolFileHandle != INVALID_HANDLE_VALUE {
            hHandle = (*pHandle).hSpoolFileHandle;
        } else {
            cpid = GetCurrentProcessId();
            FileInfoContainer.Level = 1 as DWORD;
            FileInfoContainer.FileInfo.pFileInfo1 = &raw mut wsplfi;
            // Do the RPC call (RpcTryExcept body).
            dwErrorCode = rpc_try(|| _RpcGetSpoolFileInfo2(
                &raw mut (*pHandle).hPrinter as WINSPOOL_PRINTER_HANDLE,
                cpid,
                1 as DWORD,
                &raw mut FileInfoContainer,
            )).unwrap_or_else(|code| code as _);
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER) body.
            if _SEH2_Volatile0 != 0 || 0 as ::core::ffi::c_int != 0 && 1 as ::core::ffi::c_int != 0
            {
                dwErrorCode = RpcExceptionCode!() as DWORD;
                _SEH2_VolatileExceptionCode = dwErrorCode as ::core::ffi::c_int;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"win32ss/printing/base/winspool/spoolfile.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"GetSpoolFileHandle\0" as *const u8
                            as *const ::core::ffi::c_char,
                        47 as ::core::ffi::c_int,
                        b"_RpcGetSpoolFileInfo failed with exception code %lu!\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        dwErrorCode,
                    );
                }
            }
            if dwErrorCode == ERROR_SUCCESS as DWORD {
                (*pHandle).hSpoolFileHandle = wsplfi.hSpoolFileHandle as HANDLE;
                (*pHandle).dwOptions = wsplfi.dwOptions;
                hHandle = (*pHandle).hSpoolFileHandle;
            }
        }
        IntUnprotectHandle(pHandle as HANDLE);
    }
    SetLastError(dwErrorCode);
    return hHandle;
}

#[no_mangle]
pub unsafe extern "system" fn CommitSpoolData(hPrinter: HANDLE, hSpoolFile: HANDLE, cbCommit: DWORD) -> HANDLE {
    let mut dwErrorCode: DWORD = 0;
    let mut cpid: DWORD = 0;
    let mut FileInfoContainer: WINSPOOL_FILE_INFO_CONTAINER = core::mem::zeroed();
    let mut wsplfi: WINSPOOL_FILE_INFO_1 = core::mem::zeroed();
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut hHandle: HANDLE = INVALID_HANDLE_VALUE;

    if IntProtectHandle(hPrinter, FALSE) != 0 {
        return hHandle;
    }

    if (*pHandle).hSpoolFileHandle == INVALID_HANDLE_VALUE || (*pHandle).hSpoolFileHandle != hSpoolFile {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    } else {
        cpid = GetCurrentProcessId();

        FileInfoContainer.Level = 1 as DWORD;
        FileInfoContainer.FileInfo.pFileInfo1 = core::ptr::addr_of_mut!(wsplfi);

        // Do the RPC call.
        // RpcTryExcept
        dwErrorCode = rpc_try(|| _RpcCommitSpoolData2(
            core::ptr::addr_of_mut!((*pHandle).hPrinter) as WINSPOOL_PRINTER_HANDLE,
            cpid,
            cbCommit,
            1 as DWORD,
            core::ptr::addr_of_mut!(FileInfoContainer),
        )).unwrap_or_else(|code| code as _);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        if _SEH2_Volatile0 != 0 {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS as DWORD {
            if wsplfi.hSpoolFileHandle != INVALID_HANDLE_VALUE as WINSPOOL_HANDLE {
                CloseHandle((*pHandle).hSpoolFileHandle);
                (*pHandle).hSpoolFileHandle = wsplfi.hSpoolFileHandle as HANDLE;
            }
            hHandle = (*pHandle).hSpoolFileHandle;
        }
        IntUnprotectHandle(pHandle as HANDLE);
    }
    SetLastError(dwErrorCode);
    return hHandle;
}

#[no_mangle]
pub unsafe extern "system" fn CloseSpoolFileHandle(hPrinter: HANDLE, hSpoolFile: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/spoolfile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CloseSpoolFileHandle\0" as *const u8 as *const ::core::ffi::c_char,
            124 as ::core::ffi::c_int,
            b"CloseSpoolFileHandle(%p, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hPrinter,
            hSpoolFile,
        );
    }
    if IntProtectHandle(hPrinter, FALSE as BOOL) != 0 {
        return FALSE as BOOL;
    }
    if (*pHandle).hSpoolFileHandle == hSpoolFile {
        CloseHandle((*pHandle).hSpoolFileHandle);
        (*pHandle).hSpoolFileHandle = INVALID_HANDLE_VALUE;

        // Do the RPC call.
        dwErrorCode = rpc_try(|| _RpcCloseSpoolFileHandle(
            core::ptr::addr_of_mut!((*pHandle).hPrinter) as WINSPOOL_PRINTER_HANDLE,
        )).unwrap_or_else(|code| code as _);
        if _SEH2_Volatile0 != 0 {
            dwErrorCode = RpcExceptionCode!() as DWORD;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/base/winspool/spoolfile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"CloseSpoolFileHandle\0" as *const u8 as *const ::core::ffi::c_char,
                    143 as ::core::ffi::c_int,
                    b"_RpcloseSpoolFileHandle failed with exception code %lu!\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    dwErrorCode,
                );
            }
        }
    } else {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    }
    IntUnprotectHandle(pHandle as HANDLE);
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as ::core::ffi::c_int;
}
