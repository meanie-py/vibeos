use crate::localspl::jobs::CreateJob;
use crate::localspl::ports::FindPort;
use crate::localspl::monitors::FindPrintMonitor;
use crate::localspl::jobs::FreeJob;
use crate::localspl::jobs::GetJobFilePath;
use crate::localspl::jobs::InitializePrinterJobList;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_446DFBEA34C5: [u16; 9] = [0x0044u16, 0x0061, 0x0074, 0x0061, 0x0074, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_4B731B4B251F: [u16; 6] = [0x0050u16, 0x006F, 0x0072, 0x0074, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_6AC123285056: [u16; 12] = [
    0x0044u16, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_6DE973DF96C9: [u16; 7] = [0x0053u16, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_9043A8690EC7: [u16; 9] = [0x004Cu16, 0x006F, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_929955077606: [u16; 16] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0020, 0x0044, 0x0065,
    0x0076, 0x004D, 0x006F, 0x0064, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_9665DD34268C: [u16; 5] = [0x0050u16, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_A2D227186F0E: [u16; 16] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0020, 0x0050, 0x0072, 0x006F, 0x0063,
    0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_A38E93E48CB3: [u16; 11] = [
    0x0041u16, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_AE3CF03C0158: [u16; 15] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0020, 0x0044, 0x0072,
    0x0069, 0x0076, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_C1615E893D5D: [u16; 4] = [0x0058u16, 0x0063, 0x0076, 0x0000];
pub static RUSTCONV_WSTR_D575D22E81A4: [u16; 4] = [0x0053u16, 0x0050, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_E19ED02EC71C: [u16; 5] = [0x004Au16, 0x006F, 0x0062, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_F5C28976B418: [u16; 9] = [0x004Du16, 0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0020, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cl: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... ) -> core::ffi::c_int;
}
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PrinterList: SKIPLIST = _SKIPLIST {
    Head: _SKIPLIST_NODE {
        Distance: [0; 16],
        Next: [::core::ptr::null::<_SKIPLIST_NODE>() as *mut _SKIPLIST_NODE; 16],
        Element: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    },
    MaximumLevel: 0,
    NodeCount: 0,
    AllocateRoutine: None,
    CompareRoutine: None,
    FreeRoutine: None,
};

static mut pfnGetPrinterLevels: [PLocalGetPrinterLevelFunc; 10] = unsafe {
    [
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_STRESS,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel0
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_STRESS,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_1W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel1
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_1W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_2W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel2
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_2W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_3,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel3
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_3,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_4W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel4
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_4W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_5W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel5
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_5W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_6,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel6
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_6,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_7W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel7
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_7W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_8W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel8
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_8W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_9W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
            >,
            PLocalGetPrinterLevelFunc,
        >(Some(
            _LocalGetPrinterLevel9
                as unsafe extern "C" fn(
                    PLOCAL_PRINTER,
                    *mut PPRINTER_INFO_9W,
                    *mut PBYTE,
                    PDWORD,
                    DWORD,
                    PCWSTR,
                ) -> (),
        )),
    ]
};

static mut dwPrinterInfo0Offsets: [DWORD; 2] = [
    core::mem::offset_of!(PRINTER_INFO_STRESS, pPrinterName) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwPrinterInfo1Offsets: [DWORD; 4] = [
    core::mem::offset_of!(PRINTER_INFO_1W, pName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_1W, pComment) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_1W, pDescription) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwPrinterInfo2Offsets: [DWORD; 11] = [
    core::mem::offset_of!(PRINTER_INFO_2W, pPrinterName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pShareName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pPortName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pDriverName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pComment) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pLocation) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pSepFile) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pPrintProcessor) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pDatatype) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_2W, pParameters) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwPrinterInfo4Offsets: [DWORD; 2] = [
    core::mem::offset_of!(PRINTER_INFO_4W, pPrinterName) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwPrinterInfo5Offsets: [DWORD; 3] = [
    core::mem::offset_of!(PRINTER_INFO_5W, pPrinterName) as DWORD,
    core::mem::offset_of!(PRINTER_INFO_5W, pPortName) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwDeviceNotSelectedTimeout: DWORD = 15000 as DWORD;

static mut dwTransmissionRetryTimeout: DWORD = 45000 as DWORD;

pub type PLocalGetPrinterLevelFunc = unsafe extern "C" fn(PLOCAL_PRINTER, PVOID, *mut PBYTE, PDWORD, DWORD, PCWSTR);
unsafe extern "system" fn _PrinterListCompareRoutine(FirstStruct: PVOID, SecondStruct: PVOID) -> i32 {
    let A: PLOCAL_PRINTER = FirstStruct as PLOCAL_PRINTER;
    let B: PLOCAL_PRINTER = SecondStruct as PLOCAL_PRINTER;

    return _wcsicmp((*A).pwszPrinterName as *const u16, (*B).pwszPrinterName as *const u16);
}
#[no_mangle]
pub unsafe extern "C" fn InitializePrinterList() -> BOOL {
    let mut cbData: DWORD = 0;
    let mut cchPrinterName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwSubKeys: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut();
    let mut pPrinter: PLOCAL_PRINTER = core::ptr::null_mut();
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();
    let mut pwszPort: PWSTR = core::ptr::null_mut();
    let mut pwszPrintProcessor: PWSTR = core::ptr::null_mut();
    let mut wszPrinterName: [WCHAR; MAX_PRINTER_NAME as usize + 1] =
        [0; MAX_PRINTER_NAME as usize + 1];

    // Initialize an empty list for our printers.
    InitializeSkiplist(
        core::ptr::addr_of_mut!(PrinterList),
        Some(DllAllocSplMem),
        Some(_PrinterListCompareRoutine),
        core::mem::transmute::<Option<unsafe extern "system" fn(PVOID) -> BOOL>, PSKIPLIST_FREE_ROUTINE>(
            Some(DllFreeSplMem),
        ),
    );

    // Get the number of subkeys of the printers registry key. Each subkey is a local printer there.
    dwErrorCode = RegQueryInfoKeyW(
        hPrintersKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwSubKeys),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as DWORD;

    'cleanup: {
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Loop through all available local printers.
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

                if !pPrinter.is_null() {
                    if !(*pPrinter).pDefaultDevMode.is_null() {
                        DllFreeSplMem((*pPrinter).pDefaultDevMode as PVOID);
                    }

                    if !(*pPrinter).pwszDefaultDatatype.is_null() {
                        DllFreeSplStr((*pPrinter).pwszDefaultDatatype);
                    }

                    if !(*pPrinter).pwszDescription.is_null() {
                        DllFreeSplStr((*pPrinter).pwszDescription);
                    }

                    if !(*pPrinter).pwszPrinterDriver.is_null() {
                        DllFreeSplStr((*pPrinter).pwszPrinterDriver);
                    }

                    if !(*pPrinter).pwszPrinterName.is_null() {
                        DllFreeSplStr((*pPrinter).pwszPrinterName);
                    }

                    DllFreeSplMem(pPrinter as PVOID);
                    pPrinter = core::ptr::null_mut();
                }

                if !pwszPrintProcessor.is_null() {
                    DllFreeSplStr(pwszPrintProcessor);
                    pwszPrintProcessor = core::ptr::null_mut();
                }

                // Get the name of this printer.
                cchPrinterName = _countof!(wszPrinterName) as DWORD;
                dwErrorCode = RegEnumKeyExW(
                    hPrintersKey,
                    i,
                    core::ptr::addr_of_mut!(wszPrinterName) as *mut WCHAR as LPWSTR,
                    core::ptr::addr_of_mut!(cchPrinterName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                if dwErrorCode == ERROR_MORE_DATA {
                    // This printer name exceeds the maximum length and is invalid.
                    break 'body;
                } else if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Open this Printer's registry key.
                dwErrorCode = RegOpenKeyExW(
                    hPrintersKey,
                    core::ptr::addr_of_mut!(wszPrinterName) as *mut WCHAR as LPCWSTR,
                    0,
                    KEY_READ,
                    core::ptr::addr_of_mut!(hSubKey),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Get the Print Processor.
                pwszPrintProcessor = AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_A2D227186F0E.as_ptr());
                if pwszPrintProcessor.is_null() {
                    break 'body;
                }

                // Try to find it in the Print Processor List.
                pPrintProcessor = FindPrintProcessor(pwszPrintProcessor as PCWSTR);
                if pPrintProcessor.is_null() {
                    break 'body;
                }

                // Get the Port.
                pwszPort = AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_9665DD34268C.as_ptr());
                if pwszPort.is_null() {
                    break 'body;
                }

                // Try to find it in the Port List.
                pPort = FindPort(pwszPort as PCWSTR);
                if pPort.is_null() {
                    break 'body;
                }

                // Create a new LOCAL_PRINTER structure for it.
                pPrinter = DllAllocSplMem(core::mem::size_of::<LOCAL_PRINTER>() as DWORD) as PLOCAL_PRINTER;
                if pPrinter.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }

                (*pPrinter).pwszPrinterName = AllocSplStr(
                    core::ptr::addr_of_mut!(wszPrinterName) as *mut WCHAR as PCWSTR,
                );
                (*pPrinter).pPrintProcessor = pPrintProcessor;
                (*pPrinter).pPort = pPort;
                InitializePrinterJobList(pPrinter);

                // Get the location.
                (*pPrinter).pwszLocation = AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_9043A8690EC7.as_ptr());
                if (*pPrinter).pwszLocation.is_null() {
                    break 'body;
                }

                // Get the printer driver.
                (*pPrinter).pwszPrinterDriver =
                    AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_AE3CF03C0158.as_ptr());
                if (*pPrinter).pwszPrinterDriver.is_null() {
                    break 'body;
                }

                // Get the description.
                (*pPrinter).pwszDescription =
                    AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_6AC123285056.as_ptr());
                if (*pPrinter).pwszDescription.is_null() {
                    break 'body;
                }

                // Get the default datatype.
                (*pPrinter).pwszDefaultDatatype =
                    AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_446DFBEA34C5.as_ptr());
                if (*pPrinter).pwszDefaultDatatype.is_null() {
                    break 'body;
                }

                // Verify that it's valid.
                if FindDatatype(
                    pPrintProcessor,
                    (*pPrinter).pwszDefaultDatatype as PCWSTR,
                ) == 0
                {
                    break 'body;
                }

                // Determine the size of the DevMode.
                dwErrorCode = RegQueryValueExW(
                    hSubKey,
                    RUSTCONV_WSTR_929955077606.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(cbData),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Allocate enough memory for the DevMode.
                (*pPrinter).pDefaultDevMode = DllAllocSplMem(cbData) as PDEVMODEW;
                if (*pPrinter).pDefaultDevMode.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }

                // Get the default DevMode.
                dwErrorCode = RegQueryValueExW(
                    hSubKey,
                    RUSTCONV_WSTR_929955077606.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*pPrinter).pDefaultDevMode as PBYTE,
                    core::ptr::addr_of_mut!(cbData),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Get the Attributes.
                cbData = core::mem::size_of::<DWORD>() as DWORD;
                dwErrorCode = RegQueryValueExW(
                    hSubKey,
                    RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*pPrinter).dwAttributes) as PBYTE,
                    core::ptr::addr_of_mut!(cbData),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Get the Status.
                cbData = core::mem::size_of::<DWORD>() as DWORD;
                dwErrorCode = RegQueryValueExW(
                    hSubKey,
                    RUSTCONV_WSTR_6DE973DF96C9.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*pPrinter).dwStatus) as PBYTE,
                    core::ptr::addr_of_mut!(cbData),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS {
                    break 'body;
                }

                // Add this printer to the printer list.
                if InsertElementSkiplist(core::ptr::addr_of_mut!(PrinterList), pPrinter as PVOID)
                    == 0
                {
                    break 'cleanup;
                }

                // Don't let the cleanup routines free this.
                pPrinter = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    // Inside the loop
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    if !pPrinter.is_null() {
        if !(*pPrinter).pDefaultDevMode.is_null() {
            DllFreeSplMem((*pPrinter).pDefaultDevMode as PVOID);
        }

        if !(*pPrinter).pwszDefaultDatatype.is_null() {
            DllFreeSplStr((*pPrinter).pwszDefaultDatatype);
        }

        if !(*pPrinter).pwszDescription.is_null() {
            DllFreeSplStr((*pPrinter).pwszDescription);
        }

        if !(*pPrinter).pwszPrinterDriver.is_null() {
            DllFreeSplStr((*pPrinter).pwszPrinterDriver);
        }

        if !(*pPrinter).pwszPrinterName.is_null() {
            DllFreeSplStr((*pPrinter).pwszPrinterName);
        }

        DllFreeSplMem(pPrinter as PVOID);
    }

    if !pwszPrintProcessor.is_null() {
        DllFreeSplStr(pwszPrintProcessor);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn BroadcastChange(mut pHandle: PLOCAL_HANDLE) {
    let mut pPrinter: PLOCAL_PRINTER = core::ptr::null_mut();
    let mut pNode: PSKIPLIST_NODE = core::ptr::null_mut();
    let mut cchMachineName: DWORD = 0 as DWORD;
    let mut wszMachineName: [WCHAR; MAX_PATH as usize] = [0 as WCHAR; MAX_PATH as usize];
    pNode = PrinterList.Head.Next[0 as usize] as PSKIPLIST_NODE;
    while !pNode.is_null() {
        pPrinter = (*pNode).Element as PLOCAL_PRINTER;
        StringCchCopyW(
            (core::ptr::addr_of_mut!(wszMachineName) as *mut WCHAR).add(cchMachineName as usize) as STRSAFE_LPWSTR,
            core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as size_t,
            (*pPrinter).pwszPrinterName as STRSAFE_LPCWSTR,
        );
        PostMessageW(
            HWND_BROADCAST as HWND,
            WM_DEVMODECHANGE as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(wszMachineName) as LPARAM,
        );
        pNode = (*pNode).Next[0 as usize] as PSKIPLIST_NODE;
    }
}

unsafe extern "C" fn _LocalEnumPrintersCheckName(
    mut Flags: DWORD,
    mut Name: PCWSTR,
    mut pwszComputerName: PWSTR,
    mut pcchComputerName: PDWORD,
) -> DWORD {
    let mut pName: PCWSTR = core::ptr::null();
    let mut pComputerName: PCWSTR = core::ptr::null();

    // If there is no Name parameter to check, we can just continue in EnumPrinters.
    if Name.is_null() {
        return ERROR_SUCCESS;
    }

    // Check if Name does not begin with two backslashes (required for specifying Computer Names).
    if *Name.offset(0 as isize) != 0x005Cu16
        || *Name.offset(1 as isize) != 0x005Cu16
    {
        if (Flags & PRINTER_ENUM_NAME) != 0 {
            // Compare with the Print Provider Name.
            if _wcsicmp(Name, wszPrintProviderInfo[0 as usize])
                == 0
            {
                return ERROR_SUCCESS;
            }

            // Dismiss anything else.
            return ERROR_INVALID_NAME;
        } else {
            // If PRINTER_ENUM_NAME is not specified, we just ignore anything that is not a Computer Name.
            return ERROR_SUCCESS;
        }
    }

    // Prepend the backslashes to the output computer name.
    *pwszComputerName.offset(0 as isize) = 0x005Cu16;
    *pwszComputerName.offset(1 as isize) = 0x005Cu16;

    // Get the local computer name for comparison.
    *pcchComputerName = MAX_COMPUTERNAME_LENGTH + 1;
    if GetComputerNameW(
        pwszComputerName.offset(2 as isize) as LPWSTR,
        pcchComputerName as LPDWORD,
    ) == 0
    {
        return GetLastError();
    }

    // Add the leading slashes to the total length.
    *pcchComputerName += 2;

    // Compare both names.
    pComputerName = pwszComputerName.offset(2 as isize) as PCWSTR;
    pName = Name.offset(2 as isize) as PCWSTR;
    loop {
        // Are we at the end of the local Computer Name string?
        if *pComputerName == 0 {
            // Are we also at the end of the supplied Name parameter?
            if *pName == 0 {
                // Add a trailing backslash for that.
                let fresh31 = *pcchComputerName;
                *pcchComputerName = (*pcchComputerName).wrapping_add(1);
                *pwszComputerName.offset(fresh31 as isize) = 0x005Cu16;
                *pwszComputerName.offset(*pcchComputerName as isize) = 0;
                return ERROR_SUCCESS;
            } else if *pName == 0x005Cu16 {
                if (Flags & PRINTER_ENUM_NAME) != 0 {
                    // Anything else (like "\\COMPUTERNAME\") is dismissed.
                    return ERROR_INVALID_NAME;
                } else {
                    // Only the Computer Name between the backslashes is checked then.
                    *pwszComputerName = 0;
                    *pcchComputerName = 0;
                    return ERROR_SUCCESS;
                }
            }
        }

        // Compare both Computer Names case-insensitively and reject with ERROR_INVALID_NAME if they don't match.
        if towlower(*pName) != towlower(*pComputerName) {
            return ERROR_INVALID_NAME;
        }

        pName = pName.offset(1);
        pComputerName = pComputerName.offset(1);
    }
}

unsafe extern "C" fn _DumpLevel1PrintProviderInformation(pPrinterEnum: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> DWORD {
    let mut i: ::core::ffi::c_int;

    // Count the needed bytes for Print Provider information.
    *pcbNeeded = core::mem::size_of::<PRINTER_INFO_1W>() as DWORD;

    i = 0 as ::core::ffi::c_int;
    while i < 3 as ::core::ffi::c_int {
        *pcbNeeded = (*pcbNeeded).wrapping_add(
            ((wcslen(wszPrintProviderInfo[i as usize]) as usize)
                .wrapping_add(1 as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>())) as DWORD,
        );
        i += 1;
    }

    // Check if the supplied buffer is large enough.
    if cbBuf < *pcbNeeded {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }

    // Copy over the Print Provider information.
    (*(pPrinterEnum as PPRINTER_INFO_1W)).Flags = 0 as DWORD;
    PackStrings(
        core::ptr::addr_of_mut!(wszPrintProviderInfo) as *mut PCWSTR,
        pPrinterEnum,
        core::ptr::addr_of_mut!(dwPrinterInfo1Offsets) as *mut DWORD,
        pPrinterEnum.add(*pcbNeeded as usize),
    );
    *pcReturned = 1 as DWORD;

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _LocalGetPrinterLevel0(
    pPrinter: PLOCAL_PRINTER,
    ppPrinterInfo: *mut PPRINTER_INFO_STRESS,
    ppPrinterInfoEnd: *mut PBYTE,
    pcbNeeded: PDWORD,
    cchComputerName: DWORD,
    wszComputerName: PCWSTR,
) {
    let mut cbName: size_t = 0;
    let mut p: PWSTR = core::ptr::null_mut();
    let mut Allocation: PWSTR = core::ptr::null_mut();
    let mut pwszStrings: [PCWSTR; 1] = [core::ptr::null(); 1];
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();

    // Calculate the string lengths.
    cbName = ((cchComputerName as size_t)
        .wrapping_add(wcslen((*pPrinter).pwszPrinterName as *const WCHAR) as size_t)
        .wrapping_add(1 as size_t))
        .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t);

    if ppPrinterInfo.is_null() {
        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<PRINTER_INFO_STRESS>() as size_t).wrapping_add(cbName) as DWORD,
        );
        return;
    }

    // Set the general fields.
    ZeroMemory!(
        *ppPrinterInfo as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTER_INFO_STRESS>() as size_t,
    );
    (**ppPrinterInfo).cJobs = (*pPrinter).JobList.NodeCount;
    (**ppPrinterInfo).dwGetVersion = GetVersion();
    (**ppPrinterInfo).Status = (*pPrinter).dwStatus;

    (**ppPrinterInfo).fFreeBuild = 1;

    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));
    (**ppPrinterInfo).dwNumberOfProcessors = SystemInfo.dwNumberOfProcessors;
    (**ppPrinterInfo).dwProcessorType = SystemInfo.dwProcessorType;
    (**ppPrinterInfo).wProcessorArchitecture = SystemInfo
        .__bindgen_anon_1
        .__bindgen_anon_1
        .wProcessorArchitecture as USHORT;
    (**ppPrinterInfo).wProcessorLevel = SystemInfo.wProcessorLevel as USHORT;

    // Copy the Printer Name.
    Allocation = DllAllocSplMem(cbName as DWORD) as PWSTR;
    p = Allocation;
    pwszStrings[0 as usize] = Allocation as PCWSTR;
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbName,
        wszComputerName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbName),
        0 as STRSAFE_DWORD,
    );
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbName,
        (*pPrinter).pwszPrinterName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbName),
        0 as STRSAFE_DWORD,
    );

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppPrinterInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppPrinterInfo as PBYTE,
        core::ptr::addr_of!(dwPrinterInfo0Offsets) as *mut DWORD,
        *ppPrinterInfoEnd,
    );
    *ppPrinterInfo = (*ppPrinterInfo).add(1);

    // Free the memory for temporary strings.
    DllFreeSplMem(Allocation as PVOID);
}

unsafe extern "C" fn _LocalGetPrinterLevel1(pPrinter: PLOCAL_PRINTER, ppPrinterInfo: *mut PPRINTER_INFO_1W, ppPrinterInfoEnd: *mut PBYTE, pcbNeeded: PDWORD, cchComputerName: DWORD, wszComputerName: PCWSTR) {
    let wszComma: [WCHAR; 2] = [0x002Cu16, 0x0000];

    let mut cbName: size_t;
    let mut cbComment: size_t;
    let mut cbDescription: size_t;
    let mut p: PWSTR;
    let mut Allocation1: PWSTR;
    let mut Allocation2: PWSTR;
    let mut pwszStrings: [PCWSTR; 3] = [core::ptr::null(); 3];

    // Calculate the string lengths.
    cbName = (cchComputerName as size_t + wcslen((*pPrinter).pwszPrinterName as *const WCHAR) as size_t + 1) * core::mem::size_of::<WCHAR>() as size_t;
    cbComment = (wcslen((*pPrinter).pwszDescription as *const WCHAR) as size_t + 1) * core::mem::size_of::<WCHAR>() as size_t;
    cbDescription = cbName + (wcslen((*pPrinter).pwszPrinterDriver as *const WCHAR) as size_t + 1 + wcslen((*pPrinter).pwszLocation as *const WCHAR) as size_t + 1) * core::mem::size_of::<WCHAR>() as size_t;

    if ppPrinterInfo.is_null() {
        *pcbNeeded += (core::mem::size_of::<PRINTER_INFO_1W>() as size_t + cbName + cbComment + cbDescription) as DWORD;
        return;
    }

    // Indicate that this is a Printer.
    (**ppPrinterInfo).Flags = PRINTER_ENUM_ICON8 as DWORD;

    // Copy the Printer Name.
    Allocation1 = DllAllocSplMem(cbName as DWORD) as PWSTR;
    p = Allocation1;
    pwszStrings[0 as usize] = Allocation1 as PCWSTR;
    StringCbCopyExW(p as PWSTR, cbName, wszComputerName, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbName), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbName, (*pPrinter).pwszPrinterName, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbName), 0 as DWORD);

    // Copy the Printer comment (equals the "Description" registry value).
    pwszStrings[1 as usize] = (*pPrinter).pwszDescription as PCWSTR;

    // Copy the description, which for PRINTER_INFO_1W has the form "Name,Printer Driver,Location"
    Allocation2 = DllAllocSplMem(cbDescription as DWORD) as PWSTR;
    p = Allocation2;
    pwszStrings[2 as usize] = Allocation2 as PCWSTR;
    StringCbCopyExW(p as PWSTR, cbDescription, wszComputerName, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbDescription, (*pPrinter).pwszPrinterName, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbDescription, core::ptr::addr_of!(wszComma) as PCWSTR, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbDescription, (*pPrinter).pwszPrinterDriver, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbDescription, core::ptr::addr_of!(wszComma) as PCWSTR, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);
    StringCbCopyExW(p as PWSTR, cbDescription, (*pPrinter).pwszLocation, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(cbDescription), 0 as DWORD);

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppPrinterInfoEnd = PackStrings(core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR, *ppPrinterInfo as PBYTE, core::ptr::addr_of_mut!(dwPrinterInfo1Offsets) as *mut DWORD, *ppPrinterInfoEnd);
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);

    // Free the memory for temporary strings.
    DllFreeSplMem(Allocation1 as PVOID);
    DllFreeSplMem(Allocation2 as PVOID);
}

unsafe extern "C" fn _LocalGetPrinterLevel2(
    pPrinter: PLOCAL_PRINTER,
    ppPrinterInfo: *mut PPRINTER_INFO_2W,
    ppPrinterInfoEnd: *mut PBYTE,
    pcbNeeded: PDWORD,
    cchComputerName: DWORD,
    wszComputerName: PCWSTR,
) {
    let mut wszEmpty: [WCHAR; 1] = [0x0000u16];

    let mut cbDevMode: usize;
    let mut cbPrinterName: usize;
    let mut cbShareName: usize;
    let mut cbPortName: usize;
    let mut cbDriverName: usize;
    let mut cbComment: usize;
    let mut cbLocation: usize;
    let mut cbSepFile: usize;
    let mut cbPrintProcessor: usize;
    let mut cbDatatype: usize;
    let mut cbParameters: usize;
    let mut p: PWSTR;
    let mut Allocation: PWSTR;
    let mut pwszStrings: [PCWSTR; 10] =
        [core::ptr::null(); 10];

    // Calculate the string lengths.
    cbDevMode = (*(*pPrinter).pDefaultDevMode).dmSize as usize
        + (*(*pPrinter).pDefaultDevMode).dmDriverExtra as usize;
    cbPrinterName = ((cchComputerName as usize)
        + wcslen((*pPrinter).pwszPrinterName as *const WCHAR) as usize
        + 1)
        * core::mem::size_of::<WCHAR>();

    if ppPrinterInfo.is_null() {
        // Attention: pComment equals the "Description" registry value.
        cbShareName = core::mem::size_of::<[WCHAR; 1]>();
        cbPortName = (wcslen((*(*pPrinter).pPort).pwszName as *const WCHAR) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbDriverName = (wcslen((*pPrinter).pwszPrinterDriver as *const WCHAR) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbComment = (wcslen((*pPrinter).pwszDescription as *const WCHAR) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbLocation = (wcslen((*pPrinter).pwszLocation as *const WCHAR) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbSepFile = core::mem::size_of::<[WCHAR; 1]>();
        cbPrintProcessor = (wcslen(
            (*(*pPrinter).pPrintProcessor).pwszName as *const WCHAR,
        ) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbDatatype = (wcslen((*pPrinter).pwszDefaultDatatype as *const WCHAR) as usize
            + 1)
            * core::mem::size_of::<WCHAR>();
        cbParameters = core::mem::size_of::<[WCHAR; 1]>();

        *pcbNeeded = (*pcbNeeded as usize
            + (core::mem::size_of::<PRINTER_INFO_2W>()
                + cbDevMode
                + cbPrinterName
                + cbShareName
                + cbPortName
                + cbDriverName
                + cbComment
                + cbLocation
                + cbSepFile
                + cbPrintProcessor
                + cbDatatype
                + cbParameters)) as DWORD;

        return;
    }

    // Set the general fields.
    ZeroMemory!(
        *ppPrinterInfo as *mut core::ffi::c_void,
        (core::mem::size_of::<PRINTER_INFO_2W>()) as usize,
    );
    (**ppPrinterInfo).Attributes = (*pPrinter).dwAttributes;
    (**ppPrinterInfo).cJobs = (*pPrinter).JobList.NodeCount;
    (**ppPrinterInfo).Status = (*pPrinter).dwStatus;

    // Set the pDevMode field (and copy the DevMode).
    *ppPrinterInfoEnd = (*ppPrinterInfoEnd).sub(cbDevMode);
    CopyMemory!(
        *ppPrinterInfoEnd as *mut core::ffi::c_void,
        (*pPrinter).pDefaultDevMode as *const core::ffi::c_void,
        cbDevMode,
    );
    (**ppPrinterInfo).pDevMode = *ppPrinterInfoEnd as PDEVMODEW;

    // Set the pPrinterName field.
    Allocation = DllAllocSplMem(cbPrinterName as DWORD) as PWSTR;
    p = Allocation;
    pwszStrings[0 as usize] = Allocation as PCWSTR;
    StringCbCopyExW(
        p,
        cbPrinterName,
        wszComputerName,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbPrinterName),
        0,
    );
    StringCbCopyExW(
        p,
        cbPrinterName,
        (*pPrinter).pwszPrinterName,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbPrinterName),
        0,
    );

    // Set the pShareName field.
    pwszStrings[1 as usize] =
        core::ptr::addr_of_mut!(wszEmpty) as *mut WCHAR as PCWSTR;

    // Set the pPortName field.
    pwszStrings[2 as usize] =
        (*(*pPrinter).pPort).pwszName as PCWSTR;

    // Set the pDriverName field.
    pwszStrings[3 as usize] =
        (*pPrinter).pwszPrinterDriver as PCWSTR;

    // Set the pComment field ((equals the "Description" registry value).
    pwszStrings[4 as usize] =
        (*pPrinter).pwszDescription as PCWSTR;

    // Set the pLocation field.
    pwszStrings[5 as usize] =
        (*pPrinter).pwszLocation as PCWSTR;

    // Set the pSepFile field.
    pwszStrings[6 as usize] =
        core::ptr::addr_of_mut!(wszEmpty) as *mut WCHAR as PCWSTR;

    // Set the pPrintProcessor field.
    pwszStrings[7 as usize] =
        (*(*pPrinter).pPrintProcessor).pwszName as PCWSTR;

    // Set the pDatatype field.
    pwszStrings[8 as usize] =
        (*pPrinter).pwszDefaultDatatype as PCWSTR;

    // Set the pParameters field.
    pwszStrings[9 as usize] =
        core::ptr::addr_of_mut!(wszEmpty) as *mut WCHAR as PCWSTR;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppPrinterInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppPrinterInfo as PBYTE,
        core::ptr::addr_of!(dwPrinterInfo2Offsets) as *mut DWORD,
        *ppPrinterInfoEnd,
    );
    *ppPrinterInfo = (*ppPrinterInfo).add(1);

    // Free the memory for temporary strings.
    DllFreeSplMem(Allocation as *mut core::ffi::c_void);
}

unsafe extern "C" fn _LocalGetPrinterLevel3(pPrinter: PLOCAL_PRINTER, ppPrinterInfo: *mut *mut PRINTER_INFO_3, ppPrinterInfoEnd: *mut PBYTE, pcbNeeded: PDWORD, cchComputerName: DWORD, wszComputerName: PCWSTR) {
    let mut SecurityDescriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();

    if ppPrinterInfo.is_null() {
        *pcbNeeded += (core::mem::size_of::<PRINTER_INFO_3>() + core::mem::size_of::<_SECURITY_DESCRIPTOR>()) as DWORD;
        return;
    }

    // Set the pSecurityDescriptor field (and copy the Security Descriptor).
    *ppPrinterInfoEnd = (*ppPrinterInfoEnd).offset(-(core::mem::size_of::<_SECURITY_DESCRIPTOR>() as isize));
    CopyMemory!(*ppPrinterInfoEnd as *mut core::ffi::c_void, core::ptr::addr_of_mut!(SecurityDescriptor) as *const core::ffi::c_void, core::mem::size_of::<_SECURITY_DESCRIPTOR>());
    (**ppPrinterInfo).pSecurityDescriptor = *ppPrinterInfoEnd as PSECURITY_DESCRIPTOR;

    // Advance to the next structure.
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);
}

unsafe extern "C" fn _LocalGetPrinterLevel4(
    mut pPrinter: PLOCAL_PRINTER,
    mut ppPrinterInfo: *mut PPRINTER_INFO_4W,
    mut ppPrinterInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
    mut cchComputerName: DWORD,
    mut wszComputerName: PCWSTR,
) {
    let mut cbPrinterName: size_t = 0;
    let mut p: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut Allocation: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut pwszStrings: [PCWSTR; 1] = [::core::ptr::null::<WCHAR>(); 1];
    cbPrinterName = (cchComputerName as usize)
        .wrapping_add(wcslen((*pPrinter).pwszPrinterName as *const wchar_t) as usize)
        .wrapping_add(1 as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as size_t;
    if ppPrinterInfo.is_null() {
        *pcbNeeded = (*pcbNeeded as ::core::ffi::c_ulonglong).wrapping_add(
            (::core::mem::size_of::<PRINTER_INFO_4W>() as usize)
                .wrapping_add(cbPrinterName as usize) as ::core::ffi::c_ulonglong,
        ) as DWORD as DWORD;
        return;
    }
    (**ppPrinterInfo).pServerName = ::core::ptr::null_mut::<WCHAR>();
    (**ppPrinterInfo).Attributes = (*pPrinter).dwAttributes;
    Allocation = DllAllocSplMem(cbPrinterName as DWORD) as PWSTR;
    p = Allocation;
    pwszStrings[0 as usize] = Allocation as PCWSTR;
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbPrinterName,
        wszComputerName as STRSAFE_LPCWSTR,
        &raw mut p,
        &raw mut cbPrinterName,
        0 as STRSAFE_DWORD,
    );
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbPrinterName,
        (*pPrinter).pwszPrinterName as STRSAFE_LPCWSTR,
        &raw mut p,
        &raw mut cbPrinterName,
        0 as STRSAFE_DWORD,
    );
    *ppPrinterInfoEnd = PackStrings(
        &raw mut pwszStrings as *mut PCWSTR,
        *ppPrinterInfo as PBYTE,
        &raw mut dwPrinterInfo4Offsets as *mut DWORD,
        *ppPrinterInfoEnd,
    );
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);
    DllFreeSplMem(Allocation as PVOID);
}

unsafe extern "C" fn _LocalGetPrinterLevel5(
    mut pPrinter: PLOCAL_PRINTER,
    mut ppPrinterInfo: *mut PPRINTER_INFO_5W,
    mut ppPrinterInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
    mut cchComputerName: DWORD,
    mut wszComputerName: PCWSTR,
) {
    let mut cbPrinterName: size_t = 0;
    let mut cbPortName: size_t = 0;
    let mut p: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut Allocation: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pwszStrings: [PCWSTR; 2] =
        [core::ptr::null::<WCHAR>(); 2];

    // Calculate the string lengths.
    cbPrinterName = ((cchComputerName as usize)
        + wcslen((*pPrinter).pwszPrinterName as PCWSTR) as usize
        + 1 as usize)
        * core::mem::size_of::<WCHAR>() as usize as size_t;

    if ppPrinterInfo.is_null() {
        cbPortName = (wcslen((*(*pPrinter).pPort).pwszName as PCWSTR) as usize
            + 1 as usize)
            * core::mem::size_of::<WCHAR>() as usize as size_t;

        *pcbNeeded = ((*pcbNeeded as usize)
            + core::mem::size_of::<PRINTER_INFO_5W>() as usize
            + cbPrinterName as usize
            + cbPortName as usize) as DWORD;
        return;
    }

    // Set the general fields.
    (**ppPrinterInfo).Attributes = (*pPrinter).dwAttributes;
    (**ppPrinterInfo).DeviceNotSelectedTimeout = dwDeviceNotSelectedTimeout;
    (**ppPrinterInfo).TransmissionRetryTimeout = dwTransmissionRetryTimeout;

    // Set the pPrinterName field.
    Allocation = DllAllocSplMem(cbPrinterName as DWORD) as PWSTR;
    p = Allocation;
    pwszStrings[0 as usize] = Allocation as PCWSTR;
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbPrinterName,
        wszComputerName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbPrinterName),
        0 as STRSAFE_DWORD,
    );
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbPrinterName,
        (*pPrinter).pwszPrinterName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbPrinterName),
        0 as STRSAFE_DWORD,
    );

    // Set the pPortName field.
    pwszStrings[1 as usize] = (*(*pPrinter).pPort).pwszName as PCWSTR;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppPrinterInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppPrinterInfo as PBYTE,
        core::ptr::addr_of_mut!(dwPrinterInfo5Offsets) as *mut DWORD,
        *ppPrinterInfoEnd,
    );
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);

    // Free the memory for temporary strings.
    DllFreeSplMem(Allocation as PVOID);
}

unsafe extern "C" fn _LocalGetPrinterLevel6(
    mut pPrinter: PLOCAL_PRINTER,
    mut ppPrinterInfo: *mut PPRINTER_INFO_6,
    mut ppPrinterInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
    mut cchComputerName: DWORD,
    mut wszComputerName: PCWSTR,
) {
    if ppPrinterInfo.is_null() {
        *pcbNeeded = (*pcbNeeded).wrapping_add(
            core::mem::size_of::<PRINTER_INFO_6>() as DWORD
        );
        return;
    }

    // Set the general fields.
    (**ppPrinterInfo).dwStatus = (*pPrinter).dwStatus;

    // Advance to the next structure.
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);
}

unsafe extern "C" fn _LocalGetPrinterLevel7(
    mut pPrinter: PLOCAL_PRINTER,
    mut ppPrinterInfo: *mut PPRINTER_INFO_7W,
    mut ppPrinterInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
    mut cchComputerName: DWORD,
    mut wszComputerName: PCWSTR,
) {
    if ppPrinterInfo.is_null() {
        *pcbNeeded = (*pcbNeeded as u64)
            .wrapping_add(core::mem::size_of::<PRINTER_INFO_7W>() as u64) as DWORD;
        return;
    }
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/printers.c\0"
                as *const u8 as *const i8,
            b"_LocalGetPrinterLevel7\0" as *const u8 as *const i8,
            825i32,
            b"No Directory Support, returning DSPRINT_UNPUBLISH for PRINTER_INFO_7 all the time!\n\0"
                as *const u8 as *const i8,
        );
    }
    (**ppPrinterInfo).dwAction = DSPRINT_UNPUBLISH as DWORD;
    (**ppPrinterInfo).pszObjectGUID = core::ptr::null_mut();
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);
}

unsafe extern "C" fn _LocalGetPrinterLevel8(
    pPrinter: PLOCAL_PRINTER,
    ppPrinterInfo: *mut PPRINTER_INFO_8W,
    ppPrinterInfoEnd: *mut PBYTE,
    pcbNeeded: PDWORD,
    cchComputerName: DWORD,
    wszComputerName: PCWSTR,
) {
    let mut cbDevMode: DWORD;

    // Calculate the string lengths.
    cbDevMode = ((*(*pPrinter).pDefaultDevMode).dmSize as DWORD) + ((*(*pPrinter).pDefaultDevMode).dmDriverExtra as DWORD);

    if ppPrinterInfo.is_null() {
        *pcbNeeded += (core::mem::size_of::<PRINTER_INFO_8W>() as DWORD) + cbDevMode;
        return;
    }

    // Set the pDevMode field (and copy the DevMode).
    *ppPrinterInfoEnd = (*ppPrinterInfoEnd).offset(-(cbDevMode as isize));
    CopyMemory!(
        *ppPrinterInfoEnd as *mut core::ffi::c_void,
        (*pPrinter).pDefaultDevMode as *const core::ffi::c_void,
        cbDevMode as usize,
    );
    (**ppPrinterInfo).pDevMode = *ppPrinterInfoEnd as PDEVMODEW as LPDEVMODEW;

    // Advance to the next structure.
    *ppPrinterInfo = (*ppPrinterInfo).offset(1);
}

unsafe extern "C" fn _LocalGetPrinterLevel9(pPrinter: PLOCAL_PRINTER, ppPrinterInfo: *mut PPRINTER_INFO_9W, ppPrinterInfoEnd: *mut PBYTE, pcbNeeded: PDWORD, cchComputerName: DWORD, wszComputerName: PCWSTR) {
    let mut cbDevMode: DWORD;

    // Calculate the string lengths.
    cbDevMode = ((*(*pPrinter).pDefaultDevMode).dmSize as DWORD).wrapping_add((*(*pPrinter).pDefaultDevMode).dmDriverExtra as DWORD);

    if ppPrinterInfo.is_null() {
        *pcbNeeded = (*pcbNeeded).wrapping_add((core::mem::size_of::<PRINTER_INFO_9W>() as DWORD).wrapping_add(cbDevMode));
        return;
    }

    // Set the pDevMode field (and copy the DevMode).
    *ppPrinterInfoEnd = (*ppPrinterInfoEnd).offset(-(cbDevMode as isize));
    CopyMemory!(*ppPrinterInfoEnd as *mut core::ffi::c_void, (*pPrinter).pDefaultDevMode as *const core::ffi::c_void, cbDevMode as usize);
    (**ppPrinterInfo).pDevMode = *ppPrinterInfoEnd as PDEVMODEW;

    // Advance to the next structure.
    *ppPrinterInfo = (*ppPrinterInfo).add(1);
}
#[no_mangle]
pub unsafe extern "system" fn LocalEnumPrinters(mut Flags: DWORD, mut Name: LPWSTR, mut Level: DWORD, mut pPrinterEnum: LPBYTE, mut cbBuf: DWORD, mut pcbNeeded: LPDWORD, mut pcReturned: LPDWORD) -> BOOL {
    let mut cchComputerName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pPrinterInfoEnd: PBYTE = core::ptr::null_mut();
    let mut pNode: PSKIPLIST_NODE = core::ptr::null_mut();
    let mut wszComputerName: [WCHAR; (2 + MAX_COMPUTERNAME_LENGTH + 1 + 1) as usize] = [0 as WCHAR; (2 + MAX_COMPUTERNAME_LENGTH + 1 + 1) as usize];
    let mut pPrinter: PLOCAL_PRINTER = core::ptr::null_mut();

    // Begin counting.
    *pcbNeeded = 0;
    *pcReturned = 0;

    'Cleanup: {
        if (Flags & PRINTER_ENUM_CONNECTIONS) != 0 || (Flags & PRINTER_ENUM_REMOTE) != 0 || (Flags & PRINTER_ENUM_NETWORK) != 0 {
            dwErrorCode = ERROR_INVALID_NAME;
            break 'Cleanup;
        }

        if !((Flags & PRINTER_ENUM_LOCAL) != 0 || (Flags & PRINTER_ENUM_NAME) != 0) {
            dwErrorCode = ERROR_SUCCESS;
            break 'Cleanup;
        }

        if Level == 3 || Level > 5 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if Level == 1 && (Flags & PRINTER_ENUM_NAME) != 0 && Name.is_null() {
            dwErrorCode = _DumpLevel1PrintProviderInformation(pPrinterEnum, cbBuf, pcbNeeded, pcReturned);
            break 'Cleanup;
        }

        // Check the supplied Name parameter (if any).
        dwErrorCode = _LocalEnumPrintersCheckName(Flags, Name, core::ptr::addr_of_mut!(wszComputerName) as PWSTR, core::ptr::addr_of_mut!(cchComputerName));
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Count the required buffer size and the number of printers.
        pNode = PrinterList.Head.Next[0 as usize] as PSKIPLIST_NODE;
        while !pNode.is_null() {
            'body: {
                pPrinter = (*pNode).Element as PLOCAL_PRINTER;

                if (Flags & PRINTER_ENUM_SHARED) != 0 {
                    break 'body;
                }

                (pfnGetPrinterLevels[Level as usize])(pPrinter, core::ptr::null_mut(), core::ptr::null_mut(), pcbNeeded, cchComputerName, core::ptr::addr_of_mut!(wszComputerName) as *mut WCHAR as PCWSTR);
            }
            pNode = (*pNode).Next[0 as usize] as PSKIPLIST_NODE;
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy over the Printer information.
        pPrinterInfoEnd = pPrinterEnum.offset(*pcbNeeded as isize);

        pNode = PrinterList.Head.Next[0 as usize] as PSKIPLIST_NODE;
        while !pNode.is_null() {
            'body: {
                pPrinter = (*pNode).Element as PLOCAL_PRINTER;

                if (Flags & PRINTER_ENUM_SHARED) != 0 {
                    break 'body;
                }

                (pfnGetPrinterLevels[Level as usize])(pPrinter, core::ptr::addr_of_mut!(pPrinterEnum) as PVOID, core::ptr::addr_of_mut!(pPrinterInfoEnd), core::ptr::null_mut(), cchComputerName, core::ptr::addr_of_mut!(wszComputerName) as *mut WCHAR as PCWSTR);
                *pcReturned = (*pcReturned).wrapping_add(1);
            }
            pNode = (*pNode).Next[0 as usize] as PSKIPLIST_NODE;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalGetPrinter(hPrinter: HANDLE, Level: DWORD, mut pPrinter: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    // We never prepend a Computer Name to the output, but need to provide an empty string,
    // because this variable is passed to StringCbCopyExW.
    let wszDummyComputerName: [WCHAR; 1] = [0x0000u16];

    let mut dwErrorCode: DWORD = 0;
    let mut pPrinterEnd: PBYTE = core::ptr::null_mut();
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/printers.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalGetPrinter\0" as *const u8 as *const ::core::ffi::c_char,
            996 as ::core::ffi::c_int,
            b"LocalGetPrinter(%p, %lu, %p, %lu, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hPrinter,
            Level,
            pPrinter,
            cbBuf,
            pcbNeeded,
        );
    }

    'cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        // Check if this is a printer handle.
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        if Level > 9 {
            // The caller supplied an invalid level for GetPrinter.
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        // Count the required buffer size.
        *pcbNeeded = 0;
        pfnGetPrinterLevels[Level as usize](
            (*pPrinterHandle).pPrinter,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pcbNeeded,
            0,
            core::ptr::addr_of!(wszDummyComputerName) as PCWSTR,
        );

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'cleanup;
        }

        // Copy over the Printer information.
        pPrinterEnd = pPrinter.offset(*pcbNeeded as isize);
        pfnGetPrinterLevels[Level as usize](
            (*pPrinterHandle).pPrinter,
            core::ptr::addr_of_mut!(pPrinter) as PVOID,
            core::ptr::addr_of_mut!(pPrinterEnd),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of!(wszDummyComputerName) as PCWSTR,
        );
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

unsafe extern "C" fn _LocalOpenPortHandle(pwszPortName: PWSTR, phPrinter: PHANDLE) -> DWORD {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hPort: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut::<_LOCAL_HANDLE>();
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut::<_LOCAL_PORT>();
    let mut pPortHandle: PLOCAL_PORT_HANDLE = core::ptr::null_mut::<_LOCAL_PORT_HANDLE>();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut::<_LOCAL_PRINT_MONITOR>();

    'Failure: {
        // Look for this port in our Print Monitor Port list.
        pPort = FindPort(pwszPortName as PCWSTR);
        if pPort.is_null() {
            // The supplied port is unknown to all our Print Monitors.
            dwErrorCode = ERROR_INVALID_NAME as DWORD;
            break 'Failure;
        }

        pPrintMonitor = (*pPort).pPrintMonitor;

        // Call the monitor's OpenPort function.
        if (*pPrintMonitor).bIsLevel2 != 0 {
            bReturnValue = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnOpenPort
                .unwrap_unchecked()(
                (*pPrintMonitor).hMonitor,
                pwszPortName as LPWSTR,
                &raw mut hPort,
            );
        } else {
            bReturnValue = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnOpenPort
                .unwrap_unchecked()(
                pwszPortName as LPWSTR,
                &raw mut hPort,
            );
        }

        if bReturnValue == 0 {
            // The OpenPort function failed. Return its last error.
            dwErrorCode = GetLastError();
            break 'Failure;
        }

        // Create a new generic handle.
        pHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_HANDLE>() as DWORD,
        ) as PLOCAL_HANDLE;
        if pHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Failure;
        }

        // Create a new LOCAL_PORT_HANDLE.
        pPortHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_PORT_HANDLE>() as DWORD,
        ) as PLOCAL_PORT_HANDLE;
        if pPortHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Failure;
        }

        (*pPortHandle).hPort = hPort;
        (*pPortHandle).pPort = pPort;

        // Make the generic handle a Port handle.
        (*pHandle).HandleType = HandleType_Port;
        (*pHandle).pSpecificHandle = pPortHandle as PVOID;

        // Return it.
        *phPrinter = pHandle as HANDLE as *mut core::ffi::c_void;
        return ERROR_SUCCESS as DWORD;
    }

    // Failure:
    if !pHandle.is_null() {
        DllFreeSplMem(pHandle as PVOID);
    }

    if !pPortHandle.is_null() {
        DllFreeSplMem(pPortHandle as PVOID);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _LocalOpenPrinterHandle(
    mut pwszPrinterName: PWSTR,
    mut pwszJobParameter: PWSTR,
    mut phPrinter: PHANDLE,
    mut pDefault: PPRINTER_DEFAULTSW,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut dwJobID: DWORD = 0;
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut pPrinter: PLOCAL_PRINTER = core::ptr::null_mut();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();
    let mut wszFullPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    'body: {
        // Retrieve the printer from the list.
        pPrinter = LookupElementSkiplist(
            core::ptr::addr_of_mut!(PrinterList),
            core::ptr::addr_of_mut!(pwszPrinterName) as PVOID,
            core::ptr::null_mut(),
        ) as PLOCAL_PRINTER;
        if pPrinter.is_null() {
            // The printer does not exist.
            dwErrorCode = ERROR_INVALID_NAME;
            break 'body;
        }

        // Create a new generic handle.
        pHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_HANDLE>() as DWORD,
        ) as PLOCAL_HANDLE;
        if pHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'body;
        }

        // Create a new LOCAL_PRINTER_HANDLE.
        pPrinterHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_PRINTER_HANDLE>() as DWORD,
        ) as PLOCAL_PRINTER_HANDLE;
        if pPrinterHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'body;
        }

        (*pPrinterHandle).hSPLFile = INVALID_HANDLE_VALUE;
        (*pPrinterHandle).pPrinter = pPrinter;

        // Check if a datatype was given.
        if !pDefault.is_null() && !(*pDefault).pDatatype.is_null() {
            // Use the datatype if it's valid.
            if FindDatatype((*pPrinter).pPrintProcessor, (*pDefault).pDatatype) == 0 {
                dwErrorCode = ERROR_INVALID_DATATYPE;
                break 'body;
            }

            (*pPrinterHandle).pwszDatatype = AllocSplStr((*pDefault).pDatatype);
        } else {
            // Use the default datatype.
            (*pPrinterHandle).pwszDatatype =
                AllocSplStr((*pPrinter).pwszDefaultDatatype);
        }

        // Check if a DevMode was given, otherwise use the default.
        if !pDefault.is_null() && !(*pDefault).pDevMode.is_null() {
            (*pPrinterHandle).pDevMode = DuplicateDevMode((*pDefault).pDevMode);
        } else {
            (*pPrinterHandle).pDevMode = DuplicateDevMode((*pPrinter).pDefaultDevMode);
        }

        // Check if the caller wants a handle to an existing Print Job.
        if !pwszJobParameter.is_null() {
            // The "Job " string has to follow now.
            if wcsncmp(
                pwszJobParameter as *const u16,
                RUSTCONV_WSTR_E19ED02EC71C.as_ptr(),
                4,
            ) != 0
            {
                dwErrorCode = ERROR_INVALID_NAME;
                break 'body;
            }

            // Skip the "Job " string.
            pwszJobParameter = pwszJobParameter.offset(4 as isize);

            // Skip even more whitespace.
            while *pwszJobParameter == ' ' as u16 {
                pwszJobParameter = pwszJobParameter.offset(1);
            }

            // Finally extract the desired Job ID.
            dwJobID = wcstoul(
                pwszJobParameter as *const u16,
                core::ptr::null_mut(),
                10,
            ) as DWORD;
            if !IS_VALID_JOB_ID!(dwJobID) {
                // The user supplied an invalid Job ID.
                dwErrorCode = ERROR_INVALID_NAME;
                break 'body;
            }

            // Look for this job in the Global Job List.
            pJob = LookupElementSkiplist(
                core::ptr::addr_of_mut!(GlobalJobList),
                core::ptr::addr_of_mut!(dwJobID) as PVOID,
                core::ptr::null_mut(),
            ) as PLOCAL_JOB;
            if pJob.is_null() || (*pJob).pPrinter != pPrinter {
                // The user supplied a non-existing Job ID or the Job ID does not belong to the supplied printer name.
                dwErrorCode = ERROR_INVALID_PRINTER_NAME;
                break 'body;
            }

            // Try to open its SPL file.
            GetJobFilePath(RUSTCONV_WSTR_D575D22E81A4.as_ptr(), dwJobID, core::ptr::addr_of_mut!(wszFullPath) as PWSTR);
            (*pPrinterHandle).hSPLFile = CreateFileW(
                core::ptr::addr_of_mut!(wszFullPath) as PWSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
            if (*pPrinterHandle).hSPLFile == INVALID_HANDLE_VALUE {
                dwErrorCode = GetLastError();
                break 'body;
            }

            // Associate the job to our Printer Handle, but don't set bStartedDoc.
            // This prevents the caller from doing further StartDocPrinter, WritePrinter, etc. calls on it.
            (*pPrinterHandle).pJob = pJob;
        }

        // Make the generic handle a Printer handle.
        (*pHandle).HandleType = HandleType_Printer;
        (*pHandle).pSpecificHandle = pPrinterHandle as PVOID;

        // Return it.
        *phPrinter = pHandle as HANDLE;
        return ERROR_SUCCESS;
    }

    // Failure:
    if !pHandle.is_null() {
        DllFreeSplMem(pHandle as PVOID);
    }

    if !pPrinterHandle.is_null() {
        if !(*pPrinterHandle).pwszDatatype.is_null() {
            DllFreeSplStr((*pPrinterHandle).pwszDatatype);
        }

        if !(*pPrinterHandle).pDevMode.is_null() {
            DllFreeSplMem((*pPrinterHandle).pDevMode as PVOID);
        }

        DllFreeSplMem(pPrinterHandle as PVOID);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _LocalOpenPrintServerHandle(phPrinter: PHANDLE) -> DWORD {
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();

    // Create a new generic handle.
    pHandle = DllAllocSplMem(core::mem::size_of::<LOCAL_HANDLE>() as DWORD) as PLOCAL_HANDLE;
    if pHandle.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    // Make the generic handle a Print Server handle.
    (*pHandle).HandleType = HandleType_PrintServer;
    (*pHandle).pSpecificHandle = core::ptr::null_mut();

    // Return it.
    *phPrinter = pHandle as HANDLE;
    return ERROR_SUCCESS;
}

unsafe extern "C" fn _LocalOpenXcvHandle(mut pwszParameter: PWSTR, phPrinter: PHANDLE) -> DWORD {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hXcv: HANDLE = core::ptr::null_mut();
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pXcvHandle: PLOCAL_XCV_HANDLE = core::ptr::null_mut();

    // Skip the "Xcv" string.
    pwszParameter = pwszParameter.add(3 as usize);

    'failure: {
        // Is XcvMonitor or XcvPort requested?
        if wcsncmp(
            pwszParameter as *const u16,
            RUSTCONV_WSTR_F5C28976B418.as_ptr(),
            8 as usize,
        ) == 0 as i32
        {
            // Skip the "Monitor " string.
            pwszParameter = pwszParameter.add(8 as usize);

            // Look for this monitor in our Print Monitor list.
            pPrintMonitor = FindPrintMonitor(pwszParameter as PCWSTR);
            if pPrintMonitor.is_null() {
                // The caller supplied a non-existing Monitor name.
                dwErrorCode = ERROR_INVALID_NAME as DWORD;
                break 'failure;
            }
        } else if wcsncmp(
            pwszParameter as *const u16,
            RUSTCONV_WSTR_4B731B4B251F.as_ptr(),
            5 as usize,
        ) == 0 as i32
        {
            // Skip the "Port " string.
            pwszParameter = pwszParameter.add(5 as usize);

            // Look for this port in our Print Monitor Port list.
            pPort = FindPort(pwszParameter as PCWSTR);
            if pPort.is_null() {
                // The supplied port is unknown to all our Print Monitors.
                dwErrorCode = ERROR_INVALID_NAME as DWORD;
                break 'failure;
            }

            pPrintMonitor = (*pPort).pPrintMonitor;
        } else {
            dwErrorCode = ERROR_INVALID_NAME as DWORD;
            break 'failure;
        }

        // Call the monitor's XcvOpenPort function.
        if (*pPrintMonitor).bIsLevel2 != 0 {
            bReturnValue = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnXcvOpenPort
                .unwrap_unchecked()(
                (*pPrintMonitor).hMonitor,
                pwszParameter as LPCWSTR,
                SERVER_EXECUTE as ACCESS_MASK,
                core::ptr::addr_of_mut!(hXcv),
            );
        } else {
            bReturnValue = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnXcvOpenPort
                .unwrap_unchecked()(
                pwszParameter as LPCWSTR,
                SERVER_EXECUTE as ACCESS_MASK,
                core::ptr::addr_of_mut!(hXcv),
            );
        }

        if bReturnValue == 0 {
            // The XcvOpenPort function failed. Return its last error.
            dwErrorCode = GetLastError();
            break 'failure;
        }

        // Create a new generic handle.
        pHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_HANDLE>() as DWORD,
        ) as PLOCAL_HANDLE;
        if pHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'failure;
        }

        // Create a new LOCAL_XCV_HANDLE.
        pXcvHandle = DllAllocSplMem(
            core::mem::size_of::<LOCAL_XCV_HANDLE>() as DWORD,
        ) as PLOCAL_XCV_HANDLE;
        if pXcvHandle.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'failure;
        }

        (*pXcvHandle).hXcv = hXcv;
        (*pXcvHandle).pPrintMonitor = pPrintMonitor;

        // Make the generic handle a Xcv handle.
        (*pHandle).HandleType = HandleType_Xcv;
        (*pHandle).pSpecificHandle = pXcvHandle as PVOID;

        // Return it.
        *phPrinter = pHandle as HANDLE;
        return ERROR_SUCCESS as DWORD;
    }

    // Failure:
    if !pHandle.is_null() {
        DllFreeSplMem(pHandle as PVOID);
    }

    if !pXcvHandle.is_null() {
        DllFreeSplMem(pXcvHandle as PVOID);
    }

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn LocalPrinterMessageBox(hPrinter: HANDLE, Error: DWORD, hWnd: HWND, pText: LPWSTR, pCaption: LPWSTR, dwType: DWORD) -> DWORD {
    SetLastError(ERROR_INVALID_HANDLE as DWORD);
    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn LocalOpenPrinter(mut lpPrinterName: PWSTR, phPrinter: *mut HANDLE, pDefault: PPRINTER_DEFAULTSW) -> BOOL {
    let mut cchComputerName: DWORD = 0;
    let mut cchFirstParameter: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut p: PWSTR = lpPrinterName;
    let mut pwszFirstParameter: PWSTR = core::ptr::null_mut();
    let mut pwszSecondParameter: PWSTR = core::ptr::null_mut();
    let mut wszComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];

    *phPrinter = core::ptr::null_mut();

    'Cleanup: {
        if lpPrinterName.is_null() {
            // The caller wants a Print Server handle and provided a NULL string.
            dwErrorCode = _LocalOpenPrintServerHandle(phPrinter);
            break 'Cleanup;
        }

        // Skip any server name in the first parameter.
        // Does lpPrinterName begin with two backslashes to indicate a server name?
        if *lpPrinterName.add((0) as usize) == 0x005Cu16
            && *lpPrinterName.add((1) as usize) == 0x005Cu16
        {
            // Skip these two backslashes.
            lpPrinterName = lpPrinterName.add((2) as usize);

            // Look for the terminating null character or closing backslash.
            p = lpPrinterName;
            while *p != 0x0000u16 && *p != 0x005Cu16 {
                p = p.add(1);
            }

            // Get the local computer name for comparison.
            cchComputerName = _countof!(wszComputerName) as DWORD;
            if GetComputerNameW(wszComputerName.as_mut_ptr(), &raw mut cchComputerName) == 0 {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // Now compare this string excerpt with the local computer name.
            // The input parameter may not be writable, so we can't null-terminate the input string at this point.
            // This print provider only supports local printers, so both strings have to match.
            if (p.offset_from(lpPrinterName) as i64) != (cchComputerName as i64)
                || _wcsnicmp(
                    lpPrinterName,
                    wszComputerName.as_mut_ptr(),
                    cchComputerName as usize,
                ) != 0
            {
                dwErrorCode = ERROR_INVALID_NAME;
                break 'Cleanup;
            }

            // If lpPrinterName is only "\\COMPUTERNAME" with nothing more, the caller wants a handle to the local Print Server.
            if *p == 0 {
                // The caller wants a Print Server handle and provided a string like:
                //    "\\COMPUTERNAME"
                dwErrorCode = _LocalOpenPrintServerHandle(phPrinter);
                break 'Cleanup;
            }

            // We have checked the server name and don't need it anymore.
            lpPrinterName = p.add((1) as usize);
        }

        // Look for a comma. If it exists, it indicates the end of the first parameter.
        pwszSecondParameter = wcschr(lpPrinterName, 0x002Cu16) as PWSTR;
        if !pwszSecondParameter.is_null() {
            cchFirstParameter = pwszSecondParameter.offset_from(p) as DWORD;
        } else {
            cchFirstParameter = wcslen(lpPrinterName) as DWORD;
        }

        // We must have at least one parameter.
        if cchFirstParameter == 0 && pwszSecondParameter.is_null() {
            dwErrorCode = ERROR_INVALID_NAME;
            break 'Cleanup;
        }

        // Do we have a first parameter?
        if cchFirstParameter != 0 {
            // Yes, extract it.
            // No null-termination is necessary here, because DllAllocSplMem returns a zero-initialized buffer.
            pwszFirstParameter = DllAllocSplMem(
                (cchFirstParameter + (1) as DWORD)
                    * (core::mem::size_of::<WCHAR>() as DWORD),
            ) as PWSTR;
            CopyMemory!(
                pwszFirstParameter as *mut core::ffi::c_void,
                lpPrinterName as *const core::ffi::c_void,
                (cchFirstParameter as usize) * core::mem::size_of::<WCHAR>(),
            );
        }

        // Do we have a second parameter?
        if !pwszSecondParameter.is_null() {
            // Yes, skip the comma at the beginning.
            pwszSecondParameter = pwszSecondParameter.add(1);

            // Skip whitespace as well.
            while *pwszSecondParameter == 0x0020u16 {
                pwszSecondParameter = pwszSecondParameter.add(1);
            }
        }

        // Now we can finally check the type of handle actually requested.
        if !pwszFirstParameter.is_null()
            && !pwszSecondParameter.is_null()
            && wcsncmp(
                pwszSecondParameter,
                RUSTCONV_WSTR_9665DD34268C.as_ptr(),
                (4) as usize,
            ) == 0
        {
            // The caller wants a port handle and provided a string like:
            //    "LPT1:, Port"
            //    "\\COMPUTERNAME\LPT1:, Port"
            dwErrorCode = _LocalOpenPortHandle(pwszFirstParameter, phPrinter);
        } else if pwszFirstParameter.is_null()
            && !pwszSecondParameter.is_null()
            && wcsncmp(
                pwszSecondParameter,
                RUSTCONV_WSTR_C1615E893D5D.as_ptr(),
                (3) as usize,
            ) == 0
        {
            // The caller wants an Xcv handle and provided a string like:
            //    ", XcvMonitor Local Port"
            //    "\\COMPUTERNAME\, XcvMonitor Local Port"
            //    ", XcvPort LPT1:"
            //    "\\COMPUTERNAME\, XcvPort LPT1:"
            dwErrorCode = _LocalOpenXcvHandle(pwszSecondParameter, phPrinter);
        } else {
            // The caller wants a Printer or Printer Job handle and provided a string like:
            //    "HP DeskJet"
            //    "\\COMPUTERNAME\HP DeskJet"
            //    "HP DeskJet, Job 5"
            //    "\\COMPUTERNAME\HP DeskJet, Job 5"
            dwErrorCode = _LocalOpenPrinterHandle(
                pwszFirstParameter,
                pwszSecondParameter,
                phPrinter,
                pDefault,
            );
        }
    }

    // Cleanup:
    if !pwszFirstParameter.is_null() {
        DllFreeSplMem(pwszFirstParameter as PVOID);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalReadPrinter(hPrinter: HANDLE, pBuf: PVOID, cbBuf: DWORD, pNoBytesRead: PDWORD) -> BOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD;
    let pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPortHandle: PLOCAL_PORT_HANDLE = core::ptr::null_mut();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Port handles are an entirely different thing.
        if (*pHandle).HandleType == HandleType_Port {
            pPortHandle = (*pHandle).pSpecificHandle as PLOCAL_PORT_HANDLE;

            // Call the monitor's ReadPort function.
            if (*(*(*pPortHandle).pPort).pPrintMonitor).bIsLevel2 != 0 {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as PMONITOR2))
                    .pfnReadPort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    pBuf as LPBYTE,
                    cbBuf,
                    pNoBytesRead as LPDWORD,
                );
            } else {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as LPMONITOREX))
                    .Monitor
                    .pfnReadPort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    pBuf as LPBYTE,
                    cbBuf,
                    pNoBytesRead as LPDWORD,
                );
            }

            if bReturnValue == 0 {
                // The ReadPort function failed. Return its last error.
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // We were successful!
            dwErrorCode = ERROR_SUCCESS;
            break 'Cleanup;
        }

        // The remaining function deals with Printer handles only.
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // ReadPrinter needs an opened SPL file to work.
        // This only works if a Printer Job Handle was requested in OpenPrinter.
        if (*pPrinterHandle).hSPLFile == INVALID_HANDLE_VALUE {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Pass the parameters to ReadFile.
        if ReadFile(
            (*pPrinterHandle).hSPLFile,
            pBuf,
            cbBuf,
            pNoBytesRead,
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalStartDocPrinter(hPrinter: HANDLE, Level: DWORD, pDocInfo: LPBYTE) -> DWORD {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwReturnValue: DWORD = 0;
    let mut pDocInfo1: PDOC_INFO_1W = pDocInfo as PDOC_INFO_1W;
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPortHandle: PLOCAL_PORT_HANDLE = core::ptr::null_mut();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Port handles are an entirely different thing.
        if (*pHandle).HandleType as core::ffi::c_uint
            == HandleType_Port as core::ffi::c_int as core::ffi::c_uint
        {
            pPortHandle = (*pHandle).pSpecificHandle as PLOCAL_PORT_HANDLE;

            // Claim the job exclusively for this port handle.
            pJob = (*(*pPortHandle).pPort).pNextJobToProcess;
            (*(*pPortHandle).pPort).pNextJobToProcess = core::ptr::null_mut();
            if pJob.is_null() {
                RtlAssert(
                    b"pJob\0".as_ptr() as *const core::ffi::c_char as PVOID,
                    b"printers.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
                    1636 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            };

            // Call the monitor's StartDocPort function.
            if (*(*(*pPortHandle).pPort).pPrintMonitor).bIsLevel2 != 0 {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as PMONITOR2))
                    .pfnStartDocPort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    (*(*pJob).pPrinter).pwszPrinterName as LPWSTR,
                    (*pJob).dwJobID,
                    Level,
                    pDocInfo as LPBYTE,
                );
            } else {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as LPMONITOREX))
                    .Monitor
                    .pfnStartDocPort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    (*(*pJob).pPrinter).pwszPrinterName as LPWSTR,
                    (*pJob).dwJobID,
                    Level,
                    pDocInfo as LPBYTE,
                );
            }

            if bReturnValue == 0 {
                // The StartDocPort function failed. Return its last error.
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // We were successful!
            dwErrorCode = ERROR_SUCCESS;
            dwReturnValue = (*pJob).dwJobID;
            break 'Cleanup;
        }

        // The remaining function deals with Printer handles only.
        if (*pHandle).HandleType as core::ffi::c_uint
            != HandleType_Printer as core::ffi::c_int as core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        if pDocInfo1.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // pJob may already be occupied if this is a Print Job handle.
        if !(*pPrinterHandle).pJob.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Check the validity of the datatype if we got one.
        if !(*pDocInfo1).pDatatype.is_null()
            && FindDatatype(
                (*(*pPrinterHandle).pJob).pPrintProcessor,
                (*pDocInfo1).pDatatype as PCWSTR,
            ) == 0
        {
            dwErrorCode = ERROR_INVALID_DATATYPE;
            break 'Cleanup;
        }

        // Check if this is the right document information level.
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // All requirements are met - create a new job.
        dwErrorCode = CreateJob(pPrinterHandle);
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Use any given datatype.
        if !(*pDocInfo1).pDatatype.is_null()
            && ReallocSplStr(
                core::ptr::addr_of_mut!((*(*pPrinterHandle).pJob).pwszDatatype),
                (*pDocInfo1).pDatatype as PCWSTR,
            ) == 0
        {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        // Use any given document name.
        if !(*pDocInfo1).pDocName.is_null()
            && ReallocSplStr(
                core::ptr::addr_of_mut!((*(*pPrinterHandle).pJob).pwszDocumentName),
                (*pDocInfo1).pDocName as PCWSTR,
            ) == 0
        {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        // We were successful!
        dwErrorCode = ERROR_SUCCESS;
        dwReturnValue = (*(*pPrinterHandle).pJob).dwJobID;
    }
    SetLastError(dwErrorCode);
    return dwReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn LocalStartPagePrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() || (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // We require StartDocPrinter or AddJob to be called first.
        if (*pPrinterHandle).bStartedDoc() == 0 {
            dwErrorCode = ERROR_SPL_NO_STARTDOC;
            break 'Cleanup;
        }

        // Increase the page count.
        (*(*pPrinterHandle).pJob).dwTotalPages = (*(*pPrinterHandle).pJob).dwTotalPages.wrapping_add(1);
        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalWritePrinter(hPrinter: HANDLE, pBuf: LPVOID, cbBuf: DWORD, pcWritten: LPDWORD) -> BOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPortHandle: PLOCAL_PORT_HANDLE = core::ptr::null_mut();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Port handles are an entirely different thing.
        if (*pHandle).HandleType == HandleType_Port {
            pPortHandle = (*pHandle).pSpecificHandle as PLOCAL_PORT_HANDLE;

            // Call the monitor's WritePort function.
            if (*(*(*pPortHandle).pPort).pPrintMonitor).bIsLevel2 != 0 {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as PMONITOR2))
                    .pfnWritePort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    pBuf as LPBYTE,
                    cbBuf,
                    pcWritten as LPDWORD,
                );
            } else {
                bReturnValue = (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as LPMONITOREX))
                    .Monitor
                    .pfnWritePort
                    .unwrap_unchecked()(
                    (*pPortHandle).hPort,
                    pBuf as LPBYTE,
                    cbBuf,
                    pcWritten as LPDWORD,
                );
            }

            if bReturnValue == 0 {
                // The WritePort function failed. Return its last error.
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // We were successful!
            dwErrorCode = ERROR_SUCCESS;
            break 'Cleanup;
        }

        // The remaining function deals with Printer handles only.
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // We require StartDocPrinter or AddJob to be called first.
        if (*pPrinterHandle).bStartedDoc() == 0 {
            dwErrorCode = ERROR_SPL_NO_STARTDOC;
            break 'Cleanup;
        }

        // TODO: This function is only called when doing non-spooled printing.
        // This needs to be investigated further. We can't just use pPrinterHandle->hSPLFile here, because that's currently reserved for Printer Job handles (see LocalReadPrinter).

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalEndPagePrinter(hPrinter: HANDLE) -> BOOL {
    let dwErrorCode: DWORD;
    let pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    'cleanup: {
        // Sanity checks.
        if pHandle.is_null() || (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        // This function doesn't do anything else for now.
        dwErrorCode = ERROR_SUCCESS;
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalEndDocPrinter(hPrinter: HANDLE) -> BOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPortHandle: PLOCAL_PORT_HANDLE = ::core::ptr::null_mut::<_LOCAL_PORT_HANDLE>();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE =
        ::core::ptr::null_mut::<_LOCAL_PRINTER_HANDLE>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/printers.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalEndDocPrinter\0" as *const u8 as *const ::core::ffi::c_char,
            1865 as ::core::ffi::c_int,
            b"LocalEndDocPrinter(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hPrinter,
        );
    }
    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
            break 'Cleanup;
        }

        // Port handles are an entirely different thing.
        if (*pHandle).HandleType as ::core::ffi::c_uint
            == HandleType_Port as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            pPortHandle = (*pHandle).pSpecificHandle as PLOCAL_PORT_HANDLE;

            // Call the monitor's EndDocPort function.
            if (*(*(*pPortHandle).pPort).pPrintMonitor).bIsLevel2 != 0 {
                bReturnValue =
                    (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as PMONITOR2))
                        .pfnEndDocPort
                        .unwrap_unchecked()((*pPortHandle).hPort);
            } else {
                bReturnValue =
                    (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as LPMONITOREX))
                        .Monitor
                        .pfnEndDocPort
                        .unwrap_unchecked()((*pPortHandle).hPort);
            }

            if bReturnValue == 0 {
                // The EndDocPort function failed. Return its last error.
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            // We were successful!
            dwErrorCode = ERROR_SUCCESS as DWORD;
            break 'Cleanup;
        }

        // The remaining function deals with Printer handles only.
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // We require StartDocPrinter or AddJob to be called first.
        if (*pPrinterHandle).bStartedDoc() == 0 {
            dwErrorCode = ERROR_SPL_NO_STARTDOC as DWORD;
            break 'Cleanup;
        }

        // TODO: Something like ScheduleJob

        // Finish the job.
        (*pPrinterHandle).set_bStartedDoc(FALSE as BOOL);
        (*pPrinterHandle).pJob = ::core::ptr::null_mut::<_LOCAL_JOB>();
        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

unsafe extern "C" fn _LocalClosePortHandle(pPortHandle: PLOCAL_PORT_HANDLE) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/printers.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"_LocalClosePortHandle\0" as *const u8 as *const ::core::ffi::c_char,
            1928 as ::core::ffi::c_int,
            b"LocalClosePortHandle\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*(*(*pPortHandle).pPort).pPrintMonitor).bIsLevel2 != 0 {
        (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as PMONITOR2))
            .pfnClosePort
            .unwrap_unchecked()((*pPortHandle).hPort);
    } else {
        (*((*(*(*pPortHandle).pPort).pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnClosePort
            .unwrap_unchecked()((*pPortHandle).hPort);
    };
}

unsafe extern "C" fn _LocalClosePrinterHandle(pPrinterHandle: PLOCAL_PRINTER_HANDLE) {
    // Terminate any started job.
    if !(*pPrinterHandle).pJob.is_null() {
        FreeJob((*pPrinterHandle).pJob);
    }

    // Free memory for the fields.
    DllFreeSplMem((*pPrinterHandle).pDevMode as PVOID);
    DllFreeSplStr((*pPrinterHandle).pwszDatatype);
}

unsafe extern "C" fn _LocalCloseXcvHandle(pXcvHandle: PLOCAL_XCV_HANDLE) {
    if (*(*pXcvHandle).pPrintMonitor).bIsLevel2 != 0 {
        (*((*(*pXcvHandle).pPrintMonitor).pMonitor as PMONITOR2))
            .pfnXcvClosePort
            .unwrap_unchecked()((*pXcvHandle).hXcv);
    } else {
        (*((*(*pXcvHandle).pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnXcvClosePort
            .unwrap_unchecked()((*pXcvHandle).hXcv);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LocalClosePrinter(hPrinter: HANDLE) -> BOOL {
    let pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;

    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE as BOOL;
    }

    if (*pHandle).HandleType == HandleType_Port {
        _LocalClosePortHandle((*pHandle).pSpecificHandle as PLOCAL_PORT_HANDLE);
    } else if (*pHandle).HandleType == HandleType_Printer {
        _LocalClosePrinterHandle((*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE);
    } else if (*pHandle).HandleType == HandleType_PrintServer {
        // Nothing to do.
    } else if (*pHandle).HandleType == HandleType_Xcv {
        _LocalCloseXcvHandle((*pHandle).pSpecificHandle as PLOCAL_XCV_HANDLE);
    }

    // Free memory for the handle and the specific handle (if any).
    if !(*pHandle).pSpecificHandle.is_null() {
        DllFreeSplMem((*pHandle).pSpecificHandle);
    }

    DllFreeSplMem(pHandle as PVOID);

    return TRUE as BOOL;
}
