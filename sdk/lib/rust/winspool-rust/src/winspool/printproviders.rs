use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddPrintProvidorA(pName: PSTR, Level: DWORD, pProviderInfo: PBYTE) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut pi1W: PROVIDOR_INFO_1W = core::mem::zeroed();
    let mut pi2W: PROVIDOR_INFO_2W = core::mem::zeroed();
    let mut len: DWORD = 0;
    let mut res: BOOL = 0;
    let mut pPI: PBYTE = core::ptr::null_mut();

    ZeroMemory!(core::ptr::addr_of_mut!(pi1W) as *mut core::ffi::c_void, (core::mem::size_of::<PROVIDOR_INFO_1W>()) as usize);
    pi2W.pOrder = core::ptr::null_mut();

    match Level {
        v if v == 1 => {
            let pi1A: *mut PROVIDOR_INFO_1A = pProviderInfo as *mut PROVIDOR_INFO_1A;
            if !(*pi1A).pName.is_null() {
                len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pName as LPCSTR, -(1) as ::core::ffi::c_int, core::ptr::null_mut(), 0 as ::core::ffi::c_int) as DWORD;
                pi1W.pName = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pName as LPCSTR, -(1) as ::core::ffi::c_int, pi1W.pName, len as ::core::ffi::c_int);
            }
            if !(*pi1A).pEnvironment.is_null() {
                len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pEnvironment as LPCSTR, -(1) as ::core::ffi::c_int, core::ptr::null_mut(), 0 as ::core::ffi::c_int) as DWORD;
                pi1W.pEnvironment = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pEnvironment as LPCSTR, -(1) as ::core::ffi::c_int, pi1W.pEnvironment, len as ::core::ffi::c_int);
            }
            if !(*pi1A).pDLLName.is_null() {
                len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pDLLName as LPCSTR, -(1) as ::core::ffi::c_int, core::ptr::null_mut(), 0 as ::core::ffi::c_int) as DWORD;
                pi1W.pDLLName = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi1A).pDLLName as LPCSTR, -(1) as ::core::ffi::c_int, pi1W.pDLLName, len as ::core::ffi::c_int);
            }
            pPI = core::ptr::addr_of_mut!(pi1W) as PBYTE;
        }

        v if v == 2 => {
            let pi2A: *mut PROVIDOR_INFO_2A = pProviderInfo as *mut PROVIDOR_INFO_2A;
            if !(*pi2A).pOrder.is_null() {
                len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi2A).pOrder as LPCSTR, -(1) as ::core::ffi::c_int, core::ptr::null_mut(), 0 as ::core::ffi::c_int) as DWORD;
                pi2W.pOrder = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, (*pi2A).pOrder as LPCSTR, -(1) as ::core::ffi::c_int, pi2W.pOrder, len as ::core::ffi::c_int);
            }
            pPI = core::ptr::addr_of_mut!(pi2W) as PBYTE;
        }

        _ => {
            SetLastError(ERROR_INVALID_LEVEL as DWORD);
            return FALSE;
        }
    }

    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, pName as LPCSTR, -(1) as ::core::ffi::c_int, core::ptr::null_mut(), 0 as ::core::ffi::c_int) as DWORD;
        nameW = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, pName as LPCSTR, -(1) as ::core::ffi::c_int, nameW, len as ::core::ffi::c_int);
    }

    res = AddPrintProvidorW(nameW as PWSTR, Level, pPI) as BOOL;

    if !pName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, nameW as PVOID);
    }
    if !pi1W.pName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pi1W.pName as PVOID);
    }
    if !pi1W.pEnvironment.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pi1W.pEnvironment as PVOID);
    }
    if !pi1W.pDLLName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pi1W.pDLLName as PVOID);
    }
    if !pi2W.pOrder.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pi2W.pOrder as PVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrintProvidorW(pName: PWSTR, Level: DWORD, pProviderInfo: PBYTE) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut ProvidorContainer: WINSPOOL_PROVIDOR_CONTAINER = _WINSPOOL_PROVIDOR_CONTAINER {
        Level: 0,
        ProvidorInfo: _WINSPOOL_PROVIDOR_CONTAINER__bindgen_ty_1 {
            pProvidorInfo1: core::ptr::null_mut::<WINSPOOL_PROVIDOR_INFO_1>(),
        },
    };
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printproviders.c\0"
                as *const u8 as *const i8,
            b"AddPrintProvidorW\0" as *const u8 as *const i8,
            94 as i32,
            b"AddPrintProvidorW(%S, %lu, %p)\n\0" as *const u8 as *const i8,
            pName,
            Level,
            pProviderInfo,
        );
    }
    if Level < 1 || Level > 2 {
        SetLastError(ERROR_INVALID_LEVEL as DWORD);
        return FALSE;
    }
    ProvidorContainer.ProvidorInfo.pProvidorInfo1 = pProviderInfo as *mut WINSPOOL_PROVIDOR_INFO_1;
    ProvidorContainer.Level = Level;
    dwErrorCode = _RpcAddPrintProvidor(
        pName as WINSPOOL_HANDLE,
        core::ptr::addr_of_mut!(ProvidorContainer),
    );
    if _SEH2_Volatile0 != 0 {
        dwErrorCode = RpcExceptionCode!() as DWORD;
        if (*__wine_dbch___default).flags as i32
            & (1 as i32) << __WINE_DBCL_ERR as i32
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/printing/base/winspool/printproviders.c\0"
                    as *const u8 as *const i8,
                b"AddPrintProvidorW\0" as *const u8 as *const i8,
                112 as i32,
                b"_RpcEnumPorts failed with exception code %lu!\n\0" as *const u8 as *const i8,
                dwErrorCode,
            );
        }
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProvidorA(mut pName: PSTR, mut pEnvironment: PSTR, mut pPrintProviderName: PSTR) -> WINBOOL {
    let mut NameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut EnvW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ProviderW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Ret: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printproviders.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DeletePrintProvidorA\0" as *const u8 as *const ::core::ffi::c_char,
            126 as ::core::ffi::c_int,
            b"DeletePrintProvidorW(%s, %s, %s)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pName,
            pEnvironment,
            pPrintProviderName,
        );
    }

    AsciiToUnicode(core::ptr::addr_of_mut!(NameW), pName as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(EnvW), pEnvironment as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(ProviderW), pPrintProviderName as LPCSTR);

    Ret = DeletePrintProvidorW(NameW.Buffer, EnvW.Buffer, ProviderW.Buffer);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ProviderW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(EnvW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProvidorW(mut pName: PWSTR, mut pEnvironment: PWSTR, mut pPrintProviderName: PWSTR) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printproviders.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DeletePrintProvidorW\0" as *const u8 as *const ::core::ffi::c_char,
            146 as ::core::ffi::c_int,
            b"DeletePrintProvidorW(%S, %S, %S)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pName,
            pEnvironment,
            pPrintProviderName,
        );
    }
    dwErrorCode = _RpcDeletePrintProvidor(
        pName as WINSPOOL_HANDLE,
        pEnvironment as *mut WCHAR,
        pPrintProviderName as *mut WCHAR,
    );
    if _SEH2_Volatile0 != 0 || 0 as ::core::ffi::c_int != 0 && 1 as ::core::ffi::c_int != 0 {
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
                b"win32ss/printing/base/winspool/printproviders.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DeletePrintProvidorW\0" as *const u8 as *const ::core::ffi::c_char,
                155 as ::core::ffi::c_int,
                b"_RpcEnumPorts failed with exception code %lu!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                dwErrorCode,
            );
        }
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == (ERROR_SUCCESS as DWORD)) as BOOL;
}
