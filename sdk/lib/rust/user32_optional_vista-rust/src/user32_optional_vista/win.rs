use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(win): per-file debug channel name

#[no_mangle]
pub unsafe extern "system" fn GetWindowDisplayAffinity(hwnd: HWND, affinity: *mut DWORD) -> BOOL {
    if hwnd.is_null() || affinity.is_null() {
        SetLastError(
            (if !hwnd.is_null() {
                ERROR_NOACCESS
            } else {
                ERROR_INVALID_WINDOW_HANDLE
            }) as DWORD,
        );
        return (FALSE) as BOOL;
    }

    *affinity = WDA_NONE as DWORD;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowDisplayAffinity(hwnd: HWND, affinity: DWORD) -> BOOL {
    if hwnd.is_null() {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return (FALSE) as BOOL;
    }

    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowCompositionAttribute(hwnd: HWND, data: *mut core::ffi::c_void) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
