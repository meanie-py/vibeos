//! SCM driver control interface — Rust flip of base/system/services/driver.c
//! (SCM wave 5; first non-group SERVICE-mirror consumer).
//!
//! Loads/unloads kernel & file-system drivers via Nt{Load,Unload}Driver and
//! reads their running state out of the \Driver / \FileSystem object dirs.
//! ScmStartDriver / ScmControlDriver are the cdecl exports consumed by
//! database.c / rpcserver.c; the three ScmLoadDriver/ScmUnloadDriver/
//! ScmGetDriverStatus helpers were `static` in C and stay private here.

#![allow(non_snake_case)]

use crate::types::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::nt::STATUS_SUCCESS;
use winsrv_sys::consts::SE_LOAD_DRIVER_PRIVILEGE;

// ntstatus.h codes not carried by winsrv-sys.
const STATUS_NO_MORE_ENTRIES: NTSTATUS = 0x8000001Au32 as NTSTATUS;
const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC0000010u32 as NTSTATUS;

// CRT wide-string helpers (msvcrt) — not in the winsrv-sys bindgen surface.
extern "C" {
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
}

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

// L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\" — 52 WCHARs + NUL.
static DRIVER_KEY_PREFIX: [u16; 53] = [
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
    b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
    b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
    b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16,
    b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16, 0,
];

// L"\\Driver" and L"\\FileSystem" object-directory names.
static DIR_DRIVER: [u16; 8] = [
    b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16,
    b'r' as u16, 0,
];
static DIR_FILESYSTEM: [u16; 12] = [
    b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'S' as u16,
    b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0,
];

/// Build a UNICODE_STRING that points at a NUL-terminated static [u16].
#[inline]
unsafe fn init_unicode_static(us: *mut UNICODE_STRING, src: *const u16) {
    RtlInitUnicodeString(us, src);
}

unsafe fn ScmLoadDriver(lpService: PSERVICE) -> DWORD {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut WasPrivilegeEnabled: BOOLEAN = 0; // FALSE
    let mut DriverPath: UNICODE_STRING = core::mem::zeroed();

    /* Build the driver path */
    /* 52 = wcslen(L"\Registry\Machine\System\CurrentControlSet\Services\") */
    let pszDriverPath = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((52 + wcslen((*lpService).lpServiceName) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if pszDriverPath.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy(pszDriverPath, DRIVER_KEY_PREFIX.as_ptr());
    wcscat(pszDriverPath, (*lpService).lpServiceName);

    init_unicode_static(&mut DriverPath, pszDriverPath);

    /* Acquire driver-loading privilege */
    Status = RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        1, // TRUE
        0, // FALSE
        &mut WasPrivilegeEnabled,
    );
    if !nt_success(Status) {
        /* We encountered a failure, exit properly */
        HeapFree(GetProcessHeap(), 0, pszDriverPath as PVOID);
        return RtlNtStatusToDosError(Status);
    }

    Status = NtLoadDriver(&mut DriverPath);

    /* Release driver-loading privilege */
    RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        WasPrivilegeEnabled,
        0, // FALSE
        &mut WasPrivilegeEnabled,
    );

    HeapFree(GetProcessHeap(), 0, pszDriverPath as PVOID);
    RtlNtStatusToDosError(Status)
}

unsafe fn ScmUnloadDriver(lpService: PSERVICE) -> DWORD {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut WasPrivilegeEnabled: BOOLEAN = 0; // FALSE
    let mut DriverPath: UNICODE_STRING = core::mem::zeroed();
    let dwError: DWORD;

    let pszDriverPath = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((52 + wcslen((*lpService).lpServiceName) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if pszDriverPath.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy(pszDriverPath, DRIVER_KEY_PREFIX.as_ptr());
    wcscat(pszDriverPath, (*lpService).lpServiceName);

    init_unicode_static(&mut DriverPath, pszDriverPath);

    /* Acquire driver-unloading privilege */
    Status = RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        1, // TRUE
        0, // FALSE
        &mut WasPrivilegeEnabled,
    );
    if !nt_success(Status) {
        HeapFree(GetProcessHeap(), 0, pszDriverPath as PVOID);
        return RtlNtStatusToDosError(Status);
    }

    Status = NtUnloadDriver(&mut DriverPath);
    if Status == STATUS_INVALID_DEVICE_REQUEST {
        dwError = ERROR_INVALID_SERVICE_CONTROL;
    } else {
        dwError = RtlNtStatusToDosError(Status);
    }

    /* Release driver-unloading privilege */
    RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        WasPrivilegeEnabled,
        0, // FALSE
        &mut WasPrivilegeEnabled,
    );

    HeapFree(GetProcessHeap(), 0, pszDriverPath as PVOID);
    dwError
}

unsafe fn ScmGetDriverStatus(lpService: PSERVICE, lpServiceStatus: LPSERVICE_STATUS) -> DWORD {
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirName: UNICODE_STRING = core::mem::zeroed();
    let mut DirHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Index: ULONG;
    let mut DataLength: ULONG = 0;
    let dwError: DWORD = ERROR_SUCCESS;
    let mut bFound: BOOLEAN = 0; // FALSE
    let dwPreviousState: DWORD;

    /* Zero output buffer if any */
    if !lpServiceStatus.is_null() {
        core::ptr::write_bytes(lpServiceStatus as *mut u8, 0, core::mem::size_of::<SERVICE_STATUS>());
    }

    /* Select the appropriate object directory based on driver type */
    if (*lpService).Status.dwServiceType == SERVICE_KERNEL_DRIVER {
        init_unicode_static(&mut DirName, DIR_DRIVER.as_ptr());
    } else {
        // SERVICE_FILE_SYSTEM_DRIVER
        init_unicode_static(&mut DirName, DIR_FILESYSTEM.as_ptr());
    }

    /* InitializeObjectAttributes(&ObjectAttributes, &DirName, 0, NULL, NULL) */
    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DirName;
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the object directory where loaded drivers are */
    Status = NtOpenDirectoryObject(
        &mut DirHandle,
        DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
        &mut ObjectAttributes,
    );
    if !nt_success(Status) {
        return RtlNtStatusToDosError(Status);
    }

    /* Allocate a buffer big enough for querying the object */
    let BufferLength: ULONG = core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>() as ULONG
        + 2 * MAX_PATH * core::mem::size_of::<u16>() as ULONG;
    let DirInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, BufferLength as SIZE_T)
        as POBJECT_DIRECTORY_INFORMATION;

    /* Now, start browsing entry by entry */
    Index = 0;
    loop {
        Status = NtQueryDirectoryObject(
            DirHandle,
            DirInfo as PVOID,
            BufferLength,
            1, // TRUE — ReturnSingleEntry
            0, // FALSE — RestartScan
            &mut Index,
            &mut DataLength,
        );
        /* End of enumeration, the driver was not found */
        if Status == STATUS_NO_MORE_ENTRIES {
            break;
        }

        /* Other error, fail */
        if !nt_success(Status) {
            break;
        }

        /* Compare names to check whether it matches our driver */
        if _wcsicmp((*lpService).lpServiceName, (*DirInfo).Name.Buffer) == 0 {
            /* That's our driver, bail out! */
            bFound = 1; // TRUE
            break;
        }
    }

    /* Release resources we don't need */
    HeapFree(GetProcessHeap(), 0, DirInfo as PVOID);
    NtClose(DirHandle);

    /* Only quit if there's a failure
     * Not having found the driver is legit!
     * It means the driver was registered as a service, but not loaded
     */
    if !nt_success(Status) && Status != STATUS_NO_MORE_ENTRIES {
        return RtlNtStatusToDosError(Status);
    }

    /* Now, we have two cases:
     * We found the driver: it means it's running
     * We didn't find the driver: it wasn't running
     */
    if bFound != 0 {
        /* Found, return it's running */
        dwPreviousState = (*lpService).Status.dwCurrentState;

        /* It is running */
        (*lpService).Status.dwCurrentState = SERVICE_RUNNING;

        if dwPreviousState == SERVICE_STOPPED {
            /* Make it run if it was stopped before */
            (*lpService).Status.dwWin32ExitCode = ERROR_SUCCESS;
            (*lpService).Status.dwServiceSpecificExitCode = ERROR_SUCCESS;
            (*lpService).Status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
            (*lpService).Status.dwCheckPoint = 0;
            (*lpService).Status.dwWaitHint = 0;
        }

        if (*lpService).Status.dwWin32ExitCode == ERROR_SERVICE_NEVER_STARTED {
            (*lpService).Status.dwWin32ExitCode = ERROR_SUCCESS;
        }
    } else {
        /* Not found, return it's stopped */
        if (*lpService).Status.dwCurrentState == SERVICE_STOP_PENDING {
            /* Stopped successfully */
            (*lpService).Status.dwWin32ExitCode = ERROR_SUCCESS;
            (*lpService).Status.dwCurrentState = SERVICE_STOPPED;
            (*lpService).Status.dwControlsAccepted = 0;
            (*lpService).Status.dwCheckPoint = 0;
            (*lpService).Status.dwWaitHint = 0;
        } else if (*lpService).Status.dwCurrentState == SERVICE_STOPPED {
            /* Don't change the current status */
        } else {
            (*lpService).Status.dwWin32ExitCode = ERROR_GEN_FAILURE;
            (*lpService).Status.dwCurrentState = SERVICE_STOPPED;
            (*lpService).Status.dwControlsAccepted = 0;
            (*lpService).Status.dwCheckPoint = 0;
            (*lpService).Status.dwWaitHint = 0;
        }
    }

    /* Copy service status if required */
    if !lpServiceStatus.is_null() {
        core::ptr::copy_nonoverlapping(
            &(*lpService).Status as *const SERVICE_STATUS,
            lpServiceStatus,
            1,
        );
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmStartDriver(pService: PSERVICE) -> DWORD {
    let dwError = ScmLoadDriver(pService);
    if dwError == ERROR_SUCCESS {
        (*pService).Status.dwCurrentState = SERVICE_RUNNING;
        (*pService).Status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
        (*pService).Status.dwWin32ExitCode = ERROR_SUCCESS;
    }
    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmControlDriver(
    lpService: PSERVICE,
    dwControl: DWORD,
    lpServiceStatus: LPSERVICE_STATUS,
) -> DWORD {
    let mut dwError: DWORD;

    match dwControl {
        SERVICE_CONTROL_STOP => {
            'done: loop {
                /* Check the drivers status */
                dwError = ScmGetDriverStatus(lpService, lpServiceStatus);
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }

                /* Fail, if it is not running */
                if (*lpService).Status.dwCurrentState != SERVICE_RUNNING {
                    dwError = ERROR_INVALID_SERVICE_CONTROL;
                    break 'done;
                }

                /* Unload the driver */
                dwError = ScmUnloadDriver(lpService);
                if dwError == ERROR_INVALID_SERVICE_CONTROL {
                    /* The driver cannot be stopped, mark it non-stoppable */
                    (*lpService).Status.dwControlsAccepted = 0;
                    break 'done;
                }

                /* Make the driver 'stop pending' */
                (*lpService).Status.dwCurrentState = SERVICE_STOP_PENDING;

                /* Check the drivers status again */
                dwError = ScmGetDriverStatus(lpService, lpServiceStatus);
                break 'done;
            }
        }
        SERVICE_CONTROL_INTERROGATE => {
            dwError = ScmGetDriverStatus(lpService, lpServiceStatus);
        }
        _ => {
            dwError = ERROR_INVALID_SERVICE_CONTROL;
        }
    }

    dwError
}
