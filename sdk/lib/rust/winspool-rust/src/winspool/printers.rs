use crate::winspool::jobs::AddJobW;
use crate::winspool::utils::AllocSplStr;
use crate::winspool::utils::DllAllocSplMem;
use crate::winspool::utils::DllFreeSplMem;
use crate::winspool::jobs::GetJobW;
use crate::winspool::printerdrivers::GetPrinterDriverW;
use crate::winspool::utils::IntProtectHandle;
use crate::winspool::devmode::IsValidDevmodeNoSizeW;
use crate::winspool::marshalling::MarshallUpStructure;
use crate::winspool::marshalling::MarshallUpStructuresArray;
use crate::winspool::devmode::RosConvertAnsiDevModeToUnicodeDevmode;
use crate::winspool::devmode::RosConvertUnicodeDevModeToAnsiDevmode;
use crate::winspool::jobs::ScheduleJob;
use crate::winspool::jobs::SetJobW;
use crate::winspool::printerdata::SetPrinterDataW;
use crate::winspool::utils::UpdateTrayIcon;
use crate::winspool::utils::get_sd;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_C0DCB9330636: [u16; 28] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0050, 0x0072, 0x006F,
    0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x0050, 0x0065, 0x0072,
    0x006D, 0x0069, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut core::ffi::c_void) -> i32;
    fn SetLastError(dwErrorCode: DWORD);
}
// MIDL client stubs (winspool_c.h): `DWORD __cdecl _RpcClosePrinter(WINSPOOL_PRINTER_HANDLE *)`.
extern "C" {
    fn _RpcDeletePrinter(hPrinter: HANDLE) -> DWORD;
    fn _RpcClosePrinter(hPrinter: HANDLE) -> DWORD;
}

// ---- file-scope preamble (translated) ----
pub static mut hinstWinSpool: HINSTANCE = core::ptr::null_mut();

pub type DEVICECAPABILITIES = Option<unsafe extern "system" fn(HANDLE, PWSTR, WORD, PVOID, PDEVMODEW) -> DWORD>;
pub static mut fpDeviceCapabilities: DEVICECAPABILITIES = None;

pub type DEVICEPROPERTYSHEETS = Option<unsafe extern "system" fn(PPROPSHEETUI_INFO, LPARAM) -> LONG>;
pub static mut fpDevicePropertySheets: DEVICEPROPERTYSHEETS = None;

pub type DOCUMENTPROPERTYSHEETS = Option<unsafe extern "system" fn(PPROPSHEETUI_INFO, LPARAM) -> LONG>;
pub static mut fpDocumentPropertySheets: DOCUMENTPROPERTYSHEETS = None;

pub type COMMONPROPERTYSHEETUIW = Option<unsafe extern "system" fn(HWND, PFNPROPSHEETUI, LPARAM, LPDWORD) -> LONG>;
pub static mut fpCommonPropertySheetUIW: COMMONPROPERTYSHEETUIW = None;

pub type QUERYCOLORPROFILE = Option<unsafe extern "system" fn(HANDLE, PDEVMODEW, ULONG, PVOID, *mut ULONG, *mut FLONG) -> LONG>;
pub static mut fpQueryColorProfile: QUERYCOLORPROFILE = None;

pub type SPOOLERPRINTEREVENT = Option<unsafe extern "system" fn(LPWSTR, i32, DWORD, LPARAM) -> BOOL>;
pub static mut fpPrinterEvent: SPOOLERPRINTEREVENT = None;

pub type DEVQUERYPRINT = Option<unsafe extern "system" fn(HANDLE, LPDEVMODEW, *mut DWORD) -> BOOL>;
pub static mut fpDevQueryPrint: DEVQUERYPRINT = None;

pub type DEVQUERYPRINTEX = Option<unsafe extern "system" fn(PDEVQUERYPRINT_INFO) -> BOOL>;
pub static mut fpDevQueryPrintEx: DEVQUERYPRINTEX = None;

pub type CONSTRUCTPRINTERFRIENDLYNAME = Option<unsafe extern "system" fn(PWSTR, PVOID, LPDWORD) -> LONG>;
pub static mut fpConstructPrinterFriendlyName: CONSTRUCTPRINTERFRIENDLYNAME = None;

#[repr(C)]
pub struct _COMPUI_USERDATA {
    pub hModule: HMODULE,
    pub pszPrinterName: LPWSTR,
}
pub type COMPUI_USERDATA = _COMPUI_USERDATA;
pub type PCOMPUI_USERDATA = *mut _COMPUI_USERDATA;

pub static wszWindowsKey: [u16; 53] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x0000,
];
pub static wszDeviceValue: [u16; 7] = [0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];

pub const QSM_DOWNLOADINGFONTS: u32 = 0x0001;

pub const QSM_XPS_PASS: u32 = 0x0002; // Guessing. PRINTER_DRIVER_XPS?

pub static FILE_Port: [u16; 6] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x003A, 0x0000];

const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;
unsafe extern "C" fn _StartDocPrinterSpooled(pHandle: PSPOOLER_HANDLE, pDocInfo1: PDOC_INFO_1W, pAddJobInfo1: PADDJOB_INFO_1W) -> DWORD {
    let mut cbNeeded: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pJobInfo1: PJOB_INFO_1W = core::ptr::null_mut();

    'cleanup: {
        // Create the spool file.
        (*pHandle).hSPLFile = CreateFileW((*pAddJobInfo1).Path, GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, core::ptr::null_mut(), CREATE_ALWAYS, 0, core::ptr::null_mut());
        if (*pHandle).hSPLFile == INVALID_HANDLE_VALUE {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Get the size of the job information.
        GetJobW(pHandle as HANDLE, (*pAddJobInfo1).JobId, 1, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(cbNeeded));
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Allocate enough memory for the returned job information.
        pJobInfo1 = HeapAlloc(hProcessHeap, 0, cbNeeded as usize) as PJOB_INFO_1W;
        if pJobInfo1.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        // Get the job information.
        if GetJobW(pHandle as HANDLE, (*pAddJobInfo1).JobId, 1, pJobInfo1 as PBYTE, cbNeeded, core::ptr::addr_of_mut!(cbNeeded)) == 0 {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Add our document information.
        if !(*pDocInfo1).pDatatype.is_null() {
            (*pJobInfo1).pDatatype = (*pDocInfo1).pDatatype;
        }

        (*pJobInfo1).pDocument = (*pDocInfo1).pDocName;

        // Set the new job information.
        if SetJobW(pHandle as HANDLE, (*pAddJobInfo1).JobId, 1, pJobInfo1 as PBYTE, 0) == 0 {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // We were successful!
        (*pHandle).dwJobID = (*pAddJobInfo1).JobId;
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pJobInfo1.is_null() {
        HeapFree(hProcessHeap, 0, pJobInfo1 as LPVOID);
    }

    dwErrorCode
}

unsafe extern "C" fn _StartDocPrinterWithRPC(pHandle: PSPOOLER_HANDLE, pDocInfo1: PDOC_INFO_1W) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut DocInfoContainer: WINSPOOL_DOC_INFO_CONTAINER = core::mem::zeroed();

    DocInfoContainer.Level = 1;
    DocInfoContainer.DocInfo.pDocInfo1 = pDocInfo1 as *mut WINSPOOL_DOC_INFO_1;

    // RpcTryExcept { ... }
    dwErrorCode = _RpcStartDocPrinter(
        (*pHandle).hPrinter,
        core::ptr::addr_of_mut!(DocInfoContainer),
        core::ptr::addr_of_mut!((*pHandle).dwJobID),
    );
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) { dwErrorCode = RpcExceptionCode(); }
    // The handler runs only if the RPC call above raised an exception; that
    // SEH path is owned by the RPC runtime and has no direct Rust lowering here.
    // RpcEndExcept;

    dwErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn AbortPrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        (*pHandle).set_bStartedDoc(FALSE);
        (*pHandle).set_bTrayIcon(FALSE);

        if (*pHandle).hSPLFile != INVALID_HANDLE_VALUE && (*pHandle).bJob() != 0 {
            // Close any open file handle.
            CloseHandle((*pHandle).hSPLFile);
            (*pHandle).hSPLFile = INVALID_HANDLE_VALUE;

            SetJobW(
                hPrinter,
                (*pHandle).dwJobID,
                0,
                core::ptr::null_mut(),
                JOB_CONTROL_DELETE
            );

            return ScheduleJob(hPrinter, (*pHandle).dwJobID);
        }

        // Do the RPC call.
        // RpcTryExcept / RpcExcept: exception path is never taken on the normal flow.
        let __rpc_exc: bool = false;
        if !__rpc_exc {
            dwErrorCode = _RpcAbortPrinter(core::ptr::addr_of_mut!((*pHandle).hPrinter) as WINSPOOL_PRINTER_HANDLE);
        } else {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterA(pName: PSTR, Level: DWORD, pPrinter: PBYTE) -> HANDLE {
    let mut pNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut usBuffer: _UNICODE_STRING = core::mem::zeroed();
    let mut pwstrNameW: PWSTR = core::ptr::null_mut();
    let ppi2w: *mut PRINTER_INFO_2W = pPrinter as *mut PRINTER_INFO_2W;
    let ppi2a: *mut PRINTER_INFO_2A = pPrinter as *mut PRINTER_INFO_2A;
    let mut ret: HANDLE = core::ptr::null_mut();
    let mut pwszPrinterName: PWSTR = core::ptr::null_mut();
    let mut pwszServerName: PWSTR = core::ptr::null_mut();
    let mut pwszShareName: PWSTR = core::ptr::null_mut();
    let mut pwszPortName: PWSTR = core::ptr::null_mut();
    let mut pwszDriverName: PWSTR = core::ptr::null_mut();
    let mut pwszComment: PWSTR = core::ptr::null_mut();
    let mut pwszLocation: PWSTR = core::ptr::null_mut();
    let mut pwszSepFile: PWSTR = core::ptr::null_mut();
    let mut pwszPrintProcessor: PWSTR = core::ptr::null_mut();
    let mut pwszDatatype: PWSTR = core::ptr::null_mut();
    let mut pwszParameters: PWSTR = core::ptr::null_mut();
    let mut pdmw: PDEVMODEW = core::ptr::null_mut();

    if Level != 2
    {
        SetLastError(ERROR_INVALID_LEVEL);
        return core::ptr::null_mut();
    }

    pwstrNameW = AsciiToUnicode(core::ptr::addr_of_mut!(pNameW), pName);

    'cleanup: {
        if !(*ppi2a).pShareName.is_null()
        {
            pwszShareName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pShareName);
            (*ppi2w).pShareName = pwszShareName;
            if pwszShareName.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pPortName.is_null()
        {
            pwszPortName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPortName);
            (*ppi2w).pPortName = pwszPortName;
            if pwszPortName.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pDriverName.is_null()
        {
            pwszDriverName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pDriverName);
            (*ppi2w).pDriverName = pwszDriverName;
            if pwszDriverName.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pComment.is_null()
        {
            pwszComment = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pComment);
            (*ppi2w).pComment = pwszComment;
            if pwszComment.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pLocation.is_null()
        {
            pwszLocation = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pLocation);
            (*ppi2w).pLocation = pwszLocation;
            if pwszLocation.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pSepFile.is_null()
        {
            pwszSepFile = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pSepFile);
            (*ppi2w).pSepFile = pwszSepFile;
            if pwszSepFile.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pServerName.is_null()
        {
            pwszPrintProcessor = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPrintProcessor);
            (*ppi2w).pPrintProcessor = pwszPrintProcessor;
            if pwszPrintProcessor.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pDatatype.is_null()
        {
            pwszDatatype = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pDatatype);
            (*ppi2w).pDatatype = pwszDatatype;
            if pwszDatatype.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pParameters.is_null()
        {
            pwszParameters = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pParameters);
            (*ppi2w).pParameters = pwszParameters;
            if pwszParameters.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pDevMode.is_null()
        {
            RosConvertAnsiDevModeToUnicodeDevmode((*ppi2a).pDevMode, core::ptr::addr_of_mut!(pdmw));
            (*ppi2w).pDevMode = pdmw;
        }
        if !(*ppi2a).pServerName.is_null()
        {
            pwszServerName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pServerName);
            (*ppi2w).pPrinterName = pwszServerName;
            if pwszServerName.is_null() { break 'cleanup; }
        }
        if !(*ppi2a).pPrinterName.is_null()
        {
            pwszPrinterName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPrinterName);
            (*ppi2w).pPrinterName = pwszPrinterName;
            if pwszPrinterName.is_null() { break 'cleanup; }
        }

        ret = AddPrinterW(pwstrNameW, Level, ppi2w as LPBYTE);
    }

    // Cleanup:
    if !pdmw.is_null() { HeapFree(hProcessHeap, 0, pdmw as *mut core::ffi::c_void); }
    if !pwszPrinterName.is_null() { HeapFree(hProcessHeap, 0, pwszPrinterName as *mut core::ffi::c_void); }
    if !pwszServerName.is_null() { HeapFree(hProcessHeap, 0, pwszServerName as *mut core::ffi::c_void); }
    if !pwszShareName.is_null() { HeapFree(hProcessHeap, 0, pwszShareName as *mut core::ffi::c_void); }
    if !pwszPortName.is_null() { HeapFree(hProcessHeap, 0, pwszPortName as *mut core::ffi::c_void); }
    if !pwszDriverName.is_null() { HeapFree(hProcessHeap, 0, pwszDriverName as *mut core::ffi::c_void); }
    if !pwszComment.is_null() { HeapFree(hProcessHeap, 0, pwszComment as *mut core::ffi::c_void); }
    if !pwszLocation.is_null() { HeapFree(hProcessHeap, 0, pwszLocation as *mut core::ffi::c_void); }
    if !pwszSepFile.is_null() { HeapFree(hProcessHeap, 0, pwszSepFile as *mut core::ffi::c_void); }
    if !pwszPrintProcessor.is_null() { HeapFree(hProcessHeap, 0, pwszPrintProcessor as *mut core::ffi::c_void); }
    if !pwszDatatype.is_null() { HeapFree(hProcessHeap, 0, pwszDatatype as *mut core::ffi::c_void); }
    if !pwszParameters.is_null() { HeapFree(hProcessHeap, 0, pwszParameters as *mut core::ffi::c_void); }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(pNameW));
    ret
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterW(pName: PWSTR, Level: DWORD, pPrinter: PBYTE) -> HANDLE {
    let mut dwErrorCode: DWORD = 0;
    let mut PrinterContainer: WINSPOOL_PRINTER_CONTAINER = core::mem::zeroed();
    let mut DevModeContainer: WINSPOOL_DEVMODE_CONTAINER = core::mem::zeroed();
    let mut SecurityContainer: WINSPOOL_SECURITY_CONTAINER = core::mem::zeroed();
    let mut sd: *mut _SECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut hHandle: HANDLE = core::ptr::null_mut();
    let mut pHandle: PSPOOLER_HANDLE = core::ptr::null_mut();

    DevModeContainer.cbBuf = 0;
    DevModeContainer.pDevMode = core::ptr::null_mut();

    SecurityContainer.cbBuf = 0;
    SecurityContainer.pSecurity = core::ptr::null_mut();

    if Level != 2 {
        SetLastError(ERROR_INVALID_LEVEL);
        return hHandle;
    } else {
        let mut pi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
        if !pi2w.is_null() {
            if !(*pi2w).pDevMode.is_null() {
                if IsValidDevmodeNoSizeW((*pi2w).pDevMode) != 0 {
                    DevModeContainer.cbBuf = ((*(*pi2w).pDevMode).dmSize as DWORD)
                        + ((*(*pi2w).pDevMode).dmDriverExtra as DWORD);
                    DevModeContainer.pDevMode = (*pi2w).pDevMode as PBYTE;
                }
            }

            if !(*pi2w).pSecurityDescriptor.is_null() {
                sd = get_sd(
                    (*pi2w).pSecurityDescriptor as *mut _SECURITY_DESCRIPTOR,
                    core::ptr::addr_of_mut!(size),
                );
                if !sd.is_null() {
                    SecurityContainer.cbBuf = size;
                    SecurityContainer.pSecurity = sd as PBYTE;
                }
            }
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
            return hHandle;
        }
    }

    PrinterContainer.PrinterInfo.pPrinterInfo1 = pPrinter as *mut WINSPOOL_PRINTER_INFO_1;
    PrinterContainer.Level = Level;

    // RpcTryExcept body (SEH frame not representable in Rust; exception path elided)
    {
        dwErrorCode = _RpcAddPrinter(
            pName,
            core::ptr::addr_of_mut!(PrinterContainer),
            core::ptr::addr_of_mut!(DevModeContainer),
            core::ptr::addr_of_mut!(SecurityContainer),
            core::ptr::addr_of_mut!(hPrinter),
        );
    }

    'cleanup: {
        if !hPrinter.is_null() {
            // Create a new SPOOLER_HANDLE structure.
            pHandle = HeapAlloc(
                hProcessHeap,
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<SPOOLER_HANDLE>(),
            ) as PSPOOLER_HANDLE;
            if pHandle.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                // ERR("HeapAlloc failed!\n");
                _RpcDeletePrinter(hPrinter);
                _RpcClosePrinter(hPrinter);
                break 'cleanup;
            }

            (*pHandle).Sig = SPOOLER_HANDLE_SIG as _;
            (*pHandle).hPrinter = hPrinter;
            (*pHandle).hSPLFile = INVALID_HANDLE_VALUE;
            (*pHandle).hSpoolFileHandle = INVALID_HANDLE_VALUE;
            hHandle = pHandle as HANDLE;
        }
    }
    // Cleanup:
    if !sd.is_null() {
        HeapFree(GetProcessHeap(), 0, sd as *mut core::ffi::c_void);
    }
    SetLastError(dwErrorCode);
    return hHandle;
}

#[no_mangle]
pub unsafe extern "system" fn ClosePrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut rpc_exc: bool = false;

    'Cleanup: {
        // Sanity checks.
        if IntProtectHandle(hPrinter, TRUE as BOOL) != 0 {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call.
        // RpcTryExcept body
        dwErrorCode = _RpcClosePrinter(core::ptr::addr_of_mut!((*pHandle).hPrinter) as HANDLE);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER) body
        if rpc_exc {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept

        // Close any open file handle.
        if (*pHandle).hSPLFile != INVALID_HANDLE_VALUE {
            CloseHandle((*pHandle).hSPLFile);
        }

        (*pHandle).Sig = (-1isize) as _;

        // Free the memory for the handle.
        HeapFree(hProcessHeap, 0, pHandle as _);
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call.
        // RpcTryExcept body
        dwErrorCode = _RpcDeletePrinter(core::ptr::addr_of_mut!((*pHandle).hPrinter) as HANDLE);

        // RpcExcept(EXCEPTION_EXECUTE_HANDLER) body: SEH has no Rust equivalent,
        // so the handler is kept as a statically unreachable branch.
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LoadPrinterDriver(hspool: HANDLE) -> HMODULE {
    let mut iTries: INT = 0;
    let mut Size: DWORD = (core::mem::size_of::<u16>() as DWORD * MAX_PATH) * 2; // DRIVER_INFO_5W + plus strings.
    let mut pdi: PDRIVER_INFO_5W = core::ptr::null_mut();
    let mut hLibrary: HMODULE = core::ptr::null_mut();

    loop {
        iTries += 1;

        pdi = RtlAllocateHeap(GetProcessHeap(), 0, Size as SIZE_T) as PDRIVER_INFO_5W;

        if pdi.is_null() {
            break;
        }

        if GetPrinterDriverW(hspool, core::ptr::null_mut(), 5, pdi as LPBYTE, Size, core::ptr::addr_of_mut!(Size)) != 0 {

            // Name and load configure library (for example, C:\DRIVERS\Pscrptui.dll). Not printui.dll!

            hLibrary = LoadLibrary((*pdi).pConfigFile);

            RtlFreeHeap(GetProcessHeap(), 0, pdi as LPVOID);
            return hLibrary;
        }

        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            iTries += 1;
        }

        RtlFreeHeap(GetProcessHeap(), 0, pdi as LPVOID);

        if !(iTries < 2) {
            break;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn DeviceCapabilitiesA(pDevice: LPCSTR, pPort: LPCSTR, fwCapability: WORD, pOutput: LPSTR, pDevMode: *const DEVMODEA) -> DWORD {
    let mut pwszDeviceName: PWSTR = core::ptr::null_mut();
    let mut pdmwInput: PDEVMODEW = core::ptr::null_mut();
    let mut bReturnValue: BOOL = GDI_ERROR as BOOL;
    let mut cch: DWORD = 0;

    'cleanup: {
        if !pDevice.is_null() {
            // Convert pName to a Unicode string pwszDeviceName.
            cch = strlen(pDevice as *const i8) as DWORD;

            pwszDeviceName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize * core::mem::size_of::<u16>())) as PWSTR;
            if pwszDeviceName.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                break 'cleanup;
            }

            MultiByteToWideChar(CP_ACP, 0, pDevice, -(1 as i32), pwszDeviceName, (cch + 1) as i32);
        }

        if !pDevMode.is_null() {
            RosConvertAnsiDevModeToUnicodeDevmode(pDevMode as *mut DEVMODEA, core::ptr::addr_of_mut!(pdmwInput));
        }

        // pPort is ignored so no need to pass it.
        bReturnValue = DeviceCapabilitiesW(pwszDeviceName, core::ptr::null(), fwCapability, pOutput as LPWSTR, pdmwInput as *const DEVMODEW) as BOOL;
    }

    // Cleanup:
    if !pwszDeviceName.is_null() {
        HeapFree(hProcessHeap, 0, pwszDeviceName as *mut core::ffi::c_void);
    }

    if !pdmwInput.is_null() {
        HeapFree(hProcessHeap, 0, pdmwInput as *mut core::ffi::c_void);
    }

    return bReturnValue as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DeviceCapabilitiesW(pDevice: LPCWSTR, pPort: LPCWSTR, fwCapability: WORD, pOutput: LPWSTR, pDevMode: *const DEVMODEW) -> DWORD {
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut hLibrary: HMODULE = core::ptr::null_mut();
    let mut iDevCap: DWORD = GDI_ERROR;

    if !pDevMode.is_null() {
        if IsValidDevmodeNoSizeW(pDevMode as PDEVMODEW) == 0 {
            return (0 as DWORD).wrapping_sub(1 as DWORD);
        }
    }

    if OpenPrinterW(pDevice as LPWSTR, core::ptr::addr_of_mut!(hPrinter), core::ptr::null_mut()) != 0 {
        hLibrary = LoadPrinterDriver(hPrinter);

        if hLibrary as usize != 0 {
            fpDeviceCapabilities = core::mem::transmute(GetProcAddress(hLibrary, b"DrvDeviceCapabilities\0".as_ptr() as *const i8));

            if let Some(fpDevCap) = fpDeviceCapabilities {
                iDevCap = fpDevCap(hPrinter, pDevice as PWSTR, fwCapability, pOutput as PVOID, pDevMode as PDEVMODEW) as _;
            }

            FreeLibrary(hLibrary);
        }

        ClosePrinter(hPrinter);
    }

    return iDevCap;
}

#[no_mangle]
pub unsafe extern "system" fn DevQueryPrint(hPrinter: HANDLE, pDevMode: LPDEVMODEW, pResID: *mut DWORD) -> BOOL {
    let mut hLibrary: HMODULE;
    let mut Ret: BOOL = FALSE as BOOL;

    hLibrary = LoadPrinterDriver(hPrinter);

    if !hLibrary.is_null() {
        fpDevQueryPrint = core::mem::transmute(GetProcAddress(hLibrary, b"DevQueryPrint\0".as_ptr() as *const i8));

        if fpDevQueryPrint.is_some() {
            Ret = (fpDevQueryPrint.unwrap_unchecked())(hPrinter, pDevMode, pResID);
        }

        FreeLibrary(hLibrary);
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DevQueryPrintEx(pDQPInfo: PDEVQUERYPRINT_INFO) -> BOOL {
    let mut hLibrary: HMODULE;
    let mut Ret: BOOL = FALSE as BOOL;

    hLibrary = LoadPrinterDriver((*pDQPInfo).hPrinter);

    if !hLibrary.is_null() {
        fpDevQueryPrintEx = core::mem::transmute(GetProcAddress(hLibrary, b"DevQueryPrintEx\0".as_ptr() as *const i8));

        if fpDevQueryPrintEx.is_some() {
            Ret = (fpDevQueryPrintEx.unwrap_unchecked())(pDQPInfo);
        }

        FreeLibrary(hLibrary);
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DocumentEvent(
    hPrinter: HANDLE,
    hdc: HDC,
    iEsc: i32,
    cbIn: ULONG,
    pvIn: PVOID,
    cbOut: ULONG,
    pvOut: PVOID,
) -> INT {
    // FIXME("DocumentEvent(%p, %p, %lu, %lu, %p, %lu, %p)\n", hPrinter, hdc, iEsc, cbIn, pvIn, cbOut, pvOut);
    // UNIMPLEMENTED;
    DOCUMENTEVENT_UNSUPPORTED as INT
}

#[no_mangle]
pub unsafe extern "system" fn DocumentPropertiesA(hWnd: HWND, hPrinter: HANDLE, pDeviceName: LPSTR, pDevModeOutput: PDEVMODEA, pDevModeInput: PDEVMODEA, fMode: DWORD) -> LONG {
    let mut pwszDeviceName: PWSTR = core::ptr::null_mut();
    let mut pdmwInput: PDEVMODEW = core::ptr::null_mut();
    let mut pdmwOutput: PDEVMODEW = core::ptr::null_mut();
    let mut lReturnValue: LONG = -1;
    let mut cch: DWORD;

    'cleanup: {
        if !pDeviceName.is_null() {
            // Convert pName to a Unicode string pwszDeviceName.
            cch = strlen(pDeviceName) as DWORD;

            pwszDeviceName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * core::mem::size_of::<u16>()) as PWSTR;
            if pwszDeviceName.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                break 'cleanup;
            }

            MultiByteToWideChar(CP_ACP, 0, pDeviceName, -1, pwszDeviceName, (cch + 1) as core::ffi::c_int);
        }

        if !pDevModeInput.is_null() {
            // Create working buffer for input to DocumentPropertiesW.
            RosConvertAnsiDevModeToUnicodeDevmode(pDevModeInput, core::ptr::addr_of_mut!(pdmwInput));
        }

        if !pDevModeOutput.is_null() {
            // Create working buffer for output from DocumentPropertiesW.

            // Do it RIGHT! Get the F...ing Size!
            let Size: LONG = DocumentPropertiesW(hWnd, hPrinter, pwszDeviceName, core::ptr::null_mut(), core::ptr::null_mut(), 0);

            if Size < 0 {
                break 'cleanup;
            }

            pdmwOutput = HeapAlloc(hProcessHeap, 0, Size as usize) as PDEVMODEW;
            if pdmwOutput.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                break 'cleanup;
            }
        }

        lReturnValue = DocumentPropertiesW(hWnd, hPrinter, pwszDeviceName, pdmwOutput, pdmwInput, fMode);

        if lReturnValue < 0 {
            break 'cleanup;
        }

        if !pdmwOutput.is_null() {
            RosConvertUnicodeDevModeToAnsiDevmode(pdmwOutput, pDevModeOutput);
        }
    }

    // Cleanup:
    if !pwszDeviceName.is_null() {
        HeapFree(hProcessHeap, 0, pwszDeviceName as *mut core::ffi::c_void);
    }

    if !pdmwInput.is_null() {
        HeapFree(hProcessHeap, 0, pdmwInput as *mut core::ffi::c_void);
    }

    if !pdmwOutput.is_null() {
        HeapFree(hProcessHeap, 0, pdmwOutput as *mut core::ffi::c_void);
    }

    lReturnValue
}
#[no_mangle]
pub unsafe extern "C" fn get_devmodeW(hprn: HANDLE) -> *mut PRINTER_INFO_9W {
    let mut pi9: *mut PRINTER_INFO_9W = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut res: BOOL;

    res = GetPrinterW(hprn, 9, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(needed));
    if res == 0 && (GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
        pi9 = HeapAlloc(hProcessHeap, 0, needed as usize) as *mut PRINTER_INFO_9W;
        if pi9.is_null() {
            return core::ptr::null_mut();
        }
        res = GetPrinterW(hprn, 9, pi9 as LPBYTE, needed, core::ptr::addr_of_mut!(needed));
    }

    if res != 0 {
        return pi9;
    }

    HeapFree(hProcessHeap, 0, pi9 as *mut core::ffi::c_void);
    return core::ptr::null_mut();
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CreateUIUserData(puserdata: *mut usize, hPrinter: HANDLE) -> i32 {
    let pcui_ud: *mut COMPUI_USERDATA = DllAllocSplMem(core::mem::size_of::<COMPUI_USERDATA>() as DWORD) as *mut COMPUI_USERDATA;

    *puserdata = pcui_ud as usize;

    if !pcui_ud.is_null() {
        (*pcui_ud).hModule = LoadPrinterDriver(hPrinter);

        if (*pcui_ud).hModule.is_null() {
            DllFreeSplMem(pcui_ud as PVOID);
            *puserdata = 0;
        }
    }
    return (*puserdata != 0) as i32;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CreateUIUserData(puserdata: *mut usize, hPrinter: HANDLE) -> i32 {
    let pcui_ud: *mut COMPUI_USERDATA = DllAllocSplMem(core::mem::size_of::<COMPUI_USERDATA>() as DWORD) as *mut COMPUI_USERDATA;

    *puserdata = pcui_ud as usize;

    if !pcui_ud.is_null() {
        (*pcui_ud).hModule = LoadPrinterDriver(hPrinter);

        if (*pcui_ud).hModule.is_null() {
            DllFreeSplMem(pcui_ud as PVOID);
            *puserdata = 0;
        }
    }
    return (*puserdata != 0) as i32;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DestroyUIUserData(puserdata: *mut ULONG_PTR) {
    let pcui_ud: PCOMPUI_USERDATA = *puserdata as PCOMPUI_USERDATA;

    if !pcui_ud.is_null() {
        if !(*pcui_ud).hModule.is_null() {
            FreeLibrary((*pcui_ud).hModule);
            (*pcui_ud).hModule = core::ptr::null_mut();
        }

        if !(*pcui_ud).pszPrinterName.is_null() {
            DllFreeSplMem((*pcui_ud).pszPrinterName as PVOID);
            (*pcui_ud).pszPrinterName = core::ptr::null_mut();
        }

        DllFreeSplMem(pcui_ud as PVOID);
        *puserdata = 0;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DestroyUIUserData(puserdata: *mut ULONG_PTR) {
    let pcui_ud: PCOMPUI_USERDATA = *puserdata as PCOMPUI_USERDATA;

    if !pcui_ud.is_null() {
        if !(*pcui_ud).hModule.is_null() {
            FreeLibrary((*pcui_ud).hModule);
            (*pcui_ud).hModule = core::ptr::null_mut();
        }

        if !(*pcui_ud).pszPrinterName.is_null() {
            DllFreeSplMem((*pcui_ud).pszPrinterName as *mut core::ffi::c_void);
            (*pcui_ud).pszPrinterName = core::ptr::null_mut();
        }

        DllFreeSplMem(pcui_ud as *mut core::ffi::c_void);
        *puserdata = 0;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFixUpDevModeNames(pdphdr: PDOCUMENTPROPERTYHEADER) -> BOOL {
    let mut pi2: *mut PRINTER_INFO_2W = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut res: BOOL;

    if ((*pdphdr).fMode & DM_OUT_BUFFER) == 0
        || ((*pdphdr).fMode & DM_NOPERMISSION) != 0 // Do not allow the user to modify properties on the displayed property sheet pages.
        || (*pdphdr).pdmOut.is_null()
    {
        return FALSE as BOOL;
    }

    res = GetPrinterW(
        (*pdphdr).hPrinter,
        2,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(needed),
    );
    if res == 0 && (GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
        pi2 = HeapAlloc(hProcessHeap, 0, needed as usize) as *mut PRINTER_INFO_2W;
        if pi2.is_null() {
            return FALSE as BOOL;
        }
        res = GetPrinterW(
            (*pdphdr).hPrinter,
            2,
            pi2 as LPBYTE,
            needed,
            core::ptr::addr_of_mut!(needed),
        );
    }

    'exit: {
        if res != 0 {
            /* Check if the provided buffer is large enough */
            let cbDevMode: DWORD = (*(*pi2).pDevMode).dmSize as DWORD + (*(*pi2).pDevMode).dmDriverExtra as DWORD;
            if (*pdphdr).cbOut < cbDevMode {
                res = FALSE as BOOL;
                break 'exit;
            }

            /* Copy the devmode */
            RtlCopyMemory!(
                (*pdphdr).pdmOut as *mut core::ffi::c_void,
                (*pi2).pDevMode as *const core::ffi::c_void,
                cbDevMode as usize,
            );

            StringCchCopyW(
                core::ptr::addr_of_mut!((*(*pdphdr).pdmOut).dmDeviceName) as *mut u16,
                (CCHDEVICENAME - 1) as usize,
                (*pi2).pPrinterName,
            );
            (*(*pdphdr).pdmOut).dmDeviceName[(CCHDEVICENAME - 1) as usize] = 0;
        }
    }

    // Exit:
    HeapFree(hProcessHeap, 0, pi2 as LPVOID);
    return res;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFixUpDevModeNames(pdphdr: PDOCUMENTPROPERTYHEADER) -> BOOL {
    let mut pi2: *mut PRINTER_INFO_2W = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut res: BOOL;

    if ((*pdphdr).fMode & DM_OUT_BUFFER) == 0
        || ((*pdphdr).fMode & DM_NOPERMISSION) != 0 // Do not allow the user to modify properties on the displayed property sheet pages.
        || (*pdphdr).pdmOut.is_null()
    {
        return FALSE as BOOL;
    }

    res = GetPrinterW(
        (*pdphdr).hPrinter,
        2,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(needed),
    );
    if res == 0 && (GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
        pi2 = HeapAlloc(hProcessHeap, 0, needed as usize) as *mut PRINTER_INFO_2W;
        if pi2.is_null() {
            return FALSE as BOOL;
        }
        res = GetPrinterW(
            (*pdphdr).hPrinter,
            2,
            pi2 as LPBYTE,
            needed,
            core::ptr::addr_of_mut!(needed),
        );
    }

    'exit: {
        if res != 0 {
            /* Check if the provided buffer is large enough */
            let cbDevMode: DWORD = (*(*pi2).pDevMode).dmSize as DWORD + (*(*pi2).pDevMode).dmDriverExtra as DWORD;
            if (*pdphdr).cbOut < cbDevMode {
                res = FALSE as BOOL;
                break 'exit;
            }

            /* Copy the devmode */
            RtlCopyMemory!(
                (*pdphdr).pdmOut as *mut core::ffi::c_void,
                (*pi2).pDevMode as *const core::ffi::c_void,
                cbDevMode as usize,
            );

            StringCchCopyW(
                core::ptr::addr_of_mut!((*(*pdphdr).pdmOut).dmDeviceName) as *mut u16,
                (CCHDEVICENAME - 1) as usize,
                (*pi2).pPrinterName,
            );
            (*(*pdphdr).pdmOut).dmDeviceName[(CCHDEVICENAME - 1) as usize] = 0;
        }
    }

    // Exit:
    HeapFree(hProcessHeap, 0, pi2 as LPVOID);
    return res;
}
#[no_mangle]
pub unsafe extern "system" fn CreatePrinterFriendlyName(pcui_ud: PCOMPUI_USERDATA, pszPrinterName: LPWSTR) -> LONG {
    let mut Result: LONG = 0;
    let mut Size: DWORD = 0;
    let mut hLibrary: HMODULE = core::ptr::null_mut();

    hLibrary = LoadLibraryA(b"printui.dll\0".as_ptr() as *const i8);

    if !hLibrary.is_null() {
        fpConstructPrinterFriendlyName = core::mem::transmute(GetProcAddress(hLibrary, b"ConstructPrinterFriendlyName\0".as_ptr() as *const i8));

        if fpConstructPrinterFriendlyName.is_some() {
            if (fpConstructPrinterFriendlyName.unwrap_unchecked())(pszPrinterName, core::ptr::null_mut(), core::ptr::addr_of_mut!(Size)) == 0 {
                if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                    let pwstr: PWSTR = DllAllocSplMem((Size + 1) * (core::mem::size_of::<u16>() as DWORD)) as PWSTR;

                    (*pcui_ud).pszPrinterName = pwstr;

                    if !pwstr.is_null() {
                        Result = (fpConstructPrinterFriendlyName.unwrap_unchecked())(pszPrinterName, pwstr as PVOID, core::ptr::addr_of_mut!(Size));
                    }
                }
            }
        }
        FreeLibrary(hLibrary);
    }

    if Result == 0 {
        DllFreeSplMem((*pcui_ud).pszPrinterName as PVOID);
        (*pcui_ud).pszPrinterName = AllocSplStr(pszPrinterName);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn DocumentPropertySheets(pCPSUIInfo: PPROPSHEETUI_INFO, lparam: LPARAM) -> LONG {
    let mut Result: LONG = -1;
    let mut pdphdr: PDOCUMENTPROPERTYHEADER = core::ptr::null_mut();

    // If pPSUIInfo is NULL, and if either lParam -> fMode is zero or lParam -> pdmOut is NULL,
    // this function should return the size, in bytes, of the printer's DEVMODEW structure.
    if pCPSUIInfo.is_null() && lparam != 0 {
        pdphdr = lparam as PDOCUMENTPROPERTYHEADER;

        if ((*pdphdr).cbSize as usize >= core::mem::size_of::<PDOCUMENTPROPERTYHEADER>())
            && (((*pdphdr).fMode & DM_PROMPT) == 0)
        {
            let hLibrary: HMODULE = LoadPrinterDriver((*pdphdr).hPrinter);

            if !hLibrary.is_null() {
                fpDocumentPropertySheets = core::mem::transmute(GetProcAddress(hLibrary, b"DrvDocumentPropertySheets\0".as_ptr() as *const i8));

                if let Some(fpDocPS) = fpDocumentPropertySheets {
                    Result = fpDocPS(pCPSUIInfo, lparam as _);
                } else {
                    //
                    // ReactOS backup!!! Currently no supporting UI driver.
                    //
                    let pi9: *mut PRINTER_INFO_9W = get_devmodeW((*pdphdr).hPrinter);
                    if !pi9.is_null() {
                        Result = (*(*pi9).pDevMode).dmSize as LONG + (*(*pi9).pDevMode).dmDriverExtra as LONG;

                        HeapFree(hProcessHeap, 0, pi9 as *mut core::ffi::c_void);
                    }
                }

                FreeLibrary(hLibrary);

                if Result > 0 {
                    IntFixUpDevModeNames(pdphdr);
                }

                return Result;
            } else {
                SetLastError(ERROR_INVALID_HANDLE);
            }
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
        return Result;
    }

    Result = 0;

    if !pCPSUIInfo.is_null() {
        let mut psri: PSETRESULT_INFO = core::ptr::null_mut();
        let mut ppsuiihdr: PPROPSHEETUI_INFO_HEADER = core::ptr::null_mut();
        let mut pcui_ud: PCOMPUI_USERDATA = core::ptr::null_mut();
        pdphdr = (*pCPSUIInfo).lParamInit as PDOCUMENTPROPERTYHEADER;

        if ((*pdphdr).cbSize as usize) < core::mem::size_of::<PDOCUMENTPROPERTYHEADER>() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return Result;
        }

        'sw: {
            match (*pCPSUIInfo).Reason as u32 {
                PROPSHEETUI_REASON_INIT => {
                    // (no trace in original)
                    if CreateUIUserData(core::ptr::addr_of_mut!((*pCPSUIInfo).UserData) as *mut usize, (*pdphdr).hPrinter) != 0 {
                        pcui_ud = (*pCPSUIInfo).UserData as PCOMPUI_USERDATA;

                        fpDocumentPropertySheets = core::mem::transmute(GetProcAddress((*pcui_ud).hModule, b"DrvDocumentPropertySheets\0".as_ptr() as *const i8));

                        if fpDocumentPropertySheets.is_some() {
                            ((*pCPSUIInfo).pfnComPropSheet.unwrap_unchecked())(
                                (*pCPSUIInfo).hComPropSheet,
                                CPSFUNC_SET_FUSION_CONTEXT,
                                -3, // What type of handle is this?
                                0,  // Not used, must be zero.
                            );

                            Result = ((*pCPSUIInfo).pfnComPropSheet.unwrap_unchecked())(
                                (*pCPSUIInfo).hComPropSheet,
                                CPSFUNC_ADD_PFNPROPSHEETUIW,
                                core::mem::transmute::<DOCUMENTPROPERTYSHEETS, usize>(fpDocumentPropertySheets) as LPARAM,
                                (*pCPSUIInfo).lParamInit,
                            ) as LONG;
                            break 'sw;
                        }

                        DestroyUIUserData(core::ptr::addr_of_mut!((*pCPSUIInfo).UserData));
                    }
                    break 'sw;
                }

                PROPSHEETUI_REASON_GET_INFO_HEADER => {
                    // FIXME("DocPS : PROPSHEETUI_REASON_GET_INFO_HEADER\n");

                    ppsuiihdr = lparam as PPROPSHEETUI_INFO_HEADER;

                    pcui_ud = (*pCPSUIInfo).UserData as PCOMPUI_USERDATA;

                    CreatePrinterFriendlyName(pcui_ud, (*pdphdr).pszPrinterName);

                    (*ppsuiihdr).Flags = (PSUIHDRF_NOAPPLYNOW | PSUIHDRF_PROPTITLE) as WORD;
                    (*ppsuiihdr).pTitle = (*pcui_ud).pszPrinterName;
                    (*ppsuiihdr).hInst = hinstWinSpool;
                    (*ppsuiihdr).__bindgen_anon_1.IconID = IDI_CPSUI_DOCUMENT as _;

                    Result = CPSUI_OK as LONG;
                    break 'sw;
                }

                PROPSHEETUI_REASON_DESTROY => {
                    // FIXME("DocPS : PROPSHEETUI_REASON_DESTROY\n");
                    DestroyUIUserData(core::ptr::addr_of_mut!((*pCPSUIInfo).UserData));
                    Result = CPSUI_OK as LONG;
                    break 'sw;
                }

                PROPSHEETUI_REASON_SET_RESULT => {
                    // FIXME("DocPS : PROPSHEETUI_REASON_SET_RESULT\n");

                    psri = lparam as PSETRESULT_INFO;

                    (*pCPSUIInfo).Result = (*psri).Result as _;
                    if (*pCPSUIInfo).Result > 0 {
                        IntFixUpDevModeNames(pdphdr);
                    }
                    Result = CPSUI_OK as LONG;
                    break 'sw;
                }

                _ => {}
            }
        }
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn DevicePropertySheets(pCPSUIInfo: PPROPSHEETUI_INFO, lparam: LPARAM) -> LONG {
    let mut Result: LONG = 0;
    let mut pdphdr: PDEVICEPROPERTYHEADER;

    if !pCPSUIInfo.is_null() {
        let mut psri: PSETRESULT_INFO;
        let mut ppsuiihdr: PPROPSHEETUI_INFO_HEADER;
        let mut pcui_ud: PCOMPUI_USERDATA;
        pdphdr = (*pCPSUIInfo).lParamInit as PDEVICEPROPERTYHEADER;

        if ((*pdphdr).cbSize as usize) < core::mem::size_of::<DEVICEPROPERTYHEADER>() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return Result;
        }

        loop {
            match (*pCPSUIInfo).Reason as u32 {
                PROPSHEETUI_REASON_INIT => {
                    {
                        if CreateUIUserData(
                            core::ptr::addr_of_mut!((*pCPSUIInfo).UserData) as *mut usize,
                            (*pdphdr).hPrinter,
                        ) != 0
                        {
                            pcui_ud = (*pCPSUIInfo).UserData as PCOMPUI_USERDATA;

                            let proc_ptr: PVOID = core::mem::transmute(GetProcAddress(
                                (*pcui_ud).hModule,
                                b"DrvDevicePropertySheets\0".as_ptr() as *const i8,
                            ));
                            fpDevicePropertySheets = core::mem::transmute(proc_ptr);

                            if !proc_ptr.is_null() {
                                ((*pCPSUIInfo).pfnComPropSheet.unwrap_unchecked())(
                                    (*pCPSUIInfo).hComPropSheet,
                                    CPSFUNC_SET_FUSION_CONTEXT,
                                    -3, // What type of handle is this?
                                    0, // Not used, must be zero.
                                );

                                Result = ((*pCPSUIInfo).pfnComPropSheet.unwrap_unchecked())(
                                    (*pCPSUIInfo).hComPropSheet,
                                    CPSFUNC_ADD_PFNPROPSHEETUIW,
                                    proc_ptr as usize as LPARAM,
                                    (*pCPSUIInfo).lParamInit,
                                ) as LONG;
                                break;
                            }

                            DestroyUIUserData(core::ptr::addr_of_mut!((*pCPSUIInfo).UserData) as *mut ULONG_PTR);
                        }
                    }
                    break;
                }

                PROPSHEETUI_REASON_GET_INFO_HEADER => {
                    FIXME!(b"DevPS : PROPSHEETUI_REASON_GET_INFO_HEADER\n\0".as_ptr() as *const i8);

                    ppsuiihdr = lparam as PPROPSHEETUI_INFO_HEADER;

                    pcui_ud = (*pCPSUIInfo).UserData as PCOMPUI_USERDATA;

                    CreatePrinterFriendlyName(pcui_ud, (*pdphdr).pszPrinterName);

                    (*ppsuiihdr).Flags = (PSUIHDRF_NOAPPLYNOW | PSUIHDRF_PROPTITLE) as WORD;
                    (*ppsuiihdr).pTitle = (*pcui_ud).pszPrinterName;
                    (*ppsuiihdr).hInst = hinstWinSpool;
                    (*ppsuiihdr).__bindgen_anon_1.IconID = IDI_CPSUI_DOCUMENT as ULONG_PTR;

                    Result = CPSUI_OK as LONG;
                    break;
                }

                PROPSHEETUI_REASON_DESTROY => {
                    FIXME!(b"DevPS : PROPSHEETUI_REASON_DESTROY\n\0".as_ptr() as *const i8);
                    DestroyUIUserData(core::ptr::addr_of_mut!((*pCPSUIInfo).UserData) as *mut ULONG_PTR);
                    Result = CPSUI_OK as LONG;
                    break;
                }

                PROPSHEETUI_REASON_SET_RESULT => {
                    FIXME!(b"DevPS : PROPSHEETUI_REASON_SET_RESULT\n\0".as_ptr() as *const i8);
                    psri = lparam as PSETRESULT_INFO;
                    (*pCPSUIInfo).Result = (*psri).Result as ULONG_PTR;
                    Result = CPSUI_OK as LONG;
                    break;
                }

                _ => {}
            }
            break;
        }
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn CallCommonPropertySheetUI(hWnd: HWND, pfnPropSheetUI: PFNPROPSHEETUI, lparam: LPARAM, pResult: LPDWORD) -> LONG {
    let mut hLibrary: HMODULE = core::ptr::null_mut();
    let mut Ret: LONG = ERR_CPSUI_GETLASTERROR;

    hLibrary = LoadLibraryA(b"compstui.dll\0".as_ptr() as LPCSTR);
    if !hLibrary.is_null() {
        fpCommonPropertySheetUIW = core::mem::transmute(GetProcAddress(hLibrary, b"CommonPropertySheetUIW\0".as_ptr() as LPCSTR));

        if fpCommonPropertySheetUIW.is_some() {
            Ret = (fpCommonPropertySheetUIW.unwrap_unchecked())(hWnd, pfnPropSheetUI, lparam, pResult);
        }

        FreeLibrary(hLibrary);
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DocumentPropertiesW(hWnd: HWND, hPrinter: HANDLE, pDeviceName: LPWSTR, pDevModeOutput: PDEVMODEW, mut pDevModeInput: PDEVMODEW, mut fMode: DWORD) -> LONG {
    unsafe extern "C" fn DocumentPropertySheetsCdecl(pCPSUIInfo: *mut _PROPSHEETUI_INFO, lparam: LPARAM) -> LONG {
        DocumentPropertySheets(pCPSUIInfo, lparam)
    }

    let mut hUseHandle: HANDLE = core::ptr::null_mut();
    let mut docprophdr: DOCUMENTPROPERTYHEADER = core::mem::zeroed();
    let mut Result: LONG = IDOK as LONG;

    if !hPrinter.is_null() {
        hUseHandle = hPrinter;
    } else if OpenPrinterW(pDeviceName, core::ptr::addr_of_mut!(hUseHandle), core::ptr::null_mut()) == 0 {
        return -1;
    }

    if (fMode & DM_IN_BUFFER) == 0
        || (!pDevModeInput.is_null() && IsValidDevmodeNoSizeW(pDevModeInput as PDEVMODEW) == 0)
    {
        pDevModeInput = core::ptr::null_mut();
        fMode &= !DM_IN_BUFFER;
    }

    docprophdr.cbSize = core::mem::size_of::<DOCUMENTPROPERTYHEADER>() as _;
    docprophdr.Reserved = 0;
    docprophdr.hPrinter = hUseHandle;
    docprophdr.pszPrinterName = pDeviceName;
    docprophdr.cbOut = 0;

    if !pDevModeOutput.is_null() {
        docprophdr.pdmIn = core::ptr::null_mut();
        docprophdr.pdmOut = core::ptr::null_mut();
        docprophdr.fMode = 0;
        docprophdr.cbOut = DocumentPropertySheets(core::ptr::null_mut(), core::ptr::addr_of_mut!(docprophdr) as LPARAM) as DWORD;
    }

    docprophdr.pdmIn = pDevModeInput;
    docprophdr.pdmOut = pDevModeOutput;
    docprophdr.fMode = fMode;

    if (fMode & DM_IN_PROMPT) != 0 {
        Result = CPSUI_CANCEL as LONG;

        //
        // Now call the Property Sheet for Print > Properties.
        //
        if CallCommonPropertySheetUI(
            hWnd,
            Some(DocumentPropertySheetsCdecl),
            core::ptr::addr_of_mut!(docprophdr) as LPARAM,
            core::ptr::addr_of_mut!(Result) as LPDWORD,
        ) < 0
        {
            Result = ERR_CPSUI_GETLASTERROR;
        } else {
            Result = if Result == CPSUI_OK as LONG { IDOK as LONG } else { IDCANCEL as LONG };
        }
    } else {
        Result = DocumentPropertySheets(core::ptr::null_mut(), core::ptr::addr_of_mut!(docprophdr) as LPARAM);
    }

    if Result != ERR_CPSUI_GETLASTERROR || Result != ERR_CPSUI_ALLOCMEM_FAILED {
        if !pDevModeOutput.is_null() {
            if IsValidDevmodeNoSizeW(pDevModeOutput) == 0 {
                Result = -1;
            }
        } else {
        }
    }

    if !hUseHandle.is_null() && hPrinter.is_null() {
        ClosePrinter(hUseHandle);
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn PrinterProperties(hWnd: HWND, hPrinter: HANDLE) -> BOOL {
    // PFNPROPSHEETUI is a cdecl pointer (compstui.h: no calling convention) while
    // DevicePropertySheets is WINAPI; the C passes it as-is, which on i386 makes the callee pop
    // arguments the caller also pops. Bridge with a cdecl thunk, as DocumentPropertiesW does.
    unsafe extern "C" fn DevicePropertySheetsCdecl(pCPSUIInfo: *mut _PROPSHEETUI_INFO, lparam: LPARAM) -> LONG {
        DevicePropertySheets(pCPSUIInfo, lparam)
    }
    let mut pi2: *mut PRINTER_INFO_2W = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut Ret: LONG;
    let mut Result: LONG = 0;
    let mut res: BOOL;
    let mut devprophdr: DEVICEPROPERTYHEADER = core::mem::zeroed();

    devprophdr.cbSize = core::mem::size_of::<DEVICEPROPERTYHEADER>() as WORD;
    devprophdr.Flags = DPS_NOPERMISSION as WORD;
    devprophdr.hPrinter = hPrinter;
    devprophdr.pszPrinterName = core::ptr::null_mut();

    res = GetPrinterW(hPrinter, 2, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(needed));
    if res == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        pi2 = HeapAlloc(hProcessHeap, 0, needed as usize) as *mut PRINTER_INFO_2W;
        if pi2.is_null() {
            return FALSE;
        }
        res = GetPrinterW(hPrinter, 2, pi2 as LPBYTE, needed, core::ptr::addr_of_mut!(needed));
    }

    //
    // Above can fail, still process w/o printer name.
    //
    if res != 0 {
        devprophdr.pszPrinterName = (*pi2).pPrinterName;
    }

    needed = 1;

    if SetPrinterDataW(hPrinter, RUSTCONV_WSTR_C0DCB9330636.as_ptr() as PWSTR, REG_DWORD, core::ptr::addr_of_mut!(needed) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD) == ERROR_SUCCESS {
        devprophdr.Flags &= !DPS_NOPERMISSION as WORD;
    }

    Ret = CallCommonPropertySheetUI(hWnd, Some(DevicePropertySheetsCdecl), core::ptr::addr_of_mut!(devprophdr) as LPARAM, core::ptr::addr_of_mut!(Result) as LPDWORD);

    res = (Ret >= 0) as BOOL;

    if res == 0 {
        // (no additional handling in C)
    }

    if !pi2.is_null() {
        HeapFree(hProcessHeap, 0, pi2 as LPVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn EndDocPrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        if (*pHandle).hSPLFile != INVALID_HANDLE_VALUE {
            // For spooled jobs, the document is finished by calling _RpcScheduleJob.
            let mut rpc_raised: bool = false;
            // RpcTryExcept
            dwErrorCode = _RpcScheduleJob((*pHandle).hPrinter, (*pHandle).dwJobID);
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
            if rpc_raised {
                dwErrorCode = RpcExceptionCode!() as DWORD;
            }
            // RpcEndExcept;

            // Close the spool file handle.
            CloseHandle((*pHandle).hSPLFile);
        } else {
            // In all other cases, just call _RpcEndDocPrinter.
            let mut rpc_raised: bool = false;
            // RpcTryExcept
            dwErrorCode = _RpcEndDocPrinter((*pHandle).hPrinter);
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
            if rpc_raised {
                dwErrorCode = RpcExceptionCode!() as DWORD;
            }
            // RpcEndExcept;
        }

        // A new document can now be started again.
        (*pHandle).set_bTrayIcon(FALSE);
        (*pHandle).set_bJob(FALSE);
        (*pHandle).set_bStartedDoc(FALSE);
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EndPagePrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        if (*pHandle).hSPLFile != INVALID_HANDLE_VALUE {
            // For spooled jobs, we don't need to do anything.
            dwErrorCode = ERROR_SUCCESS;
        } else {
            // In all other cases, just call _RpcEndPagePrinter.
            // RpcTryExcept / RpcExcept(EXCEPTION_EXECUTE_HANDLER) block:
            // the RPC call is kept on the normal path; the RpcExceptionCode()
            // handler path is the SEH-only fallback for this call.
            dwErrorCode = _RpcEndPagePrinter((*pHandle).hPrinter);
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintersA(Flags: DWORD, Name: PSTR, Level: DWORD, pPrinterEnum: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut cch: DWORD = 0;
    let mut pwszName: PWSTR = core::ptr::null_mut();
    let mut pszPrinterName: PSTR = core::ptr::null_mut();
    let mut pszServerName: PSTR = core::ptr::null_mut();
    let mut pszDescription: PSTR = core::ptr::null_mut();
    let mut pszName: PSTR = core::ptr::null_mut();
    let mut pszComment: PSTR = core::ptr::null_mut();
    let mut pszShareName: PSTR = core::ptr::null_mut();
    let mut pszPortName: PSTR = core::ptr::null_mut();
    let mut pszDriverName: PSTR = core::ptr::null_mut();
    let mut pszLocation: PSTR = core::ptr::null_mut();
    let mut pszSepFile: PSTR = core::ptr::null_mut();
    let mut pszPrintProcessor: PSTR = core::ptr::null_mut();
    let mut pszDatatype: PSTR = core::ptr::null_mut();
    let mut pszParameters: PSTR = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut ppi1w: PPRINTER_INFO_1W = core::ptr::null_mut();
    let mut ppi1a: PPRINTER_INFO_1A = core::ptr::null_mut();
    let mut ppi2w: PPRINTER_INFO_2W = core::ptr::null_mut();
    let mut ppi2a: PPRINTER_INFO_2A = core::ptr::null_mut();
    let mut ppi4w: PPRINTER_INFO_4W = core::ptr::null_mut();
    let mut ppi4a: PPRINTER_INFO_4A = core::ptr::null_mut();
    let mut ppi5w: PPRINTER_INFO_5W = core::ptr::null_mut();
    let mut ppi5a: PPRINTER_INFO_5A = core::ptr::null_mut();

    'cleanup: {
        // Check for invalid levels here for early error return. MSDN says that only 1, 2, 4, and 5 are allowable.
        if Level != 1 && Level != 2 && Level != 4 && Level != 5
        {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        if !Name.is_null()
        {
            // Convert pName to a Unicode string pwszName.
            cch = strlen(Name) as DWORD;

            pwszName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * core::mem::size_of::<u16>()) as PWSTR;
            if pwszName.is_null()
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'cleanup;
            }

            MultiByteToWideChar(CP_ACP, 0, Name, -1, pwszName, (cch + 1) as i32);
        }

        /* Ref: https://stackoverflow.com/questions/41147180/why-enumprintersa-and-enumprintersw-request-the-same-amount-of-memory */
        if EnumPrintersW(Flags, pwszName, Level, pPrinterEnum, cbBuf, pcbNeeded, pcReturned) == 0
        {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        /* We are mapping multiple different pointers to the same pPrinterEnum pointer here so that */
        /* we can do in-place conversion. We read the Unicode response from the EnumPrintersW and */
        /* then we write back the ANSI conversion into the same buffer for our EnumPrintersA output */

        /* mapping to pPrinterEnum for Unicode (w) characters for Levels 1, 2, 4, and 5 */
        ppi1w = pPrinterEnum as PPRINTER_INFO_1W;
        ppi2w = pPrinterEnum as PPRINTER_INFO_2W;
        ppi4w = pPrinterEnum as PPRINTER_INFO_4W;
        ppi5w = pPrinterEnum as PPRINTER_INFO_5W;
        /* mapping to pPrinterEnum for ANSI (a) characters for Levels 1, 2, 4, and 5 */
        ppi1a = pPrinterEnum as PPRINTER_INFO_1A;
        ppi2a = pPrinterEnum as PPRINTER_INFO_2A;
        ppi4a = pPrinterEnum as PPRINTER_INFO_4A;
        ppi5a = pPrinterEnum as PPRINTER_INFO_5A;

        i = 0;
        while i < *pcReturned
        {
            // switch (Level)
            if Level == 1
            {
                if !(*ppi1w.add(i as usize)).pDescription.is_null()
                {
                    // Convert Unicode pDescription to a ANSI string pszDescription.
                    cch = wcslen((*ppi1w.add(i as usize)).pDescription) as DWORD;

                    pszDescription = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszDescription.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w.add(i as usize)).pDescription, -1, pszDescription, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a.add(i as usize)).pDescription, (cch + 1) as usize, pszDescription);

                    HeapFree(hProcessHeap, 0, pszDescription as *mut core::ffi::c_void);
                }

                if !(*ppi1w.add(i as usize)).pName.is_null()
                {
                    // Convert Unicode pName to a ANSI string pszName.
                    cch = wcslen((*ppi1w.add(i as usize)).pName) as DWORD;

                    pszName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w.add(i as usize)).pName, -1, pszName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a.add(i as usize)).pName, (cch + 1) as usize, pszName);

                    HeapFree(hProcessHeap, 0, pszName as *mut core::ffi::c_void);
                }

                if !(*ppi1w.add(i as usize)).pComment.is_null()
                {
                    // Convert Unicode pComment to a ANSI string pszComment.
                    cch = wcslen((*ppi1w.add(i as usize)).pComment) as DWORD;

                    pszComment = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszComment.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w.add(i as usize)).pComment, -1, pszComment, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a.add(i as usize)).pComment, (cch + 1) as usize, pszComment);

                    HeapFree(hProcessHeap, 0, pszComment as *mut core::ffi::c_void);
                }
            }
            else if Level == 2
            {
                if !(*ppi2w.add(i as usize)).pServerName.is_null()
                {
                    // Convert Unicode pServerName to a ANSI string pszServerName.
                    cch = wcslen((*ppi2w.add(i as usize)).pServerName) as DWORD;

                    pszServerName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszServerName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pServerName, -1, pszServerName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pServerName, (cch + 1) as usize, pszServerName);

                    HeapFree(hProcessHeap, 0, pszServerName as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pPrinterName.is_null()
                {
                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi2w.add(i as usize)).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPrinterName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pShareName.is_null()
                {
                    // Convert Unicode pShareName to a ANSI string pszShareName.
                    cch = wcslen((*ppi2w.add(i as usize)).pShareName) as DWORD;

                    pszShareName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszShareName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pShareName, -1, pszShareName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pShareName, (cch + 1) as usize, pszShareName);

                    HeapFree(hProcessHeap, 0, pszShareName as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pPortName.is_null()
                {
                    // Convert Unicode pPortName to a ANSI string pszPortName.
                    cch = wcslen((*ppi2w.add(i as usize)).pPortName) as DWORD;

                    pszPortName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPortName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pPortName, -1, pszPortName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pPortName, (cch + 1) as usize, pszPortName);

                    HeapFree(hProcessHeap, 0, pszPortName as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pDriverName.is_null()
                {
                    // Convert Unicode pDriverName to a ANSI string pszDriverName.
                    cch = wcslen((*ppi2w.add(i as usize)).pDriverName) as DWORD;

                    pszDriverName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszDriverName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pDriverName, -1, pszDriverName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pDriverName, (cch + 1) as usize, pszDriverName);

                    HeapFree(hProcessHeap, 0, pszDriverName as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pComment.is_null()
                {
                    // Convert Unicode pComment to a ANSI string pszComment.
                    cch = wcslen((*ppi2w.add(i as usize)).pComment) as DWORD;

                    pszComment = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszComment.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pComment, -1, pszComment, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pComment, (cch + 1) as usize, pszComment);

                    HeapFree(hProcessHeap, 0, pszComment as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pLocation.is_null()
                {
                    // Convert Unicode pLocation to a ANSI string pszLocation.
                    cch = wcslen((*ppi2w.add(i as usize)).pLocation) as DWORD;

                    pszLocation = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszLocation.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pLocation, -1, pszLocation, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pLocation, (cch + 1) as usize, pszLocation);

                    HeapFree(hProcessHeap, 0, pszLocation as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pSepFile.is_null()
                {
                    // Convert Unicode pSepFile to a ANSI string pszSepFile.
                    cch = wcslen((*ppi2w.add(i as usize)).pSepFile) as DWORD;

                    pszSepFile = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszSepFile.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pSepFile, -1, pszSepFile, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pSepFile, (cch + 1) as usize, pszSepFile);

                    HeapFree(hProcessHeap, 0, pszSepFile as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pPrintProcessor.is_null()
                {
                    // Convert Unicode pPrintProcessor to a ANSI string pszPrintProcessor.
                    cch = wcslen((*ppi2w.add(i as usize)).pPrintProcessor) as DWORD;

                    pszPrintProcessor = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPrintProcessor.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pPrintProcessor, -1, pszPrintProcessor, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pPrintProcessor, (cch + 1) as usize, pszPrintProcessor);

                    HeapFree(hProcessHeap, 0, pszPrintProcessor as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pDatatype.is_null()
                {
                    // Convert Unicode pDatatype to a ANSI string pszDatatype.
                    cch = wcslen((*ppi2w.add(i as usize)).pDatatype) as DWORD;

                    pszDatatype = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszDatatype.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pDatatype, -1, pszDatatype, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pDatatype, (cch + 1) as usize, pszDatatype);

                    HeapFree(hProcessHeap, 0, pszDatatype as *mut core::ffi::c_void);
                }

                if !(*ppi2w.add(i as usize)).pParameters.is_null()
                {
                    // Convert Unicode pParameters to a ANSI string pszParameters.
                    cch = wcslen((*ppi2w.add(i as usize)).pParameters) as DWORD;

                    pszParameters = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszParameters.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w.add(i as usize)).pParameters, -1, pszParameters, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a.add(i as usize)).pParameters, (cch + 1) as usize, pszParameters);

                    HeapFree(hProcessHeap, 0, pszParameters as *mut core::ffi::c_void);
                }
                if !(*ppi2w.add(i as usize)).pDevMode.is_null()
                {
                    RosConvertUnicodeDevModeToAnsiDevmode((*ppi2w.add(i as usize)).pDevMode, (*ppi2a.add(i as usize)).pDevMode);
                }
            }
            else if Level == 4
            {
                if !(*ppi4w.add(i as usize)).pPrinterName.is_null()
                {
                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi4w.add(i as usize)).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPrinterName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi4w.add(i as usize)).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi4a.add(i as usize)).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi4w.add(i as usize)).pServerName.is_null()
                {
                    // Convert Unicode pServerName to a ANSI string pszServerName.
                    cch = wcslen((*ppi4w.add(i as usize)).pServerName) as DWORD;

                    pszServerName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszServerName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi4w.add(i as usize)).pServerName, -1, pszServerName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi4a.add(i as usize)).pServerName, (cch + 1) as usize, pszServerName);

                    HeapFree(hProcessHeap, 0, pszServerName as *mut core::ffi::c_void);
                }
            }
            else if Level == 5
            {
                if !(*ppi5w.add(i as usize)).pPrinterName.is_null()
                {
                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi5w.add(i as usize)).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPrinterName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi5w.add(i as usize)).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi5a.add(i as usize)).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi5w.add(i as usize)).pPortName.is_null()
                {
                    // Convert Unicode pPortName to a ANSI string pszPortName.
                    cch = wcslen((*ppi5w.add(i as usize)).pPortName) as DWORD;

                    pszPortName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * (core::mem::size_of::<CHAR>() as usize)) as PSTR;
                    if pszPortName.is_null()
                    {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi5w.add(i as usize)).pPortName, -1, pszPortName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi5a.add(i as usize)).pPortName, (cch + 1) as usize, pszPortName);

                    HeapFree(hProcessHeap, 0, pszPortName as *mut core::ffi::c_void);
                }
            }

            i += 1;
        }   // for

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pwszName.is_null()
    {
        HeapFree(hProcessHeap, 0, pwszName as *mut core::ffi::c_void);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintersW(Flags: DWORD, Name: PWSTR, Level: DWORD, pPrinterEnum: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;

    'Cleanup: {
        // Dismiss invalid levels already at this point.
        if Level == 3 || Level > 5 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if cbBuf != 0 && !pPrinterEnum.is_null() {
            ZeroMemory!(pPrinterEnum as *mut core::ffi::c_void, cbBuf as usize);
        }

        // Do the RPC call
        // RpcTryExcept body (SEH is not representable here; the try body runs directly,
        // and the RpcExcept handler, which calls RpcExceptionCode, is only reached on an exception).
        {
            dwErrorCode = _RpcEnumPrinters(Flags, Name, Level, pPrinterEnum, cbBuf, pcbNeeded, pcReturned);
        }

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructuresArray(cbBuf, pPrinterEnum as PVOID, *pcReturned, (*pPrinterInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterInfoMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FlushPrinter(
    hPrinter: HANDLE,
    pBuf: PVOID,
    cbBuf: DWORD,
    pcWritten: PDWORD,
    cSleep: DWORD,
) -> BOOL {
    // TRACE("FlushPrinter(%p, %p, %lu, %p, %lu)\n", hPrinter, pBuf, cbBuf, pcWritten, cSleep);
    // UNIMPLEMENTED;
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetDefaultPrinterA(pszBuffer: LPSTR, pcchBuffer: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pwszBuffer: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        // Sanity check.
        if pcchBuffer.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Check if an ANSI buffer was given and if so, allocate a Unicode buffer of the same size.
        if !pszBuffer.is_null() && *pcchBuffer != 0 {
            pwszBuffer = HeapAlloc(
                hProcessHeap,
                0,
                (*pcchBuffer as usize) * core::mem::size_of::<WCHAR>(),
            ) as PWSTR;
            if pwszBuffer.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }
        }

        if GetDefaultPrinterW(pwszBuffer, pcchBuffer) == 0 {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // We successfully got a string in pwszBuffer, so convert the Unicode string to ANSI.
        WideCharToMultiByte(
            CP_ACP,
            0,
            pwszBuffer,
            -1,
            pszBuffer,
            *pcchBuffer as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pwszBuffer.is_null() {
        HeapFree(hProcessHeap, 0, pwszBuffer as LPVOID);
    }

    SetLastError(dwErrorCode);
    if dwErrorCode == ERROR_SUCCESS { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn GetDefaultPrinterW(pszBuffer: LPWSTR, pcchBuffer: LPDWORD) -> BOOL {
    let mut cbNeeded: DWORD = 0;
    let mut cchInputBuffer: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hWindowsKey: HKEY = core::ptr::null_mut();
    let mut pwszDevice: PWSTR = core::ptr::null_mut();
    let mut pwszComma: PWSTR;

    'Cleanup: {
        // Sanity check.
        if pcchBuffer.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        cchInputBuffer = *pcchBuffer;

        // Open the registry key where the default printer for the current user is stored.
        dwErrorCode = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            core::ptr::addr_of!(wszWindowsKey) as *const u16,
            0,
            KEY_READ,
            &mut hWindowsKey,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Determine the size of the required buffer.
        dwErrorCode = RegQueryValueExW(
            hWindowsKey,
            core::ptr::addr_of!(wszDeviceValue) as *const u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut cbNeeded,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Allocate it.
        pwszDevice = HeapAlloc(hProcessHeap, 0, cbNeeded as usize) as PWSTR;
        if pwszDevice.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        // Now get the actual value.
        dwErrorCode = RegQueryValueExW(
            hWindowsKey,
            core::ptr::addr_of!(wszDeviceValue) as *const u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pwszDevice as PBYTE,
            &mut cbNeeded,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // We get a string "<Printer Name>,winspool,<Port>:".
        // Extract the printer name from it.
        pwszComma = wcschr(pwszDevice, 0x002Cu16) as PWSTR;
        if pwszComma.is_null() {
            dwErrorCode = ERROR_INVALID_NAME;
            break 'Cleanup;
        }

        // Store the length of the Printer Name (including the terminating NUL character!) in *pcchBuffer.
        *pcchBuffer = (pwszComma.offset_from(pwszDevice) as DWORD) + 1;

        // Check if the supplied buffer is large enough.
        if pszBuffer.is_null() || cchInputBuffer < *pcchBuffer {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy the default printer.
        *pwszComma = 0;
        CopyMemory!(
            pszBuffer as *mut _,
            pwszDevice as *const _,
            (*pcchBuffer as usize) * core::mem::size_of::<u16>(),
        );

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !hWindowsKey.is_null() {
        RegCloseKey(hWindowsKey);
    }

    if !pwszDevice.is_null() {
        HeapFree(hProcessHeap, 0, pwszDevice as _);
    }

    SetLastError(dwErrorCode);
    (dwErrorCode == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterA(hPrinter: HANDLE, Level: DWORD, pPrinter: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let ppi1a: PPRINTER_INFO_1A = pPrinter as PPRINTER_INFO_1A;
    let ppi1w: PPRINTER_INFO_1W = pPrinter as PPRINTER_INFO_1W;
    let ppi2a: PPRINTER_INFO_2A = pPrinter as PPRINTER_INFO_2A;
    let ppi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
    let ppi4a: PPRINTER_INFO_4A = pPrinter as PPRINTER_INFO_4A;
    let ppi4w: PPRINTER_INFO_4W = pPrinter as PPRINTER_INFO_4W;
    let ppi5a: PPRINTER_INFO_5A = pPrinter as PPRINTER_INFO_5A;
    let ppi5w: PPRINTER_INFO_5W = pPrinter as PPRINTER_INFO_5W;
    let ppi7a: PPRINTER_INFO_7A = pPrinter as PPRINTER_INFO_7A;
    let ppi7w: PPRINTER_INFO_7W = pPrinter as PPRINTER_INFO_7W;
    let ppi9a: PPRINTER_INFO_9A = pPrinter as PPRINTER_INFO_9A;
    let ppi9w: PPRINTER_INFO_9W = pPrinter as PPRINTER_INFO_9W;
    let mut cch: DWORD = 0;

    'cleanup: {
        // Check for invalid levels here for early error return. Should be 1-9.
        if Level < 1 || Level > 9 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        if GetPrinterW(hPrinter, Level, pPrinter, cbBuf, pcbNeeded) == 0 {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        match Level {
            _ if Level == 1 => {
                if !(*ppi1w).pDescription.is_null() {
                    let pszDescription: PSTR;

                    // Convert Unicode pDescription to a ANSI string pszDescription.
                    cch = wcslen((*ppi1w).pDescription) as DWORD;

                    pszDescription = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszDescription.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w).pDescription, -1, pszDescription, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a).pDescription, (cch + 1) as usize, pszDescription);

                    HeapFree(hProcessHeap, 0, pszDescription as *mut core::ffi::c_void);
                }

                if !(*ppi1w).pName.is_null() {
                    let pszName: PSTR;

                    // Convert Unicode pName to a ANSI string pszName.
                    cch = wcslen((*ppi1w).pName) as DWORD;

                    pszName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w).pName, -1, pszName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a).pName, (cch + 1) as usize, pszName);

                    HeapFree(hProcessHeap, 0, pszName as *mut core::ffi::c_void);
                }

                if !(*ppi1w).pComment.is_null() {
                    let pszComment: PSTR;

                    // Convert Unicode pComment to a ANSI string pszComment.
                    cch = wcslen((*ppi1w).pComment) as DWORD;

                    pszComment = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszComment.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi1w).pComment, -1, pszComment, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi1a).pComment, (cch + 1) as usize, pszComment);

                    HeapFree(hProcessHeap, 0, pszComment as *mut core::ffi::c_void);
                }
            }

            _ if Level == 2 => {
                if !(*ppi2w).pServerName.is_null() {
                    let pszServerName: PSTR;

                    // Convert Unicode pServerName to a ANSI string pszServerName.
                    cch = wcslen((*ppi2w).pServerName) as DWORD;

                    pszServerName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszServerName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pServerName, -1, pszServerName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pServerName, (cch + 1) as usize, pszServerName);

                    HeapFree(hProcessHeap, 0, pszServerName as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pPrinterName.is_null() {
                    let pszPrinterName: PSTR;

                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi2w).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPrinterName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pShareName.is_null() {
                    let pszShareName: PSTR;

                    // Convert Unicode pShareName to a ANSI string pszShareName.
                    cch = wcslen((*ppi2w).pShareName) as DWORD;

                    pszShareName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszShareName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pShareName, -1, pszShareName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pShareName, (cch + 1) as usize, pszShareName);

                    HeapFree(hProcessHeap, 0, pszShareName as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pPortName.is_null() {
                    let pszPortName: PSTR;

                    // Convert Unicode pPortName to a ANSI string pszPortName.
                    cch = wcslen((*ppi2w).pPortName) as DWORD;

                    pszPortName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPortName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pPortName, -1, pszPortName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pPortName, (cch + 1) as usize, pszPortName);

                    HeapFree(hProcessHeap, 0, pszPortName as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pDriverName.is_null() {
                    let pszDriverName: PSTR;

                    // Convert Unicode pDriverName to a ANSI string pszDriverName.
                    cch = wcslen((*ppi2w).pDriverName) as DWORD;

                    pszDriverName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszDriverName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pDriverName, -1, pszDriverName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pDriverName, (cch + 1) as usize, pszDriverName);

                    HeapFree(hProcessHeap, 0, pszDriverName as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pComment.is_null() {
                    let pszComment: PSTR;

                    // Convert Unicode pComment to a ANSI string pszComment.
                    cch = wcslen((*ppi2w).pComment) as DWORD;

                    pszComment = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszComment.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pComment, -1, pszComment, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pComment, (cch + 1) as usize, pszComment);

                    HeapFree(hProcessHeap, 0, pszComment as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pLocation.is_null() {
                    let pszLocation: PSTR;

                    // Convert Unicode pLocation to a ANSI string pszLocation.
                    cch = wcslen((*ppi2w).pLocation) as DWORD;

                    pszLocation = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszLocation.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pLocation, -1, pszLocation, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pLocation, (cch + 1) as usize, pszLocation);

                    HeapFree(hProcessHeap, 0, pszLocation as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pSepFile.is_null() {
                    let pszSepFile: PSTR;

                    // Convert Unicode pSepFile to a ANSI string pszSepFile.
                    cch = wcslen((*ppi2w).pSepFile) as DWORD;

                    pszSepFile = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszSepFile.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pSepFile, -1, pszSepFile, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pSepFile, (cch + 1) as usize, pszSepFile);

                    HeapFree(hProcessHeap, 0, pszSepFile as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pPrintProcessor.is_null() {
                    let pszPrintProcessor: PSTR;

                    // Convert Unicode pPrintProcessor to a ANSI string pszPrintProcessor.
                    cch = wcslen((*ppi2w).pPrintProcessor) as DWORD;

                    pszPrintProcessor = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPrintProcessor.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pPrintProcessor, -1, pszPrintProcessor, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pPrintProcessor, (cch + 1) as usize, pszPrintProcessor);

                    HeapFree(hProcessHeap, 0, pszPrintProcessor as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pDatatype.is_null() {
                    let pszDatatype: PSTR;

                    // Convert Unicode pDatatype to a ANSI string pszDatatype.
                    cch = wcslen((*ppi2w).pDatatype) as DWORD;

                    pszDatatype = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszDatatype.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pDatatype, -1, pszDatatype, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pDatatype, (cch + 1) as usize, pszDatatype);

                    HeapFree(hProcessHeap, 0, pszDatatype as *mut core::ffi::c_void);
                }

                if !(*ppi2w).pParameters.is_null() {
                    let pszParameters: PSTR;

                    // Convert Unicode pParameters to a ANSI string pszParameters.
                    cch = wcslen((*ppi2w).pParameters) as DWORD;

                    pszParameters = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszParameters.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi2w).pParameters, -1, pszParameters, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi2a).pParameters, (cch + 1) as usize, pszParameters);

                    HeapFree(hProcessHeap, 0, pszParameters as *mut core::ffi::c_void);
                }
                if !(*ppi2w).pDevMode.is_null() {
                    RosConvertUnicodeDevModeToAnsiDevmode((*ppi2w).pDevMode, (*ppi2a).pDevMode);
                }
            }

            _ if Level == 4 => {
                if !(*ppi4w).pPrinterName.is_null() {
                    let pszPrinterName: PSTR;

                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi4w).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPrinterName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi4w).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi4a).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi4w).pServerName.is_null() {
                    let pszServerName: PSTR;

                    // Convert Unicode pServerName to a ANSI string pszServerName.
                    cch = wcslen((*ppi4w).pServerName) as DWORD;

                    pszServerName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszServerName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi4w).pServerName, -1, pszServerName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi4a).pServerName, (cch + 1) as usize, pszServerName);

                    HeapFree(hProcessHeap, 0, pszServerName as *mut core::ffi::c_void);
                }
            }

            _ if Level == 5 => {
                if !(*ppi5w).pPrinterName.is_null() {
                    let pszPrinterName: PSTR;

                    // Convert Unicode pPrinterName to a ANSI string pszPrinterName.
                    cch = wcslen((*ppi5w).pPrinterName) as DWORD;

                    pszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPrinterName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi5w).pPrinterName, -1, pszPrinterName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi5a).pPrinterName, (cch + 1) as usize, pszPrinterName);

                    HeapFree(hProcessHeap, 0, pszPrinterName as *mut core::ffi::c_void);
                }

                if !(*ppi5w).pPortName.is_null() {
                    let pszPortName: PSTR;

                    // Convert Unicode pPortName to a ANSI string pszPortName.
                    cch = wcslen((*ppi5w).pPortName) as DWORD;

                    pszPortName = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszPortName.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi5w).pPortName, -1, pszPortName, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi5a).pPortName, (cch + 1) as usize, pszPortName);

                    HeapFree(hProcessHeap, 0, pszPortName as *mut core::ffi::c_void);
                }
            }

            _ if Level == 7 => {
                if !(*ppi7w).pszObjectGUID.is_null() {
                    let pszaObjectGUID: PSTR;

                    // Convert Unicode pszObjectGUID to a ANSI string pszaObjectGUID.
                    cch = wcslen((*ppi7w).pszObjectGUID) as DWORD;

                    pszaObjectGUID = HeapAlloc(hProcessHeap, 0, ((cch as usize) + 1) * core::mem::size_of::<CHAR>()) as PSTR;
                    if pszaObjectGUID.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    WideCharToMultiByte(CP_ACP, 0, (*ppi7w).pszObjectGUID, -1, pszaObjectGUID, (cch + 1) as i32, core::ptr::null_mut(), core::ptr::null_mut());
                    StringCchCopyA((*ppi7a).pszObjectGUID, (cch + 1) as usize, pszaObjectGUID);

                    HeapFree(hProcessHeap, 0, pszaObjectGUID as *mut core::ffi::c_void);
                }
            }

            _ if Level == 8 || Level == 9 => {
                RosConvertUnicodeDevModeToAnsiDevmode((*ppi9w).pDevMode, (*ppi9a).pDevMode);
            }

            _ => {}
        } // switch

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterW(hPrinter: HANDLE, Level: DWORD, pPrinter: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Dismiss invalid levels already at this point.
        if Level > 9 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if cbBuf != 0 && !pPrinter.is_null() {
            ZeroMemory!(pPrinter, cbBuf as usize);
        }

        // Do the RPC call
        // RpcTryExcept
        dwErrorCode = _RpcGetPrinter((*pHandle).hPrinter, Level, pPrinter, cbBuf, pcbNeeded);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): structured exception handling has no
        // Rust equivalent; the handler is kept as a branch that is never taken.
        let _rpc_exception_raised: bool = false;
        if _rpc_exception_raised {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructure(
                cbBuf,
                pPrinter as PVOID,
                (*pPrinterInfoMarshalling[Level as usize]).pInfo.as_ptr(),
                (*pPrinterInfoMarshalling[Level as usize]).cbStructureSize,
                TRUE as BOOL
            );
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OpenPrinterA(pPrinterName: LPSTR, phPrinter: LPHANDLE, pDefault: LPPRINTER_DEFAULTSA) -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cch: DWORD = 0;
    let mut pwszPrinterName: PWSTR = core::ptr::null_mut();
    let mut wDefault: PRINTER_DEFAULTSW = core::mem::zeroed();

    'Cleanup: {
        if !pPrinterName.is_null() {
            // Convert pPrinterName to a Unicode string pwszPrinterName
            cch = strlen(pPrinterName) as DWORD;

            pwszPrinterName = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * core::mem::size_of::<u16>()) as PWSTR;
            if pwszPrinterName.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                break 'Cleanup;
            }

            MultiByteToWideChar(CP_ACP, 0, pPrinterName, -1, pwszPrinterName, (cch + 1) as i32);
        }

        if !pDefault.is_null() {
            wDefault.DesiredAccess = (*pDefault).DesiredAccess;

            if !(*pDefault).pDatatype.is_null() {
                // Convert pDefault->pDatatype to a Unicode string wDefault.pDatatype
                cch = strlen((*pDefault).pDatatype) as DWORD;

                wDefault.pDatatype = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * core::mem::size_of::<u16>()) as PWSTR;
                if wDefault.pDatatype.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                    break 'Cleanup;
                }

                MultiByteToWideChar(CP_ACP, 0, (*pDefault).pDatatype, -1, wDefault.pDatatype, (cch + 1) as i32);
            }

            if !(*pDefault).pDevMode.is_null() {
                wDefault.pDevMode = GdiConvertToDevmodeW((*pDefault).pDevMode);
            }
        }

        bReturnValue = OpenPrinterW(pwszPrinterName, phPrinter, &mut wDefault);

        if bReturnValue != 0 {
            let pHandle: PSPOOLER_HANDLE = (*phPrinter) as PSPOOLER_HANDLE;
            (*pHandle).set_bAnsi(TRUE);
        }
    }

    // Cleanup:
    if !wDefault.pDatatype.is_null() {
        HeapFree(hProcessHeap, 0, wDefault.pDatatype as LPVOID);
    }

    if !wDefault.pDevMode.is_null() {
        HeapFree(hProcessHeap, 0, wDefault.pDevMode as LPVOID);
    }

    if !pwszPrinterName.is_null() {
        HeapFree(hProcessHeap, 0, pwszPrinterName as LPVOID);
    }

    return bReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn OpenPrinterW(pPrinterName: LPWSTR, phPrinter: LPHANDLE, pDefault: LPPRINTER_DEFAULTSW) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut pHandle: PSPOOLER_HANDLE = core::ptr::null_mut();
    let mut pDatatype: PWSTR = core::ptr::null_mut();
    let mut DevModeContainer: WINSPOOL_DEVMODE_CONTAINER = core::mem::zeroed();
    let mut AccessRequired: ACCESS_MASK = 0;

    // Sanity check
    'Cleanup: {
        if phPrinter.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Prepare the additional parameters in the format required by _RpcOpenPrinter
        if !pDefault.is_null() {
            pDatatype = (*pDefault).pDatatype;
            DevModeContainer.cbBuf = core::mem::size_of::<DEVMODEW>() as DWORD;
            DevModeContainer.pDevMode = (*pDefault).pDevMode as *mut BYTE;
            AccessRequired = (*pDefault).DesiredAccess;
        }

        // Do the RPC call
        dwErrorCode = _RpcOpenPrinter(
            pPrinterName,
            core::ptr::addr_of_mut!(hPrinter),
            pDatatype,
            core::ptr::addr_of_mut!(DevModeContainer),
            AccessRequired,
        );

        if dwErrorCode == ERROR_SUCCESS {
            // Create a new SPOOLER_HANDLE structure.
            pHandle = HeapAlloc(hProcessHeap, HEAP_ZERO_MEMORY, core::mem::size_of::<SPOOLER_HANDLE>()) as PSPOOLER_HANDLE;
            if pHandle.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            (*pHandle).Sig = SPOOLER_HANDLE_SIG as _;
            (*pHandle).hPrinter = hPrinter;
            (*pHandle).hSPLFile = INVALID_HANDLE_VALUE;
            (*pHandle).hSpoolFileHandle = INVALID_HANDLE_VALUE;

            // Return it as phPrinter.
            *phPrinter = pHandle as HANDLE;
        }
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PrinterMessageBoxA(
    hPrinter: HANDLE,
    Error: DWORD,
    hWnd: HWND,
    pText: LPSTR,
    pCaption: LPSTR,
    dwType: DWORD,
) -> DWORD {
    50
}

#[no_mangle]
pub unsafe extern "system" fn PrinterMessageBoxW(
    hPrinter: HANDLE,
    Error: DWORD,
    hWnd: HWND,
    pText: LPWSTR,
    pCaption: LPWSTR,
    dwType: DWORD,
) -> DWORD {
    50
}

#[no_mangle]
pub unsafe extern "system" fn QueryColorProfile(
    hPrinter: HANDLE,
    pdevmode: PDEVMODEW,
    ulQueryMode: ULONG,
    pvProfileData: *mut core::ffi::c_void,
    pcbProfileData: *mut ULONG,
    pflProfileData: *mut FLONG,
) -> BOOL {
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut Ret: BOOL = FALSE as BOOL;
    let mut hLibrary: HMODULE;

    if (*pHandle).bNoColorProfile() != FALSE as BOOL {
        Ret = SP_ERROR;
    } else {
        if !pdevmode.is_null() {
            if IsValidDevmodeNoSizeW(pdevmode) == 0 {
                return FALSE as BOOL;
            }
        }

        hLibrary = LoadPrinterDriver(hPrinter);

        if !hLibrary.is_null() {
            let fp = GetProcAddress(hLibrary, b"DrvQueryColorProfile\0".as_ptr() as *const i8);
            fpQueryColorProfile = core::ptr::read(core::ptr::addr_of!(fp) as *const _);

            if fpQueryColorProfile.is_some() {
                Ret = (fpQueryColorProfile.unwrap_unchecked())(hPrinter, pdevmode, ulQueryMode, pvProfileData, pcbProfileData, pflProfileData);
            } else {
                (*pHandle).set_bNoColorProfile(TRUE as BOOL);
                Ret = SP_ERROR;
            }

            FreeLibrary(hLibrary);
        }
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn QuerySpoolMode(hPrinter: HANDLE, downloadFontsFlags: PDWORD, dwVersion: PDWORD) -> BOOL {
    let mut pi2: *mut PRINTER_INFO_2W = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut res: BOOL;

    res = GetPrinterW(hPrinter, 2, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(needed));
    if res == 0 && (GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
        pi2 = HeapAlloc(hProcessHeap, 0, needed as usize) as *mut PRINTER_INFO_2W;
        if pi2.is_null() {
            return FALSE as BOOL;
        }
        res = GetPrinterW(hPrinter, 2, pi2 as LPBYTE, needed, core::ptr::addr_of_mut!(needed));
    }

    if res != 0 {
        *dwVersion = 0x10000;
        *downloadFontsFlags = 0;

        if !(*pi2).pServerName.is_null() {
            *downloadFontsFlags |= QSM_DOWNLOADINGFONTS;
        }
    }

    HeapFree(hProcessHeap, 0, pi2 as LPVOID);
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn QueryRemoteFonts(hPrinter: HANDLE, pufi: PUNIVERSAL_FONT_ID, NumberOfUFIs: ULONG) -> DWORD {
    let mut hIC: HANDLE;
    let mut Result: DWORD = (0 as DWORD).wrapping_sub(1);
    let mut cOut: DWORD;
    let mut cIn: DWORD = 0;
    let mut pOut: PBYTE;

    hIC = CreatePrinterIC(hPrinter, core::ptr::null_mut());
    if !hIC.is_null() {
        cOut = (NumberOfUFIs * core::mem::size_of::<UNIVERSAL_FONT_ID>() as DWORD) + core::mem::size_of::<DWORD>() as DWORD; // Include "DWORD" first part to return size.

        pOut = HeapAlloc(hProcessHeap, 0, cOut as usize) as PBYTE;
        if !pOut.is_null() {
            if PlayGdiScriptOnPrinterIC(hIC, addr_of_mut!(cIn) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD, pOut, cOut, 0) != 0 {
                cIn = *(pOut as PDWORD); // Fisrt part is the size of the UFID object.

                Result = cIn; // Return the required size.

                if NumberOfUFIs < cIn {
                    cIn = NumberOfUFIs;
                }
                //     Copy whole object back to GDI32, exclude first DWORD part.
                memcpy(pufi as *mut core::ffi::c_void, pOut.add(core::mem::size_of::<DWORD>()) as *const core::ffi::c_void, (cIn * core::mem::size_of::<UNIVERSAL_FONT_ID>() as DWORD) as usize);
            }
            HeapFree(hProcessHeap, 0, pOut as *mut core::ffi::c_void);
        }
        DeletePrinterIC(hIC);
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ReadPrinter(hPrinter: HANDLE, pBuf: PVOID, cbBuf: DWORD, pNoBytesRead: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call (RpcTryExcept)
        dwErrorCode = _RpcReadPrinter((*pHandle).hPrinter, pBuf as *mut BYTE, cbBuf, pNoBytesRead);

        // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler: SEH is not catchable in Rust,
        // so this handler body is kept but never entered.
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ResetPrinterA(hPrinter: HANDLE, pDefault: PPRINTER_DEFAULTSA) -> BOOL {
    let mut pNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut pdmw: PDEVMODEW = core::ptr::null_mut();
    let pdw: PPRINTER_DEFAULTSW = pDefault as PPRINTER_DEFAULTSW;
    let ret: BOOL;

    if (*pDefault).pDatatype == ((-1) as isize) as LPSTR {
        (*pdw).pDatatype = ((-1) as isize) as LPWSTR;
    } else {
        (*pdw).pDatatype = AsciiToUnicode(core::ptr::addr_of_mut!(pNameW), (*pDefault).pDatatype);
    }
    if (*pDefault).pDevMode == ((-1) as isize) as LPDEVMODEA {
        (*pdw).pDevMode = ((-1) as isize) as LPDEVMODEW;
    } else {
        if !(*pDefault).pDevMode.is_null() {
            RosConvertAnsiDevModeToUnicodeDevmode((*pDefault).pDevMode, core::ptr::addr_of_mut!(pdmw));
            (*pdw).pDevMode = pdmw;
        }
    }

    ret = ResetPrinterW(hPrinter, pdw);

    if !pdmw.is_null() {
        HeapFree(hProcessHeap, 0, pdmw as LPVOID);
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(pNameW));

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ResetPrinterW(
    hPrinter: HANDLE,
    pDefault: PPRINTER_DEFAULTSW,
) -> BOOL {
    // TRACE("ResetPrinterW(%p, %p)\n", hPrinter, pDefault);
    // UNIMPLEMENTED;
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SeekPrinter(hPrinter: HANDLE, liDistanceToMove: LARGE_INTEGER, pliNewPointer: PLARGE_INTEGER, dwMoveMethod: DWORD, bWrite: BOOL) -> BOOL {
    let mut dwErrorCode: DWORD;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE;
        // goto Cleanup;
    } else {
        // RpcTryExcept
        // Structured exception handling has no Rust equivalent; the try body runs unguarded.
        let rpc_raised: BOOL = 0;
        dwErrorCode = _RpcSeekPrinter((*pHandle).hPrinter, liDistanceToMove, pliNewPointer, dwMoveMethod, bWrite);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        if rpc_raised != 0 {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetDefaultPrinterA(pszPrinter: LPCSTR) -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cch: DWORD;
    let mut pwszPrinter: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        if !pszPrinter.is_null() {
            // Convert pszPrinter to a Unicode string pwszPrinter
            cch = strlen(pszPrinter as *const i8) as DWORD;

            pwszPrinter = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as usize) * core::mem::size_of::<u16>(),
            ) as PWSTR;
            if pwszPrinter.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);

                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pszPrinter as *const i8,
                -(1 as i32),
                pwszPrinter,
                (cch + 1) as i32,
            );
        }

        bReturnValue = SetDefaultPrinterW(pwszPrinter);
    }

    // Cleanup:
    if !pwszPrinter.is_null() {
        HeapFree(hProcessHeap, 0, pwszPrinter as LPVOID);
    }

    return bReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn SetDefaultPrinterW(mut pszPrinter: LPCWSTR) -> BOOL {
    let wszDevicesKey: [u16; 53] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0073, 0x0000,
];

    let mut cbDeviceValueData: DWORD = 0;
    let mut cbPrinterValueData: DWORD = 0;
    let mut cchPrinter: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hDevicesKey: HKEY = core::ptr::null_mut();
    let mut hWindowsKey: HKEY = core::ptr::null_mut();
    let mut pwszDeviceValueData: PWSTR = core::ptr::null_mut();
    let mut wszPrinter: [u16; MAX_PRINTER_NAME as usize + 1] = [0u16; MAX_PRINTER_NAME as usize + 1];

    'cleanup: {
        // Open the Devices registry key.
        dwErrorCode = RegOpenKeyExW(HKEY_CURRENT_USER, wszDevicesKey.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hDevicesKey)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Did the caller give us a printer to set as default?
        if !pszPrinter.is_null() && *pszPrinter != 0 {
            // Check if the given printer exists and query the value data size.
            dwErrorCode = RegQueryValueExW(hDevicesKey, pszPrinter, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(cbPrinterValueData)) as DWORD;
            if dwErrorCode == ERROR_FILE_NOT_FOUND {
                dwErrorCode = ERROR_INVALID_PRINTER_NAME;
                break 'cleanup;
            } else if dwErrorCode != ERROR_SUCCESS {
                break 'cleanup;
            }

            cchPrinter = wcslen(pszPrinter) as DWORD;
        } else {
            // If there is already a default printer, we're done!
            cchPrinter = _countof!(wszPrinter) as DWORD;
            if GetDefaultPrinterW(wszPrinter.as_mut_ptr(), core::ptr::addr_of_mut!(cchPrinter)) != 0 {
                dwErrorCode = ERROR_SUCCESS;
                break 'cleanup;
            }

            // Otherwise, get us the first printer from the "Devices" key to later set it as default and query the value data size.
            cchPrinter = _countof!(wszPrinter) as DWORD;
            dwErrorCode = RegEnumValueW(hDevicesKey, 0, wszPrinter.as_mut_ptr(), core::ptr::addr_of_mut!(cchPrinter), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(cbPrinterValueData)) as DWORD;
            if dwErrorCode != ERROR_MORE_DATA {
                break 'cleanup;
            }

            pszPrinter = wszPrinter.as_ptr();
        }

        // We now need to query the value data, which has the format "winspool,<Port>:"
        // and make "<Printer Name>,winspool,<Port>:" out of it.
        // Allocate a buffer large enough for the final data.
        cbDeviceValueData = (cchPrinter + 1) * (core::mem::size_of::<u16>() as DWORD) + cbPrinterValueData;
        pwszDeviceValueData = HeapAlloc(hProcessHeap, 0, cbDeviceValueData as usize) as PWSTR;
        if pwszDeviceValueData.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        // Copy the Printer Name and a comma into it.
        CopyMemory!(pwszDeviceValueData as *mut core::ffi::c_void, pszPrinter as *const core::ffi::c_void, cchPrinter as usize * core::mem::size_of::<u16>());
        *pwszDeviceValueData.add(cchPrinter as usize) = 0x002Cu16;

        // Append the value data, which has the format "winspool,<Port>:"
        dwErrorCode = RegQueryValueExW(hDevicesKey, pszPrinter, core::ptr::null_mut(), core::ptr::null_mut(), pwszDeviceValueData.add((cchPrinter + 1) as usize) as PBYTE, core::ptr::addr_of_mut!(cbPrinterValueData)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Open the Windows registry key.
        dwErrorCode = RegOpenKeyExW(HKEY_CURRENT_USER, wszWindowsKey.as_ptr(), 0, KEY_SET_VALUE, core::ptr::addr_of_mut!(hWindowsKey)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Store our new default printer.
        dwErrorCode = RegSetValueExW(hWindowsKey, wszDeviceValue.as_ptr(), 0, REG_SZ, pwszDeviceValueData as PBYTE, cbDeviceValueData) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }
    }

    // Cleanup:
    if !hDevicesKey.is_null() {
        RegCloseKey(hDevicesKey);
    }

    if !hWindowsKey.is_null() {
        RegCloseKey(hWindowsKey);
    }

    if !pwszDeviceValueData.is_null() {
        HeapFree(hProcessHeap, 0, pwszDeviceValueData as *mut core::ffi::c_void);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterA(hPrinter: HANDLE, Level: DWORD, mut pPrinter: PBYTE, Command: DWORD) -> BOOL {
    let mut Ret: BOOL = FALSE as BOOL;
    let mut usBuffer: _UNICODE_STRING = core::mem::zeroed();
    let ppisa: PPRINTER_INFO_STRESS = pPrinter as PPRINTER_INFO_STRESS;
    let ppisw: PPRINTER_INFO_STRESS = pPrinter as PPRINTER_INFO_STRESS;
    let ppi2a: PPRINTER_INFO_2A = pPrinter as PPRINTER_INFO_2A;
    let ppi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
    let ppi7a: PPRINTER_INFO_7A = pPrinter as PPRINTER_INFO_7A;
    let ppi7w: PPRINTER_INFO_7W = pPrinter as PPRINTER_INFO_7W;
    let ppi9a: PPRINTER_INFO_9A = pPrinter as PPRINTER_INFO_9A;
    let ppi9w: PPRINTER_INFO_9W = pPrinter as PPRINTER_INFO_9W;
    let mut pwszPrinterName: PWSTR = core::ptr::null_mut();
    let mut pwszServerName: PWSTR = core::ptr::null_mut();
    let mut pwszShareName: PWSTR = core::ptr::null_mut();
    let mut pwszPortName: PWSTR = core::ptr::null_mut();
    let mut pwszDriverName: PWSTR = core::ptr::null_mut();
    let mut pwszComment: PWSTR = core::ptr::null_mut();
    let mut pwszLocation: PWSTR = core::ptr::null_mut();
    let mut pwszSepFile: PWSTR = core::ptr::null_mut();
    let mut pwszPrintProcessor: PWSTR = core::ptr::null_mut();
    let mut pwszDatatype: PWSTR = core::ptr::null_mut();
    let mut pwszParameters: PWSTR = core::ptr::null_mut();
    let mut pdmw: PDEVMODEW = core::ptr::null_mut();

    let mut Level = Level;
    let mut Command = Command;

    'cleanup: {
        'sw: {
            match Level {
                0 => {
                    if Command == 0 {
                        if !(*ppisa).pPrinterName.is_null() {
                            pwszPrinterName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppisa).pPrinterName as LPCSTR);
                            (*ppisw).pPrinterName = pwszPrinterName;
                            if pwszPrinterName.is_null() { break 'cleanup; }
                        }
                        if !(*ppisa).pServerName.is_null() {
                            pwszServerName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppisa).pServerName as LPCSTR);
                            (*ppisw).pPrinterName = pwszServerName;
                            if pwszServerName.is_null() { break 'cleanup; }
                        }
                    }
                    if Command == PRINTER_CONTROL_SET_STATUS {
                        // Set the pPrinter parameter to a pointer to a DWORD value that specifies the new printer status.
                        let mut pi6: PRINTER_INFO_6 = core::mem::zeroed();
                        pi6.dwStatus = pPrinter as DWORD;
                        pPrinter = core::ptr::addr_of_mut!(pi6) as LPBYTE;
                        Level = 6;
                        Command = 0;
                    }
                    break 'sw;
                }
                2 => {
                    {
                        if !(*ppi2a).pShareName.is_null() {
                            pwszShareName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pShareName);
                            (*ppi2w).pShareName = pwszShareName;
                            if pwszShareName.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pPortName.is_null() {
                            pwszPortName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPortName);
                            (*ppi2w).pPortName = pwszPortName;
                            if pwszPortName.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pDriverName.is_null() {
                            pwszDriverName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pDriverName);
                            (*ppi2w).pDriverName = pwszDriverName;
                            if pwszDriverName.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pComment.is_null() {
                            pwszComment = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pComment);
                            (*ppi2w).pComment = pwszComment;
                            if pwszComment.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pLocation.is_null() {
                            pwszLocation = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pLocation);
                            (*ppi2w).pLocation = pwszLocation;
                            if pwszLocation.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pSepFile.is_null() {
                            pwszSepFile = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pSepFile);
                            (*ppi2w).pSepFile = pwszSepFile;
                            if pwszSepFile.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pServerName.is_null() {
                            pwszPrintProcessor = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPrintProcessor);
                            (*ppi2w).pPrintProcessor = pwszPrintProcessor;
                            if pwszPrintProcessor.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pDatatype.is_null() {
                            pwszDatatype = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pDatatype);
                            (*ppi2w).pDatatype = pwszDatatype;
                            if pwszDatatype.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pParameters.is_null() {
                            pwszParameters = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pParameters);
                            (*ppi2w).pParameters = pwszParameters;
                            if pwszParameters.is_null() { break 'cleanup; }
                        }

                        if !(*ppi2a).pDevMode.is_null() {
                            RosConvertAnsiDevModeToUnicodeDevmode((*ppi2a).pDevMode, core::ptr::addr_of_mut!(pdmw));
                            (*ppi2w).pDevMode = pdmw;
                        }
                    }
                    // These two strings are relitive and common to these three Levels.
                    // Fall through (duplicated here for the 0007/0008 body).
                    {
                        if !(*ppi2a).pServerName.is_null() {
                            pwszServerName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pServerName);
                            (*ppi2w).pPrinterName = pwszServerName;
                            if pwszServerName.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pPrinterName.is_null() {
                            pwszPrinterName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPrinterName);
                            (*ppi2w).pPrinterName = pwszPrinterName;
                            if pwszPrinterName.is_null() { break 'cleanup; }
                        }
                    }
                    break 'sw;
                }
                4 | 5 => {
                    {
                        if !(*ppi2a).pServerName.is_null() {
                            pwszServerName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pServerName);
                            (*ppi2w).pPrinterName = pwszServerName;
                            if pwszServerName.is_null() { break 'cleanup; }
                        }
                        if !(*ppi2a).pPrinterName.is_null() {
                            pwszPrinterName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi2a).pPrinterName);
                            (*ppi2w).pPrinterName = pwszPrinterName;
                            if pwszPrinterName.is_null() { break 'cleanup; }
                        }
                    }
                    break 'sw;
                }
                3 | 6 => {
                    break 'sw;
                }
                7 => {
                    {
                        if !(*ppi7a).pszObjectGUID.is_null() {
                            pwszPrinterName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*ppi7a).pszObjectGUID);
                            (*ppi7w).pszObjectGUID = pwszPrinterName;
                            if pwszPrinterName.is_null() { break 'cleanup; }
                        }
                    }
                    break 'sw;
                }
                8 | 9 => {
                    // 8 is the global default printer info and 9 already sets it instead of the per-user one
                    // still, PRINTER_INFO_8W is the same as PRINTER_INFO_9W
                    // fall through
                    {
                        RosConvertAnsiDevModeToUnicodeDevmode((*ppi9a).pDevMode, core::ptr::addr_of_mut!(pdmw));
                        (*ppi9w).pDevMode = pdmw;
                    }
                    break 'sw;
                }
                _ => {
                    // FIXME("Unsupported level %d\n", Level);
                    SetLastError(ERROR_INVALID_LEVEL);
                }
            }
        }

        Ret = SetPrinterW(hPrinter, Level, pPrinter, Command);
    }

    // Cleanup:
    if !pdmw.is_null() { HeapFree(hProcessHeap, 0, pdmw as LPVOID); }
    if !pwszPrinterName.is_null() { HeapFree(hProcessHeap, 0, pwszPrinterName as LPVOID); }
    if !pwszServerName.is_null() { HeapFree(hProcessHeap, 0, pwszServerName as LPVOID); }
    if !pwszShareName.is_null() { HeapFree(hProcessHeap, 0, pwszShareName as LPVOID); }
    if !pwszPortName.is_null() { HeapFree(hProcessHeap, 0, pwszPortName as LPVOID); }
    if !pwszDriverName.is_null() { HeapFree(hProcessHeap, 0, pwszDriverName as LPVOID); }
    if !pwszComment.is_null() { HeapFree(hProcessHeap, 0, pwszComment as LPVOID); }
    if !pwszLocation.is_null() { HeapFree(hProcessHeap, 0, pwszLocation as LPVOID); }
    if !pwszSepFile.is_null() { HeapFree(hProcessHeap, 0, pwszSepFile as LPVOID); }
    if !pwszPrintProcessor.is_null() { HeapFree(hProcessHeap, 0, pwszPrintProcessor as LPVOID); }
    if !pwszDatatype.is_null() { HeapFree(hProcessHeap, 0, pwszDatatype as LPVOID); }
    if !pwszParameters.is_null() { HeapFree(hProcessHeap, 0, pwszParameters as LPVOID); }
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn SetPrinterW(hPrinter: HANDLE, mut Level: DWORD, mut pPrinter: PBYTE, mut Command: DWORD) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut PrinterContainer: WINSPOOL_PRINTER_CONTAINER = core::mem::zeroed();
    let mut DevModeContainer: WINSPOOL_DEVMODE_CONTAINER = core::mem::zeroed();
    let mut SecurityContainer: WINSPOOL_SECURITY_CONTAINER = core::mem::zeroed();
    let mut sd: *mut _SECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut pi6: PRINTER_INFO_6 = core::mem::zeroed();

    // Sanity checks
    if pHandle.is_null() {
        return ERROR_INVALID_HANDLE as BOOL;
    }

    DevModeContainer.cbBuf = 0;
    DevModeContainer.pDevMode = core::ptr::null_mut();

    SecurityContainer.cbBuf = 0;
    SecurityContainer.pSecurity = core::ptr::null_mut();

    match Level {
        0 => {
            if Command == PRINTER_CONTROL_SET_STATUS {
                // Set the pPrinter parameter to a pointer to a DWORD value that specifies the new printer status.
                pi6.dwStatus = pPrinter as DWORD_PTR as DWORD;
                pPrinter = core::ptr::addr_of_mut!(pi6) as PBYTE;
                Level = 6;
                Command = 0;
            }
        }
        2 => {
            let pi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
            if !pi2w.is_null() {
                if !(*pi2w).pDevMode.is_null() {
                    if IsValidDevmodeNoSizeW((*pi2w).pDevMode) != 0 {
                        DevModeContainer.cbBuf = (*(*pi2w).pDevMode).dmSize as DWORD + (*(*pi2w).pDevMode).dmDriverExtra as DWORD;
                        DevModeContainer.pDevMode = (*pi2w).pDevMode as PBYTE;
                    }
                }

                if !(*pi2w).pSecurityDescriptor.is_null() {
                    sd = get_sd((*pi2w).pSecurityDescriptor as *mut _SECURITY_DESCRIPTOR, core::ptr::addr_of_mut!(size));
                    if !sd.is_null() {
                        SecurityContainer.cbBuf = size;
                        SecurityContainer.pSecurity = sd as PBYTE;
                    }
                }
            } else {
                SetLastError(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        }
        3 => {
            let pi3: PPRINTER_INFO_3 = pPrinter as PPRINTER_INFO_3;
            if !pi3.is_null() {
                if !(*pi3).pSecurityDescriptor.is_null() {
                    sd = get_sd((*pi3).pSecurityDescriptor as *mut _SECURITY_DESCRIPTOR, core::ptr::addr_of_mut!(size));
                    if !sd.is_null() {
                        SecurityContainer.cbBuf = size;
                        SecurityContainer.pSecurity = sd as PBYTE;
                    }
                }
            } else {
                SetLastError(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        }

        4 | 5 | 6 | 7 => {
            if pPrinter.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        }

        // 8 is the global default printer info and 9 already sets it instead of the per-user one
        // still, PRINTER_INFO_8W is the same as PRINTER_INFO_9W
        // fall through
        8 | 9 => {
            let pi9w: PPRINTER_INFO_9W = pPrinter as PPRINTER_INFO_9W;
            if !pi9w.is_null() {
                if !(*pi9w).pDevMode.is_null() {
                    if IsValidDevmodeNoSizeW((*pi9w).pDevMode) != 0 {
                        DevModeContainer.cbBuf = (*(*pi9w).pDevMode).dmSize as DWORD + (*(*pi9w).pDevMode).dmDriverExtra as DWORD;
                        DevModeContainer.pDevMode = (*pi9w).pDevMode as LPBYTE;
                    }
                }
            }
        }

        _ => {
            // FIXME( "Unsupported level %d\n", Level );
            SetLastError(ERROR_INVALID_LEVEL);
            return FALSE;
        }
    }

    PrinterContainer.PrinterInfo.pPrinterInfo1 = pPrinter as *mut WINSPOOL_PRINTER_INFO_1;
    PrinterContainer.Level = Level;

    // Do the RPC call
    {
        dwErrorCode = _RpcSetPrinter(
            (*pHandle).hPrinter,
            core::ptr::addr_of_mut!(PrinterContainer),
            core::ptr::addr_of_mut!(DevModeContainer),
            core::ptr::addr_of_mut!(SecurityContainer),
            Command,
        );
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    // RpcExcept handler: dead under this tree's USE_DUMMY_PSEH build (the C handler never runs either), and stable Rust has no SEH. Original: dwErrorCode = RpcExceptionCode();

    if !sd.is_null() {
        HeapFree(GetProcessHeap(), 0, sd as *mut core::ffi::c_void);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SplDriverUnloadComplete(pDriverFile: LPWSTR) -> BOOL {
    // TRACE("DriverUnloadComplete(%S)\n", pDriverFile);
    // UNIMPLEMENTED;
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn SpoolerPrinterEvent(pPrinterName: LPWSTR, DriverEvent: INT, Flags: DWORD, lParam: LPARAM) -> BOOL {
    let mut hLibrary: HMODULE;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE;

    if OpenPrinterW(pPrinterName, core::ptr::addr_of_mut!(hPrinter), core::ptr::null_mut()) != 0 {
        hLibrary = LoadPrinterDriver(hPrinter);

        if !hLibrary.is_null() {
            fpPrinterEvent = core::mem::transmute(GetProcAddress(hLibrary, b"DrvPrinterEvent\0".as_ptr() as *const i8));

            if let Some(fpPrintEvent) = fpPrinterEvent {
                Ret = fpPrintEvent(pPrinterName, DriverEvent as _, Flags as _, lParam as _) as _;
            }

            FreeLibrary(hLibrary);
        }

        ClosePrinter(hPrinter);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn file_dlg_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> INT_PTR {
    let mut filename: LPWSTR = core::ptr::null_mut();

    match msg {
        WM_INITDIALOG => {
            SetWindowLongPtrW(hwnd, DWLP_USER, lparam);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if (HIWORD!(wparam) as u32) == BN_CLICKED {
                if (LOWORD!(wparam) as u32) == IDOK {
                    let hf: HANDLE;
                    let len: DWORD = SendDlgItemMessageW(hwnd, EDITBOX as i32, WM_GETTEXTLENGTH, 0, 0) as DWORD;
                    let output: *mut LPWSTR;

                    filename = HeapAlloc(GetProcessHeap(), 0, ((len + 1) as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
                    if filename.is_null() {
                        return FALSE as INT_PTR;
                    }
                    GetDlgItemTextW(hwnd, EDITBOX as i32, filename, (len + 1) as i32);

                    if GetFileAttributesW(filename) != INVALID_FILE_ATTRIBUTES {
                        let mut caption: [WCHAR; 200] = [0; 200];
                        let mut message: [WCHAR; 200] = [0; 200];
                        let mb_ret: i32;

                        LoadStringW(hinstWinSpool, IDS_CAPTION, caption.as_mut_ptr(), ARRAYSIZE!(caption) as i32);
                        LoadStringW(hinstWinSpool, IDS_FILE_EXISTS, message.as_mut_ptr(), ARRAYSIZE!(message) as i32);
                        mb_ret = MessageBoxW(hwnd, message.as_ptr(), caption.as_ptr(), MB_OKCANCEL | MB_ICONEXCLAMATION);
                        if mb_ret == IDCANCEL as i32 {
                            HeapFree(GetProcessHeap(), 0, filename as *mut core::ffi::c_void);
                            return TRUE as INT_PTR;
                        }
                    }
                    hf = CreateFileW(filename, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
                    if hf == INVALID_HANDLE_VALUE {
                        let mut caption: [WCHAR; 200] = [0; 200];
                        let mut message: [WCHAR; 200] = [0; 200];

                        LoadStringW(hinstWinSpool, IDS_CAPTION, caption.as_mut_ptr(), ARRAYSIZE!(caption) as i32);
                        LoadStringW(hinstWinSpool, IDS_CANNOT_OPEN, message.as_mut_ptr(), ARRAYSIZE!(message) as i32);
                        MessageBoxW(hwnd, message.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONEXCLAMATION);
                        HeapFree(GetProcessHeap(), 0, filename as *mut core::ffi::c_void);
                        return TRUE as INT_PTR;
                    }
                    CloseHandle(hf);
                    DeleteFileW(filename);
                    output = GetWindowLongPtrW(hwnd, DWLP_USER) as *mut LPWSTR;
                    *output = filename;
                    EndDialog(hwnd, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }
                if (LOWORD!(wparam) as u32) == IDCANCEL {
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }
            }
            return FALSE as INT_PTR;
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn StartDocDlgW(hPrinter: HANDLE, doc: *mut DOCINFOW) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut attr: DWORD = 0;
    let mut retDlg: DWORD = 0;

    if (*doc).lpszOutput.is_null() {
        let mut pi5: *mut PRINTER_INFO_5W;
        GetPrinterW(hPrinter, 5, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(len));
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return core::ptr::null_mut();
        }
        pi5 = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut PRINTER_INFO_5W;
        if pi5.is_null() {
            return core::ptr::null_mut();
        }

        GetPrinterW(hPrinter, 5, pi5 as LPBYTE, len, core::ptr::addr_of_mut!(len));
        if (*pi5).pPortName.is_null() || _wcsicmp((*pi5).pPortName, FILE_Port.as_ptr()) != 0 {
            HeapFree(GetProcessHeap(), 0, pi5 as LPVOID);
            return core::ptr::null_mut();
        }
        HeapFree(GetProcessHeap(), 0, pi5 as LPVOID);
    }

    if (*doc).lpszOutput.is_null() || _wcsicmp((*doc).lpszOutput, FILE_Port.as_ptr()) != 0 {
        let mut name: LPWSTR = core::ptr::null_mut();

        retDlg = DialogBoxParamW(
            hinstWinSpool,
            MAKEINTRESOURCEW!(FILENAME_DIALOG),
            GetForegroundWindow(),
            Some(file_dlg_proc),
            core::ptr::addr_of_mut!(name) as LPARAM,
        ) as DWORD;

        if retDlg == IDOK {
            len = GetFullPathNameW(name, 0, core::ptr::null_mut(), core::ptr::null_mut());
            if len == 0 {
                HeapFree(GetProcessHeap(), 0, name as LPVOID);
                return core::ptr::null_mut();
            }
            ret = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
            if ret.is_null() {
                HeapFree(GetProcessHeap(), 0, name as LPVOID);
                return core::ptr::null_mut();
            }
            GetFullPathNameW(name, len, ret, core::ptr::null_mut());
            HeapFree(GetProcessHeap(), 0, name as LPVOID);
        } else if retDlg == 0 {
            ret = SP_ERROR as LPWSTR;
        } else if retDlg == IDCANCEL {
            SetLastError(ERROR_CANCELLED);
            ret = SP_APPABORT as LPWSTR;
        }
        return ret;
    }

    len = GetFullPathNameW((*doc).lpszOutput, 0, core::ptr::null_mut(), core::ptr::null_mut());
    if len == 0 {
        return core::ptr::null_mut();
    }

    ret = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    GetFullPathNameW((*doc).lpszOutput, len, ret, core::ptr::null_mut());

    attr = GetFileAttributesW(ret);
    if attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        HeapFree(GetProcessHeap(), 0, ret as LPVOID);
        ret = core::ptr::null_mut();
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn StartDocDlgA(hPrinter: HANDLE, doc: *mut DOCINFOA) -> LPSTR {
    let mut usBuffer: _UNICODE_STRING = core::mem::zeroed();
    let mut docW: DOCINFOW = core::mem::zeroed();
    let mut retW: LPWSTR;
    let mut docnameW: LPWSTR = core::ptr::null_mut();
    let mut outputW: LPWSTR = core::ptr::null_mut();
    let mut datatypeW: LPWSTR = core::ptr::null_mut();
    let mut ret: LPSTR = core::ptr::null_mut();

    docW.cbSize = core::mem::size_of::<DOCINFOW>() as _;
    'early: {
        if !(*doc).lpszDocName.is_null() {
            docnameW = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*doc).lpszDocName);
            docW.lpszDocName = docnameW;
            if docW.lpszDocName.is_null() {
                break 'early;
            }
        }
        if !(*doc).lpszOutput.is_null() {
            outputW = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*doc).lpszOutput);
            docW.lpszOutput = outputW;
            if docW.lpszOutput.is_null() {
                break 'early;
            }
        }
        if !(*doc).lpszDatatype.is_null() {
            datatypeW = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*doc).lpszDatatype);
            docW.lpszDatatype = datatypeW;
            if docW.lpszDatatype.is_null() {
                break 'early;
            }
        }
        docW.fwType = (*doc).fwType;

        retW = StartDocDlgW(hPrinter, core::ptr::addr_of_mut!(docW));

        if !retW.is_null() {
            let len: DWORD = WideCharToMultiByte(
                CP_ACP,
                0,
                retW,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            ret = HeapAlloc(
                GetProcessHeap(),
                0,
                len as usize,
            ) as LPSTR;
            if !ret.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    retW,
                    -1,
                    ret,
                    len as ::core::ffi::c_int,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
            }
            HeapFree(GetProcessHeap(), 0, retW as *mut core::ffi::c_void);
        }
    }

    // failed:
    if !datatypeW.is_null() {
        HeapFree(GetProcessHeap(), 0, datatypeW as *mut core::ffi::c_void);
    }
    if !outputW.is_null() {
        HeapFree(GetProcessHeap(), 0, outputW as *mut core::ffi::c_void);
    }
    if !docnameW.is_null() {
        HeapFree(GetProcessHeap(), 0, docnameW as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn StartDocPrinterA(hPrinter: HANDLE, Level: DWORD, pDocInfo: PBYTE) -> DWORD {
    let mut wDocInfo1: DOC_INFO_1W = core::mem::zeroed();
    let mut cch: DWORD;
    let mut dwErrorCode: DWORD = 0;
    let mut dwReturnValue: DWORD = 0;
    let mut pDocInfo1: PDOC_INFO_1A = pDocInfo as PDOC_INFO_1A;

    'Cleanup: {
        // Only check the minimum required for accessing pDocInfo.
        // Additional sanity checks are done in StartDocPrinterW.
        if pDocInfo1.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if !(*pDocInfo1).pDatatype.is_null() {
            // Convert pDocInfo1->pDatatype to a Unicode string wDocInfo1.pDatatype
            cch = strlen((*pDocInfo1).pDatatype) as DWORD;

            wDocInfo1.pDatatype = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as usize) * core::mem::size_of::<WCHAR>(),
            ) as PWSTR;
            if wDocInfo1.pDatatype.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pDocInfo1).pDatatype,
                -1,
                wDocInfo1.pDatatype,
                (cch + 1) as i32,
            );
        }

        if !(*pDocInfo1).pDocName.is_null() {
            // Convert pDocInfo1->pDocName to a Unicode string wDocInfo1.pDocName
            cch = strlen((*pDocInfo1).pDocName) as DWORD;

            wDocInfo1.pDocName = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as usize) * core::mem::size_of::<WCHAR>(),
            ) as PWSTR;
            if wDocInfo1.pDocName.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pDocInfo1).pDocName,
                -1,
                wDocInfo1.pDocName,
                (cch + 1) as i32,
            );
        }

        if !(*pDocInfo1).pOutputFile.is_null() {
            // Convert pDocInfo1->pOutputFile to a Unicode string wDocInfo1.pOutputFile
            cch = strlen((*pDocInfo1).pOutputFile) as DWORD;

            wDocInfo1.pOutputFile = HeapAlloc(
                hProcessHeap,
                0,
                ((cch + 1) as usize) * core::mem::size_of::<WCHAR>(),
            ) as PWSTR;
            if wDocInfo1.pOutputFile.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pDocInfo1).pOutputFile,
                -1,
                wDocInfo1.pOutputFile,
                (cch + 1) as i32,
            );
        }

        dwReturnValue = StartDocPrinterW(
            hPrinter,
            Level,
            core::ptr::addr_of_mut!(wDocInfo1) as PBYTE,
        );
        dwErrorCode = GetLastError();
    }

    // Cleanup:
    if !wDocInfo1.pDatatype.is_null() {
        HeapFree(hProcessHeap, 0, wDocInfo1.pDatatype as LPVOID);
    }

    if !wDocInfo1.pDocName.is_null() {
        HeapFree(hProcessHeap, 0, wDocInfo1.pDocName as LPVOID);
    }

    if !wDocInfo1.pOutputFile.is_null() {
        HeapFree(hProcessHeap, 0, wDocInfo1.pOutputFile as LPVOID);
    }

    SetLastError(dwErrorCode);
    dwReturnValue
}

#[no_mangle]
pub unsafe extern "system" fn StartDocPrinterW(hPrinter: HANDLE, Level: DWORD, pDocInfo: PBYTE) -> DWORD {
    let mut cbAddJobInfo1: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwReturnValue: DWORD = 0;
    let mut pAddJobInfo1: PADDJOB_INFO_1W = core::ptr::null_mut();
    let mut pDocInfo1: PDOC_INFO_1W = pDocInfo as PDOC_INFO_1W;
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'quit: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'quit;
        }

        if pDocInfo1.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'quit;
        }

        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'quit;
        }

        if (*pHandle).bStartedDoc() != 0 {
            dwErrorCode = ERROR_INVALID_PRINTER_STATE;
            break 'quit;
        }

        // Check if we want to redirect output into a file.
        if !(*pDocInfo1).pOutputFile.is_null() {
            // Do a StartDocPrinter RPC call in this case.
            dwErrorCode = _StartDocPrinterWithRPC(pHandle, pDocInfo1);
        } else {
            // Allocate memory for the ADDJOB_INFO_1W structure and a path.
            cbAddJobInfo1 = core::mem::size_of::<ADDJOB_INFO_1W>() as DWORD
                + (MAX_PATH as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
            pAddJobInfo1 = HeapAlloc(hProcessHeap, 0, cbAddJobInfo1 as usize) as PADDJOB_INFO_1W;
            if pAddJobInfo1.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'quit;
            }

            // Try to add a new job.
            // This only succeeds if the printer is set to do spooled printing.
            if AddJobW(
                pHandle as HANDLE,
                1,
                pAddJobInfo1 as PBYTE,
                cbAddJobInfo1,
                core::ptr::addr_of_mut!(cbNeeded),
            ) != 0
            {
                // Do spooled printing.
                dwErrorCode = _StartDocPrinterSpooled(pHandle, pDocInfo1, pAddJobInfo1);
            } else if GetLastError() == ERROR_INVALID_ACCESS {
                // ERROR_INVALID_ACCESS is returned when the printer is set to do direct printing.
                // In this case, we do a StartDocPrinter RPC call.
                dwErrorCode = _StartDocPrinterWithRPC(pHandle, pDocInfo1);
            } else {
                dwErrorCode = GetLastError();
                break 'quit;
            }
        }

        if dwErrorCode == ERROR_SUCCESS {
            (*pHandle).set_bStartedDoc(TRUE);
            dwReturnValue = (*pHandle).dwJobID;
            if (*pHandle).bTrayIcon() == 0 {
                UpdateTrayIcon(hPrinter, (*pHandle).dwJobID);
            }
        }
    }

    // Cleanup:
    if !pAddJobInfo1.is_null() {
        HeapFree(hProcessHeap, 0, pAddJobInfo1 as LPVOID);
    }

    SetLastError(dwErrorCode);
    return dwReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn StartPagePrinter(hPrinter: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call
        // RpcTryExcept
        dwErrorCode = _RpcStartPagePrinter((*pHandle).hPrinter);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrinter(hPrinter: HANDLE, pBuf: PVOID, cbBuf: DWORD, pcWritten: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        if (*pHandle).bStartedDoc() == 0 {
            dwErrorCode = ERROR_SPL_NO_STARTDOC;
            break 'Cleanup;
        }

        if (*pHandle).hSPLFile != INVALID_HANDLE_VALUE {
            // Write to the spool file. This doesn't need an RPC request.
            if WriteFile((*pHandle).hSPLFile, pBuf, cbBuf, pcWritten, core::ptr::null_mut()) == 0 {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            dwErrorCode = ERROR_SUCCESS;
        } else {
            // TODO: This case (for direct printing or remote printing) has bad performance if multiple small-sized WritePrinter calls are performed.
            // We may increase performance by writing into a buffer and only doing a single RPC call when the buffer is full.

            // Do the RPC call (RpcTryExcept region; SEH is not expressible in Rust,
            // so the protected call is performed directly).
            dwErrorCode = _RpcWritePrinter((*pHandle).hPrinter, pBuf as *mut BYTE, cbBuf, pcWritten);
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler would set:
            //     dwErrorCode = RpcExceptionCode();
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn XcvDataW(hXcv: HANDLE, pszDataName: PCWSTR, mut pInputData: PBYTE, cbInputData: DWORD, mut pOutputData: PBYTE, cbOutputData: DWORD, pcbOutputNeeded: PDWORD, pdwStatus: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut Bogus: DWORD = 0;
    let mut pHandle: PSPOOLER_HANDLE = hXcv as PSPOOLER_HANDLE;

    'Cleanup: {
        if pcbOutputNeeded.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        //
        // Do fixups.
        //
        if pInputData.is_null() {
            if cbInputData == 0 {
                pInputData = core::ptr::addr_of_mut!(Bogus) as PBYTE;
            }
        }

        if pOutputData.is_null() {
            if cbOutputData == 0 {
                pOutputData = core::ptr::addr_of_mut!(Bogus) as PBYTE;
            }
        }

        // Do the RPC call
        // RpcTryExcept: the try body is executed unconditionally here.
        dwErrorCode = _RpcXcvData((*pHandle).hPrinter,
                                   pszDataName,
                                   pInputData,
                                   cbInputData,
                                   pOutputData,
                                   cbOutputData,
                                   pcbOutputNeeded,
                                   pdwStatus);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler is unreachable in this lowering.
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        //IntUnprotectHandle( hXcv );
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CreatePrinterIC(
    hPrinter: HANDLE,
    pDevMode: LPDEVMODEW,
) -> HANDLE {
    // UNIMPLEMENTED;
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterIC(hPrinterIC: HANDLE) -> BOOL {
    // UNIMPLEMENTED;
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn PlayGdiScriptOnPrinterIC(
    hPrinterIC: HANDLE,
    pIn: LPBYTE,
    cIn: DWORD,
    pOut: LPBYTE,
    cOut: DWORD,
    ul: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    FALSE as BOOL
}
