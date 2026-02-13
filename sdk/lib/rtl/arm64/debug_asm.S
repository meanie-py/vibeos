/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            lib/rtl/arm64/debug_asm.S
 * PURPOSE:         Cross-privilege Debugging Support for ARM64
 */

.text

/* ULONG NTAPI DebugService(ULONG Service, PVOID Arg1, PVOID Arg2, PVOID Arg3, PVOID Arg4) */
.globl DebugService
DebugService:
    brk #0xF000
    mov x0, #0
    ret

/* VOID NTAPI DebugService2(PVOID Arg1, PVOID Arg2, ULONG Service) */
.globl DebugService2
DebugService2:
    brk #0xF000
    ret

/* VOID DbgBreakPoint(VOID) */
.globl DbgBreakPoint
DbgBreakPoint:
    brk #0xF000
    ret

/* VOID DbgUserBreakPoint(VOID) */
.globl DbgUserBreakPoint
DbgUserBreakPoint:
    brk #0xF000
    ret

/* VOID DbgBreakPointWithStatus(ULONG Status) */
.globl DbgBreakPointWithStatus
DbgBreakPointWithStatus:
    brk #0xF000
    ret

/* VOID RtlpBreakWithStatusInstruction(VOID) */
.globl RtlpBreakWithStatusInstruction
RtlpBreakWithStatusInstruction:
    brk #0xF001
    ret

/* VOID RtlCaptureContext(PCONTEXT ContextRecord) */
.globl RtlCaptureContext
RtlCaptureContext:
    /* Save general purpose registers */
    stp x0, x1, [x0, #0x8]
    stp x2, x3, [x0, #0x18]
    stp x4, x5, [x0, #0x28]
    stp x6, x7, [x0, #0x38]
    stp x8, x9, [x0, #0x48]
    stp x10, x11, [x0, #0x58]
    stp x12, x13, [x0, #0x68]
    stp x14, x15, [x0, #0x78]
    stp x16, x17, [x0, #0x88]
    stp x18, x19, [x0, #0x98]
    stp x20, x21, [x0, #0xA8]
    stp x22, x23, [x0, #0xB8]
    stp x24, x25, [x0, #0xC8]
    stp x26, x27, [x0, #0xD8]
    stp x28, x29, [x0, #0xE8]
    str x30, [x0, #0xF8]

    /* Save SP and PC (return address) */
    mov x1, sp
    str x1, [x0, #0x100]
    str x30, [x0, #0x108]

    /* Set context flags: CONTEXT_ARM64 | CONTEXT_CONTROL | CONTEXT_INTEGER */
    movz w1, #0x0040, lsl #16
    movk w1, #0x0003
    str w1, [x0]

    /* Save NZCV flags as CPSR */
    mrs x1, nzcv
    str w1, [x0, #0x4]

    ret

/* VOID RtlGetCallersAddress(PVOID *CallersAddress, PVOID *CallersCaller) */
.globl RtlGetCallersAddress
RtlGetCallersAddress:
    str x30, [x0]
    str xzr, [x1]
    ret

/* EOF */
