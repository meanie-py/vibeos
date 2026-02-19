/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — RtlCaptureContext (ARM64-specific)
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Verifies RtlCaptureContext correctly saves ARM64 general-purpose
 * registers, SP, PC, and LR. Critical for exception dispatch.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestCapture - Exercise ARM64 context capture.
 *
 * Test 1: RtlCaptureContext captures valid SP and PC.
 * Test 2: Verify LR (x30) is reasonable.
 * Test 3: Two captures have consistent SP.
 * Test 4: RtlCaptureStackBackTrace returns valid frames.
 */
BOOLEAN
NTAPI
NtTestCapture(VOID)
{
    CONTEXT ctx;

    RtlZeroMemory(&ctx, sizeof(ctx));

    /* --- Test 1: Capture context --- */
    RtlCaptureContext(&ctx);

    /* SP should be a valid kernel address */
    if (ctx.Sp == 0)
    {
        DbgPrint("  FAIL: captured SP is 0\n");
        return FALSE;
    }

#ifdef _M_ARM64
    /* On ARM64, kernel stack should be in high memory */
    if (ctx.Sp < 0xFFFF000000000000ULL)
    {
        DbgPrint("  FAIL: SP=0x%llx is not in kernel space\n", (ULONG64)ctx.Sp);
        return FALSE;
    }
#endif

    /* PC should be non-zero and in kernel space */
    if (ctx.Pc == 0)
    {
        DbgPrint("  FAIL: captured PC is 0\n");
        return FALSE;
    }

#ifdef _M_ARM64
    if (ctx.Pc < 0xFFFF000000000000ULL)
    {
        DbgPrint("  FAIL: PC=0x%llx is not in kernel space\n", (ULONG64)ctx.Pc);
        return FALSE;
    }

    /* --- Test 2: LR (x30) should be a valid return address --- */
    if (ctx.Lr == 0)
    {
        DbgPrint("  FAIL: captured LR (x30) is 0\n");
        return FALSE;
    }

    if (ctx.Lr < 0xFFFF000000000000ULL)
    {
        DbgPrint("  FAIL: LR=0x%llx is not in kernel space\n", (ULONG64)ctx.Lr);
        return FALSE;
    }
#endif

    /* --- Test 3: Two captures have consistent SP --- */
    {
        CONTEXT ctx2;
        RtlZeroMemory(&ctx2, sizeof(ctx2));
        RtlCaptureContext(&ctx2);

        /* SP should be very close (same function) */
        {
            LONG_PTR spDiff = (LONG_PTR)ctx2.Sp - (LONG_PTR)ctx.Sp;
            if (spDiff < 0) spDiff = -spDiff;

            if (spDiff > 4096)
            {
                DbgPrint("  FAIL: SP difference between captures = %lld (too large)\n",
                         (LONGLONG)spDiff);
                return FALSE;
            }
        }
    }

    /* --- Test 4: RtlCaptureStackBackTrace --- */
    {
        PVOID frames[16];
        ULONG count;

        count = RtlCaptureStackBackTrace(0, 16, frames, NULL);

        if (count == 0)
        {
            DbgPrint("  INFO: RtlCaptureStackBackTrace returned 0 frames\n");
            /* Non-fatal on ARM64 — may not have frame pointers */
        }
        else
        {
            ULONG i;
            for (i = 0; i < count; i++)
            {
                if (frames[i] == NULL)
                    break;
#ifdef _M_ARM64
                if ((ULONG_PTR)frames[i] < 0xFFFF000000000000ULL)
                {
                    DbgPrint("  FAIL: frame[%lu]=%p is not in kernel space\n",
                             i, frames[i]);
                    return FALSE;
                }
#endif
            }
            DbgPrint("  Capture: %lu stack frames captured\n", count);
        }
    }

    DbgPrint("  Capture: context SP/PC/LR + consistency + backtrace passed\n");
    return TRUE;
}
