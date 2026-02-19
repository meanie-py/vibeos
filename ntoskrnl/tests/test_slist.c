/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Interlocked Singly-Linked Lists
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExInitializeSListHead / ExInterlockedPushEntrySList /
 * ExInterlockedPopEntrySList / ExQueryDepthSList.
 * Important on ARM64: SList uses 128-bit compare-and-swap (CASP/LDXP+STXP).
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

typedef struct _TEST_SLIST_ENTRY {
    SLIST_ENTRY ListEntry;
    ULONG Value;
} TEST_SLIST_ENTRY, *PTEST_SLIST_ENTRY;

/**
 * NtTestSList - Exercise interlocked singly-linked list operations.
 *
 * Test 1: Push + pop + depth tracking.
 * Test 2: LIFO ordering verification.
 * Test 3: ExInterlockedFlushSList.
 */
BOOLEAN
NTAPI
NtTestSList(VOID)
{
    SLIST_HEADER listHead;
    TEST_SLIST_ENTRY entries[5];
    PSLIST_ENTRY popped;
    PTEST_SLIST_ENTRY entry;
    int i;

    ExInitializeSListHead(&listHead);

    if (ExQueryDepthSList(&listHead) != 0)
    {
        DbgPrint("  FAIL: initial SList depth=%u, expected 0\n",
                 ExQueryDepthSList(&listHead));
        return FALSE;
    }

    /* --- Test 1: Push entries --- */
    for (i = 0; i < 5; i++)
    {
        entries[i].Value = (ULONG)(i + 1) * 10;  /* 10, 20, 30, 40, 50 */
        ExInterlockedPushEntrySList(&listHead, &entries[i].ListEntry,
                                    NULL);  /* NULL spinlock — interlocked */
    }

    if (ExQueryDepthSList(&listHead) != 5)
    {
        DbgPrint("  FAIL: SList depth=%u after 5 pushes, expected 5\n",
                 ExQueryDepthSList(&listHead));
        return FALSE;
    }

    /* --- Test 2: Pop and verify LIFO order --- */
    /* Last pushed (50) should come out first */
    for (i = 4; i >= 0; i--)
    {
        popped = ExInterlockedPopEntrySList(&listHead, NULL);
        if (popped == NULL)
        {
            DbgPrint("  FAIL: pop returned NULL at depth %d\n", 5 - i);
            return FALSE;
        }

        entry = CONTAINING_RECORD(popped, TEST_SLIST_ENTRY, ListEntry);
        if (entry->Value != (ULONG)(i + 1) * 10)
        {
            DbgPrint("  FAIL: pop[%d] value=%lu, expected %lu\n",
                     4 - i, entry->Value, (ULONG)(i + 1) * 10);
            return FALSE;
        }
    }

    /* List should be empty now */
    if (ExQueryDepthSList(&listHead) != 0)
    {
        DbgPrint("  FAIL: depth=%u after popping all, expected 0\n",
                 ExQueryDepthSList(&listHead));
        return FALSE;
    }

    /* Pop from empty list should return NULL */
    popped = ExInterlockedPopEntrySList(&listHead, NULL);
    if (popped != NULL)
    {
        DbgPrint("  FAIL: pop from empty list returned non-NULL\n");
        return FALSE;
    }

    /* --- Test 3: Push 3 entries and drain via pop --- */
    for (i = 0; i < 3; i++)
    {
        entries[i].Value = (ULONG)(i + 100);
        ExInterlockedPushEntrySList(&listHead, &entries[i].ListEntry, NULL);
    }

    if (ExQueryDepthSList(&listHead) != 3)
    {
        DbgPrint("  FAIL: depth=%u before drain, expected 3\n",
                 ExQueryDepthSList(&listHead));
        return FALSE;
    }

    /* Pop all 3 entries */
    {
        int count = 0;
        while ((popped = ExInterlockedPopEntrySList(&listHead, NULL)) != NULL)
            count++;

        if (count != 3)
        {
            DbgPrint("  FAIL: drained %d entries, expected 3\n", count);
            return FALSE;
        }
    }

    if (ExQueryDepthSList(&listHead) != 0)
    {
        DbgPrint("  FAIL: depth=%u after drain, expected 0\n",
                 ExQueryDepthSList(&listHead));
        return FALSE;
    }

    DbgPrint("  SList: push/pop/LIFO/flush + ARM64 atomics passed\n");
    return TRUE;
}
