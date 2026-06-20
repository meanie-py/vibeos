use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    _KTRAP_FRAME, _KTHREAD, _KUSER_SHARED_DATA, _KTIMER_TABLE_ENTRY,
    PKTRAP_FRAME, PKTHREAD, PKPRCB,
    KSYSTEM_TIME, ULARGE_INTEGER,
    HalRequestSoftwareInterrupt,
};

#[no_mangle]
pub static mut KeTimeAdjustment: ULONG = 0;
#[no_mangle]
pub static mut KiTimeAdjustmentEnabled: BOOLEAN = 0u8;

extern "C" {
    static SharedUserData: *mut u8;
    static KdDebuggerEnabled: BOOLEAN;
}
// Use actual definitions from sibling modules, not extern declarations (lto=false splits them)
// KiTimerTableListHead, KiAdjustDpcThreshold, KiMaximumDpcQueueDepth come via ke_defs re-exports
use super::clock::{KiTickOffset, KeTickCount, KeMaximumIncrement};
use super::dpc::{KiIdealDpcRate, KiDPCTimeout};
extern "system" {
    fn KdPollBreakIn() -> BOOLEAN;
    fn DbgBreakPointWithStatus(Status: ULONG);
}

const TIMER_TABLE_SIZE: usize = 512;
const CLOCK_QUANTUM_DECREMENT: i8 = 3;
const DBG_STATUS_CONTROL_C: ULONG = 1;
const EFLAGS_V86_MASK: ULONG = 0x00020000;

#[cfg(target_arch = "x86")]
extern "C" {
    fn KiEndInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME);
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
unsafe fn KiUserTrap(TrapFrame: PKTRAP_FRAME) -> bool {
    (*TrapFrame).SegCs & 1 != 0
}

#[inline(always)]
unsafe fn KiWriteSystemTime(SystemTime: *mut KSYSTEM_TIME, NewTime: ULARGE_INTEGER) {
    #[cfg(target_pointer_width = "64")]
    {
        *(SystemTime as *mut u64) = NewTime.QuadPart;
        (*SystemTime).High2Time = NewTime.__bindgen_anon_1.HighPart as i32;
    }
    #[cfg(target_pointer_width = "32")]
    {
        // Write order must be: High1Time, LowPart, High2Time (matching C code)
        // This allows readers to detect torn reads by comparing High1Time == High2Time
        (*SystemTime).High1Time = NewTime.__bindgen_anon_1.HighPart as i32;
        (*SystemTime).LowPart = NewTime.__bindgen_anon_1.LowPart;
        (*SystemTime).High2Time = NewTime.__bindgen_anon_1.HighPart as i32;
    }
}

#[inline(always)]
unsafe fn KiCheckForTimerExpiration(
    Prcb: PKPRCB,
    TrapFrame: PKTRAP_FRAME,
    InterruptTime: ULARGE_INTEGER,
) {
    let Hand = (*(core::ptr::addr_of!(KeTickCount) as *const KSYSTEM_TIME)).LowPart & ((TIMER_TABLE_SIZE - 1) as u32);

    if KiTimerTableListHead[Hand as usize].Time.QuadPart <= InterruptTime.QuadPart {
        if (*Prcb).TimerRequest == 0 {
            (*Prcb).TimerRequest = TrapFrame as usize as _;
            (*Prcb).TimerHand = Hand as _;
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeUpdateSystemTime(
    TrapFrame: PKTRAP_FRAME,
    Increment: ULONG,
    Irql: KIRQL,
) {
    KeUpdateSystemTimeImpl(TrapFrame, Increment, Irql);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeUpdateSystemTime(
    TrapFrame: PKTRAP_FRAME,
    Increment: ULONG,
    Irql: KIRQL,
) {
    KeUpdateSystemTimeImpl(TrapFrame, Increment, Irql);
}

unsafe fn KeUpdateSystemTimeImpl(
    TrapFrame: PKTRAP_FRAME,
    Increment: ULONG,
    Irql: KIRQL,
) {
    let Prcb = KeGetCurrentPrcb();
    let mut InterruptTime: ULARGE_INTEGER = core::mem::zeroed();
    let OldTickOffset: LONG;

    if (*Prcb).SkipTick != 0 {
        (*Prcb).SkipTick = 0u8;
        (*Prcb).InterruptCount += 1;

        #[cfg(target_arch = "x86")]
        KiEndInterrupt(Irql, TrapFrame);

        return;
    }

    let shared_interrupt_time = (SharedUserData as usize + core::mem::offset_of!(_KUSER_SHARED_DATA, InterruptTime)) as *mut u64;

    // [TIMER-BASE FIX] The timer table relies on the invariant
    // InterruptTime / KeMaximumIncrement == KeTickCount (both 0-based since boot):
    // insertion (KiComputeTimerTableIndex) hashes absolute InterruptTime by
    // KeMaximumIncrement, while expiration sweeps KeTickCount.LowPart & 511. On amd64
    // both KUSER InterruptTime and KeTickCount arrive at the first clock tick pre-seeded
    // to large, mutually-inconsistent values (offset ~106 buckets), so every relative
    // KeDelay/KeWait-with-timeout lands in a bucket the expiration hand just passed and
    // waits a full ~512-tick table revolution (~8s). (Masked previously by the broken
    // RTC clock; exposed when the LAPIC clock made timers actually fire — 22356522c58.)
    // Zero both on the first tick so they advance in lockstep, restoring the invariant.
    #[cfg(target_arch = "x86_64")]
    {
        static mut CLOCK_BASE_RESET: bool = false;
        if !CLOCK_BASE_RESET {
            CLOCK_BASE_RESET = true;
            *shared_interrupt_time = 0;
            let shared_tick = (SharedUserData as usize
                + core::mem::offset_of!(_KUSER_SHARED_DATA, __bindgen_anon_2)) as *mut KSYSTEM_TIME;
            (*shared_tick).LowPart = 0;
            (*shared_tick).High1Time = 0;
            (*shared_tick).High2Time = 0;
            KeTickCount.LowPart = 0;
            KeTickCount.High1Time = 0;
            KeTickCount.High2Time = 0;
        }
    }

    InterruptTime.QuadPart = *shared_interrupt_time;
    InterruptTime.QuadPart += Increment as u64;
    KiWriteSystemTime(
        (SharedUserData as usize + core::mem::offset_of!(_KUSER_SHARED_DATA, InterruptTime)) as *mut KSYSTEM_TIME,
        InterruptTime,
    );

    KiCheckForTimerExpiration(Prcb, TrapFrame, InterruptTime);

    OldTickOffset = InterlockedExchangeAdd(
        core::ptr::addr_of_mut!(KiTickOffset),
        -(Increment as i32),
    );

    if KdDebuggerEnabled != 0 && KdPollBreakIn() != 0 {
        DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
    }

    if OldTickOffset <= Increment as LONG {
        let mut CurrentTime: ULARGE_INTEGER = core::mem::zeroed();

        let shared_system_time = (SharedUserData as usize + core::mem::offset_of!(_KUSER_SHARED_DATA, SystemTime)) as *mut u64;
        CurrentTime.QuadPart = *shared_system_time;
        CurrentTime.QuadPart += KeTimeAdjustment as u64;
        KiWriteSystemTime(
            (SharedUserData as usize + core::mem::offset_of!(_KUSER_SHARED_DATA, SystemTime)) as *mut KSYSTEM_TIME,
            CurrentTime,
        );

        CurrentTime.QuadPart = *(core::ptr::addr_of!(KeTickCount) as *const u64) + 1;
        KiWriteSystemTime(core::ptr::addr_of_mut!(KeTickCount), CurrentTime);

        let shared_tick_count = (SharedUserData as usize + core::mem::offset_of!(_KUSER_SHARED_DATA, __bindgen_anon_2)) as *mut KSYSTEM_TIME;
        KiWriteSystemTime(shared_tick_count, CurrentTime);

        KiCheckForTimerExpiration(Prcb, TrapFrame, InterruptTime);

        KiTickOffset += KeMaximumIncrement as i32;

        KeUpdateRunTime(TrapFrame, Irql);
    } else {
        (*Prcb).InterruptCount += 1;
    }

    #[cfg(target_arch = "x86")]
    KiEndInterrupt(Irql, TrapFrame);
}

#[no_mangle]
pub unsafe extern "system" fn KeUpdateRunTime(
    TrapFrame: PKTRAP_FRAME,
    Irql: KIRQL,
) {
    let Thread = PsGetCurrentThread();
    let Prcb = KeGetCurrentPrcb();

    if (*Prcb).SkipTick != 0 {
        (*Prcb).SkipTick = 0u8;
        return;
    }

    (*Prcb).InterruptCount += 1;

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    let from_user_mode = KiUserTrap(TrapFrame) || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0;
    
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let from_user_mode = (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode == 1i8;

    if from_user_mode {
        (*Prcb).UserTime += 1;
        *kt_UserTime(Thread) += 1;
    } else {
        (*Prcb).KernelTime += 1;
        if Irql > DISPATCH_LEVEL {
            (*Prcb).InterruptTime += 1;
        } else if (Irql < DISPATCH_LEVEL) || ((*Prcb).DpcRoutineActive == 0) {
            *kt_KernelTime(Thread) += 1;
        } else {
            (*Prcb).DpcTime += 1;

            #[cfg(debug_assertions)]
            {
                (*Prcb).DebugDpcTime += 1;

                if (*Prcb).DebugDpcTime == KiDPCTimeout {
                    crate::dbg_print!(b"*** DPC routine > 1 sec --- This is not a break in KeUpdateSystemTime\n\0");

                    if KdDebuggerEnabled != 0 {
                        DbgBreakPoint();
                    }

                    (*Prcb).DebugDpcTime = 0;
                }
            }
        }
    }

    (*Prcb).DpcRequestRate = (((*Prcb).DpcData[0].DpcCount - (*Prcb).DpcLastCount) + (*Prcb).DpcRequestRate) >> 1;
    (*Prcb).DpcLastCount = (*Prcb).DpcData[0].DpcCount;

    if ((*Prcb).DpcData[0].DpcQueueDepth != 0) && ((*Prcb).DpcRoutineActive == 0) {
        (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;
        HalRequestSoftwareInterrupt(DISPATCH_LEVEL);

        if ((*Prcb).DpcRequestRate < KiIdealDpcRate) && ((*Prcb).MaximumDpcQueueDepth > 1) {
            (*Prcb).MaximumDpcQueueDepth -= 1;
        }
    } else {
        (*Prcb).AdjustDpcThreshold -= 1;
        if (*Prcb).AdjustDpcThreshold == 0 {
            (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;
            if KiMaximumDpcQueueDepth as i32 != (*Prcb).MaximumDpcQueueDepth as i32 {
                (*Prcb).MaximumDpcQueueDepth += 1;
            }
        }
    }

    *kt_Quantum(Thread) -= CLOCK_QUANTUM_DECREMENT;

    if (*kt_Quantum(Thread) <= 0) && (Thread != (*Prcb).IdleThread as PKTHREAD) {
        (*Prcb).QuantumEnd = 1u8;
        HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ke/time.c
//  * PURPOSE:         Implements timebase functionality
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// LONG KiTickOffset;
// ULONG KeTimeAdjustment;
// BOOLEAN KiTimeAdjustmentEnabled = FALSE;
//
// /* FUNCTIONS ******************************************************************/
//
// FORCEINLINE
// VOID
// KiWriteSystemTime(volatile KSYSTEM_TIME *SystemTime, ULARGE_INTEGER NewTime)
// {
// #ifdef _WIN64
//     /* Do a single atomic write */
//     *(ULONGLONG*)SystemTime = NewTime.QuadPart;
// #else
//     /* Update in 3 steps, so that a reader can recognize partial updates */
//     SystemTime->High1Time = NewTime.HighPart;
//     SystemTime->LowPart = NewTime.LowPart;
// #endif
//     SystemTime->High2Time = NewTime.HighPart;
// }
//
// FORCEINLINE
// VOID
// KiCheckForTimerExpiration(
//     PKPRCB Prcb,
//     PKTRAP_FRAME TrapFrame,
//     ULARGE_INTEGER InterruptTime)
// {
//     ULONG Hand;
//
//     /* Check for timer expiration */
//     Hand = KeTickCount.LowPart & (TIMER_TABLE_SIZE - 1);
//     if (KiTimerTableListHead[Hand].Time.QuadPart <= InterruptTime.QuadPart)
//     {
//         /* Check if we are already doing expiration */
//         if (!Prcb->TimerRequest)
//         {
//             /* Request a DPC to handle this */
//             Prcb->TimerRequest = (ULONG_PTR)TrapFrame;
//             Prcb->TimerHand = Hand;
//             HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//         }
//     }
// }
//
// VOID
// FASTCALL
// KeUpdateSystemTime(IN PKTRAP_FRAME TrapFrame,
//                    IN ULONG Increment,
//                    IN KIRQL Irql)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     ULARGE_INTEGER CurrentTime, InterruptTime;
//     LONG OldTickOffset;
//
//     /* Check if this tick is being skipped */
//     if (Prcb->SkipTick)
//     {
//         /* Handle it next time */
//         Prcb->SkipTick = FALSE;
//
//         /* Increase interrupt count and end the interrupt */
//         Prcb->InterruptCount++;
//
// #ifdef _M_IX86 // x86 optimization
//         KiEndInterrupt(Irql, TrapFrame);
// #endif
//
//         /* Note: non-x86 return back to the caller! */
//         return;
//     }
//
//     /* Add the increment time to the shared data */
//     InterruptTime.QuadPart = *(ULONGLONG*)&SharedUserData->InterruptTime;
//     InterruptTime.QuadPart += Increment;
//     KiWriteSystemTime(&SharedUserData->InterruptTime, InterruptTime);
//
//     /* Check for timer expiration */
//     KiCheckForTimerExpiration(Prcb, TrapFrame, InterruptTime);
//
//     /* Update the tick offset */
//     OldTickOffset = InterlockedExchangeAdd(&KiTickOffset, -(LONG)Increment);
//
//     /* If the debugger is enabled, check for break-in request */
//     if (KdDebuggerEnabled && KdPollBreakIn())
//     {
//         /* Break-in requested! */
//         DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
//     }
//
//     /* Check for full tick */
//     if (OldTickOffset <= (LONG)Increment)
//     {
//         /* Update the system time */
//         CurrentTime.QuadPart = *(ULONGLONG*)&SharedUserData->SystemTime;
//         CurrentTime.QuadPart += KeTimeAdjustment;
//         KiWriteSystemTime(&SharedUserData->SystemTime, CurrentTime);
//
//         /* Update the tick count */
//         CurrentTime.QuadPart = (*(ULONGLONG*)&KeTickCount) + 1;
//         KiWriteSystemTime(&KeTickCount, CurrentTime);
//
//         /* Update it in the shared user data */
//         KiWriteSystemTime(&SharedUserData->TickCount, CurrentTime);
//
//         /* Check for expiration with the new tick count as well */
//         KiCheckForTimerExpiration(Prcb, TrapFrame, InterruptTime);
//
//         /* Reset the tick offset */
//         KiTickOffset += KeMaximumIncrement;
//
//         /* Update processor/thread runtime */
//         KeUpdateRunTime(TrapFrame, Irql);
//     }
//     else
//     {
//         /* Increase interrupt count only */
//         Prcb->InterruptCount++;
//     }
//
// #ifdef _M_IX86 // x86 optimization
//     /* Disable interrupts and end the interrupt */
//     KiEndInterrupt(Irql, TrapFrame);
// #endif
// }
//
// VOID
// NTAPI
// KeUpdateRunTime(IN PKTRAP_FRAME TrapFrame,
//                 IN KIRQL Irql)
// {
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Check if this tick is being skipped */
//     if (Prcb->SkipTick)
//     {
//         /* Handle it next time */
//         Prcb->SkipTick = FALSE;
//         return;
//     }
//
//     /* Increase interrupt count */
//     Prcb->InterruptCount++;
//
//     /* Check if we came from user mode */
// #if !defined(_M_ARM) && !defined(_M_ARM64)
//     if (KiUserTrap(TrapFrame) || (TrapFrame->EFlags & EFLAGS_V86_MASK))
// #else
//     if (TrapFrame->PreviousMode == UserMode)
// #endif
//     {
//         /* Increase thread user time */
//         Prcb->UserTime++;
//         Thread->UserTime++;
//     }
//     else
//     {
//         /* See if we were in an ISR */
//         Prcb->KernelTime++;
//         if (Irql > DISPATCH_LEVEL)
//         {
//             /* Handle that */
//             Prcb->InterruptTime++;
//         }
//         else if ((Irql < DISPATCH_LEVEL) || !(Prcb->DpcRoutineActive))
//         {
//             /* Handle being in kernel mode */
//             Thread->KernelTime++;
//         }
//         else
//         {
//             /* Handle being in a DPC */
//             Prcb->DpcTime++;
//
// #if DBG
//             /* Update the DPC time */
//             Prcb->DebugDpcTime++;
//
//             /* Check if we have timed out */
//             if (Prcb->DebugDpcTime == KiDPCTimeout)
//             {
//                 /* We did! */
//                 DbgPrint("*** DPC routine > 1 sec --- This is not a break in KeUpdateSystemTime\n");
//
//                 /* Break if debugger is enabled */
//                 if (KdDebuggerEnabled) DbgBreakPoint();
//
//                 /* Clear state */
//                 Prcb->DebugDpcTime = 0;
//             }
// #endif
//         }
//     }
//
//     /* Update DPC rates */
//     Prcb->DpcRequestRate = ((Prcb->DpcData[0].DpcCount - Prcb->DpcLastCount) +
//                             Prcb->DpcRequestRate) >> 1;
//     Prcb->DpcLastCount = Prcb->DpcData[0].DpcCount;
//
//     /* Check if the queue is large enough */
//     if ((Prcb->DpcData[0].DpcQueueDepth) && !(Prcb->DpcRoutineActive))
//     {
//         /* Request a DPC */
//         Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//         HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//
//         /* Fix the maximum queue depth */
//         if ((Prcb->DpcRequestRate < KiIdealDpcRate) &&
//             (Prcb->MaximumDpcQueueDepth > 1))
//         {
//             /* Make it smaller */
//             Prcb->MaximumDpcQueueDepth--;
//         }
//     }
//     else
//     {
//         /* Check if we've reached the adjustment limit */
//         if (!(--Prcb->AdjustDpcThreshold))
//         {
//             /* Reset it, and check the queue maximum */
//             Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//             if (KiMaximumDpcQueueDepth != Prcb->MaximumDpcQueueDepth)
//             {
//                 /* Increase it */
//                 Prcb->MaximumDpcQueueDepth++;
//             }
//         }
//     }
//
//     /* Decrement the thread quantum */
//     Thread->Quantum -= CLOCK_QUANTUM_DECREMENT;
//
//     /* Check if the time expired */
//     if ((Thread->Quantum <= 0) && (Thread != Prcb->IdleThread))
//     {
//         /* Schedule a quantum end */
//         Prcb->QuantumEnd = 1;
//         HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
