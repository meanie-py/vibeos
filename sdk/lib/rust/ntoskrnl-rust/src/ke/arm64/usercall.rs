use crate::ke::ke_defs::*;
// KeGdiFlushUserBatch: ps-rust global read across the crate boundary as a link symbol
// (#[no_mangle] in ps-rust). crate-split Phase 1b.
extern "C" {
    static mut KeGdiFlushUserBatch: ntkrnl_sys::ddk::PGDI_BATCHFLUSH_ROUTINE;
}
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _KTHREAD,
    KPROCESSOR_MODE,
    KTRAP_FRAME, PKTRAP_FRAME,
    PKTHREAD,
    PTEB,
    PKCALLOUT_FRAME,
    PKEXCEPTION_FRAME,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// Get current thread as PKTHREAD (avoids PVOID ambiguity from mm::KeGetCurrentThread)
#[inline(always)]
unsafe fn KiGetCurrentThread() -> PKTHREAD {
    ntkrnl_sys::mm::KeGetCurrentThread() as PKTHREAD
}

extern "system" {
    fn KiCallUserMode(OutputBuffer: *mut PVOID, OutputLength: *mut ULONG) -> NTSTATUS;
    fn KiExceptionExit(TrapFrame: PKTRAP_FRAME, ExceptionFrame: PKEXCEPTION_FRAME) -> !;
    fn KiCallbackReturn(CalloutFrame: PVOID, Status: NTSTATUS) -> !;
    fn MmAccessFault(FaultCode: ULONG, Address: PVOID, Mode: KPROCESSOR_MODE, TrapInformation: PVOID) -> NTSTATUS;
    fn MmGrowKernelStack(StackPointer: PVOID) -> NTSTATUS;
    fn MmIsAddressValid(Address: PVOID) -> BOOLEAN;
    static KeUserCallbackDispatcher: PVOID;
}
extern "C" {
    fn KiGetUserModeStackAddress() -> *mut ULONG_PTR;
}

const KERNEL_STACK_SIZE: ULONG_PTR = 0xC000; // 48KB — matches xdk/arm64/ke.h (bumped from i386's 12KB)
const USER_MODE: KPROCESSOR_MODE = 1;
const OriginalApcEnvironment: u8 = 0;

#[no_mangle]
pub unsafe extern "C" fn KiUserModeCallout(CalloutFrame: PKCALLOUT_FRAME) -> NTSTATUS {
    let CurrentThread: PKTHREAD = KiGetCurrentThread();

    let TrapFrameSize = core::mem::size_of::<KTRAP_FRAME>() as ULONG_PTR;
    let InitialStack: ULONG_PTR = ALIGN_DOWN_BY(CalloutFrame as ULONG_PTR, 16);

    if (InitialStack - KERNEL_STACK_SIZE) < (*CurrentThread).StackLimit as ULONG_PTR {
        let Status = MmGrowKernelStack(InitialStack as PVOID);
        if Status < 0 {
            return Status;
        }
    }

    (*CalloutFrame).CallbackStack = (*CurrentThread).CallbackStack as u64;
    (*CalloutFrame).InitialStack = (*CurrentThread).InitialStack as u64;

    let TrapFrame: PKTRAP_FRAME = (*CurrentThread).TrapFrame;
    (*CalloutFrame).TrapFrame = TrapFrame as u64;

    (*CurrentThread).CallbackStack = CalloutFrame as PVOID;
    (*CurrentThread).InitialStack = InitialStack as PVOID;

    let CallbackTrapFrame: PKTRAP_FRAME = (InitialStack - TrapFrameSize) as PKTRAP_FRAME;

    // The callback frame is carved BELOW our own live SP (this function's
    // frame sits at [CalloutFrame - F, CalloutFrame); the carve-out spans
    // [CalloutFrame - 0x160, CalloutFrame)). An IRQ taken between the copy
    // below and KiExceptionExit's `mov sp` pushes ITS trap frame at
    // [SP - 0x160, SP) — inside the carve-out — clobbering the low-offset
    // fields: Spsr (+0xA0) is exposed while Pc (+0x158) survives. Observed
    // live 2026-07-11: eret to KiUserCallbackDispatcher with Spsr=0x102085
    // (IRQ-frame fragment) → Illegal Execution State → 0x1E/C000001D flake.
    // Mask IRQs for the window; eret reloads PSTATE.DAIF from the frame's
    // Spsr (=0), so the mask never leaks into the resumed user context
    // (same argument as KiTrapReturn's daifset).
    core::arch::asm!("msr daifset, #3", options(nomem, nostack, preserves_flags));

    // Snapshot the trap frame into the callback frame. A typed struct
    // assignment (`*dst = *src`) of this union-bearing KTRAP_FRAME was
    // observed to NOT copy the X[18] (x18) slot on arm64 — the callback
    // frame's x18 stayed kernel-stack garbage, so KiTrapReturn ereted into
    // KeUserCallbackDispatcher with a bogus x18; ntdll then read TEB->PEB via
    // [x18+0x60] == NULL → AV → recursive KiUserExceptionDispatcher stack
    // overflow → winlogon death → BUGCHECK 0xF4. Force a full byte copy.
    core::ptr::copy_nonoverlapping(
        TrapFrame as *const u8,
        CallbackTrapFrame as *mut u8,
        core::mem::size_of::<KTRAP_FRAME>(),
    );

    // x18 is the arm64 platform (TEB) register; the dispatcher relies on it.
    // Set it authoritatively from the thread's TEB regardless of the copy.
    (*CallbackTrapFrame).__bindgen_anon_3.X[18] = (*CurrentThread).Teb as u64;

    (*CallbackTrapFrame).Pc = KeUserCallbackDispatcher as u64;

    (*CallbackTrapFrame).Spsr = 0;

    (*CallbackTrapFrame).__bindgen_anon_3.X[0] = (*TrapFrame).__bindgen_anon_3.X[0];
    (*CallbackTrapFrame).__bindgen_anon_3.X[1] = (*TrapFrame).__bindgen_anon_3.X[1];
    (*CallbackTrapFrame).__bindgen_anon_3.X[2] = (*TrapFrame).__bindgen_anon_3.X[2];
    (*CallbackTrapFrame).Sp = (*TrapFrame).Sp;

    (*CallbackTrapFrame).ExceptionActive = 1u8;
    // The copy carried VfpState, a DISTANCE to the NEON area below the ORIGINAL
    // frame (trap.S NEON_SAVE); measured from here it lands on this function's own
    // stack, and KiExceptionExit would load q0-q31 from it. The callback starts a
    // fresh call, so it needs no vector state.
    (*CallbackTrapFrame).VfpState = 0;

    KiExceptionExit(CallbackTrapFrame, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn KeUserModeCallback(
    RoutineIndex: ULONG,
    Argument: PVOID,
    ArgumentLength: ULONG,
    Result: *mut PVOID,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let Argument = opaque_ptr(Argument);
    let Result = opaque_ptr(Result);
    let ResultLength = opaque_ptr(ResultLength);
    let UserStackPointer: *mut ULONG_PTR = KiGetUserModeStackAddress();
    let OldStack: ULONG_PTR = *UserStackPointer;

    let ArgBase: ULONG_PTR = ALIGN_DOWN_BY(OldStack - ArgumentLength as ULONG_PTR, 16);

    {
        let PageStart: ULONG_PTR = ArgBase & !(PAGE_SIZE - 1);
        let PageEnd: ULONG_PTR = (ArgBase + ArgumentLength as ULONG_PTR + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let mut Page: ULONG_PTR = PageStart;

        while Page < PageEnd {
            if MmIsAddressValid(Page as PVOID) == 0u8 {
                let FaultStatus = MmAccessFault(0x02, Page as PVOID, USER_MODE, core::ptr::null_mut());
                if FaultStatus < 0 {
                    return STATUS_ACCESS_VIOLATION;
                }
            }
            Page += PAGE_SIZE;
        }
    }

    core::ptr::copy_nonoverlapping(
        Argument as *const u8,
        ArgBase as *mut u8,
        ArgumentLength as usize,
    );

    {
        let TrapFrame: PKTRAP_FRAME = (*KiGetCurrentThread()).TrapFrame;
        (*TrapFrame).__bindgen_anon_3.X[0] = RoutineIndex as u64;
        (*TrapFrame).__bindgen_anon_3.X[1] = ArgBase as u64;
        (*TrapFrame).__bindgen_anon_3.X[2] = ArgumentLength as u64;
        (*TrapFrame).Sp = ArgBase as u64;
    }

    *UserStackPointer = ArgBase;

    let Teb: PTEB = (*KiGetCurrentThread()).Teb;

    let CallbackStatus: NTSTATUS = KiCallUserMode(Result, ResultLength);

    // TEB32's count for a WoW64 guest (trap.rs GdiBatchCountOf).
    let mut GdiBatchCount: ULONG = 0;
    if !Teb.is_null() {
        let Count = crate::ke::arm64::trap::GdiBatchCountOf(Teb as *const _);
        if MmIsAddressValid(Count as PVOID) != 0u8 {
            GdiBatchCount = *Count;
        }
    }

    if GdiBatchCount != 0 {
        *UserStackPointer -= 256;
        if let Some(flush) = KeGdiFlushUserBatch {
            flush();
        }
    }

    *UserStackPointer = OldStack;

    CallbackStatus
}

#[no_mangle]
pub unsafe extern "system" fn NtCallbackReturn(
    Result: PVOID,
    ResultLength: ULONG,
    CallbackStatus: NTSTATUS,
) -> NTSTATUS {
    let Result = opaque_ptr(Result);
    let CurrentThread: PKTHREAD = KiGetCurrentThread();

    let CalloutFrame: PKCALLOUT_FRAME = (*CurrentThread).CallbackStack as PKCALLOUT_FRAME;
    if CalloutFrame.is_null() {
        return STATUS_NO_CALLBACK_ACTIVE;
    }

    *(((*CalloutFrame).OutputBuffer as *mut PVOID)) = Result;
    *(((*CalloutFrame).OutputLength as *mut ULONG)) = ResultLength;

    (*CurrentThread).TrapFrame = (*CalloutFrame).TrapFrame as PKTRAP_FRAME;
    (*CurrentThread).InitialStack = (*CalloutFrame).InitialStack as PVOID;
    (*CurrentThread).CallbackStack = (*CalloutFrame).CallbackStack as PVOID;

    KiCallbackReturn(CalloutFrame as PVOID, CallbackStatus);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL-2.0-or-later
//  * FILE:            ntoskrnl/ke/arm64/usercall.c
//  * PURPOSE:         ARM64 User-Mode Callback implementation
//  *
//  * Implements KeUserModeCallback, KiUserModeCallout, and NtCallbackReturn
//  * for the ARM64 architecture.
//  *
//  * Architecture:
//  *   Win32k calls KeUserModeCallback(Index, Arg, ArgLen, &Result, &ResultLen)
//  *     → Copies argument buffer onto user stack
//  *     → Sets trap frame X0/X1/X2 = Index/ArgPtr/ArgLen
//  *     → KiCallUserMode (ASM setjmp: saves callee-saved regs)
//  *       → KiUserModeCallout (C: saves Thread state, copies trap frame)
//  *         → KiExceptionExit (ASM: eret to user mode)
//  *
//  *   User mode: KiUserCallbackDispatcher (ntdll)
//  *     → PEB->KernelCallbackTable[Index](Argument, ArgLen)
//  *       → ZwCallbackReturn(Result, ResultLen, Status) → SVC
//  *
//  *   NtCallbackReturn: restores Thread state from KCALLOUT_FRAME
//  *     → KiCallbackReturn (ASM longjmp: restores SP, returns to KiCallUserMode)
//  */
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Assembly functions (declared in internal/ke.h but repeated for clarity) */
// NTSTATUS NTAPI KiCallUserMode(_Out_ PVOID *OutputBuffer, _Out_ PULONG OutputLength);
//
// /* KiExceptionExit — restores trap frame and erets (from trap.S) */
// DECLSPEC_NORETURN VOID KiExceptionExit(_In_ PKTRAP_FRAME TrapFrame, _In_ PKEXCEPTION_FRAME ExceptionFrame);
//
// /* User-mode callback dispatcher address (resolved from ntdll by PspLookupSystemDllEntryPoint) */
// extern PVOID KeUserCallbackDispatcher;
//
// /* MmAccessFault — fault in pages (internal MM function) */
// NTSTATUS NTAPI MmAccessFault(_In_ ULONG FaultCode, _In_ PVOID Address,
//                               _In_ KPROCESSOR_MODE Mode, _In_ PVOID TrapInformation);
//
//
// /*
//  * KiUserModeCallout — called from KiCallUserMode with the KCALLOUT_FRAME
//  * as parameter.  Saves thread state, sets up a callback trap frame pointing
//  * at KeUserCallbackDispatcher, and erets to user mode.
//  *
//  * Returns NTSTATUS only on error (e.g. stack growth failure).
//  * On success, does NOT return (erets to user mode).
//  */
// NTSTATUS
// KiUserModeCallout(
//     _Inout_ PKCALLOUT_FRAME CalloutFrame)
// {
//     PKTHREAD CurrentThread;
//     PKTRAP_FRAME TrapFrame;
//     PKTRAP_FRAME CallbackTrapFrame;
//     ULONG_PTR InitialStack;
//
//     CurrentThread = KeGetCurrentThread();
//
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//     ASSERT(CurrentThread->ApcStateIndex == OriginalApcEnvironment);
//
//     /* Compute InitialStack = 16-byte aligned bottom of KCALLOUT_FRAME.
//      * CfSize is 0x90 (144), already a multiple of 16, so CalloutFrame
//      * should be 16-aligned if the stack was aligned before KiCallUserMode. */
//     InitialStack = ALIGN_DOWN_BY((ULONG_PTR)CalloutFrame, 16);
//
//     /* Check if we need to grow the kernel stack */
//     if ((InitialStack - KERNEL_STACK_SIZE) < (ULONG_PTR)CurrentThread->StackLimit)
//     {
//         NTSTATUS Status = MmGrowKernelStack((PVOID)InitialStack);
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     /* Save current thread state into the callout frame */
//     CalloutFrame->CallbackStack = (ULONG64)CurrentThread->CallbackStack;
//     CalloutFrame->InitialStack  = (ULONG64)CurrentThread->InitialStack;
//
//     TrapFrame = CurrentThread->TrapFrame;
//     CalloutFrame->TrapFrame = (ULONG64)TrapFrame;
//
//     /* Set the new callback stack */
//     CurrentThread->CallbackStack = CalloutFrame;
//
//     /* Set the new initial stack (next SVC trap frame goes below this) */
//     CurrentThread->InitialStack = (PVOID)InitialStack;
//
//     /* Place the callback trap frame at InitialStack - sizeof(KTRAP_FRAME).
//      * After KiExceptionExit → eret, SP_EL1 = InitialStack.
//      * The next SVC from user mode will push its trap frame starting from
//      * InitialStack, which is just above the KCALLOUT_FRAME — preserving it. */
//     CallbackTrapFrame = (PKTRAP_FRAME)(InitialStack - sizeof(KTRAP_FRAME));
//
//     /* Copy the current trap frame (has X0-X2 and Sp set by KeUserModeCallback) */
//     *CallbackTrapFrame = *TrapFrame;
//
//     /* Set PC to the user-mode callback dispatcher in ntdll */
//     CallbackTrapFrame->Pc = (ULONG64)KeUserCallbackDispatcher;
//
//     /* Ensure SPSR returns to EL0t (user mode): clear M[3:0] bits */
//     CallbackTrapFrame->Spsr &= ~0xFu;
//
//     /* Mark as active exception frame */
//     CallbackTrapFrame->ExceptionActive = TRUE;
//
//     /* Exit to user mode — DOES NOT RETURN */
//     KiExceptionExit(CallbackTrapFrame, NULL);
// }
//
//
// /*
//  * KeUserModeCallback — kernel entry point for win32k to invoke user-mode
//  * callback functions (window procedures, hook callbacks, etc.).
//  *
//  * Copies the argument buffer onto the user stack, sets up callback
//  * parameters in the trap frame registers, and calls KiCallUserMode
//  * which saves kernel state and erets to KiUserCallbackDispatcher in ntdll.
//  */
// NTSTATUS
// NTAPI
// KeUserModeCallback(
//     _In_ ULONG RoutineIndex,
//     _In_ PVOID Argument,
//     _In_ ULONG ArgumentLength,
//     _Out_ PVOID *Result,
//     _Out_ PULONG ResultLength)
// {
//     PULONG_PTR UserStackPointer;
//     ULONG_PTR OldStack;
//     ULONG_PTR ArgBase;
//     NTSTATUS CallbackStatus;
//     PTEB Teb;
//     ULONG GdiBatchCount = 0;
//
//     ASSERT(KeGetCurrentThread()->ApcState.KernelApcInProgress == FALSE);
//
//     /* If the current thread has no trap frame, it was created from kernel mode
//      * (e.g. desktop thread, RIT) and cannot transition to user mode for a callback.
//      * Return STATUS_NO_CALLBACK_ACTIVE so the caller can handle it gracefully. */
//     if (!KeGetCurrentThread()->TrapFrame)
//     {
//         DPRINT1("KeUserModeCallback: Thread %p has no TrapFrame, skipping callback %lu\n",
//                 KeGetCurrentThread(), RoutineIndex);
//         return STATUS_NO_CALLBACK_ACTIVE;
//     }
//
//     /* Get the current user-mode stack pointer (from trap frame) */
//     UserStackPointer = KiGetUserModeStackAddress();
//     OldStack = *UserStackPointer;
//
//     /* Allocate space on the user stack for the argument buffer (16-aligned) */
//     ArgBase = ALIGN_DOWN_BY(OldStack - ArgumentLength, 16);
//
//     /* ARM64 DUMMY_PSEH workaround: pre-validate and fault-in user stack pages.
//      * On x86/amd64, _SEH2_TRY catches page faults during the copy.
//      * On ARM64, we must ensure pages are present beforehand. */
//     {
//         ULONG_PTR PageStart = ArgBase & ~(PAGE_SIZE - 1);
//         ULONG_PTR PageEnd = (ArgBase + ArgumentLength + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
//         ULONG_PTR Page;
//
//         for (Page = PageStart; Page < PageEnd; Page += PAGE_SIZE)
//         {
//             if (!MmIsAddressValid((PVOID)Page))
//             {
//                 /* Fault in the page (write access = 0x02, user mode) */
//                 NTSTATUS FaultStatus = MmAccessFault(0x02, (PVOID)Page, UserMode, NULL);
//                 if (!NT_SUCCESS(FaultStatus))
//                 {
//                     DPRINT1("KeUserModeCallback: Failed to fault in user stack page %p (0x%lx)\n",
//                             (PVOID)Page, FaultStatus);
//                     return STATUS_ACCESS_VIOLATION;
//                 }
//             }
//         }
//     }
//
//     /* Copy the argument buffer to the user stack */
//     RtlCopyMemory((PVOID)ArgBase, Argument, ArgumentLength);
//
//     /* Set callback parameters in the trap frame registers.
//      * ARM64 passes these in X0/X1/X2 which KiUserCallbackDispatcher
//      * receives as (ULONG Index, PVOID Argument, ULONG ArgumentLength). */
//     {
//         PKTRAP_FRAME TrapFrame = KeGetCurrentThread()->TrapFrame;
//         TrapFrame->X[0] = RoutineIndex;
//         TrapFrame->X[1] = ArgBase;
//         TrapFrame->X[2] = ArgumentLength;
//     }
//
//     /* Update the user stack pointer */
//     *UserStackPointer = ArgBase;
//
//     /* Save TEB pointer for GDI batch check */
//     Teb = KeGetCurrentThread()->Teb;
//
//     /* Enter the callback: setjmp + eret to user mode.
//      * Returns when NtCallbackReturn longjmps back with status. */
//     CallbackStatus = KiCallUserMode(Result, ResultLength);
//
//     /* Read GDI batch count (if TEB is accessible) */
//     if (Teb && MmIsAddressValid(&Teb->GdiBatchCount))
//     {
//         GdiBatchCount = Teb->GdiBatchCount;
//     }
//
//     /* Flush GDI batch if needed */
//     if (GdiBatchCount)
//     {
//         *UserStackPointer -= 256;
//         KeGdiFlushUserBatch();  /* KeGdiFlushUserBatch is a function pointer (PGDI_BATCHFLUSH_ROUTINE) */
//     }
//
//     /* Restore the original user stack pointer */
//     *UserStackPointer = OldStack;
//
//     return CallbackStatus;
// }
//
//
// /*
//  * NtCallbackReturn — system call invoked by user-mode ZwCallbackReturn
//  * to return from a KeUserModeCallback round-trip.
//  *
//  * Stores the callback result, restores thread state from the KCALLOUT_FRAME,
//  * and longjmps back to KiCallUserMode via KiCallbackReturn.
//  */
// NTSTATUS
// NTAPI
// NtCallbackReturn(
//     _In_ PVOID Result,
//     _In_ ULONG ResultLength,
//     _In_ NTSTATUS CallbackStatus)
// {
//     PKTHREAD CurrentThread;
//     PKCALLOUT_FRAME CalloutFrame;
//
//     CurrentThread = KeGetCurrentThread();
//
//     /* Get the current callback stack frame */
//     CalloutFrame = (PKCALLOUT_FRAME)CurrentThread->CallbackStack;
//     if (CalloutFrame == NULL)
//     {
//         return STATUS_NO_CALLBACK_ACTIVE;
//     }
//
//     /* Store the callback results into the caller's output pointers */
//     *((PVOID *)CalloutFrame->OutputBuffer) = Result;
//     *((ULONG *)CalloutFrame->OutputLength) = ResultLength;
//
//     /* Restore thread state from the callout frame */
//     CurrentThread->TrapFrame     = (PKTRAP_FRAME)CalloutFrame->TrapFrame;
//     CurrentThread->InitialStack  = (PVOID)CalloutFrame->InitialStack;
//     CurrentThread->CallbackStack = (PVOID)CalloutFrame->CallbackStack;
//
//     /* Longjmp back to KiCallUserMode — DOES NOT RETURN */
//     KiCallbackReturn((PVOID)CalloutFrame, CallbackStatus);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
