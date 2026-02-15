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
    mov x0, x19                 /* ExceptionRecord */
    mov x1, x20                 /* Context */
    mov w2, #0                  /* FirstChance = FALSE */
    bl NtRaiseException
    brk #0xF003                 /* should not return */

/* EOF */
