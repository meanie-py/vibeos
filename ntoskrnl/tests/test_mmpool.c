/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — memory pool allocation
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExAllocatePoolWithTag / ExFreePoolWithTag for NonPagedPool
 * and PagedPool at various sizes, with fill-pattern verification.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

#define TEST_TAG 'tsTK'  /* KTst */

/**
 * NtTestMmPool - Exercise pool allocator.
 *
 * Allocates blocks of various sizes in both NonPaged and Paged pools,
 * fills with a pattern, verifies, and frees. Returns TRUE if all pass.
 */
BOOLEAN
NTAPI
NtTestMmPool(VOID)
{
    static const ULONG Sizes[] = { 1, 16, 64, 256, 1024, 4096, 65536, 0 };
    POOL_TYPE pools[] = { NonPagedPool, PagedPool };
    const char *poolNames[] = { "NonPagedPool", "PagedPool" };
    ULONG pi, si;

    for (pi = 0; pi < 2; pi++)
    {
        for (si = 0; Sizes[si] != 0; si++)
        {
            ULONG size = Sizes[si];
            PUCHAR block;
            ULONG i;
            UCHAR pattern = (UCHAR)((size + pi) & 0xFF);

            block = (PUCHAR)ExAllocatePoolWithTag(pools[pi], size, TEST_TAG);
            if (block == NULL)
            {
                DbgPrint("  FAIL: ExAllocatePoolWithTag(%s, %lu) returned NULL\n",
                         poolNames[pi], size);
                return FALSE;
            }

            /* Fill with pattern */
            for (i = 0; i < size; i++)
                block[i] = (UCHAR)((pattern + i) & 0xFF);

            /* Verify */
            for (i = 0; i < size; i++)
            {
                if (block[i] != (UCHAR)((pattern + i) & 0xFF))
                {
                    DbgPrint("  FAIL: %s size=%lu byte[%lu] expected 0x%02x got 0x%02x\n",
                             poolNames[pi], size, i,
                             (UCHAR)((pattern + i) & 0xFF), block[i]);
                    ExFreePoolWithTag(block, TEST_TAG);
                    return FALSE;
                }
            }

            ExFreePoolWithTag(block, TEST_TAG);
        }
    }

    DbgPrint("  MmPool: all allocations passed (NonPaged + Paged, 1B-64KB)\n");
    return TRUE;
}
