use crate::svchost::globals::MemAlloc;
use crate::svchost::globals::MemFree;
use crate::svchost::globals::MemInit;
use crate::svchost::registry::RegQueryDword;
use crate::svchost::registry::RegQueryString;
use crate::svchost::registry::RegQueryStringA;
use crate::svchost::globals::SvchostBuildSharedGlobals;
use crate::svchost::globals::SvchostCharLowerW;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_23BC2055C607: [u16; 20] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0052, 0x0070, 0x0063,
    0x0053, 0x0074, 0x0061, 0x0063, 0x006B, 0x0053, 0x0069, 0x007A, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_2948F33B5A9E: [u16; 12] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x004D, 0x0061, 0x0069,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_2A0A7FF1C0E2: [u16; 7] = [0x0028u16, 0x006E, 0x006F, 0x006E, 0x0065, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_666D14E5D6DA: [u16; 23] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x006C, 0x006C,
    0x0055, 0x006E, 0x006C, 0x006F, 0x0061, 0x0064, 0x004F, 0x006E, 0x0053, 0x0074,
    0x006F, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_7230EE40A9B6: [u16; 16] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x004D, 0x0061, 0x006E,
    0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_80D12598FCD4: [u16; 27] = [
    0x0041u16, 0x0075, 0x0074, 0x0068, 0x0065, 0x006E, 0x0074, 0x0069, 0x0063, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0043, 0x0061, 0x0070, 0x0061, 0x0062, 0x0069,
    0x006C, 0x0069, 0x0074, 0x0069, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_85D5C37ED300: [u16; 34] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_99C4EAFCB663: [u16; 7] = [0x0028u16, 0x006E, 0x0075, 0x006C, 0x006C, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_9EADC9D3A146: [u16; 11] = [
    0x0050u16, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_B286F96F5A79: [u16; 15] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0043, 0x0072, 0x0069, 0x0074,
    0x0069, 0x0063, 0x0061, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_C050A7D8FA50: [u16; 11] = [
    0x0044u16, 0x0065, 0x0062, 0x0075, 0x0067, 0x0042, 0x0072, 0x0065, 0x0061, 0x006B,
    0x0000,
];
pub static RUSTCONV_WSTR_C2FC987EE980: [u16; 20] = [
    0x0041u16, 0x0075, 0x0074, 0x0068, 0x0065, 0x006E, 0x0074, 0x0069, 0x0063, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x004C, 0x0065, 0x0076, 0x0065, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_C46A70999F35: [u16; 11] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0044, 0x006C, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_C666C334031D: [u16; 53] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0053, 0x0076, 0x0063, 0x0068, 0x006F,
    0x0073, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_DD349F201F40: [u16; 26] = [
    0x0043u16, 0x006F, 0x0049, 0x006E, 0x0069, 0x0074, 0x0069, 0x0061, 0x006C, 0x0069,
    0x007A, 0x0065, 0x0053, 0x0065, 0x0063, 0x0075, 0x0072, 0x0069, 0x0074, 0x0079,
    0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_E96E0F8292D6: [u16; 19] = [
    0x0049u16, 0x006D, 0x0070, 0x0065, 0x0072, 0x0073, 0x006F, 0x006E, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x004C, 0x0065, 0x0076, 0x0065, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_EEE6D6305C20: [u16; 3] = [0x004Eu16, 0x006F, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DllList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut ListLock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

#[no_mangle]
pub static mut ServiceCount: DWORD = 0;

#[no_mangle]
pub static mut ServiceNames: LPCWSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut ServiceArray: PSVCHOST_SERVICE =
    ::core::ptr::null::<_SVCHOST_SERVICE>() as *mut _SVCHOST_SERVICE;

// NOTE: source ends with a dangling `__callback LONG` fragment with no declarator; nothing emitted for it.

#[no_mangle]
pub unsafe extern "system" fn SvchostUnhandledExceptionFilter(
    ExceptionInfo: *mut _EXCEPTION_POINTERS,
) -> LONG {
    /* Call the default OS handler */
    return RtlUnhandledExceptionFilter(ExceptionInfo);
}

#[no_mangle]
pub unsafe extern "system" fn DummySvchostCtrlHandler(dwControl: DWORD) {}

#[no_mangle]
pub unsafe extern "system" fn BuildCommandOptions(pszCmdLine: LPWSTR) -> PSVCHOST_OPTIONS {
    let mut cbCmdLine: ULONG;
    let mut pOptions: PSVCHOST_OPTIONS;
    let mut pch: LPWSTR;
    let mut pGroupNameStart: LPWSTR;
    let mut lpServiceGroup: LPWSTR;
    let mut pGroupName: *mut LPWSTR = core::ptr::null_mut();

    /* Nothing to do without a command-line */
    if pszCmdLine.is_null() {
        return ERROR_SUCCESS as usize as PSVCHOST_OPTIONS;
    }

    /* Get the length if the command line*/
    cbCmdLine = (core::mem::size_of_val(&UNICODE_NULL) as ULONG)
        * (lstrlenW(pszCmdLine) as ULONG)
        + (core::mem::size_of_val(&UNICODE_NULL) as ULONG);

    /* Allocate a buffer for the parsed options */
    pOptions = MemAlloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((cbCmdLine as usize) + core::mem::size_of::<_SVCHOST_OPTIONS>()) as DWORD,
    ) as PSVCHOST_OPTIONS;
    if pOptions.is_null() {
        return ERROR_SUCCESS as usize as PSVCHOST_OPTIONS;
    }

    /* Copy the buffer into the parsed options block */
    (*pOptions).CmdLineBuffer = pOptions.offset(1 as isize) as PWCHAR;
    memcpy(
        (*pOptions).CmdLineBuffer as *mut core::ffi::c_void,
        pszCmdLine as *const core::ffi::c_void,
        cbCmdLine as usize,
    );

    /* Set the command-line as being inside the block itself */
    pch = (*pOptions).CmdLineBuffer as LPWSTR;

    (*pOptions).CmdLine = pch as PWCHAR;

    /* Now scan it */
    while *pch != UNICODE_NULL {
        /* And look for the first space or TAB */
        if (*pch == (' ' as u16)) || (*pch == ('\t' as u16)) {
            /* Terminate the command-line there */
            let fresh15 = pch;
            pch = pch.offset(1);
            *fresh15 = UNICODE_NULL;
            break;
        }

        /* Keep searching */
        pch = pch.offset(1);
    }

    /* Lowercase the entire command line, but not the options */
    SvchostCharLowerW((*pOptions).CmdLine as LPCWSTR);

    /* Loop over the command-line */
    while *pch != UNICODE_NULL {
        /* Do an inner loop within it */
        while *pch != UNICODE_NULL {
            /* And keep going until we find a space or TAB */
            if (*pch != (' ' as u16)) && (*pch != ('\t' as u16)) {
                break;
            }
            pch = pch.offset(1);
        }

        /* Did we reach the end? If so, bail out */
        if *pch == UNICODE_NULL {
            break;
        }

        /* We found the space -- is it followed by a dash or slash? */
        if (*pch == ('-' as u16) || *pch == ('/' as u16))
            && (*pch.offset(1 as isize) != UNICODE_NULL)
        {
            /* Yep, and there's at least one character. Is it k or K? */
            pch = pch.offset(1);
            if *pch == ('k' as u16) || *pch == ('K' as u16) {
                /* Yep, we have a proper command line with a group! */
                (*pOptions).HasServiceGroup = TRUE as BOOL;
                pGroupName = core::ptr::addr_of_mut!((*pOptions).ServiceGroupName);
            }

            /* Keep going -- the name follows between spaces or quotes */
            pch = pch.offset(1);
        } else {
            /* Nope, so this might be the group being entered */
            pGroupNameStart = pch;

            /* Check if there's a quote */
            if (*pch == ('"' as u16))
                && (*pch.offset(1 as isize) != UNICODE_NULL)
            {
                /* There is, keep going until the quote is over with */
                pch = pch.offset(1);
                pGroupNameStart = pch;
                while *pch != 0 {
                    let fresh16 = pch;
                    pch = pch.offset(1);
                    if *fresh16 == ('"' as u16) {
                        break;
                    }
                }
            } else {
                /* No quote, so keep going until the next space or TAB */
                while (*pch != 0) && ((*pch != (' ' as u16)) && (*pch != ('\t' as u16))) {
                    pch = pch.offset(1);
                }
            }

            /* Now we have a space or quote delimited name, terminate it */
            if *pch != 0 {
                let fresh17 = pch;
                pch = pch.offset(1);
                *fresh17 = UNICODE_NULL;
            }

            /* Ok, so we have a string -- was it preceded by a -K or /K? */
            if !pGroupName.is_null() {
                /* Yes it was, remember this, and don't overwrite it later */
                *pGroupName = pGroupNameStart;
                pGroupName = core::ptr::null_mut();
            }
        }
    }

    /* Print out the service group for the debugger, and the command line */
    DBG_TRACE!("Command line     : %ws\n",
        pszCmdLine,
    );
    lpServiceGroup = if (*pOptions).HasServiceGroup != 0 {
        (*pOptions).ServiceGroupName as LPWSTR
    } else {
        RUSTCONV_WSTR_EEE6D6305C20.as_ptr() as LPWSTR
    };
    DBG_TRACE!("Service Group    : %ws\n",
        lpServiceGroup,
    );

    /* Was a group specified? */
    if (*pOptions).HasServiceGroup == FALSE as BOOL {
        /* This isn't valid. Print out help on the debugger and fail */
        DBG_TRACE!("Generic Service Host\n\n%ws [-k <key>] | [-r] | <service>\n\n   -k <key>   Host all services whose ImagePath matches\n              %ws -k <key>.\n\n",
            lpServiceGroup,
        );
        MemFree(pOptions as LPVOID);
        pOptions = core::ptr::null_mut();
    }

    /* Return the allocated option block, if valid*/
    return pOptions;
}

#[no_mangle]
pub unsafe extern "system" fn ReadPerInstanceRegistryParameters(
    hKey: HKEY,
    pOptions: PSVCHOST_OPTIONS,
) -> DWORD {
    let mut dwError: DWORD;
    let mut dwData: DWORD = 0;
    let mut hSubKey: HKEY = core::mem::zeroed();

    /* We should have a name for this service group... */

    /* Query the group to see what services are part of it */
    dwError = RegQueryString(
        hKey,
        (*pOptions).ServiceGroupName as LPCWSTR,
        REG_MULTI_SZ as DWORD,
        core::ptr::addr_of_mut!(ServiceNames) as *mut PBYTE,
    );
    if (dwError == ERROR_SUCCESS as DWORD)
        && ((ServiceNames.is_null()) || (*ServiceNames == UNICODE_NULL as u16))
    {
        /* If the key exists but there's no data, fail */
        return ERROR_INVALID_DATA as DWORD;
    }

    /* Open the key for this group */
    if RegOpenKeyExW(
        hKey,
        (*pOptions).ServiceGroupName as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hSubKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        /* If we couldn't, bail out */
        return dwError;
    }

    /* Check if we should initialize COM */
    if RegQueryDword(
        hSubKey,
        RUSTCONV_WSTR_DD349F201F40.as_ptr(),
        core::ptr::addr_of_mut!(dwData),
    ) == ERROR_SUCCESS as DWORD
    {
        /* Yes, remember the parameter to be sent when we do so */
        (*pOptions).CoInitializeSecurityParam = dwData;
    }

    /* Also, if COM is requested, we must read a bunch more data... */
    if (*pOptions).CoInitializeSecurityParam != 0 {
        /* First, read the authentication level, use a default if none is set */
        if RegQueryDword(
            hSubKey,
            RUSTCONV_WSTR_C2FC987EE980.as_ptr(),
            core::ptr::addr_of_mut!(dwData),
        ) != 0
        {
            (*pOptions).AuthenticationLevel = RPC_C_AUTHN_LEVEL_PKT as DWORD;
        } else {
            (*pOptions).AuthenticationLevel = dwData;
        }

        /* Do the same for the impersonation level */
        if RegQueryDword(
            hSubKey,
            RUSTCONV_WSTR_E96E0F8292D6.as_ptr(),
            core::ptr::addr_of_mut!(dwData),
        ) != 0
        {
            (*pOptions).ImpersonationLevel = RPC_C_IMP_LEVEL_IDENTIFY as SECURITY_IMPERSONATION_LEVEL;
        } else {
            (*pOptions).ImpersonationLevel = dwData as SECURITY_IMPERSONATION_LEVEL;
        }

        /* Do the same for the authentication capabilities */
        if RegQueryDword(
            hSubKey,
            RUSTCONV_WSTR_80D12598FCD4.as_ptr(),
            core::ptr::addr_of_mut!(dwData),
        ) != 0
        {
            (*pOptions).AuthenticationCapabilities = (EOAC_NO_CUSTOM_MARSHAL as DWORD)
                | (EOAC_DISABLE_AAA as DWORD);
        } else {
            (*pOptions).AuthenticationCapabilities = dwData;
        }
    }

    /* Check if we need a specific RPC stack size, if not, we'll set it later */
    if RegQueryDword(
        hSubKey,
        RUSTCONV_WSTR_23BC2055C607.as_ptr(),
        core::ptr::addr_of_mut!(dwData),
    ) == 0
    {
        (*pOptions).DefaultRpcStackSize = dwData;
    }

    /* Finally, check if the services should be marked critical later on */
    if RegQueryDword(
        hSubKey,
        RUSTCONV_WSTR_B286F96F5A79.as_ptr(),
        core::ptr::addr_of_mut!(dwData),
    ) == 0
    {
        (*pOptions).SystemCritical = dwData as BOOLEAN;
    }

    /* All done reading the settings */
    RegCloseKey(hSubKey);
    return dwError;
}
#[no_mangle]
pub unsafe extern "system" fn BuildServiceArray(lpOptions: PSVCHOST_OPTIONS) {
    let mut dwError: DWORD;
    let mut pService: PSVCHOST_SERVICE;
    let mut pszServiceName: LPCWSTR;
    let mut hKey: HKEY = core::mem::zeroed();

    /* Open the service host key */
    dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            RUSTCONV_WSTR_C666C334031D.as_ptr(),
                            0,
                            KEY_READ,
                            &mut hKey) as DWORD;
    if dwError != ERROR_SUCCESS {
        return;
    }

    /* Read all the parameters for this service group */
    dwError = ReadPerInstanceRegistryParameters(hKey, lpOptions);
    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS {
        return;
    }

    /* Acquire the database lock while we create the service group */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Loop the array of services for this group */
    pszServiceName = ServiceNames;
    while *pszServiceName != UNICODE_NULL {
        /* For each one, increment the amount of services we'llhost */
        ServiceCount += 1;
        pszServiceName = pszServiceName
            .offset((lstrlenW(pszServiceName) + 1) as isize);
    }

    /* We should host at least one... */
    if ServiceCount == 0 {
        RtlAssert(
            b"ServiceCount\0".as_ptr() as PVOID,
            b"base/services/svchost/svchost.c\0".as_ptr() as PVOID,
            311 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    /* Now allocate an array to hold all their pointers */
    ServiceArray = MemAlloc(
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<SVCHOST_SERVICE>() as DWORD).wrapping_mul(ServiceCount),
    ) as PSVCHOST_SERVICE;
    if !ServiceArray.is_null() {
        /* Start at the beginning of the array */
        pService = ServiceArray;

        /* Loop through all the services */
        pszServiceName = ServiceNames;
        while *pszServiceName != UNICODE_NULL {
            /* Save the service's name and move to the next entry */
            let fresh = pService;
            pService = pService.offset(1);
            (*fresh).pszServiceName = pszServiceName;
            pszServiceName = pszServiceName
                .offset((lstrlenW(pszServiceName) + 1) as isize);
        }

        /* We should have gotten the same count as earlier! */
        if pService != ServiceArray.offset(ServiceCount as isize) {
            RtlAssert(
                b"pService == ServiceArray + ServiceCount\0".as_ptr() as PVOID,
                b"base/services/svchost/svchost.c\0".as_ptr() as PVOID,
                330 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    /* We're done, drop the lock */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));
}

#[no_mangle]
pub unsafe extern "system" fn FDebugBreakForService(ServiceName: LPCWSTR) -> BOOL {
    let mut Data: DWORD;
    let mut DebugBreakOn: BOOL = FALSE as BOOL;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();

    /* Open the key for Svchost itself */
    if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                      RUSTCONV_WSTR_C666C334031D.as_ptr(),
                      0,
                      KEY_READ,
                      &mut hKey) == ERROR_SUCCESS as LONG
    {
        /* That worked, now open the key for this service, if it exists */
        if RegOpenKeyExW(hKey, ServiceName, 0, KEY_READ, &mut hSubKey) == ERROR_SUCCESS as LONG
        {
            /* It does -- is there a DebugBreak value set? */
            Data = 0;
            if RegQueryDword(hSubKey, RUSTCONV_WSTR_C050A7D8FA50.as_ptr(), &mut Data) == ERROR_SUCCESS as DWORD
            {
                /* There is! Check if it's 0 or 1 */
                DebugBreakOn = (Data != 0) as BOOL;
            }

            /* Done, close the service key */
            RegCloseKey(hSubKey);
        }

        /* Done, close the svchost key */
        RegCloseKey(hKey);
    }

    /* Return if the value was set or not */
    return DebugBreakOn;
}

#[no_mangle]
pub unsafe extern "system" fn OpenServiceParametersKey(
    lpSubKey: LPCWSTR,
    phKey: PHKEY,
) -> DWORD {
    let mut dwError: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Open the services key */
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_85D5C37ED300.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;
    if dwError == ERROR_SUCCESS {
        /* Now open the service specific key, returning its handle */
        dwError = RegOpenKeyExW(
            hKey,
            lpSubKey,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hSubKey),
        ) as DWORD;
        if dwError == ERROR_SUCCESS {
            /* And if that worked, return a handle to the parameters key */
            dwError = RegOpenKeyExW(
                hSubKey,
                RUSTCONV_WSTR_9EADC9D3A146.as_ptr(),
                0,
                KEY_READ,
                phKey,
            ) as DWORD;
        }
    }

    /* Clean up any leftovers*/
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }
    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn UnloadServiceDll(pService: PSVCHOST_SERVICE) {
    let mut cbData: DWORD;
    let mut Data: DWORD;
    let mut dwType: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    cbData = 4;
    Data = 0;

    /* Grab the lock while we touch the thread count and potentially unload */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* There is one less active callout into the DLL, it may be unloadabl now */

    (*pService).cServiceActiveThreadCount -= 1;

    /* Try to open the service registry key */
    if OpenServiceParametersKey((*pService).pszServiceName, &mut hKey) == ERROR_SUCCESS
    {
        /* It worked -- check if we should unload the service when stopped */
        if RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_666D14E5D6DA.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut Data as *mut DWORD as LPBYTE,
            &mut cbData,
        ) == ERROR_SUCCESS as i32
        {
            /* Make sure the data is correctly formatted */
            if (dwType == REG_DWORD) && (Data == 1)
            {
                /* Does the service still have active threads? */
                if (*pService).cServiceActiveThreadCount != 0
                {
                    /* Yep, so we can't kill it just yet */
                    DBG_TRACE!("Skip Unload dll %ws for service %ws, active threads %d\n",
                        (*(*pService).pDll).pszDllPath,
                        (*pService).pszServiceName,
                        (*pService).cServiceActiveThreadCount,
                    );
                }
                else
                {
                    /* Nope, we're good to go */
                    DBG_TRACE!("Unload dll %ws for service %ws\n",
                        (*(*pService).pDll).pszDllPath,
                        (*pService).pszServiceName,
                    );

                    /* Free the library and clear the module handle */
                    FreeLibrary((*(*pService).pDll).hModule);
                    (*(*pService).pDll).hModule = core::ptr::null_mut();
                }
            }
        }
    }

    /* Drop the lock, unload is complete */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Close the parameters key if we had it */
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "system" fn SvchostStopCallback(Context: PVOID, TimerOrWaitFired: BOOLEAN) {
    let mut pSvcsStopCbContext: PSVCHOST_CALLBACK_CONTEXT = Context as PSVCHOST_CALLBACK_CONTEXT;
    let mut pfnStopCallback: PSVCHOST_STOP_CALLBACK;

    /* Hold the lock while we grab the callback */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Grab the callback, then clear it */
    pfnStopCallback = (*(*pSvcsStopCbContext).pService).pfnStopCallback;

    (*(*pSvcsStopCbContext).pService).pfnStopCallback = None;

    /* Now release the lock, making sure the above was atomic */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Now make the callout */
    DBG_TRACE!("Call stop callback for service %ws, active threads %d\n",
        (*(*pSvcsStopCbContext).pService).pszServiceName,
        (*(*pSvcsStopCbContext).pService).cServiceActiveThreadCount,
    );
    (pfnStopCallback.unwrap_unchecked())((*pSvcsStopCbContext).pContext, TimerOrWaitFired);

    /* Decrement the active threads -- maybe the DLL can unload now */
    UnloadServiceDll((*pSvcsStopCbContext).pService);

    /* We no longer need the context */
    MemFree(pSvcsStopCbContext as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SvcRegisterStopCallback(phNewWaitObject: PHANDLE, ServiceName: PCWSTR, hObject: HANDLE, pfnStopCallback: PSVCHOST_STOP_CALLBACK, pContext: PVOID, dwFlags: ULONG) -> DWORD {
    let mut i: ULONG = 0;
    let mut pSvcsStopCbContext: PSVCHOST_CALLBACK_CONTEXT = core::ptr::null_mut();
    let mut pService: PSVCHOST_SERVICE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    /* All parameters must be present */
    if phNewWaitObject.is_null()
        || ServiceName.is_null()
        || hObject.is_null()
        || pfnStopCallback.is_none()
    {
        /* Fail otherwise */
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    /* Don't allow new DLLs while we send notifications */
    EnterCriticalSection(&raw mut ListLock);

    /* Scan all registered services */
    i = 0 as ULONG;
    while i < ServiceCount as ULONG {
        /* Search for the service entry matching this service name */
        if lstrcmpiW(
            ServiceName as LPCWSTR,
            (*ServiceArray.offset(i as isize)).pszServiceName,
        ) == 0
        {
            /* Found it */
            pService = ServiceArray.offset(i as isize) as *mut _SVCHOST_SERVICE as PSVCHOST_SERVICE;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    /* Do we have a service? Does it already have a callback? */
    if pService.is_null() || (*pService).pfnStopCallback.is_some() {
        /* Affirmative, nothing for us to do */
        dwError = ERROR_INVALID_DATA as DWORD;
        DBG_ERR!("Service %ws missing or already registered for callback, status %d\n",
            ServiceName,
            dwError,
        );
    } else {
        /* Allocate our internal context */
        pSvcsStopCbContext = MemAlloc(
            HEAP_ZERO_MEMORY as DWORD,
            core::mem::size_of::<_SVCHOST_CALLBACK_CONTEXT>() as DWORD,
        ) as PSVCHOST_CALLBACK_CONTEXT;
        if pSvcsStopCbContext.is_null() {
            /* We failed, bail out */
            dwError = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            DBG_ERR!("MemAlloc for context block for service %ws failed, status %d\n",
                ServiceName,
                dwError,
            );
        } else {
            /* Setup the context and register for the wait */
            (*pSvcsStopCbContext).pContext = pContext;
            (*pSvcsStopCbContext).pService = pService;
            if RegisterWaitForSingleObject(
                phNewWaitObject,
                hObject,
                Some(SvchostStopCallback as unsafe extern "system" fn(PVOID, BOOLEAN) -> ()),
                pSvcsStopCbContext as PVOID,
                INFINITE as ULONG,
                dwFlags,
            ) != 0
            {
                /* We now have an active thread associated with this wait */
                (*pService).cServiceActiveThreadCount += 1;
                (*pService).pfnStopCallback = pfnStopCallback;
                DBG_TRACE!("Registered stop callback for service %ws, active threads %d\n",
                    ServiceName,
                    (*pService).cServiceActiveThreadCount,
                );
            } else {
                /* Registration failed, bail out */
                dwError = GetLastError() as DWORD;
                DBG_ERR!("Registration for stop callback for service %ws failed, status %d\n",
                    ServiceName,
                    dwError,
                );
            }
        }
    }

    /* Drop the lock, and free the context if we failed */
    LeaveCriticalSection(&raw mut ListLock);
    if dwError != ERROR_SUCCESS as DWORD {
        MemFree(pSvcsStopCbContext as LPVOID);
    }
    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn AbortSvchostService(lpServiceName: LPCWSTR, dwExitCode: DWORD) {
    let mut scHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS = SERVICE_STATUS {
        dwServiceType: 0,
        dwCurrentState: 0,
        dwControlsAccepted: 0,
        dwWin32ExitCode: 0,
        dwServiceSpecificExitCode: 0,
        dwCheckPoint: 0,
        dwWaitHint: 0,
    };

    /* Make us stopped and accept only query commands */
    ServiceStatus.dwCheckPoint = 0;
    ServiceStatus.dwWaitHint = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCurrentState = SERVICE_STOPPED;
    ServiceStatus.dwControlsAccepted = SERVICE_QUERY_CONFIG;
    ServiceStatus.dwServiceType = SERVICE_WIN32;
    ServiceStatus.dwWin32ExitCode = dwExitCode;

    /* Register a handler that will do nothing while we are being stopped */
    scHandle = RegisterServiceCtrlHandlerW(lpServiceName, Some(DummySvchostCtrlHandler));
    if !scHandle.is_null() {
        /* Stop us */
        if SetServiceStatus(scHandle, core::ptr::addr_of_mut!(ServiceStatus)) == 0 {
            /* Tell the debugger if this didn't work */
            DBG_ERR!("AbortSvchostService: SetServiceStatus error %ld\n",
                GetLastError(),
            );
        }
    } else {
        /* Tell the debugger if we couldn't register the handler */
        DBG_ERR!("AbortSvchostService: RegisterServiceCtrlHandler failed %d\n",
            GetLastError(),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetServiceDllFunction(
    pDll: PSVCHOST_DLL,
    lpProcName: LPCSTR,
    lpdwError: PDWORD,
) -> PVOID {
    let mut hModule: HMODULE;
    let mut lpAddress: PVOID = core::ptr::null_mut();
    let mut ulCookie: ULONG_PTR = 0;

    /* Activate the context */
    if ActivateActCtx((*pDll).hActCtx, &mut ulCookie as *mut ULONG_PTR) == FALSE as BOOL
    {
        /* We couldn't, bail out */
        if !lpdwError.is_null() {
            *lpdwError = GetLastError();
        }
        DBG_ERR!("ActivateActCtx for %ws failed.  Error %d.\n",
            (*pDll).pszDllPath,
            GetLastError(),
        );
        return lpAddress;
    }

    /* Check if we already have a loaded module for this service */
    hModule = (*pDll).hModule as HMODULE;
    if hModule.is_null() {
        /* We don't -- load it */
        hModule = LoadLibraryExW(
            (*pDll).pszDllPath,
            core::ptr::null_mut(),
            LOAD_WITH_ALTERED_SEARCH_PATH,
        ) as HMODULE;
        if hModule.is_null() {
            /* We failed to load it, bail out */
            if !lpdwError.is_null() {
                *lpdwError = GetLastError();
            }
            DBG_ERR!("LoadLibrary(%ws) failed. Error %d.\n",
                (*pDll).pszDllPath,
                GetLastError(),
            );
            DeactivateActCtx(0, ulCookie);
            return lpAddress;
        }

        /* Loaded! */
        (*pDll).hModule = hModule as HINSTANCE;
    }

    /* Next, get the address being looked up*/
    lpAddress = core::mem::transmute::<FARPROC, PVOID>(GetProcAddress(
        hModule as HINSTANCE,
        lpProcName,
    ));
    if lpAddress.is_null() && !lpdwError.is_null() {
        /* We couldn't find it, write the error code and a debug statement */
        *lpdwError = GetLastError();
        DBG_ERR!("GetProcAddress (%s) failed on DLL %ws.  Error %d.\n",
            lpProcName,
            (*pDll).pszDllPath,
            GetLastError(),
        );
    }

    /* All done, get rid of the activation context */
    DeactivateActCtx(0, ulCookie);
    return lpAddress;
}

#[no_mangle]
pub unsafe extern "system" fn FindDll(
    pszManifestPath: LPCWSTR,
    pszDllPath: LPCWSTR,
    pService: PSVCHOST_SERVICE,
) -> PSVCHOST_DLL {
    let mut pDll: PSVCHOST_DLL;
    let mut pFoundDll: PSVCHOST_DLL = core::ptr::null_mut();
    let mut pNextEntry: PLIST_ENTRY;

    /* Lock the DLL database */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Scan through the database */
    pNextEntry = (*core::ptr::addr_of_mut!(DllList)).Flink;
    while pNextEntry != core::ptr::addr_of_mut!(DllList) {
        /* Search for an existing DLL with the same parameters */
        pDll = CONTAINING_RECORD!(pNextEntry, SVCHOST_DLL, DllList);
        if (lstrcmpW((*pDll).pszDllPath, pszDllPath) == 0) &&
            (lstrcmpW((*pDll).pszManifestPath, pszManifestPath) == 0) &&
            (lstrcmpW((*(*pDll).pService).pszServiceName, (*pService).pszServiceName) == 0)
        {
            /* Found it! */
            pFoundDll = pDll;
            break;
        }

        /* Keep searching */
        pNextEntry = (*pNextEntry).Flink;
    }

    /* All done, release the lock and return what we found, if anything */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));
    return pFoundDll;
}
#[no_mangle]
pub unsafe extern "system" fn AddDll(
    pszManifestPath: LPCWSTR,
    pszDllPath: LPCWSTR,
    pService: PSVCHOST_SERVICE,
    lpdwError: PDWORD,
) -> PSVCHOST_DLL {
    let mut pDll: PSVCHOST_DLL;
    let mut nDllPathLength: ULONG;
    let mut nManifestPathLength: ULONG;

    /* Compute the length of the two paths */
    nDllPathLength = lstrlenW(pszDllPath) as ULONG;
    nManifestPathLength = lstrlenW(pszManifestPath) as ULONG;

    /* Allocate the DLL entry for this service */
    pDll = MemAlloc(
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::size_of::<_SVCHOST_DLL>()
            + (nDllPathLength + nManifestPathLength) as usize * core::mem::size_of::<WCHAR>()
            + 2 as usize * core::mem::size_of_val(&UNICODE_NULL))
            as DWORD,
    ) as PSVCHOST_DLL;
    if pDll.is_null() {
        /* Bail out with failure */
        *lpdwError = ERROR_NOT_ENOUGH_MEMORY as DWORD;
        return pDll;
    }

    /* Put the DLL path right after the DLL entry */
    (*pDll).pszDllPath = pDll.add(1 as usize) as LPCWSTR;

    /* Put the manifest path right past the DLL path (note the pointer math) */
    (*pDll).pszManifestPath = (*pDll)
        .pszDllPath
        .add(nDllPathLength as usize)
        .add(1 as usize);

    /* Now copy both paths */
    CopyMemory!(
        (*pDll).pszDllPath as PVOID,
        pszDllPath as PVOID,
        core::mem::size_of::<WCHAR>() * nDllPathLength as usize,
    );
    CopyMemory!(
        (*pDll).pszManifestPath as PVOID,
        pszManifestPath as PVOID,
        core::mem::size_of::<WCHAR>() * nManifestPathLength as usize,
    );

    /* Now set the service, and make sure both paths are NULL-terminated */
    (*pDll).pService = pService;

    /* Finally, add the entry to the DLL database, while holding the lock */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));
    InsertTailList(
        core::ptr::addr_of_mut!(DllList),
        core::ptr::addr_of_mut!((*pDll).DllList),
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* And return it */
    return pDll;
}

#[no_mangle]
pub unsafe extern "system" fn GetServiceMainFunctions(
    pService: PSVCHOST_SERVICE,
    pServiceMain: *mut PVOID,
    pPushServiceGlobals: *mut PVOID,
    lpdwError: PDWORD,
) {
    let mut dwError: DWORD = 0;
    let mut cbDllLength: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut pDll: PSVCHOST_DLL = core::ptr::null_mut();
    let mut actCtx: ACTCTXW = core::mem::zeroed();
    let mut pszDllPath: LPCWSTR = core::ptr::null();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hActCtx: HANDLE = core::ptr::null_mut();
    let mut lpData: LPWSTR = core::ptr::null_mut();
    let mut szDllBuffer: [WCHAR; (MAX_PATH + 2) as usize] =
        [0; (MAX_PATH + 2) as usize];
    let mut szManifestBuffer: [WCHAR; (MAX_PATH + 2) as usize] =
        [0; (MAX_PATH + 2) as usize];

    /* Initialize the activation context we might need later */
    RtlZeroMemory!(
        core::ptr::addr_of_mut!(actCtx) as PVOID,
        core::mem::size_of::<ACTCTXW>(),
    );
    actCtx.cbSize = core::mem::size_of::<ACTCTXW>() as DWORD;

    /* We clean these up in our failure path so initialize them to NULL here */
    hActCtx = core::ptr::null_mut();
    hKey = core::ptr::null_mut();
    lpData = core::ptr::null_mut();
    *lpdwError = (ERROR_SUCCESS) as DWORD;

    /* Null terminate the string buffers */
    szDllBuffer[(0) as usize] = (UNICODE_NULL) as WCHAR;
    szManifestBuffer[(0) as usize] = (UNICODE_NULL) as WCHAR;

    'quickie: {
        'havedll: {
            /* Do we already have a DLL ready to go for this service? */
            pDll = (*pService).pDll;
            if !pDll.is_null() {
                break 'havedll;
            }

            /* Nope, we're starting from scratch. Open a handle to parameters key */
            dwError = OpenServiceParametersKey(
                (*pService).pszServiceName,
                core::ptr::addr_of_mut!(hKey),
            );
            if dwError != 0 {
                *lpdwError = dwError;
                if !(*lpdwError != 0) {
                    RtlAssert(
                        b"*lpdwError != NO_ERROR\0".as_ptr() as PVOID,
                        b"base/services/svchost/svchost.c\0"
                            .as_ptr() as PVOID,
                        858,
                        core::ptr::null_mut(),
                    );
                }
                break 'quickie;
            }

            /* Allocate enough space to hold a unicode path (NULL-terminated) */
            cbData = (MAX_PATH as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD)
                + (core::mem::size_of_val(&UNICODE_NULL) as DWORD);
            lpData = MemAlloc(
                (0) as DWORD,
                cbData,
            ) as LPWSTR;
            if lpData.is_null() {
                /* No memory, bail out */
                *lpdwError = (ERROR_NOT_ENOUGH_MEMORY) as DWORD;
                break 'quickie;
            }

            /* Query the DLL path */
            *lpData.add((0) as usize) = (UNICODE_NULL) as WCHAR;
            dwError = RegQueryValueExW(
                hKey,
                RUSTCONV_WSTR_C46A70999F35.as_ptr(),
                (0) as *mut DWORD,
                core::ptr::addr_of_mut!(dwType),
                lpData as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            ) as DWORD;
            if dwError != (ERROR_SUCCESS) as DWORD {
                *lpdwError = dwError;
                DBG_ERR!("RegQueryValueEx for the ServiceDll parameter of the %ws service returned %u\n",
                    (*pService).pszServiceName,
                    dwError,
                );
                break 'quickie;
            }

            /* Is the registry data valid and present? */
            if (dwType != (REG_EXPAND_SZ) as DWORD)
                || (*lpData.add((0) as usize)
                    == (UNICODE_NULL) as WCHAR)
            {
                /* Nope, bail out */
                *lpdwError = (ERROR_FILE_NOT_FOUND) as DWORD;
                DBG_ERR!("The ServiceDll parameter for the %ws service is not of type REG_EXPAND_SZ\n",
                    (*pService).pszServiceName,
                );
                break 'quickie;
            }

            /* Convert the expandable path into an absolute path */
            ExpandEnvironmentStringsW(
                lpData as LPCWSTR,
                core::ptr::addr_of_mut!(szDllBuffer) as *mut WCHAR as LPWSTR,
                (MAX_PATH) as DWORD,
            );
            SvchostCharLowerW(
                core::ptr::addr_of_mut!(szDllBuffer) as *mut WCHAR as LPCWSTR,
            );

            /* Check if the service has a manifest file associated with it */
            cbData = (MAX_PATH as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD)
                + (core::mem::size_of_val(&UNICODE_NULL) as DWORD);
            dwError = RegQueryValueExW(
                hKey,
                RUSTCONV_WSTR_7230EE40A9B6.as_ptr(),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(dwType),
                lpData as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            ) as DWORD;
            if dwError != (ERROR_SUCCESS) as DWORD {
                /* Did we fail because one wasn't set? */
                if (dwError != (ERROR_PATH_NOT_FOUND) as DWORD)
                    && (dwError != (ERROR_FILE_NOT_FOUND) as DWORD)
                {
                    /* We failed for some other reason, bail out */
                    *lpdwError = dwError;
                    DBG_ERR!("RegQueryValueEx for the ServiceManifest parameter of the %ws service returned %u\n",
                        (*pService).pszServiceName,
                        dwError,
                    );
                    break 'quickie;
                }

                /* We have no manifest, make sure the buffer is empty */
                szManifestBuffer[(0) as usize] =
                    (UNICODE_NULL) as WCHAR;

                /* We're done with this buffer */
                MemFree(lpData as LPVOID);
                lpData = core::ptr::null_mut();

                /* Use the whole DLL path, since we don't have a manifest */
                pszDllPath = core::ptr::addr_of_mut!(szDllBuffer) as *mut WCHAR as LPCWSTR;
            } else {
                /* Do we have invalid registry data? */
                if (dwType != (REG_EXPAND_SZ) as DWORD)
                    || (*lpData == (UNICODE_NULL) as WCHAR)
                {
                    /* Yes, pretend there's no manifest and bail out */
                    *lpdwError = (ERROR_FILE_NOT_FOUND) as DWORD;
                    DBG_ERR!("The ServiceManifest parameter for the %ws service is not of type REG_EXPAND_SZ\n",
                        (*pService).pszServiceName,
                    );
                    break 'quickie;
                }

                /* Expand the string into our stack buffer */
                ExpandEnvironmentStringsW(
                    lpData as LPCWSTR,
                    core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR as LPWSTR,
                    (MAX_PATH) as DWORD,
                );

                /* We no longer need the heap buffer*/
                MemFree(lpData as LPVOID);
                lpData = core::ptr::null_mut();

                /* Lowercase the manifest path */
                SvchostCharLowerW(
                    core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR as LPCWSTR,
                );

                /* Now loop over the DLL path */
                cbDllLength = lstrlenW(
                    core::ptr::addr_of_mut!(szDllBuffer) as *mut WCHAR as LPCWSTR,
                ) as DWORD;
                while cbDllLength != 0 {
                    /* From the end, until we find the first path separator */
                    if szDllBuffer[cbDllLength as usize] == '\\' as WCHAR
                        || szDllBuffer[cbDllLength as usize] == '/' as WCHAR
                    {
                        /* Use just a short name (cut out the rest of the path) */
                        pszDllPath = (core::ptr::addr_of_mut!(szDllBuffer) as *mut WCHAR)
                            .add((cbDllLength + (1) as DWORD) as usize)
                            as LPCWSTR;
                        break;
                    }

                    /* Try the next character */
                    cbDllLength = cbDllLength.wrapping_sub(1);
                }
            }

            /* See if we already have a matching DLL and manifest for this service */
            pDll = FindDll(
                core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR as LPCWSTR,
                pszDllPath,
                pService,
            );
            if pDll.is_null() {
                /* We don't have it yet -- does this DLL have a manifest? */
                if szManifestBuffer[(0) as usize]
                    != (UNICODE_NULL) as WCHAR
                {
                    /* Create an activation context for it */
                    actCtx.lpSource =
                        core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR as LPCWSTR;
                    hActCtx = CreateActCtxW(core::ptr::addr_of_mut!(actCtx) as PCACTCTXW);
                    if hActCtx == (INVALID_HANDLE_VALUE) as HANDLE {
                        /* We've failed to create one, bail out */
                        *lpdwError = GetLastError();
                        DBG_ERR!("CreateActCtxW(%ws) for the %ws service returned %u\n",
                            core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR,
                            (*pService).pszServiceName,
                            *lpdwError,
                        );
                        break 'quickie;
                    }
                }

                /* Add this new DLL into the database */
                pDll = AddDll(
                    core::ptr::addr_of_mut!(szManifestBuffer) as *mut WCHAR as LPCWSTR,
                    pszDllPath,
                    pService,
                    lpdwError,
                );
                if !pDll.is_null() {
                    /* Save the activation context and zero it so we don't release later */
                    (*pDll).hActCtx = hActCtx;
                    hActCtx = core::ptr::null_mut();
                }
            }

            /* We either found, added, or failed to add, the DLL for this service */
            if !(*pService).pDll.is_null() {
                RtlAssert(
                    b"!pService->pDll\0".as_ptr() as PVOID,
                    b"base/services/svchost/svchost.c\0"
                        .as_ptr() as PVOID,
                    1011,
                    core::ptr::null_mut(),
                );
            }
            (*pService).pDll = pDll;

            /* In all cases, we will query the ServiceMain function, however*/
            RegQueryStringA(
                hKey,
                RUSTCONV_WSTR_2948F33B5A9E.as_ptr(),
                (REG_SZ) as DWORD,
                core::ptr::addr_of_mut!((*pService).pszServiceMain),
            );

            /* And now we'll check if we were able to create it earlier (or find it) */
            if (*pService).pDll.is_null() {
                /* We were not, so there's no point in going on */
                if !(*lpdwError != 0) {
                    RtlAssert(
                        b"*lpdwError != NO_ERROR\0".as_ptr() as PVOID,
                        b"base/services/svchost/svchost.c\0"
                            .as_ptr() as PVOID,
                        1024,
                        core::ptr::null_mut(),
                    );
                }
                break 'quickie;
            }
        }

        /* We do have a valid DLL, so now get the service main routine out of it */
        /* HaveDll: */
        *pServiceMain = GetServiceDllFunction(
            pDll,
            if !(*pService).pszServiceMain.is_null() {
                (*pService).pszServiceMain
            } else {
                b"ServiceMain\0".as_ptr() as LPCSTR
            },
            lpdwError,
        );

        /* And now get the globals routine out of it (this one is optional) */
        *pPushServiceGlobals = GetServiceDllFunction(
            pDll,
            b"SvchostPushServiceGlobals\0".as_ptr() as LPCSTR,
            core::ptr::null_mut::<DWORD>(),
        );
    }

    /* Quickie: We're done, cleanup any resources we left behind */
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    if !lpData.is_null() {
        MemFree(lpData as LPVOID);
    }
    if !hActCtx.is_null() && hActCtx != (INVALID_HANDLE_VALUE) as HANDLE {
        ReleaseActCtx(hActCtx);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ServiceStarter(
    mut dwNumServicesArgs: DWORD,
    mut lpServiceArgVectors: *mut LPWSTR,
) {
    let mut i: DWORD = 0;
    let mut dwError: DWORD = ERROR_FILE_NOT_FOUND as DWORD;
    let mut ServiceEntry: PSVCHOST_SERVICE = core::ptr::null_mut();
    let mut lpServiceName: LPCWSTR = core::ptr::null();
    let mut ServiceMain: LPSERVICE_MAIN_FUNCTIONW = None;
    let mut ServiceInitGlobals: PSVCHOST_INIT_GLOBALS = None;

    /* Hold the lock while we start a service */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Get this service's name, and loop the database */
    lpServiceName = *lpServiceArgVectors as LPCWSTR;
    i = 0 as DWORD;
    while i < ServiceCount {
        /* Try to find a match by name */
        if lstrcmpiW(lpServiceName, (*ServiceArray.offset(i as isize)).pszServiceName) == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    /* Check if we didn't find it */
    if i == ServiceCount {
        /* This looks like a bogus attempt, bail out */
        LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));
        return;
    }

    /* We have it */
    ServiceEntry = ServiceArray.offset(i as isize) as *mut _SVCHOST_SERVICE as PSVCHOST_SERVICE;

    /* Should we breakpoint before loading this service? */
    if FDebugBreakForService(lpServiceName) != FALSE as i32 {
        /* Yep -- do it */
        DBG_TRACE!("[SVCHOST] %lx.%lx: Attaching debugger before getting ServiceMain for %ws...\n",
            GetCurrentProcessId(),
            GetCurrentThreadId(),
            lpServiceName,
        );
        DebugBreak();
    }

    /* Get the callbacks for this service */
    GetServiceMainFunctions(
        ServiceArray.offset(i as isize) as PSVCHOST_SERVICE,
        core::ptr::addr_of_mut!(ServiceMain) as *mut PVOID,
        core::ptr::addr_of_mut!(ServiceInitGlobals) as *mut PVOID,
        core::ptr::addr_of_mut!(dwError),
    );

    /* If this service is valid and needs globals, but we don't have them... */
    if (ServiceMain.is_some())
        && (ServiceInitGlobals.is_some())
        && (g_pSvchostSharedGlobals.is_null())
    {
        /* Go and build them for the first time! */
        SvchostBuildSharedGlobals();
    }

    /* We call a service if it has a main, or if wants globals and we have them */
    if ((ServiceInitGlobals.is_some()) && (!g_pSvchostSharedGlobals.is_null()))
        || ((ServiceMain.is_some()) && (ServiceInitGlobals.is_none()))
    {
        /* Therefore, make sure it won't be unloaded as we drop the lock */
        (*ServiceEntry).cServiceActiveThreadCount += 1;
    }

    /* Drop the lock while we call into the service DLL */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* First: does this service want globals? */
    if ServiceInitGlobals.is_some() {
        /* Do we have any to give? */
        if !g_pSvchostSharedGlobals.is_null() {
            /* Yes, push them to the service */
            (ServiceInitGlobals.unwrap_unchecked())(g_pSvchostSharedGlobals);

            /* Does the service NOT have an entrypoint? */
            if ServiceMain.is_none() {
                /* We're going to abort loading, it serves no use */
                if !lpServiceName.is_null() {
                    AbortSvchostService(lpServiceName, dwError);
                }

                /* And drop a reference count since it's not active anymore */
                UnloadServiceDll(ServiceEntry);
            }
        } else if !lpServiceName.is_null() {
            /* We have no globals to give, so don't even bother calling it */
            AbortSvchostService(lpServiceName, dwError);
        }
    }

    /* Next up, does it have an entrypoint? */
    if ServiceMain.is_some() {
        /* It does, so call it */
        DBG_TRACE!("[SVCHOST] %lx.%lx: Calling ServiceMain for %ws...\n",
            GetCurrentProcessId(),
            GetCurrentThreadId(),
            lpServiceName,
        );
        (ServiceMain.unwrap_unchecked())(dwNumServicesArgs, lpServiceArgVectors);

        /* We're out of the service now, so we can drop a reference */
        UnloadServiceDll(ServiceEntry);
    } else if !lpServiceName.is_null() {
        /* It has no entrypoint, so abort its launch */
        AbortSvchostService(lpServiceName, dwError);
    }
}
#[no_mangle]
pub unsafe extern "system" fn BuildServiceTable() -> *mut SERVICE_TABLE_ENTRYW {
    let mut pServiceTable: *mut SERVICE_TABLE_ENTRYW = core::ptr::null_mut();
    let mut i: ULONG = 0;

    /* Acquire the database lock while we go over the services */
    EnterCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Allocate space for a NULL entry at the end as well, Windows needs this */
    pServiceTable = MemAlloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((ServiceCount.wrapping_add(1 as DWORD)) as usize
            * core::mem::size_of::<SERVICE_TABLE_ENTRYW>()) as DWORD,
    ) as *mut SERVICE_TABLE_ENTRYW;
    if !pServiceTable.is_null() {
        /* Go over all our registered services */
        i = 0;
        while i < ServiceCount {
            /* And set their parameters in the service table */
            let ref mut fresh19 = (*pServiceTable.offset(i as isize)).lpServiceName;
            *fresh19 = (*ServiceArray.offset(i as isize)).pszServiceName as LPWSTR;
            let ref mut fresh20 = (*pServiceTable.offset(i as isize)).lpServiceProc;
            *fresh20 = Some(ServiceStarter);
            DBG_TRACE!("Added service table entry for %ws\n",
                (*pServiceTable.offset(i as isize)).lpServiceName,
            );
            i = i.wrapping_add(1);
        }
    }

    /* All done, can release the lock now */
    LeaveCriticalSection(core::ptr::addr_of_mut!(ListLock));
    return pServiceTable;
}

#[no_mangle]
pub unsafe extern "system" fn CallPerInstanceInitFunctions(pOptions: PSVCHOST_OPTIONS) -> BOOLEAN {
    let mut pNtHeaders: PIMAGE_NT_HEADERS = core::ptr::null_mut();
    let mut bResult: BOOLEAN;

    /* Is COM required for this host? */
    if (*pOptions).CoInitializeSecurityParam != 0 {
        /* Yes, initialize COM security and parameters */
        bResult = (InitializeSecurity(
            (*pOptions).CoInitializeSecurityParam,
            (*pOptions).AuthenticationLevel,
            (*pOptions).ImpersonationLevel as DWORD,
            (*pOptions).AuthenticationCapabilities,
        )) as BOOLEAN;
        if bResult == FALSE as BOOLEAN {
            return FALSE as BOOLEAN;
        }
    }

    /* Do we have a custom RPC stack size? */
    if (*pOptions).DefaultRpcStackSize != 0 {
        /* Yes, go set it */
        RpcMgmtSetServerStackSize(
            ((*pOptions).DefaultRpcStackSize as ULONG) << 10,
        );
    } else {
        /* Nope, get the NT headers from the image */
        pNtHeaders = RtlImageNtHeader((*NtCurrentPeb()).ImageBaseAddress);
        if !pNtHeaders.is_null() {
            /* And just use whatever the default thread stack is */
            RpcMgmtSetServerStackSize((*pNtHeaders).OptionalHeader.SizeOfStackCommit as ULONG);
        }
    }

    /* Is this host holding critical services? */
    if (*pOptions).SystemCritical != FALSE as BOOLEAN {
        /* Mark the process as critical if so */
        RtlSetProcessIsCritical(
            TRUE as BOOLEAN,
            core::ptr::null_mut(),
            TRUE as BOOLEAN,
        );
    }

    /* All done */
    return TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn wmainCRTStartup() {
    let mut lpOptions: PSVCHOST_OPTIONS = core::ptr::null_mut();
    let mut pServiceTable: *mut SERVICE_TABLE_ENTRYW = core::ptr::null_mut();
    let mut pszCmdLine: LPWSTR = core::ptr::null_mut();

    /* Set a generic SEH filter and hard fail all critical errors */
    SetUnhandledExceptionFilter(Some(SvchostUnhandledExceptionFilter));
    SetErrorMode(SEM_FAILCRITICALERRORS);

    /* Initialize the heap allocator */
    MemInit(GetProcessHeap());

    /* Initialize the DLL database and lock */
    InitializeListHead(core::ptr::addr_of_mut!(DllList));
    InitializeCriticalSection(core::ptr::addr_of_mut!(ListLock));

    /* Get the command-line and parse it to get the service group */
    pszCmdLine = GetCommandLineW();
    lpOptions = BuildCommandOptions(pszCmdLine);
    /* Loud diagnosis (2026-07 perf investigation): svchost has been exiting
     * 0 within ~0.4s of spawn on every boot, costing the SCM a full 31s
     * pipe-connect timeout per svchost-hosted service (DcomLaunch et al).
     * Print what this process actually SEES so the failing layer (empty
     * cmdline from CreateProcessW/PEB vs option-parse vs service table) is
     * attributable from the boot serial log. */
    DBG_ERR!("wmain: cmdline=\"%ws\" options=%p group=%ws\n",
        if !pszCmdLine.is_null() {
            pszCmdLine as *const u16
        } else {
            RUSTCONV_WSTR_99C4EAFCB663.as_ptr()
        },
        lpOptions,
        if !lpOptions.is_null()
            && (*lpOptions).HasServiceGroup != 0
            && !(*lpOptions).ServiceGroupName.is_null()
        {
            (*lpOptions).ServiceGroupName as *const u16
        } else {
            RUSTCONV_WSTR_2A0A7FF1C0E2.as_ptr()
        },
    );
    if lpOptions.is_null() {
        /* Without a valid command-line, there's nothing for us to do */
        DBG_ERR!("wmain: NO valid options — ExitProcess(0) (cmdline was \"%ws\")\n",
            if !pszCmdLine.is_null() {
                pszCmdLine as *const u16
            } else {
                RUSTCONV_WSTR_99C4EAFCB663.as_ptr()
            },
        );
        ExitProcess(0);
    }

    /* Now use the service group information to lookup all the services */
    BuildServiceArray(lpOptions);

    /* Convert the list of services in this group to the SCM format */
    pServiceTable = BuildServiceTable();
    if pServiceTable.is_null() {
        /* This is critical, bail out without it */
        DBG_ERR!("wmain: BuildServiceTable returned NULL (%d services) — exiting\n",
            ServiceCount,
        );
        MemFree(lpOptions as LPVOID);
        return;
    }

    /* Initialize COM and RPC as needed for this service group */
    if CallPerInstanceInitFunctions(lpOptions) == FALSE as BOOLEAN {
        /* Exit with a special code indicating COM/RPC setup has failed */
        DBG_ERR!("%s",
            b"CallPerInstanceInitFunctions failed -- exiting!\n\0".as_ptr() as *const i8,
        );
        ExitProcess(1);
    }

    /* We're ready to go -- free the options buffer */
    MemFree(lpOptions as LPVOID);

    /* And call into ADVAPI32 to get our services going */
    StartServiceCtrlDispatcherW(pServiceTable);
}
