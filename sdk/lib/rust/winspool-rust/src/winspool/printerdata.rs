// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
pub static RUSTCONV_WSTR_05CF8A9FE74B: [u16; 18] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0044, 0x0072, 0x0069,
    0x0076, 0x0065, 0x0072, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32) -> i32;
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut BOOL) -> i32;
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedSetupDialog(
    hWnd: HWND,
    _Unknown: INT,
    pDevModeInput: PDEVMODEA,
    pDevModeOutput: PDEVMODEA,
) -> LONG {
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut Ret: LONG = -1;

    if OpenPrinterA(
        core::ptr::addr_of_mut!((*pDevModeInput).dmDeviceName) as *mut BYTE as LPSTR,
        core::ptr::addr_of_mut!(hPrinter),
        core::ptr::null_mut(),
    ) != 0
    {
        Ret = AdvancedDocumentPropertiesA(
            hWnd,
            hPrinter,
            core::ptr::addr_of_mut!((*pDevModeInput).dmDeviceName) as *mut BYTE as PSTR,
            pDevModeOutput,
            pDevModeInput,
        );
        ClosePrinter(hPrinter);
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedDocumentPropertiesA(
    hWnd: HWND,
    hPrinter: HANDLE,
    pDeviceName: PSTR,
    pDevModeOutput: PDEVMODEA,
    pDevModeInput: PDEVMODEA,
) -> LONG {
    // TRACE("AdvancedDocumentPropertiesA(%p, %p, %s, %p, %p)\n", hWnd, hPrinter, pDeviceName, pDevModeOutput, pDevModeInput);
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedDocumentPropertiesW(
    hWnd: HWND,
    hPrinter: HANDLE,
    pDeviceName: PWSTR,
    pDevModeOutput: PDEVMODEW,
    pDevModeInput: PDEVMODEW,
) -> LONG {
    // TRACE("AdvancedDocumentPropertiesW(%p, %p, %S, %p, %p)\n", hWnd, hPrinter, pDeviceName, pDevModeOutput, pDevModeInput);
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataA(hPrinter: HANDLE, pValueName: PSTR) -> DWORD {
    let mut valuenameW: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: INT = 0;
    let mut res: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printerdata.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DeletePrinterDataA\0" as *const u8 as *const ::core::ffi::c_char,
            49 as ::core::ffi::c_int,
            b"DeletePrinterDataA(%p, %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hPrinter,
            pValueName,
        );
    }
    if !pValueName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pValueName as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        valuenameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pValueName as LPCSTR,
            -(1 as ::core::ffi::c_int),
            valuenameW,
            len as ::core::ffi::c_int,
        );
    }
    res = DeletePrinterDataW(hPrinter, valuenameW as PWSTR);
    if !valuenameW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            valuenameW as PVOID,
        );
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataExA(hPrinter: HANDLE, pKeyName: PCSTR, pValueName: PCSTR) -> DWORD {
    let mut keynameW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut valuenameW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut len: INT = 0;
    let mut res: DWORD = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printerdata.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DeletePrinterDataExA\0" as *const u8 as *const core::ffi::c_char,
            74 as core::ffi::c_int,
            b"DeletePrinterDataExA(%p, %s, %s)\n\0" as *const u8
                as *const core::ffi::c_char,
            hPrinter,
            pKeyName,
            pValueName,
        );
    }
    if !pKeyName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pKeyName as LPCSTR,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as core::ffi::c_int,
        ) as INT;
        keynameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pKeyName as LPCSTR,
            -(1 as core::ffi::c_int),
            keynameW,
            len as core::ffi::c_int,
        );
    }
    if !pValueName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pValueName as LPCSTR,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as core::ffi::c_int,
        ) as INT;
        valuenameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pValueName as LPCSTR,
            -(1 as core::ffi::c_int),
            valuenameW,
            len as core::ffi::c_int,
        );
    }
    res = DeletePrinterDataExW(hPrinter, keynameW as PCWSTR, valuenameW as PCWSTR);
    if !keynameW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            keynameW as PVOID,
        );
    }
    if !valuenameW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            valuenameW as PVOID,
        );
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataExW(
    hPrinter: HANDLE,
    pKeyName: PCWSTR,
    pValueName: PCWSTR,
) -> DWORD {
    // TRACE("DeletePrinterDataExW(%p, %S, %S)\n", hPrinter, pKeyName, pValueName);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataW(hPrinter: HANDLE, pValueName: PWSTR) -> DWORD {
    // TRACE("DeletePrinterDataW(%p, %S)\n", hPrinter, pValueName);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterKeyA(hPrinter: HANDLE, pKeyName: PCSTR) -> DWORD {
    let mut keynameW: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: INT;
    let mut res: DWORD;

    if !pKeyName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pKeyName as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        keynameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pKeyName as LPCSTR,
            -(1 as ::core::ffi::c_int),
            keynameW,
            len as ::core::ffi::c_int,
        );
    }

    res = DeletePrinterKeyW(hPrinter, keynameW as PCWSTR);

    if !keynameW.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            keynameW as PVOID,
        );
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterKeyW(hPrinter: HANDLE, pKeyName: PCWSTR) -> DWORD {
    // TRACE("DeletePrinterKeyW(%p, %S)\n", hPrinter, pKeyName);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataA(
    hPrinter: HANDLE,
    dwIndex: DWORD,
    pValueName: PSTR,
    cbValueName: DWORD,
    pcbValueName: PDWORD,
    pType: PDWORD,
    pData: PBYTE,
    cbData: DWORD,
    pcbData: PDWORD,
) -> DWORD {
    // TRACE("EnumPrinterDataA(%p, %lu, %s, %lu, %p, %p, %p, %lu, %p)\n", hPrinter, dwIndex, pValueName, cbValueName, pcbValueName, pType, pData, cbData, pcbData);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataExA(hPrinter: HANDLE, pKeyName: PCSTR, pEnumValues: PBYTE, cbEnumValues: DWORD, pcbEnumValues: PDWORD, pnEnumValues: PDWORD) -> DWORD {
    let mut len: INT;
    let mut pKeyNameW: LPWSTR;
    let mut ret: DWORD;
    let mut dwIndex: DWORD;
    let mut dwBufSize: DWORD;
    let mut hHeap: HANDLE;
    let mut pBuffer: LPSTR;
    let file = b"win32ss/printing/base/winspool/printerdata.c\0".as_ptr() as *const ::core::ffi::c_char;
    let func = b"EnumPrinterDataExA\0".as_ptr() as *const ::core::ffi::c_char;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            file,
            func,
            162 as ::core::ffi::c_int,
            b"EnumPrinterDataExA(%p, %s, %p, %lu, %p, %p)\n\0".as_ptr() as *const ::core::ffi::c_char,
            hPrinter,
            pKeyName,
            pEnumValues,
            cbEnumValues,
            pcbEnumValues,
            pnEnumValues,
        );
    }

    if pKeyName.is_null() || (*pKeyName as ::core::ffi::c_int) == 0 {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pKeyName as LPCSTR,
        -(1),
        ::core::ptr::null_mut::<WCHAR>(),
        0,
    ) as INT;
    if len == 0 {
        ret = GetLastError();
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                171 as ::core::ffi::c_int,
                b"MultiByteToWideChar failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                ret,
            );
        }
        return ret;
    }

    hHeap = GetProcessHeap();
    if hHeap.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                178 as ::core::ffi::c_int,
                b"GetProcessHeap failed\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return ERROR_OUTOFMEMORY as DWORD;
    }

    pKeyNameW = HeapAlloc(
        hHeap,
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pKeyNameW.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                186 as ::core::ffi::c_int,
                b"Failed to allocate %i bytes from process heap\n\0".as_ptr() as *const ::core::ffi::c_char,
                (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as LONG,
            );
        }
        return ERROR_OUTOFMEMORY as DWORD;
    }

    if MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pKeyName as LPCSTR,
        -(1),
        pKeyNameW,
        len as ::core::ffi::c_int,
    ) == 0
    {
        ret = GetLastError();
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                193 as ::core::ffi::c_int,
                b"MultiByteToWideChar failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                ret,
            );
        }
        if HeapFree(hHeap, 0 as DWORD, pKeyNameW as PVOID) == 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int) != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    file,
                    func,
                    195 as ::core::ffi::c_int,
                    b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                    GetLastError(),
                );
            }
        }
        return ret;
    }

    ret = EnumPrinterDataExW(hPrinter, pKeyNameW as PCWSTR, pEnumValues, cbEnumValues, pcbEnumValues, pnEnumValues);

    if ret != ERROR_SUCCESS as DWORD {
        if HeapFree(hHeap, 0 as DWORD, pKeyNameW as PVOID) == 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int) != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    file,
                    func,
                    204 as ::core::ffi::c_int,
                    b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                    GetLastError(),
                );
            }
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                file,
                func,
                205 as ::core::ffi::c_int,
                b"EnumPrinterDataExW returned %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                ret,
            );
        }
        return ret;
    }

    if HeapFree(hHeap, 0 as DWORD, pKeyNameW as PVOID) == 0 {
        ret = GetLastError();
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                212 as ::core::ffi::c_int,
                b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                ret,
            );
        }
        return ret;
    }

    if *pnEnumValues == 0 as DWORD {
        return ERROR_SUCCESS as DWORD;
    }

    dwBufSize = 0 as DWORD;
    dwIndex = 0 as DWORD;
    while dwIndex < *pnEnumValues {
        let ppev: PPRINTER_ENUM_VALUESW = (pEnumValues as PPRINTER_ENUM_VALUESW)
            .offset(dwIndex as isize) as PPRINTER_ENUM_VALUESW;

        if dwBufSize < (*ppev).cbValueName {
            dwBufSize = (*ppev).cbValueName;
        }

        if dwBufSize < (*ppev).cbData
            && ((*ppev).dwType == REG_SZ as DWORD
                || (*ppev).dwType == REG_EXPAND_SZ as DWORD
                || (*ppev).dwType == REG_MULTI_SZ as DWORD)
        {
            dwBufSize = (*ppev).cbData;
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int) != 0
    {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            file,
            func,
            232 as ::core::ffi::c_int,
            b"Largest Unicode name or value is %i bytes\n\0".as_ptr() as *const ::core::ffi::c_char,
            dwBufSize,
        );
    }

    pBuffer = HeapAlloc(hHeap, 0 as DWORD, dwBufSize as SIZE_T) as LPSTR;
    if pBuffer.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                237 as ::core::ffi::c_int,
                b"Failed to allocate %i bytes from process heap\n\0".as_ptr() as *const ::core::ffi::c_char,
                dwBufSize,
            );
        }
        return ERROR_OUTOFMEMORY as DWORD;
    }

    dwIndex = 0 as DWORD;
    while dwIndex < *pnEnumValues {
        let ppev_0: PPRINTER_ENUM_VALUESW = (pEnumValues as PPRINTER_ENUM_VALUESW)
            .offset(dwIndex as isize) as PPRINTER_ENUM_VALUESW;

        len = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            (*ppev_0).pValueName as LPCWSTR,
            ((*ppev_0).cbValueName as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
            pBuffer,
            dwBufSize as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) as INT;
        if len == 0 {
            ret = GetLastError();
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    file,
                    func,
                    252 as ::core::ffi::c_int,
                    b"WideCharToMultiByte failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                    ret,
                );
            }
            if HeapFree(hHeap, 0 as DWORD, pBuffer as PVOID) == 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int) != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_WARN,
                        __wine_dbch___default,
                        file,
                        func,
                        254 as ::core::ffi::c_int,
                        b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
            }
            return ret;
        }

        memcpy((*ppev_0).pValueName as *mut ::core::ffi::c_void, pBuffer as *const ::core::ffi::c_void, len as usize);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                file,
                func,
                260 as ::core::ffi::c_int,
                b"Converted '%s' from Unicode to ASCII\n\0".as_ptr() as *const ::core::ffi::c_char,
                pBuffer,
            );
        }

        'body: {
            if (*ppev_0).dwType != REG_SZ as DWORD
                && (*ppev_0).dwType != REG_EXPAND_SZ as DWORD
                && (*ppev_0).dwType != REG_MULTI_SZ as DWORD
            {
                break 'body;
            }

            len = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*ppev_0).pData as LPWSTR as LPCWSTR,
                ((*ppev_0).cbData as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
                pBuffer,
                dwBufSize as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) as INT;
            if len == 0 {
                ret = GetLastError();
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        file,
                        func,
                        271 as ::core::ffi::c_int,
                        b"WideCharToMultiByte failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                        ret,
                    );
                }
                if HeapFree(hHeap, 0 as DWORD, pBuffer as PVOID) == 0 {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int) != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_WARN,
                            __wine_dbch___default,
                            file,
                            func,
                            273 as ::core::ffi::c_int,
                            b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                            GetLastError(),
                        );
                    }
                }
                return ret;
            }

            memcpy((*ppev_0).pData as *mut ::core::ffi::c_void, pBuffer as *const ::core::ffi::c_void, len as usize);

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    file,
                    func,
                    279 as ::core::ffi::c_int,
                    b"Converted '%s' from Unicode to ASCII\n\0".as_ptr() as *const ::core::ffi::c_char,
                    pBuffer,
                );
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int) != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    file,
                    func,
                    280 as ::core::ffi::c_int,
                    b"  (only first string of REG_MULTI_SZ printed)\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    if HeapFree(hHeap, 0 as DWORD, pBuffer as PVOID) == 0 {
        ret = GetLastError();
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int) != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                file,
                func,
                286 as ::core::ffi::c_int,
                b"HeapFree failed with code %i\n\0".as_ptr() as *const ::core::ffi::c_char,
                ret,
            );
        }
        return ret;
    }

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataExW(hPrinter: HANDLE, pKeyName: PCWSTR, pEnumValues: PBYTE, cbEnumValues: DWORD, pcbEnumValues: PDWORD, pnEnumValues: PDWORD) -> DWORD {
    // TRACE("EnumPrinterDataExW(%p, %S, %p, %lu, %p, %p)\n", hPrinter, pKeyName, pEnumValues, cbEnumValues, pcbEnumValues, pnEnumValues);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataW(
    hPrinter: HANDLE,
    dwIndex: DWORD,
    pValueName: PWSTR,
    cbValueName: DWORD,
    pcbValueName: PDWORD,
    pType: PDWORD,
    pData: PBYTE,
    cbData: DWORD,
    pcbData: PDWORD,
) -> DWORD {
    // TRACE("EnumPrinterDataW(%p, %lu, %S, %lu, %p, %p, %p, %lu, %p)\n", hPrinter, dwIndex, pValueName, cbValueName, pcbValueName, pType, pData, cbData, pcbData);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterKeyA(
    hPrinter: HANDLE,
    pKeyName: PCSTR,
    pSubkey: PSTR,
    cbSubkey: DWORD,
    pcbSubkey: PDWORD,
) -> DWORD {
    // TRACE("EnumPrinterKeyA(%p, %s, %s, %lu, %p)\n", hPrinter, pKeyName, pSubkey, cbSubkey, pcbSubkey);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterKeyW(
    hPrinter: HANDLE,
    pKeyName: PCWSTR,
    pSubkey: PWSTR,
    cbSubkey: DWORD,
    pcbSubkey: PDWORD,
) -> DWORD {
    // TRACE("EnumPrinterKeyW(%p, %S, %S, %lu, %p)\n", hPrinter, pKeyName, pSubkey, cbSubkey, pcbSubkey);
    // UNIMPLEMENTED;
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataA(hPrinter: HANDLE, pValueName: LPSTR, pType: LPDWORD, pData: LPBYTE, nSize: DWORD, pcbNeeded: LPDWORD) -> DWORD {
    return GetPrinterDataExA(hPrinter, b"PrinterDriverData\0".as_ptr() as LPCSTR, pValueName as LPCSTR, pType, pData, nSize, pcbNeeded);
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataExA(
    hPrinter: HANDLE,
    pKeyName: LPCSTR,
    pValueName: LPCSTR,
    mut pType: LPDWORD,
    pData: LPBYTE,
    nSize: DWORD,
    pcbNeeded: LPDWORD,
) -> DWORD {
    let mut cbUnicodeData: DWORD = 0;
    let mut cch: DWORD = 0;
    let mut dwReturnValue: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut pInfoA: POSVERSIONINFOEXA = core::ptr::null_mut();
    let mut pInfoW: POSVERSIONINFOEXW = core::ptr::null_mut();
    let mut pUnicodeData: PVOID = core::ptr::null_mut();
    let mut pwszKeyName: PWSTR = core::ptr::null_mut();
    let mut pwszValueName: PWSTR = core::ptr::null_mut();

    'cleanup: {
        if !pKeyName.is_null() {
            // Convert pKeyName to a Unicode string pwszKeyName
            cch = strlen(pKeyName as *const i8) as DWORD;

            pwszKeyName = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszKeyName.is_null() {
                dwReturnValue = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'cleanup;
            }

            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                pKeyName,
                -(1 as i32),
                pwszKeyName as LPWSTR,
                (cch + 1) as i32,
            );
        }

        if !pValueName.is_null() {
            // Convert pValueName to a Unicode string pwszValueName
            cch = strlen(pValueName as *const i8) as DWORD;

            pwszValueName = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszValueName.is_null() {
                dwReturnValue = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'cleanup;
            }

            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                pValueName,
                -(1 as i32),
                pwszValueName as LPWSTR,
                (cch + 1) as i32,
            );
        }

        // We need the data type information, even if no pData was passed.
        if pType.is_null() {
            pType = core::ptr::addr_of_mut!(dwType);
        }

        // Call GetPrinterDataExW for the first time.
        dwReturnValue = GetPrinterDataExW(
            hPrinter,
            pwszKeyName as LPCWSTR,
            pwszValueName as LPCWSTR,
            pType,
            pData,
            nSize,
            pcbNeeded,
        );

        // If a critical error occurred, just return it.
        if dwReturnValue != ERROR_SUCCESS as DWORD
            && dwReturnValue != ERROR_MORE_DATA as DWORD
        {
            break 'cleanup;
        }

        // Save the needed buffer size for the Unicode data.
        cbUnicodeData = *pcbNeeded;

        if *pType == REG_SZ as DWORD
            || *pType == REG_MULTI_SZ as DWORD
            || *pType == REG_EXPAND_SZ as DWORD
        {
            // Output the required buffer size for the ANSI string.
            *pcbNeeded = ((*pcbNeeded as usize) / core::mem::size_of::<WCHAR>()) as DWORD;
        } else if *pType == REG_NONE as DWORD {
            if cbUnicodeData as usize == core::mem::size_of::<OSVERSIONINFOW>()
                && _wcsicmp(
                    pwszValueName as *const u16,
                    SPLREG_OS_VERSION,
                ) == 0
            {
                *pcbNeeded = core::mem::size_of::<OSVERSIONINFOA>() as DWORD;
            } else if cbUnicodeData as usize == core::mem::size_of::<OSVERSIONINFOEXW>()
                && _wcsicmp(
                    pwszValueName as *const u16,
                    SPLREG_OS_VERSIONEX,
                ) == 0
            {
                *pcbNeeded = core::mem::size_of::<OSVERSIONINFOEXA>() as DWORD;
            } else {
                // Other REG_NONE value, nothing to do.
                break 'cleanup;
            }
        }

        // Check if the supplied buffer is large enough for the ANSI data.
        if nSize < *pcbNeeded {
            dwReturnValue = ERROR_MORE_DATA as DWORD;
            break 'cleanup;
        }

        // Allocate a temporary buffer for the Unicode data.
        pUnicodeData = HeapAlloc(
            hProcessHeap,
            0,
            cbUnicodeData as SIZE_T,
        );
        if pUnicodeData.is_null() {
            dwReturnValue = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'cleanup;
        }

        if dwReturnValue == ERROR_SUCCESS as DWORD {
            // ERROR_SUCCESS: copy the Unicode data from pData to pUnicodeData.
            CopyMemory!(pUnicodeData, pData as *const core::ffi::c_void, cbUnicodeData as usize);
        } else {
            // ERROR_MORE_DATA: call GetPrinterDataExW again with the temporary buffer.
            dwReturnValue = GetPrinterDataExW(
                hPrinter,
                pwszKeyName as LPCWSTR,
                pwszValueName as LPCWSTR,
                core::ptr::null_mut::<DWORD>(),
                pUnicodeData as LPBYTE,
                cbUnicodeData,
                core::ptr::addr_of_mut!(cbUnicodeData),
            );
            if dwReturnValue != ERROR_SUCCESS as DWORD {
                break 'cleanup;
            }
        }

        if *pType == REG_SZ as DWORD
            || *pType == REG_MULTI_SZ as DWORD
            || *pType == REG_EXPAND_SZ as DWORD
        {
            // Convert the Unicode string to ANSI.
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                pUnicodeData as PWSTR as LPCWSTR,
                -(1 as i32),
                pData as LPSTR,
                *pcbNeeded as i32,
                core::ptr::null::<i8>(),
                core::ptr::null_mut::<BOOL>(),
            );
        } else {
            // REG_NONE with either OSVERSIONINFOW or OSVERSIONINFOEXW.
            pInfoW = pUnicodeData as POSVERSIONINFOEXW;
            pInfoA = pData as POSVERSIONINFOEXA;
            (*pInfoA).dwMajorVersion = (*pInfoW).dwMajorVersion;
            (*pInfoA).dwMinorVersion = (*pInfoW).dwMinorVersion;
            (*pInfoA).dwBuildNumber = (*pInfoW).dwBuildNumber;
            (*pInfoA).dwPlatformId = (*pInfoW).dwPlatformId;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*pInfoW).szCSDVersion) as LPCWSTR,
                -(1 as i32),
                core::ptr::addr_of_mut!((*pInfoA).szCSDVersion) as LPSTR,
                core::mem::size_of::<[i8; 128]>() as i32,
                core::ptr::null::<i8>(),
                core::ptr::null_mut::<BOOL>(),
            );

            if cbUnicodeData as usize == core::mem::size_of::<OSVERSIONINFOW>() {
                (*pInfoA).dwOSVersionInfoSize =
                    core::mem::size_of::<OSVERSIONINFOA>() as DWORD;
            } else {
                (*pInfoA).dwOSVersionInfoSize =
                    core::mem::size_of::<OSVERSIONINFOEXA>() as DWORD;
                (*pInfoA).wServicePackMajor = (*pInfoW).wServicePackMajor;
                (*pInfoA).wServicePackMinor = (*pInfoW).wServicePackMinor;
                (*pInfoA).wSuiteMask = (*pInfoW).wSuiteMask;
                (*pInfoA).wProductType = (*pInfoW).wProductType;
                (*pInfoA).wReserved = (*pInfoW).wReserved;
            }
        }
    }

    // Cleanup:
    if !pwszKeyName.is_null() {
        HeapFree(
            hProcessHeap,
            0 as DWORD,
            pwszKeyName as PVOID,
        );
    }

    if !pwszValueName.is_null() {
        HeapFree(
            hProcessHeap,
            0 as DWORD,
            pwszValueName as PVOID,
        );
    }

    if !pUnicodeData.is_null() {
        HeapFree(
            hProcessHeap,
            0 as DWORD,
            pUnicodeData,
        );
    }

    return dwReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataExW(mut hPrinter: HANDLE, mut pKeyName: LPCWSTR, mut pValueName: LPCWSTR, mut pType: LPDWORD, mut pData: LPBYTE, mut nSize: DWORD, mut pcbNeeded: LPDWORD) -> DWORD {
    let wszEmptyString: [u16; 1] = [0x0000u16];
    let mut DummyData: BYTE = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwType: DWORD = REG_NONE;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    // Sanity checks
    if pHandle.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if pKeyName.is_null() {
        pKeyName = &raw const wszEmptyString as *const WCHAR as LPCWSTR;
    }

    if pType.is_null() {
        pType = &raw mut dwType as LPDWORD;
    }

    if pData.is_null() && nSize == 0 {
        pData = &raw mut DummyData as LPBYTE;
    }

    // Do the RPC call
    dwErrorCode = rpc_try(|| _RpcGetPrinterDataEx(
        (*pHandle).hPrinter as WINSPOOL_PRINTER_HANDLE,
        pKeyName as *const WCHAR,
        pValueName as *const WCHAR,
        pType as *mut DWORD,
        pData as *mut BYTE,
        nSize,
        pcbNeeded as *mut DWORD,
    )).unwrap_or_else(|code| code as _);

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataW(hPrinter: HANDLE, pValueName: LPWSTR, pType: LPDWORD, pData: LPBYTE, nSize: DWORD, pcbNeeded: LPDWORD) -> DWORD {
    return GetPrinterDataExW(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr(), pValueName, pType, pData, nSize, pcbNeeded);
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataA(hPrinter: HANDLE, pValueName: PSTR, Type: DWORD, pData: PBYTE, cbData: DWORD) -> DWORD {
    return SetPrinterDataExA(hPrinter, b"PrinterDriverData\0".as_ptr() as LPCSTR, pValueName as LPCSTR, Type, pData as LPBYTE, cbData);
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataExA(
    hPrinter: HANDLE,
    pKeyName: LPCSTR,
    pValueName: LPCSTR,
    mut Type: DWORD,
    mut pData: LPBYTE,
    mut cbData: DWORD,
) -> DWORD {
    let mut cch: DWORD = 0;
    let mut dwReturnValue: DWORD = 0;
    let mut pwszKeyName: PWSTR = core::ptr::null_mut();
    let mut pwszValueName: PWSTR = core::ptr::null_mut();
    let mut pUnicodeData: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        if !pKeyName.is_null() {
            // Convert pKeyName to a Unicode string pwszKeyName
            cch = strlen(pKeyName as *const i8) as DWORD;

            pwszKeyName = HeapAlloc(
                hProcessHeap,
                0,
                (cch.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszKeyName.is_null() {
                dwReturnValue = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pKeyName,
                -1,
                pwszKeyName as LPWSTR,
                cch.wrapping_add(1) as i32,
            );
        }

        if !pValueName.is_null() {
            // Convert pValueName to a Unicode string pwszValueName
            cch = strlen(pValueName as *const i8) as DWORD;

            pwszValueName = HeapAlloc(
                hProcessHeap,
                0,
                (cch.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszValueName.is_null() {
                dwReturnValue = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pValueName,
                -1,
                pwszValueName as LPWSTR,
                cch.wrapping_add(1) as i32,
            );
        }

        if Type == REG_SZ || Type == REG_MULTI_SZ || Type == REG_EXPAND_SZ {
            // Convert pData to a Unicode string pUnicodeData.
            pUnicodeData = HeapAlloc(
                hProcessHeap,
                0,
                cbData.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as SIZE_T,
            ) as PWSTR;
            if pUnicodeData.is_null() {
                dwReturnValue = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pData as PCSTR,
                -1,
                pUnicodeData as LPWSTR,
                cbData as i32,
            );

            pData = pUnicodeData as PBYTE as LPBYTE;
            cbData = cbData.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        dwReturnValue = SetPrinterDataExW(
            hPrinter,
            pwszKeyName as LPCWSTR,
            pwszValueName as LPCWSTR,
            Type,
            pData,
            cbData,
        );
    }

    // Cleanup:
    if !pwszKeyName.is_null() {
        HeapFree(hProcessHeap, 0, pwszKeyName as PVOID);
    }

    if !pwszValueName.is_null() {
        HeapFree(hProcessHeap, 0, pwszValueName as PVOID);
    }

    if !pUnicodeData.is_null() {
        HeapFree(hProcessHeap, 0, pUnicodeData as PVOID);
    }

    return dwReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataExW(mut hPrinter: HANDLE, mut pKeyName: LPCWSTR, mut pValueName: LPCWSTR, mut Type: DWORD, mut pData: LPBYTE, mut cbData: DWORD) -> DWORD {
    let wszEmptyString: [u16; 1] = [0x0000u16];

    let mut dwErrorCode: DWORD;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printerdata.c\0".as_ptr() as *const ::core::ffi::c_char,
            b"SetPrinterDataExW\0".as_ptr() as *const ::core::ffi::c_char,
            633 as ::core::ffi::c_int,
            b"SetPrinterDataExW(%p, %S, %S, %lu, %p, %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            hPrinter,
            pKeyName,
            pValueName,
            Type,
            pData,
            cbData,
        );
    }

    // Sanity checks
    if pHandle.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if pKeyName.is_null() {
        pKeyName = &raw const wszEmptyString as *const WCHAR as LPCWSTR;
    }

    // Do the RPC call
    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcSetPrinterDataEx(
        (*pHandle).hPrinter as WINSPOOL_PRINTER_HANDLE,
        pKeyName as *const WCHAR,
        pValueName as *const WCHAR,
        Type,
        pData as *mut BYTE,
        cbData,
    )).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    if _SEH2_Volatile0 != 0 || 0 as ::core::ffi::c_int != 0 && 1 as ::core::ffi::c_int != 0 {
        dwErrorCode = RpcExceptionCode!() as DWORD;
    }
    // RpcEndExcept;

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataW(hPrinter: HANDLE, pValueName: PWSTR, Type: DWORD, pData: PBYTE, cbData: DWORD) -> DWORD {
    return SetPrinterDataExW(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr(), pValueName as LPCWSTR, Type, pData as LPBYTE, cbData);
}
// enforce_import_abi: moved 2 import decl(s) into the extern block their
// exporting DLL's .spec requires: _wcsicmp->C, strlen->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}
