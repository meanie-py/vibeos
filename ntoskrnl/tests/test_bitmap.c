/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — RTL Bitmap Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests RtlInitializeBitMap / RtlSetBit / RtlClearBit / RtlSetBits /
 * RtlClearBits / RtlFindSetBits / RtlFindClearBits /
 * RtlNumberOfSetBits / RtlNumberOfClearBits.
 * Pure in-memory operations, no kernel infrastructure dependencies.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestBitmap - Exercise RTL bitmap operations.
 *
 * Test 1: Initialize + set/clear individual bits + count.
 * Test 2: Set/clear ranges + find operations.
 * Test 3: Boundary conditions (bit 0, last bit).
 */
BOOLEAN
NTAPI
NtTestBitmap(VOID)
{
    RTL_BITMAP bitmap;
    ULONG buffer[8];  /* 256 bits */
    ULONG count;
    ULONG index;

    /* --- Test 1: Basic set/clear/count --- */
    RtlInitializeBitMap(&bitmap, buffer, 256);

    /* Clear all bits */
    RtlClearAllBits(&bitmap);

    if (RtlNumberOfSetBits(&bitmap) != 0)
    {
        DbgPrint("  FAIL: set bits after clear-all = %lu, expected 0\n",
                 RtlNumberOfSetBits(&bitmap));
        return FALSE;
    }

    if (RtlNumberOfClearBits(&bitmap) != 256)
    {
        DbgPrint("  FAIL: clear bits after clear-all = %lu, expected 256\n",
                 RtlNumberOfClearBits(&bitmap));
        return FALSE;
    }

    /* Set individual bits */
    RtlSetBit(&bitmap, 0);
    RtlSetBit(&bitmap, 7);
    RtlSetBit(&bitmap, 31);
    RtlSetBit(&bitmap, 255);

    if (RtlNumberOfSetBits(&bitmap) != 4)
    {
        DbgPrint("  FAIL: set bits = %lu, expected 4\n",
                 RtlNumberOfSetBits(&bitmap));
        return FALSE;
    }

    /* Test individual bits */
    if (!RtlTestBit(&bitmap, 0))
    {
        DbgPrint("  FAIL: bit 0 should be set\n");
        return FALSE;
    }

    if (!RtlTestBit(&bitmap, 255))
    {
        DbgPrint("  FAIL: bit 255 should be set\n");
        return FALSE;
    }

    if (RtlTestBit(&bitmap, 1))
    {
        DbgPrint("  FAIL: bit 1 should be clear\n");
        return FALSE;
    }

    /* Clear a set bit */
    RtlClearBit(&bitmap, 7);
    if (RtlTestBit(&bitmap, 7))
    {
        DbgPrint("  FAIL: bit 7 should be clear after RtlClearBit\n");
        return FALSE;
    }

    if (RtlNumberOfSetBits(&bitmap) != 3)
    {
        DbgPrint("  FAIL: set bits after clearing bit 7 = %lu, expected 3\n",
                 RtlNumberOfSetBits(&bitmap));
        return FALSE;
    }

    /* --- Test 2: Range operations --- */
    RtlClearAllBits(&bitmap);

    /* Set bits 10-19 (10 bits) */
    RtlSetBits(&bitmap, 10, 10);

    count = RtlNumberOfSetBits(&bitmap);
    if (count != 10)
    {
        DbgPrint("  FAIL: set bits after range set = %lu, expected 10\n", count);
        return FALSE;
    }

    /* Verify range boundaries */
    if (RtlTestBit(&bitmap, 9))
    {
        DbgPrint("  FAIL: bit 9 should be clear (before range)\n");
        return FALSE;
    }

    if (!RtlTestBit(&bitmap, 10))
    {
        DbgPrint("  FAIL: bit 10 should be set (range start)\n");
        return FALSE;
    }

    if (!RtlTestBit(&bitmap, 19))
    {
        DbgPrint("  FAIL: bit 19 should be set (range end)\n");
        return FALSE;
    }

    if (RtlTestBit(&bitmap, 20))
    {
        DbgPrint("  FAIL: bit 20 should be clear (after range)\n");
        return FALSE;
    }

    /* Find clear bits */
    index = RtlFindClearBits(&bitmap, 5, 0);
    if (index == 0xFFFFFFFF)
    {
        DbgPrint("  FAIL: RtlFindClearBits(5) returned -1\n");
        return FALSE;
    }

    /* Result should be in range 0-9 or 20+ (not overlapping set bits) */
    if (index >= 10 && index < 20)
    {
        DbgPrint("  FAIL: RtlFindClearBits returned %lu (overlaps set range)\n", index);
        return FALSE;
    }

    /* Find set bits */
    index = RtlFindSetBits(&bitmap, 5, 0);
    if (index == 0xFFFFFFFF)
    {
        DbgPrint("  FAIL: RtlFindSetBits(5) returned -1\n");
        return FALSE;
    }

    if (index < 10 || index > 15)
    {
        DbgPrint("  FAIL: RtlFindSetBits returned %lu, expected 10-15\n", index);
        return FALSE;
    }

    /* Clear the range */
    RtlClearBits(&bitmap, 10, 10);
    if (RtlNumberOfSetBits(&bitmap) != 0)
    {
        DbgPrint("  FAIL: set bits after range clear = %lu, expected 0\n",
                 RtlNumberOfSetBits(&bitmap));
        return FALSE;
    }

    /* --- Test 3: Set all + verify --- */
    RtlSetAllBits(&bitmap);

    if (RtlNumberOfSetBits(&bitmap) != 256)
    {
        DbgPrint("  FAIL: set bits after set-all = %lu, expected 256\n",
                 RtlNumberOfSetBits(&bitmap));
        return FALSE;
    }

    if (RtlNumberOfClearBits(&bitmap) != 0)
    {
        DbgPrint("  FAIL: clear bits after set-all = %lu, expected 0\n",
                 RtlNumberOfClearBits(&bitmap));
        return FALSE;
    }

    /* Find clear bits should fail (none available) */
    index = RtlFindClearBits(&bitmap, 1, 0);
    if (index != 0xFFFFFFFF)
    {
        DbgPrint("  FAIL: RtlFindClearBits on full bitmap returned %lu\n", index);
        return FALSE;
    }

    DbgPrint("  Bitmap: set/clear/test/range/find/count all passed\n");
    return TRUE;
}
