use crate::netsh::context::FindContextByGuid;
use crate::netsh::context::pRootContext;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_4B83808CDCBA: [u16; 67] = [
    0x0048u16, 0x0065, 0x006C, 0x0070, 0x0065, 0x0072, 0x0020, 0x0047, 0x0055, 0x0049,
    0x0044, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0044, 0x004C, 0x004C, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0043, 0x006F,
    0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_8BEBC22EE0C1: [u16; 66] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x006C, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034,
    0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058,
    0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030,
    0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058,
    0x007D, 0x0020, 0x0020, 0x0025, 0x002D, 0x0031, 0x0036, 0x0073, 0x0020, 0x0020,
    0x0025, 0x0073, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_975C768AD50B: [u16; 68] = [
    0x002Du16, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x0020, 0x0020,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x0020, 0x0020, 0x002D, 0x002D,
    0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut pDllListHead: PDLL_LIST_ENTRY =
    ::core::ptr::null::<_DLL_LIST_ENTRY>() as *mut _DLL_LIST_ENTRY;

#[no_mangle]
pub static mut pDllListTail: PDLL_LIST_ENTRY =
    ::core::ptr::null::<_DLL_LIST_ENTRY>() as *mut _DLL_LIST_ENTRY;

#[no_mangle]
pub static mut pHelperListHead: PHELPER_ENTRY =
    ::core::ptr::null::<_HELPER_ENTRY>() as *mut _HELPER_ENTRY;

#[no_mangle]
pub static mut pHelperListTail: PHELPER_ENTRY =
    ::core::ptr::null::<_HELPER_ENTRY>() as *mut _HELPER_ENTRY;

#[no_mangle]
pub static mut pCurrentDll: PDLL_LIST_ENTRY =
    ::core::ptr::null::<_DLL_LIST_ENTRY>() as *mut _DLL_LIST_ENTRY;

pub const REG_NETSH_PATH: [::core::ffi::c_ushort; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_ushort; 25]>(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0N\0e\0t\0S\0h\0\0\0",
    )
};

// #define NDEBUG (valueless flag; no Rust constant emitted)

unsafe extern "C" fn StartHelpers(mut pHelper: PHELPER_ENTRY) {
    let mut pSubHelper: PHELPER_ENTRY = core::ptr::null_mut();
    let mut dwError: DWORD = 0;
    while !pHelper.is_null() {
        if (*pHelper).bStarted == FALSE as BOOL {
            if (*pHelper).Attributes.pfnStart.is_some() {
                dwError = (*pHelper)
                    .Attributes
                    .pfnStart
                    .expect("non-null function pointer")(
                    core::ptr::null::<GUID>(),
                    0 as DWORD,
                );
                if dwError == ERROR_SUCCESS as DWORD {
                    (*pHelper).bStarted = TRUE as BOOL;
                }
            }
        }

        pSubHelper = (*pHelper).pSubHelperHead as PHELPER_ENTRY;
        while !pSubHelper.is_null() {
            StartHelpers(pSubHelper);
            pSubHelper = (*pSubHelper).pNext as PHELPER_ENTRY;
        }

        pHelper = (*pHelper).pNext as PHELPER_ENTRY;
    }
}

unsafe extern "C" fn RegisterHelperDll(pEntry: PDLL_LIST_ENTRY) {
    let mut pszValueName: PWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;

    dwError = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        REG_NETSH_PATH,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    ) as DWORD;
    if dwError == ERROR_SUCCESS as DWORD {
        RegSetValueExW(
            hKey,
            (*pEntry).pszValueName,
            0,
            REG_SZ,
            (*pEntry).pszDllName as PBYTE,
            ((wcslen((*pEntry).pszDllName as *const WCHAR) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD),
        );

        RegCloseKey(hKey);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        pszValueName as PVOID,
    );
}

unsafe extern "C" fn FreeHelperDll(pEntry: PDLL_LIST_ENTRY) {
    if !(*pEntry).hModule.is_null() {
        FreeLibrary((*pEntry).hModule);
    }

    if !(*pEntry).pszValueName.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pEntry).pszValueName as PVOID);
    }

    if !(*pEntry).pszShortName.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pEntry).pszShortName as PVOID);
    }

    if !(*pEntry).pszDllName.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pEntry).pszDllName as PVOID);
    }

    HeapFree(GetProcessHeap(), 0, pEntry as PVOID);
}

unsafe extern "C" fn LoadHelperDll(mut pszDllName: PWSTR, mut bRegister: BOOL) -> DWORD {
    let mut pInitHelperDll: PNS_DLL_INIT_FN = None;
    let mut pEntry: PDLL_LIST_ENTRY = core::ptr::null_mut();
    let mut pszStart: PWSTR = core::ptr::null_mut();
    let mut pszEnd: PWSTR = core::ptr::null_mut();
    let mut bInserted: BOOL = FALSE;
    let mut dwError: DWORD = 0;

    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<DLL_LIST_ENTRY>() as usize,
    ) as PDLL_LIST_ENTRY;
    if pEntry.is_null() {
        return ERROR_OUTOFMEMORY as DWORD;
    }

    (*pEntry).pszDllName = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(pszDllName as *const u16) as usize + 1 as usize)
            * core::mem::size_of::<u16>(),
    ) as PWSTR;
    'done: {
        if (*pEntry).pszDllName.is_null() {
            dwError = ERROR_OUTOFMEMORY as DWORD;
            break 'done;
        }

        wcscpy(
            (*pEntry).pszDllName as *mut u16,
            pszDllName as *const u16,
        );

        pszStart = wcsrchr(pszDllName as *const u16, 0x005Cu16) as PWSTR;
        if pszStart.is_null() {
            pszStart = pszDllName;
        }

        (*pEntry).pszShortName = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (wcslen(pszStart as *const u16) as usize + 1 as usize)
                * core::mem::size_of::<u16>(),
        ) as PWSTR;
        if (*pEntry).pszShortName.is_null() {
            dwError = ERROR_OUTOFMEMORY as DWORD;
            break 'done;
        }

        wcscpy(
            (*pEntry).pszShortName as *mut u16,
            pszStart as *const u16,
        );

        (*pEntry).pszValueName = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (wcslen((*pEntry).pszShortName as *const u16) as usize
                + 1 as usize)
                * core::mem::size_of::<u16>(),
        ) as PWSTR;
        if (*pEntry).pszValueName.is_null() {
            dwError = ERROR_OUTOFMEMORY as DWORD;
            break 'done;
        }

        wcscpy(
            (*pEntry).pszValueName as *mut u16,
            (*pEntry).pszShortName as *const u16,
        );

        pszEnd = wcsrchr((*pEntry).pszValueName as *const u16, 0x002Eu16) as PWSTR;
        if !pszEnd.is_null() {
            *pszEnd = UNICODE_NULL;
        }

        if pDllListTail.is_null() {
            (*pEntry).pPrev = core::ptr::null_mut();
            (*pEntry).pNext = core::ptr::null_mut();
            pDllListHead = pEntry;
            pDllListTail = pEntry;
        } else {
            (*pEntry).pPrev = core::ptr::null_mut();
            (*pEntry).pNext = pDllListHead;
            (*pDllListHead).pPrev = pEntry;
            pDllListHead = pEntry;
        }

        bInserted = TRUE;

        (*pEntry).hModule = LoadLibraryW((*pEntry).pszDllName as *const u16) as HMODULE;
        if (*pEntry).hModule.is_null() {
            dwError = GetLastError();
            DPRINT1!(
                "Could not load the helper dll %S (Error: %lu)\n",
                (*pEntry).pszDllName,
                dwError
            );
            break 'done;
        }

        pInitHelperDll = ::core::mem::transmute::<FARPROC, PNS_DLL_INIT_FN>(GetProcAddress(
            (*pEntry).hModule as HINSTANCE,
            b"InitHelperDll\0".as_ptr() as *const i8,
        ));
        if pInitHelperDll.is_none() {
            dwError = GetLastError();
            DPRINT1!("Could not find 'InitHelperDll' (Error: %lu)\n", dwError);
            break 'done;
        }

        pCurrentDll = pEntry;
        let pInitHelperDll = pInitHelperDll.expect("non-null function pointer");
        dwError = pInitHelperDll(5 as DWORD, core::ptr::null_mut());
        pCurrentDll = core::ptr::null_mut();

        DPRINT!("InitHelperDll returned %lu\n", dwError);
        if dwError != ERROR_SUCCESS as DWORD {
            DPRINT1!("Call to InitHelperDll failed (Error: %lu)\n", dwError);
            break 'done;
        }

        if bRegister != 0 {
            RegisterHelperDll(pEntry);
        }
    }

    // done:
    if dwError != ERROR_SUCCESS as DWORD {
        if bInserted != 0 {
            if !(*pEntry).pPrev.is_null() {
                (*(*pEntry).pPrev).pNext = (*pEntry).pNext;
            }
            if !(*pEntry).pNext.is_null() {
                (*(*pEntry).pNext).pPrev = (*pEntry).pPrev;
            }
            if pDllListTail == pEntry {
                pDllListTail = (*pEntry).pPrev;
            }
            if pDllListHead == pEntry {
                pDllListHead = (*pEntry).pNext;
            }
            (*pEntry).pPrev = core::ptr::null_mut();
            (*pEntry).pNext = core::ptr::null_mut();
        }

        FreeHelperDll(pEntry);
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn CreateRootHelper() -> DWORD {
    let mut pHelper: PHELPER_ENTRY;

    pHelper = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<HELPER_ENTRY>()) as PHELPER_ENTRY;
    if pHelper.is_null() {
        return ERROR_OUTOFMEMORY as DWORD;
    }

    pHelperListHead = pHelper;
    pHelperListTail = pHelper;

    return ERROR_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn LoadHelpers() {
    let mut pszNameBuffer: PWSTR = core::ptr::null_mut();
    let mut pszValueBuffer: PWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValueCount: DWORD = 0;
    let mut dwMaxNameLength: DWORD = 0;
    let mut dwMaxValueLength: DWORD = 0;
    let mut dwNameLength: DWORD = 0;
    let mut dwValueLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwError: DWORD = 0;

    DPRINT!("LoadHelpers()\n");

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        REG_NETSH_PATH,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;
    if dwError != ERROR_SUCCESS as DWORD {
        return;
    }

    'done: {
        dwError = RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwValueCount),
            core::ptr::addr_of_mut!(dwMaxNameLength),
            core::ptr::addr_of_mut!(dwMaxValueLength),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwError != ERROR_SUCCESS as DWORD {
            break 'done;
        }

        pszNameBuffer = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (dwMaxNameLength.wrapping_add(1 as DWORD) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if pszNameBuffer.is_null() {
            break 'done;
        }

        pszValueBuffer = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (dwMaxValueLength as SIZE_T).wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if pszValueBuffer.is_null() {
            break 'done;
        }

        dwIndex = 0 as DWORD;
        while dwIndex < dwValueCount {
            dwNameLength = dwMaxNameLength.wrapping_add(1 as DWORD);
            dwValueLength = dwMaxValueLength.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD);
            dwError = RegEnumValueW(
                hKey,
                dwIndex,
                pszNameBuffer,
                core::ptr::addr_of_mut!(dwNameLength),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(dwType),
                pszValueBuffer as PBYTE,
                core::ptr::addr_of_mut!(dwValueLength),
            ) as DWORD;
            if dwError != ERROR_SUCCESS as DWORD {
                break;
            }

            DPRINT!("Dll: %S --> %S  %lu\n", pszNameBuffer, pszValueBuffer, dwError);
            LoadHelperDll(pszValueBuffer, FALSE as BOOL);
            dwIndex = dwIndex.wrapping_add(1);
        }
    }

    if !pszValueBuffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pszValueBuffer as PVOID,
        );
    }

    if !pszNameBuffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pszNameBuffer as PVOID,
        );
    }

    RegCloseKey(hKey);

    StartHelpers(pHelperListHead);
}
#[no_mangle]
pub unsafe extern "C" fn UnloadHelpers() {
    let mut pEntry: PDLL_LIST_ENTRY;

    while !pDllListHead.is_null() {
        pEntry = pDllListHead;
        pDllListHead = (*pEntry).pNext as PDLL_LIST_ENTRY;

        FreeHelperDll(pEntry);
    }

    pDllListTail = core::ptr::null_mut::<_DLL_LIST_ENTRY>() as PDLL_LIST_ENTRY;
}
#[no_mangle]
pub unsafe extern "C" fn FindHelper(
    pguidHelper: *const GUID,
    mut pHelper: PHELPER_ENTRY,
) -> PHELPER_ENTRY {
    let mut pFoundHelper: PHELPER_ENTRY;
    while !pHelper.is_null() {
        if IsEqualGUID(pguidHelper, core::ptr::addr_of!((*pHelper).Attributes.guidHelper)) != 0 {
            return pHelper;
        }
        pFoundHelper = FindHelper(pguidHelper, (*pHelper).pSubHelperHead);
        if !pFoundHelper.is_null() {
            return pFoundHelper;
        }
        pHelper = (*pHelper).pNext;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RegisterHelper(
    pguidParentHelper: *const GUID,
    pHelperAttributes: *const NS_HELPER_ATTRIBUTES,
) -> DWORD {
    let mut pHelper: PHELPER_ENTRY = core::ptr::null_mut();
    let mut pParentHelper: PHELPER_ENTRY = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!("RegisterHelper(%p %p)\n", pguidParentHelper, pHelperAttributes);

    if !FindHelper(core::ptr::addr_of!((*pHelperAttributes).guidHelper), pHelperListHead).is_null()
    {
        DPRINT1!("The Helper has already been registered!\n");
        return ERROR_INVALID_PARAMETER;
    }

    pHelper = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<HELPER_ENTRY>(),
    ) as PHELPER_ENTRY;
    if pHelper.is_null()
    {
        dwError = ERROR_OUTOFMEMORY;
        // goto done;
        return dwError;
    }

    DPRINT!("Guid: %lx\n", (*pHelperAttributes).guidHelper.Data1);
    DPRINT!("dwVersion: %lu\n", (*pHelperAttributes).dwVersion);
    DPRINT!("pfnStart: %p\n", (*pHelperAttributes).pfnStart);
    DPRINT!("pfnStop: %p\n", (*pHelperAttributes).pfnStop);

    CopyMemory(
        core::ptr::addr_of_mut!((*pHelper).Attributes) as *mut core::ffi::c_void,
        pHelperAttributes as *const core::ffi::c_void,
        core::mem::size_of::<NS_HELPER_ATTRIBUTES>(),
    );
    (*pHelper).pDllEntry = pCurrentDll;
    DPRINT!("pHelper->pDllEntry: %p\n", (*pHelper).pDllEntry);

    if pguidParentHelper.is_null()
    {
        if pHelperListHead.is_null() && pHelperListTail.is_null()
        {
            pHelperListHead = pHelper;
            pHelperListTail = pHelper;
        }
        else
        {
            (*pHelper).pNext = pHelperListHead;
            (*pHelperListHead).pPrev = pHelper;
            pHelperListHead = pHelper;
        }
    }
    else
    {
        CopyMemory(
            core::ptr::addr_of_mut!((*pHelper).ParentHelperGuid) as *mut core::ffi::c_void,
            pguidParentHelper as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        pParentHelper = FindHelper(pguidParentHelper, pHelperListHead);
        if pParentHelper.is_null()
        {
            DPRINT!("Parent helper %lx not found!\n", (*pguidParentHelper).Data1);
            return ERROR_INVALID_PARAMETER;
        }

        if (*pParentHelper).pSubHelperHead.is_null() && (*pParentHelper).pSubHelperTail.is_null()
        {
            (*pParentHelper).pSubHelperHead = pHelper;
            (*pParentHelper).pSubHelperTail = pHelper;
        }
        else
        {
            (*pHelper).pPrev = (*pParentHelper).pSubHelperTail;
            (*(*pParentHelper).pSubHelperTail).pNext = pHelper;
            (*pParentHelper).pSubHelperTail = pHelper;
        }
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn AddHelperCommand(
    pwszMachine: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    DPRINT!("AddHelperCommand()\n");

    if dwArgCount == 2 as DWORD {
        return 1 as DWORD;
    }

    dwError = LoadHelperDll(
        *ppwcArguments.offset(2 as isize) as PWSTR,
        TRUE as BOOL,
    );
    if dwError != ERROR_SUCCESS as DWORD {
        return dwError;
    }

    StartHelpers(pHelperListHead);

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteHelperCommand(
    pwszMachine: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut pEntry: PDLL_LIST_ENTRY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD = 0;

    DPRINT!("DeleteHelper()\n");

    if dwArgCount == 2 {
        // ConResPrintf(StdErr, IDS_INVALID_SYNTAX);
        // ConResPrintf(StdErr, IDS_HLP_DEL_HELPER_EX);
        return 1;
    }

    pEntry = pDllListHead;
    while !pEntry.is_null() {
        if wcscmp(
            (*pEntry).pszShortName,
            *ppwcArguments.offset(2 as isize),
        ) == 0
        {
            DPRINT1!("remove %S\n", (*pEntry).pszShortName);

            if !(*pEntry).pPrev.is_null() {
                (*(*pEntry).pPrev).pNext = (*pEntry).pNext;
            }
            if !(*pEntry).pNext.is_null() {
                (*(*pEntry).pNext).pPrev = (*pEntry).pPrev;
            }
            if pDllListTail == pEntry {
                pDllListTail = (*pEntry).pPrev;
            }
            if pDllListHead == pEntry {
                pDllListHead = (*pEntry).pNext;
            }
            (*pEntry).pPrev = core::ptr::null_mut();
            (*pEntry).pNext = core::ptr::null_mut();

            dwError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                REG_NETSH_PATH,
                0,
                KEY_WRITE,
                &raw mut hKey,
            ) as DWORD;
            if dwError == ERROR_SUCCESS {
                RegDeleteValue(hKey, (*pEntry).pszValueName as LPCWSTR);
                RegCloseKey(hKey);
            }

            FreeHelperDll(pEntry);

            return 1;
        }

        pEntry = (*pEntry).pNext;
    }

    return ERROR_SUCCESS;
}

unsafe extern "C" fn PrintHelpers(pHelperListHead: PHELPER_ENTRY, mut dwLevel: DWORD) {
    let mut pHelper: PHELPER_ENTRY;
    let mut pContext: PCONTEXT_ENTRY;
    let mut szIndent: [WCHAR; 22] = [0; 22];
    let mut i: DWORD;
    let mut RootGuid: GUID = NETSH_ROOT_GUID;

    i = 0 as DWORD;
    while i < min(dwLevel, 10 as DWORD).wrapping_mul(2 as DWORD) {
        szIndent[i as usize] = 0x0020u16;
        i = i.wrapping_add(1);
    }
    szIndent[dwLevel.wrapping_mul(2 as DWORD) as usize] = UNICODE_NULL as WCHAR;

    pHelper = pHelperListHead;
    while !pHelper.is_null() {
        DPRINT!("Helper GUID: %lx\n", (*pHelper).Attributes.guidHelper.Data1);

        if IsEqualGUID(
            core::ptr::addr_of_mut!((*pHelper).Attributes.guidHelper),
            core::ptr::addr_of_mut!(RootGuid),
        ) == 0
        {
            pContext = FindContextByGuid(core::ptr::addr_of_mut!((*pHelper).Attributes.guidHelper));
            DPRINT!("pContext: %p\n", pContext);
            if !pContext.is_null() {
                ConPrintf(
                    StdOut,
                    RUSTCONV_WSTR_8BEBC22EE0C1.as_ptr(),
                    (*pHelper).Attributes.guidHelper.Data1,
                    (*pHelper).Attributes.guidHelper.Data2,
                    (*pHelper).Attributes.guidHelper.Data3,
                    (*pHelper).Attributes.guidHelper.Data4[0 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[1 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[2 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[3 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[4 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[5 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[6 as usize],
                    (*pHelper).Attributes.guidHelper.Data4[7 as usize],
                    (*(*pHelper).pDllEntry).pszShortName,
                    core::ptr::addr_of_mut!(szIndent) as *mut WCHAR,
                    (*pContext).pszContextName,
                );
            }

            dwLevel = dwLevel.wrapping_add(1);
        }

        PrintHelpers((*pHelper).pSubHelperHead, dwLevel);

        pHelper = (*pHelper).pNext;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ShowHelperCommand(
    pwszMachine: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("ShowHelperCommand()\n");

    ConPrintf(StdOut, RUSTCONV_WSTR_4B83808CDCBA.as_ptr());
    ConPrintf(StdOut, RUSTCONV_WSTR_975C768AD50B.as_ptr());

    if pRootContext.is_null() {
        return ERROR_SUCCESS;
    }

    PrintHelpers(pHelperListHead, 0);

    return ERROR_SUCCESS;
}
