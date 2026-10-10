// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_59A7FF375010: [u16; 7] = [0x0077u16, 0x0064, 0x006D, 0x0061, 0x0075, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_7EEFF07B597F: [u16; 9] = [0x0073u16, 0x0079, 0x0073, 0x0061, 0x0075, 0x0064, 0x0069, 0x006F, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn WaitForService(hService: SC_HANDLE, RetryCount: ULONG) -> BOOL {
    let mut Index: ULONG = 0 as ULONG;
    let mut dwSize: DWORD = 0;
    let mut Info: SERVICE_STATUS_PROCESS = core::mem::zeroed();

    loop {
        if !(QueryServiceStatusEx(
            hService,
            SC_STATUS_PROCESS_INFO,
            core::ptr::addr_of_mut!(Info) as LPBYTE,
            core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
            core::ptr::addr_of_mut!(dwSize),
        ) != 0)
        {
            DPRINT!("QueryServiceStatusEx failed %x\n", GetLastError());
            break;
        }

        if Info.dwCurrentState == SERVICE_RUNNING as DWORD {
            return TRUE as BOOL;
        }

        Sleep(1000);

        let fresh = Index;
        Index = Index.wrapping_add(1);
        if !(fresh < RetryCount) {
            break;
        }
    }

    DPRINT!("Timeout while waiting for service to become ready %p\n", hService);

    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn StartAudioService(
    hSCManager: SC_HANDLE,
    ServiceName: LPWSTR,
    RetryCount: ULONG,
) -> BOOL {
    let mut hService: SC_HANDLE;
    let mut ret: BOOL;

    hService = OpenService(hSCManager, ServiceName, SERVICE_ALL_ACCESS);
    if hService.is_null() {
        DPRINT!(b"Failed to open service %S %x\n\0".as_ptr() as *const i8, ServiceName, GetLastError());
        return FALSE as BOOL;
    }

    if StartService(hService, 0, core::ptr::null_mut()) == 0 {
        DPRINT!(b"Failed to start service %S %x\n\0".as_ptr() as *const i8, ServiceName, GetLastError());
        CloseServiceHandle(hService);
        return FALSE as BOOL;
    }

    ret = WaitForService(hService, RetryCount);

    CloseServiceHandle(hService);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn StartSystemAudioServices() -> BOOL {
    let mut hSCManager: SC_HANDLE;

    DPRINT!("Starting system audio services\n");

    hSCManager = OpenSCManager(core::ptr::null_mut(), core::ptr::null_mut(), SC_MANAGER_CONNECT);
    if hSCManager.is_null() {
        DPRINT!("Failed to open service manager %x\n", GetLastError());
        return FALSE as BOOL;
    }

    DPRINT!("Starting sysaudio service\n");
    StartAudioService(hSCManager, RUSTCONV_WSTR_7EEFF07B597F.as_ptr() as LPWSTR, 20);
    DPRINT!("Starting wdmaud service\n");
    StartAudioService(hSCManager, RUSTCONV_WSTR_59A7FF375010.as_ptr() as LPWSTR, 20);

    CloseServiceHandle(hSCManager);
    return TRUE as BOOL;
}
