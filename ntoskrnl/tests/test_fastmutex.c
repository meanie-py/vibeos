/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Fast Mutex and Executive Resources
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExInitializeFastMutex / ExAcquireFastMutex / ExReleaseFastMutex
 * and ExInitializeResourceLite / ExAcquireResourceExclusiveLite /
 * ExAcquireResourceSharedLite / ExReleaseResourceLite.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestFastMutex - Exercise fast mutex and executive resource locks.
 *
 * Test 1: Fast mutex acquire/release, verify IRQL behavior.
 * Test 2: ERESOURCE exclusive acquire/release.
 * Test 3: ERESOURCE shared acquire (multiple readers).
 * Test 4: ExTryToAcquireFastMutex.
 */
BOOLEAN
NTAPI
NtTestFastMutex(VOID)
{
    FAST_MUTEX fmutex;
    ERESOURCE resource;
    NTSTATUS status;

    /* --- Test 1: Fast Mutex --- */
    ExInitializeFastMutex(&fmutex);

    /* Acquire — IRQL should be raised to APC_LEVEL */
    ExAcquireFastMutex(&fmutex);

    if (KeGetCurrentIrql() != APC_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u while holding fast mutex, expected APC_LEVEL(%u)\n",
                 KeGetCurrentIrql(), APC_LEVEL);
        ExReleaseFastMutex(&fmutex);
        return FALSE;
    }

    ExReleaseFastMutex(&fmutex);

    if (KeGetCurrentIrql() != PASSIVE_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u after releasing fast mutex, expected PASSIVE\n",
                 KeGetCurrentIrql());
        return FALSE;
    }

    /* --- Test 4: TryToAcquire --- */
    if (!ExTryToAcquireFastMutex(&fmutex))
    {
        DbgPrint("  FAIL: ExTryToAcquireFastMutex returned FALSE on free mutex\n");
        return FALSE;
    }

    /* Mutex is now held — release it */
    ExReleaseFastMutex(&fmutex);

    /* --- Test 2: ERESOURCE exclusive --- */
    status = ExInitializeResourceLite(&resource);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ExInitializeResourceLite returned 0x%08lx\n", status);
        return FALSE;
    }

    KeEnterCriticalRegion();
    if (!ExAcquireResourceExclusiveLite(&resource, TRUE))
    {
        DbgPrint("  FAIL: ExAcquireResourceExclusiveLite returned FALSE\n");
        KeLeaveCriticalRegion();
        ExDeleteResourceLite(&resource);
        return FALSE;
    }

    /* Should be exclusively owned */
    if (!ExIsResourceAcquiredExclusiveLite(&resource))
    {
        DbgPrint("  FAIL: resource not exclusively owned after acquire\n");
        ExReleaseResourceLite(&resource);
        KeLeaveCriticalRegion();
        ExDeleteResourceLite(&resource);
        return FALSE;
    }

    ExReleaseResourceLite(&resource);
    KeLeaveCriticalRegion();

    /* --- Test 3: ERESOURCE shared (multiple readers) --- */
    KeEnterCriticalRegion();

    if (!ExAcquireResourceSharedLite(&resource, TRUE))
    {
        DbgPrint("  FAIL: first ExAcquireResourceSharedLite returned FALSE\n");
        KeLeaveCriticalRegion();
        ExDeleteResourceLite(&resource);
        return FALSE;
    }

    /* Acquire shared again (recursive shared should work on same thread) */
    if (!ExAcquireResourceSharedLite(&resource, TRUE))
    {
        DbgPrint("  FAIL: second shared acquire returned FALSE\n");
        ExReleaseResourceLite(&resource);
        KeLeaveCriticalRegion();
        ExDeleteResourceLite(&resource);
        return FALSE;
    }

    /* Should have 2 shared acquisitions */
    if (ExIsResourceAcquiredSharedLite(&resource) < 2)
    {
        DbgPrint("  FAIL: shared count=%lu, expected >= 2\n",
                 ExIsResourceAcquiredSharedLite(&resource));
    }

    ExReleaseResourceLite(&resource);
    ExReleaseResourceLite(&resource);
    KeLeaveCriticalRegion();

    ExDeleteResourceLite(&resource);

    DbgPrint("  FastMutex: fast mutex + try-acquire + ERESOURCE excl/shared passed\n");
    return TRUE;
}
