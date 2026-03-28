use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _IO_TIMER, PIO_TIMER, PDEVICE_OBJECT, PIO_TIMER_ROUTINE,
    PEXTENDED_DEVOBJ_EXTENSION, KTIMER, PKSPIN_LOCK as DdkPKSPIN_LOCK,
};

// ExInterlockedInsertTailList is only in bindgen on amd64/arm64.
// On i386 it's ExfInterlockedInsertTailList (fastcall).
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::ExInterlockedInsertTailList;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedInsertTailList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedInsertTailList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedInsertTailList(ListHead, ListEntry, Lock)
}

#[no_mangle]
pub static mut IopTimerLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut IopTimerQueueHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut IopTimerDpc: KDPC = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut IopTimer: KTIMER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IopTimerCount: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn IopTimerDispatch(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let mut OldIrql: KIRQL = 0;
    let mut TimerEntry: PLIST_ENTRY;
    let mut Timer: PIO_TIMER;
    let mut i: ULONG;

    if IopTimerCount != 0 {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!(IopTimerLock),
            core::ptr::addr_of_mut!(OldIrql),
        );

        TimerEntry = IopTimerQueueHead.Flink;
        i = IopTimerCount;

        while TimerEntry != core::ptr::addr_of_mut!(IopTimerQueueHead) as PLIST_ENTRY && i != 0 {
            // CONTAINING_RECORD(TimerEntry, IO_TIMER, IoTimerList)
            Timer = (TimerEntry as usize - core::mem::offset_of!(_IO_TIMER, IoTimerList)) as PIO_TIMER;

            if (*Timer).TimerEnabled != 0 {
                ((*Timer).TimerRoutine.unwrap())((*Timer).DeviceObject, (*Timer).Context);
                i -= 1;
            }

            TimerEntry = (*TimerEntry).Flink;
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!(IopTimerLock), OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopRemoveTimerFromTimerList(Timer: PIO_TIMER) {
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(IopTimerLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    RemoveEntryList(core::ptr::addr_of_mut!((*Timer).IoTimerList));
    if (*Timer).TimerEnabled != 0 {
        IopTimerCount -= 1;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(IopTimerLock), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn IoInitializeTimer(
    DeviceObject: PDEVICE_OBJECT,
    TimerRoutine: PIO_TIMER_ROUTINE,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Context = opaque_ptr(Context);
    let mut IoTimer: PIO_TIMER = (*DeviceObject).Timer;

    if IoTimer.is_null() {
        IoTimer = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_IO_TIMER>(),
            0x546D4F49u32,
        ) as PIO_TIMER;

        if IoTimer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::write_bytes(IoTimer as *mut u8, 0, core::mem::size_of::<_IO_TIMER>());
        (*IoTimer).Type = 9;
        (*IoTimer).DeviceObject = DeviceObject;
        (*DeviceObject).Timer = IoTimer;
    }

    (*IoTimer).TimerRoutine = TimerRoutine;
    (*IoTimer).Context = Context;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(IopTimerQueueHead),
        core::ptr::addr_of_mut!((*IoTimer).IoTimerList),
        core::ptr::addr_of_mut!(IopTimerLock) as *mut _,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoStartTimer(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut OldIrql: KIRQL = 0;
    let IoTimer: PIO_TIMER = (*DeviceObject).Timer;

    let extension = (*DeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;
    let flags = (*extension).ExtensionFlags;

    if (flags & (0x00000001 | 0x00000002 | 0x00000004 | 0x00000008)) == 0 {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!(IopTimerLock),
            core::ptr::addr_of_mut!(OldIrql),
        );

        if (*IoTimer).TimerEnabled == 0 {
            (*IoTimer).TimerEnabled = 1u16;
            IopTimerCount += 1;
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!(IopTimerLock), OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoStopTimer(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut OldIrql: KIRQL = 0;
    let IoTimer: PIO_TIMER = (*DeviceObject).Timer;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!(IopTimerLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if (*IoTimer).TimerEnabled != 0 {
        (*IoTimer).TimerEnabled = 0u16;
        IopTimerCount -= 1;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(IopTimerLock), OldIrql);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/iotimer.c
//  * PURPOSE:         I/O Wrappers for Executive Timers
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* Timer Database */
// KSPIN_LOCK IopTimerLock;
// LIST_ENTRY IopTimerQueueHead;
//
// /* Timer Firing */
// KDPC IopTimerDpc;
// KTIMER IopTimer;
//
// /* Keep count of how many timers we have */
// ULONG IopTimerCount = 0;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// IopTimerDispatch(IN PKDPC Dpc,
//                  IN PVOID DeferredContext,
//                  IN PVOID SystemArgument1,
//                  IN PVOID SystemArgument2)
// {
//     KIRQL OldIrql;
//     PLIST_ENTRY TimerEntry;
//     PIO_TIMER Timer;
//     ULONG i;
//
//     /* Check if any Timers are actually enabled as of now */
//     if (IopTimerCount)
//     {
//         /* Lock the Timers */
//         KeAcquireSpinLock(&IopTimerLock, &OldIrql);
//
//         /* Call the Timer Routine of each enabled Timer */
//         for (TimerEntry = IopTimerQueueHead.Flink, i = IopTimerCount;
//             (TimerEntry != &IopTimerQueueHead) && i;
//             TimerEntry = TimerEntry->Flink)
//         {
//             /* Get the timer and check if it's enabled */
//             Timer = CONTAINING_RECORD(TimerEntry, IO_TIMER, IoTimerList);
//             if (Timer->TimerEnabled)
//             {
//                 /* Call the timer routine */
//                 Timer->TimerRoutine(Timer->DeviceObject, Timer->Context);
//                 i--;
//             }
//         }
//
//         /* Unlock the Timers */
//         KeReleaseSpinLock(&IopTimerLock, OldIrql);
//     }
// }
//
// VOID
// NTAPI
// IopRemoveTimerFromTimerList(IN PIO_TIMER Timer)
// {
//     KIRQL OldIrql;
//
//     /* Lock Timers */
//     KeAcquireSpinLock(&IopTimerLock, &OldIrql);
//
//     /* Remove Timer from the List and Drop the Timer Count if Enabled */
//     RemoveEntryList(&Timer->IoTimerList);
//     if (Timer->TimerEnabled) IopTimerCount--;
//
//     /* Unlock the Timers */
//     KeReleaseSpinLock(&IopTimerLock, OldIrql);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoInitializeTimer(IN PDEVICE_OBJECT DeviceObject,
//                   IN PIO_TIMER_ROUTINE TimerRoutine,
//                   IN PVOID Context)
// {
//     PIO_TIMER IoTimer = DeviceObject->Timer;
//     PAGED_CODE();
//
//     /* Check if we don't have a timer yet */
//     if (!IoTimer)
//     {
//         /* Allocate Timer */
//         IoTimer = ExAllocatePoolWithTag(NonPagedPool,
//                                         sizeof(IO_TIMER),
//                                         TAG_IO_TIMER);
//         if (!IoTimer) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Set up the Timer Structure */
//         RtlZeroMemory(IoTimer, sizeof(IO_TIMER));
//         IoTimer->Type = IO_TYPE_TIMER;
//         IoTimer->DeviceObject = DeviceObject;
//         DeviceObject->Timer = IoTimer;
//     }
//
//     /* Setup the timer routine and context */
//     IoTimer->TimerRoutine = TimerRoutine;
//     IoTimer->Context = Context;
//
//     /* Add it to the Timer List */
//     ExInterlockedInsertTailList(&IopTimerQueueHead,
//                                 &IoTimer->IoTimerList,
//                                 &IopTimerLock);
//
//     /* Return Success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoStartTimer(IN PDEVICE_OBJECT DeviceObject)
// {
//     KIRQL OldIrql;
//     PIO_TIMER IoTimer = DeviceObject->Timer;
//
//     /* Make sure the device isn't unloading */
//     if (!(((PEXTENDED_DEVOBJ_EXTENSION)(DeviceObject->DeviceObjectExtension))->
//             ExtensionFlags & (DOE_UNLOAD_PENDING |
//                               DOE_DELETE_PENDING |
//                               DOE_REMOVE_PENDING |
//                               DOE_REMOVE_PROCESSED)))
//     {
//         /* Lock Timers */
//         KeAcquireSpinLock(&IopTimerLock, &OldIrql);
//
//         /* Check if the timer isn't already enabled */
//         if (!IoTimer->TimerEnabled)
//         {
//             /* Enable it and increase the timer count */
//             IoTimer->TimerEnabled = TRUE;
//             IopTimerCount++;
//         }
//
//         /* Unlock Timers */
//         KeReleaseSpinLock(&IopTimerLock, OldIrql);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoStopTimer(PDEVICE_OBJECT DeviceObject)
// {
//     KIRQL OldIrql;
//     PIO_TIMER IoTimer = DeviceObject->Timer;
//
//     /* Lock Timers */
//     KeAcquireSpinLock(&IopTimerLock, &OldIrql);
//
//     /* Check if the timer is enabled */
//     if (IoTimer->TimerEnabled)
//     {
//         /* Disable it and decrease the timer count */
//         IoTimer->TimerEnabled = FALSE;
//         IopTimerCount--;
//     }
//
//     /* Unlock Timers */
//     KeReleaseSpinLock(&IopTimerLock, OldIrql);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
