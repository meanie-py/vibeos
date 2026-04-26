/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Memory Operations (RtlCopy/Fill/Zero/Compare)
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Validates ARM64 memory intrinsics, alignment handling, and bulk
 * operations. Critical for correct data handling across the kernel.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

BOOLEAN
NTAPI
NtTestMemOps(VOID)
{
    UCHAR srcBuf[256];
    UCHAR dstBuf[256];
    ULONG i;
    SIZE_T result;

    /* --- Test 1: RtlFillMemory + RtlZeroMemory --- */
    RtlFillMemory(srcBuf, sizeof(srcBuf), 0xAB);
    for (i = 0; i < 256; i++)
    {
        if (srcBuf[i] != 0xAB)
        {
            DbgPrint("  FAIL: RtlFillMemory srcBuf[%lu] = 0x%02x, expected 0xAB\n",
                     i, srcBuf[i]);
            return FALSE;
        }
    }

    RtlZeroMemory(dstBuf, sizeof(dstBuf));
    for (i = 0; i < 256; i++)
    {
        if (dstBuf[i] != 0)
        {
            DbgPrint("  FAIL: RtlZeroMemory dstBuf[%lu] = 0x%02x, expected 0\n",
                     i, dstBuf[i]);
            return FALSE;
        }
    }

    /* --- Test 2: RtlCopyMemory — aligned --- */
    for (i = 0; i < 256; i++)
        srcBuf[i] = (UCHAR)i;

    RtlCopyMemory(dstBuf, srcBuf, 256);
    for (i = 0; i < 256; i++)
    {
        if (dstBuf[i] != (UCHAR)i)
        {
            DbgPrint("  FAIL: RtlCopyMemory dstBuf[%lu] = 0x%02x, expected 0x%02x\n",
                     i, dstBuf[i], (UCHAR)i);
            return FALSE;
        }
    }

    /* --- Test 3: RtlCopyMemory — misaligned (offset by 1) --- */
    RtlZeroMemory(dstBuf, sizeof(dstBuf));
    RtlCopyMemory(dstBuf + 1, srcBuf + 3, 100);
    for (i = 0; i < 100; i++)
    {
        if (dstBuf[i + 1] != srcBuf[i + 3])
        {
            DbgPrint("  FAIL: misaligned copy dstBuf[%lu] = 0x%02x, expected 0x%02x\n",
                     i + 1, dstBuf[i + 1], srcBuf[i + 3]);
            return FALSE;
        }
    }
    /* Check byte before and after are untouched */
    if (dstBuf[0] != 0)
    {
        DbgPrint("  FAIL: byte before misaligned copy was overwritten\n");
        return FALSE;
    }
    if (dstBuf[101] != 0)
    {
        DbgPrint("  FAIL: byte after misaligned copy was overwritten\n");
        return FALSE;
    }

    /* --- Test 4: RtlCompareMemory --- */
    RtlFillMemory(srcBuf, 128, 0x55);
    RtlFillMemory(dstBuf, 128, 0x55);

    result = RtlCompareMemory(srcBuf, dstBuf, 128);
    if (result != 128)
    {
        DbgPrint("  FAIL: RtlCompareMemory identical = %lu, expected 128\n",
                 (ULONG)result);
        return FALSE;
    }

    /* Differ at byte 64 */
    dstBuf[64] = 0xAA;
    result = RtlCompareMemory(srcBuf, dstBuf, 128);
    if (result != 64)
    {
        DbgPrint("  FAIL: RtlCompareMemory differ-at-64 = %lu, expected 64\n",
                 (ULONG)result);
        return FALSE;
    }

    /* --- Test 5: RtlMoveMemory — overlapping (forward overlap) --- */
    for (i = 0; i < 256; i++)
        srcBuf[i] = (UCHAR)i;

    /* Move bytes 0-99 to offset 50 (overlapping) */
    RtlMoveMemory(srcBuf + 50, srcBuf, 100);
    for (i = 0; i < 100; i++)
    {
        if (srcBuf[i + 50] != (UCHAR)i)
        {
            DbgPrint("  FAIL: RtlMoveMemory overlap srcBuf[%lu] = 0x%02x, expected 0x%02x\n",
                     i + 50, srcBuf[i + 50], (UCHAR)i);
            return FALSE;
        }
    }

    /* --- Test 6: RtlFillMemoryUlong --- */
    {
        ULONG ulongBuf[32];
        RtlFillMemoryUlong(ulongBuf, sizeof(ulongBuf), 0xDEADBEEF);
        for (i = 0; i < 32; i++)
        {
            if (ulongBuf[i] != 0xDEADBEEF)
            {
                DbgPrint("  FAIL: RtlFillMemoryUlong[%lu] = 0x%08lx, expected 0xDEADBEEF\n",
                         i, ulongBuf[i]);
                return FALSE;
            }
        }
    }

    /* --- Test 7: Large pool copy (4KB) --- */
    {
        PUCHAR bigSrc, bigDst;
        bigSrc = ExAllocatePoolWithTag(NonPagedPool, 4096, 'TSEM');
        bigDst = ExAllocatePoolWithTag(NonPagedPool, 4096, 'TSEM');
        if (!bigSrc || !bigDst)
        {
            DbgPrint("  FAIL: pool alloc for large copy failed\n");
            if (bigSrc) ExFreePoolWithTag(bigSrc, 'TSEM');
            if (bigDst) ExFreePoolWithTag(bigDst, 'TSEM');
            return FALSE;
        }

        for (i = 0; i < 4096; i++)
            bigSrc[i] = (UCHAR)(i & 0xFF);

        RtlCopyMemory(bigDst, bigSrc, 4096);

        result = RtlCompareMemory(bigSrc, bigDst, 4096);
        if (result != 4096)
        {
            DbgPrint("  FAIL: 4KB copy differs at byte %lu\n", (ULONG)result);
            ExFreePoolWithTag(bigSrc, 'TSEM');
            ExFreePoolWithTag(bigDst, 'TSEM');
            return FALSE;
        }

        ExFreePoolWithTag(bigSrc, 'TSEM');
        ExFreePoolWithTag(bigDst, 'TSEM');
    }

    DbgPrint("  MemOps: fill/zero/copy/compare/move/ulong/large all passed\n");
    return TRUE;
}
