/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/ctxswitch.S
 * PURPOSE:         ARM64 Thread Context Switching
 * PROGRAMMERS:     ReactOS ARM64 Port
 */

/*
 * KTHREAD field offsets (verified by _Static_assert in thrdini.c)
 * These must match the actual structure layout for the ReactOS NTDDI version.
 */
#define ThKernelStack   0x38    /* KTHREAD.KernelStack */
#define ThSwapBusy      0x95    /* KTHREAD.SwapBusy */
#define ThTeb           0xB0    /* KTHREAD.Teb */

/*
 * PCR/PRCB offsets
 * TPIDR_EL1 points to KIPCR.
 * KIPCR.Prcb is at offset 0xD80, KPRCB.CurrentThread is at offset 0x8.
 */
#define PcPrcb              0xD80
#define PbCurrentThread     0x8
#define PcCurrentThread     (PcPrcb + PbCurrentThread)

.text

/*
 * VOID KiThreadStartup(VOID)
 *
 * Entry point for newly created threads. When KiSwapContext switches to a
 * new thread for the first time, it restores the KEXCEPTION_FRAME which has
 * LR = KiThreadStartup. The KSTART_FRAME is above us on the stack.
 *
 * Stack layout on entry:
 *   SP -> KSTART_FRAME
 *     +0x00: P1Home = StartRoutine
 *     +0x08: P2Home = StartContext
 *     +0x10: P3Home = SystemRoutine
 *     +0x18: Return = KiInvalidSystemThreadStartupExit
 */
.globl KiThreadStartup
KiThreadStartup:
    /*
     * Enable IRQs. A new thread inherits PSTATE.DAIF from whichever context
     * performed the swap-in — if that context was DAIF-masked (an SVC wait
     * or an IRQ-exception wake chain), a system thread would otherwise run
     * masked FOREVER (it never ERETs), eventually starving the clock and
     * wedging the machine. Threads start at APC_LEVEL with interrupts on,
     * matching NT (2026-07-05, arm64-interrupts quest).
     */
    msr     daifclr, #2

    /* Lower IRQL to APC_LEVEL (1) to allow APCs */
    mov     x0, #1
    bl      KfLowerIrql

    /* Load start frame parameters from stack */
    ldp     x0, x1, [sp]           /* x0 = StartRoutine, x1 = StartContext */
    ldr     x2, [sp, #16]          /* x2 = SystemRoutine */

    /* Call SystemRoutine(StartRoutine, StartContext) */
    blr     x2

    /*
     * For system threads, SystemRoutine (PspSystemThreadStartup) should
     * never return. If it does, the 'ret' will go to KSTART_FRAME.Return
     * = KiInvalidSystemThreadStartupExit (loaded via ldp x29, x30 below).
     */
    ldr     x30, [sp, #24]         /* Load Return address */
    add     sp, sp, #32            /* Pop KSTART_FRAME */
    ret                            /* Jump to KiInvalidSystemThreadStartupExit */

/*
 * KiInvalidSystemThreadStartupExit
 *
 * A system thread returned from its system routine - this should never happen.
 */
.globl KiInvalidSystemThreadStartupExit
KiInvalidSystemThreadStartupExit:
    brk     #0x200

/*
 * KiUserThreadStartupExit
 *
 * After KiThreadStartup calls SystemRoutine (PspUserThreadStartup) and returns,
 * it pops the KSTART_FRAME (32 bytes) and rets to this address.
 * At this point SP points at the KTRAP_FRAME that was set up by
 * KiInitializeContextThread with the user-mode context.
 *
 * We jump to KiExceptionExit which sets SP = x0 and does KiTrapReturn → eret,
 * transitioning to EL0 (user mode) because SPSR M[3:0] == 0 (EL0t).
 */
.globl KiUserThreadStartupExit
KiUserThreadStartupExit:
    mov     x0, sp
    mov     x1, #0          /* No ExceptionFrame */
    b       KiExceptionExit


/*
 * BOOLEAN FASTCALL
 * KiSwapContext(
 *     _In_ KIRQL WaitIrql,       // x0
 *     _In_ PKTHREAD OldThread);  // x1
 *
 * Saves callee-saved registers (KEXCEPTION_FRAME) and calls
 * KiSwapContextInternal to perform the actual stack switch.
 * When this thread is switched back in, restores the registers and returns.
 */
.globl KiSwapContext
KiSwapContext:
    /*
     * Save KEXCEPTION_FRAME: x19-x28, fp(x29), lr(x30) = 12 regs * 8 = 96 bytes,
     * then d8-d15 (64 bytes). Pre-decrement SP by 160 and store pairs.
     */
    stp     x19, x20, [sp, #-160]!
    stp     x21, x22, [sp, #16]
    stp     x23, x24, [sp, #32]
    stp     x25, x26, [sp, #48]
    stp     x27, x28, [sp, #64]
    stp     x29, x30, [sp, #80]
    /* d8-d15 are callee-saved (AAPCS64), and KiSwapContext is an ordinary
     * call: whatever the next thread leaves in them must not reach this one's
     * caller. This frame is private -- a NEW thread's first switch-in returns
     * from KiSwapContextInternal straight to KiThreadStartup and never runs
     * the restore below, so its initial stack (thrdini.rs) is unaffected. */
    stp     d8,  d9,  [sp, #96]
    stp     d10, d11, [sp, #112]
    stp     d12, d13, [sp, #128]
    stp     d14, d15, [sp, #144]

    /* Get NewThread from PCR->Prcb.CurrentThread
     * (Caller has already set Prcb->CurrentThread = NewThread) */
    mrs     x2, tpidr_el1
    ldr     x2, [x2, #PcCurrentThread]

    /* Call KiSwapContextInternal(WaitIrql=x0, OldThread=x1, NewThread=x2) */
    bl      KiSwapContextInternal

    /* Restore KEXCEPTION_FRAME */
    ldp     x19, x20, [sp]
    ldp     x21, x22, [sp, #16]
    ldp     x23, x24, [sp, #32]
    ldp     x25, x26, [sp, #48]
    ldp     x27, x28, [sp, #64]
    ldp     x29, x30, [sp, #80]
    ldp     d8,  d9,  [sp, #96]
    ldp     d10, d11, [sp, #112]
    ldp     d12, d13, [sp, #128]
    ldp     d14, d15, [sp, #144]
    add     sp, sp, #160

    ret


/*
 * KiSwapContextInternal
 *
 * Internal function that performs the actual stack switch.
 *
 * Parameters:
 *   x0 = WaitIrql / ApcBypass
 *   x1 = OldThread
 *   x2 = NewThread
 *
 * This saves the KSWITCH_FRAME (ApcBypass + return address) on the old
 * thread's stack, switches SP to the new thread's stack, and calls
 * KiSwapContextResume to handle process switch and APC checks.
 */
KiSwapContextInternal:
    /* Wait for SwapBusy on the new thread to clear */
.Lbusy:
    ldrb    w3, [x2, #ThSwapBusy]
    cbnz    w3, .Lbusy

    /*
     * Save KSWITCH_FRAME on old thread's stack:
     *   [sp+0] = ApcBypass (x0)
     *   [sp+8] = Return address (x30)
     */
    stp     x0, x30, [sp, #-16]!

    /* Save old thread's kernel stack */
    mov     x3, sp
    str     x3, [x1, #ThKernelStack]

    /* Load new thread's kernel stack */
    ldr     x3, [x2, #ThKernelStack]
    mov     sp, x3

    /* Set x18 = NewThread->Teb (platform register = TEB on Windows ARM64) */
    ldr     x18, [x2, #ThTeb]

    /* Restore KSWITCH_FRAME from new thread's stack */
    ldp     x0, x30, [sp], #16     /* x0 = ApcBypass, x30 = return address */

    /* Call KiSwapContextResume(ApcBypass, OldThread, NewThread)
     * x0 = ApcBypass (already in x0)
     * x1 = OldThread (already in x1)
     * x2 = NewThread (already in x2)
     */
    b       KiSwapContextResume     /* Tail call - returns to our caller */
