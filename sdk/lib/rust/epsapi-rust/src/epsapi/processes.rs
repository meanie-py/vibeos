use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn PsaCaptureProcessesAndThreads(
    ProcessesAndThreads: *mut PSYSTEM_PROCESS_INFORMATION,
) -> NTSTATUS {
    let mut pInfoBuffer: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut nSize: SIZE_T = 0x8000 as SIZE_T;
    let mut Status: NTSTATUS;

    if ProcessesAndThreads.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }

    loop {
        let mut pTmp: PVOID;

        PsaiFree(pInfoBuffer as *mut core::ffi::c_void);
        pTmp = PsaiMalloc(nSize) as PVOID;

        if pTmp.is_null() {
            DPRINT!(FAILED_WITH_STATUS, "PsaiMalloc", STATUS_NO_MEMORY);
            Status = STATUS_NO_MEMORY;
            break;
        }

        pInfoBuffer = pTmp as PSYSTEM_PROCESS_INFORMATION;

        Status = NtQuerySystemInformation(
            SystemProcessInformation,
            pInfoBuffer as PVOID,
            nSize as ULONG,
            core::ptr::null_mut::<ULONG>(),
        );

        nSize = nSize.wrapping_mul(2 as SIZE_T);

        if !(Status == STATUS_INFO_LENGTH_MISMATCH) {
            break;
        }
    }

    if !NT_SUCCESS!(Status) {
        DPRINT!(FAILED_WITH_STATUS, "NtQuerySystemInformation", Status);
        return Status;
    }

    *ProcessesAndThreads = pInfoBuffer;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PsaWalkProcessesAndThreads(
    mut ProcessesAndThreads: PSYSTEM_PROCESS_INFORMATION,
    mut ProcessCallback: PPROC_ENUM_ROUTINE,
    mut ProcessCallbackContext: PVOID,
    mut ThreadCallback: PTHREAD_ENUM_ROUTINE,
    mut ThreadCallbackContext: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;

    if ProcessCallback.is_none() && ThreadCallback.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = STATUS_SUCCESS;

    ProcessesAndThreads = PsaWalkFirstProcess(ProcessesAndThreads);

    'Bail: {
        /* scan the process list */
        loop {
            if ProcessCallback.is_some() {
                Status = (ProcessCallback.unwrap_unchecked())(ProcessesAndThreads, ProcessCallbackContext);

                if !NT_SUCCESS!(Status) {
                    break;
                }
            }

            /* if the caller provided a thread callback */
            if ThreadCallback.is_some() {
                let mut i: ULONG = 0;
                let mut pCurThread: PSYSTEM_THREAD_INFORMATION =
                    core::ptr::null_mut::<_SYSTEM_THREAD_INFORMATION>();

                /* scan the current process's thread list */
                i = 0;
                pCurThread = PsaWalkFirstThread(ProcessesAndThreads);
                while i < (*ProcessesAndThreads).NumberOfThreads {
                    Status = (ThreadCallback.unwrap_unchecked())(pCurThread, ThreadCallbackContext);

                    if !NT_SUCCESS!(Status) {
                        break 'Bail;
                    }

                    i = i.wrapping_add(1);
                    pCurThread = PsaWalkNextThread(pCurThread);
                }
            }

            /* move to the next process */
            ProcessesAndThreads = PsaWalkNextProcess(ProcessesAndThreads);
            if ProcessesAndThreads.is_null() {
                break;
            }
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PsaEnumerateProcessesAndThreads(
    ProcessCallback: PPROC_ENUM_ROUTINE,
    ProcessCallbackContext: PVOID,
    ThreadCallback: PTHREAD_ENUM_ROUTINE,
    ThreadCallbackContext: PVOID,
) -> NTSTATUS {
    let mut pInfoBuffer: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if ProcessCallback.is_none() && ThreadCallback.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    /* get the processes and threads list */
    'Bail: {
        Status = PsaCaptureProcessesAndThreads(core::ptr::addr_of_mut!(pInfoBuffer));

        if !NT_SUCCESS!(Status) {
            break 'Bail;
        }

        /* walk the processes and threads list */
        Status = PsaWalkProcessesAndThreads(
            pInfoBuffer,
            ProcessCallback,
            ProcessCallbackContext,
            ThreadCallback,
            ThreadCallbackContext,
        );
    }

    PsaFreeCapture(pInfoBuffer as PVOID);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PsaFreeCapture(Capture: PVOID) {
    PsaiFree(Capture);
}

#[no_mangle]
pub unsafe extern "system" fn PsaWalkProcesses(
    ProcessesAndThreads: PSYSTEM_PROCESS_INFORMATION,
    Callback: PPROC_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    return PsaWalkProcessesAndThreads(
        ProcessesAndThreads,
        Callback,
        CallbackContext,
        None,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn PsaWalkThreads(
    ProcessesAndThreads: PSYSTEM_PROCESS_INFORMATION,
    Callback: PTHREAD_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    return PsaWalkProcessesAndThreads(
        ProcessesAndThreads,
        None,
        core::ptr::null_mut(),
        Callback,
        CallbackContext,
    );
}
#[no_mangle]
pub unsafe extern "system" fn PsaEnumerateProcesses(Callback: PPROC_ENUM_ROUTINE, CallbackContext: PVOID) -> NTSTATUS {
    return PsaEnumerateProcessesAndThreads(Callback, CallbackContext, None, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn PsaEnumerateThreads(
    Callback: PTHREAD_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    return PsaEnumerateProcessesAndThreads(
        None,
        core::ptr::null_mut(),
        Callback,
        CallbackContext,
    );
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkFirstProcess(
    ProcessesAndThreads: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_PROCESS_INFORMATION {
    ProcessesAndThreads
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkFirstProcess(
    ProcessesAndThreads: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_PROCESS_INFORMATION {
    ProcessesAndThreads
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkNextProcess(
    mut CurrentProcess: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_PROCESS_INFORMATION {
    if (*CurrentProcess).NextEntryOffset == 0 {
        return core::ptr::null_mut();
    } else {
        return (CurrentProcess as ULONG_PTR)
            .wrapping_add((*CurrentProcess).NextEntryOffset as ULONG_PTR)
            as PSYSTEM_PROCESS_INFORMATION;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkNextProcess(
    mut CurrentProcess: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_PROCESS_INFORMATION {
    if (*CurrentProcess).NextEntryOffset == 0 {
        return core::ptr::null_mut();
    } else {
        return (CurrentProcess as ULONG_PTR)
            .wrapping_add((*CurrentProcess).NextEntryOffset as ULONG_PTR)
            as PSYSTEM_PROCESS_INFORMATION;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkFirstThread(
    mut CurrentProcess: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_THREAD_INFORMATION {
    static mut nOffsetOfThreads: SIZE_T = 0;

    /* get the offset of the Threads field */
    nOffsetOfThreads = ::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as SIZE_T;

    return (CurrentProcess as ULONG_PTR).wrapping_add(nOffsetOfThreads as ULONG_PTR) as PSYSTEM_THREAD_INFORMATION;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkFirstThread(
    mut CurrentProcess: PSYSTEM_PROCESS_INFORMATION,
) -> PSYSTEM_THREAD_INFORMATION {
    static mut nOffsetOfThreads: SIZE_T = 0;

    /* get the offset of the Threads field */
    nOffsetOfThreads = ::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as SIZE_T;

    return (CurrentProcess as ULONG_PTR).wrapping_add(nOffsetOfThreads as ULONG_PTR) as PSYSTEM_THREAD_INFORMATION;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkNextThread(
    mut CurrentThread: PSYSTEM_THREAD_INFORMATION,
) -> PSYSTEM_THREAD_INFORMATION {
    return (CurrentThread as usize).wrapping_add(
        (::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as usize)
            .wrapping_add(::core::mem::size_of::<SYSTEM_THREAD_INFORMATION>() as usize)
            .wrapping_sub(::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as usize),
    ) as PSYSTEM_THREAD_INFORMATION;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkNextThread(
    mut CurrentThread: PSYSTEM_THREAD_INFORMATION,
) -> PSYSTEM_THREAD_INFORMATION {
    return (CurrentThread as usize).wrapping_add(
        (::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as usize)
            .wrapping_add(::core::mem::size_of::<SYSTEM_THREAD_INFORMATION>() as usize)
            .wrapping_sub(::core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as usize),
    ) as PSYSTEM_THREAD_INFORMATION;
}
