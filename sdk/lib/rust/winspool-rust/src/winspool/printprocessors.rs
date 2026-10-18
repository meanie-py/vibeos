use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub unsafe extern "system" fn AddPrintProcessorA(
    pName: PSTR,
    pEnvironment: PSTR,
    pPathName: PSTR,
    pPrintProcessorName: PSTR,
) -> BOOL {
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvW: _UNICODE_STRING = core::mem::zeroed();
    let mut PathW: _UNICODE_STRING = core::mem::zeroed();
    let mut ProcessorW: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL;

    AsciiToUnicode(core::ptr::addr_of_mut!(NameW), pName as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(EnvW), pEnvironment as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(PathW), pPathName as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(ProcessorW), pPrintProcessorName as LPCSTR);

    Ret = AddPrintProcessorW(NameW.Buffer, EnvW.Buffer, PathW.Buffer, ProcessorW.Buffer);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ProcessorW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(EnvW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrintProcessorW(
    pName: PWSTR,
    pEnvironment: PWSTR,
    pPathName: PWSTR,
    pPrintProcessorName: PWSTR,
) -> i32 {
    let mut dwErrorCode: DWORD = 0;

    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcAddPrintProcessor(
        pName as WINSPOOL_HANDLE,
        pEnvironment as *mut WCHAR,
        pPathName as *mut WCHAR,
        pPrintProcessorName as *mut WCHAR,
    )).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcEndExcept

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProcessorA(pName: PSTR, pEnvironment: PSTR, pPrintProcessorName: PSTR) -> WINBOOL {
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvW: _UNICODE_STRING = core::mem::zeroed();
    let mut ProcessorW: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printprocessors.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DeletePrintProcessorA\0" as *const u8 as *const core::ffi::c_char,
            63 as core::ffi::c_int,
            b"DeletePrintProcessorA(%s, %s, %s)\n\0" as *const u8
                as *const core::ffi::c_char,
            pName,
            pEnvironment,
            pPrintProcessorName,
        );
    }

    AsciiToUnicode(core::ptr::addr_of_mut!(NameW), pName as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(EnvW), pEnvironment as LPCSTR);
    AsciiToUnicode(core::ptr::addr_of_mut!(ProcessorW), pPrintProcessorName as LPCSTR);

    Ret = DeletePrintProcessorW(NameW.Buffer, EnvW.Buffer, ProcessorW.Buffer);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ProcessorW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(EnvW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProcessorW(mut pName: PWSTR, mut pEnvironment: PWSTR, mut pPrintProcessorName: PWSTR) -> WINBOOL {
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
            b"win32ss/printing/base/winspool/printprocessors.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DeletePrintProcessorW\0" as *const u8 as *const ::core::ffi::c_char,
            83 as ::core::ffi::c_int,
            b"DeletePrintProcessorW(%S, %S, %S)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pName,
            pEnvironment,
            pPrintProcessorName,
        );
    }
    dwErrorCode = rpc_try(|| _RpcDeletePrintProcessor(
        pName as WINSPOOL_HANDLE,
        pEnvironment as *mut WCHAR,
        pPrintProcessorName as *mut WCHAR,
    )).unwrap_or_else(|code| code as _);
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorDatatypesA(
    pName: PSTR,
    pPrintProcessorName: LPSTR,
    Level: DWORD,
    pDatatypes: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    // TRACE("EnumPrintProcessorDatatypesA(%s, %s, %lu, %p, %lu, %p, %p)\n", pName, pPrintProcessorName, Level, pDatatypes, cbBuf, pcbNeeded, pcReturned);
    // UNIMPLEMENTED;
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorDatatypesW(pName: PWSTR, pPrintProcessorName: LPWSTR, Level: DWORD, pDatatypes: PBYTE, mut cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD;
    // SEH frame state for RpcTryExcept / RpcExcept
    'Cleanup: {
        // Sanity checks
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Do the RPC call
        dwErrorCode = rpc_try(|| _RpcEnumPrintProcessorDatatypes(pName, pPrintProcessorName, Level, pDatatypes, cbBuf, pcbNeeded, pcReturned)).unwrap_or_else(|code| code as _);

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructuresArray(cbBuf, pDatatypes as PVOID, *pcReturned, DatatypesInfo1Marshalling.pInfo.as_ptr(), DatatypesInfo1Marshalling.cbStructureSize, TRUE as BOOL);
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorsA(
    pName: PSTR,
    pEnvironment: PSTR,
    Level: DWORD,
    pPrintProcessorInfo: PBYTE,
    mut cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    let mut res: BOOL;
    let mut bufferW: LPBYTE = core::ptr::null_mut();
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut envW: LPWSTR = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut numentries: DWORD = 0;
    let mut len: INT;

    /* convert names to unicode */
    if !pName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            pName as LPCSTR,
            -1,
            core::ptr::null_mut(),
            0,
        ) as INT;
        nameW = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            pName as LPCSTR,
            -1,
            nameW,
            len,
        );
    }
    if !pEnvironment.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            pEnvironment as LPCSTR,
            -1,
            core::ptr::null_mut(),
            0,
        ) as INT;
        envW = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            pEnvironment as LPCSTR,
            -1,
            envW,
            len,
        );
    }

    /* alloc (userbuffersize*sizeof(WCHAR) and try to enum the monitors */
    needed = ((cbBuf as usize) * core::mem::size_of::<WCHAR>()) as DWORD;
    if needed != 0 {
        bufferW = HeapAlloc(
            GetProcessHeap(),
            0,
            needed as SIZE_T,
        ) as LPBYTE;
    }
    res = EnumPrintProcessorsW(
        nameW as PWSTR,
        envW as PWSTR,
        Level,
        bufferW as PBYTE,
        needed,
        pcbNeeded,
        pcReturned,
    ) as BOOL;

    if res == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        if !pcbNeeded.is_null() {
            needed = *pcbNeeded;
        }
        /* HeapReAlloc return NULL, when bufferW was NULL */
        bufferW = if !bufferW.is_null() {
            HeapReAlloc(
                GetProcessHeap(),
                0,
                bufferW as PVOID,
                needed as SIZE_T,
            ) as LPBYTE
        } else {
            HeapAlloc(
                GetProcessHeap(),
                0,
                needed as SIZE_T,
            ) as LPBYTE
        };

        /* Try again with the large Buffer */
        res = EnumPrintProcessorsW(
            nameW as PWSTR,
            envW as PWSTR,
            Level,
            bufferW as PBYTE,
            needed,
            pcbNeeded,
            pcReturned,
        ) as BOOL;
    }
    numentries = if !pcReturned.is_null() {
        *pcReturned
    } else {
        0
    };
    needed = 0;

    'epp_cleanup: {
        if res != 0 {
            /* EnumPrintProcessorsW collected all Data. Parse them to calculate ANSI-Size */
            let mut index: DWORD;
            let mut ptr: LPSTR;
            let mut ppiw: PPRINTPROCESSOR_INFO_1W;
            let mut ppia: PPRINTPROCESSOR_INFO_1A;

            /* First pass: calculate the size for all Entries */
            ppiw = bufferW as PPRINTPROCESSOR_INFO_1W;
            ppia = pPrintProcessorInfo as PPRINTPROCESSOR_INFO_1A;
            index = 0;
            while index < numentries {
                index = index.wrapping_add(1);
                needed = needed.wrapping_add(
                    core::mem::size_of::<PRINTPROCESSOR_INFO_1A>() as DWORD,
                );

                needed = needed.wrapping_add(WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*ppiw).pName as LPCWSTR,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                ) as DWORD);

                ppiw = (ppiw as LPBYTE)
                    .add(core::mem::size_of::<PRINTPROCESSOR_INFO_1W>())
                    as PPRINTPROCESSOR_INFO_1W;
                ppia = (ppia as LPBYTE)
                    .add(core::mem::size_of::<PRINTPROCESSOR_INFO_1A>())
                    as PPRINTPROCESSOR_INFO_1A;
            }

            /* check for errors and quit on failure */
            if cbBuf < needed {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                res = FALSE;
                break 'epp_cleanup;
            }

            len = (numentries as usize * core::mem::size_of::<PRINTPROCESSOR_INFO_1A>())
                as INT; /* room for structs */
            ptr = pPrintProcessorInfo.add(len as usize) as LPSTR; /* start of strings */
            cbBuf = cbBuf.wrapping_sub(len as DWORD); /* free Bytes in the user-Buffer */
            ppiw = bufferW as PPRINTPROCESSOR_INFO_1W;
            ppia = pPrintProcessorInfo as PPRINTPROCESSOR_INFO_1A;
            index = 0;
            /* Second Pass: Fill the User Buffer (if we have one) */
            while index < numentries && !pPrintProcessorInfo.is_null() {
                index = index.wrapping_add(1);

                (*ppia).pName = ptr;
                len = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*ppiw).pName as LPCWSTR,
                    -1,
                    ptr,
                    cbBuf as INT,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                ) as INT;
                ptr = ptr.offset(len as isize);
                cbBuf = cbBuf.wrapping_sub(len as DWORD);

                ppiw = (ppiw as LPBYTE)
                    .add(core::mem::size_of::<PRINTPROCESSOR_INFO_1W>())
                    as PPRINTPROCESSOR_INFO_1W;
                ppia = (ppia as LPBYTE)
                    .add(core::mem::size_of::<PRINTPROCESSOR_INFO_1A>())
                    as PPRINTPROCESSOR_INFO_1A;
            }
        }
    }

    if !pcbNeeded.is_null() {
        *pcbNeeded = needed;
    }
    if !pcReturned.is_null() {
        *pcReturned = if res != 0 {
            numentries
        } else {
            0
        };
    }

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as PVOID);
    }
    if !envW.is_null() {
        HeapFree(GetProcessHeap(), 0, envW as PVOID);
    }
    if !bufferW.is_null() {
        HeapFree(GetProcessHeap(), 0, bufferW as PVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorsW(mut pName: PWSTR, mut pEnvironment: PWSTR, mut Level: DWORD, mut pPrintProcessorInfo: PBYTE, mut cbBuf: DWORD, mut pcbNeeded: PDWORD, mut pcReturned: PDWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/base/winspool/printprocessors.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"EnumPrintProcessorsW\0" as *const u8 as *const core::ffi::c_char,
            264 as core::ffi::c_int,
            b"EnumPrintProcessorsW(%S, %S, %lu, %p, %lu, %p, %p)\n\0" as *const u8
                as *const core::ffi::c_char,
            pName,
            pEnvironment,
            Level,
            pPrintProcessorInfo,
            cbBuf,
            pcbNeeded,
            pcReturned,
        );
    }
    if pEnvironment.is_null() {
        pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PWSTR;
    }
    dwErrorCode = rpc_try(|| _RpcEnumPrintProcessors(
        pName as WINSPOOL_HANDLE,
        pEnvironment as *mut WCHAR,
        Level,
        pPrintProcessorInfo as *mut BYTE,
        cbBuf,
        pcbNeeded as *mut DWORD,
        pcReturned as *mut DWORD,
    )).unwrap_or_else(|code| code as _);
    if dwErrorCode == ERROR_SUCCESS {
        MarshallUpStructuresArray(
            cbBuf,
            pPrintProcessorInfo as PVOID,
            *pcReturned,
            PrintProcessorInfo1Marshalling.pInfo.as_ptr(),
            PrintProcessorInfo1Marshalling.cbStructureSize,
            TRUE as BOOL,
        );
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrintProcessorDirectoryA(
    pName: PSTR,
    pEnvironment: PSTR,
    Level: DWORD,
    pPrintProcessorInfo: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
) -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cch: DWORD = 0;
    let mut pwszName: PWSTR = core::ptr::null_mut();
    let mut pwszEnvironment: PWSTR = core::ptr::null_mut();
    let mut pwszPrintProcessorInfo: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        if !pName.is_null() {
            // Convert pName to a Unicode string pwszName.
            cch = strlen(pName as *const ::core::ffi::c_char) as DWORD;

            pwszName = HeapAlloc(
                hProcessHeap,
                0,
                (cch.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszName.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pName as LPCSTR,
                -1,
                pwszName as LPWSTR,
                cch.wrapping_add(1) as ::core::ffi::c_int,
            );
        }

        if !pEnvironment.is_null() {
            // Convert pEnvironment to a Unicode string pwszEnvironment.
            cch = strlen(pEnvironment as *const ::core::ffi::c_char) as DWORD;

            pwszEnvironment = HeapAlloc(
                hProcessHeap,
                0,
                (cch.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if pwszEnvironment.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pEnvironment as LPCSTR,
                -1,
                pwszEnvironment as LPWSTR,
                cch.wrapping_add(1) as ::core::ffi::c_int,
            );
        }

        if cbBuf != 0 && !pPrintProcessorInfo.is_null() {
            // Allocate a temporary buffer for the Unicode result.
            pwszPrintProcessorInfo = HeapAlloc(
                hProcessHeap,
                0,
                cbBuf as SIZE_T,
            ) as PWSTR;
            if pwszPrintProcessorInfo.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'Cleanup;
            }
        }

        bReturnValue = GetPrintProcessorDirectoryW(
            pwszName,
            pwszEnvironment,
            Level,
            pwszPrintProcessorInfo as PBYTE,
            cbBuf,
            pcbNeeded,
        );

        if bReturnValue != 0 {
            // Convert pwszPrintProcessorInfo to an ANSI string pPrintProcessorInfo.
            WideCharToMultiByte(
                CP_ACP,
                0,
                pwszPrintProcessorInfo as LPCWSTR,
                -1,
                pPrintProcessorInfo as LPSTR,
                cbBuf as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        }
    }

    // Cleanup:
    if !pwszName.is_null() {
        HeapFree(hProcessHeap, 0, pwszName as PVOID);
    }

    if !pwszEnvironment.is_null() {
        HeapFree(hProcessHeap, 0, pwszEnvironment as PVOID);
    }

    if !pwszPrintProcessorInfo.is_null() {
        HeapFree(hProcessHeap, 0, pwszPrintProcessorInfo as PVOID);
    }

    return bReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrintProcessorDirectoryW(
    mut pName: PWSTR,
    mut pEnvironment: PWSTR,
    mut Level: DWORD,
    mut pPrintProcessorInfo: PBYTE,
    mut cbBuf: DWORD,
    mut pcbNeeded: PDWORD,
) -> BOOL {
    let mut dwErrorCode: DWORD = 0;

    'Cleanup: {
        // Sanity checks
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Choose our current environment if the caller didn't give any.
        if pEnvironment.is_null() {
            pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PWSTR;
        }

        // Do the RPC call
        // RpcTryExcept
        {
            dwErrorCode = rpc_try(|| _RpcGetPrintProcessorDirectory(
                pName as WINSPOOL_HANDLE,
                pEnvironment as *mut WCHAR,
                Level,
                pPrintProcessorInfo as *mut BYTE,
                cbBuf,
                pcbNeeded as *mut DWORD,
            )).unwrap_or_else(|code| code as _);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
