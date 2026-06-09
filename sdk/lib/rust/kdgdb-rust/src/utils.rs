use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
// Use the bindgen process/thread types (full layout), NOT the opaque ps:: stubs.
use ntkrnl_sys::ddk::{_EPROCESS, _ETHREAD, EPROCESS, LIST_ENTRY, PEPROCESS, PETHREAD};
use core::ffi::c_void;

use crate::kdgdb_h::{gdb_pid_to_handle, gdb_tid_to_handle};
use crate::kdpacket::{CurrentStateChange, ProcessListHead, TheIdleProcess, TheIdleThread};
#[no_mangle]
pub unsafe extern "C" fn find_process(Pid: ULONG_PTR) -> PEPROCESS {
    let ProcessId = gdb_pid_to_handle(Pid);
    let mut ProcessEntry: *mut LIST_ENTRY;
    let mut Process: PEPROCESS;

    if ProcessId.is_null() {
        return TheIdleProcess;
    }

    ProcessEntry = (*ProcessListHead).Flink;
    while ProcessEntry != ProcessListHead {
        Process = ((ProcessEntry as usize) - core::mem::offset_of!(_EPROCESS, ActiveProcessLinks)) as PEPROCESS;

        if (*Process).UniqueProcessId == ProcessId {
            return Process;
        }

        ProcessEntry = (*ProcessEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn find_thread(
    Pid: ULONG_PTR,
    Tid: ULONG_PTR
) -> PETHREAD {
    let ThreadId = gdb_tid_to_handle(Tid);
    let mut Thread: PETHREAD;
    let mut Process: PEPROCESS;
    let mut ThreadEntry: *mut LIST_ENTRY;
    #[cfg(feature = "MONOPROCESS")]
    let mut ProcessEntry: *mut LIST_ENTRY;

    // C: `#if !MONOPROCESS (Pid == 0) && #endif (Tid == 0)`
    #[cfg(not(feature = "MONOPROCESS"))]
    let any_thread = (Pid == 0) && (Tid == 0);
    #[cfg(feature = "MONOPROCESS")]
    let any_thread = Tid == 0;
    if any_thread {
        /* Zero means any, so use the current one */
        return CurrentStateChange.Thread as PETHREAD;
    }

    #[cfg(feature = "MONOPROCESS")]
    {
        if Tid == 1 {
            return TheIdleThread;
        }

        ProcessEntry = (*ProcessListHead).Flink;
        while ProcessEntry != ProcessListHead {
            Process = ((ProcessEntry as usize) - core::mem::offset_of!(_EPROCESS, ActiveProcessLinks)) as PEPROCESS;

            ThreadEntry = (*Process).ThreadListHead.Flink;
            while ThreadEntry != core::ptr::addr_of_mut!((*Process).ThreadListHead) {
                Thread = ((ThreadEntry as usize) - core::mem::offset_of!(_ETHREAD, ThreadListEntry)) as PETHREAD;
                if ((*Thread).Cid.UniqueThread == ThreadId) || (Tid == 0) {
                    return Thread;
                }
                ThreadEntry = (*ThreadEntry).Flink;
            }

            ProcessEntry = (*ProcessEntry).Flink;
        }
    }

    #[cfg(not(feature = "MONOPROCESS"))]
    {
        Process = find_process(Pid);

        if (Process == TheIdleProcess) && (Tid == 1) {
            return TheIdleThread;
        }

        if Process.is_null() {
            return core::ptr::null_mut();
        }

        ThreadEntry = (*Process).ThreadListHead.Flink;
        while ThreadEntry != core::ptr::addr_of_mut!((*Process).ThreadListHead) {
            Thread = ((ThreadEntry as usize) - core::mem::offset_of!(_ETHREAD, ThreadListEntry)) as PETHREAD;
            if ((*Thread).Cid.UniqueThread == ThreadId) || (Tid == 0) {
                return Thread;
            }
            ThreadEntry = (*ThreadEntry).Flink;
        }
    }

    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/utils.c
//  * PURPOSE:         Misc helper functions.
//  */
//
// #include "kdgdb.h"
//
// /*
//  * We cannot use PsLookupProcessThreadByCid or alike as we could be running at any IRQL.
//  * So we have to loop over the process list.
//  */
//
// PEPROCESS
// find_process(
//      UINT_PTR Pid)
// {
//     HANDLE ProcessId = gdb_pid_to_handle(Pid);
//     LIST_ENTRY* ProcessEntry;
//     PEPROCESS Process;
//
//     /* Special case for idle process */
//     if (ProcessId == NULL)
//         return TheIdleProcess;
//
//     for (ProcessEntry = ProcessListHead->Flink;
//             ProcessEntry != ProcessListHead;
//             ProcessEntry = ProcessEntry->Flink)
//     {
//         Process = CONTAINING_RECORD(ProcessEntry, EPROCESS, ActiveProcessLinks);
//
//         if (Process->UniqueProcessId == ProcessId)
//             return Process;
//     }
//
//     return NULL;
// }
//
// PETHREAD
// find_thread(
//      UINT_PTR Pid,
//      UINT_PTR Tid)
// {
//     HANDLE ThreadId = gdb_tid_to_handle(Tid);
//     PETHREAD Thread;
//     PEPROCESS Process;
//     LIST_ENTRY* ThreadEntry;
// #if MONOPROCESS
//     LIST_ENTRY* ProcessEntry;
// #endif
//
//     if (
// #if !MONOPROCESS
//     (Pid == 0) &&
// #endif
//     (Tid == 0))
//     {
//         /* Zero means any, so use the current one */
//         return (PETHREAD)(ULONG_PTR)CurrentStateChange.Thread;
//     }
//
// #if MONOPROCESS
//
//     /* Special case for the idle thread */
//     if (Tid == 1)
//         return TheIdleThread;
//
//     for (ProcessEntry = ProcessListHead->Flink;
//         ProcessEntry != ProcessListHead;
//         ProcessEntry = ProcessEntry->Flink)
//     {
//         Process = CONTAINING_RECORD(ProcessEntry, EPROCESS, ActiveProcessLinks);
// #else
//
//     Process = find_process(Pid);
//
//     /* Special case for the idle thread */
//     if ((Process == TheIdleProcess) && (Tid == 1))
//         return TheIdleThread;
//
//     if (!Process)
//         return NULL;
//
// #endif
//
//     for (ThreadEntry = Process->ThreadListHead.Flink;
//             ThreadEntry != &Process->ThreadListHead;
//             ThreadEntry = ThreadEntry->Flink)
//     {
//         Thread = CONTAINING_RECORD(ThreadEntry, ETHREAD, ThreadListEntry);
//         /* For GDB, Tid == 0 means any thread */
//         if ((Thread->Cid.UniqueThread == ThreadId) || (Tid == 0))
//         {
//             return Thread;
//         }
//     }
//
// #if MONOPROCESS
//     }
// #endif
//
//     return NULL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
