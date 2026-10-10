use crate::localspl::jobs::GetJobFilePath;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D575D22E81A4: [u16; 4] = [0x0053u16, 0x0050, 0x004C, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn PrintingThreadProc(pJob: PLOCAL_JOB) -> DWORD {
    let cchMaxJobIdDigits: DWORD = 5;
    let wszJobAppendix: [u16; 7] = [0x002Cu16, 0x0020, 0x004A, 0x006F, 0x0062, 0x0020, 0x0000];
    let cchJobAppendix: DWORD = (_countof!(wszJobAppendix) as DWORD).wrapping_sub(1);
    let wszPortAppendix: [u16; 7] = [0x002Cu16, 0x0020, 0x0050, 0x006F, 0x0072, 0x0074, 0x0000];

    let mut cchPortName: DWORD = 0;
    let mut cchPrinterName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hPrintProcessor: HANDLE = core::ptr::null_mut();
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = (*pJob).pPrintProcessor;
    let mut OpenData: PRINTPROCESSOROPENDATA = core::mem::zeroed();
    let mut pwszPrinterAndJob: PWSTR = core::ptr::null_mut();
    let mut pwszPrinterPort: PWSTR = core::ptr::null_mut();
    let mut pwszSPLFile: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        // Prepare the pPrinterName parameter.
        cchPortName = wcslen((*(*(*pJob).pPrinter).pPort).pwszName) as DWORD;
        pwszPrinterPort = DllAllocSplMem(
            cchPortName
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD)
                .wrapping_add(core::mem::size_of::<[u16; 7]>() as DWORD),
        ) as PWSTR;
        if pwszPrinterPort.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        CopyMemory!(
            pwszPrinterPort as *mut core::ffi::c_void,
            (*(*(*pJob).pPrinter).pPort).pwszName as *const core::ffi::c_void,
            cchPortName.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as usize,
        );
        CopyMemory!(
            pwszPrinterPort.add(cchPortName as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszPortAppendix) as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 7]>() as usize,
        );

        // Prepare the pPrintProcessorOpenData parameter.
        OpenData.JobId = (*pJob).dwJobID;
        OpenData.pDatatype = (*pJob).pwszDatatype as LPWSTR;
        OpenData.pDevMode = (*pJob).pDevMode;
        OpenData.pDocumentName = (*pJob).pwszDocumentName as LPWSTR;
        OpenData.pOutputFile = core::ptr::null_mut();
        OpenData.pParameters = (*pJob).pwszPrintProcessorParameters as LPWSTR;
        OpenData.pPrinterName = (*(*pJob).pPrinter).pwszPrinterName as LPWSTR;

        // Associate our job to the port.
        (*(*(*pJob).pPrinter).pPort).pNextJobToProcess = pJob;

        // Open a handle to the Print Processor.
        hPrintProcessor = (*pPrintProcessor)
            .pfnOpenPrintProcessor
            .unwrap_unchecked()(
            pwszPrinterPort as LPWSTR,
            core::ptr::addr_of_mut!(OpenData),
        );
        if hPrintProcessor.is_null() {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // Let other functions use the Print Processor as well while it's opened.
        (*pJob).hPrintProcessor = hPrintProcessor;

        // Prepare the pDocumentName parameter.
        cchPrinterName = wcslen(OpenData.pPrinterName as *const u16) as DWORD;
        pwszPrinterAndJob = DllAllocSplMem(
            cchPrinterName
                .wrapping_add(cchJobAppendix)
                .wrapping_add(cchMaxJobIdDigits)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
        ) as PWSTR;
        if pwszPrinterAndJob.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        CopyMemory!(
            pwszPrinterAndJob as *mut core::ffi::c_void,
            OpenData.pPrinterName as *const core::ffi::c_void,
            cchPrinterName.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as usize,
        );
        CopyMemory!(
            pwszPrinterAndJob.add(cchPrinterName as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszJobAppendix) as *const core::ffi::c_void,
            cchJobAppendix.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD) as usize,
        );
        _ultow(
            OpenData.JobId,
            pwszPrinterAndJob.add(cchPrinterName.wrapping_add(cchJobAppendix) as usize),
            10,
        );

        // Printing starts here.
        (*pJob).dwStatus |= JOB_STATUS_PRINTING;

        // Print the document.
        if (*pPrintProcessor)
            .pfnPrintDocumentOnPrintProcessor
            .unwrap_unchecked()(hPrintProcessor, pwszPrinterAndJob as LPWSTR)
            == 0
        {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // Close the Print Processor.
        (*pPrintProcessor)
            .pfnClosePrintProcessor
            .unwrap_unchecked()(hPrintProcessor);
        hPrintProcessor = core::ptr::null_mut();

        // Delete the spool file.
        pwszSPLFile = DllAllocSplMem(GetJobFilePath(
            RUSTCONV_WSTR_D575D22E81A4.as_ptr(),
            0,
            core::ptr::null_mut(),
        )) as PWSTR;
        if pwszSPLFile.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'Cleanup;
        }

        GetJobFilePath(RUSTCONV_WSTR_D575D22E81A4.as_ptr(), OpenData.JobId, pwszSPLFile);
        DeleteFileW(pwszSPLFile);

        // We were successful!
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !hPrintProcessor.is_null() {
        (*pPrintProcessor)
            .pfnClosePrintProcessor
            .unwrap_unchecked()(hPrintProcessor);
    }

    if !pwszPrinterPort.is_null() {
        DllFreeSplMem(pwszPrinterPort as PVOID);
    }

    if !pwszPrinterAndJob.is_null() {
        DllFreeSplMem(pwszPrinterAndJob as PVOID);
    }

    if !pwszSPLFile.is_null() {
        DllFreeSplMem(pwszSPLFile as PVOID);
    }

    return dwErrorCode;
}
