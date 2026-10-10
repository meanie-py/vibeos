use crate::ke::ke_defs::*;
use crate::CONTAINING_RECORD;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::macros::*;

use ntkrnl_sys::ddk::{
    CSHORT, DISPATCH_INFO, KINTERRUPT, KINTERRUPT_MODE, KIRQL, PCHAR, PDISPATCH_INFO, PKINTERRUPT,
    PKINTERRUPT_ROUTINE, PKSERVICE_ROUTINE, PKSYNCHRONIZE_ROUTINE, PKTRAP_FRAME, PVOID, UCHAR,
    ULONG, USHORT, _KINTERRUPT,
    _CONNECT_TYPE_ChainConnect as ChainConnect,
    _CONNECT_TYPE_NoConnect as NoConnect,
    _CONNECT_TYPE_NormalConnect as NormalConnect,
    _CONNECT_TYPE_UnknownConnect as UnknownConnect,
    _KINTERRUPT_MODE_LevelSensitive as LevelSensitive,
    _KINTERRUPT_MODE_Latched as Latched,
    HalBeginSystemInterrupt, HalDisableSystemInterrupt, HalEnableSystemInterrupt,
    HalEndSystemInterrupt, HalSystemVectorDispatchEntry, KeBugCheck, KeInitializeSpinLock,
    KeRevertToUserAffinityThread, KeSetSystemAffinityThread, KiAcquireDispatcherLock,
    KiReleaseDispatcherLock,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::ke::processor::KeNumberProcessors;
extern "system" {
    fn HalVectorToIDTEntry(Vector: ULONG) -> UCHAR;
}
extern "C" {
    fn KxAcquireSpinLock(SpinLock: PKSPIN_LOCK);
    fn KxReleaseSpinLock(SpinLock: PKSPIN_LOCK);
    fn KiEnterInterruptTrap(TrapFrame: PKTRAP_FRAME);
    fn KiInterruptTemplate();
    static KiInterruptTemplateObject: PVOID;
    static KiUnexpectedEntrySize: ULONG;
}
extern "system" {
    fn KiStartUnexpectedRange();
}
extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}
extern "system" {
    fn KeQueryInterruptHandler(Vector: ULONG) -> PVOID;
    fn KeRegisterInterruptHandler(Vector: ULONG, Handler: PVOID);
}
extern "fastcall" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
}

static mut KiISRTimeout: ULONG = 55;
static mut KiISROverflow: USHORT = 30000;

const PRIMARY_VECTOR_BASE: ULONG = 0x30;
const DISPATCH_LENGTH: usize = 106;
const MAXULONG: ULONG = 0xFFFFFFFF;

#[cfg(not(feature = "smp"))]
const SYNCH_LEVEL: KIRQL = 2; // UP kernel: SYNCH_LEVEL == DISPATCH_LEVEL
#[cfg(feature = "smp")]
const SYNCH_LEVEL: KIRQL = 27; // IPI_LEVEL - 2 (WS03+ NDK) — keep equal to ke_defs::SYNCH_LEVEL

unsafe fn KiGetVectorDispatch(Vector: ULONG, Dispatch: PDISPATCH_INFO) {
    let mut Handler: PKINTERRUPT_ROUTINE;
    let Current: PVOID;
    let Type: UCHAR;
    let Entry: UCHAR;

    Type = HalSystemVectorDispatchEntry(
        Vector,
        &mut (*Dispatch).FlatDispatch,
        &mut (*Dispatch).NoDispatch,
    );

    Entry = HalVectorToIDTEntry(Vector);

    (*Dispatch).NoDispatch = core::mem::transmute::<ULONG_PTR, PKINTERRUPT_ROUTINE>(
        (KiStartUnexpectedRange as ULONG_PTR)
        + ((Entry as ULONG - PRIMARY_VECTOR_BASE) * KiUnexpectedEntrySize) as ULONG_PTR);

    (*Dispatch).InterruptDispatch = core::mem::transmute(KiInterruptDispatch as usize);
    (*Dispatch).FloatingDispatch = None;
    (*Dispatch).ChainedDispatch = core::mem::transmute(KiChainedDispatch as usize);
    (*Dispatch).FlatDispatch = core::ptr::null_mut();

    Current = KeQueryInterruptHandler(Vector);

    (*Dispatch).Interrupt = CONTAINING_RECORD!(
        Current,
        _KINTERRUPT,
        DispatchCode
    ) as PKINTERRUPT;

    if Current == core::mem::transmute::<PKINTERRUPT_ROUTINE, PVOID>((*Dispatch).NoDispatch) {
        (*Dispatch).Type = NoConnect;
    } else {
        Handler = (*(*Dispatch).Interrupt).DispatchAddress;
        if Handler == (*Dispatch).ChainedDispatch as PKINTERRUPT_ROUTINE {
            (*Dispatch).Type = ChainConnect;
        } else if Handler == (*Dispatch).InterruptDispatch as PKINTERRUPT_ROUTINE
            || Handler == (*Dispatch).FloatingDispatch as PKINTERRUPT_ROUTINE
        {
            (*Dispatch).Type = NormalConnect;
        } else {
            (*Dispatch).Type = UnknownConnect;
        }
    }
}

unsafe fn KiConnectVectorToInterrupt(Interrupt: PKINTERRUPT, Type: u32) {
    let mut Dispatch: DISPATCH_INFO = core::mem::zeroed();
    let Handler: PKINTERRUPT_ROUTINE;

    KiGetVectorDispatch((*Interrupt).Vector, &mut Dispatch);

    if Type == NoConnect {
        KeRegisterInterruptHandler((*Interrupt).Vector, core::mem::transmute(Dispatch.NoDispatch));
    } else {
        let handler_func = if Type == NormalConnect {
            Dispatch.InterruptDispatch
        } else {
            Dispatch.ChainedDispatch
        };

        (*Interrupt).DispatchAddress = handler_func as PKINTERRUPT_ROUTINE;

        KeRegisterInterruptHandler(
            (*Interrupt).Vector,
            core::ptr::addr_of_mut!((*Interrupt).DispatchCode) as PVOID,
        );
    }
}

unsafe fn KiExitInterrupt(TrapFrame: PKTRAP_FRAME, OldIrql: KIRQL, Spurious: BOOLEAN) -> ! {
    if Spurious == 0 {
        core::arch::asm!("cli");
        HalEndSystemInterrupt(OldIrql, TrapFrame);
    }

    KiEoiHelper(TrapFrame);
}

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "arm"))]
pub unsafe extern "C" fn KiUnexpectedInterrupt() -> ! {
    KeBugCheck(ntkrnl_sys::bugcodes::TRAP_CAUSE_UNKNOWN);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiUnexpectedInterruptTailHandler(TrapFrame: PKTRAP_FRAME) -> ! {
    let mut OldIrql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(HIGH_LEVEL, (*TrapFrame).ErrCode as ULONG, &mut OldIrql) != 0 {
        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiUnexpectedInterruptTailHandler(TrapFrame: PKTRAP_FRAME) -> ! {
    let mut OldIrql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(HIGH_LEVEL, (*TrapFrame).ErrCode as ULONG, &mut OldIrql) != 0 {
        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

type PKI_INTERRUPT_DISPATCH =
    unsafe extern "fastcall" fn(TrapFrame: PKTRAP_FRAME, Interrupt: PKINTERRUPT);

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiInterruptDispatch(
    TrapFrame: PKTRAP_FRAME,
    Interrupt: PKINTERRUPT,
) -> ! {
    let mut OldIrql: KIRQL = 0;

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(
        (*Interrupt).SynchronizeIrql,
        (*Interrupt).Vector,
        &mut OldIrql,
    ) != 0
    {
        KxAcquireSpinLock((*Interrupt).ActualLock as PKSPIN_LOCK);

        ((*Interrupt).ServiceRoutine.unwrap())(Interrupt, (*Interrupt).ServiceContext);

        KxReleaseSpinLock((*Interrupt).ActualLock as PKSPIN_LOCK);

        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiInterruptDispatch(TrapFrame: PKTRAP_FRAME, Interrupt: PKINTERRUPT) -> ! {
    let mut OldIrql: KIRQL = 0;

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(
        (*Interrupt).SynchronizeIrql,
        (*Interrupt).Vector,
        &mut OldIrql,
    ) != 0
    {
        KxAcquireSpinLock((*Interrupt).ActualLock as PKSPIN_LOCK);

        ((*Interrupt).ServiceRoutine.unwrap())(Interrupt, (*Interrupt).ServiceContext);

        KxReleaseSpinLock((*Interrupt).ActualLock as PKSPIN_LOCK);

        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiChainedDispatch(
    TrapFrame: PKTRAP_FRAME,
    Interrupt: PKINTERRUPT,
) -> ! {
    let mut OldIrql: KIRQL = 0;
    let mut OldInterruptIrql: KIRQL = 0;
    let mut Handled: BOOLEAN;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut CurrentInterrupt = Interrupt;

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(
        (*CurrentInterrupt).Irql,
        (*CurrentInterrupt).Vector,
        &mut OldIrql,
    ) != 0
    {
        ListHead = core::ptr::addr_of_mut!((*CurrentInterrupt).InterruptListEntry);
        NextEntry = ListHead;
        loop {
            if (*CurrentInterrupt).SynchronizeIrql > (*CurrentInterrupt).Irql {
                OldInterruptIrql = KfRaiseIrql((*CurrentInterrupt).SynchronizeIrql);
            }

            KxAcquireSpinLock((*CurrentInterrupt).ActualLock as PKSPIN_LOCK);

            Handled = ((*CurrentInterrupt).ServiceRoutine.unwrap())(
                CurrentInterrupt,
                (*CurrentInterrupt).ServiceContext,
            );

            KxReleaseSpinLock((*CurrentInterrupt).ActualLock as PKSPIN_LOCK);

            if (*CurrentInterrupt).SynchronizeIrql > (*CurrentInterrupt).Irql {
                KfLowerIrql(OldInterruptIrql);
            }

            if Handled != 0 && (*CurrentInterrupt).Mode == LevelSensitive {
                break;
            }

            NextEntry = (*NextEntry).Flink;

            if NextEntry == ListHead {
                if (*CurrentInterrupt).Mode == LevelSensitive {
                    break;
                }

                if Handled == 0 {
                    break;
                }
            }

            CurrentInterrupt = CONTAINING_RECORD!(
                NextEntry,
                _KINTERRUPT,
                InterruptListEntry
            ) as PKINTERRUPT;
        }

        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiChainedDispatch(TrapFrame: PKTRAP_FRAME, Interrupt: PKINTERRUPT) -> ! {
    let mut OldIrql: KIRQL = 0;
    let mut OldInterruptIrql: KIRQL = 0;
    let mut Handled: BOOLEAN;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut CurrentInterrupt = Interrupt;

    (*KeGetCurrentPrcb()).InterruptCount += 1;

    if HalBeginSystemInterrupt(
        (*CurrentInterrupt).Irql,
        (*CurrentInterrupt).Vector,
        &mut OldIrql,
    ) != 0
    {
        ListHead = core::ptr::addr_of_mut!((*CurrentInterrupt).InterruptListEntry);
        NextEntry = ListHead;
        loop {
            if (*CurrentInterrupt).SynchronizeIrql > (*CurrentInterrupt).Irql {
                OldInterruptIrql = KfRaiseIrql((*CurrentInterrupt).SynchronizeIrql);
            }

            KxAcquireSpinLock((*CurrentInterrupt).ActualLock as PKSPIN_LOCK);

            Handled = ((*CurrentInterrupt).ServiceRoutine.unwrap())(
                CurrentInterrupt,
                (*CurrentInterrupt).ServiceContext,
            );

            KxReleaseSpinLock((*CurrentInterrupt).ActualLock as PKSPIN_LOCK);

            if (*CurrentInterrupt).SynchronizeIrql > (*CurrentInterrupt).Irql {
                KfLowerIrql(OldInterruptIrql);
            }

            if Handled != 0 && (*CurrentInterrupt).Mode == LevelSensitive {
                break;
            }

            NextEntry = (*NextEntry).Flink;

            if NextEntry == ListHead {
                if (*CurrentInterrupt).Mode == LevelSensitive {
                    break;
                }

                if Handled == 0 {
                    break;
                }
            }

            CurrentInterrupt = CONTAINING_RECORD!(
                NextEntry,
                _KINTERRUPT,
                InterruptListEntry
            ) as PKINTERRUPT;
        }

        KiExitInterrupt(TrapFrame, OldIrql, 0);
    } else {
        KiExitInterrupt(TrapFrame, OldIrql, 1);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiInterruptTemplateHandler(
    TrapFrame: PKTRAP_FRAME,
    Interrupt: PKINTERRUPT,
) {
    let Interrupt = opaque_ptr(Interrupt);
    KiEnterInterruptTrap(TrapFrame);

    let dispatcher: PKI_INTERRUPT_DISPATCH =
        core::mem::transmute((*Interrupt).DispatchAddress);

    dispatcher(TrapFrame, Interrupt);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiInterruptTemplateHandler(TrapFrame: PKTRAP_FRAME, Interrupt: PKINTERRUPT) {
    KiEnterInterruptTrap(TrapFrame);

    let dispatcher: unsafe extern "C" fn(PKTRAP_FRAME, PKINTERRUPT) =
        core::mem::transmute((*Interrupt).DispatchAddress);
    dispatcher(TrapFrame, Interrupt);
}

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
    let mut DispatchCode =
        core::ptr::addr_of_mut!((*Interrupt).DispatchCode) as *mut ULONG;
    let mut Patch = DispatchCode;

    (*Interrupt).Type = 22;
    (*Interrupt).Size = core::mem::size_of::<_KINTERRUPT>() as CSHORT;

    if !SpinLock.is_null() {
        (*Interrupt).ActualLock = SpinLock as *mut _;
    } else {
        KeInitializeSpinLock(core::ptr::addr_of_mut!((*Interrupt).SpinLock));
        (*Interrupt).ActualLock = core::ptr::addr_of_mut!((*Interrupt).SpinLock);
    }

    (*Interrupt).ServiceRoutine = ServiceRoutine;
    (*Interrupt).ServiceContext = ServiceContext;
    (*Interrupt).Vector = Vector;
    (*Interrupt).Irql = Irql;
    (*Interrupt).SynchronizeIrql = SynchronizeIrql;
    (*Interrupt).Mode = InterruptMode;
    (*Interrupt).ShareVector = ShareVector;
    (*Interrupt).Number = ProcessorNumber;
    (*Interrupt).FloatingSave = FloatingSave;
    (*Interrupt).TickCount = MAXULONG;
    (*Interrupt).DispatchCount = MAXULONG;

    for i in 0..DISPATCH_LENGTH {
        *DispatchCode = *((KiInterruptTemplate as *const ULONG).add(i));
        DispatchCode = DispatchCode.add(1);
    }

    Patch = ((Patch as ULONG_PTR)
        + ((core::ptr::addr_of!(KiInterruptTemplateObject) as ULONG_PTR) - (KiInterruptTemplate as ULONG_PTR))
        - 4) as *mut ULONG;

    *Patch = Interrupt as ULONG;

    (*Interrupt).Connected = 0;
}

#[no_mangle]
pub unsafe extern "system" fn KeConnectInterrupt(Interrupt: PKINTERRUPT) -> BOOLEAN {
    let Interrupt = opaque_ptr(Interrupt);
    let mut Connected: BOOLEAN;
    let mut Error: BOOLEAN;
    let mut Status: BOOLEAN;
    let Irql: KIRQL;
    let mut OldIrql: KIRQL;
    let Number: UCHAR;
    let Vector: ULONG;
    let mut Dispatch: DISPATCH_INFO = core::mem::zeroed();

    Number = (*Interrupt).Number as UCHAR;
    Vector = (*Interrupt).Vector;
    Irql = (*Interrupt).Irql;

    if Irql > HIGH_LEVEL
        || Number >= KeNumberProcessors as u8
        || (*Interrupt).SynchronizeIrql < Irql
        || (*Interrupt).FloatingSave != 0
    {
        return 0;
    }

    Connected = 0;
    Error = 0;

    KeSetSystemAffinityThread(1 << Number);
    OldIrql = KiAcquireDispatcherLock();

    if (*Interrupt).Connected == 0 {
        KiGetVectorDispatch(Vector, &mut Dispatch);

        if Dispatch.Type == NoConnect {
            (*Interrupt).Connected = 1;
            Connected = 1;

            InitializeListHead(core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry));

            KiConnectVectorToInterrupt(Interrupt, NormalConnect);
            Status = HalEnableSystemInterrupt(Vector, Irql, (*Interrupt).Mode);
            if Status == 0 {
                Error = 1;
            }
        } else if Dispatch.Type != UnknownConnect
            && (*Interrupt).ShareVector != 0
            && (*Dispatch.Interrupt).ShareVector != 0
            && (*Dispatch.Interrupt).Mode == (*Interrupt).Mode
        {
            (*Interrupt).Connected = 1;
            Connected = 1;

            if Dispatch.Type != ChainConnect {
                KiConnectVectorToInterrupt(Dispatch.Interrupt, ChainConnect);
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*Dispatch.Interrupt).InterruptListEntry) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry) as PLIST_ENTRY,
            );
        }
    }

    KiReleaseDispatcherLock(OldIrql);
    KeRevertToUserAffinityThread();

    if Connected != 0 && Error != 0 {
        KeDisconnectInterrupt(Interrupt);
        Connected = 0;
    }

    Connected
}

#[no_mangle]
pub unsafe extern "system" fn KeDisconnectInterrupt(Interrupt: PKINTERRUPT) -> BOOLEAN {
    let Interrupt = opaque_ptr(Interrupt);
    let mut OldIrql: KIRQL;
    let Irql: KIRQL;
    let Vector: ULONG;
    let mut Dispatch: DISPATCH_INFO = core::mem::zeroed();
    let mut NextInterrupt: PKINTERRUPT;
    let State: BOOLEAN;

    KeSetSystemAffinityThread(1 << ((*Interrupt).Number as UCHAR));

    OldIrql = KiAcquireDispatcherLock();

    State = (*Interrupt).Connected;
    if State != 0 {
        Irql = (*Interrupt).Irql;
        Vector = (*Interrupt).Vector;

        KiGetVectorDispatch(Vector, &mut Dispatch);

        if Dispatch.Type == ChainConnect {
            if Interrupt == Dispatch.Interrupt {
                Dispatch.Interrupt = CONTAINING_RECORD!(
                    (*Dispatch.Interrupt).InterruptListEntry.Flink,
                    _KINTERRUPT,
                    InterruptListEntry
                ) as PKINTERRUPT;

                KiConnectVectorToInterrupt(Dispatch.Interrupt, ChainConnect);
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*Interrupt).InterruptListEntry)
                as PLIST_ENTRY);

            NextInterrupt = CONTAINING_RECORD!(
                (*Dispatch.Interrupt).InterruptListEntry.Flink,
                _KINTERRUPT,
                InterruptListEntry
            ) as PKINTERRUPT;

            if Dispatch.Interrupt == NextInterrupt {
                KiConnectVectorToInterrupt(Dispatch.Interrupt, NormalConnect);
            }
        } else {
            HalDisableSystemInterrupt((*Interrupt).Vector, Irql);
            KiConnectVectorToInterrupt(Interrupt, NoConnect);
        }

        (*Interrupt).Connected = 0;
    }

    KiReleaseDispatcherLock(OldIrql);
    KeRevertToUserAffinityThread();

    State
}

extern "system" {
    fn IopSynchronizePassiveInterrupt(
        Interrupt: PKINTERRUPT,
        Routine: PKSYNCHRONIZE_ROUTINE,
        Context: PVOID,
        Result: *mut BOOLEAN,
    ) -> BOOLEAN;
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
    let mut Passive: BOOLEAN = 0;
    if IopSynchronizePassiveInterrupt(Interrupt, SynchronizeRoutine, SynchronizeContext, &mut Passive) != 0 {
        return Passive;
    }
    let Success: BOOLEAN;
    let mut OldIrql: KIRQL = 0;

    KeRaiseIrql((*Interrupt).SynchronizeIrql, &mut OldIrql);

    KeAcquireSpinLockAtDpcLevel((*Interrupt).ActualLock as PKSPIN_LOCK);

    Success = (SynchronizeRoutine.unwrap())(SynchronizeContext);

    KeReleaseSpinLockFromDpcLevel((*Interrupt).ActualLock as PKSPIN_LOCK);

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
//  * FILE:            ntoskrnl/ke/i386/irqobj.c
//  * PURPOSE:         Manages the Kernel's IRQ support for external drivers,
//  *                  for the purposes of connecting, disconnecting and setting
//  *                  up ISRs for drivers. The backend behind the Io* Interrupt
//  *                  routines.
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
// ULONG KiISRTimeout = 55;
// USHORT KiISROverflow = 30000;
// extern ULONG NTAPI KiChainedDispatch2ndLvl(VOID);
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KiGetVectorDispatch(IN ULONG Vector,
//                     IN PDISPATCH_INFO Dispatch)
// {
//     PKINTERRUPT_ROUTINE Handler;
//     PVOID Current;
//     UCHAR Type;
//     UCHAR Entry;
//
//     /* Check if this is a primary or 2nd-level dispatch */
//     Type = HalSystemVectorDispatchEntry(Vector,
//                                         &Dispatch->FlatDispatch,
//                                         &Dispatch->NoDispatch);
//     ASSERT(Type == 0);
//
//     /* Get the IDT entry for this vector */
//     Entry = HalVectorToIDTEntry(Vector);
//
//     /* Setup the unhandled dispatch */
//     Dispatch->NoDispatch = (PVOID)(((ULONG_PTR)&KiStartUnexpectedRange) +
//                                    (Entry - PRIMARY_VECTOR_BASE) *
//                                    KiUnexpectedEntrySize);
//
//     /* Setup the handlers */
//     Dispatch->InterruptDispatch = (PVOID)KiInterruptDispatch;
//     Dispatch->FloatingDispatch = NULL; // Floating Interrupts are not supported
//     Dispatch->ChainedDispatch = (PVOID)KiChainedDispatch;
//     Dispatch->FlatDispatch = NULL;
//
//     /* Get the current handler */
//     Current = KeQueryInterruptHandler(Vector);
//
//     /* Set the interrupt */
//     Dispatch->Interrupt = CONTAINING_RECORD(Current,
//                                             KINTERRUPT,
//                                             DispatchCode);
//
//     /* Check what this interrupt is connected to */
//     if ((PKINTERRUPT_ROUTINE)Current == Dispatch->NoDispatch)
//     {
//         /* Not connected */
//         Dispatch->Type = NoConnect;
//     }
//     else
//     {
//         /* Get the handler */
//         Handler = Dispatch->Interrupt->DispatchAddress;
//         if (Handler == Dispatch->ChainedDispatch)
//         {
//             /* It's a chained interrupt */
//             Dispatch->Type = ChainConnect;
//         }
//         else if ((Handler == Dispatch->InterruptDispatch) ||
//                  (Handler == Dispatch->FloatingDispatch))
//         {
//             /* It's unchained */
//             Dispatch->Type = NormalConnect;
//         }
//         else
//         {
//             /* Unknown */
//             Dispatch->Type = UnknownConnect;
//         }
//     }
// }
//
// VOID
// NTAPI
// KiConnectVectorToInterrupt(IN PKINTERRUPT Interrupt,
//                            IN CONNECT_TYPE Type)
// {
//     DISPATCH_INFO Dispatch;
//     PKINTERRUPT_ROUTINE Handler;
//
//     /* Get vector data */
//     KiGetVectorDispatch(Interrupt->Vector, &Dispatch);
//
//     /* Check if we're only disconnecting */
//     if (Type == NoConnect)
//     {
//         /* Set the handler to NoDispatch */
//         Handler = Dispatch.NoDispatch;
//     }
//     else
//     {
//         /* Get the right handler */
//         Handler = (Type == NormalConnect) ?
//                   Dispatch.InterruptDispatch:
//                   Dispatch.ChainedDispatch;
//         ASSERT(Interrupt->FloatingSave == FALSE);
//
//         /* Set the handler */
//         Interrupt->DispatchAddress = Handler;
//
//         /* Read note in trap.s -- patching not needed since JMP is static */
//
//         /* Now set the final handler address */
//         ASSERT(Dispatch.FlatDispatch == NULL);
//         Handler = (PVOID)&Interrupt->DispatchCode;
//     }
//
//     /* Register the interrupt */
//     KeRegisterInterruptHandler(Interrupt->Vector, Handler);
// }
//
// FORCEINLINE
// DECLSPEC_NORETURN
// VOID
// KiExitInterrupt(IN PKTRAP_FRAME TrapFrame,
//                 IN KIRQL OldIrql,
//                 IN BOOLEAN Spurious)
// {
//     /* Check if this was a real interrupt */
//     if (!Spurious)
//     {
//         /* It was, disable interrupts and restore the IRQL */
//         _disable();
//         HalEndSystemInterrupt(OldIrql, TrapFrame);
//     }
//
//     /* Now exit the trap */
//     KiEoiHelper(TrapFrame);
// }
//
// DECLSPEC_NORETURN
// VOID
// __cdecl
// KiUnexpectedInterrupt(VOID)
// {
//     /* Crash the machine */
//     KeBugCheck(TRAP_CAUSE_UNKNOWN);
// }
//
// VOID
// FASTCALL
// KiUnexpectedInterruptTailHandler(IN PKTRAP_FRAME TrapFrame)
// {
//     KIRQL OldIrql;
//
//     /* Enter trap */
//     KiEnterInterruptTrap(TrapFrame);
//
//     /* Increase interrupt count */
//     KeGetCurrentPrcb()->InterruptCount++;
//
//     /* Start the interrupt */
//     if (HalBeginSystemInterrupt(HIGH_LEVEL, TrapFrame->ErrCode, &OldIrql))
//     {
//         /* Warn user */
//         DPRINT1("\n\x7\x7!!! Unexpected Interrupt 0x%02lx !!!\n", TrapFrame->ErrCode);
//
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, FALSE);
//     }
//     else
//     {
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, TRUE);
//     }
// }
//
// typedef
// VOID
// (FASTCALL *PKI_INTERRUPT_DISPATCH)(
//     IN PKTRAP_FRAME TrapFrame,
//     IN PKINTERRUPT Interrupt
// );
//
// VOID
// FASTCALL
// KiInterruptDispatch(IN PKTRAP_FRAME TrapFrame,
//                     IN PKINTERRUPT Interrupt)
// {
//     KIRQL OldIrql;
//
//     /* Increase interrupt count */
//     KeGetCurrentPrcb()->InterruptCount++;
//
//     /* Begin the interrupt, making sure it's not spurious */
//     if (HalBeginSystemInterrupt(Interrupt->SynchronizeIrql,
//                                 Interrupt->Vector,
//                                 &OldIrql))
//     {
//         /* Acquire interrupt lock */
//         KxAcquireSpinLock(Interrupt->ActualLock);
//
//         /* Call the ISR */
//         Interrupt->ServiceRoutine(Interrupt, Interrupt->ServiceContext);
//
//         /* Release interrupt lock */
//         KxReleaseSpinLock(Interrupt->ActualLock);
//
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, FALSE);
//     }
//     else
//     {
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, TRUE);
//     }
// }
//
// VOID
// FASTCALL
// KiChainedDispatch(IN PKTRAP_FRAME TrapFrame,
//                   IN PKINTERRUPT Interrupt)
// {
//     KIRQL OldIrql, OldInterruptIrql = 0;
//     BOOLEAN Handled;
//     PLIST_ENTRY NextEntry, ListHead;
//
//     /* Increase interrupt count */
//     KeGetCurrentPrcb()->InterruptCount++;
//
//     /* Begin the interrupt, making sure it's not spurious */
//     if (HalBeginSystemInterrupt(Interrupt->Irql,
//                                 Interrupt->Vector,
//                                 &OldIrql))
//     {
//         /* Get list pointers */
//         ListHead = &Interrupt->InterruptListEntry;
//         NextEntry = ListHead; /* The head is an entry! */
//         while (TRUE)
//         {
//             /* Check if this interrupt's IRQL is higher than the current one */
//             if (Interrupt->SynchronizeIrql > Interrupt->Irql)
//             {
//                 /* Raise to higher IRQL */
//                 OldInterruptIrql = KfRaiseIrql(Interrupt->SynchronizeIrql);
//             }
//
//             /* Acquire interrupt lock */
//             KxAcquireSpinLock(Interrupt->ActualLock);
//
//             /* Call the ISR */
//             Handled = Interrupt->ServiceRoutine(Interrupt,
//                                                 Interrupt->ServiceContext);
//
//             /* Release interrupt lock */
//             KxReleaseSpinLock(Interrupt->ActualLock);
//
//             /* Check if this interrupt's IRQL is higher than the current one */
//             if (Interrupt->SynchronizeIrql > Interrupt->Irql)
//             {
//                 /* Lower the IRQL back */
//                 ASSERT(OldInterruptIrql == Interrupt->Irql);
//                 KfLowerIrql(OldInterruptIrql);
//             }
//
//             /* Check if the interrupt got handled and it's level */
//             if ((Handled) && (Interrupt->Mode == LevelSensitive)) break;
//
//             /* What's next? */
//             NextEntry = NextEntry->Flink;
//
//             /* Is this the last one? */
//             if (NextEntry == ListHead)
//             {
//                 /* Level should not have gotten here */
//                 if (Interrupt->Mode == LevelSensitive) break;
//
//                 /* As for edge, we can only exit once nobody can handle the interrupt */
//                 if (!Handled) break;
//             }
//
//             /* Get the interrupt object for the next pass */
//             Interrupt = CONTAINING_RECORD(NextEntry, KINTERRUPT, InterruptListEntry);
//         }
//
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, FALSE);
//     }
//     else
//     {
//         /* Now call the epilogue code */
//         KiExitInterrupt(TrapFrame, OldIrql, TRUE);
//     }
// }
//
// VOID
// FASTCALL
// KiInterruptTemplateHandler(IN PKTRAP_FRAME TrapFrame,
//                            IN PKINTERRUPT Interrupt)
// {
//     /* Enter interrupt frame */
//     KiEnterInterruptTrap(TrapFrame);
//
//     /* Call the correct dispatcher */
//     ((PKI_INTERRUPT_DISPATCH)Interrupt->DispatchAddress)(TrapFrame, Interrupt);
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeInterrupt(IN PKINTERRUPT Interrupt,
//                       IN PKSERVICE_ROUTINE ServiceRoutine,
//                       IN PVOID ServiceContext,
//                       IN PKSPIN_LOCK SpinLock,
//                       IN ULONG Vector,
//                       IN KIRQL Irql,
//                       IN KIRQL SynchronizeIrql,
//                       IN KINTERRUPT_MODE InterruptMode,
//                       IN BOOLEAN ShareVector,
//                       IN CHAR ProcessorNumber,
//                       IN BOOLEAN FloatingSave)
// {
//     ULONG i;
//     PULONG DispatchCode = &Interrupt->DispatchCode[0], Patch = DispatchCode;
//
//     /* Set the Interrupt Header */
//     Interrupt->Type = InterruptObject;
//     Interrupt->Size = sizeof(KINTERRUPT);
//
//     /* Check if we got a spinlock */
//     if (SpinLock)
//     {
//         Interrupt->ActualLock = SpinLock;
//     }
//     else
//     {
//         /* Use the built-in one */
//         KeInitializeSpinLock(&Interrupt->SpinLock);
//         Interrupt->ActualLock = &Interrupt->SpinLock;
//     }
//
//     /* Set the other settings */
//     Interrupt->ServiceRoutine = ServiceRoutine;
//     Interrupt->ServiceContext = ServiceContext;
//     Interrupt->Vector = Vector;
//     Interrupt->Irql = Irql;
//     Interrupt->SynchronizeIrql = SynchronizeIrql;
//     Interrupt->Mode = InterruptMode;
//     Interrupt->ShareVector = ShareVector;
//     Interrupt->Number = ProcessorNumber;
//     Interrupt->FloatingSave = FloatingSave;
//     Interrupt->TickCount = MAXULONG;
//     Interrupt->DispatchCount = MAXULONG;
//
//     /* Loop the template in memory */
//     for (i = 0; i < DISPATCH_LENGTH; i++)
//     {
//         /* Copy the dispatch code */
//         *DispatchCode++ = ((PULONG)KiInterruptTemplate)[i];
//     }
//
//     /* Jump to the last 4 bytes */
//     Patch = (PULONG)((ULONG_PTR)Patch +
//                      ((ULONG_PTR)&KiInterruptTemplateObject -
//                       (ULONG_PTR)KiInterruptTemplate) - 4);
//
//     /* Apply the patch */
//     *Patch = PtrToUlong(Interrupt);
//
//     /* Disconnect it at first */
//     Interrupt->Connected = FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeConnectInterrupt(IN PKINTERRUPT Interrupt)
// {
//     BOOLEAN Connected, Error, Status;
//     KIRQL Irql, OldIrql;
//     UCHAR Number;
//     ULONG Vector;
//     DISPATCH_INFO Dispatch;
//
//     /* Get data from interrupt */
//     Number = Interrupt->Number;
//     Vector = Interrupt->Vector;
//     Irql = Interrupt->Irql;
//
//     /* Validate the settings */
//     if ((Irql > HIGH_LEVEL) ||
//         (Number >= KeNumberProcessors) ||
//         (Interrupt->SynchronizeIrql < Irql) ||
//         (Interrupt->FloatingSave))
//     {
//         return FALSE;
//     }
//
//     /* Set defaults */
//     Connected = FALSE;
//     Error = FALSE;
//
//     /* Set the system affinity and acquire the dispatcher lock */
//     KeSetSystemAffinityThread(1 << Number);
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if it's already been connected */
//     if (!Interrupt->Connected)
//     {
//         /* Get vector dispatching information */
//         KiGetVectorDispatch(Vector, &Dispatch);
//
//         /* Check if the vector is already connected */
//         if (Dispatch.Type == NoConnect)
//         {
//             /* Do the connection */
//             Interrupt->Connected = Connected = TRUE;
//
//             /* Initialize the list */
//             InitializeListHead(&Interrupt->InterruptListEntry);
//
//             /* Connect and enable the interrupt */
//             KiConnectVectorToInterrupt(Interrupt, NormalConnect);
//             Status = HalEnableSystemInterrupt(Vector, Irql, Interrupt->Mode);
//             if (!Status) Error = TRUE;
//         }
//         else if ((Dispatch.Type != UnknownConnect) &&
//                 (Interrupt->ShareVector) &&
//                 (Dispatch.Interrupt->ShareVector) &&
//                 (Dispatch.Interrupt->Mode == Interrupt->Mode))
//         {
//             /* The vector is shared and the interrupts are compatible */
//             Interrupt->Connected = Connected = TRUE;
//
//             /*
//              * Verify the IRQL for chained connect,
//              */
// #if defined(CONFIG_SMP)
//             ASSERT(Irql <= SYNCH_LEVEL);
// #else
//             ASSERT(Irql <= (IPI_LEVEL - 2));
// #endif
//
//             /* Check if this is the first chain */
//             if (Dispatch.Type != ChainConnect)
//             {
//                 /* This is not supported */
//                 ASSERT(Dispatch.Interrupt->Mode != Latched);
//
//                 /* Setup the chainned handler */
//                 KiConnectVectorToInterrupt(Dispatch.Interrupt, ChainConnect);
//             }
//
//             /* Insert into the interrupt list */
//             InsertTailList(&Dispatch.Interrupt->InterruptListEntry,
//                            &Interrupt->InterruptListEntry);
//         }
//     }
//
//     /* Unlock the dispatcher and revert affinity */
//     KiReleaseDispatcherLock(OldIrql);
//     KeRevertToUserAffinityThread();
//
//     /* Check if we failed while trying to connect */
//     if ((Connected) && (Error))
//     {
//         DPRINT1("HalEnableSystemInterrupt failed\n");
//         KeDisconnectInterrupt(Interrupt);
//         Connected = FALSE;
//     }
//
//     /* Return to caller */
//     return Connected;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeDisconnectInterrupt(IN PKINTERRUPT Interrupt)
// {
//     KIRQL OldIrql, Irql;
//     ULONG Vector;
//     DISPATCH_INFO Dispatch;
//     PKINTERRUPT NextInterrupt;
//     BOOLEAN State;
//
//     /* Set the affinity */
//     KeSetSystemAffinityThread(1 << Interrupt->Number);
//
//     /* Lock the dispatcher */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if it's actually connected */
//     State = Interrupt->Connected;
//     if (State)
//     {
//         /* Get the vector and IRQL */
//         Irql = Interrupt->Irql;
//         Vector = Interrupt->Vector;
//
//         /* Get vector dispatch data */
//         KiGetVectorDispatch(Vector, &Dispatch);
//
//         /* Check if it was chained */
//         if (Dispatch.Type == ChainConnect)
//         {
//             /* Check if the top-level interrupt is being removed */
// #if defined(CONFIG_SMP)
//             ASSERT(Irql <= SYNCH_LEVEL);
// #else
//             ASSERT(Irql <= (IPI_LEVEL - 2));
// #endif
//             if (Interrupt == Dispatch.Interrupt)
//             {
//                 /* Get the next one */
//                 Dispatch.Interrupt = CONTAINING_RECORD(Dispatch.Interrupt->
//                                                        InterruptListEntry.Flink,
//                                                        KINTERRUPT,
//                                                        InterruptListEntry);
//
//                 /* Reconnect it */
//                 KiConnectVectorToInterrupt(Dispatch.Interrupt, ChainConnect);
//             }
//
//             /* Remove it */
//             RemoveEntryList(&Interrupt->InterruptListEntry);
//
//             /* Get the next one */
//             NextInterrupt = CONTAINING_RECORD(Dispatch.Interrupt->
//                                               InterruptListEntry.Flink,
//                                               KINTERRUPT,
//                                               InterruptListEntry);
//
//             /* Check if this is the only one left */
//             if (Dispatch.Interrupt == NextInterrupt)
//             {
//                 /* Connect it in non-chained mode */
//                 KiConnectVectorToInterrupt(Dispatch.Interrupt, NormalConnect);
//             }
//         }
//         else
//         {
//             /* Only one left, disable and remove it */
//             HalDisableSystemInterrupt(Interrupt->Vector, Irql);
//             KiConnectVectorToInterrupt(Interrupt, NoConnect);
//         }
//
//         /* Disconnect it */
//         Interrupt->Connected = FALSE;
//     }
//
//     /* Unlock the dispatcher and revert affinity */
//     KiReleaseDispatcherLock(OldIrql);
//     KeRevertToUserAffinityThread();
//
//     /* Return to caller */
//     return State;
// }
//
// /*
//  * @implemented
//  */
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
//     KeRaiseIrql(Interrupt->SynchronizeIrql,
//                 &OldIrql);
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
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
