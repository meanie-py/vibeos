/*
 * KiArm64ZwDispatch — kernel-mode Zw* dispatch without SVC.
 *
 * Called via tail-call (b, not bl) from Zw* stubs with:
 *   x8    = syscall ID
 *   x0-x7 = original arguments
 *   x30   = original caller's return address (from the Zw* call)
 *   sp    = original caller's stack (with any stack args at [sp], [sp+8]...)
 *
 * Must preserve SP across the call so stack arguments stay aligned.
 */

#define PcPrcb          0xD80
#define PbCurrentThread 0x8
#define ThPreviousMode  0x153

.text
.globl KiArm64ZwDispatch
.p2align 2
KiArm64ZwDispatch:
    /* Save only what we need in callee-saved regs.
     * x19 = syscall ID, x20 = current thread ptr, x21 = old PreviousMode
     * We must push a frame for these + lr. */
    stp x29, x30, [sp, #-48]!
    stp x19, x20, [sp, #16]
    str x21, [sp, #32]
    mov x29, sp

    /* Save syscall ID */
    mov w19, w8

    /* Get current thread */
    mrs x20, tpidr_el1
    ldr x20, [x20, #(PcPrcb + PbCurrentThread)]

    /* Save old PreviousMode and set KernelMode (0) */
    ldrb w21, [x20, #ThPreviousMode]
    strb wzr, [x20, #ThPreviousMode]

    /* Look up SSDT entry */
    adrp x9, KeServiceDescriptorTable
    add  x9, x9, :lo12:KeServiceDescriptorTable
    ldr  x9, [x9]                  /* x9 = Base pointer */
    ldr  x9, [x9, x19, lsl #3]    /* x9 = Base[SyscallId] */

    /* PROBLEM: stack args from caller are at [sp+32] due to our frame.
     * The Nt* function expects them at [sp] (after its own prologue).
     * On ARM64, stack args for > 8 params are passed at the CALLER's SP.
     * We ARE the caller for the Nt* function, so it expects them at OUR SP.
     * But the real stack args are 32 bytes above our SP.
     *
     * Solution: copy stack args down. We support up to 8 extra stack args
     * (max 16 total params, which covers all NT syscalls). */
    ldr x10, [sp, #48]         /* copy stack arg 0 */
    ldr x11, [sp, #56]         /* copy stack arg 1 */
    ldr x12, [sp, #64]         /* copy stack arg 2 */
    ldr x13, [sp, #72]         /* copy stack arg 3 */
    ldr x14, [sp, #80]         /* copy stack arg 4 */
    ldr x15, [sp, #88]         /* copy stack arg 5 */
    ldr x16, [sp, #96]         /* copy stack arg 6 */
    ldr x17, [sp, #104]        /* copy stack arg 7 */
    /* Allocate space for stack args below our frame */
    sub sp, sp, #64
    str x10, [sp]
    str x11, [sp, #8]
    str x12, [sp, #16]
    str x13, [sp, #24]
    str x14, [sp, #32]
    str x15, [sp, #40]
    str x16, [sp, #48]
    str x17, [sp, #56]

    /* Call Nt* function — x0-x7 still have original args */
    blr x9

    /* Deallocate stack arg space */
    add sp, sp, #64

    /* Restore PreviousMode */
    strb w21, [x20, #ThPreviousMode]

    /* Restore callee-saved regs and return */
    ldr x21, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret
