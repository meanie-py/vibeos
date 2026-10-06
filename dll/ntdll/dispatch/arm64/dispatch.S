/*
 * PROJECT:         ReactOS NT Library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            dll/ntdll/dispatch/arm64/dispatch.S
 * PURPOSE:         User-Mode Dispatchers for ARM64
 */

.text

/*
 * VOID LdrInitializeThunk(PCONTEXT NormalContext, PVOID SystemArgument1,
 *                         PVOID SystemArgument2, PVOID SystemArgument3)
 * x0 = context, received in x9 per APC calling convention
 */
.globl LdrInitializeThunk
LdrInitializeThunk:
    mov x29, #0
    mov x0, x9          /* Context from APC parameter */
    b LdrpInit

/*
 * VOID KiUserApcDispatcher(PVOID NormalContext, PVOID SystemArgument1,
 *                          PVOID SystemArgument2, PKNORMAL_ROUTINE NormalRoutine)
 *
 * On entry: sp points to a CONTEXT structure, followed by APC parameters.
 * x0 = NormalContext, x1 = SystemArgument1, x2 = SystemArgument2,
 * x3 = NormalRoutine (the APC function to call)
 */
.globl KiUserApcDispatcher
KiUserApcDispatcher:
    /* In an x86-64 guest process wowx64.dll fills WowX64UserApcHook
     * (dispatch/mod.rs): the routine may be guest x86-64 code, which it runs
     * itself and never comes back from. It returns only for a native routine,
     * so the four APC registers are kept across it and the call below is
     * exactly the hookless one. */
    adrp x10, WowX64UserApcHook
    ldr x10, [x10, :lo12:WowX64UserApcHook]
    cbz x10, 1f
    stp x0, x1, [sp, #-32]!
    stp x2, x3, [sp, #16]
    add x4, sp, #32     /* CONTEXT */
    blr x10
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp], #32
1:
    /* Save CONTEXT pointer for NormalRoutine (e.g., LdrInitializeThunk reads x9) */
    mov x9, sp

    /* Call the APC routine: NormalRoutine(NormalContext, SysArg1, SysArg2) */
    blr x3

    /* After APC routine returns, call NtContinue to resume */
    mov x0, sp          /* PCONTEXT */
    mov w1, #1          /* TestAlert = TRUE */
    bl NtContinue

    /* Should not return; break if it does */
    brk #0xF000

/*
 * VOID KiUserExceptionDispatcher(PEXCEPTION_RECORD ExceptionRecord, PCONTEXT Context)
 *
 * On entry from kernel: x0 = ExceptionRecord, x1 = Context, sp = user exception stack.
 * Dispatches exception to user-mode handlers via RtlDispatchException.
 */
.globl KiUserExceptionDispatcher
KiUserExceptionDispatcher:
    stp x29, x30, [sp, #-32]!
    mov x29, sp
    stp x19, x20, [sp, #16]    /* save ExcRec/Ctx across calls */
    mov x19, x0                 /* x19 = ExceptionRecord */
    mov x20, x1                 /* x20 = Context */

    bl RtlDispatchException     /* RtlDispatchException(ExcRec, Ctx) */

    cbz w0, .LRaiseException

    mov x0, x20                 /* Context */
    mov w1, #0                  /* TestAlert = FALSE */
    bl NtContinue
    brk #0xF003                 /* should not return */

.LRaiseException:
#ifdef _USE_DUMMY_PSEH
    /* DUMMY_PSEH swallow — mirrors amd64/dispatch.S: SEH handlers are dead
     * code, so no handler ever catches. Continuable software exceptions
     * (code < 0xC0000000, e.g. DBG_PRINTEXCEPTION_C) are swallowed by
     * resuming, simulating _SEH2_EXCEPT(EXECUTE_HANDLER) fall-through.
     * NT error statuses (hardware faults) go second-chance: resuming them
     * would re-execute the faulting instruction in an infinite loop. */
    ldr w2, [x19, #4]           /* ExceptionRecord->ExceptionFlags */
    tbnz w2, #0, .LSecondChance /* EXCEPTION_NONCONTINUABLE (bit 0) */
    ldr w2, [x19]               /* ExceptionRecord->ExceptionCode */
    mov w3, #0xC0000000
    cmp w2, w3
    b.hs .LSecondChance
    /* STATUS_BREAKPOINT (0x80000003): on arm64 Pc points AT the brk (no
     * x86-style Rip-1 rewind) — step over it (+4) or resume re-executes it. */
    movz w3, #0x8000, lsl #16
    movk w3, #0x0003
    cmp w2, w3
    b.ne .LNoBrkAdjust
    ldr x2, [x20, #0x108]       /* Context->Pc (X0..X28+Fp+Lr end at 0x100) */
    add x2, x2, #4
    str x2, [x20, #0x108]
.LNoBrkAdjust:
    mov x0, x20                 /* Context */
    mov w1, #0                  /* TestAlert = FALSE */
    bl NtContinue
    brk #0xF003                 /* should not return */

.LSecondChance:
#endif
    mov x0, x19                 /* ExceptionRecord */
    mov x1, x20                 /* Context */
    mov w2, #0                  /* FirstChance = FALSE */
    bl NtRaiseException
    brk #0xF003                 /* should not return */

/* EOF */
