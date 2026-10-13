/*
 * PROJECT:     vibeos kernel-mode tests
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     MmMapIoSpaceEx, the Windows 10 MmMapIoSpace that takes a page protection
 *              instead of a cache type. Graded on Windows 11 first (`vibeos-cli winref
 *              kmtest`): a test Windows fails is a wrong test.
 *
 * Looked up with MmGetSystemRoutineAddress, never imported (see ExPool2.c). The page it
 * maps is the test's own nonpaged pool page, cached: RAM is the only physical memory
 * every machine this runs on is sure to have, and a non-cached alias of cached RAM is a
 * conflicting-attribute mapping no test should make.
 */

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

typedef PVOID (NTAPI *PFN_MAP_EX)(PHYSICAL_ADDRESS, SIZE_T, ULONG);

#define TAG_MIO 'xEoM'

START_TEST(MmMapIoSpaceEx)
{
    UNICODE_STRING Name;
    PFN_MAP_EX MapEx;
    PULONG Page;
    PULONG View;
    PHYSICAL_ADDRESS Phys;
    ULONG i;
    BOOLEAN Same;

    RtlInitUnicodeString(&Name, L"MmMapIoSpaceEx");
    MapEx = (PFN_MAP_EX)MmGetSystemRoutineAddress(&Name);
    ok(MapEx != NULL, "MmMapIoSpaceEx is not exported\n");
    if (!MapEx)
        return;

    Page = ExAllocatePoolWithTag(NonPagedPool, PAGE_SIZE, TAG_MIO);
    ok(Page != NULL, "no pool page\n");
    if (!Page)
        return;
    for (i = 0; i < PAGE_SIZE / sizeof(ULONG); i++)
        Page[i] = 0x5A000000 | i;
    Phys = MmGetPhysicalAddress(Page);

    /* Read-write, cached: the same bytes, and a store through either address is seen
     * through the other. */
    View = MapEx(Phys, PAGE_SIZE, PAGE_READWRITE);
    ok(View != NULL, "PAGE_READWRITE mapping failed\n");
    if (View)
    {
        ok(View != Page, "the view is the pool address itself\n");
        Same = TRUE;
        for (i = 0; i < PAGE_SIZE / sizeof(ULONG); i++)
            if (View[i] != (0x5A000000 | i))
                Same = FALSE;
        ok(Same, "the view does not hold the page's bytes\n");
        View[7] = 0xC0FFEE;
        ok_eq_hex(Page[7], 0xC0FFEEUL);
        Page[9] = 0xBEEF;
        ok_eq_hex(View[9], 0xBEEFUL);
        MmUnmapIoSpace(View, PAGE_SIZE);
    }

    /* A sub-page range keeps the byte offset into its page. */
    Phys.QuadPart += 0x10;
    View = MapEx(Phys, 0x20, PAGE_READONLY);
    ok(View != NULL, "PAGE_READONLY mapping failed\n");
    if (View)
    {
        ok_eq_hex((ULONG)((ULONG_PTR)View & (PAGE_SIZE - 1)), 0x10UL);
        ok_eq_hex(View[0], 0x5A000004UL);
        MmUnmapIoSpace(View, 0x20);
    }

    ExFreePoolWithTag(Page, TAG_MIO);
}
