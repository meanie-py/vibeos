/*
 * PROJECT:     ReactOS vcruntime library
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     Implementation of __longjmp_noframe for ARM64
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 */

#include <kxarm64.h>

/* _JUMP_BUFFER field offsets for ARM64 */
#define JbFrame     0x00
#define JbReserved  0x08
#define JbX19       0x10
#define JbX20       0x18
#define JbX21       0x20
#define JbX22       0x28
#define JbX23       0x30
#define JbX24       0x38
#define JbX25       0x40
#define JbX26       0x48
#define JbX27       0x50
#define JbX28       0x58
#define JbFp        0x60
#define JbLr        0x68
#define JbSp        0x70
#define JbFpcr      0x78
#define JbFpsr      0x7c
#define JbD8        0x80

    TEXTAREA

//
// __declspec(noreturn) void __longjmp_noframe(_JUMP_BUFFER* _Buf, int _Value);
//
    LEAF_ENTRY __longjmp_noframe

    /* Restore callee-saved NEON registers D8-D15 */
    ldp d8, d9, [x0, #JbD8]
    ldp d10, d11, [x0, #(JbD8 + 16)]
    ldp d12, d13, [x0, #(JbD8 + 32)]
    ldp d14, d15, [x0, #(JbD8 + 48)]

    /* Restore floating point control and status registers */
    ldr w2, [x0, #JbFpcr]
    msr fpcr, x2
    ldr w2, [x0, #JbFpsr]
    msr fpsr, x2

    /* Restore callee-saved registers X19-X28 */
    ldp x19, x20, [x0, #JbX19]
    ldp x21, x22, [x0, #JbX21]
    ldp x23, x24, [x0, #JbX23]
    ldp x25, x26, [x0, #JbX25]
    ldp x27, x28, [x0, #JbX27]

    /* Restore FP (x29), LR (x30), and SP */
    ldr x29, [x0, #JbFp]
    ldr x30, [x0, #JbLr]
    ldr x2, [x0, #JbSp]
    mov sp, x2

    /* Return _Value or 1 if _Value is 0 */
    cmp x1, #0
    csinc x0, x1, xzr, ne

    ret

    LEAF_END __longjmp_noframe

    END

/* EOF */
