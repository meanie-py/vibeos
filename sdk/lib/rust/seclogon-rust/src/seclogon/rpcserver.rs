// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0FD6FEFD9900: [u16; 9] = [0x0073u16, 0x0065, 0x0063, 0x006C, 0x006F, 0x0067, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;

#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len) as *mut core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut ::core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 as DWORD, ptr as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn StartRpcServer() -> DWORD {
    let mut Status: NTSTATUS = 0;
    Status = (*lpServiceGlobals)
        .StartRpcServer
        .unwrap_unchecked()(
        RUSTCONV_WSTR_0FD6FEFD9900.as_ptr(),
        ISeclogon_v1_0_s_ifspec,
    ) as NTSTATUS;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"base/services/seclogon/rpcserver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"StartRpcServer\0" as *const u8 as *const ::core::ffi::c_char,
            37 as ::core::ffi::c_int,
            b"StartRpcServer returned 0x%08lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            Status,
        );
    }
    return RtlNtStatusToDosError(Status) as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn StopRpcServer() -> DWORD {
    let mut Status: NTSTATUS = 0;
    Status = (*lpServiceGlobals)
        .StopRpcServer
        .unwrap_unchecked()(ISeclogon_v1_0_s_ifspec) as NTSTATUS;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"base/services/seclogon/rpcserver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"StopRpcServer\0" as *const u8 as *const ::core::ffi::c_char,
            49 as ::core::ffi::c_int,
            b"StopRpcServer returned 0x%08lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            Status,
        );
    }
    return RtlNtStatusToDosError(Status) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SeclCreateProcessWithLogonW(
    hBinding: handle_t,
    pRequest: *mut SECL_REQUEST,
    pResponse: *mut SECL_RESPONSE,
) {
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInfo: PROCESS_INFORMATION = core::mem::zeroed();
    let mut ProfileInfo: PROFILEINFOW = core::mem::zeroed();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut hTargetProcessHandle: HANDLE = core::ptr::null_mut();
    let mut dwError: ULONG = ERROR_SUCCESS as ULONG;
    let mut rc: BOOL = 0;

    'done: {
        if !pRequest.is_null() {
        }

        hTargetProcessHandle = OpenProcess(
            PROCESS_DUP_HANDLE as DWORD,
            FALSE as BOOL,
            (*pRequest).dwProcessId,
        );
        if hTargetProcessHandle.is_null() {
            dwError = GetLastError() as ULONG;
            WARN!("OpenProcess() failed with Error %lu\n", dwError);
            break 'done;
        }

        ZeroMemory!(
            core::ptr::addr_of_mut!(ProfileInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<PROFILEINFOW>(),
        );

        /* Logon */
        rc = LogonUser(
            (*pRequest).Username as LPWSTR,
            (*pRequest).Domain as LPWSTR,
            (*pRequest).Password as LPWSTR,
            LOGON32_LOGON_INTERACTIVE as DWORD,
            LOGON32_PROVIDER_DEFAULT as DWORD,
            core::ptr::addr_of_mut!(hToken),
        );
        if rc == FALSE as BOOL {
            dwError = GetLastError() as ULONG;
            WARN!("LogonUser() failed with Error %lu\n", dwError);
            break 'done;
        }

        /* Load the user profile */
        if ((*pRequest).dwLogonFlags & (LOGON_WITH_PROFILE as DWORD)) != 0 {
            ProfileInfo.dwSize = core::mem::size_of::<PROFILEINFOW>() as DWORD;
            ProfileInfo.lpUserName = (*pRequest).Username as LPWSTR;

            rc = LoadUserProfileW(
                hToken,
                core::ptr::addr_of_mut!(ProfileInfo),
            );
            if rc == FALSE as BOOL {
                dwError = GetLastError() as ULONG;
                WARN!("LoadUserProfile() failed with Error %lu\n", dwError);
                break 'done;
            }
        }

        /* Initialize the startup information */
        ZeroMemory!(
            core::ptr::addr_of_mut!(StartupInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<STARTUPINFOW>(),
        );
        StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;

        /* FIXME: Get startup info from the caller */

        /* Initialize the process information */
        ZeroMemory!(
            core::ptr::addr_of_mut!(ProcessInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<PROCESS_INFORMATION>(),
        );

        /* Create Process */
        rc = CreateProcessAsUserW(
            hToken,
            (*pRequest).ApplicationName as LPCWSTR,
            (*pRequest).CommandLine as LPWSTR,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(), // lpProcessAttributes
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(), // lpThreadAttributes
            FALSE as BOOL, // bInheritHandles
            (*pRequest).dwCreationFlags,
            (*pRequest).Environment as PVOID, // lpEnvironment
            (*pRequest).CurrentDirectory as LPCWSTR,
            core::ptr::addr_of_mut!(StartupInfo),
            core::ptr::addr_of_mut!(ProcessInfo),
        );
        if rc == FALSE as BOOL {
            dwError = GetLastError() as ULONG;
            WARN!("CreateProcessAsUser() failed with Error %lu\n", dwError);
            break 'done;
        }

        /* Return process info to the caller */
        if !pResponse.is_null() {
            DuplicateHandle(
                GetCurrentProcess(),
                ProcessInfo.hProcess,
                hTargetProcessHandle,
                core::ptr::addr_of_mut!((*pResponse).hProcess) as PHANDLE,
                0 as DWORD,
                FALSE as BOOL,
                DUPLICATE_SAME_ACCESS as DWORD,
            );

            DuplicateHandle(
                GetCurrentProcess(),
                ProcessInfo.hThread,
                hTargetProcessHandle,
                core::ptr::addr_of_mut!((*pResponse).hThread) as PHANDLE,
                0 as DWORD,
                FALSE as BOOL,
                DUPLICATE_SAME_ACCESS as DWORD,
            );

            (*pResponse).dwProcessId = ProcessInfo.dwProcessId;
            (*pResponse).dwThreadId = ProcessInfo.dwThreadId;
        }
    }

    // done:
    if !hTargetProcessHandle.is_null() {
        CloseHandle(hTargetProcessHandle);
    }

    if !ProcessInfo.hThread.is_null() {
        CloseHandle(ProcessInfo.hThread);
    }

    if !ProcessInfo.hProcess.is_null() {
        CloseHandle(ProcessInfo.hProcess);
    }

    if !ProfileInfo.hProfile.is_null() {
        UnloadUserProfile(hToken, ProfileInfo.hProfile);
    }

    if !hToken.is_null() {
        CloseHandle(hToken);
    }

    if !pResponse.is_null() {
        (*pResponse).dwError = dwError as DWORD;
    }
}
