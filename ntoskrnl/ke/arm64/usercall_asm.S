/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/usercall_asm.S
 * PURPOSE:         ARM64 User-Mode Callback setjmp/longjmp
 *
 * KiCallUserMode  — setjmp: saves callee-saved regs into KCALLOUT_FRAME,
 *                   calls KiUserModeCallout which erets to user mode.
 * KiCallbackReturn — longjmp: restores SP to KCALLOUT_FRAME and jumps
 *                    back to KiCallUserModeReturn.
 */

.text

/* KCALLOUT_FRAME offsets (from ndk/arm64/ketypes.h) */
.equ CfX19,           0x00
.equ CfX20,           0x08
.equ CfX21,           0x10
.equ CfX22,           0x18
.equ CfX23,           0x20
.equ CfX24,           0x28
.equ CfX25,           0x30
.equ CfX26,           0x38
.equ CfX27,           0x40
.equ CfX28,           0x48
.equ CfFp,            0x50
.equ CfLr,            0x58
.equ CfOutputBuffer,  0x60
.equ CfOutputLength,  0x68
.equ CfSize,          0x90   /* 144 bytes, 16-byte aligned */


/*
 * NTSTATUS
 * NTAPI
 * KiCallUserMode(
 *     _Out_ PVOID *OutputBuffer,   // x0
 *     _Out_ PULONG OutputLength)   // x1
 *
 * Allocates a KCALLOUT_FRAME on the kernel stack and saves callee-saved
 * registers into it.  Calls KiUserModeCallout(CalloutFrame) which erets
 * to user mode and DOES NOT RETURN through the normal call path.
 *
 * Returns here ONLY via KiCallbackReturn (longjmp from NtCallbackReturn)
 * with the callback status in x0.
 *
 * If KiUserModeCallout returns an error (e.g. stack growth failure),
 * that error is returned directly.
 */
.globl KiCallUserMode
KiCallUserMode:
    /* d8-d15 are callee-saved (AAPCS64) and the callback runs arbitrary user
     * code plus nested traps before we return. KCALLOUT_FRAME has a fixed
     * ketypes.h layout, so they go in their own area just above it;
     * KiCallbackReturn lands on the frame and the return below pops both. */
    stp     d8,  d9,  [sp, #-64]!
    stp     d10, d11, [sp, #16]
    stp     d12, d13, [sp, #32]
    stp     d14, d15, [sp, #48]

    /* Allocate KCALLOUT_FRAME on kernel stack */
    sub     sp, sp, #CfSize

    /* Save callee-saved registers */
    stp     x19, x20, [sp, #CfX19]
    stp     x21, x22, [sp, #CfX21]
    stp     x23, x24, [sp, #CfX23]
    stp     x25, x26, [sp, #CfX25]
    stp     x27, x28, [sp, #CfX27]
    stp     x29, x30, [sp, #CfFp]       /* FP and LR */

    /* Save output pointers for NtCallbackReturn to store results */
    str     x0, [sp, #CfOutputBuffer]
    str     x1, [sp, #CfOutputLength]

    /* Call KiUserModeCallout(CalloutFrame = sp).
     * On success this does NOT return (erets to user mode).
     * On failure (stack growth) it returns NTSTATUS in x0. */
    mov     x0, sp
    bl      KiUserModeCallout

    /* If we reach here, KiUserModeCallout returned an error.
     * Fall through to restore and return the error status. */

/* === LONGJMP TARGET ===
 * Reached via KiCallbackReturn: sp = KCALLOUT_FRAME, x0 = status */
.globl KiCallUserModeReturn
KiCallUserModeReturn:

    /* Restore callee-saved registers */
    ldp     x19, x20, [sp, #CfX19]
    ldp     x21, x22, [sp, #CfX21]
    ldp     x23, x24, [sp, #CfX23]
    ldp     x25, x26, [sp, #CfX25]
    ldp     x27, x28, [sp, #CfX27]
    ldp     x29, x30, [sp, #CfFp]       /* Restore FP and LR */

    /* Deallocate KCALLOUT_FRAME */
    add     sp, sp, #CfSize

    ldp     d10, d11, [sp, #16]
    ldp     d12, d13, [sp, #32]
    ldp     d14, d15, [sp, #48]
    ldp     d8,  d9,  [sp], #64

    /* Return to KeUserModeCallback with status in x0 */
    ret


/*
 * DECLSPEC_NORETURN
 * VOID
 * KiCallbackReturn(
 *     _In_ PKCALLOUT_FRAME CalloutFrame,   // x0
 *     _In_ NTSTATUS CallbackStatus)         // w1
 *
 * Longjmp back to KiCallUserMode by restoring SP to the saved
 * KCALLOUT_FRAME and branching to KiCallUserModeReturn.
 */
.globl KiCallbackReturn
KiCallbackReturn:
    /* Restore kernel SP to the KCALLOUT_FRAME */
    mov     sp, x0

    /* Set return value (status) in x0 */
    mov     w0, w1

    /* Jump to the restore/return sequence */
    b       KiCallUserModeReturn

/* EOF */
