use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The marshalling tables are generated from marshalling/printprocessors.h (marshalling_tables/).

#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrintProcessor(pName: WINSPOOL_HANDLE, pEnvironment: *mut WCHAR, pPathName: *mut WCHAR, pPrintProcessorName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS as DWORD {
        return dwErrorCode;
    }
    if AddPrintProcessorW(
        pName as LPWSTR,
        pEnvironment as LPWSTR,
        pPathName as LPWSTR,
        pPrintProcessorName as LPWSTR,
    ) == 0
    {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrintProcessor(
    pName: WINSPOOL_HANDLE,
    pEnvironment: *mut WCHAR,
    pPrintProcessorName: *mut WCHAR,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS as DWORD {
        return dwErrorCode;
    }
    if DeletePrintProcessorW(pName, pEnvironment, pPrintProcessorName) == 0 {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrintProcessorDatatypes(mut pName: WINSPOOL_HANDLE, mut pPrintProcessorName: *mut WCHAR, mut Level: DWORD, mut pDatatypes: *mut BYTE, mut cbBuf: DWORD, mut pcbNeeded: *mut DWORD, mut pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut pDatatypesAligned: PBYTE = core::ptr::null_mut::<BYTE>();
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS as DWORD {
        if (*__wine_dbch___default).flags as i32
            & (1 as i32) << __WINE_DBCL_ERR as i32
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/printing/base/spoolsv/printprocessors.c\0"
                    as *const u8 as *const i8,
                b"_RpcEnumPrintProcessorDatatypes\0" as *const u8
                    as *const i8,
                58 as i32,
                b"RpcImpersonateClient failed with error %lu!\n\0" as *const u8
                    as *const i8,
                dwErrorCode,
            );
        }
        return dwErrorCode;
    }
    pDatatypesAligned = AlignRpcPtr(pDatatypes as PVOID, core::ptr::addr_of_mut!(cbBuf)) as PBYTE;
    if EnumPrintProcessorDatatypesW(
        pName as LPWSTR,
        pPrintProcessorName as LPWSTR,
        Level,
        pDatatypesAligned as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
        pcReturned as LPDWORD,
    ) != 0
    {
        MarshallDownStructuresArray(
            pDatatypesAligned as PVOID,
            *pcReturned,
            core::ptr::addr_of!(DatatypesInfo1Marshalling.pInfo) as *const MARSHALLING_INFO,
            DatatypesInfo1Marshalling.cbStructureSize,
            TRUE as BOOL,
        );
    } else {
        dwErrorCode = GetLastError();
    }
    RpcRevertToSelf();
    UndoAlignRpcPtr(
        pDatatypes as PVOID,
        pDatatypesAligned as PVOID,
        cbBuf,
        pcbNeeded as PDWORD,
    );
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrintProcessors(
    pName: WINSPOOL_HANDLE,
    pEnvironment: *mut WCHAR,
    Level: DWORD,
    pPrintProcessorInfo: *mut BYTE,
    mut cbBuf: DWORD,
    pcbNeeded: *mut DWORD,
    pcReturned: *mut DWORD,
) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pPrintProcessorInfoAligned: PBYTE;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pPrintProcessorInfoAligned = AlignRpcPtr(pPrintProcessorInfo as PVOID, core::ptr::addr_of_mut!(cbBuf)) as PBYTE;

    if EnumPrintProcessorsW(
        pName as LPWSTR,
        pEnvironment as LPWSTR,
        Level,
        pPrintProcessorInfoAligned as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
        pcReturned as LPDWORD,
    ) != 0
    {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructuresArray(
            pPrintProcessorInfoAligned as PVOID,
            *pcReturned,
            core::ptr::addr_of!(PrintProcessorInfo1Marshalling.pInfo) as *const MARSHALLING_INFO,
            PrintProcessorInfo1Marshalling.cbStructureSize,
            TRUE as BOOL,
        );
    } else {
        dwErrorCode = GetLastError() as DWORD;
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr(
        pPrintProcessorInfo as PVOID,
        pPrintProcessorInfoAligned as PVOID,
        cbBuf,
        pcbNeeded as PDWORD,
    );

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrintProcessorDirectory(
    pName: WINSPOOL_HANDLE,
    pEnvironment: *mut WCHAR,
    Level: DWORD,
    pPrintProcessorDirectory: *mut BYTE,
    cbBuf: DWORD,
    pcbNeeded: *mut DWORD,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if GetPrintProcessorDirectoryW(pName, pEnvironment, Level, pPrintProcessorDirectory, cbBuf, pcbNeeded) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
