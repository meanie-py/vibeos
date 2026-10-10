// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0FA88D88CEB2: [u16; 6] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_3DB566BC3DA4: [u16; 4] = [0x0043u16, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_7AF58FDC50E8: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x006F, 0x0072, 0x0074, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A268A999D52D: [u16; 11] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0020, 0x0050, 0x006F, 0x0072, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_C372F2CCF22D: [u16; 4] = [0x004Cu16, 0x0050, 0x0054, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut wszNonspooledPrefix: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"N\0O\0N\0S\0P\0O\0O\0L\0E\0D\0_\0\0\0")
};

static mut cchNonspooledPrefix: DWORD = (::core::mem::size_of::<[WCHAR; 12]>() as usize)
        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
        .wrapping_sub(1 as usize) as DWORD;

static mut dwPortInfo1Offsets: [DWORD; 2] = [
    0 as ::core::ffi::c_ulonglong as LONG as DWORD,
    MAXDWORD as DWORD,
];

static mut dwPortInfo2Offsets: [DWORD; 4] = [
    0 as ::core::ffi::c_ulonglong as LONG as DWORD,
    8 as ::core::ffi::c_ulonglong as LONG as DWORD,
    16 as ::core::ffi::c_ulonglong as LONG as DWORD,
    MAXDWORD as DWORD,
];

unsafe extern "C" fn _GetNonspooledPortName(pwszPortNameWithoutColon: PCWSTR, ppwszNonspooledPortName: *mut PWSTR) -> DWORD {
    let mut cchPortNameWithoutColon: DWORD;

    cchPortNameWithoutColon = wcslen(pwszPortNameWithoutColon as *const WCHAR) as DWORD;

    *ppwszNonspooledPortName = DllAllocSplMem(
        ((cchNonspooledPrefix + cchPortNameWithoutColon + 1) as usize
            * core::mem::size_of::<WCHAR>()) as DWORD,
    ) as PWSTR;
    if (*ppwszNonspooledPortName).is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    CopyMemory!(
        *ppwszNonspooledPortName as *mut core::ffi::c_void,
        core::ptr::addr_of!(wszNonspooledPrefix) as *const core::ffi::c_void,
        (cchNonspooledPrefix as usize) * core::mem::size_of::<WCHAR>(),
    );
    CopyMemory!(
        (*ppwszNonspooledPortName).offset(cchNonspooledPrefix as isize) as *mut core::ffi::c_void,
        pwszPortNameWithoutColon as *const core::ffi::c_void,
        ((cchPortNameWithoutColon + 1) as usize)
            * core::mem::size_of::<WCHAR>(),
    );

    return ERROR_SUCCESS;
}

unsafe extern "C" fn _IsLegacyPort(pwszPortName: PCWSTR, pwszPortType: PCWSTR) -> BOOL {
    let cchPortType: DWORD = 3 as DWORD;
    let mut p: PCWSTR = pwszPortName;

    // The port name must begin with pwszPortType.
    if _wcsnicmp(p as *const wchar_t, pwszPortType as *const wchar_t, cchPortType as size_t) != 0 {
        return FALSE as BOOL;
    }

    p = p.offset(cchPortType as isize);

    // Now an arbitrary number of digits may follow.
    while *p >= 0x0030u16 && *p <= 0x0039u16 {
        p = p.offset(1);
    }

    // Finally, the legacy port must be terminated by a colon.
    if *p != ':' as u16 {
        return FALSE as BOOL;
    }

    // If this is the end of the string, we have a legacy port.
    p = p.offset(1);
    return (*p == 0x0000u16) as BOOL;
}

unsafe extern "C" fn _ClosePortHandles(mut pPort: PLOCALMON_PORT) {
    let mut pwszNonspooledPortName: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pwszPortNameWithoutColon: PWSTR = core::ptr::null_mut::<WCHAR>();
    if (*pPort).hFile == INVALID_HANDLE_VALUE {
        return;
    }
    CloseHandle((*pPort).hFile);
    (*pPort).hFile = INVALID_HANDLE_VALUE;
    if (*pPort).pwszMapping.is_null() {
        return;
    }
    DllFreeSplStr((*pPort).pwszMapping);
    (*pPort).pwszMapping = core::ptr::null_mut::<WCHAR>();
    if GetPortNameWithoutColon(
        (*pPort).pwszPortName as PCWSTR,
        core::ptr::addr_of_mut!(pwszPortNameWithoutColon),
    ) == ERROR_SUCCESS
    {
        if _GetNonspooledPortName(
            pwszPortNameWithoutColon as PCWSTR,
            core::ptr::addr_of_mut!(pwszNonspooledPortName),
        ) == ERROR_SUCCESS
        {
            DefineDosDeviceW(
                DDD_REMOVE_DEFINITION as DWORD,
                pwszNonspooledPortName as LPCWSTR,
                core::ptr::null::<WCHAR>(),
            );
            DllFreeSplMem(pwszNonspooledPortName as PVOID);
        }
        DllFreeSplMem(pwszPortNameWithoutColon as PVOID);
    }
}

unsafe extern "C" fn _CreateNonspooledPort(mut pPort: PLOCALMON_PORT) -> BOOL {
    let wszLocalSlashes: [u16; 5] = [0x005Cu16, 0x005C, 0x002E, 0x005C, 0x0000];
    let cchLocalSlashes: DWORD = (_countof!(wszLocalSlashes) - 1) as DWORD;

    let wszSpoolerNamedPipe: [u16; 27] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0064, 0x0050, 0x0069, 0x0070, 0x0065, 0x005C, 0x0053, 0x0070,
    0x006F, 0x006F, 0x006C, 0x0065, 0x0072, 0x005C, 0x0000,
];
    let cchSpoolerNamedPipe: DWORD = (_countof!(wszSpoolerNamedPipe) - 1) as DWORD;

    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cchPortNameWithoutColon: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut p: PWSTR = core::ptr::null_mut();
    let mut pwszDeviceMappings: PWSTR = core::ptr::null_mut();
    let mut pwszNonspooledFileName: PWSTR = core::ptr::null_mut();
    let mut pwszNonspooledPortName: PWSTR = core::ptr::null_mut();
    let mut pwszPipeName: PWSTR = core::ptr::null_mut();
    let mut pwszPortNameWithoutColon: PWSTR = core::ptr::null_mut();

    'cleanup: {
        // We need the port name without the trailing colon.
        dwErrorCode = GetPortNameWithoutColon(
            (*pPort).pwszPortName as PCWSTR,
            core::ptr::addr_of_mut!(pwszPortNameWithoutColon),
        );
        if dwErrorCode == ERROR_INVALID_PARAMETER {
            // This port has no trailing colon, so we also need no NONSPOOLED mapping for it.
            dwErrorCode = ERROR_SUCCESS;
            break 'cleanup;
        } else if dwErrorCode != ERROR_SUCCESS {
            // Another unexpected failure.
            break 'cleanup;
        }

        cchPortNameWithoutColon = wcslen(pwszPortNameWithoutColon as PCWSTR) as DWORD;

        // The spooler has usually remapped the legacy port to a named pipe of the format in wszSpoolerNamedPipe.
        // Construct the device name of this pipe.
        pwszPipeName = DllAllocSplMem(
            ((cchSpoolerNamedPipe + cchPortNameWithoutColon + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD)) as DWORD,
        ) as PWSTR;
        if pwszPipeName.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        CopyMemory!(
            pwszPipeName as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszSpoolerNamedPipe) as *const core::ffi::c_void,
            (cchSpoolerNamedPipe as usize) * core::mem::size_of::<WCHAR>(),
        );
        CopyMemory!(
            pwszPipeName.add(cchSpoolerNamedPipe as usize) as *mut core::ffi::c_void,
            pwszPortNameWithoutColon as *const core::ffi::c_void,
            ((cchPortNameWithoutColon + 1) as usize)
                * core::mem::size_of::<WCHAR>(),
        );

        // QueryDosDeviceW gives no information about the required buffer size, so MAX_PATH * sizeof(WCHAR) is used.
        pwszDeviceMappings = DllAllocSplMem(
            (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if pwszDeviceMappings.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        // Switch to the SYSTEM context, because we're only interested in creating NONSPOOLED ports for system-wide ports.
        // User-local ports (like _some_ redirected networked ones) aren't remapped by the spooler and can be opened directly.
        hToken = RevertToPrinterSelf();
        if hToken.is_null() {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // QueryDosDeviceW returns the current mapping and a list of prior mappings of this legacy port.
        if QueryDosDeviceW(
            pwszPortNameWithoutColon as LPCWSTR,
            pwszDeviceMappings as LPWSTR,
            MAX_PATH,
        ) == 0
        {
            // No system-wide port exists, so we also need no NONSPOOLED mapping.
            dwErrorCode = ERROR_SUCCESS;
            break 'cleanup;
        }

        // Check if this port has already been opened by _CreateNonspooledPort previously.
        if !(*pPort).pwszMapping.is_null() {
            // In this case, we just need to do something if the mapping has changed.
            if wcscmp(
                (*pPort).pwszMapping as *const WCHAR,
                pwszDeviceMappings as *const WCHAR,
            ) == 0
            {
                // We don't need to do anything in this case.
                dwErrorCode = ERROR_SUCCESS;
                break 'cleanup;
            } else {
                // Close the open file handle and free the memory for pwszMapping before remapping.
                CloseHandle((*pPort).hFile);
                (*pPort).hFile = INVALID_HANDLE_VALUE;

                DllFreeSplStr((*pPort).pwszMapping);
                (*pPort).pwszMapping = core::ptr::null_mut();
            }
        }

        // What we now need for accessing the actual port is the most recent mapping different from the named pipe.
        p = pwszDeviceMappings;

        loop {
            if *p == 0 {
                // We reached the end of the list without finding a mapping.
                break 'cleanup;
            }

            if _wcsicmp(p as *const WCHAR, pwszPipeName as *const WCHAR)
                != 0
            {
                break;
            }

            // Advance to the next mapping in the list.
            p = p.add(
                (wcslen(p as *const WCHAR) + 1) as usize,
            );
        }

        // Create a DOS device "NONSPOOLED_<PortName>" to this mapping, so that CreateFileW can open it.
        dwErrorCode = _GetNonspooledPortName(
            pwszPortNameWithoutColon as PCWSTR,
            core::ptr::addr_of_mut!(pwszNonspooledPortName),
        );
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Delete a possibly existing NONSPOOLED device before creating the new one.
        DefineDosDeviceW(
            DDD_REMOVE_DEFINITION,
            pwszNonspooledPortName as LPCWSTR,
            core::ptr::null::<WCHAR>(),
        );

        if DefineDosDeviceW(
            DDD_RAW_TARGET_PATH,
            pwszNonspooledPortName as LPCWSTR,
            p as LPCWSTR,
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // This is all we needed to do in SYSTEM context.
        ImpersonatePrinterClient(hToken);
        hToken = core::ptr::null_mut();

        // Construct the file name to our created device for CreateFileW.
        pwszNonspooledFileName = DllAllocSplMem(
            ((cchLocalSlashes + cchNonspooledPrefix + cchPortNameWithoutColon + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD)) as DWORD,
        ) as PWSTR;
        if pwszNonspooledFileName.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        CopyMemory!(
            pwszNonspooledFileName as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszLocalSlashes) as *const core::ffi::c_void,
            (cchLocalSlashes as usize) * core::mem::size_of::<WCHAR>(),
        );
        CopyMemory!(
            pwszNonspooledFileName.add(cchLocalSlashes as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszNonspooledPrefix) as *const core::ffi::c_void,
            (cchNonspooledPrefix as usize) * core::mem::size_of::<WCHAR>(),
        );
        CopyMemory!(
            pwszNonspooledFileName
                .add((cchLocalSlashes + cchNonspooledPrefix) as usize) as *mut core::ffi::c_void,
            pwszPortNameWithoutColon as *const core::ffi::c_void,
            ((cchPortNameWithoutColon + 1) as usize)
                * core::mem::size_of::<WCHAR>(),
        );

        // Finally open it for reading and writing.
        (*pPort).hFile = CreateFileW(
            pwszNonspooledFileName as LPCWSTR,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
        if (*pPort).hFile == INVALID_HANDLE_VALUE {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Store the current mapping of the port, so that we can check if it has changed.
        (*pPort).pwszMapping = AllocSplStr(pwszDeviceMappings as PCWSTR);
        if (*pPort).pwszMapping.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        bReturnValue = TRUE as BOOL;
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !hToken.is_null() {
        ImpersonatePrinterClient(hToken);
    }

    if !pwszDeviceMappings.is_null() {
        DllFreeSplMem((pwszDeviceMappings).cast());
    }

    if !pwszNonspooledFileName.is_null() {
        DllFreeSplMem((pwszNonspooledFileName).cast());
    }

    if !pwszNonspooledPortName.is_null() {
        DllFreeSplMem((pwszNonspooledPortName).cast());
    }

    if !pwszPipeName.is_null() {
        DllFreeSplMem((pwszPipeName).cast());
    }

    if !pwszPortNameWithoutColon.is_null() {
        DllFreeSplMem((pwszPortNameWithoutColon).cast());
    }

    SetLastError(dwErrorCode);
    return bReturnValue;
}

unsafe extern "C" fn _FindPort(pLocalmon: PLOCALMON_HANDLE, pwszPortName: PCWSTR) -> PLOCALMON_PORT {
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    pEntry = (*pLocalmon).RegistryPorts.Flink;
    while pEntry != core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts) {
        pPort = (pEntry as usize)
            .wrapping_sub(core::mem::offset_of!(LOCALMON_PORT, Entry))
            as PLOCALMON_PORT;

        if wcscmp((*pPort).pwszPortName, pwszPortName) == 0 {
            return pPort;
        }
        pEntry = (*pEntry).Flink;
    }
    core::ptr::null_mut()
}

unsafe extern "C" fn _LocalmonGetPortLevel1(
    mut pPort: PLOCALMON_PORT,
    mut ppPortInfo: *mut PPORT_INFO_1W,
    mut ppPortInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
) {
    let mut cbPortName: DWORD = 0;
    let mut pwszStrings: [PCWSTR; 1] = [core::ptr::null::<WCHAR>(); 1];

    // Calculate the string lengths.
    if ppPortInfo.is_null() {
        cbPortName = (wcslen((*pPort).pwszPortName as *const wchar_t) as DWORD)
            .wrapping_add(1 as DWORD)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);

        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<PORT_INFO_1W>() as DWORD).wrapping_add(cbPortName),
        );
        return;
    }

    // Set the pName field.
    pwszStrings[0 as usize] = (*pPort).pwszPortName as PCWSTR;

    // Copy the structure and advance to the next one in the output buffer.
    *ppPortInfoEnd = PackStrings(
        &raw mut pwszStrings as *mut PCWSTR,
        *ppPortInfo as PBYTE,
        &raw mut dwPortInfo1Offsets as *mut DWORD,
        *ppPortInfoEnd,
    );
    *ppPortInfo = (*ppPortInfo).offset(1);
}

unsafe extern "C" fn _LocalmonGetPortLevel2(
    mut pPort: PLOCALMON_PORT,
    mut ppPortInfo: *mut PPORT_INFO_2W,
    mut ppPortInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
) {
    let mut cbPortName: DWORD = 0;
    let mut pwszStrings: [PCWSTR; 3] = [core::ptr::null::<WCHAR>(); 3];

    // Calculate the string lengths.
    if ppPortInfo.is_null() {
        cbPortName = ((wcslen((*pPort).pwszPortName as *const wchar_t) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD;

        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<PORT_INFO_2W>() as DWORD)
                .wrapping_add(cbPortName)
                .wrapping_add(cbLocalMonitor as DWORD)
                .wrapping_add(cbLocalPort as DWORD),
        );
        return;
    }

    // All local ports are writable and readable.
    (**ppPortInfo).fPortType = (PORT_TYPE_WRITE | PORT_TYPE_READ) as DWORD;
    (**ppPortInfo).Reserved = 0 as DWORD;

    // Set the pPortName field.
    pwszStrings[0 as usize] = (*pPort).pwszPortName as PCWSTR;

    // Set the pMonitorName field.
    pwszStrings[1 as usize] = pwszLocalMonitor as PWSTR as PCWSTR;

    // Set the pDescription field.
    pwszStrings[2 as usize] = pwszLocalPort as PWSTR as PCWSTR;

    // Copy the structure and advance to the next one in the output buffer.
    *ppPortInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppPortInfo as PBYTE,
        core::ptr::addr_of_mut!(dwPortInfo2Offsets) as *mut DWORD,
        *ppPortInfoEnd,
    );
    *ppPortInfo = (*ppPortInfo).offset(1);
}

unsafe extern "C" fn _SetTransmissionRetryTimeout(pPort: PLOCALMON_PORT) -> BOOL {
    let mut CommTimeouts: COMMTIMEOUTS = core::mem::zeroed();

    // Get the timeout from the port.
    if GetCommTimeouts((*pPort).hFile, core::ptr::addr_of_mut!(CommTimeouts)) == 0 {
        return FALSE as BOOL;
    }

    // Set the timeout using the value from registry.
    CommTimeouts.WriteTotalTimeoutConstant = GetLPTTransmissionRetryTimeout().wrapping_mul(1000 as DWORD);
    SetCommTimeouts((*pPort).hFile, core::ptr::addr_of_mut!(CommTimeouts));

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonClosePort(hPort: HANDLE) -> BOOL {
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    // Sanity checks
    if pPort.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    // Close the file handle, free memory for pwszMapping and delete any NONSPOOLED port.
    _ClosePortHandles(pPort);

    // Close any open printer handle.
    if !(*pPort).hPrinter.is_null() {
        ClosePrinter((*pPort).hPrinter);
        (*pPort).hPrinter = core::ptr::null_mut();
    }

    // Free virtual FILE: ports which were created in LocalmonOpenPort.
    if (*pPort).PortType == PortType_FILE {
        EnterCriticalSection(core::ptr::addr_of_mut!((*(*pPort).pLocalmon).Section));
        RemoveEntryList(core::ptr::addr_of_mut!((*pPort).Entry));
        LeaveCriticalSection(core::ptr::addr_of_mut!((*(*pPort).pLocalmon).Section));
        DllFreeSplMem(pPort as PVOID);
    }

    SetLastError(ERROR_SUCCESS);
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonEndDocPort(hPort: HANDLE) -> BOOL {
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    // Sanity checks
    if pPort.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    // Ending a document requires starting it first :-P
    if (*pPort).bStartedDoc != 0 {
        // Close all ports opened in StartDocPort.
        // That is, all but physical LPT ports (opened in OpenPort).
        if (*pPort).PortType != PortType_PhysicalLPT {
            _ClosePortHandles(pPort);
        }

        // Report our progress.
        SetJobW(
            (*pPort).hPrinter,
            (*pPort).dwJobID,
            0,
            core::ptr::null_mut(),
            JOB_CONTROL_SENT_TO_PRINTER,
        );

        // We're done with the printer.
        ClosePrinter((*pPort).hPrinter);
        (*pPort).hPrinter = core::ptr::null_mut();

        // A new document can now be started again.
        (*pPort).bStartedDoc = FALSE as BOOL;
    }

    SetLastError(ERROR_SUCCESS);
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonEnumPorts(hMonitor: HANDLE, pName: PWSTR, Level: DWORD, mut pPorts: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pPortInfoEnd: PBYTE = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();

    'Cleanup: {
        if pLocalmon.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Begin counting.
        *pcbNeeded = 0;
        *pcReturned = 0;

        EnterCriticalSection(&raw mut (*pLocalmon).Section);

        // Count the required buffer size and the number of ports.
        pEntry = (*pLocalmon).RegistryPorts.Flink as PLIST_ENTRY;
        while pEntry != (&raw mut (*pLocalmon).RegistryPorts as PLIST_ENTRY) {
            pPort = CONTAINING_RECORD!(pEntry, LOCALMON_PORT, Entry);

            if Level == 1 {
                _LocalmonGetPortLevel1(pPort, core::ptr::null_mut(), core::ptr::null_mut(), pcbNeeded);
            } else if Level == 2 {
                _LocalmonGetPortLevel2(pPort, core::ptr::null_mut(), core::ptr::null_mut(), pcbNeeded);
            }
            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            LeaveCriticalSection(&raw mut (*pLocalmon).Section);
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy over the Port information.
        pPortInfoEnd = pPorts.add(*pcbNeeded as usize) as PBYTE;

        pEntry = (*pLocalmon).RegistryPorts.Flink as PLIST_ENTRY;
        while pEntry != (&raw mut (*pLocalmon).RegistryPorts as PLIST_ENTRY) {
            pPort = CONTAINING_RECORD!(pEntry, LOCALMON_PORT, Entry);

            if Level == 1 {
                _LocalmonGetPortLevel1(pPort, &raw mut pPorts as *mut PPORT_INFO_1W, &raw mut pPortInfoEnd, core::ptr::null_mut());
            } else if Level == 2 {
                _LocalmonGetPortLevel2(pPort, &raw mut pPorts as *mut PPORT_INFO_2W, &raw mut pPortInfoEnd, core::ptr::null_mut());
            }

            *pcReturned = (*pcReturned).wrapping_add(1);
            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }

        LeaveCriticalSection(&raw mut (*pLocalmon).Section);
        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonGetPrinterDataFromPort(hPort: HANDLE, ControlID: DWORD, pValueName: PWSTR, lpInBuffer: PWSTR, cbInBuffer: DWORD, lpOutBuffer: PWSTR, cbOutBuffer: DWORD, lpcbReturned: PDWORD) -> BOOL {
    let mut bOpenedPort: BOOL = FALSE as BOOL;
    let mut dwErrorCode: DWORD;
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/monitors/localmon/ports.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalmonGetPrinterDataFromPort\0" as *const u8
                as *const ::core::ffi::c_char,
            636 as ::core::ffi::c_int,
            b"LocalmonGetPrinterDataFromPort(%p, %lu, %p, %p, %lu, %p, %lu, %p)\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            hPort,
            ControlID,
            pValueName,
            lpInBuffer,
            cbInBuffer,
            lpOutBuffer,
            cbOutBuffer,
            lpcbReturned,
        );
    }

    'Cleanup: {
        // Sanity checks
        if pPort.is_null() || ControlID == 0 || lpcbReturned.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // If this is a serial port, a temporary file handle may be opened.
        if (*pPort).PortType as ::core::ffi::c_uint
            == PortType_PhysicalCOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if _CreateNonspooledPort(pPort) != 0 {
                bOpenedPort = TRUE as BOOL;
            } else if GetLastError() != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        } else if (*pPort).hFile == INVALID_HANDLE_VALUE {
            // All other port types need to be opened already.
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Pass the parameters to DeviceIoControl.
        if DeviceIoControl(
            (*pPort).hFile,
            ControlID,
            lpInBuffer as LPVOID,
            cbInBuffer,
            lpOutBuffer as LPVOID,
            cbOutBuffer,
            lpcbReturned as LPDWORD,
            ::core::ptr::null_mut::<_OVERLAPPED>(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/monitors/localmon/ports.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LocalmonGetPrinterDataFromPort\0" as *const u8
                        as *const ::core::ffi::c_char,
                    669 as ::core::ffi::c_int,
                    b"DeviceIoControl failed with error %lu!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    dwErrorCode,
                );
            }
            break 'Cleanup;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if bOpenedPort != 0 {
        _ClosePortHandles(pPort);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonOpenPort(hMonitor: HANDLE, pName: PWSTR, pHandle: PHANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pPort: PLOCALMON_PORT;

    'cleanup: {
        // Sanity checks
        if pLocalmon.is_null() || pName.is_null() || pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        EnterCriticalSection(&raw mut (*pLocalmon).Section);

        // Check if this is a FILE: port.
        if _wcsicmp(pName as PCWSTR, RUSTCONV_WSTR_0FA88D88CEB2.as_ptr()) == 0 {
            // For FILE:, we create a virtual port for each request.
            pPort = DllAllocSplMem(core::mem::size_of::<LOCALMON_PORT>() as DWORD) as PLOCALMON_PORT;
            (*pPort).pLocalmon = pLocalmon;
            (*pPort).PortType = PortType_FILE;
            (*pPort).hFile = INVALID_HANDLE_VALUE;

            // Add it to the list of file ports.
            InsertTailList(&raw mut (*pLocalmon).FilePorts, &raw mut (*pPort).Entry);
        } else {
            // Check if the port name is valid.
            pPort = _FindPort(pLocalmon, pName as PCWSTR);
            if pPort.is_null() {
                LeaveCriticalSection(&raw mut (*pLocalmon).Section);
                dwErrorCode = ERROR_UNKNOWN_PORT as DWORD;
                break 'cleanup;
            }

            // Even if this API is called OpenPort, port file handles aren't always opened here :-P
            // Windows only does this for physical LPT ports here to enable bidirectional communication with the printer outside of jobs (using ReadPort and WritePort).
            // The others are only opened per job in StartDocPort.
            if _IsLegacyPort(pName as PCWSTR, RUSTCONV_WSTR_C372F2CCF22D.as_ptr()) != 0 {
                // Try to create a NONSPOOLED port and open it.
                if _CreateNonspooledPort(pPort) != 0 {
                    // Set the transmission retry timeout for the ReadPort and WritePort calls.
                    // This also checks if this port is a physical one.
                    if _SetTransmissionRetryTimeout(pPort) != 0 {
                        // This is definitely a physical LPT port!
                        (*pPort).PortType = PortType_PhysicalLPT;
                    } else {
                        // This is no physical port, so don't keep its handle open.
                        _ClosePortHandles(pPort);
                    }
                } else if GetLastError() != ERROR_SUCCESS as DWORD {
                    LeaveCriticalSection(&raw mut (*pLocalmon).Section);
                    dwErrorCode = GetLastError();
                    break 'cleanup;
                }
            } else if _IsLegacyPort(pName as PCWSTR, RUSTCONV_WSTR_3DB566BC3DA4.as_ptr()) != 0 {
                // COM ports can't be redirected over the network, so this is a physical one.
                (*pPort).PortType = PortType_PhysicalCOM;
            }
        }

        LeaveCriticalSection(&raw mut (*pLocalmon).Section);

        // Return our fetched LOCALMON_PORT structure in the handle.
        *pHandle = pPort as HANDLE;
        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonSetPortTimeOuts(hPort: HANDLE, lpCTO: LPCOMMTIMEOUTS, Reserved: DWORD) -> BOOL {
    let mut bOpenedPort: BOOL = FALSE as BOOL;
    let mut dwErrorCode: DWORD = 0;
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    'Cleanup: {
        // Sanity checks
        if pPort.is_null() || lpCTO.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // If this is a serial port, a temporary file handle may be opened.
        if (*pPort).PortType == PortType_PhysicalCOM {
            if _CreateNonspooledPort(pPort) != 0 {
                bOpenedPort = TRUE as BOOL;
            } else if GetLastError() != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        } else if (*pPort).hFile == INVALID_HANDLE_VALUE {
            // All other port types need to be opened already.
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Finally pass the parameters to SetCommTimeouts.
        if SetCommTimeouts((*pPort).hFile, lpCTO) == 0 {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if bOpenedPort != 0 {
        _ClosePortHandles(pPort);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonReadPort(hPort: HANDLE, pBuffer: PBYTE, cbBuffer: DWORD, pcbRead: PDWORD) -> BOOL {
    let mut bOpenedPort: BOOL = FALSE as BOOL;
    let mut dwErrorCode: DWORD = 0;
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    'Cleanup: {
        // Sanity checks
        if pPort.is_null() || (cbBuffer != 0 && pBuffer.is_null()) || pcbRead.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Reading is only supported for physical ports.
        if (*pPort).PortType != PortType_PhysicalCOM && (*pPort).PortType != PortType_PhysicalLPT {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // If this is a serial port, a temporary file handle may be opened.
        if (*pPort).PortType == PortType_PhysicalCOM {
            if _CreateNonspooledPort(pPort) != 0 {
                bOpenedPort = TRUE as BOOL;
            } else if GetLastError() != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        }

        // Pass the parameters to ReadFile.
        if ReadFile(
            (*pPort).hFile,
            pBuffer as PVOID,
            cbBuffer,
            pcbRead,
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }
    }

    if bOpenedPort != 0 {
        _ClosePortHandles(pPort);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonStartDocPort(hPort: HANDLE, pPrinterName: PWSTR, JobId: DWORD, Level: DWORD, pDocInfo: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pDocInfo1: PDOC_INFO_1W = pDocInfo as PDOC_INFO_1W;
    let pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    'Cleanup: {
        // Sanity checks
        if pPort.is_null()
            || pPrinterName.is_null()
            || ((*pPort).PortType == PortType_FILE
                && (pDocInfo1.is_null()
                    || (*pDocInfo1).pOutputFile.is_null()
                    || *(*pDocInfo1).pOutputFile == 0))
        {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        if Level > 2 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Calling StartDocPort multiple times isn't considered a failure, but we don't need to do anything then.
        if (*pPort).bStartedDoc != 0 {
            dwErrorCode = ERROR_SUCCESS;
            break 'Cleanup;
        }

        // Open a handle to the given printer for later reporting our progress using SetJobW.
        if OpenPrinterW(
            pPrinterName,
            core::ptr::addr_of_mut!((*pPort).hPrinter),
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // We need our Job ID for SetJobW as well.
        (*pPort).dwJobID = JobId;

        // Check the port type.
        if (*pPort).PortType == PortType_PhysicalLPT {
            // Update the NONSPOOLED mapping if the port mapping has changed since our OpenPort call.
            if _CreateNonspooledPort(pPort) == 0
                && GetLastError() != ERROR_SUCCESS
            {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // Update the transmission retry timeout as well.
            _SetTransmissionRetryTimeout(pPort);
        } else if (*pPort).PortType == PortType_FILE {
            // This is a FILE: port. Open the output file given in the Document Info.
            (*pPort).hFile = CreateFileW(
                (*pDocInfo1).pOutputFile as LPCWSTR,
                GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::null_mut(),
                CREATE_ALWAYS,
                0,
                core::ptr::null_mut(),
            );
            if (*pPort).hFile == INVALID_HANDLE_VALUE {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        } else {
            // This can be:
            //    - a physical COM port
            //    - a non-physical LPT port (e.g. with "net use LPT1 ...")
            //    - any other port (e.g. a file or a shared printer installed as a local port)
            //
            // For all these cases, we try to create a NONSPOOLED port per job.
            // If _CreateNonspooledPort reports that no NONSPOOLED port is necessary, we can just open the port name.
            if _CreateNonspooledPort(pPort) == 0 {
                if GetLastError() == ERROR_SUCCESS {
                    (*pPort).hFile = CreateFileW(
                        (*pPort).pwszPortName as LPCWSTR,
                        GENERIC_WRITE,
                        FILE_SHARE_READ,
                        core::ptr::null_mut(),
                        CREATE_ALWAYS,
                        0,
                        core::ptr::null_mut(),
                    );
                    if (*pPort).hFile == INVALID_HANDLE_VALUE {
                        dwErrorCode = GetLastError();
                        break 'Cleanup;
                    }
                } else {
                    dwErrorCode = GetLastError();
                    break 'Cleanup;
                }
            }
        }

        // We were successful!
        dwErrorCode = ERROR_SUCCESS;
        (*pPort).bStartedDoc = TRUE as BOOL;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonWritePort(hPort: HANDLE, pBuffer: PBYTE, cbBuf: DWORD, pcbWritten: PDWORD) -> BOOL {
    let mut bOpenedPort: BOOL = FALSE as BOOL;
    let mut dwErrorCode: DWORD = 0;
    let mut pPort: PLOCALMON_PORT = hPort as PLOCALMON_PORT;

    'Cleanup: {
        // Sanity checks
        if pPort.is_null() || (cbBuf != 0 && pBuffer.is_null()) || pcbWritten.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // If this is a serial port, a temporary file handle may be opened.
        if (*pPort).PortType == PortType_PhysicalCOM {
            if _CreateNonspooledPort(pPort) != 0 {
                bOpenedPort = TRUE as BOOL;
            } else if GetLastError() != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }
        } else if (*pPort).hFile == INVALID_HANDLE_VALUE {
            // All other port types need to be opened already.
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Pass the parameters to WriteFile.
        if WriteFile(
            (*pPort).hFile,
            pBuffer as LPCVOID,
            cbBuf,
            pcbWritten as LPDWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // If something was written down, we consider that a success, otherwise it's a timeout.
        if *pcbWritten != 0 {
            dwErrorCode = ERROR_SUCCESS;
        } else {
            dwErrorCode = ERROR_TIMEOUT;
        }
    }

    // Cleanup:
    if bOpenedPort != 0 {
        _ClosePortHandles(pPort);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonAddPortEx(hMonitor: HANDLE, pName: LPWSTR, Level: DWORD, lpBuffer: LPBYTE, lpMonitorName: LPWSTR) -> BOOL {
    let pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwErrorCode: DWORD = 0;
    let mut cbPortName: DWORD = 0;
    let pi: *mut PORT_INFO_1W = lpBuffer as *mut PORT_INFO_1W;

    'Cleanup: {
        // Sanity checks
        if pLocalmon.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        if lpMonitorName.is_null()
            || lstrcmpiW(lpMonitorName as LPCWSTR, RUSTCONV_WSTR_A268A999D52D.as_ptr()) != 0
            || pi.is_null()
            || (*pi).pName.is_null()
            || *(*pi).pName.offset(0 as isize) == '\0' as WCHAR
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE as BOOL;
        }

        if Level != 1 {
            SetLastError(ERROR_INVALID_LEVEL);
            return FALSE as BOOL;
        }

        dwErrorCode = RegOpenKeyW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if dwErrorCode == ERROR_SUCCESS {
            if DoesPortExist((*pi).pName as PCWSTR) != 0 {
                RegCloseKey(hKey);
                SetLastError(ERROR_INVALID_PARAMETER);
                return FALSE as BOOL;
            }

            cbPortName = ((wcslen((*pi).pName as *const u16) as DWORD)
                + 1)
                * core::mem::size_of::<WCHAR>() as DWORD;

            // Create a new LOCALMON_PORT structure for it.
            pPort = DllAllocSplMem(
                (core::mem::size_of::<LOCALMON_PORT>() as DWORD) + cbPortName,
            ) as PLOCALMON_PORT;
            if pPort.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                RegCloseKey(hKey);
                break 'Cleanup;
            }

            (*pPort).Sig = (SIGLCMPORT as u32);
            (*pPort).hFile = INVALID_HANDLE_VALUE;
            (*pPort).pLocalmon = pLocalmon;
            (*pPort).pwszPortName = wcscpy(
                pPort.offset(1 as isize) as PWSTR,
                (*pi).pName as *const u16,
            ) as PWSTR;

            // Insert it into the Registry list.
            InsertTailList(
                core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts),
                core::ptr::addr_of_mut!((*pPort).Entry),
            );

            dwErrorCode = RegSetValueExW(
                hKey,
                (*pi).pName as LPCWSTR,
                0,
                REG_SZ,
                RUSTCONV_WSTR_96A296D224F2.as_ptr() as *const BYTE,
                core::mem::size_of_val(&[0x0000u16]) as DWORD,
            ) as DWORD;
            RegCloseKey(hKey);
        }
    }

    if dwErrorCode != ERROR_SUCCESS {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonAddPort(hMonitor: HANDLE, pName: LPWSTR, hWnd: HWND, pMonitorName: LPWSTR) -> BOOL {
    let mut res: DWORD = 0;
    let mut cbPortName: DWORD = 0;
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    let mut PortName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize]; // Need to use a Dialog to get name.

    res = RegOpenKeyW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(), core::ptr::addr_of_mut!(hroot)) as DWORD;
    if res == ERROR_SUCCESS {
        if DoesPortExist(core::ptr::addr_of_mut!(PortName) as *mut WCHAR as PCWSTR) != 0 {
            RegCloseKey(hroot);
            res = ERROR_ALREADY_EXISTS;
            // goto Cleanup;
            SetLastError(res);
            return (res == ERROR_SUCCESS) as BOOL;
        }

        cbPortName = ((wcslen(core::ptr::addr_of_mut!(PortName) as *mut WCHAR) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD;

        // Create a new LOCALMON_PORT structure for it.
        pPort = DllAllocSplMem(
            (core::mem::size_of::<LOCALMON_PORT>() as usize).wrapping_add(cbPortName as usize) as DWORD,
        ) as PLOCALMON_PORT;
        if pPort.is_null() {
            res = ERROR_NOT_ENOUGH_MEMORY;
            RegCloseKey(hroot);
            // goto Cleanup;
            SetLastError(res);
            return (res == ERROR_SUCCESS) as BOOL;
        }

        (*pPort).Sig = (SIGLCMPORT as u32);
        (*pPort).hFile = INVALID_HANDLE_VALUE;
        (*pPort).pLocalmon = pLocalmon;
        (*pPort).pwszPortName = wcscpy(
            pPort.add(1 as usize) as PWSTR,
            core::ptr::addr_of_mut!(PortName) as *mut WCHAR,
        ) as PWSTR;

        // Insert it into the Registry list.
        InsertTailList(
            core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts),
            core::ptr::addr_of_mut!((*pPort).Entry),
        );

        res = RegSetValueExW(
            hroot,
            core::ptr::addr_of_mut!(PortName) as *mut WCHAR as LPCWSTR,
            0,
            REG_SZ,
            RUSTCONV_WSTR_96A296D224F2.as_ptr() as *const BYTE,
            core::mem::size_of_val(&[0x0000u16]) as DWORD,
        ) as DWORD;
        RegCloseKey(hroot);
    }

    // Cleanup:
    SetLastError(res);
    return (res == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonConfigurePort(hMonitor: HANDLE, pName: LPWSTR, hWnd: HWND, pPortName: LPWSTR) -> BOOL {
    // FIXME("LocalmonConfigurePort : %s\n", debugstr_w( pPortName ) );
    FALSE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn LocalmonDeletePort(hMonitor: HANDLE, pName: LPWSTR, hWnd: HWND, pPortName: LPWSTR) -> BOOL {
    let mut res: DWORD;
    let mut hroot: HKEY = core::ptr::null_mut();
    let pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pPort: PLOCALMON_PORT;

    res = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
        core::ptr::addr_of_mut!(hroot),
    ) as DWORD;
    if res == ERROR_SUCCESS {
        res = RegDeleteValueW(hroot, pPortName as LPCWSTR) as DWORD;

        RegCloseKey(hroot);

        pPort = _FindPort(pLocalmon, pPortName as PCWSTR);
        if !pPort.is_null() {
            EnterCriticalSection(core::ptr::addr_of_mut!((*(*pPort).pLocalmon).Section));
            RemoveEntryList(core::ptr::addr_of_mut!((*pPort).Entry));
            LeaveCriticalSection(core::ptr::addr_of_mut!((*(*pPort).pLocalmon).Section));

            DllFreeSplMem(pPort as PVOID);
        }
    }

    SetLastError(res);
    return (res == ERROR_SUCCESS) as BOOL;
}
