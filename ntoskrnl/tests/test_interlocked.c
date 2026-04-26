/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Interlocked Doubly-Linked List Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExInterlockedInsertHeadList / ExInterlockedInsertTailList /
 * ExInterlockedRemoveHeadList with spinlock synchronization.
 * Validates ARM64 interlocked RMW (read-modify-write) operations.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

typedef struct _TEST_LIST_ENTRY {
    LIST_ENTRY ListEntry;
    ULONG Value;
} TEST_LIST_ENTRY, *PTEST_LIST_ENTRY;

BOOLEAN
NTAPI
NtTestInterlockedList(VOID)
{
    LIST_ENTRY listHead;
    KSPIN_LOCK lock;
    TEST_LIST_ENTRY entries[5];
    PLIST_ENTRY removed;
    PTEST_LIST_ENTRY entry;
    int i;

    KeInitializeSpinLock(&lock);
    InitializeListHead(&listHead);

    /* --- Test 1: Insert at head + remove --- */
    for (i = 0; i < 5; i++)
    {
        entries[i].Value = (ULONG)(i + 1) * 100;  /* 100, 200, 300, 400, 500 */
        ExInterlockedInsertHeadList(&listHead, &entries[i].ListEntry, &lock);
    }

    /* Verify list is not empty */
    if (IsListEmpty(&listHead))
    {
        DbgPrint("  FAIL: list empty after 5 InsertHead\n");
        return FALSE;
    }

    /* Remove from head — should be LIFO (last inserted = 500 first) */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    if (removed == NULL)
    {
        DbgPrint("  FAIL: RemoveHead returned NULL\n");
        return FALSE;
    }
    entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
    if (entry->Value != 500)
    {
        DbgPrint("  FAIL: first RemoveHead value=%lu, expected 500\n",
                 entry->Value);
        return FALSE;
    }

    /* Remove next — should be 400 */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    if (removed == NULL)
    {
        DbgPrint("  FAIL: second RemoveHead returned NULL\n");
        return FALSE;
    }
    entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
    if (entry->Value != 400)
    {
        DbgPrint("  FAIL: second RemoveHead value=%lu, expected 400\n",
                 entry->Value);
        return FALSE;
    }

    /* Remove remaining 3 */
    for (i = 0; i < 3; i++)
    {
        removed = ExInterlockedRemoveHeadList(&listHead, &lock);
        if (removed == NULL)
        {
            DbgPrint("  FAIL: RemoveHead[%d] returned NULL\n", i + 2);
            return FALSE;
        }
    }

    /* List should be empty now */
    if (!IsListEmpty(&listHead))
    {
        DbgPrint("  FAIL: list not empty after removing all entries\n");
        return FALSE;
    }

    /* Remove from empty list should return NULL */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    if (removed != NULL)
    {
        DbgPrint("  FAIL: RemoveHead from empty list returned non-NULL\n");
        return FALSE;
    }

    /* --- Test 2: Insert at tail --- */
    InitializeListHead(&listHead);
    for (i = 0; i < 3; i++)
    {
        entries[i].Value = (ULONG)(i + 1) * 10;  /* 10, 20, 30 */
        ExInterlockedInsertTailList(&listHead, &entries[i].ListEntry, &lock);
    }

    /* Remove from head — should be FIFO (first inserted = 10 first) */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    if (removed == NULL)
    {
        DbgPrint("  FAIL: tail-insert RemoveHead returned NULL\n");
        return FALSE;
    }
    entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
    if (entry->Value != 10)
    {
        DbgPrint("  FAIL: tail-insert RemoveHead value=%lu, expected 10 (FIFO)\n",
                 entry->Value);
        return FALSE;
    }

    /* Next should be 20 */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
    if (entry->Value != 20)
    {
        DbgPrint("  FAIL: second tail-insert RemoveHead value=%lu, expected 20\n",
                 entry->Value);
        return FALSE;
    }

    /* Last should be 30 */
    removed = ExInterlockedRemoveHeadList(&listHead, &lock);
    entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
    if (entry->Value != 30)
    {
        DbgPrint("  FAIL: third tail-insert RemoveHead value=%lu, expected 30\n",
                 entry->Value);
        return FALSE;
    }

    /* --- Test 3: Mixed head and tail inserts --- */
    InitializeListHead(&listHead);
    entries[0].Value = 1;
    entries[1].Value = 2;
    entries[2].Value = 3;
    entries[3].Value = 4;

    ExInterlockedInsertTailList(&listHead, &entries[0].ListEntry, &lock);  /* [1] */
    ExInterlockedInsertHeadList(&listHead, &entries[1].ListEntry, &lock);  /* [2, 1] */
    ExInterlockedInsertTailList(&listHead, &entries[2].ListEntry, &lock);  /* [2, 1, 3] */
    ExInterlockedInsertHeadList(&listHead, &entries[3].ListEntry, &lock);  /* [4, 2, 1, 3] */

    /* Expected removal order: 4, 2, 1, 3 */
    {
        ULONG expected[] = {4, 2, 1, 3};
        for (i = 0; i < 4; i++)
        {
            removed = ExInterlockedRemoveHeadList(&listHead, &lock);
            if (removed == NULL)
            {
                DbgPrint("  FAIL: mixed RemoveHead[%d] returned NULL\n", i);
                return FALSE;
            }
            entry = CONTAINING_RECORD(removed, TEST_LIST_ENTRY, ListEntry);
            if (entry->Value != expected[i])
            {
                DbgPrint("  FAIL: mixed RemoveHead[%d] value=%lu, expected %lu\n",
                         i, entry->Value, expected[i]);
                return FALSE;
            }
        }
    }

    DbgPrint("  InterlockedList: head/tail/mixed insert + remove passed\n");
    return TRUE;
}
