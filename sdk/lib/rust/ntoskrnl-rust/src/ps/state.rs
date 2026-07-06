use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::{NonPagedPool};
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    pub static PsThreadType: POBJECT_TYPE;
    pub static PsProcessType: POBJECT_TYPE;
    fn KeAlertResumeThread(thread: PKTHREAD) -> ULONG;
    fn KeForceResumeThread(thread: PKTHREAD) -> ULONG;
    fn ExAllocatePoolWithQuotaTag(PoolType: ULONG, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
}

const IO_NO_INCREMENT: KPRIORITY = 0;
const TAG_PS_APC: ULONG = u32::from_le_bytes(*b"cApP");
const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017_u32 as i32;
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: u32 = 0x00000008;

unsafe extern "system" fn PspQueueApcSpecialApc(
    apc: PKAPC,
    _normal_routine: *mut PKNORMAL_ROUTINE,
    _normal_context: *mut PVOID,
    _system_argument1: *mut PVOID,
    _system_argument2: *mut PVOID,
) {
    ExFreePool(apc as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn PsResumeThread(
    thread: PETHREAD,
    previous_count: *mut ULONG,
) -> NTSTATUS {
    let thread = opaque_ptr(thread);
    let previous_count = opaque_ptr(previous_count);
    let old_count = KeResumeThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);

    if !previous_count.is_null() {
        *previous_count = old_count;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PsSuspendThread(
    thread: PETHREAD,
    previous_count: *mut ULONG,
) -> NTSTATUS {
    let thread = opaque_ptr(thread);
    let previous_count = opaque_ptr(previous_count);
    let mut status = STATUS_SUCCESS;
    let mut old_count = 0u32;

    if thread == PsGetCurrentThread() {
        old_count = KeSuspendThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);
    } else {
        if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*thread).RundownProtect)) != 0 {
            if (*thread).__bindgen_anon_7.__bindgen_anon_1.Terminated() != 0 {
                status = STATUS_THREAD_IS_TERMINATING;
            } else {
                old_count = KeSuspendThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);

                if (*thread).__bindgen_anon_7.__bindgen_anon_1.Terminated() != 0 {
                    KeForceResumeThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);
                    status = STATUS_THREAD_IS_TERMINATING;
                    old_count = 0;
                }
            }

            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*thread).RundownProtect));
        } else {
            status = STATUS_THREAD_IS_TERMINATING;
        }
    }

    if !previous_count.is_null() {
        *previous_count = old_count;
    }
    status
}

unsafe fn PsResumeProcess(process: PEPROCESS) -> NTSTATUS {
    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect)) == 0 {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    let mut thread = PsGetNextProcessThread(process, core::ptr::null_mut());
    while !thread.is_null() {
        KeResumeThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);
        thread = PsGetNextProcessThread(process, thread);
    }

    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect));
    STATUS_SUCCESS
}

unsafe fn PsSuspendProcess(process: PEPROCESS) -> NTSTATUS {
    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect)) == 0 {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    let mut thread = PsGetNextProcessThread(process, core::ptr::null_mut());
    while !thread.is_null() {
        PsSuspendThread(thread, core::ptr::null_mut());
        thread = PsGetNextProcessThread(process, thread);
    }

    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtAlertThread(thread_handle: HANDLE) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let previous_mode = ExGetPreviousMode();
    let mut thread: PETHREAD = core::ptr::null_mut();

    let status = ObReferenceObjectByHandle(
        thread_handle as PVOID,
        THREAD_SUSPEND_RESUME,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        KeAlertThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD, previous_mode);
        ObDereferenceObject(thread as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtAlertResumeThread(
    thread_handle: HANDLE,
    suspend_count: *mut ULONG,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let suspend_count = opaque_ptr(suspend_count);
    let previous_mode = ExGetPreviousMode();
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let previous_state: ULONG;

    if !suspend_count.is_null() && previous_mode != 0 {
        if let Err(s) = probe_write_ulong(suspend_count, previous_mode) {
            return s;
        }
    }

    status = ObReferenceObjectByHandle(
        thread_handle as PVOID, THREAD_SUSPEND_RESUME,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        previous_state = KeAlertResumeThread(core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD);
        ObDereferenceObject(thread as PVOID);

        if !suspend_count.is_null() {
            if let Err(s) = probe_write_user(suspend_count, previous_state, previous_mode) {
                return s;
            }
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtResumeThread(
    thread_handle: HANDLE,
    suspend_count: *mut ULONG,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let suspend_count = opaque_ptr(suspend_count);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut prev: ULONG = 0;
    let previous_mode = ExGetPreviousMode();
    let mut status: NTSTATUS;

    if !suspend_count.is_null() && previous_mode != 0 {
        if let Err(s) = probe_write_ulong(suspend_count, previous_mode) {
            return s;
        }
    }

    status = ObReferenceObjectByHandle(
        thread_handle as PVOID, THREAD_SUSPEND_RESUME,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    status = PsResumeThread(thread, &mut prev);

    if !suspend_count.is_null() {
        if let Err(s) = probe_write_user(suspend_count, prev, previous_mode) {
            return s;
        }
    }

    ObDereferenceObject(thread as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSuspendThread(
    thread_handle: HANDLE,
    previous_suspend_count: *mut ULONG,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let previous_suspend_count = opaque_ptr(previous_suspend_count);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut prev: ULONG = 0;
    let previous_mode = ExGetPreviousMode();
    let mut status: NTSTATUS;

    if !previous_suspend_count.is_null() && previous_mode != 0 {
        if let Err(s) = probe_write_ulong(previous_suspend_count, previous_mode) {
            return s;
        }
    }

    status = ObReferenceObjectByHandle(
        thread_handle as PVOID, THREAD_SUSPEND_RESUME,
        PsThreadType,
        previous_mode,
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    status = PsSuspendThread(thread, &mut prev);
    ObDereferenceObject(thread as PVOID);

    if status < 0 {
        return status;
    }

    if !previous_suspend_count.is_null() {
        if let Err(s) = probe_write_user(previous_suspend_count, prev, previous_mode) {
            return s;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSuspendProcess(process_handle: HANDLE) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let previous_mode = ExGetPreviousMode();
    let mut process: PEPROCESS = core::ptr::null_mut();

    let mut status = ObReferenceObjectByHandle(
        process_handle as PVOID,
        PROCESS_SUSPEND_RESUME,
        PsProcessType,
        previous_mode,
        &mut process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = PsSuspendProcess(process);
        ObDereferenceObject(process as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtResumeProcess(process_handle: HANDLE) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let previous_mode = ExGetPreviousMode();
    let mut process: PEPROCESS = core::ptr::null_mut();

    let mut status = ObReferenceObjectByHandle(
        process_handle as PVOID,
        PROCESS_SUSPEND_RESUME,
        PsProcessType,
        previous_mode,
        &mut process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = PsResumeProcess(process);
        ObDereferenceObject(process as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtTestAlert() -> NTSTATUS {
    if KeTestAlertThread(ExGetPreviousMode()) != 0 {
        STATUS_ALERTED
    } else {
        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtQueueApcThreadEx(
    thread_handle: HANDLE,
    _user_apc_reserve_handle: HANDLE,
    apc_routine: PKNORMAL_ROUTINE,
    normal_context: PVOID,
    system_argument1: PVOID,
    system_argument2: PVOID,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let _user_apc_reserve_handle = opaque_ptr(_user_apc_reserve_handle);
    let normal_context = opaque_ptr(normal_context);
    let system_argument1 = opaque_ptr(system_argument1);
    let system_argument2 = opaque_ptr(system_argument2);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut status = STATUS_SUCCESS;


    status = ObReferenceObjectByHandle(
        thread_handle as PVOID,
        THREAD_SET_CONTEXT,
        PsThreadType,
        ExGetPreviousMode(),
        &mut thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    if (*thread).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 {
        status = STATUS_INVALID_HANDLE;
        ObDereferenceObject(thread as PVOID);
        return status;
    }

    let apc = ExAllocatePoolWithQuotaTag(
        NonPagedPool as u32 | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE,
        core::mem::size_of::<_KAPC>() as SIZE_T,
        TAG_PS_APC,
    );

    if apc.is_null() {
        status = STATUS_NO_MEMORY;
        ObDereferenceObject(thread as PVOID);
        return status;
    }

    KeInitializeApc(
        apc as PKAPC,
        core::ptr::addr_of_mut!((*thread).Tcb) as PKTHREAD,
        _KAPC_ENVIRONMENT_OriginalApcEnvironment,
        core::mem::transmute(PspQueueApcSpecialApc as usize),
        None,
        apc_routine,
        UserMode,
        normal_context,
    );

    if KeInsertQueueApc(
        apc as PKAPC,
        system_argument1,
        system_argument2,
        IO_NO_INCREMENT,
    ) == 0
    {
        ExFreePool(apc);
        status = STATUS_UNSUCCESSFUL;
    }

    ObDereferenceObject(thread as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueueApcThread(
    thread_handle: HANDLE,
    apc_routine: PKNORMAL_ROUTINE,
    normal_context: PVOID,
    system_argument1: PVOID,
    system_argument2: PVOID,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let normal_context = opaque_ptr(normal_context);
    let system_argument1 = opaque_ptr(system_argument1);
    let system_argument2 = opaque_ptr(system_argument2);
    NtQueueApcThreadEx(
        thread_handle,
        core::ptr::null_mut(),
        apc_routine,
        normal_context,
        system_argument1,
        system_argument2,
    )
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/state.c
//  * PURPOSE:         Process Manager: Process/Thread State Control
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// PspQueueApcSpecialApc(IN PKAPC Apc,
//                       IN OUT PKNORMAL_ROUTINE* NormalRoutine,
//                       IN OUT PVOID* NormalContext,
//                       IN OUT PVOID* SystemArgument1,
//                       IN OUT PVOID* SystemArgument2)
// {
//     /* Free the APC and do nothing else */
//     ExFreePool(Apc);
// }
//
// NTSTATUS
// NTAPI
// PsResumeThread(IN PETHREAD Thread,
//                OUT PULONG PreviousCount OPTIONAL)
// {
//     ULONG OldCount;
//     PAGED_CODE();
//
//     /* Resume the thread */
//     OldCount = KeResumeThread(&Thread->Tcb);
//
//     /* Return the count if asked */
//     if (PreviousCount) *PreviousCount = OldCount;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// PsSuspendThread(
//     IN PETHREAD Thread,
//     OUT PULONG PreviousCount OPTIONAL)
// {
//     NTSTATUS Status;
//     ULONG OldCount = 0;
//     PAGED_CODE();
//
//     /* Assume success */
//     Status = STATUS_SUCCESS;
//
//     /* Check if we're suspending ourselves */
//     if (Thread == PsGetCurrentThread())
//     {
//         /* Guard with SEH because KeSuspendThread can raise an exception */
//         _SEH2_TRY
//         {
//             /* Do the suspend */
//             OldCount = KeSuspendThread(&Thread->Tcb);
//         }
//         _SEH2_EXCEPT(_SEH2_GetExceptionCode() == STATUS_SUSPEND_COUNT_EXCEEDED)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Acquire rundown protection */
//         if (ExAcquireRundownProtection(&Thread->RundownProtect))
//         {
//             /* Make sure the thread isn't terminating */
//             if (Thread->Terminated)
//             {
//                 /* Fail */
//                 Status = STATUS_THREAD_IS_TERMINATING;
//             }
//             else
//             {
//                 /* Guard with SEH because KeSuspendThread can raise an exception */
//                 _SEH2_TRY
//                 {
//                     /* Do the suspend */
//                     OldCount = KeSuspendThread(&Thread->Tcb);
//                 }
//                 _SEH2_EXCEPT(_SEH2_GetExceptionCode() == STATUS_SUSPEND_COUNT_EXCEEDED)
//                 {
//                     /* Get the exception code */
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//
//                 /* Check if it was terminated during the suspend */
//                 if (Thread->Terminated)
//                 {
//                     /* Wake it back up and fail */
//                     KeForceResumeThread(&Thread->Tcb);
//                     Status = STATUS_THREAD_IS_TERMINATING;
//                     OldCount = 0;
//                 }
//             }
//
//             /* Release rundown protection */
//             ExReleaseRundownProtection(&Thread->RundownProtect);
//         }
//         else
//         {
//             /* Thread is terminating */
//             Status = STATUS_THREAD_IS_TERMINATING;
//         }
//     }
//
//     /* Write back the previous count */
//     if (PreviousCount) *PreviousCount = OldCount;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PsResumeProcess(IN PEPROCESS Process)
// {
//     PETHREAD Thread;
//     PAGED_CODE();
//
//     /* Lock the Process */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         /* Process is terminating */
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Get the first thread */
//     Thread = PsGetNextProcessThread(Process, NULL);
//     while (Thread)
//     {
//         /* Resume it */
//         KeResumeThread(&Thread->Tcb);
//
//         /* Move to the next thread */
//         Thread = PsGetNextProcessThread(Process, Thread);
//     }
//
//     /* Unlock the process */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// PsSuspendProcess(IN PEPROCESS Process)
// {
//     PETHREAD Thread;
//     PAGED_CODE();
//
//     /* Lock the Process */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         /* Process is terminating */
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Get the first thread */
//     Thread = PsGetNextProcessThread(Process, NULL);
//     while (Thread)
//     {
//         /* Resume it */
//         PsSuspendThread(Thread, NULL);
//
//         /* Move to the next thread */
//         Thread = PsGetNextProcessThread(Process, Thread);
//     }
//
//     /* Unlock the process */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtAlertThread(IN HANDLE ThreadHandle)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PETHREAD Thread;
//     NTSTATUS Status;
//
//     /* Reference the Object */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SUSPEND_RESUME,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /*
//          * Do an alert depending on the processor mode. If some kmode code wants to
//          * enforce a umode alert it should call KeAlertThread() directly. If kmode
//          * code wants to do a kmode alert it's sufficient to call it with Zw or just
//          * use KeAlertThread() directly
//          */
//         KeAlertThread(&Thread->Tcb, PreviousMode);
//
//         /* Dereference Object */
//         ObDereferenceObject(Thread);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtAlertResumeThread(IN HANDLE ThreadHandle,
//                     OUT PULONG SuspendCount)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PETHREAD Thread;
//     NTSTATUS Status;
//     ULONG PreviousState;
//
//     /* Check if we came from user mode with a suspend count */
//     if ((SuspendCount) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the count */
//             ProbeForWriteUlong(SuspendCount);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Reference the Object */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SUSPEND_RESUME,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the Kernel Function */
//         PreviousState = KeAlertResumeThread(&Thread->Tcb);
//
//         /* Dereference Object */
//         ObDereferenceObject(Thread);
//
//         /* Check if the caller gave a suspend count */
//         if (SuspendCount)
//         {
//             /* Enter SEH for write */
//             _SEH2_TRY
//             {
//                 /* Write state back */
//                 *SuspendCount = PreviousState;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtResumeThread(IN HANDLE ThreadHandle,
//                OUT PULONG SuspendCount OPTIONAL)
// {
//     PETHREAD Thread;
//     ULONG Prev;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if caller gave a suspend count from user mode */
//     if ((SuspendCount) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the count */
//             ProbeForWriteUlong(SuspendCount);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Get the Thread Object */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SUSPEND_RESUME,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Call the internal function */
//     Status = PsResumeThread(Thread, &Prev);
//
//     /* Check if the caller wanted the count back */
//     if (SuspendCount)
//     {
//         /* Enter SEH for write back */
//         _SEH2_TRY
//         {
//             /* Write the count */
//             *SuspendCount = Prev;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Dereference and return */
//     ObDereferenceObject(Thread);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSuspendThread(IN HANDLE ThreadHandle,
//                 OUT PULONG PreviousSuspendCount OPTIONAL)
// {
//     PETHREAD Thread;
//     ULONG Prev;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if caller gave a suspend count from user mode */
//     if ((PreviousSuspendCount) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the count */
//             ProbeForWriteUlong(PreviousSuspendCount);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Get the Thread Object */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SUSPEND_RESUME,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Call the internal function */
//     Status = PsSuspendThread(Thread, &Prev);
//     ObDereferenceObject(Thread);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Protect write with SEH */
//     _SEH2_TRY
//     {
//         /* Return the Previous Count */
//         if (PreviousSuspendCount) *PreviousSuspendCount = Prev;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSuspendProcess(IN HANDLE ProcessHandle)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PEPROCESS Process;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_SUSPEND_RESUME,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal function */
//         Status = PsSuspendProcess(Process);
//         ObDereferenceObject(Process);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtResumeProcess(IN HANDLE ProcessHandle)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PEPROCESS Process;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_SUSPEND_RESUME,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the internal function */
//         Status = PsResumeProcess(Process);
//         ObDereferenceObject(Process);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtTestAlert(VOID)
// {
//     /* Check and Alert Thread if needed */
//     return KeTestAlertThread(ExGetPreviousMode()) ?
//            STATUS_ALERTED : STATUS_SUCCESS;
// }
//
// /*++
//  * @name NtQueueApcThreadEx
//  * NT4
//  *
//  *    This routine is used to queue an APC from user-mode for the specified
//  *    thread.
//  *
//  * @param ThreadHandle
//  *        Handle to the Thread.
//  *        This handle must have THREAD_SET_CONTEXT privileges.
//  *
//  * @param UserApcReserveHandle
//  *        Optional handle to reserve object (introduced in Windows 7), providing ability to
//  *        reserve memory before performing stability-critical parts of code.
//  *
//  * @param ApcRoutine
//  *        Pointer to the APC Routine to call when the APC executes.
//  *
//  * @param NormalContext
//  *        Pointer to the context to send to the Normal Routine.
//  *
//  * @param SystemArgument[1-2]
//  *        Pointer to a set of two parameters that contain untyped data.
//  *
//  * @return STATUS_SUCCESS or failure cute from associated calls.
//  *
//  * @remarks The thread must enter an alertable wait before the APC will be
//  *          delivered.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtQueueApcThreadEx(IN HANDLE ThreadHandle,
//                  IN OPTIONAL HANDLE UserApcReserveHandle,
//                  IN PKNORMAL_ROUTINE ApcRoutine,
//                  IN PVOID NormalContext,
//                  IN OPTIONAL PVOID SystemArgument1,
//                  IN OPTIONAL PVOID SystemArgument2)
// {
//     PKAPC Apc;
//     PETHREAD Thread;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Get ETHREAD from Handle */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SET_CONTEXT,
//                                        PsThreadType,
//                                        ExGetPreviousMode(),
//                                        (PVOID)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if this is a System Thread */
//     if (Thread->SystemThread)
//     {
//         /* Fail */
//         Status = STATUS_INVALID_HANDLE;
//         goto Quit;
//     }
//
//     /* Allocate an APC */
//     Apc = ExAllocatePoolWithQuotaTag(NonPagedPool |
//                                      POOL_QUOTA_FAIL_INSTEAD_OF_RAISE,
//                                      sizeof(KAPC),
//                                      TAG_PS_APC);
//     if (!Apc)
//     {
//         /* Fail */
//         Status = STATUS_NO_MEMORY;
//         goto Quit;
//     }
//
//     /* Initialize the APC */
//     KeInitializeApc(Apc,
//                     &Thread->Tcb,
//                     OriginalApcEnvironment,
//                     PspQueueApcSpecialApc,
//                     NULL,
//                     ApcRoutine,
//                     UserMode,
//                     NormalContext);
//
//     /* Queue it */
//     if (!KeInsertQueueApc(Apc,
//                           SystemArgument1,
//                           SystemArgument2,
//                           IO_NO_INCREMENT))
//     {
//         /* We failed, free it */
//         ExFreePool(Apc);
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Dereference Thread and Return */
// Quit:
//     ObDereferenceObject(Thread);
//     return Status;
// }
//
// /*++
//  * @name NtQueueApcThread
//  * NT4
//  *
//  *    This routine is used to queue an APC from user-mode for the specified
//  *    thread.
//  *
//  * @param ThreadHandle
//  *        Handle to the Thread.
//  *        This handle must have THREAD_SET_CONTEXT privileges.
//  *
//  * @param ApcRoutine
//  *        Pointer to the APC Routine to call when the APC executes.
//  *
//  * @param NormalContext
//  *        Pointer to the context to send to the Normal Routine.
//  *
//  * @param SystemArgument[1-2]
//  *        Pointer to a set of two parameters that contain untyped data.
//  *
//  * @return STATUS_SUCCESS or failure cute from associated calls.
//  *
//  * @remarks The thread must enter an alertable wait before the APC will be
//  *          delivered.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtQueueApcThread(IN HANDLE ThreadHandle,
//     IN PKNORMAL_ROUTINE ApcRoutine,
//     IN PVOID NormalContext,
//     IN PVOID SystemArgument1,
//     IN PVOID SystemArgument2)
// {
//     return NtQueueApcThreadEx(ThreadHandle, NULL, ApcRoutine, NormalContext, SystemArgument1, SystemArgument2);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
