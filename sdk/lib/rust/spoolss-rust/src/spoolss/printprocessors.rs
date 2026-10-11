use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
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

#[no_mangle]
pub unsafe extern "system" fn AddPrintProcessorW(mut pName: PWSTR, mut pEnvironment: PWSTR, mut pPathName: PWSTR, mut pPrintProcessorName: PWSTR) -> i32 {
    if pEnvironment.is_null() || *pEnvironment == 0 {
        pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as PWSTR;
    }
    return FALSE as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProcessorW(
    mut pName: PWSTR,
    mut pEnvironment: PWSTR,
    mut pPrintProcessorName: PWSTR,
) -> BOOL {
    if pEnvironment.is_null() || *pEnvironment == 0 {
        pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PWSTR;
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorDatatypesW(
    pName: PWSTR,
    pPrintProcessorName: PWSTR,
    Level: DWORD,
    pDatatypes: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    // Sanity checks
    if cbBuf != 0 && pDatatypes.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    if cbBuf != 0 && pDatatypes.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    // Always call this function on the Local Spooler.
    pPrintProvider = CONTAINING_RECORD!(PrintProviderList.Flink, SPOOLSS_PRINT_PROVIDER, Entry);
    return (*pPrintProvider)
        .PrintProvider
        .fpEnumPrintProcessorDatatypes
        .unwrap_unchecked()(
        pName,
        pPrintProcessorName,
        Level,
        pDatatypes,
        cbBuf,
        pcbNeeded,
        pcReturned,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintProcessorsW(pName: PWSTR, pEnvironment: PWSTR, Level: DWORD, pPrintProcessorInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    // Always call this function on the Local Spooler.
    let mut pPrintProvider: *mut SPOOLSS_PRINT_PROVIDER = CONTAINING_RECORD!(PrintProviderList.Flink, SPOOLSS_PRINT_PROVIDER, Entry);

    if cbBuf != 0 && pPrintProcessorInfo.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        return FALSE as BOOL;
    }

    return (*pPrintProvider)
        .PrintProvider
        .fpEnumPrintProcessors
        .unwrap_unchecked()(
        pName as LPWSTR,
        pEnvironment as LPWSTR,
        Level,
        pPrintProcessorInfo as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
        pcReturned as LPDWORD,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrintProcessorDirectoryW(pName: PWSTR, pEnvironment: PWSTR, Level: DWORD, pPrintProcessorInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    // Sanity checks
    if cbBuf != 0 && pPrintProcessorInfo.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    if cbBuf != 0 && pPrintProcessorInfo.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    // Always call this function on the Local Spooler.
    pPrintProvider = CONTAINING_RECORD!(PrintProviderList.Flink, SPOOLSS_PRINT_PROVIDER, Entry);
    return (*pPrintProvider).PrintProvider.fpGetPrintProcessorDirectory.unwrap_unchecked()(
        pName,
        pEnvironment,
        Level,
        pPrintProcessorInfo,
        cbBuf,
        pcbNeeded,
    );
}
