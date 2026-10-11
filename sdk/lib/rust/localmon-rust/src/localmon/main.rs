// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7AF58FDC50E8: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x006F, 0x0072, 0x0074, 0x0073,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut cbLocalMonitor: DWORD = 0;

#[no_mangle]
pub static mut cbLocalPort: DWORD = 0;

#[no_mangle]
pub static mut pwszLocalMonitor: PCWSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut pwszLocalPort: PCWSTR = ::core::ptr::null::<WCHAR>();

static mut _MonitorFunctions: MONITOR2 = unsafe {
    _MONITOR2 {
        cbSize: ::core::mem::size_of::<MONITOR2>() as DWORD,
        pfnEnumPorts: Some(
            LocalmonEnumPorts,
        ),
        pfnOpenPort: Some(LocalmonOpenPort),
        pfnOpenPortEx: None,
        pfnStartDocPort: Some(
            LocalmonStartDocPort,
        ),
        pfnWritePort: Some(
            LocalmonWritePort,
        ),
        pfnReadPort: Some(
            LocalmonReadPort,
        ),
        pfnEndDocPort: Some(LocalmonEndDocPort),
        pfnClosePort: Some(LocalmonClosePort),
        pfnAddPort: Some(
            LocalmonAddPort,
        ),
        pfnAddPortEx: Some(
            LocalmonAddPortEx,
        ),
        pfnConfigurePort: Some(
            LocalmonConfigurePort,
        ),
        pfnDeletePort: Some(
            LocalmonDeletePort,
        ),
        pfnGetPrinterDataFromPort: Some(
            LocalmonGetPrinterDataFromPort,
        ),
        pfnSetPortTimeOuts: Some(
            LocalmonSetPortTimeOuts,
        ),
        pfnXcvOpenPort: Some(
            LocalmonXcvOpenPort,
        ),
        pfnXcvDataPort: Some(
            LocalmonXcvDataPort,
        ),
        pfnXcvClosePort: Some(LocalmonXcvClosePort),
        pfnShutdown: Some(LocalmonShutdown),
        pfnSendRecvBidiDataFromPort: None,
    }
};

unsafe extern "C" fn _IsNEPort(pwszPortName: PCWSTR) -> BOOL {
    let mut p: PCWSTR = pwszPortName;

    // First character needs to be 'N' (uppercase or lowercase)
    if *p != 0x004Eu16 && *p != 0x006Eu16 {
        return FALSE as BOOL;
    }

    // Next character needs to be 'E' (uppercase or lowercase)
    p = p.offset(1);
    if *p != 0x0045u16 && *p != 0x0065u16 {
        return FALSE as BOOL;
    }

    // An optional hyphen may follow now.
    p = p.offset(1);
    if *p == 0x002Du16 {
        p = p.offset(1);
    }

    // Now an arbitrary number of digits may follow.
    while *p >= 0x0030u16 && *p <= 0x0039u16 {
        p = p.offset(1);
    }

    // Finally, the virtual Ne port must be terminated by a colon.
    if *p != ':' as u16 {
        return FALSE as BOOL;
    }

    // If this is the end of the string, we have a virtual Ne port.
    p = p.offset(1);
    return (*p == 0x0000u16) as BOOL;
}

unsafe extern "C" fn _LoadResources(hinstDLL: HINSTANCE) {
    LoadStringW(hinstDLL, IDS_LOCAL_MONITOR as UINT, core::ptr::addr_of_mut!(pwszLocalMonitor) as PWSTR, 0);
    cbLocalMonitor = ((wcslen(pwszLocalMonitor) as DWORD) + (1 as DWORD)) * (core::mem::size_of::<WCHAR>() as DWORD);

    LoadStringW(hinstDLL, IDS_LOCAL_PORT as UINT, core::ptr::addr_of_mut!(pwszLocalPort) as PWSTR, 0);
    cbLocalPort = ((wcslen(pwszLocalPort) as DWORD) + (1 as DWORD)) * (core::mem::size_of::<WCHAR>() as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            _LoadResources(hinstDLL);
        }
        _ => {}
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonShutdown(hMonitor: HANDLE) {
    let mut pLocalmon: PLOCALMON_HANDLE = core::ptr::null_mut();
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    let mut pXcv: PLOCALMON_XCV = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    pLocalmon = hMonitor as PLOCALMON_HANDLE;

    if (*pLocalmon).Sig != (SIGLCMMON as u32) {
        return;
    }

    // Close all virtual file ports.
    if !IsListEmpty(core::ptr::addr_of_mut!((*pLocalmon).FilePorts)) {
        pEntry = (*pLocalmon).FilePorts.Flink;
        while pEntry != core::ptr::addr_of_mut!((*pLocalmon).FilePorts) {
            pPort = CONTAINING_RECORD!(
                core::ptr::addr_of_mut!((*pLocalmon).FilePorts.Flink),
                LOCALMON_PORT,
                Entry
            );
            LocalmonClosePort(pPort as HANDLE);
            pEntry = (*pEntry).Flink;
        }
    }

    // Do the same for the open Xcv ports.
    if !IsListEmpty(core::ptr::addr_of_mut!((*pLocalmon).XcvHandles)) {
        pEntry = (*pLocalmon).XcvHandles.Flink;
        while pEntry != core::ptr::addr_of_mut!((*pLocalmon).XcvHandles) {
            pXcv = CONTAINING_RECORD!(pEntry, LOCALMON_XCV, Entry);
            LocalmonXcvClosePort(pXcv as HANDLE);
            pEntry = (*pEntry).Flink;
        }
    }

    // Now close all registry ports, remove them from the list and free their memory.
    if !IsListEmpty(core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts)) {
        pEntry = (*pLocalmon).RegistryPorts.Flink;
        while pEntry != core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts) {
            'body: {
                pPort = CONTAINING_RECORD!(pEntry, LOCALMON_PORT, Entry);
                if LocalmonClosePort(pPort as HANDLE) != 0 {
                    break 'body;
                }
                RemoveEntryList(core::ptr::addr_of_mut!((*pPort).Entry));
                DllFreeSplMem(pPort as PVOID);
            }
            pEntry = (*pEntry).Flink;
        }
    }

    // Finally clean the LOCALMON_HANDLE structure itself.
    DeleteCriticalSection(core::ptr::addr_of_mut!((*pLocalmon).Section));
    DllFreeSplMem(pLocalmon as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn InitializePrintMonitor2(pMonitorInit: PMONITORINIT, phMonitor: PHANDLE) -> PMONITOR2 {
    let mut cchMaxPortName: DWORD = 0;
    let mut cchPortName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwPortCount: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pReturnValue: PMONITOR2 = core::ptr::null_mut();
    let mut pLocalmon: PLOCALMON_HANDLE = core::ptr::null_mut();
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();

    // Create a new LOCALMON_HANDLE structure.
    pLocalmon = DllAllocSplMem((core::mem::size_of::<LOCALMON_HANDLE>()) as DWORD) as PLOCALMON_HANDLE;
    (*pLocalmon).Sig = (SIGLCMMON as u32);
    InitializeCriticalSection(core::ptr::addr_of_mut!((*pLocalmon).Section));
    InitializeListHead(core::ptr::addr_of_mut!((*pLocalmon).FilePorts));
    InitializeListHead(core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts));
    InitializeListHead(core::ptr::addr_of_mut!((*pLocalmon).XcvHandles));

    // The Local Spooler Port Monitor doesn't need to care about the given registry key and functions.
    // Instead it uses a well-known registry key for getting its information about local ports. Open this one.
    dwErrorCode = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;

    'cleanup: {
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Get the number of ports and the length of the largest port name.
        dwErrorCode = RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwPortCount),
            core::ptr::addr_of_mut!(cchMaxPortName),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Loop through all ports.
        i = 0;
        while i < dwPortCount {
            'body: {
                // Allocate memory for a new LOCALMON_PORT structure and its name.
                pPort = DllAllocSplMem(
                    core::mem::size_of::<LOCALMON_PORT>() as DWORD
                        + (cchMaxPortName + 1) * core::mem::size_of::<WCHAR>() as DWORD,
                ) as PLOCALMON_PORT;
                if pPort.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }

                (*pPort).Sig = (SIGLCMPORT as u32);
                (*pPort).pLocalmon = pLocalmon;
                (*pPort).hFile = INVALID_HANDLE_VALUE;
                (*pPort).pwszPortName = pPort.add(1) as PWSTR;

                // Get the port name.
                cchPortName = cchMaxPortName + 1;
                dwErrorCode = RegEnumValueW(
                    hKey,
                    i,
                    (*pPort).pwszPortName,
                    core::ptr::addr_of_mut!(cchPortName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                // pwszPortName can be one of the following to be valid for this Port Monitor:
                //    COMx:                        - Physical COM port
                //    LPTx:                        - Physical LPT port (or redirected one using "net use LPT1 ...")
                //    FILE:                        - Opens a prompt that asks for an output filename
                //    C:\bla.txt                   - Redirection into the file "C:\bla.txt"
                //    \\COMPUTERNAME\PrinterName   - Redirection to a shared network printer installed as a local port
                //
                // We can't detect valid and invalid ones by the name, so we can only exclude empty ports and the virtual "Ne00:", "Ne01:", ... ports.
                // Skip the invalid ones here.
                if cchPortName == 0 || _IsNEPort((*pPort).pwszPortName) != 0 {
                    DllFreeSplMem((pPort).cast());
                    pPort = core::ptr::null_mut();
                    break 'body;
                }

                // Add it to the list.
                InsertTailList(
                    core::ptr::addr_of_mut!((*pLocalmon).RegistryPorts),
                    core::ptr::addr_of_mut!((*pPort).Entry),
                );

                // Don't let the cleanup routine free this.
                pPort = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }

        // Return our handle and the Print Monitor functions.
        *phMonitor = pLocalmon as HANDLE;
        pReturnValue = core::ptr::addr_of_mut!(_MonitorFunctions) as PMONITOR2;
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pPort.is_null() {
        DllFreeSplMem((pPort).cast());
    }

    SetLastError(dwErrorCode);
    return pReturnValue;
}
