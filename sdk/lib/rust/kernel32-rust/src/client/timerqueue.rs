use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::substrate::{TRUE, FALSE};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut BasepDefaultTimerQueue: HANDLE = NULL;

#[no_mangle]
pub unsafe extern "system" fn BasepCreateDefaultTimerQueue() -> BOOL {
    if !BasepDefaultTimerQueue.is_null() {
        return 1;
    }

    // Create into a local and publish once (upstream 73e69a62d39). Creating straight into
    // the global let two first callers each make a queue; the loser's queue, and its
    // worker thread, leaked, with timers already on it.
    let mut TimerQueue: HANDLE = NULL;
    let Status: NTSTATUS = RtlCreateTimerQueue(&mut TimerQueue);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if !crate::k32_externs::InterlockedCompareExchangePointer(
        (&raw mut BasepDefaultTimerQueue).cast(),
        TimerQueue,
        NULL,
    )
    .is_null()
    {
        // Another thread won; ours was never handed out.
        RtlDeleteTimerQueueEx(TimerQueue, core::ptr::null_mut());
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn CancelTimerQueueTimer(
    mut TimerQueue: HANDLE,
    Timer: HANDLE,
) -> BOOL {
    let mut Status: NTSTATUS = 0;

    if TimerQueue.is_null() {
        TimerQueue = BasepDefaultTimerQueue;
        if TimerQueue.is_null() {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
            return FALSE;
        }
    }

    Status = RtlDeleteTimer(TimerQueue, Timer, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn ChangeTimerQueueTimer(
    TimerQueue: HANDLE,
    Timer: HANDLE,
    DueTime: ULONG,
    Period: ULONG,
) -> BOOL {
    let mut TimerQueue = TimerQueue;
    let mut Status: NTSTATUS = 0;

    if TimerQueue.is_null() {
        TimerQueue = BasepDefaultTimerQueue;
        if TimerQueue.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return FALSE;
        }
    }

    Status = RtlUpdateTimer(TimerQueue, Timer, DueTime, Period);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn CreateTimerQueue() -> HANDLE {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    Status = RtlCreateTimerQueue(&raw mut Handle);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    Handle
}

#[no_mangle]
pub unsafe extern "system" fn CreateTimerQueueTimer(
    phNewTimer: PHANDLE,
    mut TimerQueue: HANDLE,
    Callback: WAITORTIMERCALLBACK,
    Parameter: PVOID,
    DueTime: DWORD,
    Period: DWORD,
    Flags: ULONG,
) -> BOOL {
    let mut Status: NTSTATUS = 0;

    /* Parameter is not optional -- clear it now */
    *phNewTimer = core::ptr::null_mut();

    /* Check if no queue was given */
    if TimerQueue.is_null() {
        /* Create the queue if it didn't already exist */
        if BasepDefaultTimerQueue.is_null() && BasepCreateDefaultTimerQueue() == 0 {
            return 0;
        }

        /* Use the default queue */
        TimerQueue = BasepDefaultTimerQueue;
    }

    /* Create the timer. Note that no parameter checking is done here */
    Status = RtlCreateTimer(
        TimerQueue,
        phNewTimer,
        Callback,
        Parameter,
        DueTime,
        Period,
        Flags,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteTimerQueue(TimerQueue: HANDLE) -> BOOL {
    if TimerQueue.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE;
    }
    RtlDeleteTimerQueueEx(TimerQueue, core::ptr::null_mut());
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteTimerQueueEx(
    TimerQueue: HANDLE,
    CompletionEvent: HANDLE,
) -> BOOL {
    let Status: NTSTATUS;

    /* We don't allow the user to delete the default timer queue */
    if TimerQueue.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    /* Delete the timer queue */
    Status = RtlDeleteTimerQueueEx(TimerQueue, CompletionEvent);
    if ((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING)) ||
        !(NT_SUCCESS(Status))
    {
        /* In case CompletionEvent == NULL, RtlDeleteTimerQueueEx() returns before
           all callback routines returned. We set the last error code to STATUS_PENDING
           and return FALSE. In case CompletionEvent == INVALID_HANDLE_VALUE we only
           can get here if another error occured. In case CompletionEvent is something
           else, we get here and fail, even though it isn't really an error (if Status == STATUS_PENDING).
           We also handle all other failures the same way. */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteTimerQueueTimer(
    TimerQueue: HANDLE,
    Timer: HANDLE,
    CompletionEvent: HANDLE,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut TimerQueue = TimerQueue;
    let mut CompletionEvent = CompletionEvent;

    if TimerQueue.is_null() {
        TimerQueue = BasepDefaultTimerQueue;
        if TimerQueue.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return FALSE;
        }
    }

    Status = RtlDeleteTimer(TimerQueue, Timer, CompletionEvent);
    if ((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING))
        || !(NT_SUCCESS(Status))
    {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn SetTimerQueueTimer(
    mut TimerQueue: HANDLE,
    Callback: WAITORTIMERCALLBACK,
    Parameter: PVOID,
    DueTime: DWORD,
    Period: DWORD,
    PreferIo: BOOL,
) -> HANDLE {
    let mut Timer: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if TimerQueue.is_null() {
        if BasepDefaultTimerQueue.is_null() && BasepCreateDefaultTimerQueue() == 0 {
            return core::ptr::null_mut();
        }
        TimerQueue = BasepDefaultTimerQueue;
    }

    Status = RtlCreateTimer(
        TimerQueue,
        &raw mut Timer,
        Callback,
        Parameter,
        DueTime,
        Period,
        if PreferIo != 0 {
            WT_EXECUTEINIOTHREAD
        } else {
            WT_EXECUTEDEFAULT
        },
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    Timer
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/timerqueue.c
//  * PURPOSE:         Timer Queue Functions
//  * PROGRAMMERS:     Thomas Weidenmueller <w3seek@reactos.com>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// HANDLE BasepDefaultTimerQueue = NULL;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /* FIXME - make this thread safe */
// BOOL
// WINAPI
// BasepCreateDefaultTimerQueue(VOID)
// {
//     NTSTATUS Status;
//
//     /* Create the timer queue */
//     Status = RtlCreateTimerQueue(&BasepDefaultTimerQueue);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         DPRINT1("Unable to create the default timer queue!\n");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CancelTimerQueueTimer(IN HANDLE TimerQueue,
//                       IN HANDLE Timer)
// {
//     NTSTATUS Status;
//
//     if (!TimerQueue)
//     {
//         /* Use the default timer queue */
//         TimerQueue = BasepDefaultTimerQueue;
//         if (!TimerQueue)
//         {
//             /* A timer is being cancelled before one was created... fail */
//             SetLastError(ERROR_INVALID_HANDLE);
//             return FALSE;
//         }
//     }
//
//     /* Delete the timer */
//     Status = RtlDeleteTimer(TimerQueue, Timer, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ChangeTimerQueueTimer(IN HANDLE TimerQueue,
//                       IN HANDLE Timer,
//                       IN ULONG DueTime,
//                       IN ULONG Period)
// {
//     NTSTATUS Status;
//
//     if (!TimerQueue)
//     {
//         /* Use the default timer queue */
//         TimerQueue = BasepDefaultTimerQueue;
//         if (!TimerQueue)
//         {
//             /* A timer is being cancelled before one was created... fail */
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//     }
//
//     /* Delete the timer */
//     Status = RtlUpdateTimer(TimerQueue, Timer, DueTime, Period);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateTimerQueue(VOID)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//
//     /* Create the timer queue */
//     Status = RtlCreateTimerQueue(&Handle);
//     if(!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     return Handle;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateTimerQueueTimer(OUT PHANDLE phNewTimer,
//                       IN HANDLE TimerQueue,
//                       IN WAITORTIMERCALLBACK Callback,
//                       IN PVOID Parameter,
//                       IN DWORD DueTime,
//                       IN DWORD Period,
//                       IN ULONG Flags)
// {
//     NTSTATUS Status;
//
//     /* Parameter is not optional -- clear it now */
//     *phNewTimer = NULL;
//
//     /* Check if no queue was given */
//     if (!TimerQueue)
//     {
//         /* Create the queue if it didn't already exist */
//         if (!(BasepDefaultTimerQueue) && !(BasepCreateDefaultTimerQueue()))
//         {
//             return FALSE;
//         }
//
//         /* Use the default queue */
//         TimerQueue = BasepDefaultTimerQueue;
//     }
//
//     /* Create the timer. Note that no parameter checking is done here */
//     Status = RtlCreateTimer(TimerQueue,
//                             phNewTimer,
//                             Callback,
//                             Parameter,
//                             DueTime,
//                             Period,
//                             Flags);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeleteTimerQueue(IN HANDLE TimerQueue)
// {
//     /* We don't allow the user to delete the default timer queue */
//     if (!TimerQueue)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Delete the timer queue */
//     RtlDeleteTimerQueueEx(TimerQueue, 0);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeleteTimerQueueEx(IN HANDLE TimerQueue,
//                    IN HANDLE CompletionEvent)
// {
//     NTSTATUS Status;
//
//     /* We don't allow the user to delete the default timer queue */
//     if (!TimerQueue)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Delete the timer queue */
//     Status = RtlDeleteTimerQueueEx(TimerQueue, CompletionEvent);
//     if (((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING)) ||
//         !(NT_SUCCESS(Status)))
//     {
//         /* In case CompletionEvent == NULL, RtlDeleteTimerQueueEx() returns before
//            all callback routines returned. We set the last error code to STATUS_PENDING
//            and return FALSE. In case CompletionEvent == INVALID_HANDLE_VALUE we only
//            can get here if another error occured. In case CompletionEvent is something
//            else, we get here and fail, even though it isn't really an error (if Status == STATUS_PENDING).
//            We also handle all other failures the same way. */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeleteTimerQueueTimer(IN HANDLE TimerQueue,
//                       IN HANDLE Timer,
//                       IN HANDLE CompletionEvent)
// {
//     NTSTATUS Status;
//
//     if (!TimerQueue)
//     {
//         /* Use the default timer queue */
//         TimerQueue = BasepDefaultTimerQueue;
//         if (!TimerQueue)
//         {
//             /* A timer is being cancelled before one was created... fail */
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//     }
//
//     /* Delete the timer */
//     Status = RtlDeleteTimer(TimerQueue, Timer, CompletionEvent);
//     if (((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING)) ||
//         !(NT_SUCCESS(Status)))
//     {
//         /* In case CompletionEvent == NULL, RtlDeleteTimerQueueEx() returns before
//            all callback routines returned. We set the last error code to STATUS_PENDING
//            and return FALSE. In case CompletionEvent == INVALID_HANDLE_VALUE we only
//            can get here if another error occured. In case CompletionEvent is something
//            else, we get here and fail, even though it isn't really an error (if Status == STATUS_PENDING).
//            We also handle all other failures the same way. */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// SetTimerQueueTimer(IN HANDLE TimerQueue,
//                    IN WAITORTIMERCALLBACK Callback,
//                    IN PVOID Parameter,
//                    IN DWORD DueTime,
//                    IN DWORD Period,
//                    IN BOOL PreferIo)
// {
//     HANDLE Timer;
//     NTSTATUS Status;
//
//     /* Check if no queue was given */
//     if (!TimerQueue)
//     {
//         /* Create the queue if it didn't already exist */
//         if (!(BasepDefaultTimerQueue) && !(BasepCreateDefaultTimerQueue()))
//         {
//             return FALSE;
//         }
//
//         /* Use the default queue */
//         TimerQueue = BasepDefaultTimerQueue;
//     }
//
//     /* Create the timer */
//     Status = RtlCreateTimer(TimerQueue,
//                             &Timer,
//                             Callback,
//                             Parameter,
//                             DueTime,
//                             Period,
//                             PreferIo ? WT_EXECUTEINIOTHREAD : WT_EXECUTEDEFAULT);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     return Timer;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
