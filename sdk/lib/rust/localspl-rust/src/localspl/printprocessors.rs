// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_10E49A5D4D4B: [u16; 17] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0020, 0x0050, 0x0072, 0x006F, 0x0063,
    0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_3A7BCE312332: [u16; 10] = [0x0044u16, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut _PrintProcessorList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

unsafe extern "C" fn _OpenEnvironment(pEnvironment: PCWSTR, hKey: PHKEY) -> DWORD {
    let wszEnvironmentsKey: [u16; 53] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x0045,
    0x006E, 0x0076, 0x0069, 0x0072, 0x006F, 0x006E, 0x006D, 0x0065, 0x006E, 0x0074,
    0x0073, 0x005C, 0x0000,
];
    let cchEnvironmentsKey: DWORD = (_countof!(wszEnvironmentsKey) as DWORD)
        .wrapping_sub(1 as DWORD);

    let mut cchEnvironment: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pwszEnvironmentKey: PWSTR = core::ptr::null_mut();

    'cleanup: {
        // Sanity checks
        if pEnvironment.is_null() {
            dwErrorCode = ERROR_INVALID_ENVIRONMENT as DWORD;
            break 'cleanup;
        }

        // Construct the registry key of the demanded environment.
        cchEnvironment = wcslen(pEnvironment) as DWORD;
        pwszEnvironmentKey = DllAllocSplMem(
            ((cchEnvironmentsKey
                .wrapping_add(cchEnvironment)
                .wrapping_add(1 as DWORD)) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pwszEnvironmentKey.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;

            break 'cleanup;
        }

        CopyMemory!(
            pwszEnvironmentKey as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszEnvironmentsKey) as *const core::ffi::c_void,
            (cchEnvironmentsKey as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        CopyMemory!(
            pwszEnvironmentKey.offset(cchEnvironmentsKey as isize) as *mut core::ffi::c_void,
            pEnvironment as *const core::ffi::c_void,
            ((cchEnvironment.wrapping_add(1 as DWORD)) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );

        // Open the registry key.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            pwszEnvironmentKey as PCWSTR,
            0,
            KEY_READ,
            hKey,
        ) as DWORD;
        if dwErrorCode == ERROR_FILE_NOT_FOUND as DWORD {
            dwErrorCode = ERROR_INVALID_ENVIRONMENT as DWORD;
            break 'cleanup;
        } else if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'cleanup;
        }
    }

    // Cleanup:
    if !pwszEnvironmentKey.is_null() {
        DllFreeSplMem(pwszEnvironmentKey as PVOID);
    }

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn FindDatatype(pPrintProcessor: PLOCAL_PRINT_PROCESSOR, pwszDatatype: PCWSTR) -> BOOL {
    let mut i: DWORD;
    let mut pCurrentDatatype: PDATATYPES_INFO_1W = (*pPrintProcessor).pDatatypesInfo1;

    if pwszDatatype.is_null() {
        return FALSE as BOOL;
    }

    i = 0;
    while i < (*pPrintProcessor).dwDatatypeCount {
        if _wcsicmp((*pCurrentDatatype).pName, pwszDatatype) == 0 {
            return (TRUE) as BOOL;
        }

        pCurrentDatatype = pCurrentDatatype.offset(1);
        i = i.wrapping_add(1);
    }

    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn FindPrintProcessor(pwszName: PCWSTR) -> PLOCAL_PRINT_PROCESSOR {
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();

    if pwszName.is_null() {
        return core::ptr::null_mut();
    }
    pEntry = _PrintProcessorList.Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(_PrintProcessorList) {
        pPrintProcessor = (pEntry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(LOCAL_PRINT_PROCESSOR, Entry) as ULONG_PTR)
            as *mut LOCAL_PRINT_PROCESSOR as PLOCAL_PRINT_PROCESSOR;
        if _wcsicmp(
            (*pPrintProcessor).pwszName as *const wchar_t,
            pwszName as *const wchar_t,
        ) == 0
        {
            return pPrintProcessor;
        }
        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn InitializePrintProcessorList() -> BOOL {
    const DBG_FILE: *const ::core::ffi::c_char = b"win32ss/printing/providers/localspl/printprocessors.c\0".as_ptr() as *const ::core::ffi::c_char;
    const DBG_FUNC: *const ::core::ffi::c_char = b"InitializePrintProcessorList\0".as_ptr() as *const ::core::ffi::c_char;
    let mut cbDatatypes: DWORD = 0;
    let mut cbFileName: DWORD = 0;
    let mut cchPrintProcessorPath: DWORD = 0;
    let mut cchMaxSubKey: DWORD = 0;
    let mut cchPrintProcessorName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwSubKeys: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hinstPrintProcessor: HINSTANCE = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut hSubSubKey: HKEY = core::ptr::null_mut();
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();
    let mut wszFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszPrintProcessorPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    'cleanup: {
        // Initialize an empty list for our Print Processors.
        InitializeListHead(core::ptr::addr_of_mut!(_PrintProcessorList));

        // Prepare the path to the Print Processor directory.
        if LocalGetPrintProcessorDirectory(
            core::ptr::null_mut(),
            core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PWSTR,
            1,
            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR as PBYTE,
            core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as DWORD,
            core::ptr::addr_of_mut!(cchPrintProcessorPath),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // LocalGetPrintProcessorDirectory returns the number of copied bytes. Convert this into a number of characters without the terminating null-character.
        cchPrintProcessorPath = cchPrintProcessorPath / (core::mem::size_of::<WCHAR>() as DWORD);
        cchPrintProcessorPath = cchPrintProcessorPath.wrapping_sub(1);

        // Append a trailing backslash.
        wszPrintProcessorPath[cchPrintProcessorPath as usize] = 0x005Cu16;
        cchPrintProcessorPath = cchPrintProcessorPath.wrapping_add(1);

        // Open the environment registry key.
        dwErrorCode = _OpenEnvironment(
            core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PCWSTR,
            core::ptr::addr_of_mut!(hKey),
        );
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Open the "Print Processors" subkey.
        dwErrorCode = RegOpenKeyExW(
            hKey,
            RUSTCONV_WSTR_10E49A5D4D4B.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hSubKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    DBG_FILE,
                    DBG_FUNC,
                    176 as ::core::ffi::c_int,
                    b"RegOpenKeyExW failed with status %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                    dwErrorCode,
                );
            }
            break 'cleanup;
        }

        // Get the number of Print Processors and maximum sub key length.
        dwErrorCode = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwSubKeys),
            core::ptr::addr_of_mut!(cchMaxSubKey),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    DBG_FILE,
                    DBG_FUNC,
                    184 as ::core::ffi::c_int,
                    b"RegQueryInfoKeyW failed with status %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                    dwErrorCode,
                );
            }
            break 'cleanup;
        }

        // Loop through all available local Print Processors.
        i = 0;
        while i < dwSubKeys {
            'body: {
                // Cleanup tasks from the previous run
                if !hSubSubKey.is_null() {
                    RegCloseKey(hSubSubKey);
                    hSubSubKey = core::ptr::null_mut();
                }

                if !pPrintProcessor.is_null() {
                    if !(*pPrintProcessor).pwszName.is_null() {
                        DllFreeSplStr((*pPrintProcessor).pwszName);
                    }

                    if !(*pPrintProcessor).pDatatypesInfo1.is_null() {
                        DllFreeSplMem((*pPrintProcessor).pDatatypesInfo1 as PVOID);
                    }

                    DllFreeSplMem(pPrintProcessor as PVOID);
                    pPrintProcessor = core::ptr::null_mut();
                }

                // Create a new LOCAL_PRINT_PROCESSOR structure for it.
                pPrintProcessor = DllAllocSplMem(core::mem::size_of::<LOCAL_PRINT_PROCESSOR>() as DWORD)
                    as PLOCAL_PRINT_PROCESSOR;
                if pPrintProcessor.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            215 as ::core::ffi::c_int,
                            b"DllAllocSplMem failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break 'cleanup;
                }

                // Allocate memory for the Print Monitor Name.
                (*pPrintProcessor).pwszName = DllAllocSplMem(
                    (cchMaxSubKey.wrapping_add(1) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
                        as DWORD,
                ) as PWSTR;
                if (*pPrintProcessor).pwszName.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            224 as ::core::ffi::c_int,
                            b"DllAllocSplMem failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break 'cleanup;
                }

                // Get the name of this Print Processor.
                cchPrintProcessorName = cchMaxSubKey.wrapping_add(1);
                dwErrorCode = RegEnumKeyExW(
                    hSubKey,
                    i,
                    (*pPrintProcessor).pwszName as LPWSTR,
                    core::ptr::addr_of_mut!(cchPrintProcessorName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            233 as ::core::ffi::c_int,
                            b"RegEnumKeyExW failed with status %ld!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            dwErrorCode,
                        );
                    }
                    break 'body;
                }

                // Open this Print Processor's registry key.
                dwErrorCode = RegOpenKeyExW(
                    hSubKey,
                    (*pPrintProcessor).pwszName as LPCWSTR,
                    0,
                    KEY_READ,
                    core::ptr::addr_of_mut!(hSubSubKey),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            241 as ::core::ffi::c_int,
                            b"RegOpenKeyExW failed for Print Processor \"%S\" with status %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            (*pPrintProcessor).pwszName,
                            dwErrorCode,
                        );
                    }
                    break 'body;
                }

                // Get the file name of the Print Processor.
                cbFileName = core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as DWORD;
                dwErrorCode = RegQueryValueExW(
                    hSubSubKey,
                    RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(wszFileName) as *mut WCHAR as PBYTE,
                    core::ptr::addr_of_mut!(cbFileName),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            250 as ::core::ffi::c_int,
                            b"RegQueryValueExW failed for Print Processor \"%S\" with status %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            (*pPrintProcessor).pwszName,
                            dwErrorCode,
                        );
                    }
                    break 'body;
                }

                // Verify that our buffer is large enough.
                if cchPrintProcessorPath.wrapping_add(
                    cbFileName / (core::mem::size_of::<WCHAR>() as DWORD),
                ) > MAX_PATH
                {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            257 as ::core::ffi::c_int,
                            b"Print Processor directory \"%S\" for Print Processor \"%S\" is too long!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszFileName) as *mut WCHAR,
                            (*pPrintProcessor).pwszName,
                        );
                    }
                    break 'body;
                }

                // Construct the full path to the Print Processor.
                CopyMemory!(
                    (core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR)
                        .add(cchPrintProcessorPath as usize) as *mut ::core::ffi::c_void,
                    core::ptr::addr_of!(wszFileName) as *const ::core::ffi::c_void,
                    cbFileName as usize,
                );

                // Try to load it.
                hinstPrintProcessor = LoadLibraryW(
                    core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR as LPCWSTR,
                );
                if hinstPrintProcessor.is_null() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            268 as ::core::ffi::c_int,
                            b"LoadLibraryW failed for \"%S\" with error %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                            GetLastError(),
                        );
                    }
                    break 'body;
                }

                // Get and verify all its function pointers.
                (*pPrintProcessor).pfnClosePrintProcessor = core::mem::transmute::<
                    FARPROC,
                    PClosePrintProcessor,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"ClosePrintProcessor\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnClosePrintProcessor.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            276 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no ClosePrintProcessor!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                (*pPrintProcessor).pfnControlPrintProcessor = core::mem::transmute::<
                    FARPROC,
                    PControlPrintProcessor,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"ControlPrintProcessor\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnControlPrintProcessor.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            283 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no ControlPrintProcessor!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                (*pPrintProcessor).pfnEnumPrintProcessorDatatypesW = core::mem::transmute::<
                    FARPROC,
                    PEnumPrintProcessorDatatypesW,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"EnumPrintProcessorDatatypesW\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnEnumPrintProcessorDatatypesW.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            290 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no EnumPrintProcessorDatatypesW!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                (*pPrintProcessor).pfnGetPrintProcessorCapabilities = core::mem::transmute::<
                    FARPROC,
                    PGetPrintProcessorCapabilities,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"GetPrintProcessorCapabilities\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnGetPrintProcessorCapabilities.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            297 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no GetPrintProcessorCapabilities!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                (*pPrintProcessor).pfnOpenPrintProcessor = core::mem::transmute::<
                    FARPROC,
                    POpenPrintProcessor,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"OpenPrintProcessor\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnOpenPrintProcessor.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            304 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no OpenPrintProcessor!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                (*pPrintProcessor).pfnPrintDocumentOnPrintProcessor = core::mem::transmute::<
                    FARPROC,
                    PPrintDocumentOnPrintProcessor,
                >(GetProcAddress(
                    hinstPrintProcessor,
                    b"PrintDocumentOnPrintProcessor\0".as_ptr() as LPCSTR,
                ));
                if (*pPrintProcessor).pfnPrintDocumentOnPrintProcessor.is_none() {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            311 as ::core::ffi::c_int,
                            b"Print Processor \"%S\" exports no PrintDocumentOnPrintProcessor!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                        );
                    }
                    break 'body;
                }

                // Get all supported datatypes.
                (*pPrintProcessor).pfnEnumPrintProcessorDatatypesW.expect(
                    "non-null function pointer",
                )(
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(cbDatatypes),
                    core::ptr::addr_of_mut!((*pPrintProcessor).dwDatatypeCount),
                );
                (*pPrintProcessor).pDatatypesInfo1 =
                    DllAllocSplMem(cbDatatypes) as PDATATYPES_INFO_1W;
                if (*pPrintProcessor).pDatatypesInfo1.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            321 as ::core::ffi::c_int,
                            b"DllAllocSplMem failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break 'cleanup;
                }

                if (*pPrintProcessor).pfnEnumPrintProcessorDatatypesW.expect(
                    "non-null function pointer",
                )(
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    1,
                    (*pPrintProcessor).pDatatypesInfo1 as PBYTE,
                    cbDatatypes,
                    core::ptr::addr_of_mut!(cbDatatypes),
                    core::ptr::addr_of_mut!((*pPrintProcessor).dwDatatypeCount),
                ) == 0
                {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            DBG_FILE,
                            DBG_FUNC,
                            327 as ::core::ffi::c_int,
                            b"EnumPrintProcessorDatatypesW failed for Print Processor \"%S\" with error %lu!\n\0".as_ptr() as *const ::core::ffi::c_char,
                            core::ptr::addr_of_mut!(wszPrintProcessorPath) as *mut WCHAR,
                            GetLastError(),
                        );
                    }
                    break 'body;
                }

                // Add the Print Processor to the list.
                InsertTailList(
                    core::ptr::addr_of_mut!(_PrintProcessorList),
                    core::ptr::addr_of_mut!((*pPrintProcessor).Entry),
                );

                // Don't let the cleanup routines free this.
                pPrintProcessor = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup
    if !hSubSubKey.is_null() {
        RegCloseKey(hSubSubKey);
    }

    if !pPrintProcessor.is_null() {
        if !(*pPrintProcessor).pwszName.is_null() {
            DllFreeSplStr((*pPrintProcessor).pwszName);
        }

        if !(*pPrintProcessor).pDatatypesInfo1.is_null() {
            DllFreeSplMem((*pPrintProcessor).pDatatypesInfo1 as PVOID);
        }

        DllFreeSplMem(pPrintProcessor as PVOID);
    }

    // Outside the loop
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalEnumPrintProcessorDatatypes(pName: LPWSTR, pPrintProcessorName: LPWSTR, Level: DWORD, pDatatypes: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD, pcReturned: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Try to find the Print Processor.
        pPrintProcessor = FindPrintProcessor(pPrintProcessorName as PCWSTR);
        if pPrintProcessor.is_null() {
            dwErrorCode = ERROR_UNKNOWN_PRINTPROCESSOR;
            break 'Cleanup;
        }

        // Call its EnumPrintProcessorDatatypesW function.
        if (*pPrintProcessor)
            .pfnEnumPrintProcessorDatatypesW
            .unwrap_unchecked()(
            pName,
            pPrintProcessorName,
            Level,
            pDatatypes,
            cbBuf,
            pcbNeeded,
            pcReturned,
        ) != 0
        {
            dwErrorCode = ERROR_SUCCESS;
        } else {
            dwErrorCode = GetLastError();
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalEnumPrintProcessors(pName: LPWSTR, pEnvironment: LPWSTR, Level: DWORD, pPrintProcessorInfo: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD, pcReturned: LPDWORD) -> BOOL {
    let mut cchMaxSubKey: DWORD = 0;
    let mut cchPrintProcessor: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwPrintProcessorCount: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut hSubKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut pCurrentOutputPrintProcessor: PBYTE = ::core::ptr::null_mut::<BYTE>();
    let mut pCurrentOutputPrintProcessorInfo: PBYTE = ::core::ptr::null_mut::<BYTE>();
    let mut PrintProcessorInfo1: PRINTPROCESSOR_INFO_1W = _PRINTPROCESSOR_INFO_1W {
        pName: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut pwszTemp: PWSTR = ::core::ptr::null_mut::<WCHAR>();

    'Cleanup: {
        // Sanity checks
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if pcbNeeded.is_null() || pcReturned.is_null() {
            // This error is also caught by RPC and returned as RPC_X_NULL_REF_POINTER.
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Verify pEnvironment and open its registry key.
        dwErrorCode = _OpenEnvironment(pEnvironment as PCWSTR, core::ptr::addr_of_mut!(hKey));
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Open the "Print Processors" subkey.
        dwErrorCode = RegOpenKeyExW(
            hKey,
            RUSTCONV_WSTR_10E49A5D4D4B.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hSubKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Get the number of Print Processors and maximum sub key length.
        dwErrorCode = RegQueryInfoKeyW(
            hSubKey,
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwPrintProcessorCount),
            core::ptr::addr_of_mut!(cchMaxSubKey),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Allocate a temporary buffer to let RegEnumKeyExW succeed.
        pwszTemp = DllAllocSplMem(
            (cchMaxSubKey.wrapping_add(1) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pwszTemp.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        // Determine the required size of the output buffer.
        *pcbNeeded = 0;

        i = 0;
        while i < dwPrintProcessorCount {
            cchPrintProcessor = cchMaxSubKey.wrapping_add(1);
            dwErrorCode = RegEnumKeyExW(
                hSubKey,
                i,
                pwszTemp as LPWSTR,
                core::ptr::addr_of_mut!(cchPrintProcessor),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<WCHAR>(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<_FILETIME>(),
            ) as DWORD;
            if dwErrorCode != ERROR_SUCCESS {
                break 'Cleanup;
            }

            *pcbNeeded = (*pcbNeeded as usize).wrapping_add(
                ::core::mem::size_of::<PRINTPROCESSOR_INFO_1W>().wrapping_add(
                    (cchPrintProcessor.wrapping_add(1) as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>()),
                ),
            ) as DWORD;
            i = i.wrapping_add(1);
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Put the Print Processor strings right after the last PRINTPROCESSOR_INFO_1W structure.
        pCurrentOutputPrintProcessorInfo = pPrintProcessorInfo as PBYTE;
        pCurrentOutputPrintProcessor = pPrintProcessorInfo.offset(
            (dwPrintProcessorCount as usize)
                .wrapping_mul(::core::mem::size_of::<PRINTPROCESSOR_INFO_1W>()) as isize,
        ) as PBYTE;

        // Copy over all Print Processors.
        i = 0;
        while i < dwPrintProcessorCount {
            // This isn't really correct, but doesn't cause any harm, because we've extensively checked the size of the supplied buffer above.
            cchPrintProcessor = cchMaxSubKey.wrapping_add(1);

            // Copy the Print Processor name.
            dwErrorCode = RegEnumKeyExW(
                hSubKey,
                i,
                pCurrentOutputPrintProcessor as PWSTR as LPWSTR,
                core::ptr::addr_of_mut!(cchPrintProcessor),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<WCHAR>(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<_FILETIME>(),
            ) as DWORD;
            if dwErrorCode != ERROR_SUCCESS {
                break 'Cleanup;
            }

            // Fill and copy the PRINTPROCESSOR_INFO_1W structure belonging to this Print Processor.
            PrintProcessorInfo1.pName = pCurrentOutputPrintProcessor as PWSTR as LPWSTR;
            CopyMemory!(
                pCurrentOutputPrintProcessorInfo as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(PrintProcessorInfo1) as *const core::ffi::c_void,
                ::core::mem::size_of::<PRINTPROCESSOR_INFO_1W>(),
            );

            // Advance to the next PRINTPROCESSOR_INFO_1W location and string location in the output buffer.
            pCurrentOutputPrintProcessor = pCurrentOutputPrintProcessor.offset(
                (cchPrintProcessor.wrapping_add(1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()) as isize,
            );
            pCurrentOutputPrintProcessorInfo = pCurrentOutputPrintProcessorInfo.offset(
                ::core::mem::size_of::<PRINTPROCESSOR_INFO_1W>() as isize,
            );
            i = i.wrapping_add(1);
        }

        // We've finished successfully!
        *pcReturned = dwPrintProcessorCount;
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pwszTemp.is_null() {
        DllFreeSplMem(pwszTemp as PVOID);
    }

    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalGetPrintProcessorDirectory(pName: LPWSTR, pEnvironment: LPWSTR, Level: DWORD, pPrintProcessorInfo: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    let wszPath: [WCHAR; 11] = [
    0x005Cu16, 0x0050, 0x0052, 0x0054, 0x0050, 0x0052, 0x004F, 0x0043, 0x0053, 0x005C,
    0x0000,
];
    let cchPath: DWORD = (_countof!(wszPath) - 1) as DWORD;

    let mut cbDirectoryName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let pwszDirectory: PWSTR = pPrintProcessorInfo as PWSTR;

    'cleanup: {
        // Verify pEnvironment and open its registry key.
        dwErrorCode = _OpenEnvironment(pEnvironment as PCWSTR, core::ptr::addr_of_mut!(hKey)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Determine the size of the required buffer.
        dwErrorCode = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_3A7BCE312332.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(cbDirectoryName),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        *pcbNeeded = (cchSpoolDirectory.wrapping_add(cchPath))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD)
            .wrapping_add(cbDirectoryName);

        // Is the supplied buffer large enough?
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'cleanup;
        }

        // Copy the path to the "prtprocs" directory into pPrintProcessorInfo
        CopyMemory!(
            pwszDirectory as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(wszSpoolDirectory) as *const core::ffi::c_void,
            (cchSpoolDirectory as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        CopyMemory!(
            pwszDirectory.offset(cchSpoolDirectory as isize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszPath) as *const core::ffi::c_void,
            (cchPath as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );

        // Get the directory name from the registry.
        dwErrorCode = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_3A7BCE312332.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pwszDirectory.offset(cchSpoolDirectory.wrapping_add(cchPath) as isize) as PBYTE,
            core::ptr::addr_of_mut!(cbDirectoryName),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // We've finished successfully!
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
