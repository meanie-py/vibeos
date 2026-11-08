use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn StopWatchMode() -> DWORD {
    // FIXME("() stub!\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn StopWatchFlush() {
    // FIXME("() stub!\n");
}

#[no_mangle]
pub unsafe extern "system" fn StopWatchW(
    dwClass: DWORD,
    lpszStr: LPCWSTR,
    dwUnknown: DWORD,
    dwMode: DWORD,
    dwTimeStamp: DWORD,
) -> DWORD {
    // FIXME("(%d,%s,%d,%d,%d) stub!\n", dwClass, debugstr_w(lpszStr), dwUnknown, dwMode, dwTimeStamp);
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn StopWatchA(
    dwClass: DWORD,
    lpszStr: LPCSTR,
    dwUnknown: DWORD,
    dwMode: DWORD,
    dwTimeStamp: DWORD,
) -> DWORD {
    let mut retval: DWORD = 0;
    let mut szStrW: _UNICODE_STRING = core::mem::zeroed();

    if !lpszStr.is_null() {
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(szStrW), lpszStr);
    } else {
        szStrW.Buffer = core::ptr::null_mut();
    }

    retval = StopWatchW(
        dwClass,
        szStrW.Buffer as LPCWSTR,
        dwUnknown,
        dwMode,
        dwTimeStamp,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(szStrW));
    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn StopWatch_TimerHandler(
    hWnd: HWND,
    pvUnknown1: PVOID,
    bUnknown2: BOOL,
    pClassWnd: *mut HWND,
) {
    // FIXME("(%p,%p,%d,%p) stub!\n", hWnd, pvUnknown1, bUnknown2 ,pClassWnd);
}

#[no_mangle]
pub unsafe extern "system" fn StopWatch_MarkFrameStart(lpszName: LPCSTR) {
    // FIXME("(%s) stub!\n", debugstr_a(lpszName));
}

#[no_mangle]
pub unsafe extern "system" fn StopWatch_MarkJavaStop(
    lpszEvent: LPCWSTR,
    hWnd: HWND,
    dwReserved: DWORD,
) {
    // FIXME("(%s,%p,0x%08x) stub!\n", debugstr_w(lpszEvent), hWnd, dwReserved);
}

#[no_mangle]
pub unsafe extern "system" fn GetPerfTime() -> DWORD {
    static mut iCounterFreq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut iCounter: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    if (*core::ptr::addr_of!(iCounterFreq)).QuadPart == 0 {
        QueryPerformanceFrequency(core::ptr::addr_of_mut!(iCounterFreq));
    }

    QueryPerformanceCounter(core::ptr::addr_of_mut!(iCounter));
    (*core::ptr::addr_of_mut!(iCounter)).QuadPart = (*core::ptr::addr_of!(iCounter)).QuadPart
        * 1000
        / (*core::ptr::addr_of!(iCounterFreq)).QuadPart;
    return (*core::ptr::addr_of!(iCounter)).u.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn StopWatch_SetMsgLastLocation(dwUnknown: DWORD) -> DWORD {
    // FIXME("(%d) stub!\n", dwUnknown);
    dwUnknown
}
