use crate::localspl::forms::InitializeFormList;
use crate::localspl::jobs::InitializeGlobalJobList;
use crate::localspl::forms::LocalAddForm;
use crate::localspl::jobs::LocalAddJob;
use crate::localspl::forms::LocalDeleteForm;
use crate::localspl::forms::LocalEnumForms;
use crate::localspl::jobs::LocalEnumJobs;
use crate::localspl::forms::LocalGetForm;
use crate::localspl::jobs::LocalGetJob;
use crate::localspl::jobs::LocalScheduleJob;
use crate::localspl::forms::LocalSetForm;
use crate::localspl::jobs::LocalSetJob;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;

// win32ss/printing/include/prtprocenv.h: the print environment of THIS architecture. A
// non-static const global in C, so C linkage (the island declares it extern); the translation
// had baked one arch's value ("Windows ARM64") into every build.
const fn prtprocenv<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub static wszCurrentEnvironment: [WCHAR; 15] = prtprocenv(b"Windows NT x86\0");
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub static wszCurrentEnvironment: [WCHAR; 12] = prtprocenv(b"Windows x64\0");
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static wszCurrentEnvironment: [WCHAR; 14] = prtprocenv(b"Windows ARM64\0");
#[no_mangle]
pub static cbCurrentEnvironment: DWORD = core::mem::size_of_val(&wszCurrentEnvironment) as DWORD;
pub static RUSTCONV_WSTR_04E7ED89D47D: [u16; 60] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074,
    0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_0BA09314F12F: [u16; 18] = [
    0x0053u16, 0x0079, 0x006D, 0x0062, 0x006F, 0x006C, 0x0069, 0x0063, 0x004C, 0x0069,
    0x006E, 0x006B, 0x0056, 0x0061, 0x006C, 0x0075, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_745F20C64BE5: [u16; 39] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_C696D38862E6: [u16; 48] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x0050,
    0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_AB6F5D973C42: [u16; 32] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054,
    0x0020, 0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0020, 0x0050, 0x0072, 0x0069,
    0x006E, 0x0074, 0x0020, 0x0050, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x006F,
    0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D97C39AECAAA: [u16; 26] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054,
    0x0020, 0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0020, 0x0050, 0x0072, 0x0069,
    0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_ED5B630C04E2: [u16; 27] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x006C, 0x0079, 0x0020, 0x0063, 0x006F,
    0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0065, 0x0064, 0x0020, 0x0050, 0x0072,
    0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];

#[no_mangle]
pub static mut hPrintKey: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

#[no_mangle]
pub static mut hPrintersKey: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

#[no_mangle]
pub static mut wszJobDirectory: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut cchJobDirectory: DWORD = 0;

#[no_mangle]
pub static mut wszSpoolDirectory: [WCHAR; 260] = [0; 260];

#[no_mangle]
pub static mut cchSpoolDirectory: DWORD = 0;

#[no_mangle]
pub static mut dwSpoolerMajorVersion: DWORD = 3 as DWORD;

#[no_mangle]
pub static mut dwSpoolerMinorVersion: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut wszDefaultDocumentName: [WCHAR; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [WCHAR; 25]>(
        *b"L\0o\0c\0a\0l\0 \0D\0o\0w\0n\0l\0e\0v\0e\0l\0 \0D\0o\0c\0u\0m\0e\0n\0t\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPrintProviderInfo: [PCWSTR; 3] = unsafe {
    [
        RUSTCONV_C2R_WSTR_AB6F5D973C42.as_ptr(),
        RUSTCONV_C2R_WSTR_ED5B630C04E2.as_ptr(),
        RUSTCONV_C2R_WSTR_D97C39AECAAA.as_ptr(),
    ]
};

static mut _PrintProviderFunctions: PRINTPROVIDOR = unsafe {
    _PRINTPROVIDOR {
        fpOpenPrinter: Some(
            LocalOpenPrinter,
        ),
        fpSetJob: Some(
            LocalSetJob,
        ),
        fpGetJob: Some(
            LocalGetJob,
        ),
        fpEnumJobs: Some(
            LocalEnumJobs,
        ),
        fpAddPrinter: None,
        fpDeletePrinter: None,
        fpSetPrinter: None,
        fpGetPrinter: Some(
            LocalGetPrinter,
        ),
        fpEnumPrinters: Some(
            LocalEnumPrinters,
        ),
        fpAddPrinterDriver: Some(
            LocalAddPrinterDriver,
        ),
        fpEnumPrinterDrivers: Some(
            LocalEnumPrinterDrivers,
        ),
        fpGetPrinterDriver: Some(
            LocalGetPrinterDriver,
        ),
        fpGetPrinterDriverDirectory: Some(
            LocalGetPrinterDriverDirectory,
        ),
        fpDeletePrinterDriver: None,
        fpAddPrintProcessor: None,
        fpEnumPrintProcessors: Some(
            LocalEnumPrintProcessors,
        ),
        fpGetPrintProcessorDirectory: Some(
            LocalGetPrintProcessorDirectory,
        ),
        fpDeletePrintProcessor: None,
        fpEnumPrintProcessorDatatypes: Some(
            LocalEnumPrintProcessorDatatypes,
        ),
        fpStartDocPrinter: Some(
            LocalStartDocPrinter,
        ),
        fpStartPagePrinter: Some(LocalStartPagePrinter),
        fpWritePrinter: Some(
            LocalWritePrinter,
        ),
        fpEndPagePrinter: Some(LocalEndPagePrinter),
        fpAbortPrinter: None,
        fpReadPrinter: Some(
            LocalReadPrinter,
        ),
        fpEndDocPrinter: Some(LocalEndDocPrinter),
        fpAddJob: Some(
            LocalAddJob,
        ),
        fpScheduleJob: Some(LocalScheduleJob),
        fpGetPrinterData: Some(
            LocalGetPrinterData,
        ),
        fpSetPrinterData: Some(
            LocalSetPrinterData,
        ),
        fpWaitForPrinterChange: None,
        fpClosePrinter: Some(LocalClosePrinter),
        fpAddForm: Some(LocalAddForm),
        fpDeleteForm: Some(LocalDeleteForm),
        fpGetForm: Some(
            LocalGetForm,
        ),
        fpSetForm: Some(LocalSetForm),
        fpEnumForms: Some(
            LocalEnumForms,
        ),
        fpEnumMonitors: Some(
            LocalEnumMonitors,
        ),
        fpEnumPorts: Some(
            LocalEnumPorts,
        ),
        fpAddPort: Some(LocalAddPort),
        fpConfigurePort: Some(
            LocalConfigurePort,
        ),
        fpDeletePort: Some(LocalDeletePort),
        fpCreatePrinterIC: None,
        fpPlayGdiScriptOnPrinterIC: None,
        fpDeletePrinterIC: None,
        fpAddPrinterConnection: None,
        fpDeletePrinterConnection: None,
        fpPrinterMessageBox: Some(
            LocalPrinterMessageBox,
        ),
        fpAddMonitor: Some(LocalAddMonitor),
        fpDeleteMonitor: Some(
            LocalDeleteMonitor,
        ),
        fpResetPrinter: None,
        fpGetPrinterDriverEx: Some(
            LocalGetPrinterDriverEx,
        ),
        fpFindFirstPrinterChangeNotification: None,
        fpFindClosePrinterChangeNotification: None,
        fpAddPortEx: Some(
            LocalAddPortEx,
        ),
        fpShutDown: None,
        fpRefreshPrinterChangeNotification: None,
        fpOpenPrinterEx: None,
        fpAddPrinterEx: None,
        fpSetPort: Some(LocalSetPort),
        fpEnumPrinterData: None,
        fpDeletePrinterData: None,
        fpClusterSplOpen: None,
        fpClusterSplClose: None,
        fpClusterSplIsAlive: None,
        fpSetPrinterDataEx: Some(
            LocalSetPrinterDataEx,
        ),
        fpGetPrinterDataEx: Some(
            LocalGetPrinterDataEx,
        ),
        fpEnumPrinterDataEx: None,
        fpEnumPrinterKey: None,
        fpDeletePrinterDataEx: None,
        fpDeletePrinterKey: None,
        fpSeekPrinter: None,
        fpDeletePrinterDriverEx: None,
        fpAddPerMachineConnection: None,
        fpDeletePerMachineConnection: None,
        fpEnumPerMachineConnections: None,
        fpXcvData: Some(
            LocalXcvData,
        ),
        fpAddPrinterDriverEx: Some(
            LocalAddPrinterDriverEx,
        ),
        fpSplReadPrinter: None,
        fpDriverUnloadComplete: None,
        fpGetSpoolFileInfo: Some(
            LocalGetSpoolFileInfo,
        ),
        fpCommitSpoolData: Some(
            LocalCommitSpoolData,
        ),
        fpCloseSpoolFileHandle: Some(
            LocalCloseSpoolFileHandle,
        ),
        fpFlushPrinter: None,
        fpSendRecvBidiData: None,
        fpAddDriverCatalog: None,
    }
};

unsafe extern "C" fn _InitializeLocalSpooler() -> BOOL {
    let wszPrintersPath: [WCHAR; 10] = [0x005Cu16, 0x0050, 0x0052, 0x0049, 0x004E, 0x0054, 0x0045, 0x0052, 0x0053, 0x0000];
    let cchPrintersPath: DWORD = (_countof!(wszPrintersPath) as DWORD) - 1;
    let wszSpoolPath: [WCHAR; 7] = [0x005Cu16, 0x0073, 0x0070, 0x006F, 0x006F, 0x006C, 0x0000];
    let cchSpoolPath: DWORD = (_countof!(wszSpoolPath) as DWORD) - 1;
    let wszSymbolicLinkValue: [WCHAR; 78] = [
    0x005Cu16, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052, 0x0059, 0x005C,
    0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x005C, 0x0053, 0x004F,
    0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E,
    0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C, 0x0043, 0x0075,
    0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069,
    0x006F, 0x006E, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x0050,
    0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
];
    let cbSymbolicLinkValue: DWORD =
        (core::mem::size_of::<[WCHAR; 78]>() as DWORD) - (core::mem::size_of::<WCHAR>() as DWORD);

    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cbData: DWORD;
    let mut dwErrorCode: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();

    'cleanup: {
        // On startup, always create a volatile symbolic link in the registry if it doesn't exist yet.
        //   "SYSTEM\CurrentControlSet\Control\Print\Printers" -> "SOFTWARE\Microsoft\Windows NT\CurrentVersion\Print\Printers"
        //
        // According to https://learn.microsoft.com/en-us/archive/msdn-technet-forums/a683ab54-c43c-4ebe-af8f-1f7a65af2a51
        // this is needed when having >900 printers to work around a size limit of the SYSTEM registry hive.
        dwErrorCode = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_C696D38862E6.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK,
            KEY_CREATE_LINK | KEY_SET_VALUE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hKey),
            core::ptr::null_mut(),
        ) as DWORD;
        if dwErrorCode == ERROR_SUCCESS {
            // Note that wszSymbolicLink has to be stored WITHOUT the terminating null character for the symbolic link to work!
            // See cbSymbolicLinkValue above.
            dwErrorCode = RegSetValueExW(
                hKey,
                RUSTCONV_WSTR_0BA09314F12F.as_ptr(),
                0,
                REG_LINK,
                core::ptr::addr_of!(wszSymbolicLinkValue) as *const WCHAR as PBYTE,
                cbSymbolicLinkValue,
            ) as DWORD;
            if dwErrorCode != ERROR_SUCCESS {
                break 'cleanup;
            }
        } else if dwErrorCode != ERROR_ALREADY_EXISTS {
            break 'cleanup;
        }

        // Open some registry keys and leave them open. We need them multiple times throughout the Local Spooler.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_745F20C64BE5.as_ptr(),
            0,
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(hPrintKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_04E7ED89D47D.as_ptr(),
            0,
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(hPrintersKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Construct the path to "%SystemRoot%\system32\spool".
        // Forget about length checks here. If this doesn't fit into MAX_PATH, our OS has more serious problems...
        cchSpoolDirectory = GetSystemDirectoryW(
            core::ptr::addr_of_mut!(wszSpoolDirectory) as *mut WCHAR,
            MAX_PATH,
        ) as DWORD;
        CopyMemory!(
            (core::ptr::addr_of_mut!(wszSpoolDirectory) as *mut WCHAR).add(cchSpoolDirectory as usize)
                as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszSpoolPath) as *const WCHAR as *const core::ffi::c_void,
            ((cchSpoolPath + 1) as usize) * core::mem::size_of::<WCHAR>(),
        );
        cchSpoolDirectory += cchSpoolPath;

        // Query the job directory.
        cbData = core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as DWORD;
        dwErrorCode = RegQueryValueExW(
            hPrintersKey,
            SPLREG_DEFAULT_SPOOL_DIRECTORY,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(wszJobDirectory) as *mut WCHAR as PBYTE,
            core::ptr::addr_of_mut!(cbData),
        ) as DWORD;
        if dwErrorCode == ERROR_SUCCESS {
            cchJobDirectory = (cbData / (core::mem::size_of::<WCHAR>() as DWORD)) - 1;
        } else if dwErrorCode == ERROR_FILE_NOT_FOUND {
            // Use the default "%SystemRoot%\system32\spool\PRINTERS".
            CopyMemory!(
                core::ptr::addr_of_mut!(wszJobDirectory) as *mut WCHAR as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(wszSpoolDirectory) as *mut WCHAR as *const core::ffi::c_void,
                (cchSpoolDirectory as usize) * core::mem::size_of::<WCHAR>(),
            );
            CopyMemory!(
                (core::ptr::addr_of_mut!(wszJobDirectory) as *mut WCHAR).add(cchSpoolDirectory as usize)
                    as *mut core::ffi::c_void,
                core::ptr::addr_of!(wszPrintersPath) as *const WCHAR as *const core::ffi::c_void,
                ((cchPrintersPath + 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            cchJobDirectory = cchSpoolDirectory + cchPrintersPath;

            // Save this for next time.
            RegSetValueExW(
                hPrintersKey,
                SPLREG_DEFAULT_SPOOL_DIRECTORY,
                0,
                REG_SZ,
                core::ptr::addr_of_mut!(wszJobDirectory) as *mut WCHAR as PBYTE,
                ((((cchJobDirectory + 1) as usize) * core::mem::size_of::<WCHAR>())) as DWORD,
            );
        } else {
            break 'cleanup;
        }

        // Initialize all lists.
        if InitializePrintMonitorList() == 0 {
            break 'cleanup;
        }

        if InitializePortList() == 0 {
            break 'cleanup;
        }

        if InitializePrintProcessorList() == 0 {
            break 'cleanup;
        }

        if InitializePrinterList() == 0 {
            break 'cleanup;
        }

        if InitializeGlobalJobList() == 0 {
            break 'cleanup;
        }

        if InitializeFormList() == 0 {
            break 'cleanup;
        }

        if InitializePrinterDrivers() == 0 {
            break 'cleanup;
        }

        // Local Spooler Initialization finished successfully!
        bReturnValue = TRUE as BOOL;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    return bReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            return _InitializeLocalSpooler();
        }
        _ => {
            return TRUE as BOOL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitializePrintProvidor(
    pPrintProvidor: LPPRINTPROVIDOR,
    cbPrintProvidor: DWORD,
    pFullRegistryPath: LPWSTR,
) -> BOOL {
    CopyMemory!(
        pPrintProvidor as *mut core::ffi::c_void,
        core::ptr::addr_of!(_PrintProviderFunctions) as *const core::ffi::c_void,
        min(
            cbPrintProvidor as usize,
            core::mem::size_of::<PRINTPROVIDOR>(),
        ),
    );

    SetLastError(ERROR_SUCCESS);
    return TRUE as BOOL;
}
