/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/trap.S
 * PURPOSE:         ARM64 Exception Vector Table and Trap Prolog/Epilog
 */

.text

/* KTRAP_FRAME field offsets (from ndk/arm64/ketypes.h) */
.equ TfExceptionActive,    0x00
.equ TfPreviousMode,       0x03
.equ TfPreviousIrql,       0x04
.equ TfFaultAddress,       0x10
.equ TfVfpState,           0x20   /* offset_check_arm64.rs: KTRAP_FRAME.VfpState == 32 */
.equ TfSpsr,               0xA0
.equ TfEsr,                0xA4
.equ TfSp,                 0xA8
.equ TfX0,                 0xB0
.equ TfX1,                 0xB8
.equ TfX2,                 0xC0
.equ TfX3,                 0xC8
.equ TfX4,                 0xD0
.equ TfX5,                 0xD8
.equ TfX6,                 0xE0
.equ TfX7,                 0xE8
.equ TfX8,                 0xF0
.equ TfX9,                 0xF8
.equ TfX10,                0x100
.equ TfX11,                0x108
.equ TfX12,                0x110
.equ TfX13,                0x118
.equ TfX14,                0x120
.equ TfX15,                0x128
.equ TfX16,                0x130
.equ TfX17,                0x138
.equ TfX18,                0x140
.equ TfLr,                 0x148
.equ TfFp,                 0x150
.equ TfPc,                 0x158
.equ KTRAP_FRAME_LENGTH,   0x160

/* KEXCEPTION_FRAME (ndk/arm64/ketypes.h): X19..X28, Fp, Return — 0x60 bytes */
.equ ExX19,                0x00
.equ ExX21,                0x10
.equ ExX23,                0x20
.equ ExX25,                0x30
.equ ExX27,                0x40
.equ ExFp,                 0x50
.equ KEXCEPTION_FRAME_LENGTH, 0x60

/*
 * EXFRAME_SAVE / EXFRAME_RESTORE — capture/restore callee-saved x19..x28
 * around the synchronous exception handler, so KiDispatchException gets a
 * real KEXCEPTION_FRAME (x1) instead of NULL. Without it, KeTrapFrameToContext
 * can't populate Context.X19-X28, and a user-mode NtContinue resume (VEH/SEH
 * "fix up and continue" — the FEX guest-fault path) restores them as zero.
 * The restore after the handler picks up any values KiDispatchException wrote
 * back (handled kernel exception via KeContextToTrapFrame, or the Rd>=19
 * faulting-load zero fixup); for a plain syscall return it reloads the entry
 * values AAPCS already preserved — a no-op.
 * Must be paired: SAVE allocates below the trap frame, RESTORE deallocates
 * so KiTrapReturn sees sp == trap frame.
 */
.macro EXFRAME_SAVE
    sub sp, sp, #KEXCEPTION_FRAME_LENGTH
    stp x19, x20, [sp, #ExX19]
    stp x21, x22, [sp, #ExX21]
    stp x23, x24, [sp, #ExX23]
    stp x25, x26, [sp, #ExX25]
    stp x27, x28, [sp, #ExX27]
    stp x29, x30, [sp, #ExFp]
    mov x1, sp
.endm

.macro EXFRAME_RESTORE
    ldp x19, x20, [sp, #ExX19]
    ldp x21, x22, [sp, #ExX21]
    ldp x23, x24, [sp, #ExX23]
    ldp x25, x26, [sp, #ExX25]
    ldp x27, x28, [sp, #ExX27]
    add sp, sp, #KEXCEPTION_FRAME_LENGTH
.endm

/*
 * NEON_SAVE / NEON_RESTORE — the interrupted context's FP/SIMD state.
 *
 * The kernel runs with FP enabled (kiinit.rs sets CPACR_EL1.FPEN) and the
 * compiler uses the vector registers freely: memcpy/memset, struct zeroing
 * (`movi v0.2d, #0; stp q0, q0`), and copies of frames and contexts all go
 * through q0-q31. Nothing saved them, so any trap handed the interrupted
 * code whatever vector state the handler left: a user thread that took an
 * IRQ in the middle of a vectorized copy resumed with a different q0, and a
 * kernel copy in flight could come out zeroed. That is the shape of the
 * arm64 wild branches (#39: EL1 PC/LR/FP zero; explorer branching to 0x4000
 * through an import stub), and it got far likelier once delay-load thunks
 * kept all eight argument q registers live across a LoadLibrary.
 *
 * All 32 are saved, not just the AAPCS caller-saved ones: an asynchronous
 * trap can land where anything is live, and only the LOW halves of v8-v15
 * are callee-saved. Allocated below the frames so KeGetTrapFrame's
 * InitialStack - sizeof(KTRAP_FRAME) and the x0/x1 handler arguments are
 * untouched. Clobbers x2/x3, which are already in the trap frame.
 *
 * The area is FOUND from the trap frame through KTRAP_FRAME.VfpState, which
 * holds (TrapFrame - area) while the area is live and 0 otherwise. That is
 * what lets KeTrapFrameToContext/KeContextToTrapFrame carry V0-V31, FPCR and
 * FPSR through a CONTEXT, and what lets KiExceptionExit — the NtContinue /
 * NtRaiseException exit, which leaves by `mov sp, TrapFrame` and so never
 * reaches NEON_RESTORE — reload them. Before that, a thread resumed by
 * NtContinue got whatever vector state the kernel last left in the registers.
 * The distance is always one of the two NEON_AREA_BELOW_* values; a frame
 * that was zeroed (a new thread) or copied elsewhere (the user-callback frame
 * clears it) reads as "no area". NEON_RESTORE clears it again, using the
 * frame pointer the save parked at +0x210.
 */
.equ NEON_SAVE_LENGTH, 0x220   /* 32 x 16 bytes + FPCR + FPSR + owning TrapFrame */
.macro NEON_SAVE
    sub sp, sp, #NEON_SAVE_LENGTH
    stp q0,  q1,  [sp, #0x000]
    stp q2,  q3,  [sp, #0x020]
    stp q4,  q5,  [sp, #0x040]
    stp q6,  q7,  [sp, #0x060]
    stp q8,  q9,  [sp, #0x080]
    stp q10, q11, [sp, #0x0a0]
    stp q12, q13, [sp, #0x0c0]
    stp q14, q15, [sp, #0x0e0]
    stp q16, q17, [sp, #0x100]
    stp q18, q19, [sp, #0x120]
    stp q20, q21, [sp, #0x140]
    stp q22, q23, [sp, #0x160]
    stp q24, q25, [sp, #0x180]
    stp q26, q27, [sp, #0x1a0]
    stp q28, q29, [sp, #0x1c0]
    stp q30, q31, [sp, #0x1e0]
    mrs x2, fpcr
    mrs x3, fpsr
    str x2, [sp, #0x200]        /* single stores: an x-pair reaches only +504 */
    str x3, [sp, #0x208]
    str x0, [sp, #0x210]        /* x0 == the trap frame (TRAP_PROLOG) */
    mov x3, sp                  /* sp is not a legal second source of sub */
    sub x2, x0, x3
    str w2, [x0, #TfVfpState]
.endm

.macro NEON_RESTORE
    ldr x2, [sp, #0x210]
    str wzr, [x2, #TfVfpState]
    ldr x2, [sp, #0x200]
    ldr x3, [sp, #0x208]
    msr fpcr, x2
    msr fpsr, x3
    ldp q0,  q1,  [sp, #0x000]
    ldp q2,  q3,  [sp, #0x020]
    ldp q4,  q5,  [sp, #0x040]
    ldp q6,  q7,  [sp, #0x060]
    ldp q8,  q9,  [sp, #0x080]
    ldp q10, q11, [sp, #0x0a0]
    ldp q12, q13, [sp, #0x0c0]
    ldp q14, q15, [sp, #0x0e0]
    ldp q16, q17, [sp, #0x100]
    ldp q18, q19, [sp, #0x120]
    ldp q20, q21, [sp, #0x140]
    ldp q22, q23, [sp, #0x160]
    ldp q24, q25, [sp, #0x180]
    ldp q26, q27, [sp, #0x1a0]
    ldp q28, q29, [sp, #0x1c0]
    ldp q30, q31, [sp, #0x1e0]
    add sp, sp, #NEON_SAVE_LENGTH
.endm

/*
 * UNMASK_IRQ_FROM_SPSR — restore the interrupted context's IRQ mask.
 *
 * Taking ANY exception sets PSTATE.DAIF to all-masked in hardware, so without
 * this a synchronous exception handler runs with IRQs off from entry to ERET.
 * For SVC that means the ENTIRE system call is uninterruptible: interrupt
 * latency is bounded by the longest syscall, and every syscall that blocks
 * parks a DAIF-masked context on the swap chain.
 *
 * The rule is the same one x86 uses: re-enable iff the interrupted context had
 * interrupts enabled — SPSR.I (bit 7) is the saved mask. A fault taken inside
 * an IRQ handler, or inside KiTrapReturn's deliberately-masked ELR/SPSR banking
 * window, therefore stays masked.
 *
 * This does NOT weaken IRQL. IRQL on arm64 is GIC-priority-backed (GICC_PMR,
 * hal-rust arm64/clock.rs): no GIC line lives in the PASSIVE..DISPATCH band, so
 * unmasking here can only admit lines whose DIRQL is genuinely above the
 * current IRQL — which is exactly what NT allows. DAIF is the CPU-wide "no
 * interrupts at all" hammer and belongs only to exception entry/exit windows.
 *
 * Requires x0 = KTRAP_FRAME pointer. Clobbers w2 (caller-saved, already in the
 * frame, and not an argument to the two-argument handler).
 */
.macro UNMASK_IRQ_FROM_SPSR
    ldr w2, [x0, #TfSpsr]
    tbnz w2, #7, 1f
    msr daifclr, #2
1:
.endm

/*
 * TRAP_PROLOG - Save register state into a KTRAP_FRAME on the stack.
 *
 * On entry: running at EL1, using SP_EL1.
 * On exit:  x0 = pointer to KTRAP_FRAME (== sp), all caller-saved regs saved.
 */
.macro TRAP_PROLOG
    /* Allocate trap frame on stack */
    sub sp, sp, #KTRAP_FRAME_LENGTH

    /* Save X0-X18 (caller-saved + X18 platform register) */
    stp x0,  x1,  [sp, #TfX0]
    stp x2,  x3,  [sp, #TfX2]
    stp x4,  x5,  [sp, #TfX4]
    stp x6,  x7,  [sp, #TfX6]
    stp x8,  x9,  [sp, #TfX8]
    stp x10, x11, [sp, #TfX10]
    stp x12, x13, [sp, #TfX12]
    stp x14, x15, [sp, #TfX14]
    stp x16, x17, [sp, #TfX16]
    str x18,      [sp, #TfX18]

    /* Save LR (X30) and FP (X29) */
    str x30, [sp, #TfLr]
    str x29, [sp, #TfFp]

    /* Save ELR_EL1 (faulting PC) */
    mrs x1, elr_el1
    str x1, [sp, #TfPc]

    /* Save SPSR_EL1 */
    mrs x1, spsr_el1
    str w1, [sp, #TfSpsr]

    /* Save ESR_EL1 (exception syndrome) */
    mrs x1, esr_el1
    str w1, [sp, #TfEsr]

    /* Save FAR_EL1 (fault address) */
    mrs x1, far_el1
    str x1, [sp, #TfFaultAddress]

    /* Compute pre-exception SP (before our sub) and save it */
    add x1, sp, #KTRAP_FRAME_LENGTH
    str x1, [sp, #TfSp]

    /* Mark frame as active kernel exception */
    mov w1, #1
    strb w1, [sp, #TfExceptionActive]

    /* Set PreviousMode = KernelMode (0) */
    strb wzr, [sp, #TfPreviousMode]

    /* Set x0 = pointer to trap frame for C handler */
    mov x0, sp
.endm

/*
 * TRAP_PROLOG_EL0 - Save register state from a Lower EL (EL0) exception.
 *
 * Identical to TRAP_PROLOG except:
 *   - Saves SP_EL0 (user stack pointer) to TfSp instead of kernel SP
 *   - Sets TfPreviousMode = UserMode (1)
 *
 * On entry: running at EL1, using SP_EL1.
 * On exit:  x0 = pointer to KTRAP_FRAME (== sp), all caller-saved regs saved.
 */
.macro TRAP_PROLOG_EL0
    /* Allocate trap frame on stack */
    sub sp, sp, #KTRAP_FRAME_LENGTH

    /* Save X0-X18 (caller-saved + X18 platform register) */
    stp x0,  x1,  [sp, #TfX0]
    stp x2,  x3,  [sp, #TfX2]
    stp x4,  x5,  [sp, #TfX4]
    stp x6,  x7,  [sp, #TfX6]
    stp x8,  x9,  [sp, #TfX8]
    stp x10, x11, [sp, #TfX10]
    stp x12, x13, [sp, #TfX12]
    stp x14, x15, [sp, #TfX14]
    stp x16, x17, [sp, #TfX16]
    str x18,      [sp, #TfX18]

    /* Save LR (X30) and FP (X29) */
    str x30, [sp, #TfLr]
    str x29, [sp, #TfFp]

    /* Save ELR_EL1 (faulting PC) */
    mrs x1, elr_el1
    str x1, [sp, #TfPc]

    /* Save SPSR_EL1 */
    mrs x1, spsr_el1
    str w1, [sp, #TfSpsr]

    /* Save ESR_EL1 (exception syndrome) */
    mrs x1, esr_el1
    str w1, [sp, #TfEsr]

    /* Save FAR_EL1 (fault address) */
    mrs x1, far_el1
    str x1, [sp, #TfFaultAddress]

    /* Save SP_EL0 (user stack pointer) */
    mrs x1, sp_el0
    str x1, [sp, #TfSp]

    /* Mark frame as active kernel exception */
    mov w1, #1
    strb w1, [sp, #TfExceptionActive]

    /* Set PreviousMode = UserMode (1) */
    strb w1, [sp, #TfPreviousMode]

    /* Set x0 = pointer to trap frame for C handler */
    mov x0, sp
.endm

/*
 * KiTrapReturn - Restore registers from KTRAP_FRAME at [sp] and eret.
 * Internal helper, not exported.
 *
 * Handles both EL1 and EL0 returns: if SPSR M[3:0] == 0 (EL0t),
 * restore SP_EL0 from TfSp before eret.
 */
KiTrapReturn:
    /* The exception return must be atomic w.r.t. interrupts: ELR_EL1/SPSR_EL1
     * are banked per-EXCEPTION, not per-frame, so an IRQ taken after the msr
     * writes below clobbers them with the interrupted state (ELR =
     * KiTrapReturn+0xC — the insn after `msr spsr_el1`) and the outer eret
     * then targets the nested-IRQ state instead of the frame's. Observed live
     * 2026-07-04: user threads (smss.exe, explorer.exe) resuming at
     * KiTrapReturn+0xC -> UEXC C000001D kill -> 0xF4, plus wandering
     * driver-region 0x50s from the double frame-teardown flavor. Masking here
     * never leaks into the resumed context — eret reloads PSTATE.DAIF from the
     * frame's SPSR. */
    msr daifset, #3
#ifdef ARM64_TRAP_DEBUG
    /* [TRAP-PROBE] inspect the frame for a bad EL0-return state (Spsr/Pc) at
     * the point of use, just before the eret. x0/x1/x30 are dead here (all
     * reloaded from the frame below; KiTrapReturn never returns to its caller). */
    mov x0, sp
    bl KiTrapReturnProbe
#endif
    /* Restore ELR_EL1 */
    ldr x1, [sp, #TfPc]
    msr elr_el1, x1

    /* Restore SPSR_EL1 */
    ldr w1, [sp, #TfSpsr]
    msr spsr_el1, x1

    /* Check if returning to EL0: M[3:0] == 0 means EL0t */
    tst x1, #0xF
    b.ne .Lkernel_ret

    /* Returning to EL0: restore user stack pointer */
    ldr x1, [sp, #TfSp]
    msr sp_el0, x1

.Lkernel_ret:
    /* Restore FP and LR */
    ldr x29, [sp, #TfFp]
    ldr x30, [sp, #TfLr]

    /* Restore X18 */
    ldr x18, [sp, #TfX18]

    /* Restore X16-X17 */
    ldp x16, x17, [sp, #TfX16]

    /* Restore X14-X15 */
    ldp x14, x15, [sp, #TfX14]

    /* Restore X12-X13 */
    ldp x12, x13, [sp, #TfX12]

    /* Restore X10-X11 */
    ldp x10, x11, [sp, #TfX10]

    /* Restore X8-X9 */
    ldp x8,  x9,  [sp, #TfX8]

    /* Restore X6-X7 */
    ldp x6,  x7,  [sp, #TfX6]

    /* Restore X4-X5 */
    ldp x4,  x5,  [sp, #TfX4]

    /* Restore X2-X3 */
    ldp x2,  x3,  [sp, #TfX2]

    /* Restore X0-X1 last */
    ldp x0,  x1,  [sp, #TfX0]

    /* Deallocate trap frame */
    add sp, sp, #KTRAP_FRAME_LENGTH

    /* Return from exception */
    eret


/* ===================================================================== */
/* ARM64 Exception Vector Table                                          */
/*                                                                       */
/* 16 entries: 4 groups x 4 types, each 128 bytes (32 instructions max). */
/* Base must be 2048-byte (0x800) aligned.                               */
/* ===================================================================== */

.balign 2048
.globl KiArm64VectorTable
KiArm64VectorTable:

/* ---- Group 0: Current EL, SP_EL0 (not used, kernel uses SP_ELx) ---- */
/* +0x000: Synchronous */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x080: IRQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x100: FIQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x180: SError */
.balign 128
    b KiUnexpectedExceptionEntry

/* ---- Group 1: Current EL, SP_ELx (this is what the kernel uses) ---- */
/* +0x200: Synchronous */
.balign 128
    b KiSynchronousExceptionEntry

/* +0x280: IRQ */
.balign 128
    b KiIrqEntry

/* +0x300: FIQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x380: SError */
.balign 128
    b KiSErrorEntry

/* ---- Group 2: Lower EL, AArch64 (user mode exceptions) ---- */
/* +0x400: Synchronous */
.balign 128
    b KiEL0SynchronousEntry

/* +0x480: IRQ */
.balign 128
    b KiEL0IrqEntry

/* +0x500: FIQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x580: SError */
.balign 128
    b KiEL0SErrorEntry

/* ---- Group 3: Lower EL, AArch32 (not supported) ---- */
/* +0x600: Synchronous */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x680: IRQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x700: FIQ */
.balign 128
    b KiUnexpectedExceptionEntry

/* +0x780: SError */
.balign 128
    b KiUnexpectedExceptionEntry


/* ===================================================================== */
/* Handler Entries (outside vector table, no 128-byte limit)             */
/* ===================================================================== */

KiSynchronousExceptionEntry:
    TRAP_PROLOG
    EXFRAME_SAVE
    NEON_SAVE
    bl KiSynchronousExceptionHandler
    NEON_RESTORE
    EXFRAME_RESTORE
    b KiTrapReturn

KiIrqEntry:
    TRAP_PROLOG
    NEON_SAVE
    bl KiIrqHandler
    NEON_RESTORE
    b KiTrapReturn

KiSErrorEntry:
    TRAP_PROLOG
    bl KiSErrorHandler
    /* SError is fatal — hang */
1:  wfe
    b 1b

KiEL0SynchronousEntry:
    TRAP_PROLOG_EL0
    EXFRAME_SAVE
    NEON_SAVE
    /* Syscalls and user faults arrive from EL0, i.e. from PASSIVE_LEVEL with
     * interrupts on, so this always unmasks. KiTrapReturn re-masks before it
     * touches ELR/SPSR. */
    UNMASK_IRQ_FROM_SPSR
    bl KiSynchronousExceptionHandler
    NEON_RESTORE
    EXFRAME_RESTORE
    b KiTrapReturn

KiEL0IrqEntry:
    TRAP_PROLOG_EL0
    NEON_SAVE
    bl KiIrqHandler
    /* Quantum end / readied thread: switch here, while the user's NEON state is
     * still saved below the frame (another thread will use the registers). */
    bl KiEL0IrqPreempt
    NEON_RESTORE
    b KiTrapReturn

KiEL0SErrorEntry:
    TRAP_PROLOG_EL0
    bl KiSErrorHandler
    /* SError is fatal — hang */
1:  wfe
    b 1b

KiUnexpectedExceptionEntry:
    TRAP_PROLOG
    bl KiUnexpectedHandler
    /* Unexpected is fatal — hang */
1:  wfe
    b 1b


/* ===================================================================== */
/* KiExceptionExit — restore trap frame and return from exception        */
/*                                                                       */
/* DECLSPEC_NORETURN VOID KiExceptionExit(PKTRAP_FRAME TrapFrame,        */
/*                                        PKEXCEPTION_FRAME ExFrame);    */
/* x0 = TrapFrame pointer                                                */
/* ===================================================================== */

.globl KiExceptionExit
KiExceptionExit:
    /* x0 = TrapFrame, x1 = ExceptionFrame (may be NULL) */
    cbz     x1, .Lno_exframe

    /* Restore callee-saved regs from KEXCEPTION_FRAME */
    ldp     x19, x20, [x1, #0]     /* ExX19=0x00, ExX20=0x08 */
    ldp     x21, x22, [x1, #16]    /* ExX21=0x10, ExX22=0x18 */
    ldp     x23, x24, [x1, #32]    /* ExX23=0x20, ExX24=0x28 */
    ldp     x25, x26, [x1, #48]    /* ExX25=0x30, ExX26=0x38 */
    ldp     x27, x28, [x1, #64]    /* ExX27=0x40, ExX28=0x48 */
    /* Fp and Lr come from TrapFrame, not ExceptionFrame */

.Lno_exframe:
    /* Reload the vector state from the frame's NEON area, if it has one (see
     * NEON_SAVE). KeContextToTrapFrame may have written a CONTEXT's V0-V31
     * into it; otherwise it still holds what the trap saved. */
    ldr     w2, [x0, #TfVfpState]
    cmp     w2, #NEON_SAVE_LENGTH                            /* IRQ entry */
    b.eq    1f
    cmp     w2, #(NEON_SAVE_LENGTH + KEXCEPTION_FRAME_LENGTH) /* sync entry */
    b.ne    .Lno_neon           /* 0, or any value no entry path writes */
1:  sub     x3, x0, x2
    str     wzr, [x0, #TfVfpState]
    ldr     x2, [x3, #0x200]
    msr     fpcr, x2
    ldr     x2, [x3, #0x208]
    msr     fpsr, x2
    ldp     q0,  q1,  [x3, #0x000]
    ldp     q2,  q3,  [x3, #0x020]
    ldp     q4,  q5,  [x3, #0x040]
    ldp     q6,  q7,  [x3, #0x060]
    ldp     q8,  q9,  [x3, #0x080]
    ldp     q10, q11, [x3, #0x0a0]
    ldp     q12, q13, [x3, #0x0c0]
    ldp     q14, q15, [x3, #0x0e0]
    ldp     q16, q17, [x3, #0x100]
    ldp     q18, q19, [x3, #0x120]
    ldp     q20, q21, [x3, #0x140]
    ldp     q22, q23, [x3, #0x160]
    ldp     q24, q25, [x3, #0x180]
    ldp     q26, q27, [x3, #0x1a0]
    ldp     q28, q29, [x3, #0x1c0]
    ldp     q30, q31, [x3, #0x1e0]

.Lno_neon:
    /* Set SP to trap frame and return */
    mov     sp, x0
    b       KiTrapReturn

/* EOF */
