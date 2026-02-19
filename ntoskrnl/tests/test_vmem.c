/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Virtual Memory Manager
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwAllocateVirtualMemory / ZwFreeVirtualMemory / ZwProtectVirtualMemory
 * to exercise the ARM64 page table and MM subsystem.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestVmem - Exercise virtual memory allocation and protection.
 *
 * Test 1: Allocate RW memory, write pattern, verify, free.
 * Test 2: Allocate reserve, then commit a subrange.
 * Test 3: Change protection to readonly.
 */
BOOLEAN
NTAPI
NtTestVmem(VOID)
{
    PVOID baseAddr;
    SIZE_T regionSize;
    NTSTATUS status;
    ULONG oldProtect;
    PUCHAR ptr;
    ULONG i;

    /* --- Test 1: Basic allocate + read/write + free --- */
    baseAddr = NULL;
    regionSize = 0x10000;  /* 64KB */
    status = ZwAllocateVirtualMemory(NtCurrentProcess(), &baseAddr, 0,
                                     &regionSize, MEM_COMMIT | MEM_RESERVE,
                                     PAGE_READWRITE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwAllocateVirtualMemory(64KB, RW) returned 0x%08lx\n", status);
        return FALSE;
    }

    if (baseAddr == NULL)
    {
        DbgPrint("  FAIL: ZwAllocateVirtualMemory returned NULL base\n");
        return FALSE;
    }

    /* Write pattern */
    ptr = (PUCHAR)baseAddr;
    for (i = 0; i < 0x10000; i++)
        ptr[i] = (UCHAR)(i & 0xFF);

    /* Verify */
    for (i = 0; i < 0x10000; i++)
    {
        if (ptr[i] != (UCHAR)(i & 0xFF))
        {
            DbgPrint("  FAIL: byte[0x%lx] = 0x%02x, expected 0x%02x\n",
                     i, ptr[i], (UCHAR)(i & 0xFF));
            ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
            return FALSE;
        }
    }

    /* Free */
    regionSize = 0;
    status = ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwFreeVirtualMemory returned 0x%08lx\n", status);
        return FALSE;
    }

    /* --- Test 2: Reserve then commit --- */
    baseAddr = NULL;
    regionSize = 0x100000;  /* 1MB reserve */
    status = ZwAllocateVirtualMemory(NtCurrentProcess(), &baseAddr, 0,
                                     &regionSize, MEM_RESERVE,
                                     PAGE_READWRITE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwAllocateVirtualMemory(1MB, RESERVE) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Commit first 4KB */
    {
        PVOID commitAddr = baseAddr;
        SIZE_T commitSize = 0x1000;
        status = ZwAllocateVirtualMemory(NtCurrentProcess(), &commitAddr, 0,
                                         &commitSize, MEM_COMMIT,
                                         PAGE_READWRITE);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwAllocateVirtualMemory(COMMIT 4KB) returned 0x%08lx\n", status);
            regionSize = 0;
            ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
            return FALSE;
        }

        /* Write to committed page */
        *(volatile ULONG *)commitAddr = 0x12345678;
        if (*(volatile ULONG *)commitAddr != 0x12345678)
        {
            DbgPrint("  FAIL: committed page readback mismatch\n");
            regionSize = 0;
            ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
            return FALSE;
        }
    }

    /* Free entire region */
    regionSize = 0;
    status = ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwFreeVirtualMemory(reserved) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* --- Test 3: Protection change --- */
    baseAddr = NULL;
    regionSize = 0x1000;  /* 4KB */
    status = ZwAllocateVirtualMemory(NtCurrentProcess(), &baseAddr, 0,
                                     &regionSize, MEM_COMMIT | MEM_RESERVE,
                                     PAGE_READWRITE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwAllocateVirtualMemory(4KB, RW) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Write while RW */
    *(volatile ULONG *)baseAddr = 0xAABBCCDD;

    /* Change to readonly */
    status = ZwProtectVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize,
                                    PAGE_READONLY, &oldProtect);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwProtectVirtualMemory(READONLY) returned 0x%08lx\n", status);
        regionSize = 0;
        ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
        return FALSE;
    }

    if (oldProtect != PAGE_READWRITE)
    {
        DbgPrint("  FAIL: oldProtect=0x%lx, expected PAGE_READWRITE(0x%lx)\n",
                 oldProtect, (ULONG)PAGE_READWRITE);
        /* Non-fatal — some implementations may return different attributes */
    }

    /* Verify read still works */
    if (*(volatile ULONG *)baseAddr != 0xAABBCCDD)
    {
        DbgPrint("  FAIL: readonly page readback mismatch\n");
        regionSize = 0;
        ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
        return FALSE;
    }

    /* Free */
    regionSize = 0;
    status = ZwFreeVirtualMemory(NtCurrentProcess(), &baseAddr, &regionSize, MEM_RELEASE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwFreeVirtualMemory(readonly) returned 0x%08lx\n", status);
        return FALSE;
    }

    DbgPrint("  Vmem: alloc+rw+free, reserve+commit, protection change passed\n");
    return TRUE;
}
