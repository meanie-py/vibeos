use crate::prelude::*;
use core::ffi::c_void;

// ntdll's, not kernel32's SetLastError: this crate is linked into user32.dll AND into
// user32_vista.dll, and user32_vista imports only user32, gdi32 and ntdll. kernel32's
// SetLastError is this call. (Unresolved there until 2026-10-17, when the user32_vista cell
// first linked the body: cmake's link had bound the exports to import thunks instead.)
extern "system" {
    fn RtlSetLastWin32Error(dwErrCode: DWORD);
}

// ---- file-scope preamble (translated) ----
// The preamble contains only object-like macros with no value (`WIN32_NO_STATUS`, `_INC_WINDOWS`, `COM_NO_WINDOWS_H`, `NDEBUG`), which are not constants to emit, and a function prototype (`NtUserGetDC`), which is a function declaration and so is skipped per the instructions. There are no types, tables, or valued constants to translate here.
// windef.h DPI_AWARENESS_CONTEXT_*: (HANDLE)-1 .. -5.
const DPI_CTX_UNAWARE: isize = -1;
const DPI_CTX_SYSTEM_AWARE: isize = -2;
const DPI_CTX_UNAWARE_GDISCALED: isize = -5;

/// Process-wide DPI awareness (a pseudo-handle value).  Default UNAWARE.  user32-rust reads
/// it through `User32Vista_GetProcessDpiContext` (GetWindowDpiAwarenessContext etc.).
static PROCESS_DPI_CONTEXT: core::sync::atomic::AtomicIsize =
    core::sync::atomic::AtomicIsize::new(DPI_CTX_UNAWARE);

#[no_mangle]
pub extern "C" fn User32Vista_GetProcessDpiContext() -> isize {
    PROCESS_DPI_CONTEXT.load(core::sync::atomic::Ordering::SeqCst)
}

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
    // Anything but UNAWARE / UNAWARE_GDISCALED is "DPI aware".
    let c = PROCESS_DPI_CONTEXT.load(core::sync::atomic::Ordering::SeqCst);
    (c != DPI_CTX_UNAWARE && c != DPI_CTX_UNAWARE_GDISCALED) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDPIAware() -> BOOL {
    PROCESS_DPI_CONTEXT.store(DPI_CTX_SYSTEM_AWARE, core::sync::atomic::Ordering::SeqCst);
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDpiAwarenessContext(
    context: DPI_AWARENESS_CONTEXT,
) -> BOOL {
    // Only the five pseudo-handles -1..-5 are contexts.
    let c = context as isize;
    if !(DPI_CTX_UNAWARE_GDISCALED..=DPI_CTX_UNAWARE).contains(&c) {
        RtlSetLastWin32Error(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as i32;
    }
    PROCESS_DPI_CONTEXT.store(c, core::sync::atomic::Ordering::SeqCst);
    (TRUE) as i32
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
