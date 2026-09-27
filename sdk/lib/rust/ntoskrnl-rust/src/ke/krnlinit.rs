use super::ke_defs::*;
use super::timerobj::KiTimerTableListHead; // Shadow ke_defs extern — must use the actual #[no_mangle] definition
use super::bug::{KeBugcheckCallbackListHead, KeBugcheckReasonCallbackListHead, BugCheckCallbackLock};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _ETHREAD,
    _EPROCESS,
    _KNODE,
    PKNODE,
    PLOADER_PARAMETER_BLOCK,
    LARGE_INTEGER,
    _EVENT_TYPE_SynchronizationEvent,
    ExInitializeFastMutex,
    KeInitializeDpc,
    KeInitializeEvent,
    KeInitializeSpinLock,
    KeSetImportanceDpc,
    KeSetTargetProcessorDpc,
    KiInitMachineDependent,
    KiTimerExpiration,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub static mut KeProcessorArchitecture: USHORT = 0;

#[no_mangle]
pub static mut KeProcessorLevel: USHORT = 0;

#[no_mangle]
pub static mut KeProcessorRevision: USHORT = 0;

#[no_mangle]
pub static mut KeFeatureBits: u64 = 0;

#[no_mangle]
pub static mut KiServiceLimit: ULONG = NUMBER_OF_SYSCALLS;

#[no_mangle]
pub static mut KeLoaderBlock: PLOADER_PARAMETER_BLOCK = core::ptr::null_mut();

#[no_mangle]
pub static mut KiProcessorBlock: [PKPRCB; MAXIMUM_PROCESSORS as usize] = [core::ptr::null_mut(); MAXIMUM_PROCESSORS as usize];

#[no_mangle]
pub static mut KiNode0: _KNODE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KeNodeBlock: [PKNODE; 1] = unsafe { [core::ptr::addr_of_mut!(KiNode0)] };

#[no_mangle]
pub static mut KeNumberNodes: UCHAR = 1;

#[no_mangle]
pub static mut KeProcessNodeSeed: UCHAR = 0;

#[no_mangle]
pub static mut KiInitialThread: _ETHREAD = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KiInitialProcess: _EPROCESS = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KiDispatcherLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut MmPfnLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut MmSystemSpaceLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut CcBcbSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut CcMasterSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut CcVacbSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut CcWorkQueueSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut NonPagedPoolLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut MmNonPagedPoolLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut IopCancelSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut IopVpbSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut IopDatabaseLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut IopCompletionLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut NtfsStructLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut AfdWorkQueueSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KiTimerTableLock: [KSPIN_LOCK; LOCK_QUEUE_TIMER_TABLE_LOCKS] = [0; LOCK_QUEUE_TIMER_TABLE_LOCKS];

#[no_mangle]
pub static mut KiReverseStallIpiLock: KSPIN_LOCK = 0;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitSystem() {
    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckCallbackListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckReasonCallbackListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(BugCheckCallbackLock) as *mut _);

    KeInitializeDpc(
        core::ptr::addr_of_mut!(KiTimerExpireDpc),
        Some(KiTimerExpiration),
        core::ptr::null_mut(),
    );
    KeSetTargetProcessorDpc(core::ptr::addr_of_mut!(KiTimerExpireDpc), 0);

    KeInitializeSpinLock(core::ptr::addr_of_mut!(KiProfileLock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileSourceListHead));

    for i in 0..TIMER_TABLE_SIZE {
        InitializeListHead(core::ptr::addr_of_mut!(KiTimerTableListHead[i].Entry));
        KiTimerTableListHead[i].Time.u.HighPart = 0xFFFFFFFFu32;
        KiTimerTableListHead[i].Time.u.LowPart = 0;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(KiSwapEvent),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessInSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessOutSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiStackInSwapListHead));

    ExInitializeFastMutex(core::ptr::addr_of_mut!(KiGenericCallDpcMutex));

    KeServiceDescriptorTable[0].Base = core::ptr::addr_of_mut!(MainSSDT) as *mut _;
    KeServiceDescriptorTable[0].Count = core::ptr::null_mut();
    KeServiceDescriptorTable[0].Limit = KiServiceLimit;
    KeServiceDescriptorTable[1].Limit = 0;
    KeServiceDescriptorTable[0].Number = core::ptr::addr_of_mut!(MainSSPT) as *mut u8;

    RtlCopyMemory(
        core::ptr::addr_of_mut!(KeServiceDescriptorTableShadow) as PVOID,
        core::ptr::addr_of!(KeServiceDescriptorTable) as PVOID,
        core::mem::size_of_val(&KeServiceDescriptorTable),
    );
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiComputeReciprocal(
    Divisor: LONG,
    Shift: *mut UCHAR,
) -> LARGE_INTEGER {
    let Shift = opaque_ptr(Shift);
    let mut Reciprocal: LARGE_INTEGER = core::mem::zeroed();
    Reciprocal.QuadPart = 0;
    let mut BitCount: LONG = 0;
    let mut Remainder: LONG = 1;

    while Reciprocal.u.HighPart >= 0 {
        BitCount += 1;

        Reciprocal.u.HighPart =
            (Reciprocal.u.HighPart << 1) |
            ((Reciprocal.u.LowPart >> 31) as i32);
        Reciprocal.u.LowPart <<= 1;

        Remainder <<= 1;
        if Remainder >= Divisor {
            Remainder -= Divisor;
            Reciprocal.u.LowPart |= 1;
        }
    }

    if Remainder != 0 {
        if Reciprocal.u.LowPart == 0xFFFFFFFF &&
           Reciprocal.u.HighPart == 0xFFFFFFFFu32 as i32 {
            Reciprocal.u.LowPart = 0;
            Reciprocal.u.HighPart = 0x80000000u32 as i32;
            BitCount -= 1;
        } else {
            if Reciprocal.u.LowPart == 0xFFFFFFFF {
                Reciprocal.u.LowPart = 0;
                Reciprocal.u.HighPart += 1;
            } else {
                Reciprocal.u.LowPart += 1;
            }
        }
    }

    *Shift = (BitCount - 64) as UCHAR;
    Reciprocal
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitSpinLocks(Prcb: PKPRCB, Number: i8) {
    (*Prcb).QueueIndex = 1;
    (*Prcb).ReadySummary = 0;
    (*Prcb).DeferredReadyListHead.Next = core::ptr::null_mut();

    for i in 0..MAXIMUM_PRIORITY as usize {
        InitializeListHead(core::ptr::addr_of_mut!((*Prcb).DispatcherReadyListHead[i]));
    }

    InitializeListHead(core::ptr::addr_of_mut!((*Prcb).DpcData[DPC_NORMAL as usize].DpcListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Prcb).DpcData[DPC_NORMAL as usize].DpcLock));
    (*Prcb).DpcData[DPC_NORMAL as usize].DpcQueueDepth = 0;
    (*Prcb).DpcData[DPC_NORMAL as usize].DpcCount = 0;
    (*Prcb).DpcRoutineActive = 0u8;
    (*Prcb).MaximumDpcQueueDepth = KiMaximumDpcQueueDepth as _;
    (*Prcb).MinimumDpcRate = KiMinimumDpcRate;
    (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Prcb).CallDpc),
        None,
        core::ptr::null_mut(),
    );
    KeSetTargetProcessorDpc(core::ptr::addr_of_mut!((*Prcb).CallDpc), Number);
    KeSetImportanceDpc(core::ptr::addr_of_mut!((*Prcb).CallDpc), HighImportance);

    InitializeListHead(core::ptr::addr_of_mut!((*Prcb).WaitListHead));

    (*Prcb).LockQueue[LockQueueDispatcherLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueDispatcherLock as usize].Lock = core::ptr::addr_of_mut!(KiDispatcherLock) as *mut _;
    (*Prcb).LockQueue[LockQueueExpansionLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueExpansionLock as usize].Lock = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueuePfnLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueuePfnLock as usize].Lock = core::ptr::addr_of_mut!(MmPfnLock) as *mut _;
    (*Prcb).LockQueue[LockQueueSystemSpaceLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueSystemSpaceLock as usize].Lock = core::ptr::addr_of_mut!(MmSystemSpaceLock) as *mut _;
    (*Prcb).LockQueue[LockQueueVacbLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueVacbLock as usize].Lock = core::ptr::addr_of_mut!(CcVacbSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueMasterLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueMasterLock as usize].Lock = core::ptr::addr_of_mut!(CcMasterSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueNonPagedPoolLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueNonPagedPoolLock as usize].Lock = core::ptr::addr_of_mut!(NonPagedPoolLock) as *mut _;
    (*Prcb).LockQueue[LockQueueIoCancelLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueIoCancelLock as usize].Lock = core::ptr::addr_of_mut!(IopCancelSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueWorkQueueLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueWorkQueueLock as usize].Lock = core::ptr::addr_of_mut!(CcWorkQueueSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueIoVpbLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueIoVpbLock as usize].Lock = core::ptr::addr_of_mut!(IopVpbSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueIoDatabaseLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueIoDatabaseLock as usize].Lock = core::ptr::addr_of_mut!(IopDatabaseLock) as *mut _;
    (*Prcb).LockQueue[LockQueueIoCompletionLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueIoCompletionLock as usize].Lock = core::ptr::addr_of_mut!(IopCompletionLock) as *mut _;
    (*Prcb).LockQueue[LockQueueNtfsStructLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueNtfsStructLock as usize].Lock = core::ptr::addr_of_mut!(NtfsStructLock) as *mut _;
    (*Prcb).LockQueue[LockQueueAfdWorkQueueLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueAfdWorkQueueLock as usize].Lock = core::ptr::addr_of_mut!(AfdWorkQueueSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueBcbLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueBcbLock as usize].Lock = core::ptr::addr_of_mut!(CcBcbSpinLock) as *mut _;
    (*Prcb).LockQueue[LockQueueMmNonPagedPoolLock as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueMmNonPagedPoolLock as usize].Lock = core::ptr::addr_of_mut!(MmNonPagedPoolLock) as *mut _;
    (*Prcb).LockQueue[LockQueueUnusedSpare16 as usize].Next = core::ptr::null_mut();
    (*Prcb).LockQueue[LockQueueUnusedSpare16 as usize].Lock = core::ptr::null_mut();

    for i in 0..LOCK_QUEUE_TIMER_TABLE_LOCKS {
        if Number == 0 {
            KeInitializeSpinLock(core::ptr::addr_of_mut!(KiTimerTableLock[i]) as *mut _);
        }

        (*Prcb).LockQueue[LockQueueTimerTableLock as usize + i].Next = core::ptr::null_mut();
        (*Prcb).LockQueue[LockQueueTimerTableLock as usize + i].Lock =
            core::ptr::addr_of_mut!(KiTimerTableLock[i]) as *mut _;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Prcb).PrcbLock) as *mut _);

    if Number == 0 {
        KeInitializeSpinLock(core::ptr::addr_of_mut!(KiDispatcherLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(KiReverseStallIpiLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(MmPfnLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(MmSystemSpaceLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(CcBcbSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(CcMasterSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(CcVacbSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(CcWorkQueueSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(IopCancelSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(IopCompletionLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(IopDatabaseLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(IopVpbSpinLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(NonPagedPoolLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(MmNonPagedPoolLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(NtfsStructLock) as *mut _);
        KeInitializeSpinLock(core::ptr::addr_of_mut!(AfdWorkQueueSpinLock) as *mut _);
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KeInitSystem() -> BOOLEAN {
    if KeThreadDpcEnable != 0 {
        // DPRINT1("Threaded DPCs not yet supported\n");
    }

    KiInitMachineDependent();
    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/krnlinit.c
//  * PURPOSE:         Portable part of kernel initialization
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include <internal/napi.h>
//
// /* GLOBALS *******************************************************************/
//
// /* Portable CPU Features and Flags */
// USHORT KeProcessorArchitecture;
// USHORT KeProcessorLevel;
// USHORT KeProcessorRevision;
// ULONG64 KeFeatureBits;
//
// /* System call count */
// ULONG KiServiceLimit = NUMBER_OF_SYSCALLS;
//
// /* ARC Loader Block */
// PLOADER_PARAMETER_BLOCK KeLoaderBlock;
//
// /* PRCB Array */
// PKPRCB KiProcessorBlock[MAXIMUM_PROCESSORS];
//
// /* NUMA Node Support */
// KNODE KiNode0;
// PKNODE KeNodeBlock[1] = { &KiNode0 };
// UCHAR KeNumberNodes = 1;
// UCHAR KeProcessNodeSeed;
//
// /* Initial Process and Thread */
// ETHREAD KiInitialThread;
// EPROCESS KiInitialProcess;
//
// /* System-defined Spinlocks */
// KSPIN_LOCK KiDispatcherLock;
// KSPIN_LOCK MmPfnLock;
// KSPIN_LOCK MmSystemSpaceLock;
// KSPIN_LOCK CcBcbSpinLock;
// KSPIN_LOCK CcMasterSpinLock;
// KSPIN_LOCK CcVacbSpinLock;
// KSPIN_LOCK CcWorkQueueSpinLock;
// KSPIN_LOCK NonPagedPoolLock;
// KSPIN_LOCK MmNonPagedPoolLock;
// KSPIN_LOCK IopCancelSpinLock;
// KSPIN_LOCK IopVpbSpinLock;
// KSPIN_LOCK IopDatabaseLock;
// KSPIN_LOCK IopCompletionLock;
// KSPIN_LOCK NtfsStructLock;
// KSPIN_LOCK AfdWorkQueueSpinLock;
// KSPIN_LOCK KiTimerTableLock[LOCK_QUEUE_TIMER_TABLE_LOCKS];
// KSPIN_LOCK KiReverseStallIpiLock;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitSystem(VOID)
// {
//     ULONG i;
//
//     /* Initialize Bugcheck Callback data */
//     InitializeListHead(&KeBugcheckCallbackListHead);
//     InitializeListHead(&KeBugcheckReasonCallbackListHead);
//     KeInitializeSpinLock(&BugCheckCallbackLock);
//
//     /* Initialize the Timer Expiration DPC */
//     KeInitializeDpc(&KiTimerExpireDpc, KiTimerExpiration, NULL);
//     KeSetTargetProcessorDpc(&KiTimerExpireDpc, 0);
//
//     /* Initialize Profiling data */
//     KeInitializeSpinLock(&KiProfileLock);
//     InitializeListHead(&KiProfileListHead);
//     InitializeListHead(&KiProfileSourceListHead);
//
//     /* Loop the timer table */
//     for (i = 0; i < TIMER_TABLE_SIZE; i++)
//     {
//         /* Initialize the list and entries */
//         InitializeListHead(&KiTimerTableListHead[i].Entry);
//         KiTimerTableListHead[i].Time.HighPart = 0xFFFFFFFF;
//         KiTimerTableListHead[i].Time.LowPart = 0;
//     }
//
//     /* Initialize the Swap event and all swap lists */
//     KeInitializeEvent(&KiSwapEvent, SynchronizationEvent, FALSE);
//     InitializeListHead(&KiProcessInSwapListHead);
//     InitializeListHead(&KiProcessOutSwapListHead);
//     InitializeListHead(&KiStackInSwapListHead);
//
//     /* Initialize the mutex for generic DPC calls */
//     ExInitializeFastMutex(&KiGenericCallDpcMutex);
//
//     /* Initialize the syscall table */
//     KeServiceDescriptorTable[0].Base = MainSSDT;
//     KeServiceDescriptorTable[0].Count = NULL;
//     KeServiceDescriptorTable[0].Limit = KiServiceLimit;
//     KeServiceDescriptorTable[1].Limit = 0;
//     KeServiceDescriptorTable[0].Number = MainSSPT;
//
//     /* Copy the the current table into the shadow table for win32k */
//     RtlCopyMemory(KeServiceDescriptorTableShadow,
//                   KeServiceDescriptorTable,
//                   sizeof(KeServiceDescriptorTable));
// }
//
// CODE_SEG("INIT")
// LARGE_INTEGER
// NTAPI
// KiComputeReciprocal(IN LONG Divisor,
//                     OUT PUCHAR Shift)
// {
//     LARGE_INTEGER Reciprocal = {{0, 0}};
//     LONG BitCount = 0, Remainder = 1;
//
//     /* Start by calculating the remainder */
//     while (Reciprocal.HighPart >= 0)
//     {
//         /* Increase the loop (bit) count */
//         BitCount++;
//
//         /* Calculate the current fraction */
//         Reciprocal.HighPart = (Reciprocal.HighPart << 1) |
//                               (Reciprocal.LowPart >> 31);
//         Reciprocal.LowPart <<= 1;
//
//         /* Double the remainder and see if we went past the divisor */
//         Remainder <<= 1;
//         if (Remainder >= Divisor)
//         {
//             /* Set the low-bit and calculate the new remainder */
//             Remainder -= Divisor;
//             Reciprocal.LowPart |= 1;
//         }
//     }
//
//     /* Check if we have a remainder */
//     if (Remainder)
//     {
//         /* Check if the current fraction value is too large */
//         if ((Reciprocal.LowPart == 0xFFFFFFFF) &&
//             (Reciprocal.HighPart == (LONG)0xFFFFFFFF))
//         {
//             /* Set the high bit and reduce the bit count */
//             Reciprocal.LowPart = 0;
//             Reciprocal.HighPart = 0x80000000;
//             BitCount--;
//         }
//         else
//         {
//             /* Check if only the lowest bits got too large */
//             if (Reciprocal.LowPart == 0xFFFFFFFF)
//             {
//                 /* Reset them and increase the high bits instead */
//                 Reciprocal.LowPart = 0;
//                 Reciprocal.HighPart++;
//             }
//             else
//             {
//                 /* All is well, increase the low bits */
//                 Reciprocal.LowPart++;
//             }
//         }
//     }
//
//     /* Now calculate the actual shift and return the reciprocal */
//     *Shift = (UCHAR)BitCount - 64;
//     return Reciprocal;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitSpinLocks(IN PKPRCB Prcb,
//                 IN CCHAR Number)
// {
//     ULONG i;
//
//     /* Initialize Dispatcher Fields */
//     Prcb->QueueIndex = 1;
//     Prcb->ReadySummary = 0;
//     Prcb->DeferredReadyListHead.Next = NULL;
//     for (i = 0; i < MAXIMUM_PRIORITY; i++)
//     {
//         /* Initialize the ready list */
//         InitializeListHead(&Prcb->DispatcherReadyListHead[i]);
//     }
//
//     /* Initialize DPC Fields */
//     InitializeListHead(&Prcb->DpcData[DPC_NORMAL].DpcListHead);
//     KeInitializeSpinLock(&Prcb->DpcData[DPC_NORMAL].DpcLock);
//     Prcb->DpcData[DPC_NORMAL].DpcQueueDepth = 0;
//     Prcb->DpcData[DPC_NORMAL].DpcCount = 0;
//     Prcb->DpcRoutineActive = FALSE;
//     Prcb->MaximumDpcQueueDepth = KiMaximumDpcQueueDepth;
//     Prcb->MinimumDpcRate = KiMinimumDpcRate;
//     Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//     KeInitializeDpc(&Prcb->CallDpc, NULL, NULL);
//     KeSetTargetProcessorDpc(&Prcb->CallDpc, Number);
//     KeSetImportanceDpc(&Prcb->CallDpc, HighImportance);
//
//     /* Initialize the Wait List Head */
//     InitializeListHead(&Prcb->WaitListHead);
//
//     /* Initialize Queued Spinlocks */
//     Prcb->LockQueue[LockQueueDispatcherLock].Next = NULL;
//     Prcb->LockQueue[LockQueueDispatcherLock].Lock = &KiDispatcherLock;
//     Prcb->LockQueue[LockQueueExpansionLock].Next = NULL;
//     Prcb->LockQueue[LockQueueExpansionLock].Lock = NULL;
//     Prcb->LockQueue[LockQueuePfnLock].Next = NULL;
//     Prcb->LockQueue[LockQueuePfnLock].Lock = &MmPfnLock;
//     Prcb->LockQueue[LockQueueSystemSpaceLock].Next = NULL;
//     Prcb->LockQueue[LockQueueSystemSpaceLock].Lock = &MmSystemSpaceLock;
//     Prcb->LockQueue[LockQueueBcbLock].Next = NULL;
//     Prcb->LockQueue[LockQueueBcbLock].Lock = &CcBcbSpinLock;
//     Prcb->LockQueue[LockQueueMasterLock].Next = NULL;
//     Prcb->LockQueue[LockQueueMasterLock].Lock = &CcMasterSpinLock;
//     Prcb->LockQueue[LockQueueVacbLock].Next = NULL;
//     Prcb->LockQueue[LockQueueVacbLock].Lock = &CcVacbSpinLock;
//     Prcb->LockQueue[LockQueueWorkQueueLock].Next = NULL;
//     Prcb->LockQueue[LockQueueWorkQueueLock].Lock = &CcWorkQueueSpinLock;
//     Prcb->LockQueue[LockQueueNonPagedPoolLock].Next = NULL;
//     Prcb->LockQueue[LockQueueNonPagedPoolLock].Lock = &NonPagedPoolLock;
//     Prcb->LockQueue[LockQueueMmNonPagedPoolLock].Next = NULL;
//     Prcb->LockQueue[LockQueueMmNonPagedPoolLock].Lock = &MmNonPagedPoolLock;
//     Prcb->LockQueue[LockQueueIoCancelLock].Next = NULL;
//     Prcb->LockQueue[LockQueueIoCancelLock].Lock = &IopCancelSpinLock;
//     Prcb->LockQueue[LockQueueIoVpbLock].Next = NULL;
//     Prcb->LockQueue[LockQueueIoVpbLock].Lock = &IopVpbSpinLock;
//     Prcb->LockQueue[LockQueueIoDatabaseLock].Next = NULL;
//     Prcb->LockQueue[LockQueueIoDatabaseLock].Lock = &IopDatabaseLock;
//     Prcb->LockQueue[LockQueueIoCompletionLock].Next = NULL;
//     Prcb->LockQueue[LockQueueIoCompletionLock].Lock = &IopCompletionLock;
//     Prcb->LockQueue[LockQueueNtfsStructLock].Next = NULL;
//     Prcb->LockQueue[LockQueueNtfsStructLock].Lock = &NtfsStructLock;
//     Prcb->LockQueue[LockQueueAfdWorkQueueLock].Next = NULL;
//     Prcb->LockQueue[LockQueueAfdWorkQueueLock].Lock = &AfdWorkQueueSpinLock;
//     Prcb->LockQueue[LockQueueUnusedSpare16].Next = NULL;
//     Prcb->LockQueue[LockQueueUnusedSpare16].Lock = NULL;
//
//     /* Loop timer locks (shared amongst all CPUs) */
//     for (i = 0; i < LOCK_QUEUE_TIMER_TABLE_LOCKS; i++)
//     {
//         /* Setup the Queued Spinlock (done only once by the boot CPU) */
//         if (!Number)
//             KeInitializeSpinLock(&KiTimerTableLock[i]);
//
//         /* Initialize the lock */
//         Prcb->LockQueue[LockQueueTimerTableLock + i].Next = NULL;
//         Prcb->LockQueue[LockQueueTimerTableLock + i].Lock =
//             &KiTimerTableLock[i];
//     }
//
//     /* Initialize the PRCB lock */
//     KeInitializeSpinLock(&Prcb->PrcbLock);
//
//     /* Check if this is the boot CPU */
//     if (!Number)
//     {
//         /* Initialize the lock themselves */
//         KeInitializeSpinLock(&KiDispatcherLock);
//         KeInitializeSpinLock(&KiReverseStallIpiLock);
//         KeInitializeSpinLock(&MmPfnLock);
//         KeInitializeSpinLock(&MmSystemSpaceLock);
//         KeInitializeSpinLock(&CcBcbSpinLock);
//         KeInitializeSpinLock(&CcMasterSpinLock);
//         KeInitializeSpinLock(&CcVacbSpinLock);
//         KeInitializeSpinLock(&CcWorkQueueSpinLock);
//         KeInitializeSpinLock(&IopCancelSpinLock);
//         KeInitializeSpinLock(&IopCompletionLock);
//         KeInitializeSpinLock(&IopDatabaseLock);
//         KeInitializeSpinLock(&IopVpbSpinLock);
//         KeInitializeSpinLock(&NonPagedPoolLock);
//         KeInitializeSpinLock(&MmNonPagedPoolLock);
//         KeInitializeSpinLock(&NtfsStructLock);
//         KeInitializeSpinLock(&AfdWorkQueueSpinLock);
//     }
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// KeInitSystem(VOID)
// {
//     /* Check if Threaded DPCs are enabled */
//     if (KeThreadDpcEnable)
//     {
//         /* FIXME: TODO */
//         DPRINT1("Threaded DPCs not yet supported\n");
//     }
//
//     /* Initialize non-portable parts of the kernel */
//     KiInitMachineDependent();
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
