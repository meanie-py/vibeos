use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only object-like macros with no value (`WIN32_NO_STATUS`, `_INC_WINDOWS`, `COM_NO_WINDOWS_H`, `NDEBUG`), which are not constants to emit, and a function prototype (`NtUserGetDC`), which is a function declaration and so is skipped per the instructions. There are no types, tables, or valued constants to translate here.
#[no_mangle]
pub unsafe extern "system" fn GetDpiForSystem() -> UINT {
    let mut hDC: HDC = core::ptr::null_mut();
    let mut Dpi: UINT = 0;
    hDC = NtUserGetDC(core::ptr::null_mut());
    Dpi = GetDeviceCaps(hDC, (LOGPIXELSY as i32)) as UINT;
    ReleaseDC(core::ptr::null_mut(), hDC);
    return Dpi;
}


#[no_mangle]
pub unsafe extern "system" fn GetDpiForWindow(hWnd: HWND) -> UINT {
    UNIMPLEMENTED_ONCE!();
    UNREFERENCED_PARAMETER!(hWnd);
    return GetDpiForSystem();
}

#[no_mangle]
pub unsafe extern "system" fn IsProcessDPIAware() -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDPIAware() -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDpiAwarenessContext(
    context: DPI_AWARENESS_CONTEXT,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessDpiAwarenessInternal(
    process: HANDLE,
    awareness: *mut DPI_AWARENESS,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDpiAwarenessInternal(awareness: DPI_AWARENESS) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn GetDpiForMonitorInternal(
    monitor: HMONITOR,
    r#type: UINT,
    x: *mut UINT,
    y: *mut UINT,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn LogicalToPhysicalPoint(hwnd: HWND, point: *mut POINT) -> BOOL {
    // UNIMPLEMENTED;
    (TRUE) as i32
}
