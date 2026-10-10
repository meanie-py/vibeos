use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddPrintProvidorW(
    pName: PWSTR,
    Level: DWORD,
    pProviderInfo: PBYTE,
) -> BOOL {
    // FIXME("AddPrintProvidorW(%S, %lu, %p)\n", pName, Level, pProviderInfo);
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrintProvidorW(
    pName: PWSTR,
    pEnvironment: PWSTR,
    pPrintProviderName: PWSTR,
) -> BOOL {
    // FIXME("DeletePrintProvidorW(%s, %s, %s)\n", pName, pEnvironment, pPrintProviderName);
    FALSE as BOOL
}
