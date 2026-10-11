use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PrintRawJob(pHandle: PWINPRINT_HANDLE, pwszPrinterAndJob: PWSTR) -> DWORD {
    // Use a read buffer of 256 KB size like Windows does.
    let cbReadBuffer: DWORD = 262144;

    let mut bStartedDoc: BOOL = FALSE as BOOL;
    let mut DocInfo1: DOC_INFO_1W = core::mem::zeroed();
    let mut cbRead: DWORD = 0;
    let mut cbWritten: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hPrintJob: HANDLE = core::ptr::null_mut();
    let mut hPrintMonitor: HANDLE = core::ptr::null_mut();
    let mut pBuffer: PBYTE = core::ptr::null_mut();

    'quit: {
        // Open the spooled job to read from it.
        if OpenPrinterW(
            pwszPrinterAndJob as LPWSTR,
            core::ptr::addr_of_mut!(hPrintJob),
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'quit;
        }

        // Open a Print Monitor handle to write to it.
        if OpenPrinterW(
            (*pHandle).pwszPrinterPort as LPWSTR,
            core::ptr::addr_of_mut!(hPrintMonitor),
            core::ptr::null_mut(),
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'quit;
        }

        // Fill the Document Information.
        DocInfo1.pDatatype = (*pHandle).pwszDatatype;
        DocInfo1.pDocName = (*pHandle).pwszDocumentName;
        DocInfo1.pOutputFile = (*pHandle).pwszOutputFile;

        // Tell the Print Monitor that we're starting a new document.
        if StartDocPrinterW(
            hPrintMonitor,
            1,
            core::ptr::addr_of_mut!(DocInfo1) as PBYTE,
        ) == 0
        {
            dwErrorCode = GetLastError();
            break 'quit;
        }

        bStartedDoc = TRUE as BOOL;

        // Allocate a read buffer on the heap. This would easily exceed the stack size.
        pBuffer = DllAllocSplMem(cbReadBuffer) as PBYTE;
        if pBuffer.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
            break 'quit;
        }

        // Loop as long as data is available.
        while ReadPrinter(
            hPrintJob,
            pBuffer as LPVOID,
            cbReadBuffer,
            core::ptr::addr_of_mut!(cbRead),
        ) != 0
            && cbRead != 0
        {
            // Write it to the Print Monitor.
            WritePrinter(
                hPrintMonitor,
                pBuffer as LPVOID,
                cbRead,
                core::ptr::addr_of_mut!(cbWritten),
            );
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pBuffer.is_null() {
        DllFreeSplMem(pBuffer as *mut core::ffi::c_void);
    }

    if bStartedDoc != 0 {
        EndDocPrinter(hPrintMonitor);
    }

    if !hPrintMonitor.is_null() {
        ClosePrinter(hPrintMonitor);
    }

    if !hPrintJob.is_null() {
        ClosePrinter(hPrintJob);
    }

    return dwErrorCode;
}
