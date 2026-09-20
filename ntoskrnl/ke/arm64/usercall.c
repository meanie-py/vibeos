/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/usercall.c
 * PURPOSE:         ARM64 User-Mode Callback implementation
 *
 * Implements KeUserModeCallback, KiUserModeCallout, and NtCallbackReturn
 * for the ARM64 architecture.
 *
 * Architecture:
 *   Win32k calls KeUserModeCallback(Index, Arg, ArgLen, &Result, &ResultLen)
 *     → Copies argument buffer onto user stack
 *     → Sets trap frame X0/X1/X2 = Index/ArgPtr/ArgLen
 *     → KiCallUserMode (ASM setjmp: saves callee-saved regs)
 *       → KiUserModeCallout (C: saves Thread state, copies trap frame)
 *         → KiExceptionExit (ASM: eret to user mode)
 *
 *   User mode: KiUserCallbackDispatcher (ntdll)
 *     → PEB->KernelCallbackTable[Index](Argument, ArgLen)
 *       → ZwCallbackReturn(Result, ResultLen, Status) → SVC
 *
 *   NtCallbackReturn: restores Thread state from KCALLOUT_FRAME
 *     → KiCallbackReturn (ASM longjmp: restores SP, returns to KiCallUserMode)
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Assembly functions (declared in internal/ke.h but repeated for clarity) */
NTSTATUS NTAPI KiCallUserMode(_Out_ PVOID *OutputBuffer, _Out_ PULONG OutputLength);

/* KiExceptionExit — restores trap frame and erets (from trap.S) */
DECLSPEC_NORETURN VOID KiExceptionExit(_In_ PKTRAP_FRAME TrapFrame, _In_ PKEXCEPTION_FRAME ExceptionFrame);

/* User-mode callback dispatcher address (resolved from ntdll by PspLookupSystemDllEntryPoint) */
extern PVOID KeUserCallbackDispatcher;

/* MmAccessFault — fault in pages (internal MM function) */
NTSTATUS NTAPI MmAccessFault(_In_ ULONG FaultCode, _In_ PVOID Address,
                              _In_ KPROCESSOR_MODE Mode, _In_ PVOID TrapInformation);


/*
 * KiUserModeCallout — called from KiCallUserMode with the KCALLOUT_FRAME
 * as parameter.  Saves thread state, sets up a callback trap frame pointing
 * at KeUserCallbackDispatcher, and erets to user mode.
 *
 * Returns NTSTATUS only on error (e.g. stack growth failure).
 * On success, does NOT return (erets to user mode).
 */
NTSTATUS
KiUserModeCallout(
    _Inout_ PKCALLOUT_FRAME CalloutFrame)
{
    PKTHREAD CurrentThread;
    PKTRAP_FRAME TrapFrame;
    PKTRAP_FRAME CallbackTrapFrame;
    ULONG_PTR InitialStack;

    CurrentThread = KeGetCurrentThread();

    ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
    ASSERT(CurrentThread->ApcStateIndex == OriginalApcEnvironment);

    /* Compute InitialStack = 16-byte aligned bottom of KCALLOUT_FRAME.
     * CfSize is 0x90 (144), already a multiple of 16, so CalloutFrame
     * should be 16-aligned if the stack was aligned before KiCallUserMode. */
    InitialStack = ALIGN_DOWN_BY((ULONG_PTR)CalloutFrame, 16);

    /* Check if we need to grow the kernel stack */
    if ((InitialStack - KERNEL_STACK_SIZE) < (ULONG_PTR)CurrentThread->StackLimit)
    {
        NTSTATUS Status = MmGrowKernelStack((PVOID)InitialStack);
        if (!NT_SUCCESS(Status))
            return Status;
    }

    /* Save current thread state into the callout frame */
    CalloutFrame->CallbackStack = (ULONG64)CurrentThread->CallbackStack;
    CalloutFrame->InitialStack  = (ULONG64)CurrentThread->InitialStack;

    TrapFrame = CurrentThread->TrapFrame;
    CalloutFrame->TrapFrame = (ULONG64)TrapFrame;

    /* Set the new callback stack */
    CurrentThread->CallbackStack = CalloutFrame;

    /* Set the new initial stack (next SVC trap frame goes below this) */
    CurrentThread->InitialStack = (PVOID)InitialStack;

    /* Place the callback trap frame at InitialStack - sizeof(KTRAP_FRAME).
     * After KiExceptionExit → eret, SP_EL1 = InitialStack.
     * The next SVC from user mode will push its trap frame starting from
     * InitialStack, which is just above the KCALLOUT_FRAME — preserving it. */
    CallbackTrapFrame = (PKTRAP_FRAME)(InitialStack - sizeof(KTRAP_FRAME));

    /* Copy the current trap frame (has X0-X2 and Sp set by KeUserModeCallback) */
    *CallbackTrapFrame = *TrapFrame;

    /* Set PC to the user-mode callback dispatcher in ntdll */
    CallbackTrapFrame->Pc = (ULONG64)KeUserCallbackDispatcher;

    /* Ensure SPSR returns to EL0t (user mode): clear M[3:0] bits */
    CallbackTrapFrame->Spsr &= ~0xFu;

    /* Mark as active exception frame */
    CallbackTrapFrame->ExceptionActive = TRUE;

    /* Exit to user mode — DOES NOT RETURN */
    KiExceptionExit(CallbackTrapFrame, NULL);
}


/*
 * KeUserModeCallback — kernel entry point for win32k to invoke user-mode
 * callback functions (window procedures, hook callbacks, etc.).
 *
 * Copies the argument buffer onto the user stack, sets up callback
 * parameters in the trap frame registers, and calls KiCallUserMode
 * which saves kernel state and erets to KiUserCallbackDispatcher in ntdll.
 */
NTSTATUS
NTAPI
KeUserModeCallback(
    _In_ ULONG RoutineIndex,
    _In_ PVOID Argument,
    _In_ ULONG ArgumentLength,
    _Out_ PVOID *Result,
    _Out_ PULONG ResultLength)
{
    PULONG_PTR UserStackPointer;
    ULONG_PTR OldStack;
    ULONG_PTR ArgBase;
    NTSTATUS CallbackStatus;
    PTEB Teb;
    ULONG GdiBatchCount = 0;

    ASSERT(KeGetCurrentThread()->ApcState.KernelApcInProgress == FALSE);

    /* If the current thread has no trap frame, it was created from kernel mode
     * (e.g. desktop thread, RIT) and cannot transition to user mode for a callback.
     * Return STATUS_NO_CALLBACK_ACTIVE so the caller can handle it gracefully. */
    if (!KeGetCurrentThread()->TrapFrame)
    {
        DPRINT1("KeUserModeCallback: Thread %p has no TrapFrame, skipping callback %lu\n",
                KeGetCurrentThread(), RoutineIndex);
        return STATUS_NO_CALLBACK_ACTIVE;
    }

    /* Get the current user-mode stack pointer (from trap frame) */
    UserStackPointer = KiGetUserModeStackAddress();
    OldStack = *UserStackPointer;

    /* Allocate space on the user stack for the argument buffer (16-aligned) */
    ArgBase = ALIGN_DOWN_BY(OldStack - ArgumentLength, 16);

    /* ARM64 DUMMY_PSEH workaround: pre-validate and fault-in user stack pages.
     * On x86/amd64, _SEH2_TRY catches page faults during the copy.
     * On ARM64, we must ensure pages are present beforehand. */
    {
        ULONG_PTR PageStart = ArgBase & ~(PAGE_SIZE - 1);
        ULONG_PTR PageEnd = (ArgBase + ArgumentLength + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
        ULONG_PTR Page;

        for (Page = PageStart; Page < PageEnd; Page += PAGE_SIZE)
        {
            if (!MmIsAddressValid((PVOID)Page))
            {
                /* Fault in the page (write access = 0x02, user mode) */
                NTSTATUS FaultStatus = MmAccessFault(0x02, (PVOID)Page, UserMode, NULL);
                if (!NT_SUCCESS(FaultStatus))
                {
                    DPRINT1("KeUserModeCallback: Failed to fault in user stack page %p (0x%lx)\n",
                            (PVOID)Page, FaultStatus);
                    return STATUS_ACCESS_VIOLATION;
                }
            }
        }
    }

    /* Copy the argument buffer to the user stack */
    RtlCopyMemory((PVOID)ArgBase, Argument, ArgumentLength);

    /* Set callback parameters in the trap frame registers.
     * ARM64 passes these in X0/X1/X2 which KiUserCallbackDispatcher
     * receives as (ULONG Index, PVOID Argument, ULONG ArgumentLength). */
    {
        PKTRAP_FRAME TrapFrame = KeGetCurrentThread()->TrapFrame;
        TrapFrame->X[0] = RoutineIndex;
        TrapFrame->X[1] = ArgBase;
        TrapFrame->X[2] = ArgumentLength;
    }

    /* Update the user stack pointer */
    *UserStackPointer = ArgBase;

    /* Save TEB pointer for GDI batch check */
    Teb = KeGetCurrentThread()->Teb;

    /* Enter the callback: setjmp + eret to user mode.
     * Returns when NtCallbackReturn longjmps back with status. */
    CallbackStatus = KiCallUserMode(Result, ResultLength);

    /* Defined in ke/arm64/trap.c -- shared so both the syscall and the
     * user-callback return paths agree on WHICH TEB carries the count. */
    extern PULONG KiWow64GdiBatchCount(PTEB Teb);

    /* Read GDI batch count (if TEB is accessible).
     * A WoW64 guest bumps TEB32->GdiBatchCount, not TEB64's -- see
     * KiWow64GdiBatchCount in trap.c and field manual §9.41 (2nd occurrence):
     * reading the wrong TEB means the flush never runs for a guest, so
     * gdi32's batch Offset is never reset and its next append walks out of
     * GdiTebBatch.Buffer into CLIENTINFO. */
    if (Teb)
    {
        PULONG pGdiBatchCount = KiWow64GdiBatchCount(Teb);
        if (MmIsAddressValid(pGdiBatchCount))
        {
            GdiBatchCount = *pGdiBatchCount;
        }
    }

    /* Flush GDI batch if needed */
    if (GdiBatchCount)
    {
        *UserStackPointer -= 256;
        KeGdiFlushUserBatch();  /* KeGdiFlushUserBatch is a function pointer (PGDI_BATCHFLUSH_ROUTINE) */
    }

    /* Restore the original user stack pointer */
    *UserStackPointer = OldStack;

    return CallbackStatus;
}


/*
 * NtCallbackReturn — system call invoked by user-mode ZwCallbackReturn
 * to return from a KeUserModeCallback round-trip.
 *
 * Stores the callback result, restores thread state from the KCALLOUT_FRAME,
 * and longjmps back to KiCallUserMode via KiCallbackReturn.
 */
NTSTATUS
NTAPI
NtCallbackReturn(
    _In_ PVOID Result,
    _In_ ULONG ResultLength,
    _In_ NTSTATUS CallbackStatus)
{
    PKTHREAD CurrentThread;
    PKCALLOUT_FRAME CalloutFrame;

    CurrentThread = KeGetCurrentThread();

    /* Get the current callback stack frame */
    CalloutFrame = (PKCALLOUT_FRAME)CurrentThread->CallbackStack;
    if (CalloutFrame == NULL)
    {
        return STATUS_NO_CALLBACK_ACTIVE;
    }

    /* Store the callback results into the caller's output pointers */
    *((PVOID *)CalloutFrame->OutputBuffer) = Result;
    *((ULONG *)CalloutFrame->OutputLength) = ResultLength;

    /* Restore thread state from the callout frame */
    CurrentThread->TrapFrame     = (PKTRAP_FRAME)CalloutFrame->TrapFrame;
    CurrentThread->InitialStack  = (PVOID)CalloutFrame->InitialStack;
    CurrentThread->CallbackStack = (PVOID)CalloutFrame->CallbackStack;

    /* Longjmp back to KiCallUserMode — DOES NOT RETURN */
    KiCallbackReturn((PVOID)CalloutFrame, CallbackStatus);
}
