use crate::winspool::marshalling::MarshallUpStructure;
use ntdll_sys::seh::rpc_try;
use crate::winspool::marshalling::MarshallUpStructuresArray;
use crate::winspool::utils::UnicodeToAnsiInPlace;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddFormA(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut pfi2W: FORM_INFO_2W = core::mem::zeroed();
    let mut pfi2A: PFORM_INFO_2A;
    let mut len: DWORD;
    let mut res: BOOL;

    pfi2A = pForm as PFORM_INFO_2A;

    if (Level < 1) || (Level > 2) {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    if pfi2A.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(pfi2W) as PVOID, core::mem::size_of::<FORM_INFO_2W>());

    if !(*pfi2A).pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pName, -1, core::ptr::null_mut(), 0) as DWORD;
        pfi2W.pName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pName, -1, pfi2W.pName, len as i32);
    }

    pfi2W.Flags = (*pfi2A).Flags;
    pfi2W.Size = (*pfi2A).Size;
    pfi2W.ImageableArea = (*pfi2A).ImageableArea;

    if Level > 1 {
        if !(*pfi2A).pKeyword.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pKeyword, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pKeyword = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR as *const i8;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pKeyword, -1, pfi2W.pKeyword as LPWSTR, len as i32);
        }

        if !(*pfi2A).pMuiDll.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pMuiDll, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pMuiDll = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pMuiDll, -1, pfi2W.pMuiDll as LPWSTR, len as i32);
        }

        if !(*pfi2A).pDisplayName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pDisplayName, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pDisplayName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pDisplayName, -1, pfi2W.pDisplayName as LPWSTR, len as i32);
        }
        pfi2W.StringType = (*pfi2A).StringType;
        pfi2W.dwResourceId = (*pfi2A).dwResourceId;
        pfi2W.wLangId = (*pfi2A).wLangId;
    }

    res = AddFormW(hPrinter, Level, core::ptr::addr_of_mut!(pfi2W) as PBYTE);

    if !pfi2W.pName.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pName as LPVOID);
    }
    if !pfi2W.pKeyword.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pKeyword as LPVOID);
    }
    if !pfi2W.pMuiDll.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pMuiDll as LPVOID);
    }
    if !pfi2W.pDisplayName.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pDisplayName as LPVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn AddFormW(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut FormInfoContainer: WINSPOOL_FORM_CONTAINER = core::mem::zeroed();
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE;
        return FALSE as BOOL;
    }

    if (Level < 1) || (Level > 2) {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    FormInfoContainer.FormInfo.pFormInfo1 = pForm as *mut WINSPOOL_FORM_INFO_1;
    FormInfoContainer.Level = Level;

    // Do the RPC call
    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcAddForm((*pHandle).hPrinter, core::ptr::addr_of_mut!(FormInfoContainer))).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    let rpc_exception_raised: bool = false;
    if rpc_exception_raised {
        dwErrorCode = RpcExceptionCode!() as DWORD;
    }
    // RpcEndExcept;

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFormA(hPrinter: HANDLE, pFormName: PSTR) -> BOOL {
    let mut FormNameW: _UNICODE_STRING = core::mem::zeroed();
    let Ret: BOOL;

    AsciiToUnicode(core::ptr::addr_of_mut!(FormNameW), pFormName);

    Ret = DeleteFormW(hPrinter, FormNameW.Buffer);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(FormNameW));

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFormW(hPrinter: HANDLE, pFormName: PWSTR) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE;
        return FALSE as BOOL;
    }

    // Do the RPC call
    dwErrorCode = rpc_try(|| _RpcDeleteForm((*pHandle).hPrinter, pFormName)).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler body, only reached on an
    // RpcEndExcept

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumFormsA(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut i: DWORD;
    let pfi1w: PFORM_INFO_1W = pForm as PFORM_INFO_1W;
    let pfi2w: PFORM_INFO_2W = pForm as PFORM_INFO_2W;

    'Cleanup: {
        if EnumFormsW(hPrinter, Level, pForm, cbBuf, pcbNeeded, pcReturned) != 0 {
            i = 0;
            while i < *pcReturned {
                match Level {
                    2 => {
                        dwErrorCode = UnicodeToAnsiInPlace((*pfi2w.add(i as usize)).pKeyword as LPWSTR);
                        if dwErrorCode != ERROR_SUCCESS {
                            break 'Cleanup;
                        }
                        dwErrorCode = UnicodeToAnsiInPlace((*pfi2w.add(i as usize)).pMuiDll as LPWSTR);
                        if dwErrorCode != ERROR_SUCCESS {
                            break 'Cleanup;
                        }
                        dwErrorCode = UnicodeToAnsiInPlace((*pfi2w.add(i as usize)).pDisplayName as LPWSTR);
                        if dwErrorCode != ERROR_SUCCESS {
                            break 'Cleanup;
                        }
                        dwErrorCode = UnicodeToAnsiInPlace((*pfi2w.add(i as usize)).pName);
                        if dwErrorCode != ERROR_SUCCESS {
                            break 'Cleanup;
                        }
                    }
                    1 => {
                        dwErrorCode = UnicodeToAnsiInPlace((*pfi1w.add(i as usize)).pName);
                        if dwErrorCode != ERROR_SUCCESS {
                            break 'Cleanup;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            return TRUE as BOOL;
        }
    }
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn EnumFormsW(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        if (Level < 1) || (Level > 2) {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Do the RPC call
        // RpcTryExcept: SEH frame is not modelled in Rust; the try body runs directly.
        dwErrorCode = rpc_try(|| _RpcEnumForms((*pHandle).hPrinter, Level, pForm, cbBuf, pcbNeeded, pcReturned)).unwrap_or_else(|code| code as _);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler would be:
        //     dwErrorCode = RpcExceptionCode();
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructuresArray(
                cbBuf,
                pForm as PVOID,
                *pcReturned,
                core::ptr::addr_of!((*pFormInfoMarshalling[Level as usize]).pInfo) as *const _MARSHALLING_INFO,
                (*pFormInfoMarshalling[Level as usize]).cbStructureSize,
                TRUE as BOOL
            );
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetFormA(hPrinter: HANDLE, pFormName: PSTR, Level: DWORD, pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut len: DWORD = 0;
    let mut FormNameW: LPWSTR = core::ptr::null_mut();
    let pfi1w: PFORM_INFO_1W = pForm as PFORM_INFO_1W;
    let pfi2w: PFORM_INFO_2W = pForm as PFORM_INFO_2W;

    if !pFormName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pFormName, -1, core::ptr::null_mut(), 0) as DWORD;
        FormNameW = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pFormName, -1, FormNameW, len as i32);
    }

    'cleanup: {
        if GetFormW(hPrinter, FormNameW, Level, pForm, cbBuf, pcbNeeded) != 0 {
            match Level {
                2 => {
                    dwErrorCode = UnicodeToAnsiInPlace((*pfi2w).pKeyword as LPWSTR);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pfi2w).pMuiDll as LPWSTR);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pfi2w).pDisplayName as LPWSTR);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pfi2w).pName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }
                1 => {
                    dwErrorCode = UnicodeToAnsiInPlace((*pfi1w).pName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }
                _ => {}
            }
        }
    }
    // Cleanup:
    if !FormNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, FormNameW as LPVOID);
    }
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetFormW(hPrinter: HANDLE, pFormName: PWSTR, Level: DWORD, pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Dismiss invalid levels already at this point.
        if (Level < 1) || (Level > 2) {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if cbBuf != 0 && !pForm.is_null() {
            ZeroMemory!(pForm as *mut core::ffi::c_void, cbBuf as usize);
        }

        // Do the RPC call
        // RpcTryExcept: the guarded body is executed directly; the RpcExcept
        dwErrorCode = rpc_try(|| _RpcGetForm((*pHandle).hPrinter, pFormName, Level, pForm, cbBuf, pcbNeeded)).unwrap_or_else(|code| code as _);
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructure(
                cbBuf,
                pForm as PVOID,
                core::ptr::addr_of!((*pFormInfoMarshalling[Level as usize]).pInfo) as *const MARSHALLING_INFO,
                (*pFormInfoMarshalling[Level as usize]).cbStructureSize,
                TRUE as BOOL
            );
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    (dwErrorCode == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetFormA(hPrinter: HANDLE, pFormName: PSTR, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut pfi2W: FORM_INFO_2W = core::mem::zeroed();
    let mut pfi2A: *mut FORM_INFO_2A;
    let mut FormNameW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut res: BOOL;

    pfi2A = pForm as *mut FORM_INFO_2A;

    if (Level < 1) || (Level > 2) {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    if pfi2A.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if !pFormName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pFormName, -1, core::ptr::null_mut(), 0) as DWORD;
        FormNameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pFormName, -1, FormNameW, len as i32);
    }

    ZeroMemory!(core::ptr::addr_of_mut!(pfi2W) as *mut core::ffi::c_void, core::mem::size_of::<FORM_INFO_2W>());

    if !(*pfi2A).pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pName, -1, core::ptr::null_mut(), 0) as DWORD;
        pfi2W.pName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pName, -1, pfi2W.pName, len as i32);
    }

    pfi2W.Flags = (*pfi2A).Flags;
    pfi2W.Size = (*pfi2A).Size;
    pfi2W.ImageableArea = (*pfi2A).ImageableArea;

    if Level > 1 {
        if !(*pfi2A).pKeyword.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pKeyword, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pKeyword = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as *const i8;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pKeyword, -1, pfi2W.pKeyword as LPWSTR, len as i32);
        }

        if !(*pfi2A).pMuiDll.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pMuiDll, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pMuiDll = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pMuiDll, -1, pfi2W.pMuiDll as LPWSTR, len as i32);
        }

        if !(*pfi2A).pDisplayName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pDisplayName, -1, core::ptr::null_mut(), 0) as DWORD;
            pfi2W.pDisplayName = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, (*pfi2A).pDisplayName, -1, pfi2W.pDisplayName as LPWSTR, len as i32);
        }
        pfi2W.StringType = (*pfi2A).StringType;
        pfi2W.dwResourceId = (*pfi2A).dwResourceId;
        pfi2W.wLangId = (*pfi2A).wLangId;
    }

    res = SetFormW(hPrinter, FormNameW, Level, core::ptr::addr_of_mut!(pfi2W) as PBYTE);

    if !FormNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, FormNameW as *mut core::ffi::c_void);
    }
    if !pfi2W.pName.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pName as *mut core::ffi::c_void);
    }
    if !pfi2W.pKeyword.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pKeyword as *mut core::ffi::c_void);
    }
    if !pfi2W.pMuiDll.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pMuiDll as *mut core::ffi::c_void);
    }
    if !pfi2W.pDisplayName.is_null() {
        HeapFree(GetProcessHeap(), 0, pfi2W.pDisplayName as *mut core::ffi::c_void);
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn SetFormW(hPrinter: HANDLE, pFormName: PWSTR, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut FormInfoContainer: WINSPOOL_FORM_CONTAINER = core::mem::zeroed();
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE;
        return FALSE as BOOL;
    }

    FormInfoContainer.FormInfo.pFormInfo1 = pForm as *mut WINSPOOL_FORM_INFO_1;
    FormInfoContainer.Level = Level;

    // Do the RPC call
    // RpcTryExcept: the protected body runs directly; SEH wrapping is owned by integration.
    {
        dwErrorCode = rpc_try(|| _RpcSetForm((*pHandle).hPrinter, pFormName, core::ptr::addr_of_mut!(FormInfoContainer))).unwrap_or_else(|code| code as _);
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler body preserved; executed only on exception
    // (exception dispatch is owned by integration).
    // RpcEndExcept;

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
