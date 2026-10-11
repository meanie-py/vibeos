// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5DD76177866D: [u16; 22] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x0054, 0x0068, 0x0065,
    0x006D, 0x0065, 0x0053, 0x0074, 0x006F, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_614A1D97B2BD: [u16; 23] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x0054, 0x0068, 0x0065,
    0x006D, 0x0065, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0045, 0x0076, 0x0065,
    0x006E, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"T\0h\0e\0m\0e\0s\0\0\0") };

#[no_mangle]
pub static mut hThemeStartWaitObject: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hThemeStopWaitObject: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hThemeServiceWaitObject: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hStartEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hStopEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hServiceProcess: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

// WINE_DEFAULT_DEBUG_CHANNEL(shsvcs): debug-channel macro, no Rust declaration emitted.

unsafe extern "C" fn GetThemeServiceProcessHandle() -> HANDLE {
    let mut scm: SC_HANDLE = core::ptr::null_mut();
    let mut service: SC_HANDLE = core::ptr::null_mut();
    let mut status: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut dummy: DWORD = 0;
    let mut ret: HANDLE = core::ptr::null_mut();

    scm = OpenSCManagerW(core::ptr::null_mut(), core::ptr::null_mut(), 0);
    if scm.is_null() {
        ERR!("failed to open service manager\n");
        return core::ptr::null_mut();
    }
    service = OpenServiceW(
        scm,
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        SERVICE_QUERY_STATUS,
    );
    if service.is_null() {
        ERR!("failed to open themes service\n");
        CloseServiceHandle(scm);
        return core::ptr::null_mut();
    }

    if QueryServiceStatusEx(
        service,
        SC_STATUS_PROCESS_INFO,
        core::ptr::addr_of_mut!(status) as LPBYTE,
        core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
        core::ptr::addr_of_mut!(dummy),
    ) == 0
    {
        ERR!("QueryServiceStatusEx failed\n");
        CloseServiceHandle(service);
        CloseServiceHandle(scm);
        return core::ptr::null_mut();
    }

    ret = OpenProcess(SYNCHRONIZE, (FALSE) as BOOL, status.dwProcessId);

    CloseServiceHandle(service);
    CloseServiceHandle(scm);

    return ret;
}

unsafe extern "system" fn ThemeStopCallback(lpParameter: PVOID, TimerOrWaitFired: BOOLEAN) {
    CloseHandle(hServiceProcess);
    UnregisterWait(hThemeServiceWaitObject);

    ThemeWatchForStart();
    ThemeHooksRemove();
}

unsafe extern "system" fn ThemeServiceDiedCallback(lpParameter: PVOID, TimerOrWaitFired: BOOLEAN) {
    let _ = lpParameter;
    let _ = TimerOrWaitFired;
    ResetEvent(hStartEvent);
    ResetEvent(hStopEvent);

    CloseHandle(hServiceProcess);
    UnregisterWait(hThemeStopWaitObject);
    ThemeWatchForStart();
    ThemeHooksRemove();
}

unsafe extern "system" fn ThemeStartCallback(lpParameter: PVOID, TimerOrWaitFired: BOOLEAN) {
    hServiceProcess = GetThemeServiceProcessHandle();

    RegisterWaitForSingleObject(
        core::ptr::addr_of_mut!(hThemeStopWaitObject),
        hStopEvent,
        Some(ThemeStopCallback),
        core::ptr::null_mut(),
        INFINITE as ULONG,
        WT_EXECUTEONLYONCE as ULONG,
    );
    RegisterWaitForSingleObject(
        core::ptr::addr_of_mut!(hThemeServiceWaitObject),
        hServiceProcess,
        Some(ThemeServiceDiedCallback),
        core::ptr::null_mut(),
        INFINITE as ULONG,
        WT_EXECUTEONLYONCE as ULONG,
    );

    ThemeHooksInstall();
}

#[no_mangle]
pub unsafe extern "system" fn ThemeWatchForStart() -> BOOL {
    hStartEvent = CreateEventW(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, RUSTCONV_WSTR_614A1D97B2BD.as_ptr());
    hStopEvent = CreateEventW(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, RUSTCONV_WSTR_5DD76177866D.as_ptr());

    RegisterWaitForSingleObject(
        core::ptr::addr_of_mut!(hThemeStartWaitObject),
        hStartEvent,
        Some(ThemeStartCallback),
        core::ptr::null_mut(),
        INFINITE as ULONG,
        WT_EXECUTEONLYONCE as ULONG,
    );

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ThemeWaitForServiceReady(dwTimeout: DWORD) -> DWORD {
    let mut scm: SC_HANDLE = core::ptr::null_mut();
    let mut service: SC_HANDLE = core::ptr::null_mut();
    let mut status: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);
    let start_time: DWORD = GetTickCount();

    scm = OpenSCManagerW(core::ptr::null(), core::ptr::null(), 0);
    if scm.is_null() {
        ERR!("failed to open service manager\n");
        return FALSE;
    }
    service = OpenServiceW(
        scm,
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        SERVICE_QUERY_STATUS,
    );
    if service.is_null() {
        ERR!("failed to open themes service\n");
        CloseServiceHandle(scm);
        return FALSE;
    }

    loop {
        let mut dummy: DWORD = 0;

        if QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            core::ptr::addr_of_mut!(status) as LPBYTE,
            core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
            core::ptr::addr_of_mut!(dummy),
        ) == 0
        {
            break;
        }
        if status.dwCurrentState == SERVICE_RUNNING {
            ret = (TRUE as i32);
            break;
        }
        if status.dwCurrentState != SERVICE_START_PENDING {
            break;
        }

        if GetTickCount().wrapping_sub(start_time) > dwTimeout {
            break;
        }
        Sleep(100);
    }

    CloseServiceHandle(service);
    CloseServiceHandle(scm);
    return ret as DWORD;
}
