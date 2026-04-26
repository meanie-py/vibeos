/*
 * PROJECT:     ReactOS vcruntime library
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     Implementation of _setjmp for ARM64
 * COPYRIGHT:   Copyright Timo Kreuzer <timo.kreuzer@reactos.org>
 *              Copyright 2025 ReactOS ARM64 Port
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
// int _setjmpex(_JUMP_BUFFER* _Buf, void* _Frame);
//
    LEAF_ENTRY _setjmpex
    ALTERNATE_ENTRY __intrinsic_setjmpex
    ALTERNATE_ENTRY __intrinsic_setjmp
    ALTERNATE_ENTRY setjmp
    /* __wine_setjmpex is called by msvcrt's MSVCRT__setjmp trampoline */
    ALTERNATE_ENTRY __wine_setjmpex

    /*
     * _setjmp is provided as a weak symbol so that msvcrt's MSVCRT__setjmp
     * (from except_arm64.c) can override it when both are linked together
     * (e.g. in ucrtbase.dll). When only the setjmp library is linked
     * (e.g. in ftfd.dll), the weak definition is used.
     */
    .weak _setjmp
_setjmp:

    /* Save Frame (passed in x1) */
    str x1, [x0, #JbFrame]
    str xzr, [x0, #JbReserved]

    /* Save callee-saved registers X19-X28 */
    stp x19, x20, [x0, #JbX19]
    stp x21, x22, [x0, #JbX21]
    stp x23, x24, [x0, #JbX23]
    stp x25, x26, [x0, #JbX25]
    stp x27, x28, [x0, #JbX27]

    /* Save FP (x29), LR (x30), and SP */
    str x29, [x0, #JbFp]
    str x30, [x0, #JbLr]
    mov x2, sp
    str x2, [x0, #JbSp]

    /* Save floating point control and status registers */
    mrs x2, fpcr
    str w2, [x0, #JbFpcr]
    mrs x2, fpsr
    str w2, [x0, #JbFpsr]

    /* Save callee-saved NEON registers D8-D15 */
    stp d8, d9, [x0, #JbD8]
    stp d10, d11, [x0, #(JbD8 + 16)]
    stp d12, d13, [x0, #(JbD8 + 32)]
    stp d14, d15, [x0, #(JbD8 + 48)]

    /* Return 0 */
    mov x0, #0
    ret

    LEAF_END _setjmpex

    END

/* EOF */
