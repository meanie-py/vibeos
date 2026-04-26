/*
 * PROJECT:     FreeLoader UEFI Support
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     ARM64 assembly UEFI escape helper functions
 * COPYRIGHT:   Copyright 2024 ReactOS ARM64 Port
 */

#include <asm.inc>

    TEXTAREA

/*
 * External references
 */
    EXTERN UefiServiceStack
    EXTERN BasicStack
    EXTERN ExecuteLoaderCleanly
    EXTERN UefiExitBootServices

/*
 * void _exituefi(VOID)
 *
 * Exit UEFI boot services and prepare for OS boot
 */
    LEAF_ENTRY _exituefi

    /* Save callee-saved registers */
    stp x19, x20, [sp, #-16]!
    stp x21, x22, [sp, #-16]!
    stp x23, x24, [sp, #-16]!
    stp x25, x26, [sp, #-16]!
    stp x27, x28, [sp, #-16]!
    stp x29, x30, [sp, #-16]!

    /* Save the old stack pointer */
    mov x19, sp

    /* Load the UEFI service stack
     * Use adrp+add+ldr to avoid :lo12: alignment requirements in PE-COFF */
    adrp x0, UefiServiceStack
    add x0, x0, :lo12:UefiServiceStack
    ldr x0, [x0]
    mov sp, x0

    /* Call UefiExitBootServices (function, not pointer — don't dereference) */
    bl UefiExitBootServices

    /* Restore old stack */
    mov sp, x19

    /* Restore callee-saved registers */
    ldp x29, x30, [sp], #16
    ldp x27, x28, [sp], #16
    ldp x25, x26, [sp], #16
    ldp x23, x24, [sp], #16
    ldp x21, x22, [sp], #16
    ldp x19, x20, [sp], #16

    /* Return */
    ret

    LEAF_END _exituefi

/*
 * void _changestack(VOID)
 *
 * Switch to the basic stack and execute the loader
 */
    LEAF_ENTRY _changestack

    /* Save current stack pointer */
    mov x0, sp

    /* Load the basic stack
     * Use adrp+add+ldr to avoid :lo12: alignment requirements in PE-COFF */
    adrp x1, BasicStack
    add x1, x1, :lo12:BasicStack
    ldr x1, [x1]
    mov sp, x1

    /* Push old stack pointer (also serves as first arg to ExecuteLoaderCleanly) */
    str x0, [sp, #-16]!

    /* Call ExecuteLoaderCleanly(x0 = PreviousStack)
     * x0 still holds the old SP from above — function, not pointer */
    bl ExecuteLoaderCleanly

    /* Restore stack (shouldn't reach here) */
    ldr x0, [sp], #16
    mov sp, x0

    ret

    LEAF_END _changestack

    END
