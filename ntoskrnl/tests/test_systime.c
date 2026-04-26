/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — System Time and Tick Count
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeQuerySystemTime, KeQueryTickCount, and ExGetPreviousMode.
 * Validates ARM64 time infrastructure is functional.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

BOOLEAN
NTAPI
NtTestSysTime(VOID)
{
    LARGE_INTEGER time1, time2;
    LARGE_INTEGER ticks1, ticks2;

    /* --- Test 1: KeQuerySystemTime returns non-zero --- */
    KeQuerySystemTime(&time1);

    if (time1.QuadPart == 0)
    {
        DbgPrint("  FAIL: KeQuerySystemTime returned 0\n");
        return FALSE;
    }

    /* Windows epoch: Jan 1, 1601. Year 2000 is roughly 125,911 days * 864e9 100ns.
     * Any reasonable value should be > 100,000,000,000,000,000 (1e17) */
    if (time1.QuadPart < 100000000000000000LL)
    {
        DbgPrint("  INFO: system time %lld seems too small (pre-2000?)\n",
                 time1.QuadPart);
        /* Non-fatal — QEMU may not set a proper RTC */
    }

    /* --- Test 2: Two calls should be monotonic --- */
    KeQuerySystemTime(&time2);

    if (time2.QuadPart < time1.QuadPart)
    {
        DbgPrint("  FAIL: system time went backwards: %lld -> %lld\n",
                 time1.QuadPart, time2.QuadPart);
        return FALSE;
    }

    /* --- Test 3: KeQueryTickCount returns non-negative --- */
    KeQueryTickCount(&ticks1);

    /* Tick count should be >= 0 (could be 0 if timer hasn't fired) */
    if (ticks1.QuadPart < 0)
    {
        DbgPrint("  FAIL: tick count is negative: %lld\n", ticks1.QuadPart);
        return FALSE;
    }

    /* --- Test 4: Two tick count calls should be monotonic --- */
    KeQueryTickCount(&ticks2);

    if (ticks2.QuadPart < ticks1.QuadPart)
    {
        DbgPrint("  FAIL: tick count went backwards: %lld -> %lld\n",
                 ticks1.QuadPart, ticks2.QuadPart);
        return FALSE;
    }

    /* --- Test 5: ExGetPreviousMode should be KernelMode --- */
    {
        KPROCESSOR_MODE mode = ExGetPreviousMode();
        if (mode != KernelMode)
        {
            DbgPrint("  FAIL: ExGetPreviousMode = %d, expected KernelMode (0)\n",
                     (int)mode);
            return FALSE;
        }
    }

    /* --- Test 6: KeQueryTimeIncrement --- */
    {
        ULONG increment = KeQueryTimeIncrement();
        if (increment == 0)
        {
            DbgPrint("  FAIL: KeQueryTimeIncrement returned 0\n");
            return FALSE;
        }
        DbgPrint("  SysTime: time=%lld, ticks=%lld, increment=%lu\n",
                 time1.QuadPart, ticks1.QuadPart, increment);
    }

    DbgPrint("  SysTime: query + monotonic + mode + increment passed\n");
    return TRUE;
}
