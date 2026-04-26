/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Lookaside Lists
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExInitializeNPagedLookasideList / ExAllocateFromNPagedLookasideList /
 * ExFreeToNPagedLookasideList / ExDeleteNPagedLookasideList.
 * Also tests paged lookaside equivalents.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

#define LOOK_TAG 'kooL'

/**
 * NtTestLookaside - Exercise lookaside list allocation.
 *
 * Test 1: NonPaged lookaside — alloc, use, free, re-alloc (tests cache).
 * Test 2: Paged lookaside — same pattern.
 * Test 3: Multiple allocs then frees (tests depth management).
 */
BOOLEAN
NTAPI
NtTestLookaside(VOID)
{
    NPAGED_LOOKASIDE_LIST npLookaside;
    PAGED_LOOKASIDE_LIST pLookaside;
    PVOID blocks[10];
    int i;

    /* --- Test 1: NonPaged lookaside list --- */
    ExInitializeNPagedLookasideList(&npLookaside, NULL, NULL, 0,
                                    64, LOOK_TAG, 0);

    /* Allocate a block */
    {
        PVOID block = ExAllocateFromNPagedLookasideList(&npLookaside);
        if (block == NULL)
        {
            DbgPrint("  FAIL: ExAllocateFromNPagedLookasideList returned NULL\n");
            ExDeleteNPagedLookasideList(&npLookaside);
            return FALSE;
        }

        /* Write to verify it's usable */
        RtlFillMemory(block, 64, 0xCC);

        /* Free it back to lookaside */
        ExFreeToNPagedLookasideList(&npLookaside, block);

        /* Re-allocate — should come from the cache (same block or new) */
        {
            PVOID block2 = ExAllocateFromNPagedLookasideList(&npLookaside);
            if (block2 == NULL)
            {
                DbgPrint("  FAIL: re-alloc from NP lookaside returned NULL\n");
                ExDeleteNPagedLookasideList(&npLookaside);
                return FALSE;
            }
            ExFreeToNPagedLookasideList(&npLookaside, block2);
        }
    }

    /* --- Test 3a: Multiple allocs then frees (NonPaged) --- */
    for (i = 0; i < 10; i++)
    {
        blocks[i] = ExAllocateFromNPagedLookasideList(&npLookaside);
        if (blocks[i] == NULL)
        {
            DbgPrint("  FAIL: NP lookaside alloc[%d] returned NULL\n", i);
            while (--i >= 0)
                ExFreeToNPagedLookasideList(&npLookaside, blocks[i]);
            ExDeleteNPagedLookasideList(&npLookaside);
            return FALSE;
        }
        /* Write unique pattern */
        RtlFillMemory(blocks[i], 64, (UCHAR)(i + 1));
    }

    /* Verify patterns before freeing */
    for (i = 0; i < 10; i++)
    {
        PUCHAR p = (PUCHAR)blocks[i];
        if (p[0] != (UCHAR)(i + 1) || p[63] != (UCHAR)(i + 1))
        {
            DbgPrint("  FAIL: NP block[%d] pattern corrupted\n", i);
            for (i = 0; i < 10; i++)
                ExFreeToNPagedLookasideList(&npLookaside, blocks[i]);
            ExDeleteNPagedLookasideList(&npLookaside);
            return FALSE;
        }
    }

    for (i = 0; i < 10; i++)
        ExFreeToNPagedLookasideList(&npLookaside, blocks[i]);

    ExDeleteNPagedLookasideList(&npLookaside);

    /* --- Test 2: Paged lookaside list --- */
    ExInitializePagedLookasideList(&pLookaside, NULL, NULL, 0,
                                   128, LOOK_TAG, 0);

    {
        PVOID block = ExAllocateFromPagedLookasideList(&pLookaside);
        if (block == NULL)
        {
            DbgPrint("  FAIL: ExAllocateFromPagedLookasideList returned NULL\n");
            ExDeletePagedLookasideList(&pLookaside);
            return FALSE;
        }

        RtlFillMemory(block, 128, 0xDD);
        ExFreeToPagedLookasideList(&pLookaside, block);
    }

    ExDeletePagedLookasideList(&pLookaside);

    DbgPrint("  Lookaside: NonPaged + Paged + multi-alloc passed\n");
    return TRUE;
}
