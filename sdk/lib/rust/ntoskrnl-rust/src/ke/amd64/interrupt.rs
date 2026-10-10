use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _KINTERRUPT,
    PKINTERRUPT,
    PKSERVICE_ROUTINE,
    PKSYNCHRONIZE_ROUTINE,
    HalEnableSystemInterrupt,
    HalDisableSystemInterrupt,
    KeInitializeSpinLock,
    KeSetSystemAffinityThread,
    KeRevertToUserAffinityThread,
    KiAcquireDispatcherLock,
    KiReleaseDispatcherLock,
    KINTERRUPT_MODE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static KiInterruptDispatchTemplate: [UCHAR; 16];
    static KiUnexpectedRange: [KI_INTERRUPT_DISPATCH_ENTRY; 256];
    static KiUnexpectedRangeEnd: [KI_INTERRUPT_DISPATCH_ENTRY; 0];
    fn KiInterruptDispatch();
}
extern "C" {
    fn KeQueryInterruptHandler(Vector: ULONG) -> PVOID;
    fn KeRegisterInterruptHandler(Vector: ULONG, Handler: PVOID);
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}

#[repr(C, packed)]
struct KI_INTERRUPT_DISPATCH_ENTRY {
    _Op_nop: UCHAR,
    _Op_push: UCHAR,
    _Vector: UCHAR,
    _Op_jmp: UCHAR,
    RelativeAddress: ULONG,
}

const PRIMARY_VECTOR_BASE: ULONG = 0x30;
const MAXIMUM_IDTVECTOR: ULONG = 0xFF;

const InterruptObject: USHORT = 0x16;

#[no_mangle]
pub unsafe extern "system" fn KeInitializeInterrupt(
    Interrupt: PKINTERRUPT,
    ServiceRoutine: PKSERVICE_ROUTINE,
    ServiceContext: PVOID,
    SpinLock: PKSPIN_LOCK,
    Vector: ULONG,
    Irql: KIRQL,
    SynchronizeIrql: KIRQL,
    InterruptMode: KINTERRUPT_MODE,
    ShareVector: BOOLEAN,
    ProcessorNumber: CHAR,
    FloatingSave: BOOLEAN,
) {
    let Interrupt = opaque_ptr(Interrupt);
    let ServiceContext = opaque_ptr(ServiceContext);
    (*Interrupt).Type = InterruptObject as i16;
    (*Interrupt).Size = core::mem::size_of::<_KINTERRUPT>() as i16;

    let ActualSpinLock = if SpinLock.is_null() {
        core::ptr::addr_of_mut!((*Interrupt).SpinLock) as *mut u64
    } else {
        SpinLock as *mut u64
    };
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Interrupt).SpinLock));

    (*Interrupt).ServiceRoutine = ServiceRoutine;
    (*Interrupt).ServiceContext = ServiceContext;
    (*Interrupt).ActualLock = ActualSpinLock as *mut u64;
    (*Interrupt).Vector = Vector;
    (*Interrupt).Irql = Irql;
    (*Interrupt).SynchronizeIrql = SynchronizeIrql;
    (*Interrupt).Mode = InterruptMode;
    (*Interrupt).ShareVector = ShareVector;
    (*Interrupt).Number = ProcessorNumber;
    (*Interrupt).FloatingSave = FloatingSave;

    (*Interrupt).TickCount = 0;
    (*Interrupt).Connected = 0u8;
    (*Interrupt).ServiceCount = 0;
    (*Interrupt).DispatchCount = 0;
    (*Interrupt).TrapFrame = core::ptr::null_mut();
    (*Interrupt).Reserved = core::ptr::null_mut();

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!(KiInterruptDispatchTemplate) as *const u8,
        core::ptr::addr_of_mut!((*Interrupt).DispatchCode) as *mut u8,
        core::mem::size_of_val(&(*Interrupt).DispatchCode),
    );

    (*Interrupt).DispatchAddress = None;
}

#[no_mangle]
pub unsafe extern "system" fn KeConnectInterrupt(Interrupt: PKINTERRUPT) -> BOOLEAN {
    let Interrupt = opaque_ptr(Interrupt);
    if ((*Interrupt).Vector < PRIMARY_VECTOR_BASE) || ((*Interrupt).Vector > MAXIMUM_IDTVECTOR) {
        return 0u8;
    }

    if (*Interrupt).Connected != 0 {
        return 1u8;
    }

    KeSetSystemAffinityThread(1u64 << (*Interrupt).Number);
    let OldIrql = KiAcquireDispatcherLock();

    let CurrentHandler = KeQueryInterruptHandler((*Interrupt).Vector);

    let unexpected_range_start = core::ptr::addr_of!(KiUnexpectedRange) as PVOID;
    let unexpected_range_end = core::ptr::addr_of!(KiUnexpectedRangeEnd) as PVOID;

    if (CurrentHandler >= unexpected_range_start) && (CurrentHandler <= unexpected_range_end) {
        InitializeListHead(core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry) as PLIST_ENTRY);

        (*Interrupt).DispatchAddress = core::mem::transmute(KiInterruptDispatch as unsafe extern "system" fn());

        KeRegisterInterruptHandler(
            (*Interrupt).Vector,
            core::ptr::addr_of_mut!((*Interrupt).DispatchCode) as PVOID,
        );

        if HalEnableSystemInterrupt((*Interrupt).Vector, (*Interrupt).Irql, (*Interrupt).Mode) == 0 {
            KeRegisterInterruptHandler((*Interrupt).Vector, CurrentHandler);
            KiReleaseDispatcherLock(OldIrql);
            KeRevertToUserAffinityThread();
            return (*Interrupt).Connected;
        }
    } else {
        let offset = CurrentHandler as usize - core::mem::offset_of!(_KINTERRUPT, DispatchCode);
        let ConnectedInterrupt = offset as PKINTERRUPT;

        if ((*Interrupt).ShareVector == 0)
            || ((*ConnectedInterrupt).ShareVector == 0)
            || ((*Interrupt).Mode != (*ConnectedInterrupt).Mode)
        {
            KiReleaseDispatcherLock(OldIrql);
            KeRevertToUserAffinityThread();
            return (*Interrupt).Connected;
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*ConnectedInterrupt).InterruptListEntry) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry) as PLIST_ENTRY,
        );
    }

    (*Interrupt).Connected = 1u8;

    KiReleaseDispatcherLock(OldIrql);
    KeRevertToUserAffinityThread();
    (*Interrupt).Connected
}

#[no_mangle]
pub unsafe extern "system" fn KeDisconnectInterrupt(Interrupt: PKINTERRUPT) -> BOOLEAN {
    let Interrupt = opaque_ptr(Interrupt);
    KeSetSystemAffinityThread(1u64 << (*Interrupt).Number);
    let OldIrql = KiAcquireDispatcherLock();

    if (*Interrupt).Connected != 0 {
        let VectorHandler = KeQueryInterruptHandler((*Interrupt).Vector);

        let offset = VectorHandler as usize - core::mem::offset_of!(_KINTERRUPT, DispatchCode);
        let VectorFirstInterrupt = offset as PKINTERRUPT;

        let HandlerHead = core::ptr::addr_of_mut!((*VectorFirstInterrupt).InterruptListEntry) as PLIST_ENTRY;

        if IsListEmpty(HandlerHead) {
            let unexpected_entry = (core::ptr::addr_of!(KiUnexpectedRange) as *const KI_INTERRUPT_DISPATCH_ENTRY)
                .add((*Interrupt).Vector as usize);
            let UnexpectedHandler = core::ptr::addr_of!((*unexpected_entry)._Op_push) as PVOID;

            HalDisableSystemInterrupt((*Interrupt).Vector, (*Interrupt).Irql);
            KeRegisterInterruptHandler((*Interrupt).Vector, UnexpectedHandler);
        } else if VectorFirstInterrupt == Interrupt {
            let NewHandlerHead = (*HandlerHead).Flink;
            RemoveTailList(NewHandlerHead);

            let next_offset = NewHandlerHead as usize - core::mem::offset_of!(_KINTERRUPT, InterruptListEntry);
            let NextInterrupt = next_offset as PKINTERRUPT;

            KeRegisterInterruptHandler(
                (*Interrupt).Vector,
                core::ptr::addr_of_mut!((*NextInterrupt).DispatchCode) as PVOID,
            );
        } else {
            RemoveEntryList(core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry) as PLIST_ENTRY);
        }

        (*Interrupt).Connected = 0u8;
    }

    KiReleaseDispatcherLock(OldIrql);
    KeRevertToUserAffinityThread();

    1u8
}

extern "system" {
    fn IopSynchronizePassiveInterrupt(
        Interrupt: PKINTERRUPT,
        Routine: PKSYNCHRONIZE_ROUTINE,
        Context: PVOID,
    ) -> ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn KeSynchronizeExecution(
    Interrupt: PKINTERRUPT,
    SynchronizeRoutine: PKSYNCHRONIZE_ROUTINE,
    SynchronizeContext: PVOID,
) -> BOOLEAN {
    let Interrupt = opaque_ptr(Interrupt);
    let SynchronizeContext = opaque_ptr(SynchronizeContext);
    // A passive-level connection (IoConnectInterruptEx, CONNECT_MESSAGE_BASED_PASSIVE)
    // synchronizes on its own mutex, at PASSIVE_LEVEL; iomgr irq.rs owns it.
    // u32::MAX: not passive; anything else is the routine's BOOLEAN.
    let Passive = IopSynchronizePassiveInterrupt(Interrupt, SynchronizeRoutine, SynchronizeContext);
    if Passive != u32::MAX {
        return Passive as BOOLEAN;
    }
    let OldIrql = KfRaiseIrql((*Interrupt).SynchronizeIrql);

    KeAcquireSpinLockAtDpcLevel((*Interrupt).ActualLock as *mut usize);

    let Success = (SynchronizeRoutine.unwrap())(SynchronizeContext);

    KeReleaseSpinLockFromDpcLevel((*Interrupt).ActualLock as *mut usize);

    KeLowerIrql(OldIrql);

    Success
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/amd64/interrupt.c
//  * PURPOSE:         Manages the Kernel's IRQ support for external drivers,
//  *                  for the purpopses of connecting, disconnecting and setting
//  *                  up ISRs for drivers. The backend behind the Io* Interrupt
//  *                  routines.
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern UCHAR KiInterruptDispatchTemplate[16];
// extern KI_INTERRUPT_DISPATCH_ENTRY KiUnexpectedRange[256];
// extern KI_INTERRUPT_DISPATCH_ENTRY KiUnexpectedRangeEnd[];
// void KiInterruptDispatch(void);
//
//
// /* FUNCTIONS ****************************************************************/
//
//
// VOID
// NTAPI
// KeInitializeInterrupt(
//     IN PKINTERRUPT Interrupt,
//     IN PKSERVICE_ROUTINE ServiceRoutine,
//     IN PVOID ServiceContext,
//     IN PKSPIN_LOCK SpinLock,
//     IN ULONG Vector,
//     IN KIRQL Irql,
//     IN KIRQL SynchronizeIrql,
//     IN KINTERRUPT_MODE InterruptMode,
//     IN BOOLEAN ShareVector,
//     IN CHAR ProcessorNumber,
//     IN BOOLEAN FloatingSave)
// {
//
//     /* Initialize the header */
//     Interrupt->Type = InterruptObject;
//     Interrupt->Size = sizeof(KINTERRUPT);
//
//     /* If no Spinlock is given, use the internal */
//     if (!SpinLock) SpinLock = &Interrupt->SpinLock;
//     KeInitializeSpinLock(&Interrupt->SpinLock);
//
//     /* Set the given parameters */
//     Interrupt->ServiceRoutine = ServiceRoutine;
//     Interrupt->ServiceContext = ServiceContext;
//     Interrupt->ActualLock = SpinLock;
//     Interrupt->Vector = Vector;
//     Interrupt->Irql = Irql;
//     Interrupt->SynchronizeIrql = SynchronizeIrql;
//     Interrupt->Mode = InterruptMode;
//     Interrupt->ShareVector = ShareVector;
//     Interrupt->Number = ProcessorNumber;
//     Interrupt->FloatingSave = FloatingSave;
//
//     /* Set initial values */
//     Interrupt->TickCount = 0;
//     Interrupt->Connected = FALSE;
//     Interrupt->ServiceCount = 0;
//     Interrupt->DispatchCount = 0;
//     Interrupt->TrapFrame = NULL;
//     Interrupt->Reserved = 0;
//
//     /* Copy the dispatch code (its location independent, no need to patch it) */
//     RtlCopyMemory(Interrupt->DispatchCode,
//                   KiInterruptDispatchTemplate,
//                   sizeof(Interrupt->DispatchCode));
//
//     Interrupt->DispatchAddress = 0;
// }
//
// BOOLEAN
// NTAPI
// KeConnectInterrupt(IN PKINTERRUPT Interrupt)
// {
//     PVOID CurrentHandler;
//     PKINTERRUPT ConnectedInterrupt;
//     KIRQL OldIrql;
//
//     /* Validate the vector */
//     if ((Interrupt->Vector < PRIMARY_VECTOR_BASE) ||
//         (Interrupt->Vector > MAXIMUM_IDTVECTOR))
//     {
//         DPRINT1("Invalid interrupt vector: %lu\n", Interrupt->Vector);
//         return FALSE;
//     }
//
//     ASSERT(Interrupt->Number < KeNumberProcessors);
//     ASSERT(Interrupt->Irql <= HIGH_LEVEL);
//     ASSERT(Interrupt->SynchronizeIrql >= Interrupt->Irql);
//     ASSERT(Interrupt->Irql == (Interrupt->Vector >> 4));
//
//     /* Check if its already connected */
//     if (Interrupt->Connected) return TRUE;
//
//     /* Set the system affinity and acquire the dispatcher lock */
//     KeSetSystemAffinityThread(1ULL << Interrupt->Number);
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Query the current handler */
//     CurrentHandler = KeQueryInterruptHandler(Interrupt->Vector);
//
//     /* Check if the vector is unused */
//     if ((CurrentHandler >= (PVOID)KiUnexpectedRange) &&
//         (CurrentHandler <= (PVOID)KiUnexpectedRangeEnd))
//     {
//         /* Initialize the list for chained interrupts */
//         InitializeListHead(&Interrupt->InterruptListEntry);
//
//         /* Set normal dispatch address */
//         Interrupt->DispatchAddress = KiInterruptDispatch;
//
//         /* Set the new handler */
//         KeRegisterInterruptHandler(Interrupt->Vector,
//                                    Interrupt->DispatchCode);
//
//         /* Enable the interrupt */
//         if (!HalEnableSystemInterrupt(Interrupt->Vector,
//                                       Interrupt->Irql,
//                                       Interrupt->Mode))
//         {
//             /* Didn't work, restore old handler */
//             DPRINT1("HalEnableSystemInterrupt failed\n");
//             KeRegisterInterruptHandler(Interrupt->Vector, CurrentHandler);
//             goto Cleanup;
//         }
//     }
//     else
//     {
//         /* Get the connected interrupt */
//         ConnectedInterrupt = CONTAINING_RECORD(CurrentHandler, KINTERRUPT, DispatchCode);
//
//         /* Check if sharing is ok */
//         if ((Interrupt->ShareVector == 0) ||
//             (ConnectedInterrupt->ShareVector == 0) ||
//             (Interrupt->Mode != ConnectedInterrupt->Mode))
//         {
//             goto Cleanup;
//         }
//
//         /* Insert the new interrupt into the connected interrupt's list */
//         InsertTailList(&ConnectedInterrupt->InterruptListEntry,
//                        &Interrupt->InterruptListEntry);
//     }
//
//     /* Mark as connected */
//     Interrupt->Connected = TRUE;
//
// Cleanup:
//     /* Release the dispatcher lock and restore the thread affinity */
//     KiReleaseDispatcherLock(OldIrql);
//     KeRevertToUserAffinityThread();
//     return Interrupt->Connected;
// }
//
// BOOLEAN
// NTAPI
// KeDisconnectInterrupt(IN PKINTERRUPT Interrupt)
// {
//     KIRQL OldIrql;
//     PVOID VectorHandler, UnexpectedHandler;
//     PKINTERRUPT VectorFirstInterrupt, NextInterrupt;
//     PLIST_ENTRY HandlerHead;
//
//     /* Set the system affinity and acquire the dispatcher lock */
//     KeSetSystemAffinityThread(1ULL << Interrupt->Number);
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if the interrupt was connected - otherwise there's nothing to do */
//     if (Interrupt->Connected)
//     {
//         /* Get the handler for this interrupt vector */
//         VectorHandler = KeQueryInterruptHandler(Interrupt->Vector);
//
//         /* Get the first interrupt for this handler */
//         VectorFirstInterrupt = CONTAINING_RECORD(VectorHandler, KINTERRUPT, DispatchCode);
//
//         /* The first interrupt list entry is the interrupt list head */
//         HandlerHead = &VectorFirstInterrupt->InterruptListEntry;
//
//         /* If the list is empty, this is the only interrupt for this vector */
//         if (IsListEmpty(HandlerHead))
//         {
//             /* If the list is empty, and the head is not from this interrupt,
//              * this interrupt is somehow incorrectly connected */
//             ASSERT(VectorFirstInterrupt == Interrupt);
//
//             UnexpectedHandler = &KiUnexpectedRange[Interrupt->Vector]._Op_push;
//
//             /* This is the only interrupt, the handler can be disconnected */
//             HalDisableSystemInterrupt(Interrupt->Vector, Interrupt->Irql);
//             KeRegisterInterruptHandler(Interrupt->Vector, UnexpectedHandler);
//         }
//         /* If the interrupt to be disconnected is the list head, but some others follow */
//         else if (VectorFirstInterrupt == Interrupt)
//         {
//             /* Relocate the head to the next element */
//             HandlerHead = HandlerHead->Flink;
//             RemoveTailList(HandlerHead);
//
//             /* Get the next interrupt from the list head */
//             NextInterrupt = CONTAINING_RECORD(HandlerHead,
//                                               KINTERRUPT,
//                                               InterruptListEntry);
//
//             /* Set the next interrupt as the handler for this vector */
//             KeRegisterInterruptHandler(Interrupt->Vector,
//                                        NextInterrupt->DispatchCode);
//         }
//         /* If the interrupt to be disconnected is not the list head */
//         else
//         {
//             /* Remove the to be disconnected interrupt from the interrupt list */
//             RemoveEntryList(&Interrupt->InterruptListEntry);
//         }
//
//         /* Mark as not connected */
//         Interrupt->Connected = FALSE;
//     }
//
//     /* Release the dispatcher lock and restore the thread affinity */
//     KiReleaseDispatcherLock(OldIrql);
//     KeRevertToUserAffinityThread();
//
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KeSynchronizeExecution(IN OUT PKINTERRUPT Interrupt,
//                        IN PKSYNCHRONIZE_ROUTINE SynchronizeRoutine,
//                        IN PVOID SynchronizeContext OPTIONAL)
// {
//     BOOLEAN Success;
//     KIRQL OldIrql;
//
//     /* Raise IRQL */
//     OldIrql = KfRaiseIrql(Interrupt->SynchronizeIrql);
//
//     /* Acquire interrupt spinlock */
//     KeAcquireSpinLockAtDpcLevel(Interrupt->ActualLock);
//
//     /* Call the routine */
//     Success = SynchronizeRoutine(SynchronizeContext);
//
//     /* Release lock */
//     KeReleaseSpinLockFromDpcLevel(Interrupt->ActualLock);
//
//     /* Lower IRQL */
//     KeLowerIrql(OldIrql);
//
//     /* Return status */
//     return Success;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
