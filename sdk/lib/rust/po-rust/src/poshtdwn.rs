use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, PETHREAD, PEPROCESS, _UNICODE_STRING, HANDLE,
    KEVENT, KEY_VALUE_PARTIAL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION,
    LIST_ENTRY, _OBJECT_ATTRIBUTES, POWER_ACTION, PPOP_SHUTDOWN_WAIT_ENTRY,
    PWORK_QUEUE_ITEM, WORK_QUEUE_ITEM,
    _EVENT_TYPE_NotificationEvent,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
    KeInitializeEvent, KeInitializeGuardedMutex, KeWaitForSingleObject,
    KeSetEvent, KeRaiseIrqlToDpcLevel, KeSetSystemAffinityThread,
    ZwOpenKey, ZwQueryValueKey, ZwClose,
    RtlInitUnicodeString,
    PsGetNextProcess,
    IoShutdownSystem, CmShutdownSystem, ExShutdownSystem, MmShutdownSystem,
    CcShutdownSystem, CcWaitForCurrentLazyWriterActivity,
    HalReturnToFirmware, InbvIsBootDriverInstalled, InbvCheckDisplayOwnership,
    InbvAcquireDisplayOwnership, InbvResetDisplay,
    PsGetCurrentThread,
    PopSetSystemPowerState,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );
    fn HalEndOfBoot();
    fn HalSetWakeEnable(enable: BOOLEAN);
    static mut PsIdleProcess: PEPROCESS;
    static mut PopAction: PopActionStruct;
}

extern "system" {
    fn DbgUnLoadImageSymbols(name: PVOID, base: PVOID, process_id: usize);
}
extern "system" {
    fn HalHaltSystem();
}
extern "system" {
    fn ApphelpCacheShutdown();
}
extern "system" {
    fn DisplayShutdownBitmap();
}
extern "system" {
    fn DisplayShutdownText();
}

#[repr(C)]
struct PopActionStruct {
    Action: POWER_ACTION,
}

#[no_mangle]
pub static mut PopShutdownPowerOffPolicy: ULONG = 0;
#[no_mangle]
pub static mut PopShutdownEvent: KEVENT = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PopShutdownThreadList: PPOP_SHUTDOWN_WAIT_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut PopShutdownQueue: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut PopShutdownListMutex: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PopShutdownListAvailable: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn PopInitShutdownList() {
    // Initialize the global shutdown event
    KeInitializeEvent(
        core::ptr::addr_of_mut!(PopShutdownEvent),
        _EVENT_TYPE_NotificationEvent,
        0, // FALSE
    );

    // Initialize the shutdown lists
    PopShutdownThreadList = core::ptr::null_mut();
    InitializeListHead(core::ptr::addr_of_mut!(PopShutdownQueue) as PLIST_ENTRY);

    // Initialize the shutdown list lock
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex) as *mut _);

    // The list is available now
    PopShutdownListAvailable = 1; // TRUE
}

/// One serial line per shutdown phase, so a reboot that never happens names the phase it
/// stalls in (the install smoke's setup reboot sat silent for five minutes).
unsafe fn shutdown_phase(name: &[u8]) {
    ntkrnl_sys::driver_ddk::DbgPrint(c"[PO] shutdown: %s\n".as_ptr(), name.as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn PopGracefulShutdown(_Context: PVOID) {
    let _Context = opaque_ptr(_Context);
    shutdown_phase(b"begin (PopProcessShutDownLists)\0");
    // Process the registered waits and work items
    PopProcessShutDownLists();

    // Loop every process
    let mut Process: PEPROCESS = PsGetNextProcess(core::ptr::null_mut());
    while !Process.is_null() {
        // Make sure this isn't the idle or initial process
        if Process != PsInitialSystemProcess as PEPROCESS && Process != PsIdleProcess {
            // DPRINT1 - skip debug print in Rust
        }

        // Get the next process
        Process = PsGetNextProcess(Process);
    }

    // First, the HAL handles any "end of boot" special functionality
    shutdown_phase(b"HalEndOfBoot()\0");
    HalEndOfBoot();

    // Shut down the Shim cache if enabled
    shutdown_phase(b"ApphelpCacheShutdown()\0");
    ApphelpCacheShutdown();

    // In this step, the I/O manager does first-chance shutdown notification
    shutdown_phase(b"IoShutdownSystem(0)\0");
    IoShutdownSystem(0);

    // In this step, all workers are killed and hives are flushed
    shutdown_phase(b"CmShutdownSystem()\0");
    CmShutdownSystem();

    // Shut down the Executive
    shutdown_phase(b"ExShutdownSystem()\0");
    ExShutdownSystem();

    // Note that modified pages should be written here (MiShutdownSystem)
    shutdown_phase(b"MmShutdownSystem(0)\0");
    MmShutdownSystem(0);

    // Flush all user files before we start shutting down IO
    shutdown_phase(b"CcShutdownSystem()\0");
    CcShutdownSystem();

    // In this step, the I/O manager does last-chance shutdown notification
    shutdown_phase(b"IoShutdownSystem(1)\0");
    IoShutdownSystem(1);
    shutdown_phase(b"CcWaitForCurrentLazyWriterActivity()\0");
    CcWaitForCurrentLazyWriterActivity();

    // FIXME: Calling Mm shutdown phase 1 here to get page file dereference
    shutdown_phase(b"MmShutdownSystem(1)\0");
    MmShutdownSystem(1);

    // In this step, the HAL disables any wake timers
    shutdown_phase(b"HalSetWakeEnable(0)\0");
    HalSetWakeEnable(0); // FALSE

    // And finally the power request is sent
    shutdown_phase(b"PopShutdownSystem(PopAction.Action)\0");
    PopShutdownSystem(PopAction.Action);
}

#[no_mangle]
pub unsafe extern "system" fn PopReadShutdownPolicy() {
    let mut KeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;

    // Buffer for KEY_VALUE_PARTIAL_INFORMATION + ULONG
    let mut Buffer: [u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + core::mem::size_of::<ULONG>()] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + core::mem::size_of::<ULONG>()];
    let Info = Buffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;

    // Setup object attributes
    let key_path: &[u16] = &[
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16,
        b'r' as u16, b'e' as u16, b'\\' as u16, b'P' as u16, b'o' as u16, b'l' as u16,
        b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16,
        b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
        b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16,
        b'N' as u16, b'T' as u16, 0,
    ];
    RtlInitUnicodeString(&mut KeyString, key_path.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyString,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    // Open the key
    let mut Status = ZwOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        // Open the policy value and query it
        let value_name: &[u16] = &[
            b'D' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'P' as u16, b'o' as u16,
            b'w' as u16, b'e' as u16, b'r' as u16, b'O' as u16, b'f' as u16, b'f' as u16,
            b'A' as u16, b'f' as u16, b't' as u16, b'e' as u16, b'r' as u16, b'S' as u16,
            b'h' as u16, b'u' as u16, b't' as u16, b'd' as u16, b'o' as u16, b'w' as u16,
            b'n' as u16, 0,
        ];
        RtlInitUnicodeString(&mut KeyString, value_name.as_ptr());
        Status = ZwQueryValueKey(
            KeyHandle,
            &mut KeyString,
            _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
            Info as PVOID,
            Buffer.len() as ULONG,
            &mut Length,
        );
        if NT_SUCCESS(Status) && (*Info).Type == REG_DWORD {
            // Read the policy
            PopShutdownPowerOffPolicy = if *(*Info).Data.as_ptr() == 1 { 1 } else { 0 };
        }

        // Close the key
        ZwClose(KeyHandle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PoRequestShutdownWait(Thread: PETHREAD) -> NTSTATUS {
    let Thread = opaque_ptr(Thread);
    let ShutDownWaitEntry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PopShutdownWaitEntryStruct>() as usize,
        u32::from_be_bytes(*b"LSoP"),
    ) as PPOP_SHUTDOWN_WAIT_ENTRY;
    
    if ShutDownWaitEntry.is_null() {
        return STATUS_NO_MEMORY;
    }

    ObReferenceObject(Thread as PVOID);
    (*ShutDownWaitEntry).Thread = Thread;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    let Status = if PopShutdownListAvailable != 0 {
        (*ShutDownWaitEntry).NextEntry = PopShutdownThreadList;
        PopShutdownThreadList = ShutDownWaitEntry;
        STATUS_SUCCESS
    } else {
        ObDereferenceObject(Thread as PVOID);
        ExFreePoolWithTag(ShutDownWaitEntry as PVOID, u32::from_be_bytes(*b"LSoP"));
        STATUS_UNSUCCESSFUL
    };

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    Status
}

#[repr(C)]
struct PopShutdownWaitEntryStruct {
    Thread: PETHREAD,
    NextEntry: PPOP_SHUTDOWN_WAIT_ENTRY,
}

#[no_mangle]
pub unsafe extern "system" fn PopProcessShutDownLists() {
    KeSetEvent(core::ptr::addr_of_mut!(PopShutdownEvent), 0, 0);

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    PopShutdownListAvailable = 0;

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    while !IsListEmpty(core::ptr::addr_of_mut!(PopShutdownQueue) as PLIST_ENTRY) {
        let ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(PopShutdownQueue) as PLIST_ENTRY);
        let WorkItem = CONTAINING_RECORD(
            ListEntry as PVOID,
            core::mem::size_of::<WORK_QUEUE_ITEM>(),
            core::mem::offset_of!(WORK_QUEUE_ITEM, List),
        ) as PWORK_QUEUE_ITEM;

        ((*WorkItem).WorkerRoutine.unwrap())((*WorkItem).Parameter);
    }

    while !PopShutdownThreadList.is_null() {
        let ShutDownWaitEntry = PopShutdownThreadList;
        PopShutdownThreadList = (*PopShutdownThreadList).NextEntry;

        KeWaitForSingleObject(
            (*ShutDownWaitEntry).Thread as PVOID,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        ObDereferenceObject((*ShutDownWaitEntry).Thread as PVOID);

        ExFreePoolWithTag(ShutDownWaitEntry as PVOID, u32::from_be_bytes(*b"LSoP"));
    }
}

#[no_mangle]
pub unsafe extern "system" fn PopShutdownHandler() {
    KeRaiseIrqlToDpcLevel();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("cli");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("msr daifset, #2", options(nomem, nostack));

    if InbvIsBootDriverInstalled() != 0 {
        if InbvCheckDisplayOwnership() == 0 {
            InbvAcquireDisplayOwnership();
        }
        InbvResetDisplay();
        DisplayShutdownBitmap();
    } else {
        DisplayShutdownText();
    }

    loop {
        HalHaltSystem();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PopShutdownSystem(SystemAction: POWER_ACTION) {
    DbgUnLoadImageSymbols(core::ptr::null_mut(), (-1isize) as PVOID, 0);

    KeSetSystemAffinityThread(1);

    // Named, not numbered: the translation had ShutdownReset (5) and ShutdownOff (6) swapped,
    // with PowerSystemHibernate (5) for PowerSystemShutdown and PowerDown/Restart (1/2) for
    // Reboot/PowerDown. Every reboot request -- setup's, at the end of the install -- went to
    // the "safe to turn off" halt loop instead.
    use ntkrnl_sys::ddk::{
        _FIRMWARE_REENTRY_HalPowerDownRoutine as HalPowerDownRoutine,
        _FIRMWARE_REENTRY_HalRebootRoutine as HalRebootRoutine,
        _SYSTEM_POWER_STATE_PowerSystemShutdown as PowerSystemShutdown,
        POWER_ACTION_PowerActionShutdown as PowerActionShutdown,
        POWER_ACTION_PowerActionShutdownOff as PowerActionShutdownOff,
        POWER_ACTION_PowerActionShutdownReset as PowerActionShutdownReset,
    };
    match SystemAction as u32 {
        a if a == PowerActionShutdownReset as u32 => {
            PopSetSystemPowerState(PowerSystemShutdown as _, SystemAction);
            HalReturnToFirmware(HalRebootRoutine as _);
        }
        a if a == PowerActionShutdown as u32 || a == PowerActionShutdownOff as u32 => {
            // C: case PowerActionShutdown falls through to ShutdownOff; the power-off
            // policy check there has an empty (FIXFIX) body.
            PopSetSystemPowerState(PowerSystemShutdown as _, SystemAction);
            PopShutdownHandler();
            HalReturnToFirmware(HalPowerDownRoutine as _);
        }
        _ => {}
    }

    KeBugCheckEx(ntkrnl_sys::bugcodes::INTERNAL_POWER_ERROR, 5, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn PoQueueShutdownWorkItem(WorkItem: PWORK_QUEUE_ITEM) -> NTSTATUS {
    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    let Status = if PopShutdownListAvailable != 0 {
        InsertTailList(
            core::ptr::addr_of_mut!(PopShutdownQueue) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*WorkItem).List) as PLIST_ENTRY,
        );
        STATUS_SUCCESS
    } else {
        STATUS_SYSTEM_SHUTDOWN
    };

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopShutdownListMutex));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PoRequestShutdownEvent(Event: *mut PVOID) -> NTSTATUS {
    let Event = opaque_ptr(Event);
    if !Event.is_null() {
        *Event = core::ptr::null_mut();
    }

    let Status = PoRequestShutdownWait(PsGetCurrentThread());
    if Status < 0 {
        return Status;
    }

    if !Event.is_null() {
        *Event = core::ptr::addr_of_mut!(PopShutdownEvent) as PVOID;
    }
    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/po/poshtdwn.c
//  * PURPOSE:         Power Manager Shutdown Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #ifdef NEWCC
// #include <cache/newcc.h>
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// #include "inbv/logo.h"
//
// /* GLOBALS *******************************************************************/
//
// ULONG PopShutdownPowerOffPolicy;
// KEVENT PopShutdownEvent;
// PPOP_SHUTDOWN_WAIT_ENTRY PopShutdownThreadList;
// LIST_ENTRY PopShutdownQueue;
// KGUARDED_MUTEX PopShutdownListMutex;
// BOOLEAN PopShutdownListAvailable;
//
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// PopInitShutdownList(VOID)
// {
//     PAGED_CODE();
//
//     /* Initialize the global shutdown event */
//     KeInitializeEvent(&PopShutdownEvent, NotificationEvent, FALSE);
//
//     /* Initialize the shutdown lists */
//     PopShutdownThreadList = NULL;
//     InitializeListHead(&PopShutdownQueue);
//
//     /* Initialize the shutdown list lock */
//     KeInitializeGuardedMutex(&PopShutdownListMutex);
//
//     /* The list is available now */
//     PopShutdownListAvailable = TRUE;
// }
//
// NTSTATUS
// NTAPI
// PoRequestShutdownWait(
//     _In_ PETHREAD Thread)
// {
//     PPOP_SHUTDOWN_WAIT_ENTRY ShutDownWaitEntry;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Allocate a new shutdown wait entry */
//     ShutDownWaitEntry = ExAllocatePoolWithTag(PagedPool, sizeof(*ShutDownWaitEntry), 'LSoP');
//     if (ShutDownWaitEntry == NULL)
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Reference the thread and save it in the wait entry */
//     ObReferenceObject(Thread);
//     ShutDownWaitEntry->Thread = Thread;
//
//     /* Acquire the shutdown list lock */
//     KeAcquireGuardedMutex(&PopShutdownListMutex);
//
//     /* Check if the list is still available */
//     if (PopShutdownListAvailable)
//     {
//         /* Insert the item in the list */
//         ShutDownWaitEntry->NextEntry = PopShutdownThreadList;
//         PopShutdownThreadList = ShutDownWaitEntry;
//
//         /* We are successful */
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* We cannot proceed, cleanup and return failure */
//         ObDereferenceObject(Thread);
//         ExFreePoolWithTag(ShutDownWaitEntry, 'LSoP');
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Release the list lock */
//     KeReleaseGuardedMutex(&PopShutdownListMutex);
//
//     /* Return the status */
//     return Status;
// }
//
// VOID
// NTAPI
// PopProcessShutDownLists(VOID)
// {
//     PPOP_SHUTDOWN_WAIT_ENTRY ShutDownWaitEntry;
//     PWORK_QUEUE_ITEM WorkItem;
//     PLIST_ENTRY ListEntry;
//
//     /* First signal the shutdown event */
//     KeSetEvent(&PopShutdownEvent, IO_NO_INCREMENT, FALSE);
//
//     /* Acquire the shutdown list lock */
//     KeAcquireGuardedMutex(&PopShutdownListMutex);
//
//     /* Block any further attempts to register a shutdown event */
//     PopShutdownListAvailable = FALSE;
//
//     /* Release the list lock, since we are exclusively using the lists now */
//     KeReleaseGuardedMutex(&PopShutdownListMutex);
//
//     /* Process the shutdown queue */
//     while (!IsListEmpty(&PopShutdownQueue))
//     {
//         /* Get the head entry */
//         ListEntry = RemoveHeadList(&PopShutdownQueue);
//         WorkItem = CONTAINING_RECORD(ListEntry, WORK_QUEUE_ITEM, List);
//
//         /* Call the shutdown worker routine */
//         WorkItem->WorkerRoutine(WorkItem->Parameter);
//     }
//
//     /* Now process the shutdown thread list */
//     while (PopShutdownThreadList != NULL)
//     {
//         /* Get the top entry and remove it from the list */
//         ShutDownWaitEntry = PopShutdownThreadList;
//         PopShutdownThreadList = PopShutdownThreadList->NextEntry;
//
//         /* Wait for the thread to finish and dereference it */
//         KeWaitForSingleObject(ShutDownWaitEntry->Thread, 0, 0, 0, 0);
//         ObDereferenceObject(ShutDownWaitEntry->Thread);
//
//         /* Finally free the entry */
//         ExFreePoolWithTag(ShutDownWaitEntry, 'LSoP');
//     }
// }
//
// VOID
// NTAPI
// PopShutdownHandler(VOID)
// {
//     /* Stop all interrupts */
//     KeRaiseIrqlToDpcLevel();
//     _disable();
//
//     /* Do we have boot video */
//     if (InbvIsBootDriverInstalled())
//     {
//         /* Yes we do, cleanup for shutdown screen */
//         if (!InbvCheckDisplayOwnership()) InbvAcquireDisplayOwnership();
//         InbvResetDisplay();
//         // InbvEnableDisplayString(TRUE);
//         DisplayShutdownBitmap();
//     }
//     else
//     {
//         /* Do it in text-mode */
//         DisplayShutdownText();
//     }
//
//     /* Hang the system */
//     for (;;) HalHaltSystem();
// }
//
// VOID
// NTAPI
// PopShutdownSystem(IN POWER_ACTION SystemAction)
// {
//     /* Note should notify caller of NtPowerInformation(PowerShutdownNotification) */
//
//     /* Unload symbols */
//     DPRINT("It's the final countdown...%lx\n", SystemAction);
//     DbgUnLoadImageSymbols(NULL, (PVOID)-1, 0);
//
//     /* Run the thread on the boot processor */
//     KeSetSystemAffinityThread(1);
//
//     /* Now check what the caller wants */
//     switch (SystemAction)
//     {
//         /* Reset */
//         case PowerActionShutdownReset:
//
//             /* Try platform driver first, then legacy */
//             //PopInvokeSystemStateHandler(PowerStateShutdownReset, NULL);
//             PopSetSystemPowerState(PowerSystemShutdown, SystemAction);
//             HalReturnToFirmware(HalRebootRoutine);
//             break;
//
//         case PowerActionShutdown:
//
//             /* Check for group policy that says to use "it is now safe" screen */
//             if (PopShutdownPowerOffPolicy)
//             {
//                 /* FIXFIX: Switch to legacy shutdown handler */
//                 //PopPowerStateHandlers[PowerStateShutdownOff].Handler = PopShutdownHandler;
//             }
//
//         case PowerActionShutdownOff:
//
//             /* Call shutdown handler */
//             //PopInvokeSystemStateHandler(PowerStateShutdownOff, NULL);
//
//             /* ReactOS Hack */
//             PopSetSystemPowerState(PowerSystemShutdown, SystemAction);
//             PopShutdownHandler();
//
//             /* If that didn't work, call the HAL */
//             HalReturnToFirmware(HalPowerDownRoutine);
//             break;
//
//         default:
//             break;
//     }
//
//     /* Anything else should not happen */
//     KeBugCheckEx(INTERNAL_POWER_ERROR, 5, 0, 0, 0);
// }
//
// VOID
// NTAPI
// PopGracefulShutdown(IN PVOID Context)
// {
//     PEPROCESS Process = NULL;
//
//     /* Process the registered waits and work items */
//     PopProcessShutDownLists();
//
//     /* Loop every process */
//     Process = PsGetNextProcess(Process);
//     while (Process)
//     {
//         /* Make sure this isn't the idle or initial process */
//         if ((Process != PsInitialSystemProcess) && (Process != PsIdleProcess))
//         {
//             /* Print it */
//             DPRINT1("%15s is still RUNNING (%p)\n", Process->ImageFileName, Process->UniqueProcessId);
//         }
//
//         /* Get the next process */
//         Process = PsGetNextProcess(Process);
//     }
//
//     /* First, the HAL handles any "end of boot" special functionality */
//     DPRINT("HAL shutting down\n");
//     HalEndOfBoot();
//
//     /* Shut down the Shim cache if enabled */
//     ApphelpCacheShutdown();
//
//     /* In this step, the I/O manager does first-chance shutdown notification */
//     DPRINT("I/O manager shutting down in phase 0\n");
//     IoShutdownSystem(0);
//
//     /* In this step, all workers are killed and hives are flushed */
//     DPRINT("Configuration Manager shutting down\n");
//     CmShutdownSystem();
//
//     /* Shut down the Executive */
//     DPRINT("Executive shutting down\n");
//     ExShutdownSystem();
//
//     /* Note that modified pages should be written here (MiShutdownSystem) */
//     MmShutdownSystem(0);
//
//     /* Flush all user files before we start shutting down IO */
//     /* This is where modified pages are written back by the IO manager */
//     CcShutdownSystem();
//
//     /* In this step, the I/O manager does last-chance shutdown notification */
//     DPRINT("I/O manager shutting down in phase 1\n");
//     IoShutdownSystem(1);
//     CcWaitForCurrentLazyWriterActivity();
//
//     /* FIXME: Calling Mm shutdown phase 1 here to get page file dereference
//      * but it shouldn't be called here. Only phase 2 should be called.
//      */
//     MmShutdownSystem(1);
//
//     /* Note that here, we should broadcast the power IRP to devices */
//
//     /* In this step, the HAL disables any wake timers */
//     DPRINT("Disabling wake timers\n");
//     HalSetWakeEnable(FALSE);
//
//     /* And finally the power request is sent */
//     DPRINT("Taking the system down\n");
//     PopShutdownSystem(PopAction.Action);
// }
//
// VOID
// NTAPI
// PopReadShutdownPolicy(VOID)
// {
//     UNICODE_STRING KeyString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     ULONG Length;
//     UCHAR Buffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + sizeof(ULONG)];
//     PKEY_VALUE_PARTIAL_INFORMATION Info = (PVOID)Buffer;
//
//     /* Setup object attributes */
//     RtlInitUnicodeString(&KeyString,
//                          L"\\Registry\\Machine\\Software\\Policies\\Microsoft\\Windows NT");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyString,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Open the key */
//     Status = ZwOpenKey(&KeyHandle, KEY_READ, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Open the policy value and query it */
//         RtlInitUnicodeString(&KeyString, L"DontPowerOffAfterShutdown");
//         Status = ZwQueryValueKey(KeyHandle,
//                                  &KeyString,
//                                  KeyValuePartialInformation,
//                                  &Info,
//                                  sizeof(Info),
//                                  &Length);
//         if ((NT_SUCCESS(Status)) && (Info->Type == REG_DWORD))
//         {
//             /* Read the policy */
//             PopShutdownPowerOffPolicy = *Info->Data == 1;
//         }
//
//         /* Close the key */
//         ZwClose(KeyHandle);
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// PoQueueShutdownWorkItem(
//     _In_ PWORK_QUEUE_ITEM WorkItem)
// {
//     NTSTATUS Status;
//
//     /* Acquire the shutdown list lock */
//     KeAcquireGuardedMutex(&PopShutdownListMutex);
//
//     /* Check if the list is (already/still) available */
//     if (PopShutdownListAvailable)
//     {
//         /* Insert the item into the list */
//         InsertTailList(&PopShutdownQueue, &WorkItem->List);
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* We are already in shutdown */
//         Status = STATUS_SYSTEM_SHUTDOWN;
//     }
//
//     /* Release the list lock */
//     KeReleaseGuardedMutex(&PopShutdownListMutex);
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PoRequestShutdownEvent(OUT PVOID *Event)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Initialize to NULL */
//     if (Event) *Event = NULL;
//
//     /* Request a shutdown wait */
//     Status = PoRequestShutdownWait(PsGetCurrentThread());
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Return the global shutdown event */
//     if (Event) *Event = &PopShutdownEvent;
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
