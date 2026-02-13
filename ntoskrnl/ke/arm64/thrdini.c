/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/thrdini.c
 * PURPOSE:         ARM64 Thread Context Setup, Context Switch Resume, and Idle Loop
 * PROGRAMMERS:     ReactOS ARM64 Port
 */

/* INCLUDES *******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* FORWARD DECLARATIONS *******************************************************/

extern void KiThreadStartup(void);
extern void KiInvalidSystemThreadStartupExit(void);

/* TYPES **********************************************************************/

/*
 * Kernel thread initialization frame layout:
 *   KSWITCH_FRAME   (ApcBypass + Return=KiThreadStartup)
 *   KSTART_FRAME    (StartRoutine, StartContext, SystemRoutine, Return)
 *
 * No KEXCEPTION_FRAME: when a new thread is first switched to,
 * KiSwapContextInternal pops the KSWITCH_FRAME and returns directly
 * to KiThreadStartup (via the Return field), bypassing KiSwapContext's
 * callee-saved register restore.  Same design as amd64.
 */
typedef struct _KKINIT_FRAME
{
    KSWITCH_FRAME CtxSwitchFrame;
    KSTART_FRAME StartFrame;
} KKINIT_FRAME, *PKKINIT_FRAME;

/* FUNCTIONS ******************************************************************/

VOID
NTAPI
KiInitializeContextThread(
    _In_ PKTHREAD Thread,
    _In_ PKSYSTEM_ROUTINE SystemRoutine,
    _In_opt_ PKSTART_ROUTINE StartRoutine,
    _In_opt_ PVOID StartContext,
    _In_ PCONTEXT ContextRecord)
{
    PKKINIT_FRAME InitFrame;
    PKSTART_FRAME StartFrame;
    PKSWITCH_FRAME CtxSwitchFrame;
    PVOID InitialStack;

    if (ContextRecord)
    {
        /* User thread: not implemented yet */
        UNIMPLEMENTED;
        return;
    }

    /*
     * Reserve space at the top of the stack. On amd64, this holds the
     * XSAVE area. On ARM64 we don't have XSAVE, but we still need to
     * reserve space so the KKINIT_FRAME doesn't overlap with the caller's
     * saved registers when initializing the currently-running idle thread.
     * Use 512 bytes (matching amd64's minimum FXSAVE area size).
     */
    InitialStack = (PUCHAR)Thread->InitialStack - 512;
    InitialStack = (PVOID)((ULONG_PTR)InitialStack & ~(ULONG_PTR)15);
    Thread->InitialStack = InitialStack;

    /* Kernel thread: set up the initial stack frame */
    InitFrame = ((PKKINIT_FRAME)InitialStack) - 1;
    StartFrame = &InitFrame->StartFrame;
    CtxSwitchFrame = &InitFrame->CtxSwitchFrame;

    /* Save back the new value of the kernel stack */
    Thread->KernelStack = (PVOID)InitFrame;

    /* Kernel thread runs in kernel mode */
    Thread->PreviousMode = KernelMode;

    /*
     * Set up the context switch frame.
     * Return = KiThreadStartup: when KiSwapContextInternal pops this
     * frame and KiSwapContextResume returns, it goes to KiThreadStartup.
     * ApcBypass = TRUE: skip APC delivery on the first switch.
     */
    CtxSwitchFrame->Return = (ULONG64)KiThreadStartup;
    CtxSwitchFrame->ApcBypass = TRUE;

    /*
     * Set up the start frame with thread parameters.
     * KiThreadStartup will read these from SP and call:
     *   SystemRoutine(StartRoutine, StartContext)
     */
    StartFrame->P1Home = (ULONG64)StartRoutine;
    StartFrame->P2Home = (ULONG64)StartContext;
    StartFrame->P3Home = (ULONG64)SystemRoutine;
    StartFrame->Return = (ULONG64)KiInvalidSystemThreadStartupExit;
}

BOOLEAN
KiSwapContextResume(
    _In_ BOOLEAN ApcBypass,
    _In_ PKTHREAD OldThread,
    _In_ PKTHREAD NewThread)
{
    PKIPCR Pcr = (PKIPCR)KeGetPcr();
    PKPROCESS OldProcess, NewProcess;

    /* Now we are on the new thread's stack. Check for process switch */
    OldProcess = OldThread->ApcState.Process;
    NewProcess = NewThread->ApcState.Process;
    if (OldProcess != NewProcess)
    {
        /* Switch address space (no-op during early boot - all kernel threads
         * share the same address space) */
        KiSwapProcess(NewProcess, OldProcess);
    }

    /* Increase context switch counts */
    Pcr->Prcb.KeContextSwitches++;
    NewThread->ContextSwitches++;

    /* DPCs shouldn't be active */
    if (Pcr->Prcb.DpcRoutineActive)
    {
        KeBugCheckEx(ATTEMPTED_SWITCH_FROM_DPC,
                     (ULONG_PTR)OldThread,
                     (ULONG_PTR)NewThread,
                     (ULONG_PTR)OldThread->InitialStack,
                     0);
    }

    /* Old thread is no longer busy */
    OldThread->SwapBusy = FALSE;

    /* Check for pending kernel APCs */
    if (NewThread->ApcState.KernelApcPending)
    {
        if ((NewThread->SpecialApcDisable == 0) && (ApcBypass == 0))
        {
            /* Return TRUE to indicate that we want APCs to be delivered */
            return TRUE;
        }

        /* Request an APC interrupt to be delivered later */
        HalRequestSoftwareInterrupt(APC_LEVEL);
    }

    /* No kernel APCs pending */
    return FALSE;
}

DECLSPEC_NORETURN
VOID
FASTCALL
KiIdleLoop(VOID)
{
    PKPRCB Prcb = KeGetCurrentPrcb();
    PKTHREAD OldThread, NewThread;

    /* Now loop forever */
    while (TRUE)
    {
        /* Start of the idle loop: enable then disable interrupts */
        _enable();
        YieldProcessor();
        YieldProcessor();
        _disable();

        /* Check for pending timers, pending DPCs, or pending ready threads */
        if ((Prcb->DpcData[0].DpcQueueDepth) ||
            (Prcb->TimerRequest) ||
            (Prcb->DeferredReadyListHead.Next))
        {
            /* Quiesce the DPC software interrupt */
            HalClearSoftwareInterrupt(DISPATCH_LEVEL);

            /* Handle it */
            KiRetireDpcList(Prcb);
        }

        /* Check if a new thread is scheduled for execution */
        if (Prcb->NextThread)
        {
            /* Enable interrupts */
            _enable();

            /* Capture current thread data */
            OldThread = Prcb->CurrentThread;
            NewThread = Prcb->NextThread;

            /* Set new thread data */
            Prcb->NextThread = NULL;
            Prcb->CurrentThread = NewThread;

            /* The thread is now running */
            NewThread->State = Running;

            /* Switch away from the idle thread */
            KiSwapContext(APC_LEVEL, OldThread);
        }
        else
        {
            /* ARM64 Wait For Event - low power idle */
            __asm__ __volatile__("wfe");
        }
    }
}

VOID
NTAPI
KiInitializeHandBuiltThread(
    _In_ PKTHREAD Thread,
    _In_ PKPROCESS Process,
    _In_ PVOID Stack)
{
    PKPRCB Prcb = KeGetCurrentPrcb();

    /* Setup the Thread */
    KeInitializeThread(Process, Thread, NULL, NULL, NULL, NULL, NULL, Stack);

    Thread->NextProcessor = Prcb->Number;
    Thread->IdealProcessor = Prcb->Number;
    Thread->Priority = HIGH_PRIORITY;
    Thread->State = Running;
    Thread->Affinity = (ULONG_PTR)1 << Prcb->Number;
    Thread->WaitIrql = DISPATCH_LEVEL;
    Process->ActiveProcessors |= (ULONG_PTR)1 << Prcb->Number;
}

/* EOF */
