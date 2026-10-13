/*
 * PROJECT:     vibeos kernel-mode tests
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     Windows 10 kernel surface with no 2003 counterpart: ExAllocatePool2/3 and
 *              ExFreePool2 (ExPool2); ETW provider registration and feature configuration
 *              (EtwProvider, EtwActivity, FeatureConfig). Graded on Windows 11 first (`vibeos-cli winref kmtest`): a
 *              test Windows fails is a wrong test.
 *
 * Every function is looked up with MmGetSystemRoutineAddress, never imported: a static
 * import of an export one kernel lacks keeps the whole of kmtest_drv.sys from loading
 * (the KeRaiseIrqlToSynchLevel lesson, 2026-10-12), where a lookup fails one assertion.
 */

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

/* WDK wdm.h POOL_FLAG_* (learn.microsoft.com/windows-hardware/drivers/kernel/pool_flags) */
#define T_POOL_FLAG_UNINITIALIZED       0x0000000000000002ULL
#define T_POOL_FLAG_CACHE_ALIGNED       0x0000000000000008ULL
#define T_POOL_FLAG_NON_PAGED           0x0000000000000040ULL
#define T_POOL_FLAG_PAGED               0x0000000000000100ULL
#define T_POOL_FLAG_OPTIONAL_START      0x0000000100000000ULL

/* POOL_EXTENDED_PARAMETER: { ULONG64 Type:8, Optional:1, Reserved:55; 8-byte union } */
typedef struct _T_POOL_EXTENDED_PARAMETER
{
    ULONG64 Head;
    ULONG64 Value;
} T_POOL_EXTENDED_PARAMETER;
#define T_POOL_EXT_PRIORITY  1
#define T_POOL_EXT_OPTIONAL  0x100

typedef PVOID (NTAPI *PFN_ALLOC2)(ULONG64, SIZE_T, ULONG);
typedef PVOID (NTAPI *PFN_ALLOC3)(ULONG64, SIZE_T, ULONG, const T_POOL_EXTENDED_PARAMETER *, ULONG);
typedef VOID (NTAPI *PFN_FREE2)(PVOID, ULONG, const T_POOL_EXTENDED_PARAMETER *, ULONG);

#define TAG_P2 'tP2E'

static PVOID Lookup(PCWSTR Name)
{
    UNICODE_STRING String;
    RtlInitUnicodeString(&String, Name);
    return MmGetSystemRoutineAddress(&String);
}

static BOOLEAN AllZero(const UCHAR *Buffer, SIZE_T Length)
{
    SIZE_T i;
    for (i = 0; i < Length; i++)
        if (Buffer[i])
            return FALSE;
    return TRUE;
}

START_TEST(ExPool2)
{
    PFN_ALLOC2 Alloc2 = (PFN_ALLOC2)Lookup(L"ExAllocatePool2");
    PFN_ALLOC3 Alloc3 = (PFN_ALLOC3)Lookup(L"ExAllocatePool3");
    PFN_FREE2 Free2 = (PFN_FREE2)Lookup(L"ExFreePool2");
    T_POOL_EXTENDED_PARAMETER Param;
    PUCHAR Block;
    ULONG i;

    ok(Alloc2 != NULL, "ExAllocatePool2 is not exported\n");
    ok(Alloc3 != NULL, "ExAllocatePool3 is not exported\n");
    ok(Free2 != NULL, "ExFreePool2 is not exported\n");
    if (!Alloc2 || !Alloc3 || !Free2)
        return;

    /* Zeroed unless POOL_FLAG_UNINITIALIZED. Dirty a block and free it first, so the
     * zeroed allocation that follows is likely to reuse those bytes; repeat to make a
     * missing memset hard to miss. */
    for (i = 0; i < 8; i++)
    {
        Block = Alloc2(T_POOL_FLAG_NON_PAGED | T_POOL_FLAG_UNINITIALIZED, 200, TAG_P2);
        ok(Block != NULL, "uninitialized non-paged allocation failed\n");
        if (Block)
        {
            RtlFillMemory(Block, 200, 0xCC);
            ExFreePoolWithTag(Block, TAG_P2);
        }
        Block = Alloc2(T_POOL_FLAG_NON_PAGED, 200, TAG_P2);
        ok(Block != NULL, "non-paged allocation failed\n");
        if (Block)
        {
            ok(AllZero(Block, 200), "non-paged block %lu is not zeroed\n", i);
            ExFreePoolWithTag(Block, TAG_P2);
        }
    }

    Block = Alloc2(T_POOL_FLAG_PAGED, 64, TAG_P2);
    ok(Block != NULL, "paged allocation failed\n");
    if (Block)
    {
        ok(AllZero(Block, 64), "paged block is not zeroed\n");
        Free2(Block, TAG_P2, NULL, 0);
    }

    /* "If NumberOfBytes is PAGE_SIZE or greater, a page-aligned buffer is allocated." */
    Block = Alloc2(T_POOL_FLAG_NON_PAGED, 2 * PAGE_SIZE, TAG_P2);
    ok(Block != NULL, "two-page allocation failed\n");
    if (Block)
    {
        ok(((ULONG_PTR)Block & (PAGE_SIZE - 1)) == 0, "two-page block %p is not page aligned\n", Block);
        ok(AllZero(Block, 2 * PAGE_SIZE), "two-page block is not zeroed\n");
        ExFreePoolWithTag(Block, TAG_P2);
    }

    Block = Alloc2(T_POOL_FLAG_NON_PAGED | T_POOL_FLAG_CACHE_ALIGNED, 32, TAG_P2);
    ok(Block != NULL, "cache-aligned allocation failed\n");
    if (Block)
        ExFreePoolWithTag(Block, TAG_P2);

    /* Documented NULL returns: tag 0, invalid flags. */
    Block = Alloc2(T_POOL_FLAG_NON_PAGED, 32, 0);
    ok(Block == NULL, "tag 0 was accepted\n");
    if (Block) ExFreePool(Block);
    Block = Alloc2(0, 32, TAG_P2);
    ok(Block == NULL, "no pool type was accepted\n");
    if (Block) ExFreePool(Block);
    Block = Alloc2(T_POOL_FLAG_NON_PAGED | T_POOL_FLAG_PAGED, 32, TAG_P2);
    ok(Block == NULL, "two pool types were accepted\n");
    if (Block) ExFreePool(Block);
    /* Not tested: the reserved REQUIRED bits. Windows 11 accepts POOL_FLAG_RESERVED2, and
     * probing the others bugchecked it -- they are reserved, not invalid input. */

    /* An unknown OPTIONAL flag may be ignored. */
    Block = Alloc2(T_POOL_FLAG_NON_PAGED | (T_POOL_FLAG_OPTIONAL_START << 8), 32, TAG_P2);
    ok(Block != NULL, "an unknown optional flag failed the allocation\n");
    if (Block) ExFreePoolWithTag(Block, TAG_P2);

    /* ExAllocatePool3: no parameters is ExAllocatePool2. */
    Block = Alloc3(T_POOL_FLAG_NON_PAGED, 48, TAG_P2, NULL, 0);
    ok(Block != NULL, "ExAllocatePool3 without parameters failed\n");
    if (Block)
    {
        ok(AllZero(Block, 48), "ExAllocatePool3 block is not zeroed\n");
        Free2(Block, TAG_P2, NULL, 0);
    }

    Param.Head = T_POOL_EXT_PRIORITY;
    Param.Value = NormalPoolPriority;
    Block = Alloc3(T_POOL_FLAG_NON_PAGED, 48, TAG_P2, &Param, 1);
    ok(Block != NULL, "ExAllocatePool3 with a priority failed\n");
    if (Block) Free2(Block, TAG_P2, NULL, 0);

    /* An unrecognized type fails the call unless it is marked Optional. */
    Param.Head = 0x7F;
    Param.Value = 0;
    Block = Alloc3(T_POOL_FLAG_NON_PAGED, 48, TAG_P2, &Param, 1);
    ok(Block == NULL, "ExAllocatePool3 accepted an unknown required parameter\n");
    if (Block) ExFreePool(Block);
    Param.Head = 0x7F | T_POOL_EXT_OPTIONAL;
    Block = Alloc3(T_POOL_FLAG_NON_PAGED, 48, TAG_P2, &Param, 1);
    ok(Block != NULL, "ExAllocatePool3 failed on an unknown optional parameter\n");
    if (Block) Free2(Block, TAG_P2, NULL, 0);
}

/* evntprov.h's EVENT_DESCRIPTOR, by layout: 16 bytes. */
typedef struct _T_EVENT_DESCRIPTOR
{
    USHORT Id;
    UCHAR Version, Channel, Level, Opcode;
    USHORT Task;
    ULONGLONG Keyword;
} T_EVENT_DESCRIPTOR;

typedef NTSTATUS (NTAPI *PFN_ETWREGISTER)(LPCGUID, PVOID, PVOID, PULONGLONG);
typedef NTSTATUS (NTAPI *PFN_ETWUNREGISTER)(ULONGLONG);
typedef BOOLEAN (NTAPI *PFN_ETWEVENTENABLED)(ULONGLONG, const T_EVENT_DESCRIPTOR *);
typedef BOOLEAN (NTAPI *PFN_ETWPROVIDERENABLED)(ULONGLONG, UCHAR, ULONGLONG);
typedef NTSTATUS (NTAPI *PFN_ETWWRITE)(ULONGLONG, const T_EVENT_DESCRIPTOR *, LPCGUID, ULONG, PVOID);
typedef NTSTATUS (NTAPI *PFN_ETWACTIVITY)(ULONG, LPGUID);
typedef NTSTATUS (NTAPI *PFN_FEATUREQUERY)(ULONG, ULONG, PULONGLONG, PVOID);

#define T_EVENT_ACTIVITY_CTRL_CREATE_ID 3

START_TEST(EtwProvider)
{
    /* A provider no trace session has ever enabled: {6f0e9d28-0b77-4c3a-9d5a-7669626f7301} */
    static const GUID Provider =
        { 0x6f0e9d28, 0x0b77, 0x4c3a, { 0x9d, 0x5a, 0x76, 0x69, 0x62, 0x6f, 0x73, 0x01 } };
    PFN_ETWREGISTER Register = (PFN_ETWREGISTER)Lookup(L"EtwRegister");
    PFN_ETWUNREGISTER Unregister = (PFN_ETWUNREGISTER)Lookup(L"EtwUnregister");
    PFN_ETWEVENTENABLED EventEnabled = (PFN_ETWEVENTENABLED)Lookup(L"EtwEventEnabled");
    PFN_ETWPROVIDERENABLED ProviderEnabled = (PFN_ETWPROVIDERENABLED)Lookup(L"EtwProviderEnabled");
    PFN_ETWWRITE Write = (PFN_ETWWRITE)Lookup(L"EtwWrite");
    T_EVENT_DESCRIPTOR Descriptor = { 1, 0, 0, 4, 0, 0, 0 };
    ULONGLONG Handle = 0;
    NTSTATUS Status;

    ok(Register && Unregister && EventEnabled && ProviderEnabled && Write,
       "ETW provider API missing: %p %p %p %p %p\n",
       Register, Unregister, EventEnabled, ProviderEnabled, Write);

    if (Register && Unregister && EventEnabled && ProviderEnabled && Write)
    {
        Status = Register(&Provider, NULL, NULL, &Handle);
        ok_eq_hex(Status, STATUS_SUCCESS);
        ok(Handle != 0, "EtwRegister returned a zero handle\n");
        if (NT_SUCCESS(Status))
        {
            ok(!EventEnabled(Handle, &Descriptor), "an event of an unlistened provider is enabled\n");
            ok(!ProviderEnabled(Handle, 4, 0), "an unlistened provider is enabled\n");
            /* Each call into a variable first: ok_eq_hex evaluates its argument twice,
             * and a second EtwUnregister of one handle bugchecks Windows 11 (0x139,
             * list-entry corruption) -- which is how this test found it. */
            Status = Write(Handle, &Descriptor, NULL, 0, NULL);
            ok_eq_hex(Status, STATUS_SUCCESS);
            Status = Unregister(Handle);
            ok_eq_hex(Status, STATUS_SUCCESS);
        }
    }

}

START_TEST(EtwActivity)
{
    PFN_ETWACTIVITY Activity = (PFN_ETWACTIVITY)Lookup(L"EtwActivityIdControl");
    GUID Id1, Id2;
    NTSTATUS Status;

    ok(Activity != NULL, "EtwActivityIdControl is not exported\n");
    if (Activity)
    {
        RtlZeroMemory(&Id1, sizeof(Id1));
        RtlZeroMemory(&Id2, sizeof(Id2));
        Status = Activity(T_EVENT_ACTIVITY_CTRL_CREATE_ID, &Id1);
        ok_eq_hex(Status, STATUS_SUCCESS);
        Status = Activity(T_EVENT_ACTIVITY_CTRL_CREATE_ID, &Id2);
        ok_eq_hex(Status, STATUS_SUCCESS);
        ok(!IsEqualGUID(&Id1, &Id2), "two created activity ids are equal\n");
        RtlZeroMemory(&Id2, sizeof(Id2));
        ok(!IsEqualGUID(&Id1, &Id2), "a created activity id is the null GUID\n");
    }

}

START_TEST(FeatureConfig)
{
    PFN_FEATUREQUERY FeatureQuery = (PFN_FEATUREQUERY)Lookup(L"RtlQueryFeatureConfiguration");
    ULONGLONG Stamp = ~0ULL;
    UCHAR Configuration[64];
    NTSTATUS Status;

    ok(FeatureQuery != NULL, "RtlQueryFeatureConfiguration is not exported\n");
    /* A feature id nothing stages has no runtime configuration. */
    if (FeatureQuery)
    {
        Status = FeatureQuery(0x7FFFFF01, 1, &Stamp, Configuration);
        ok_eq_hex(Status, STATUS_NOT_FOUND);
    }
}
