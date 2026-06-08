use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{IRP, IoMarkIrpPending, KeInitializeDpc, KeInitializeSpinLock, KeInitializeTimer, KeSetTimerEx, LARGE_INTEGER, LOGICAL, PDEVICE_OBJECT, PIRP};
use core::ffi::c_void;

extern "C" {
    fn TracePrint(level: u32, flag: u32, format: *const i8, ...);
    fn ClasspGetCurrentTime() -> LARGE_INTEGER;
}
extern "C" {
    fn NT_ASSERT(condition: i32);
    fn ClassGetDeviceParameter( FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, SubKeyName: *const i8, ValueName: *const i8, Value: *mut u32, ) -> u32;
    fn Int32x32To64(a: i32, b: i32) -> i64;
    fn InterlockedCompareExchange(Destination: *mut i32, Exchange: i32, Comperand: i32) -> i32;
    fn KeCancelTimer(Timer: *mut KTIMER) -> i32;
    fn ServiceTransferRequest(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, PostToDpc: u8);
    fn RtlULongLongSub(Minuend: u64, Subtrahend: u64, Result: *mut u64) -> NTSTATUS;
    fn ClasspTimeDiffToMs(time: u64) -> u64;
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "C" fn ClasspStartIdleTimer(FdoData: *mut CLASS_PRIVATE_FDO_DATA) ;
#[no_mangle]
pub unsafe extern "C" fn ClasspStopIdleTimer(FdoData: *mut CLASS_PRIVATE_FDO_DATA) ;
static mut ClasspIdleTimerDpc: KDEFERRED_ROUTINE;
#[no_mangle]
pub unsafe extern "C" fn ClasspServiceIdleRequest(FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, PostToDpc: BOOLEAN) ;
#[no_mangle]
pub unsafe extern "C" fn ClasspDequeueIdleRequest(FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION) -> *mut IRP;

// Unknown constants – real values must be filled in by the caller
const TRACE_LEVEL_INFORMATION: u32 = 0; // FIXME
const TRACE_FLAG_TIMER: u32 = 0; // FIXME

// Forward declarations for unknown external symbols

// Constants not found in imports; using plausible values
const TRACE_LEVEL_INFORMATION: u32 = 4;
const TRACE_FLAG_TIMER: u32 = 0x1000;
#[no_mangle]
pub unsafe extern "C" fn EnqueueDeferredClientIrp(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let fdoExtension = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let mut oldIrql: KIRQL = core::mem::MaybeUninit::uninit().assume_init(); // KIRQL is POD, no init needed
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*fdoData).SpinLock),
        core::ptr::addr_of_mut!(oldIrql),
    );
    InsertTailList(
        core::ptr::addr_of_mut!((*fdoData).DeferredClientIrpList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.ListEntry),
    );
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*fdoData).SpinLock),
        oldIrql,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DequeueDeferredClientIrp(
    Fdo: *mut DEVICE_OBJECT,
) -> *mut IRP {
    const IO_TYPE_IRP: i16 = 6;

    let fdoExtension = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let irp: *mut IRP;

    if IsListEmpty(
        (&mut (*fdoData).DeferredClientIrpList) as *mut LIST_ENTRY,
    ) != 0 {
        irp = core::ptr::null_mut();
    } else {
        let listEntry: *mut LIST_ENTRY;
        let mut oldIrql: KIRQL = 0;

        KeAcquireSpinLock(
            &mut (*fdoData).SpinLock as *mut KSPIN_LOCK,
            &mut oldIrql,
        );

        if IsListEmpty(
            (&mut (*fdoData).DeferredClientIrpList) as *mut LIST_ENTRY,
        ) != 0 {
            listEntry = core::ptr::null_mut();
        } else {
            listEntry = RemoveHeadList(
                (&mut (*fdoData).DeferredClientIrpList) as *mut LIST_ENTRY,
            );
        }

        KeReleaseSpinLock(
            &mut (*fdoData).SpinLock as *mut KSPIN_LOCK,
            oldIrql,
        );

        if listEntry.is_null() {
            irp = core::ptr::null_mut();
        } else {
            let offset = core::mem::offset_of!(IRP, Tail.Overlay.ListEntry);
            irp = (listEntry as *const u8).byte_sub(offset) as *mut IRP;
            NT_ASSERT(((*irp).Type == IO_TYPE_IRP) as i32);
            InitializeListHead(
                (&mut (*irp).Tail.Overlay.ListEntry) as *mut LIST_ENTRY,
            );
        }
    }

    return irp;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeIdleTimer(FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION) {
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA = (*FdoExtension).PrivateFdoData;
    let mut idleInterval: u32 = CLASS_IDLE_INTERVAL;
    let mut idlePrioritySupported: u32 = 1;
    let mut activeIdleIoMax: u32 = 1;

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME as *const i8,
        CLASSP_REG_IDLE_PRIORITY_SUPPORTED as *const i8,
        addr_of_mut!(idlePrioritySupported),
    );

    if idlePrioritySupported == 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspInitializeIdleTimer: Idle priority not supported for disk %p\n\0".as_ptr() as *const i8,
            FdoExtension as *const core::ffi::c_void,
        );
        (*fdoData).IdlePrioritySupported = 0;
        (*fdoData).IdleIoCount = 0;
        (*fdoData).ActiveIoCount = 0;
        return;
    }

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME as *const i8,
        CLASSP_REG_IDLE_INTERVAL_NAME as *const i8,
        addr_of_mut!(idleInterval),
    );

    if idleInterval < CLASS_IDLE_INTERVAL_MIN || idleInterval > 0xFFFFu32 {
        idleInterval = CLASS_IDLE_INTERVAL;
    }

    (*fdoData).IdlePrioritySupported = 1;
    KeInitializeSpinLock(addr_of_mut!((*fdoData).IdleListLock));
    KeInitializeTimer(addr_of_mut!((*fdoData).IdleTimer));
    KeInitializeDpc(
        addr_of_mut!((*fdoData).IdleDpc),
        ClasspIdleTimerDpc as *const KDEFERRED_ROUTINE,
        FdoExtension as *const core::ffi::c_void,
    );
    InitializeListHead(addr_of_mut!((*fdoData).IdleIrpList) as *mut LIST_ENTRY);
    (*fdoData).IdleTimerStarted = 0;
    (*fdoData).StarvationDuration = CLASS_STARVATION_INTERVAL;
    (*fdoData).IdleInterval = idleInterval as u16;
    (*fdoData).IdleIoCount = 0;
    (*fdoData).ActiveIoCount = 0;
    (*fdoData).ActiveIdleIoCount = 0;

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME as *const i8,
        CLASSP_REG_IDLE_ACTIVE_MAX as *const i8,
        addr_of_mut!(activeIdleIoMax),
    );

    if activeIdleIoMax < 1 {
        activeIdleIoMax = 1;
    }
    if activeIdleIoMax > 0xFFFFu32 {
        activeIdleIoMax = 0xFFFFu32;
    }

    (*fdoData).IdleActiveIoMax = activeIdleIoMax as u16;

    return;
}

unsafe extern "C" {
    fn RtlULongLongSub(Minuend: u64, Subtrahend: u64, Result: *mut u64) -> NTSTATUS;
    fn ClasspTimeDiffToMs(diff: u64) -> u64;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetIdleTime(
    FdoData: *mut CLASS_PRIVATE_FDO_DATA,
    CurrentTime: LARGE_INTEGER,
) -> u64 {
    let mut idleTime: u64;
    let mut status: NTSTATUS;

    status = RtlULongLongSub(
        CurrentTime.QuadPart as u64,
        (*FdoData).LastNonIdleIoTime.QuadPart as u64,
        core::ptr::addr_of_mut!(idleTime),
    );

    if NT_SUCCESS(status) {
        idleTime = ClasspTimeDiffToMs(idleTime);
    } else {
        idleTime = (*FdoData).IdleInterval as u64;
    }

    idleTime
}

#[no_mangle]
pub unsafe extern "C" fn ClasspIdleDurationSufficient(
    FdoData: *mut CLASS_PRIVATE_FDO_DATA,
    CurrentTimeIn: *mut *mut LARGE_INTEGER,
) -> LOGICAL {
    let mut idleInterval: u64;
    let mut CurrentTime: LARGE_INTEGER;

    if (*FdoData).ActiveIoCount > 0 {
        if !CurrentTimeIn.is_null() {
            *CurrentTimeIn = core::ptr::null_mut();
        }
        return 0;
    }

    CurrentTime = ClasspGetCurrentTime();
    idleInterval = ClasspGetIdleTime(FdoData, CurrentTime);

    if !CurrentTimeIn.is_null() {
        **CurrentTimeIn = CurrentTime;
    }

    if idleInterval >= (*FdoData).IdleInterval {
        return 1;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ClasspIdleTimerDpc(
    Dpc: *mut KDPC,
    Context: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = Context as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA;
    let mut idleTime: u64 = 0;
    let mut status: NTSTATUS = 0;
    let mut currentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut pCurrentTime: *mut LARGE_INTEGER;

    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    if fdoExtension.is_null() {
        NT_ASSERT((!fdoExtension.is_null()) as LOGICAL);
        return;
    }

    fdoData = (*fdoExtension).PrivateFdoData;

    if (*fdoData).ActiveIoCount <= 0 {
        if (*fdoData).ActiveIdleIoCount >= (*fdoData).IdleActiveIoMax {
            return;
        }

        pCurrentTime = &mut currentTime as *mut LARGE_INTEGER;
        let ppCurrentTime: *mut *mut LARGE_INTEGER = &mut pCurrentTime;
        let ret = ClasspIdleDurationSufficient(fdoData, ppCurrentTime);
        if ret != 0 {
            NT_ASSERT((!pCurrentTime.is_null()) as LOGICAL);
            (*fdoData).AntiStarvationStartTime = *pCurrentTime;
            ClasspServiceIdleRequest(fdoExtension, 0u8);
        }
        return;
    }

    currentTime = ClasspGetCurrentTime();
    status = RtlULongLongSub(currentTime.QuadPart as u64, (*fdoData).AntiStarvationStartTime.QuadPart as u64, &mut idleTime as *mut u64);

    if NT_SUCCESS(status) != 0 {
        idleTime = ClasspTimeDiffToMs(idleTime);
    } else {
        idleTime = (*fdoData).StarvationDuration;
    }

    if idleTime >= (*fdoData).StarvationDuration {
        (*fdoData).AntiStarvationStartTime = currentTime;
        TracePrint(4, 0x02, b"ClasspIdleTimerDpc: Starvation timer. Send one idle request\n\0".as_ptr() as *const i8);
        ClasspServiceIdleRequest(fdoExtension, 0u8);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEnqueueIdleRequest(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> i32 {
    unsafe {
        let fdoExtension = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
        let fdoData = (*fdoExtension).PrivateFdoData;
        let mut oldIrql: KIRQL = 0;
        let mut issueRequest: u8 = 1; // TRUE
        let mut currentTime: LARGE_INTEGER = core::mem::zeroed();
        let mut pCurrentTime: *mut LARGE_INTEGER = core::ptr::addr_of_mut!(currentTime);

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspEnqueueIdleRequest: Queue idle request %p\n\0".as_ptr() as *const i8,
            Irp as *const core::ffi::c_void,
        );

        IoMarkIrpPending(Irp);

        // Reset issueRequest if the idle duration is not sufficient.
        if ClasspIdleDurationSufficient(fdoData, &raw mut pCurrentTime) == 0 {
            issueRequest = 0; // FALSE
        }

        // If there are already max active idle requests, queue this idle request.
        if (*fdoData).ActiveIdleIoCount >= (*fdoData).IdleActiveIoMax {
            issueRequest = 0; // FALSE
        }

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*fdoData).IdleListLock),
            &raw mut oldIrql,
        );
        if IsListEmpty(core::ptr::addr_of!((*fdoData).IdleIrpList) as PLIST_ENTRY) != 0 {
            NT_ASSERT(((*fdoData).IdleIoCount == 0) as i32);
        }
        InsertTailList(
            core::ptr::addr_of_mut!((*fdoData).IdleIrpList),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.ListEntry),
        );

        (*fdoData).IdleIoCount += 1;
        if (*fdoData).IdleTimerStarted == 0 {
            ClasspStartIdleTimer(fdoData);
        }

        if (*fdoData).IdleIoCount != 1 {
            issueRequest = 0; // FALSE
        }

        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*fdoData).IdleListLock),
            oldIrql,
        );

        if issueRequest != 0 {
            ClasspServiceIdleRequest(fdoExtension, 0u8); // FALSE
        }

        0x00000103 // STATUS_PENDING
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCompleteIdleRequest(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
) {
    let fdoData = (*FdoExtension).PrivateFdoData;

    //
    // Issue the next idle request if there are any left in the queue, there are
    // no non-idle requests outstanding, there are less than max idle requests
    // outstanding, and it has been long enough since the completion of the last
    // non-idle request.
    //
    if ((*fdoData).IdleIoCount > 0)
        && ((*fdoData).ActiveIdleIoCount < (*fdoData).IdleActiveIoMax)
        && ((*fdoData).ActiveIoCount <= 0)
        && (ClasspIdleDurationSufficient(fdoData, core::ptr::null_mut()) != 0)
    {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_TIMER,
            b"ClasspCompleteIdleRequest: Service next idle reqeusts\n\0".as_ptr() as *const i8,
        );
        ClasspServiceIdleRequest(FdoExtension, 1);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 1999
//
// Module Name:
//
//     clntirp.c
//
// Abstract:
//
//     Client IRP queuing routines for CLASSPNP
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "classp.h"
// #include "debug.h"
//
//
// #ifdef DEBUG_USE_WPP
// #include "clntirp.tmh"
// #endif
//
// VOID
// ClasspStartIdleTimer(
//     IN PCLASS_PRIVATE_FDO_DATA FdoData
//     );
//
// VOID
// ClasspStopIdleTimer(
//     PCLASS_PRIVATE_FDO_DATA FdoData
//     );
//
// KDEFERRED_ROUTINE ClasspIdleTimerDpc;
//
// VOID
// ClasspServiceIdleRequest(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     BOOLEAN PostToDpc
//     );
//
//
// PIRP
// ClasspDequeueIdleRequest(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     );
//
//
// /*++
//
// EnqueueDeferredClientIrp
//
// Routine Description:
//
//     Insert the deferred irp into the list.
//
//     Note: we currently do not support Cancel for storage irps.
//
// Arguments:
//
//     Fdo - Pointer to the device object
//     Irp     - Pointer to the I/O request packet
//
// Return Value:
//
//     None
//
// --*/
// VOID
// EnqueueDeferredClientIrp(
//     PDEVICE_OBJECT Fdo,
//     PIRP Irp
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     KIRQL oldIrql;
//
//
//     KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//     InsertTailList(&fdoData->DeferredClientIrpList, &Irp->Tail.Overlay.ListEntry);
//
//
//     KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
// }
//
// /*++
//
// DequeueDeferredClientIrp
//
// Routine Description:
//
//     Remove the deferred irp from the list.
//
// Arguments:
//
//     Fdo - Pointer to the device object
//
// Return Value:
//
//     Pointer to removed IRP
//
// --*/
// PIRP
// DequeueDeferredClientIrp(
//     PDEVICE_OBJECT Fdo
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     PIRP irp;
//
//     //
//     // The DeferredClientIrpList is almost always empty.
//     // We don't want to grab the spinlock every time we check it (which is on every xfer completion)
//     // so check once first before we grab the spinlock.
//     //
//     if (IsListEmpty(&fdoData->DeferredClientIrpList)){
//         irp = NULL;
//     }
//     else {
//         PLIST_ENTRY listEntry;
//         KIRQL oldIrql;
//
//         KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//         if (IsListEmpty(&fdoData->DeferredClientIrpList)){
//             listEntry = NULL;
//         }
//         else {
//             listEntry = RemoveHeadList(&fdoData->DeferredClientIrpList);
//         }
//         KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//
//         if (listEntry == NULL) {
//             irp = NULL;
//         }
//         else {
//             irp = CONTAINING_RECORD(listEntry, IRP, Tail.Overlay.ListEntry);
//             NT_ASSERT(irp->Type == IO_TYPE_IRP);
//
//
//             InitializeListHead(&irp->Tail.Overlay.ListEntry);
//         }
//     }
//
//     return irp;
// }
//
// /*++
//
// ClasspInitializeIdleTimer
//
// Routine Description:
//
//     Initialize the idle timer for the given device.
//
// Arguments:
//
//     FdoExtension    - Pointer to the device extension
//
// Return Value:
//
//     None
//
// --*/
// VOID
// ClasspInitializeIdleTimer(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     PCLASS_PRIVATE_FDO_DATA fdoData = FdoExtension->PrivateFdoData;
//     ULONG idleInterval = CLASS_IDLE_INTERVAL;
//     ULONG idlePrioritySupported = TRUE;
//     ULONG activeIdleIoMax = 1;
//
//     ClassGetDeviceParameter(FdoExtension,
//                             CLASSP_REG_SUBKEY_NAME,
//                             CLASSP_REG_IDLE_PRIORITY_SUPPORTED,
//                             &idlePrioritySupported);
//
//
//     if (idlePrioritySupported == FALSE) {
//         //
//         // User has set the registry to disable idle priority for this disk.
//         // No need to initialize any further.
//         // Always ensure that none of the other fields used for idle priority
//         // io are ever used without checking if it is supported.
//         //
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspInitializeIdleTimer: Idle priority not supported for disk %p\n", FdoExtension));
//         fdoData->IdlePrioritySupported = FALSE;
//         fdoData->IdleIoCount = 0;
//         fdoData->ActiveIoCount = 0;
//         return;
//     }
//
//     ClassGetDeviceParameter(FdoExtension,
//                             CLASSP_REG_SUBKEY_NAME,
//                             CLASSP_REG_IDLE_INTERVAL_NAME,
//                             &idleInterval);
//
//     if ((idleInterval < CLASS_IDLE_INTERVAL_MIN) || (idleInterval > USHORT_MAX)) {
//         //
//         // If the interval is too low or too high, reset it to the default value.
//         //
//         idleInterval = CLASS_IDLE_INTERVAL;
//     }
//
//     fdoData->IdlePrioritySupported = TRUE;
//     KeInitializeSpinLock(&fdoData->IdleListLock);
//     KeInitializeTimer(&fdoData->IdleTimer);
//     KeInitializeDpc(&fdoData->IdleDpc, ClasspIdleTimerDpc, FdoExtension);
//     InitializeListHead(&fdoData->IdleIrpList);
//     fdoData->IdleTimerStarted = FALSE;
//     fdoData->StarvationDuration = CLASS_STARVATION_INTERVAL;
//     fdoData->IdleInterval = (USHORT)(idleInterval);
//     fdoData->IdleIoCount = 0;
//     fdoData->ActiveIoCount = 0;
//     fdoData->ActiveIdleIoCount = 0;
//
//     ClassGetDeviceParameter(FdoExtension,
//                             CLASSP_REG_SUBKEY_NAME,
//                             CLASSP_REG_IDLE_ACTIVE_MAX,
//                             &activeIdleIoMax);
//
//     activeIdleIoMax = max(activeIdleIoMax, 1);
//     activeIdleIoMax = min(activeIdleIoMax, USHORT_MAX);
//
//     fdoData->IdleActiveIoMax = (USHORT)activeIdleIoMax;
//
//     return;
// }
//
// /*++
//
// ClasspStartIdleTimer
//
// Routine Description:
//
//     Start the idle timer if not already running. Reset the
//     timer counters before starting the timer. Use the IdleInterval
//     in the private fdo data to setup the timer.
//
// Arguments:
//
//     FdoData - Pointer to the private fdo data
//
// Return Value:
//
//     None
//
// --*/
// VOID
// ClasspStartIdleTimer(
//     IN PCLASS_PRIVATE_FDO_DATA FdoData
//     )
// {
//     LARGE_INTEGER dueTime;
//     LONG mstotimer;
//     LONG timerStarted;
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspStartIdleTimer: Start idle timer\n"));
//
//     timerStarted = InterlockedCompareExchange(&FdoData->IdleTimerStarted, 1, 0);
//
//     if (!timerStarted) {
//
//         //
//         // Reset the anti-starvation start time.
//         //
//         FdoData->AntiStarvationStartTime = ClasspGetCurrentTime();
//
//         //
//         // convert milliseconds to a relative 100ns
//         //
//         mstotimer = (-10) * 1000;
//
//         //
//         // multiply the period
//         //
//         dueTime.QuadPart = Int32x32To64(FdoData->IdleInterval, mstotimer);
//
//         KeSetTimerEx(&FdoData->IdleTimer,
//                      dueTime,
//                      FdoData->IdleInterval,
//                      &FdoData->IdleDpc);
//     }
//     return;
// }
//
// /*++
//
// ClasspStopIdleTimer
//
// Routine Description:
//
//     Stop the idle timer if running. Also reset the timer counters.
//
// Arguments:
//
//     FdoData - Pointer to the private fdo data
//
// Return Value:
//
//     None
//
// --*/
// VOID
// ClasspStopIdleTimer(
//     PCLASS_PRIVATE_FDO_DATA FdoData
//     )
// {
//     LONG timerStarted;
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspStopIdleTimer: Stop idle timer\n"));
//
//     timerStarted = InterlockedCompareExchange(&FdoData->IdleTimerStarted, 0, 1);
//
//     if (timerStarted) {
//         (VOID)KeCancelTimer(&FdoData->IdleTimer);
//     }
//     return;
// }
//
// /*++
//
// ClasspGetIdleTime
//
// Routine Description:
//
//     This routine returns how long it has been since the last non-idle request
//     completed by checking the actual time.
//
// Arguments:
//
//     FdoData - Pointer to the private fdo data
//
// Return Value:
//
//     The idle interval in ms.
//
// --*/
// ULONGLONG
// ClasspGetIdleTime (
//     IN PCLASS_PRIVATE_FDO_DATA FdoData,
//     IN LARGE_INTEGER CurrentTime
//     )
// {
//     ULONGLONG idleTime;
//     NTSTATUS status;
//
//     //
//     // Get the time difference between current time and last I/O
//     // complete time.
//     //
//
//     status = RtlULongLongSub((ULONGLONG)CurrentTime.QuadPart,
//                              (ULONGLONG)FdoData->LastNonIdleIoTime.QuadPart,
//                              &idleTime);
//
//     if (NT_SUCCESS(status)) {
//         //
//         // Convert the time to milliseconds.
//         //
//         idleTime = ClasspTimeDiffToMs(idleTime);
//     } else {
//         //
//         // Failed to get time difference, assume enough time passed.
//         //
//         idleTime = FdoData->IdleInterval;
//     }
//
//     return idleTime;
// }
//
// /*++
//
// ClasspIdleDurationSufficient
//
// Routine Description:
//
//     This routine computes whether enough idle duration has elapsed since the
//     completion of the last non-idle request.
//
// Arguments:
//
//     FdoData - Pointer to the private fdo data
//
//     CurrentTimeIn - If CurrentTimeIn is non-NULL
//         - contents are set to NULL if the time is not updated.
//         - time is updated otherwise
//
// Return Value:
//
//     TRUE if sufficient idle duration has elapsed to issue the next idle request.
//
// --*/
// LOGICAL
// ClasspIdleDurationSufficient (
//     IN PCLASS_PRIVATE_FDO_DATA FdoData,
//     OUT LARGE_INTEGER** CurrentTimeIn
//     )
// {
//     ULONGLONG idleInterval;
//     LARGE_INTEGER CurrentTime;
//
//     //
//     // If there are any outstanding non-idle requests, then there has been no
//     // idle time.
//     //
//     if (FdoData->ActiveIoCount > 0) {
//         if (CurrentTimeIn != NULL) {
//             *CurrentTimeIn = NULL;
//         }
//         return FALSE;
//     }
//
//     //
//     // Check whether an idle request should be issued now or on the next timer
//     // expiration.
//     //
//
//     CurrentTime = ClasspGetCurrentTime();
//     idleInterval = ClasspGetIdleTime(FdoData, CurrentTime);
//
//     if (CurrentTimeIn != NULL) {
//         **CurrentTimeIn = CurrentTime;
//     }
//
//     if (idleInterval >= FdoData->IdleInterval) {
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// /*++
//
// ClasspIdleTimerDpc
//
// Routine Description:
//
//     Timer dpc function. This function will be called once every
//     IdleInterval. An idle request will be queued if sufficient idle time
//     has elapsed since the last non-idle request.
//
// Arguments:
//
//     Dpc             - Pointer to DPC object
//     Context         - Pointer to the fdo device extension
//     SystemArgument1 - Not used
//     SystemArgument2 - Not used
//
// Return Value:
//
//     None
//
// --*/
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClasspIdleTimerDpc(
//     IN PKDPC Dpc,
//     IN PVOID Context,
//     IN PVOID SystemArgument1,
//     IN PVOID SystemArgument2
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Context;
//     PCLASS_PRIVATE_FDO_DATA fdoData;
//     ULONGLONG idleTime;
//     NTSTATUS status;
//     LARGE_INTEGER currentTime;
//     LARGE_INTEGER* pCurrentTime;
//
//     UNREFERENCED_PARAMETER(Dpc);
//     UNREFERENCED_PARAMETER(SystemArgument1);
//     UNREFERENCED_PARAMETER(SystemArgument2);
//
//     if (fdoExtension == NULL) {
//         NT_ASSERT(fdoExtension != NULL);
//         return;
//     }
//
//     fdoData = fdoExtension->PrivateFdoData;
//
//     if (fdoData->ActiveIoCount <= 0) {
//
//         //
//         // If there are max active idle requests, do not issue another one here.
//         //
//         if (fdoData->ActiveIdleIoCount >= fdoData->IdleActiveIoMax) {
//             return;
//         }
//
//         //
//         // Check whether enough idle time has passed since the last non-idle
//         // request has completed.
//         //
//
//         pCurrentTime = &currentTime;
//         if (ClasspIdleDurationSufficient(fdoData, &pCurrentTime)) {
//             //
//             // We are going to issue an idle request so reset the anti-starvation
//             // timer counter.
//             // If we are here (Idle duration is sufficient), pCurrentTime is
//             // expected to be set.
//             //
//             NT_ASSERT(pCurrentTime != NULL);
//             fdoData->AntiStarvationStartTime = *pCurrentTime;
//             ClasspServiceIdleRequest(fdoExtension, FALSE);
//         }
//         return;
//     }
//
//     //
//     // Get the time difference between current time and last I/O
//     // complete time.
//     //
//
//     currentTime = ClasspGetCurrentTime();
//     status = RtlULongLongSub((ULONGLONG)currentTime.QuadPart,
//                              (ULONGLONG)fdoData->AntiStarvationStartTime.QuadPart,
//                              &idleTime);
//
//     if (NT_SUCCESS(status)) {
//         //
//         // Convert the time to milliseconds.
//         //
//         idleTime = ClasspTimeDiffToMs(idleTime);
//     } else {
//         //
//         // Failed to get time difference, assume enough time passed.
//         //
//         idleTime = fdoData->StarvationDuration;
//     }
//
//     //
//     // If the timer is running then there must be at least one idle priority I/O pending
//     //
//     if (idleTime >= fdoData->StarvationDuration) {
//         fdoData->AntiStarvationStartTime = currentTime;
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspIdleTimerDpc: Starvation timer. Send one idle request\n"));
//         ClasspServiceIdleRequest(fdoExtension, FALSE);
//     }
//     return;
// }
//
// /*++
//
// ClasspEnqueueIdleRequest
//
// Routine Description:
//
//     This function will insert the idle request into the list.
//     If the inserted reqeust is the first request then it will
//     start the timer.
//
// Arguments:
//
//     DeviceObject    - Pointer to device object
//     Irp             - Pointer to the idle I/O request packet
//
// Return Value:
//
//     NT status code.
//
// --*/
// NTSTATUS
// ClasspEnqueueIdleRequest(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     KIRQL oldIrql;
//     BOOLEAN issueRequest = TRUE;
//     LARGE_INTEGER currentTime;
//     LARGE_INTEGER* pCurrentTime;
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspEnqueueIdleRequest: Queue idle request %p\n", Irp));
//
//     IoMarkIrpPending(Irp);
//
//     //
//     // Reset issueRequest if the idle duration is not sufficient.
//     //
//     pCurrentTime = &currentTime;
//     if (ClasspIdleDurationSufficient(fdoData, &pCurrentTime) == FALSE) {
//         issueRequest = FALSE;
//     }
//
//     //
//     // If there are already max active idle requests in the port driver, then
//     // queue this idle request.
//     //
//     if (fdoData->ActiveIdleIoCount >= fdoData->IdleActiveIoMax) {
//         issueRequest = FALSE;
//     }
//
//
//     KeAcquireSpinLock(&fdoData->IdleListLock, &oldIrql);
//     if (IsListEmpty(&fdoData->IdleIrpList)) {
//         NT_ASSERT(fdoData->IdleIoCount == 0);
//     }
//     InsertTailList(&fdoData->IdleIrpList, &Irp->Tail.Overlay.ListEntry);
//
//
//     fdoData->IdleIoCount++;
//     if (!fdoData->IdleTimerStarted) {
//         ClasspStartIdleTimer(fdoData);
//     }
//
//     if (fdoData->IdleIoCount != 1) {
//         issueRequest = FALSE;
//     }
//
//
//     KeReleaseSpinLock(&fdoData->IdleListLock, oldIrql);
//
//     if (issueRequest) {
//         ClasspServiceIdleRequest(fdoExtension, FALSE);
//     }
//
//     return STATUS_PENDING;
// }
//
// /*++
//
// ClasspDequeueIdleRequest
//
// Routine Description:
//
//     This function will remove the next idle request from the list.
//     If there are no requests in the queue, then it will return NULL.
//
// Arguments:
//
//     FdoExtension         - Pointer to the functional device extension
//
// Return Value:
//
//     Pointer to removed IRP
//
// --*/
// PIRP
// ClasspDequeueIdleRequest(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     PCLASS_PRIVATE_FDO_DATA fdoData = FdoExtension->PrivateFdoData;
//     PLIST_ENTRY listEntry = NULL;
//     PIRP irp = NULL;
//     KIRQL oldIrql;
//
//     KeAcquireSpinLock(&fdoData->IdleListLock, &oldIrql);
//
//     if (fdoData->IdleIoCount > 0) {
//         listEntry = RemoveHeadList(&fdoData->IdleIrpList);
//         //
//         // Make sure we actaully removed a request from the list
//         //
//         NT_ASSERT(listEntry != &fdoData->IdleIrpList);
//         //
//         // Decrement the idle I/O count.
//         //
//         fdoData->IdleIoCount--;
//         //
//         // Stop the timer on last request
//         //
//         if (fdoData->IdleIoCount == 0) {
//             ClasspStopIdleTimer(fdoData);
//         }
//         irp = CONTAINING_RECORD(listEntry, IRP, Tail.Overlay.ListEntry);
//         NT_ASSERT(irp->Type == IO_TYPE_IRP);
//
//
//         InitializeListHead(&irp->Tail.Overlay.ListEntry);
//     }
//
//     KeReleaseSpinLock(&fdoData->IdleListLock, oldIrql);
//
//
//     return irp;
// }
//
// /*++
//
// ClasspCompleteIdleRequest
//
// Routine Description:
//
//     This function will be called every time an idle request is completed.
//     This will call ClasspServiceIdleRequest to process any other pending idle requests.
//
// Arguments:
//
//     FdoExtension    - Pointer to the device extension
//
// Return Value:
//
//     None
//
// --*/
// VOID
// ClasspCompleteIdleRequest(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     PCLASS_PRIVATE_FDO_DATA fdoData = FdoExtension->PrivateFdoData;
//
//     //
//     // Issue the next idle request if there are any left in the queue, there are
//     // no non-idle requests outstanding, there are less than max idle requests
//     // outstanding, and it has been long enough since the completion of the last
//     // non-idle request.
//     //
//     if ((fdoData->IdleIoCount > 0) &&
//         (fdoData->ActiveIdleIoCount < fdoData->IdleActiveIoMax) &&
//         (fdoData->ActiveIoCount <= 0) &&
//         (ClasspIdleDurationSufficient(fdoData, NULL))) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_TIMER, "ClasspCompleteIdleRequest: Service next idle reqeusts\n"));
//         ClasspServiceIdleRequest(FdoExtension, TRUE);
//     }
//
//     return;
// }
//
// /*++
//
// ClasspServiceIdleRequest
//
// Routine Description:
//
//     Remove the next pending idle request from the queue and process it.
//     If a request was removed then it will be processed otherwise it will
//     just return.
//
// Arguments:
//
//     FdoExtension    - Pointer to the device extension
//     PostToDpc       - Flag to pass to ServiceTransferRequest to indicate if request must be posted to a DPC
//
// Return Value:
//
//     None
//
// --*/
// VOID
// ClasspServiceIdleRequest(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     BOOLEAN PostToDpc
//     )
// {
//     PIRP irp;
//
//     irp = ClasspDequeueIdleRequest(FdoExtension);
//     if (irp != NULL) {
//         ServiceTransferRequest(FdoExtension->DeviceObject, irp, PostToDpc);
//     }
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
