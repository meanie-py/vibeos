use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrintProvidor(mut pName: WINSPOOL_HANDLE, mut pProvidorContainer: *mut WINSPOOL_PROVIDOR_CONTAINER) -> DWORD {
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
                b"win32ss/printing/base/spoolsv/printproviders.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"_RpcAddPrintProvidor\0" as *const u8 as *const ::core::ffi::c_char,
                18 as ::core::ffi::c_int,
                b"RpcImpersonateClient failed with error %lu!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                dwErrorCode,
            );
        }
        return dwErrorCode;
    }
    if AddPrintProvidorW(
        pName as LPWSTR,
        (*pProvidorContainer).Level,
        (*pProvidorContainer).ProvidorInfo.pProvidorInfo1 as LPBYTE,
    ) == 0
    {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrintProvidor(mut pName: WINSPOOL_HANDLE, mut pEnvironment: *mut WCHAR, mut pPrintProviderName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/printing/base/spoolsv/printproviders.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"_RpcDeletePrintProvidor\0".as_ptr() as *const core::ffi::c_char,
                38 as core::ffi::c_int,
                b"RpcImpersonateClient failed with error %lu!\n\0".as_ptr()
                    as *const core::ffi::c_char,
                dwErrorCode,
            );
        }
        return dwErrorCode;
    }
    if DeletePrintProvidorW(pName, pEnvironment, pPrintProviderName) == 0 {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    return dwErrorCode;
}
