use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    KAFFINITY, PKIPI_CONTEXT, PKIPI_WORKER, PKIPI_BROADCAST_WORKER,
    PKTRAP_FRAME, PKEXCEPTION_FRAME,
    HalRequestSoftwareInterrupt, KeGetCurrentProcessorNumber,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Use actual definitions, not extern declarations (lto=false splits them)
use crate::ke::processor::{KeNumberProcessors, KeActiveProcessors};
extern "C" {
    static mut KiReverseStallIpiLock: KSPIN_LOCK;
}

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiGenericCallTarget(
    PacketContext: PKIPI_CONTEXT,
    BroadcastFunction: PVOID,
    Argument: PVOID,
    Count: PVOID,
) {
    let BroadcastFunction = opaque_ptr(BroadcastFunction);
    let Argument = opaque_ptr(Argument);
    let Count = opaque_ptr(Count);
    DbgBreakPoint();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSend(
    TargetProcessors: KAFFINITY,
    IpiRequest: ULONG,
) {
    DbgBreakPoint();
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSend(
    TargetProcessors: KAFFINITY,
    IpiRequest: ULONG,
) {
    DbgBreakPoint();
}

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiSendPacket(
    TargetProcessors: KAFFINITY,
    WorkerFunction: PKIPI_WORKER,
    BroadcastFunction: PKIPI_BROADCAST_WORKER,
    Context: ULONG_PTR,
    Count: *mut ULONG,
) {
    let Count = opaque_ptr(Count);
    DbgBreakPoint();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSignalPacketDone(PacketContext: PKIPI_CONTEXT) {
    DbgBreakPoint();
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSignalPacketDone(PacketContext: PKIPI_CONTEXT) {
    DbgBreakPoint();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSignalPacketDoneAndStall(
    PacketContext: PKIPI_CONTEXT,
    ReverseStall: *mut ULONG,
) {
    let ReverseStall = opaque_ptr(ReverseStall);
    DbgBreakPoint();
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSignalPacketDoneAndStall(
    PacketContext: PKIPI_CONTEXT,
    ReverseStall: *mut ULONG,
) {
    let ReverseStall = opaque_ptr(ReverseStall);
    DbgBreakPoint();
}

#[cfg(not(target_arch = "x86_64"))]
#[cfg(any(target_arch = "x86", target_arch = "arm"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiServiceRoutine(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) -> BOOLEAN {
    #[cfg(feature = "smp")]
    {
        let Prcb = KeGetCurrentPrcb();

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_APC), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_APC) != 0
        {
            HalRequestSoftwareInterrupt(APC_LEVEL);
        }

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_DPC), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_DPC) != 0
        {
            (*Prcb).DpcInterruptRequested = 1u8;
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_SYNCH_REQUEST), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_SYNCH_REQUEST) != 0
        {
            #[cfg(any(target_arch = "arm", target_arch = "x86_64"))]
            {
                DbgBreakPoint();
            }
            #[cfg(not(any(target_arch = "arm", target_arch = "x86_64")))]
            {
                InterlockedDecrementUL(
                    core::ptr::addr_of_mut!((*(*Prcb).SignalDone).CurrentPacket[1])
                );
                if core::sync::atomic::AtomicU32::from_ptr(
                    core::ptr::addr_of_mut!((*(*Prcb).SignalDone).CurrentPacket[2])
                ).load(core::sync::atomic::Ordering::SeqCst) != 0
                {
                    while core::sync::atomic::AtomicU32::from_ptr(
                        core::ptr::addr_of_mut!((*(*Prcb).SignalDone).CurrentPacket[1])
                    ).load(core::sync::atomic::Ordering::SeqCst) != 0 {}
                }
                let worker_routine: unsafe extern "system" fn(PVOID) = 
                    core::mem::transmute((*(*Prcb).SignalDone).WorkerRoutine);
                worker_routine((*(*Prcb).SignalDone).CurrentPacket[0]);
                core::sync::atomic::AtomicUsize::from_ptr(
                    core::ptr::addr_of_mut!((*(*Prcb).SignalDone).TargetSet) as *mut usize
                ).fetch_and(!(1 << KeGetCurrentProcessorNumber()), core::sync::atomic::Ordering::SeqCst);
                if core::sync::atomic::AtomicU32::from_ptr(
                    core::ptr::addr_of_mut!((*(*Prcb).SignalDone).CurrentPacket[2])
                ).load(core::sync::atomic::Ordering::SeqCst) != 0
                {
                    while core::sync::atomic::AtomicUsize::from_ptr(
                        core::ptr::addr_of_mut!((*(*Prcb).SignalDone).TargetSet) as *mut usize
                    ).load(core::sync::atomic::Ordering::SeqCst) != 0 {}
                }
                InterlockedExchangePointer(
                    core::ptr::addr_of_mut!((*Prcb).SignalDone) as *mut PVOID,
                    core::ptr::null_mut()
                );
            }
        }
    }
    1u8
}

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KeIpiGenericCall(
    Function: PKIPI_BROADCAST_WORKER,
    Argument: ULONG_PTR,
) -> ULONG_PTR {
    let mut OldIrql: KIRQL = 0;
    let mut OldIrql2: KIRQL = 0;
    #[cfg(feature = "smp")]
    let mut Affinity: KAFFINITY;
    #[cfg(feature = "smp")]
    let mut Count: ULONG;
    #[cfg(feature = "smp")]
    let Prcb: PKPRCB;

    OldIrql = KeGetCurrentIrql();
    if OldIrql < DISPATCH_LEVEL {
        KeRaiseIrql(DISPATCH_LEVEL, core::ptr::addr_of_mut!(OldIrql));
    }

    #[cfg(feature = "smp")]
    {
        Count = KeNumberProcessors;
        Affinity = KeActiveProcessors;
        Prcb = KeGetCurrentPrcb();
        Affinity &= !(*Prcb).SetMember;
    }

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KiReverseStallIpiLock));

    #[cfg(feature = "smp")]
    {
        if Affinity != 0 {
            KiIpiSendPacket(
                Affinity,
                Some(core::mem::transmute(KiIpiGenericCallTarget as *const ())),
                Function,
                Argument,
                core::ptr::addr_of_mut!(Count),
            );

            while Count != 1 {
                core::hint::spin_loop();
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    KeRaiseIrql(IPI_LEVEL, core::ptr::addr_of_mut!(OldIrql2) as *mut KIRQL);

    #[cfg(feature = "smp")]
    {
        Count = 0;
    }

    let Status = (Function.unwrap())(Argument as _);

    #[cfg(feature = "smp")]
    {
        if Affinity != 0 {
            DbgBreakPoint();
        }
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(KiReverseStallIpiLock));

    KeLowerIrql(OldIrql);
    Status as ULONG_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/ipi.c
//  * PURPOSE:         Inter-Processor Packet Interface
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern KSPIN_LOCK KiReverseStallIpiLock;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #ifndef _M_AMD64
//
// VOID
// NTAPI
// KiIpiGenericCallTarget(IN PKIPI_CONTEXT PacketContext,
//                        IN PVOID BroadcastFunction,
//                        IN PVOID Argument,
//                        IN PVOID Count)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSend(IN KAFFINITY TargetProcessors,
//           IN ULONG IpiRequest)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// NTAPI
// KiIpiSendPacket(IN KAFFINITY TargetProcessors,
//                 IN PKIPI_WORKER WorkerFunction,
//                 IN PKIPI_BROADCAST_WORKER BroadcastFunction,
//                 IN ULONG_PTR Context,
//                 IN PULONG Count)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSignalPacketDone(IN PKIPI_CONTEXT PacketContext)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSignalPacketDoneAndStall(IN PKIPI_CONTEXT PacketContext,
//                               IN volatile PULONG ReverseStall)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// #if 0
// VOID
// NTAPI
// KiIpiSendRequest(IN KAFFINITY TargetSet,
//                  IN ULONG IpiRequest)
// {
// #ifdef CONFIG_SMP
//     LONG i;
//     PKPRCB Prcb;
//     KAFFINITY Current;
//
//     for (i = 0, Current = 1; i < KeNumberProcessors; i++, Current <<= 1)
//     {
//         if (TargetSet & Current)
//         {
//             /* Get the PRCB for this CPU */
//             Prcb = KiProcessorBlock[i];
//
//             InterlockedBitTestAndSet((PLONG)&Prcb->IpiFrozen, IpiRequest);
//             HalRequestIpi(i);
//         }
//     }
// #endif
// }
//
// VOID
// NTAPI
// KiIpiSendPacket(IN KAFFINITY TargetSet,
//                 IN PKIPI_BROADCAST_WORKER WorkerRoutine,
//                 IN ULONG_PTR Argument,
//                 IN ULONG Count,
//                 IN BOOLEAN Synchronize)
// {
// #ifdef CONFIG_SMP
//     KAFFINITY Processor;
//     LONG i;
//     PKPRCB Prcb, CurrentPrcb;
//     KIRQL oldIrql;
//
//     ASSERT(KeGetCurrentIrql() == SYNCH_LEVEL);
//
//     CurrentPrcb = KeGetCurrentPrcb();
//     (void)InterlockedExchangeUL(&CurrentPrcb->TargetSet, TargetSet);
//     (void)InterlockedExchangeUL(&CurrentPrcb->WorkerRoutine, (ULONG_PTR)WorkerRoutine);
//     (void)InterlockedExchangePointer(&CurrentPrcb->CurrentPacket[0], Argument);
//     (void)InterlockedExchangeUL(&CurrentPrcb->CurrentPacket[1], Count);
//     (void)InterlockedExchangeUL(&CurrentPrcb->CurrentPacket[2], Synchronize ? 1 : 0);
//
//     for (i = 0, Processor = 1; i < KeNumberProcessors; i++, Processor <<= 1)
//     {
//         if (TargetSet & Processor)
//         {
//             Prcb = KiProcessorBlock[i];
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone, (LONG)CurrentPrcb, 0));
//             InterlockedBitTestAndSet((PLONG)&Prcb->IpiFrozen, IPI_SYNCH_REQUEST);
//             if (Processor != CurrentPrcb->SetMember)
//             {
//                 HalRequestIpi(i);
//             }
//         }
//     }
//     if (TargetSet & CurrentPrcb->SetMember)
//     {
//         KeRaiseIrql(IPI_LEVEL, &oldIrql);
//         KiIpiServiceRoutine(NULL, NULL);
//         KeLowerIrql(oldIrql);
//     }
// #endif
// }
// #endif
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KiIpiServiceRoutine(IN PKTRAP_FRAME TrapFrame,
//                     IN PKEXCEPTION_FRAME ExceptionFrame)
// {
// #ifdef CONFIG_SMP
//     PKPRCB Prcb;
//     ASSERT(KeGetCurrentIrql() == IPI_LEVEL);
//
//     Prcb = KeGetCurrentPrcb();
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_APC))
//     {
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_DPC))
//     {
//         Prcb->DpcInterruptRequested = TRUE;
//         HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//     }
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_SYNCH_REQUEST))
//     {
// #if defined(_M_ARM) || defined(_M_AMD64)
//         DbgBreakPoint();
// #else
//         (void)InterlockedDecrementUL(&Prcb->SignalDone->CurrentPacket[1]);
//         if (InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[2], 0, 0))
//         {
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[1], 0, 0));
//         }
//         ((VOID (NTAPI*)(PVOID))(Prcb->SignalDone->WorkerRoutine))(Prcb->SignalDone->CurrentPacket[0]);
//         InterlockedBitTestAndReset((PLONG)&Prcb->SignalDone->TargetSet, KeGetCurrentProcessorNumber());
//         if (InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[2], 0, 0))
//         {
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone->TargetSet, 0, 0));
//         }
//         (void)InterlockedExchangePointer((PVOID*)&Prcb->SignalDone, NULL);
// #endif // _M_ARM
//     }
// #endif
//    return TRUE;
// }
//
// /*
//  * @implemented
//  */
// ULONG_PTR
// NTAPI
// KeIpiGenericCall(IN PKIPI_BROADCAST_WORKER Function,
//                  IN ULONG_PTR Argument)
// {
//     ULONG_PTR Status;
//     KIRQL OldIrql, OldIrql2;
// #ifdef CONFIG_SMP
//     KAFFINITY Affinity;
//     ULONG Count;
//     PKPRCB Prcb = KeGetCurrentPrcb();
// #endif
//
//     /* Raise to DPC level if required */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < DISPATCH_LEVEL) KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
// #ifdef CONFIG_SMP
//     /* Get current processor count and affinity */
//     Count = KeNumberProcessors;
//     Affinity = KeActiveProcessors;
//
//     /* Exclude ourselves */
//     Affinity &= ~Prcb->SetMember;
// #endif
//
//     /* Acquire the IPI lock */
//     KeAcquireSpinLockAtDpcLevel(&KiReverseStallIpiLock);
//
// #ifdef CONFIG_SMP
//     /* Make sure this is MP */
//     if (Affinity)
//     {
//         /* Send an IPI */
//         KiIpiSendPacket(Affinity,
//                         KiIpiGenericCallTarget,
//                         Function,
//                         Argument,
//                         &Count);
//
//         /* Spin until the other processors are ready */
//         while (Count != 1)
//         {
//             YieldProcessor();
//             KeMemoryBarrierWithoutFence();
//         }
//     }
// #endif
//
//     /* Raise to IPI level */
//     KeRaiseIrql(IPI_LEVEL, &OldIrql2);
//
// #ifdef CONFIG_SMP
//     /* Let the other processors know it is time */
//     Count = 0;
// #endif
//
//     /* Call the function */
//     Status = Function(Argument);
//
// #ifdef CONFIG_SMP
//     /* If this is MP, wait for the other processors to finish */
//     if (Affinity)
//     {
//         /* Sanity check */
//         ASSERT(Prcb == KeGetCurrentPrcb());
//
//         /* FIXME: TODO */
//         ASSERTMSG("Not yet implemented\n", FALSE);
//     }
// #endif
//
//     /* Release the lock */
//     KeReleaseSpinLockFromDpcLevel(&KiReverseStallIpiLock);
//
//     /* Lower IRQL back */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// #endif // !_M_AMD64
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
