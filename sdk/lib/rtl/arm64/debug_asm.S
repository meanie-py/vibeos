/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            lib/rtl/arm64/debug_asm.S
 * PURPOSE:         Cross-privilege Debugging Support for ARM64
 */

.text

/* ULONG NTAPI DebugService(ULONG Service, PVOID Arg1, PVOID Arg2, PVOID Arg3, PVOID Arg4)
 *
 * brk #0xF002 is Windows ARM64's debug-service immediate; #0xF000 is a real breakpoint
 * (DbgBreakPoint). They used to share #0xF000 and the trap handler told them apart by
 * x0 alone, so the kernel could never dispatch a breakpoint as STATUS_BREAKPOINT. */
.globl DebugService
DebugService:
    brk #0xF002
    mov x0, #0
    ret

/* VOID NTAPI DebugService2(PVOID Arg1, PVOID Arg2, ULONG Service) */
.globl DebugService2
DebugService2:
    brk #0xF002
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

/*
 * VOID RtlpRestoreContextInternal(PCONTEXT ContextRecord)
 *
 * Same-mode register-reload + branch — the arm64 analogue of the amd64
 * RtlpRestoreContextInternal (sdk/lib/rtl/amd64/except_asm.S). It resumes
 * execution at ContextRecord->Pc with the recorded SP and integer registers,
 * WITHOUT a syscall / EL transition. This is what RtlRestoreContext (and thus
 * RtlUnwindEx -> __except landing) needs to work in KERNEL mode: the old arm64
 * RtlRestoreContext resumed via NtContinue, which is a user-return path and
 * cannot correctly resume a kernel context mid-callstack (task #10 / §9.69 —
 * every kernel-mode __try/__except silently failed to catch).
 *
 * CONTEXT layout mirrors RtlCaptureContext above:
 *   +0x04 Cpsr (NZCV)   +0x08 X0 .. +0xF8 X30   +0x100 Sp   +0x108 Pc
 * x16/x17 are the ABI intra-procedure scratch regs; we use them to stage Pc/Sp
 * and do not restore them (they never carry live values across a call, matching
 * the real ntdll implementation).
 */
.globl RtlpRestoreContextInternal
RtlpRestoreContextInternal:
    /* Restore condition flags (NZCV). */
    ldr w1, [x0, #0x04]
    msr nzcv, x1

    /* Stage Pc and Sp in the scratch registers. */
    ldr x16, [x0, #0x108]   /* Pc -> branch target */
    ldr x17, [x0, #0x100]   /* Sp */

    /* Restore x2..x15, x18..x30 (skip x16/x17 scratch), then x1, x0 last. */
    ldp x2,  x3,  [x0, #0x18]
    ldp x4,  x5,  [x0, #0x28]
    ldp x6,  x7,  [x0, #0x38]
    ldp x8,  x9,  [x0, #0x48]
    ldp x10, x11, [x0, #0x58]
    ldp x12, x13, [x0, #0x68]
    ldp x14, x15, [x0, #0x78]
    ldp x18, x19, [x0, #0x98]
    ldp x20, x21, [x0, #0xA8]
    ldp x22, x23, [x0, #0xB8]
    ldp x24, x25, [x0, #0xC8]
    ldp x26, x27, [x0, #0xD8]
    ldp x28, x29, [x0, #0xE8]
    ldr x30,      [x0, #0xF8]
    ldr x1,       [x0, #0x10]
    ldr x0,       [x0, #0x08]

    /* Install the target SP and branch to the target PC. */
    mov sp, x17
    br  x16

/* EOF */
