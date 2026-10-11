// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_903B52F937BA: [u16; 4] = [0x0052u16, 0x0041, 0x0057, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_903B52F937BA: [u16; 4] = [0x0052u16, 0x0041, 0x0057, 0x0000];

static mut _pwszDatatypes: [PCWSTR; 2] = unsafe {
    [
        RUSTCONV_C2R_WSTR_903B52F937BA.as_ptr(),
        ::core::ptr::null::<WCHAR>(),
    ]
};

#[no_mangle]
pub unsafe extern "system" fn ClosePrintProcessor(hPrintProcessor: HANDLE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut pHandle: PWINPRINT_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks
        if hPrintProcessor.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pHandle = hPrintProcessor as PWINPRINT_HANDLE;

        // Free all structure fields for which memory has been allocated.
        if !(*pHandle).pwszDatatype.is_null() {
            DllFreeSplStr((*pHandle).pwszDatatype);
        }

        if !(*pHandle).pwszDocumentName.is_null() {
            DllFreeSplStr((*pHandle).pwszDocumentName);
        }

        if !(*pHandle).pwszOutputFile.is_null() {
            DllFreeSplStr((*pHandle).pwszOutputFile);
        }

        if !(*pHandle).pwszPrinterPort.is_null() {
            DllFreeSplStr((*pHandle).pwszPrinterPort);
        }

        // Finally free the WINSPOOL_HANDLE structure itself.
        DllFreeSplMem(pHandle as PVOID);
        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ControlPrintProcessor(
    hPrintProcessor: HANDLE,
    Command: DWORD,
) -> BOOL {
    // TRACE("ControlPrintProcessor(%p, %lu)\n", hPrintProcessor, Command);
    // UNIMPLEMENTED;
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorDatatypesW(pName: PWSTR, pPrintProcessorName: PWSTR, Level: DWORD, mut pDatatypes: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut cbDatatype: DWORD;
    let mut dwDatatypeCount: DWORD = 0;
    let mut dwOffsets: [DWORD; unsafe { _countof!(_pwszDatatypes) as usize }] = [0; unsafe { _countof!(_pwszDatatypes) as usize }];
    let mut pCurrentDatatype: *mut PCWSTR;
    let mut pCurrentOffset: PDWORD = core::ptr::addr_of_mut!(dwOffsets) as PDWORD;

    // Sanity checks
    if Level != 1 || pcbNeeded.is_null() || pcReturned.is_null() {
        return FALSE as BOOL;
    }

    // Count the required buffer size and the number of datatypes.
    *pcbNeeded = 0;
    *pcReturned = 0;

    pCurrentDatatype = core::ptr::addr_of_mut!(_pwszDatatypes) as *mut PCWSTR;
    while !(*pCurrentDatatype).is_null() {
        cbDatatype = ((wcslen(*pCurrentDatatype) as DWORD).wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<DATATYPES_INFO_1W>() as DWORD).wrapping_add(cbDatatype),
        );

        // Also calculate the offset in the output buffer of the pointer to this datatype string.
        *pCurrentOffset = dwDatatypeCount
            .wrapping_mul(core::mem::size_of::<DATATYPES_INFO_1W>() as DWORD)
            .wrapping_add(FIELD_OFFSET!(DATATYPES_INFO_1W, pName) as DWORD);

        dwDatatypeCount = dwDatatypeCount.wrapping_add(1);
        pCurrentOffset = pCurrentOffset.offset(1);
        pCurrentDatatype = pCurrentDatatype.offset(1);
    }

    // Check if the supplied buffer is large enough.
    if cbBuf < *pcbNeeded {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return FALSE as BOOL;
    }

    // Check if a buffer was supplied at all.
    if pDatatypes.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    // Copy over all datatypes.
    *pCurrentOffset = MAXDWORD;
    PackStrings(
        core::ptr::addr_of_mut!(_pwszDatatypes) as *mut PCWSTR,
        pDatatypes,
        core::ptr::addr_of_mut!(dwOffsets) as *mut DWORD,
        pDatatypes.add(*pcbNeeded as usize),
    );

    *pcReturned = dwDatatypeCount;
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrintProcessorCapabilities(
    pValueName: PWSTR,
    dwAttributes: DWORD,
    pData: PBYTE,
    nSize: DWORD,
    pcbNeeded: PDWORD,
) -> DWORD {
    // TRACE("GetPrintProcessorCapabilities(%S, %lu, %p, %lu, %p)\n", pValueName, dwAttributes, pData, nSize, pcbNeeded);
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn OpenPrintProcessor(pPrinterName: PWSTR, pPrintProcessorOpenData: PPRINTPROCESSOROPENDATA) -> HANDLE {
    let mut dwErrorCode: DWORD = 0;
    let mut hReturnValue: HANDLE = core::ptr::null_mut();
    let mut pHandle: PWINPRINT_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks
        // This time a datatype needs to be given. We can't fall back to a default here.
        if pPrintProcessorOpenData.is_null()
            || (*pPrintProcessorOpenData).pDatatype.is_null()
            || *(*pPrintProcessorOpenData).pDatatype == 0
        {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'Cleanup;
        }

        // Create a new WINPRINT_HANDLE structure and fill the relevant fields.
        pHandle = DllAllocSplMem(core::mem::size_of::<WINPRINT_HANDLE>() as DWORD) as PWINPRINT_HANDLE;

        // Check what datatype was given.
        if _wcsicmp((*pPrintProcessorOpenData).pDatatype, RUSTCONV_WSTR_903B52F937BA.as_ptr()) == 0 {
            (*pHandle).Datatype = RAW as DWORD;
        } else {
            dwErrorCode = ERROR_INVALID_DATATYPE as DWORD;
            break 'Cleanup;
        }

        // Fill the relevant fields.
        (*pHandle).dwJobID = (*pPrintProcessorOpenData).JobId;
        (*pHandle).pwszDatatype = AllocSplStr((*pPrintProcessorOpenData).pDatatype as PCWSTR);
        (*pHandle).pwszDocumentName = AllocSplStr((*pPrintProcessorOpenData).pDocumentName as PCWSTR);
        (*pHandle).pwszOutputFile = AllocSplStr((*pPrintProcessorOpenData).pOutputFile as PCWSTR);
        (*pHandle).pwszPrinterPort = AllocSplStr(pPrinterName as PCWSTR);

        // We were successful! Return the handle and don't let the cleanup routine free it.
        dwErrorCode = ERROR_SUCCESS as DWORD;
        hReturnValue = pHandle as HANDLE;
        pHandle = core::ptr::null_mut();
    }

    if !pHandle.is_null() {
        DllFreeSplMem(pHandle as PVOID);
    }

    SetLastError(dwErrorCode);
    return hReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn PrintDocumentOnPrintProcessor(
    hPrintProcessor: HANDLE,
    pDocumentName: PWSTR,
) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PWINPRINT_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks
        if hPrintProcessor.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pHandle = hPrintProcessor as PWINPRINT_HANDLE;

        // Call the corresponding Print function for the datatype.
        if (*pHandle).Datatype == RAW as u32 {
            dwErrorCode = PrintRawJob(pHandle, pDocumentName);
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
