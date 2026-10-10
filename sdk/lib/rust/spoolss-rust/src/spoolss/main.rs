// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4569243C2C7C: [u16; 49] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x0050,
    0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_9E680C450A67: [u16; 13] = [
    0x0077u16, 0x0069, 0x006E, 0x0073, 0x0070, 0x006F, 0x006F, 0x006C, 0x002E, 0x0064,
    0x0072, 0x0076, 0x0000,
];
pub static RUSTCONV_WSTR_B43EB76F367C: [u16; 9] = [0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0073, 0x0070, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut PrintProviderList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

unsafe extern "C" fn _AddPrintProviderToList(pwszFileName: PCWSTR) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut hinstPrintProvider: HINSTANCE;
    let mut pfnInitializePrintProvidor: PInitializePrintProvidor;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    'cleanup: {
        // Try to load it.
        hinstPrintProvider = LoadLibraryW(pwszFileName);
        if hinstPrintProvider.is_null() {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Get the initialization routine.
        pfnInitializePrintProvidor = core::mem::transmute::<FARPROC, PInitializePrintProvidor>(
            GetProcAddress(hinstPrintProvider, b"InitializePrintProvidor\0".as_ptr() as LPCSTR),
        );
        if pfnInitializePrintProvidor.is_none() {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Create a new SPOOLSS_PRINT_PROVIDER structure for it.
        pPrintProvider = DllAllocSplMem(core::mem::size_of::<SPOOLSS_PRINT_PROVIDER>() as DWORD) as PSPOOLSS_PRINT_PROVIDER;
        if pPrintProvider.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'cleanup;
        }

        // Call the Print Provider initialization function.
        if (pfnInitializePrintProvidor.unwrap_unchecked())(
            core::ptr::addr_of_mut!((*pPrintProvider).PrintProvider),
            core::mem::size_of::<PRINTPROVIDOR>() as DWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Add this Print Provider to the list.
        InsertTailList(
            core::ptr::addr_of_mut!(PrintProviderList),
            core::ptr::addr_of_mut!((*pPrintProvider).Entry),
        );

        // Don't let the cleanup routine free this.
        pPrintProvider = core::ptr::null_mut();
        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    // Cleanup:
    if !pPrintProvider.is_null() {
        DllFreeSplMem(pPrintProvider as PVOID);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _InitializePrintProviderList() -> BOOL {
    let mut cbFileName: DWORD = 0;
    let mut cchMaxSubKey: DWORD = 0;
    let mut cchPrintProviderName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwSubKeys: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut pwszPrintProviderName: PWSTR = core::ptr::null_mut();
    let mut wszFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    'quit: {
        // Initialize an empty list for our Print Providers.
        InitializeListHead(core::ptr::addr_of_mut!(PrintProviderList));

        // First add the Local Spooler.
        // This one must exist and must be the first one in the list.
        dwErrorCode = _AddPrintProviderToList(RUSTCONV_WSTR_B43EB76F367C.as_ptr()) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'quit;
        }

        // Now add additional Print Providers from the registry.
        // First of all, open the key containing print providers.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_4569243C2C7C.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'quit;
        }

        // Get the number of Print Providers and maximum sub key length.
        dwErrorCode = RegQueryInfoKeyW(
            hKey,
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
            break 'quit;
        }

        // Allocate a temporary buffer for the Print Provider names.
        pwszPrintProviderName = DllAllocSplMem(
            (cchMaxSubKey + 1) * core::mem::size_of::<WCHAR>() as DWORD,
        ) as PWSTR;
        if pwszPrintProviderName.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'quit;
        }

        // Loop through all available Print Providers.
        i = 0;
        loop {
            if !(i < dwSubKeys) {
                break;
            }
            'body: {
                // Cleanup tasks from the previous run
                if !hSubKey.is_null() {
                    RegCloseKey(hSubKey);
                    hSubKey = core::ptr::null_mut();
                }

                // Get the name of this Print Provider.
                cchPrintProviderName = cchMaxSubKey + 1;
                dwErrorCode = RegEnumKeyExW(
                    hKey,
                    i,
                    pwszPrintProviderName,
                    core::ptr::addr_of_mut!(cchPrintProviderName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Open this Print Provider's registry key.
                dwErrorCode = RegOpenKeyExW(
                    hKey,
                    pwszPrintProviderName,
                    0,
                    KEY_READ,
                    core::ptr::addr_of_mut!(hSubKey),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Get the file name of the Print Provider.
                cbFileName = (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as DWORD;
                dwErrorCode = RegQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(wszFileName) as PBYTE,
                    core::ptr::addr_of_mut!(cbFileName),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Load and add it to the list.
                dwErrorCode = _AddPrintProviderToList(
                    core::ptr::addr_of_mut!(wszFileName) as PCWSTR,
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }
            }
            i = i.wrapping_add(1);
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Inside the loop
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    // Outside the loop
    if !pwszPrintProviderName.is_null() {
        DllFreeSplMem(pwszPrintProviderName as PVOID);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    if fdwReason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(hinstDLL);
        hProcessHeap = GetProcessHeap();
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn InitializeRouter(SpoolerStatusHandle: HANDLE) -> BOOL {
    return _InitializePrintProviderList();
}

#[no_mangle]
pub unsafe extern "system" fn SplInitializeWinSpoolDrv(pTable: *mut PVOID) -> BOOL {
    let mut hWinspool: HINSTANCE = core::ptr::null_mut();
    let mut i: core::ffi::c_int;

    hWinspool = LoadLibraryW(RUSTCONV_WSTR_9E680C450A67.as_ptr());
    if hWinspool.is_null() {
        return FALSE as BOOL;
    }

    // Get the function pointers which are meant to be returned by this function.
    *pTable.add(0) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, b"OpenPrinterW\0".as_ptr() as LPSTR),
    );
    *pTable.add(1) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, b"ClosePrinter\0".as_ptr() as LPSTR),
    );
    *pTable.add(2) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, b"SpoolerDevQueryPrintW\0".as_ptr() as LPSTR),
    );
    *pTable.add(3) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, b"SpoolerPrinterEvent\0".as_ptr() as LPSTR),
    );
    *pTable.add(4) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, b"DocumentPropertiesW\0".as_ptr() as LPSTR),
    );
    *pTable.add(5) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, (212 as usize) as LPSTR),
    );
    *pTable.add(6) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, (213 as usize) as LPSTR),
    );
    *pTable.add(7) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, (214 as usize) as LPSTR),
    );
    *pTable.add(8) = core::mem::transmute::<FARPROC, PVOID>(
        GetProcAddress(hWinspool, (215 as usize) as LPSTR),
    );

    // Verify that all calls succeeded.
    i = 0;
    while i < 9 {
        if (*pTable.add(i as usize)).is_null() {
            FreeLibrary(hWinspool);
            return FALSE as BOOL;
        }
        i += 1;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SplIsUpgrade() -> BOOL {
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SpoolerInit() -> BOOL {
    // Nothing to do here yet
    SetLastError(ERROR_SUCCESS);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn BuildOtherNamesFromMachineName(ptr1: *mut LPVOID, ptr2: *mut ULONG) -> BOOL {
    *ptr1 = core::ptr::null_mut();
    *ptr2 = 0;
    return FALSE as BOOL;
}
