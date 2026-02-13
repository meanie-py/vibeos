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
    /* Call the APC routine: NormalRoutine(NormalContext, SysArg1, SysArg2) */
    blr x3

    /* After APC routine returns, call NtContinue to resume */
    mov x0, sp          /* PCONTEXT */
    mov w1, #1          /* TestAlert = TRUE */
    bl NtContinue

    /* Should not return; break if it does */
    brk #0xF000

/* EOF */
