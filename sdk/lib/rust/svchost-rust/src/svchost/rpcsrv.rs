// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_B9AD6B89DC37: [u16; 7] = [0x005Cu16, 0x0050, 0x0049, 0x0050, 0x0045, 0x005C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut RpcpNumInstances: LONG = 0;

#[no_mangle]
pub static mut RpcpCriticalSection: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};
#[no_mangle]
pub unsafe extern "system" fn RpcpInitRpcServer() -> NTSTATUS {
    /* Clear the reference count and initialize the critical section */
    RpcpNumInstances = 0 as LONG;
    return RtlInitializeCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection) as PRTL_CRITICAL_SECTION);
}

#[no_mangle]
pub unsafe extern "system" fn RpcpStopRpcServer(IfSpec: RPC_IF_HANDLE) -> NTSTATUS {
    let mut rpcStatus: RPC_STATUS;

    /* Unregister the interface */
    rpcStatus = RpcServerUnregisterIf(IfSpec, core::ptr::null_mut::<UUID>(), TRUE);

    /* Acquire the lock while we dereference the RPC services */
    EnterCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));
    RpcpNumInstances -= 1;
    if RpcpNumInstances == 0 {
        /* All RPC services stopped, rundown the server */
        RpcMgmtStopServerListening(core::ptr::null_mut());
        RpcMgmtWaitServerListen();
    }

    /* Release the lock and return the unregister result */
    LeaveCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));
    return I_RpcMapWin32Status(rpcStatus) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn RpcpStopRpcServerEx(IfSpec: RPC_IF_HANDLE) -> NTSTATUS {
    let mut rpcStatus: RPC_STATUS;

    /* Unregister the interface */
    rpcStatus = RpcServerUnregisterIfEx(IfSpec, core::ptr::null_mut(), TRUE as ::core::ffi::c_int);

    /* Acquire the lock while we dereference the RPC services */
    EnterCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));
    RpcpNumInstances -= 1;
    if RpcpNumInstances == 0 {
        /* All RPC services stopped, rundown the server */
        RpcMgmtStopServerListening(core::ptr::null_mut());
        RpcMgmtWaitServerListen();
    }

    /* Release the lock and return the unregister result */
    LeaveCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));
    return I_RpcMapWin32Status(rpcStatus);
}

#[no_mangle]
pub unsafe extern "system" fn RpcpAddInterface(
    IfName: PCWSTR,
    IfSpec: RPC_IF_HANDLE,
) -> NTSTATUS {
    let mut endpointName: PWCHAR;
    let mut ntStatus: NTSTATUS;
    let mut rpcStatus: RPC_STATUS;

    /* Allocate space for the interface name and the \\PIPE\\ prefix */
    endpointName = LocalAlloc(
        0 as UINT,
        (core::mem::size_of::<WCHAR>() as SIZE_T)
            .wrapping_mul(wcslen(IfName as *const wchar_t) as SIZE_T)
            .wrapping_add(16 as SIZE_T),
    ) as PWCHAR;
    if !endpointName.is_null() {
        /* Copy the prefix, and then the interface name */
        wcscpy(
            endpointName as *mut wchar_t,
            RUSTCONV_WSTR_B9AD6B89DC37.as_ptr() as *const wchar_t,
        );
        wcscat(endpointName as *mut wchar_t, IfName as *const wchar_t);

        /* Create a named pipe endpoint with this name */
        rpcStatus = RpcServerUseProtseqEpW(
            RUSTCONV_WSTR_12A6BF85117F.as_ptr() as RPC_WSTR,
            RPC_C_PROTSEQ_MAX_REQS_DEFAULT as ::core::ffi::c_uint,
            endpointName as RPC_WSTR,
            core::ptr::null_mut(),
        );
        if rpcStatus != RPC_S_OK as RPC_STATUS
            && rpcStatus != RPC_S_DUPLICATE_ENDPOINT as RPC_STATUS
        {
            /* We couldn't create it, or it already existed... */
            DbgPrint(
                b"RpcServerUseProtseqW failed! rpcstatus = %u\n\0" as *const u8 as PCSTR,
                rpcStatus,
            );
        } else {
            /* It worked, register an interface on this endpoint now */
            rpcStatus = RpcServerRegisterIf(
                IfSpec,
                0 as *mut UUID,
                0 as *mut ::core::ffi::c_void,
            );
        }

        /* In both success and failure, free the name, and convert the status */
        LocalFree(endpointName as HLOCAL);
        ntStatus = I_RpcMapWin32Status(rpcStatus) as NTSTATUS;
    } else {
        /* No memory, bail out */
        ntStatus = STATUS_NO_MEMORY as NTSTATUS;
    }

    /* Return back to the caller */
    return ntStatus;
}
#[no_mangle]
pub unsafe extern "system" fn RpcpStartRpcServer(IfName: PCWSTR, IfSpec: RPC_IF_HANDLE) -> NTSTATUS {
    let mut ntStatus: NTSTATUS = 0;

    /* Acquire the lock while we instantiate a new interface */
    EnterCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));

    /* Add this interface to the service */
    ntStatus = RpcpAddInterface(IfName, IfSpec);
    if ntStatus == 0 {
        /* Increment the reference count to see if this was the first interface */
        RpcpNumInstances += 1;
        if RpcpNumInstances == 1 as LONG {
            /* It was, so put the server into listening mode now */
            ntStatus = RpcServerListen(
                1 as ::core::ffi::c_uint,
                12345 as ::core::ffi::c_uint,
                TRUE as ::core::ffi::c_uint,
            ) as NTSTATUS;
            if ntStatus == RPC_S_ALREADY_LISTENING as NTSTATUS {
                ntStatus = STATUS_SUCCESS as NTSTATUS;
            }
        }
    }

    /* Release the lock and return back the result to the caller */
    LeaveCriticalSection(core::ptr::addr_of_mut!(RpcpCriticalSection));
    return I_RpcMapWin32Status(ntStatus as RPC_STATUS) as NTSTATUS;
}
