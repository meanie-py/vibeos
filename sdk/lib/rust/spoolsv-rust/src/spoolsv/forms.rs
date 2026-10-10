use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddForm(hPrinter: WINSPOOL_PRINTER_HANDLE, pFormInfoContainer: *mut WINSPOOL_FORM_CONTAINER) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if AddFormW(hPrinter, (*pFormInfoContainer).Level, (*pFormInfoContainer).FormInfo.pFormInfo1 as PBYTE) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeleteForm(hPrinter: WINSPOOL_PRINTER_HANDLE, pFormName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if DeleteFormW(hPrinter, pFormName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumForms(hPrinter: WINSPOOL_PRINTER_HANDLE, Level: DWORD, pForm: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pFormsEnumAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pFormsEnumAligned = (AlignRpcPtr((pForm).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumFormsW(hPrinter, Level, pFormsEnumAligned, cbBuf, pcbNeeded, pcReturned) != 0
    {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructuresArray((pFormsEnumAligned).cast(), *pcReturned, (*pFormInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pFormInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pForm).cast(), (pFormsEnumAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetForm(hPrinter: WINSPOOL_PRINTER_HANDLE, pFormName: *mut WCHAR, Level: DWORD, pForm: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pFormAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pFormAligned = (AlignRpcPtr((pForm).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetFormW(hPrinter, pFormName, Level, pFormAligned, cbBuf, pcbNeeded) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructure((pFormAligned).cast(), (*pFormInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pFormInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pForm).cast(), (pFormAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetForm(hPrinter: WINSPOOL_PRINTER_HANDLE, pFormName: *mut WCHAR, pFormInfoContainer: *mut WINSPOOL_FORM_CONTAINER) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if SetFormW(hPrinter, pFormName, (*pFormInfoContainer).Level, (*pFormInfoContainer).FormInfo.pFormInfo1 as PBYTE) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
