/*
 * PROJECT:     ReactOS Setup Library - runtime probe
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Drive setuplib's pure and read-only entry points and print what they
 *              return, one fact per line. Built twice: setuplib_probe links the shipped
 *              Rust (setuplib_static), setuplib_probe_c the pre-flip C (c_oracle.cmake),
 *              and tools/payloads/modrun diffs the two outputs in the guest. Nothing here
 *              writes a disk: partitions are created and deleted in the in-memory list.
 *              Print values, never addresses -- a pointer differs between the two runs.
 */

#include "precomp.h"
#include "setuplib.h"

static void
ws(PCWSTR s)
{
    if (!s)
    {
        printf("(null)");
        return;
    }
    for (; *s; ++s)
        putchar(*s < 0x80 && *s >= 0x20 ? (char)*s : '?');
}

static void
probe_install_dirs(void)
{
    static const PCWSTR dirs[] = {
        L"", L" ", L"\\", L"\\\\", L".", L"..", L"...", L"....", L" WINNT", L"WINNT ",
        L".WINNT", L"..WINNT", L"W.INNT", L"WI.NNT", L"WIN.NT", L"WINNT.", L"WINNT..",
        L"WINNT. ", L"WINNT.e e", L"WIN^`NT", L"WINNT", L"\\WINNT", L"\\WINNT\\",
        L"\\WINNT\\.", L"WIN\\NT", L"WIN\\NT.", L"\\WIN\\NT\\", L"\\WIN\\NT.\\",
        L"\\WIN.\\NT52", L"\\WIN\\.\\NT52", L"\\WIN\\..\\NT52", L"\\WIN\\...\\NT52",
        L"\\WIN\\....\\NT52", L"win.nt.5", L"win.ntX5", L"winn.tX5", L"winnt.X5",
        L"winntX.5", L"winntX5.", L"filenamee.xte", L"filenamee.xt", L"filename.ext",
        L"file.ame.ext", L"filenam.eext", L"1 3 5 7 .abc", L"12345678.  c",
        L"123456789.a", L"12345.abcd", L"12345.ab d", L".abc", L"12.abc.d",
        L"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    };
    ULONG i;
    for (i = 0; i < ARRAYSIZE(dirs); ++i)
    {
        printf("installdir \"");
        ws(dirs[i]);
        printf("\" %d\n", IsValidInstallDirectory(dirs[i]));
    }
}

static void
probe_arc(PPARTLIST List)
{
    static const PCWSTR paths[] = {
        L"multi(5)disk()rdisk(1)partition()\\ReactOS", L"multi(0)disk(0)rdisk(0)partition(1)",
        L"multi(0)disk(0)rdisk(0)partition(1)\\ReactOS\\system32", L"ramdisk(0)",
        L"ramdisk(0)\\ReactOS\\system32\\ntoskrnl.exe", L"net(0)\\Foobar",
        L"net(0)disk(1)\\Foobar", L"scsi(2)disk(1)rdisk(3)", L"scsi(2)disk(1)fdisk(3)",
        L"multi(0)disk(0)fdisk(0)", L"multi(0)disk(0)cdrom(0)", L"eisa(0)disk(0)rdisk(0)partition(2)",
        L"signature(12345678)disk(0)rdisk(0)partition(1)", L"bogus", L"",
    };
    WCHAR norm[MAX_PATH], nt[MAX_PATH];
    UNICODE_STRING Norm, Nt;
    ULONG i;
    for (i = 0; i < ARRAYSIZE(paths); ++i)
    {
        BOOLEAN ok;
        RtlInitEmptyUnicodeString(&Norm, norm, sizeof(norm));
        ok = ArcPathNormalize(&Norm, paths[i]);
        printf("arcnorm \"");
        ws(paths[i]);
        printf("\" %d \"%.*ls\"\n", ok, ok ? Norm.Length / 2 : 0, norm);
        RtlInitEmptyUnicodeString(&Nt, nt, sizeof(nt));
        ok = ArcPathToNtPath(&Nt, paths[i], NULL);
        printf("arc2nt \"");
        ws(paths[i]);
        printf("\" %d \"%.*ls\"\n", ok, ok ? Nt.Length / 2 : 0, nt);
        if (List)
        {
            RtlInitEmptyUnicodeString(&Nt, nt, sizeof(nt));
            ok = ArcPathToNtPath(&Nt, paths[i], List);
            printf("arc2nt+list \"");
            ws(paths[i]);
            printf("\" %d \"%.*ls\"\n", ok, ok ? Nt.Length / 2 : 0, nt);
        }
    }
}

static void
probe_paths(void)
{
    WCHAR buf[64];
    NTSTATUS st;

    wcscpy(buf, L"\\a");
    st = ConcatPaths(buf, ARRAYSIZE(buf), 2, L"b\\", L"\\c");
    printf("concat 0x%08lx \"%ls\"\n", st, buf);
    wcscpy(buf, L"");
    st = ConcatPaths(buf, ARRAYSIZE(buf), 1, L"x");
    printf("concat 0x%08lx \"%ls\"\n", st, buf);
    wcscpy(buf, L"C:\\");
    st = ConcatPaths(buf, ARRAYSIZE(buf), 3, L"\\ReactOS\\", L"", L"system32");
    printf("concat 0x%08lx \"%ls\"\n", st, buf);
    wcscpy(buf, L"0123456789");
    st = ConcatPaths(buf, 12, 1, L"abcdef");
    printf("concat-small 0x%08lx \"%ls\"\n", st, buf);
    st = CombinePaths(buf, ARRAYSIZE(buf), 3, L"\\Device\\Harddisk0", L"Partition1\\", L"\\ReactOS");
    printf("combine 0x%08lx \"%ls\"\n", st, buf);
    st = CombinePaths(buf, ARRAYSIZE(buf), 0);
    printf("combine0 0x%08lx \"%ls\"\n", st, buf);
    printf("findsub \"%ls\"\n", FindSubStrI(L"Hello ReactOS World", L"reactos") ?: L"(null)");
    printf("findsub \"%ls\"\n", FindSubStrI(L"Hello", L"xyz") ?: L"(null)");
    printf("findsub \"%ls\"\n", FindSubStrI(L"abc", L"") ?: L"(null)");
}

static void
probe_math(void)
{
    static const ULONGLONG v[] = { 0, 1, 62, 63, 64, 2047, 2048, 2049, 0x7fffffffULL, 0x123456789ULL };
    static const ULONG a[] = { 1, 63, 2048, 4096 };
    ULONG i, j;
    for (i = 0; i < ARRAYSIZE(v); ++i)
        for (j = 0; j < ARRAYSIZE(a); ++j)
            printf("align %I64u %lu down=%I64u up=%I64u div=%I64u\n", v[i], a[j], AlignDown(v[i], a[j]),
                   AlignUp(v[i], a[j]), RoundingDivide(v[i], a[j]));
}

static void
probe_types(void)
{
    ULONG t;
    for (t = 0; t < 256; ++t)
    {
        UCHAR type = (UCHAR)t;
        PCSTR s = LookupPartitionTypeString(PARTITION_STYLE_MBR, &type);
        if (s)
            printf("ptype 0x%02lx %s\n", t, s);
    }
}

static void
probe_misc(void)
{
    ULONG i;
    PCWSTR name;
    for (i = 0; GetRegisteredFileSystems(i, &name); ++i)
        printf("fs %lu \"%ls\"\n", i, name);
    printf("mui kbd 0409 0x%08lx\n", MUIDefaultKeyboardLayout(L"00000409"));
    printf("mui kbd 0405 0x%08lx\n", MUIDefaultKeyboardLayout(L"00000405"));
    printf("mui oem 0409 %u\n", MUIGetOEMCodePage(L"00000409"));
    printf("mui oem 0419 %u\n", MUIGetOEMCodePage(L"00000419"));
    printf("mui oem bogus %u\n", MUIGetOEMCodePage(L"0000ffff"));
}

static void
dump_part(PCSTR tag, PPARTENTRY p)
{
    printf("%s part start=%I64u count=%I64u type=0x%02x boot=%d ondisk=%lu num=%lu idx=%lu "
           "logical=%d partitioned=%d new=%d dev=\"%ls\"",
           tag, p->StartSector.QuadPart, p->SectorCount.QuadPart, p->PartitionType, p->BootIndicator,
           p->OnDiskPartitionNumber, p->PartitionNumber, p->PartitionIndex, p->LogicalPartition,
           p->IsPartitioned, p->New, p->DeviceName);
    if (p->Volume)
        printf(" vol=\"%ls\" letter=%c label=\"%ls\" fs=\"%ls\" state=%d check=%d volnew=%d back=%d",
               p->Volume->Info.DeviceName, p->Volume->Info.DriveLetter ? (char)p->Volume->Info.DriveLetter : '-',
               p->Volume->Info.VolumeLabel, p->Volume->Info.FileSystem, p->Volume->FormatState,
               p->Volume->NeedsCheck, p->Volume->New, p->Volume->PartEntry == p);
    printf(" active=%d\n", IsPartitionActive(p));
}

static void
dump_disk(PDISKENTRY d)
{
    PLIST_ENTRY e;
    printf("disk num=%lu media=%d cyl=%I64u tpc=%lu spt=%lu bps=%lu sectors=%I64u salign=%lu calign=%lu "
           "bios=%d hw=%lu/%lu/%lu/%lu port=%u bus=%u id=%u dirty=%d new=%d style=%d driver=\"%.*ls\" "
           "superfloppy=%d ext=%d\n",
           d->DiskNumber, d->MediaType, d->Cylinders, d->TracksPerCylinder, d->SectorsPerTrack,
           d->BytesPerSector, d->SectorCount.QuadPart, d->SectorAlignment, d->CylinderAlignment, d->BiosFound,
           d->HwAdapterNumber, d->HwControllerNumber, d->HwDiskNumber, d->HwFixedDiskNumber, d->Port, d->Bus,
           d->Id, d->Dirty, d->NewDisk, d->DiskStyle, d->DriverName.Length / 2,
           d->DriverName.Buffer ? d->DriverName.Buffer : L"",
           d->DiskStyle == PARTITION_STYLE_MBR ? IsSuperFloppy(d) : -1, d->ExtendedPartition != NULL);
    for (e = d->PrimaryPartListHead.Flink; e != &d->PrimaryPartListHead; e = e->Flink)
        dump_part(" primary", CONTAINING_RECORD(e, PARTENTRY, ListEntry));
    for (e = d->LogicalPartListHead.Flink; e != &d->LogicalPartListHead; e = e->Flink)
        dump_part(" logical", CONTAINING_RECORD(e, PARTENTRY, ListEntry));
}

static void
dump_list(PCSTR when, PPARTLIST List)
{
    PLIST_ENTRY e;
    PPARTENTRY p;
    ULONG n = 0;
    printf("list %s\n", when);
    for (e = List->DiskListHead.Flink; e != &List->DiskListHead; e = e->Flink)
        dump_disk(CONTAINING_RECORD(e, DISKENTRY, ListEntry));
    for (e = List->BiosDiskListHead.Flink; e != &List->BiosDiskListHead; e = e->Flink)
    {
        PBIOSDISKENTRY b = CONTAINING_RECORD(e, BIOSDISKENTRY, ListEntry);
        printf("biosdisk %lu/%lu/%lu sig=0x%08lx sum=0x%08lx recognized=%d\n", b->AdapterNumber,
               b->ControllerNumber, b->DiskNumber, b->Signature, b->Checksum, b->DiskEntry != NULL);
    }
    for (p = GetNextPartition(List, NULL); p && n < 64; p = GetNextPartition(List, p), ++n)
        printf("next disk=%lu start=%I64u\n", p->DiskEntry->DiskNumber, p->StartSector.QuadPart);
    for (p = GetPrevPartition(List, NULL), n = 0; p && n < 64; p = GetPrevPartition(List, p), ++n)
        printf("prev disk=%lu start=%I64u\n", p->DiskEntry->DiskNumber, p->StartSector.QuadPart);
    p = List->SystemPartition;
    printf("system %s\n", p ? "set" : "none");
}

static void
probe_partlist(void)
{
    PPARTLIST List = CreatePartitionList();
    PLIST_ENTRY e;
    PDISKENTRY blank = NULL;
    PPARTENTRY p, freed;
    PGENERIC_LIST os;

    if (!List)
    {
        printf("partlist NULL\n");
        return;
    }
    dump_list("initial", List);
    probe_arc(List);
    p = FindSupportedSystemPartition(List, FALSE, NULL, NULL);
    printf("sysfind %s\n", p ? "found" : "none");
    if (p)
        dump_part(" sysfind", p);
    os = CreateNTOSInstallationsList(List);
    printf("ntos %lu\n", os ? GetNumberOfListEntries(os) : 0xFFFFFFFF);
    if (os)
        DestroyGenericList(os, TRUE);

    /* An empty disk -- the harness's blank scratch disk -- gets partitions in memory only. */
    for (e = List->DiskListHead.Flink; e != &List->DiskListHead; e = e->Flink)
    {
        PDISKENTRY d = CONTAINING_RECORD(e, DISKENTRY, ListEntry);
        PLIST_ENTRY f = d->PrimaryPartListHead.Flink;
        if (f != &d->PrimaryPartListHead && f->Flink == &d->PrimaryPartListHead &&
            !CONTAINING_RECORD(f, PARTENTRY, ListEntry)->IsPartitioned)
        {
            blank = d;
            break;
        }
    }
    printf("blank %s\n", blank ? "found" : "none");
    if (blank)
    {
        ULONGLONG mb = 1024 * 1024;
        p = CONTAINING_RECORD(blank->PrimaryPartListHead.Flink, PARTENTRY, ListEntry);
        printf("checks 0=%d 8M=%d huge=%d ext=%d\n", PartitionCreateChecks(p, 0, 0),
               PartitionCreateChecks(p, 8 * mb, 0), PartitionCreateChecks(p, 1ULL << 40, 0),
               PartitionCreateChecks(p, 8 * mb, PARTITION_EXTENDED));
        printf("create 16M %d\n", CreatePartition(List, p, 16 * mb, 0));
        dump_disk(blank);
        p = GetAdjUnpartitionedEntry(p, TRUE);
        printf("adj %s\n", p ? "found" : "none");
        if (p)
        {
            printf("create ext %d\n", CreatePartition(List, p, 24 * mb, PARTITION_EXTENDED));
            dump_disk(blank);
        }
        if (blank->ExtendedPartition)
        {
            PLIST_ENTRY l = blank->LogicalPartListHead.Flink;
            if (l != &blank->LogicalPartListHead)
            {
                p = CONTAINING_RECORD(l, PARTENTRY, ListEntry);
                printf("create logical %d\n", CreatePartition(List, p, 8 * mb, 0));
                dump_disk(blank);
            }
        }
        p = CONTAINING_RECORD(blank->PrimaryPartListHead.Flink, PARTENTRY, ListEntry);
        printf("setactive %d\n", SetActivePartition(List, p, NULL));
        freed = NULL;
        printf("delete %d freed=%s\n", DeletePartition(List, p, &freed), freed ? "yes" : "no");
        dump_disk(blank);
    }
    dump_list("final", List);
    DestroyPartitionList(List);
    printf("partlist destroyed\n");
}

int
wmain(void)
{
    ProcessHeap = RtlGetProcessHeap();
    probe_install_dirs();
    probe_paths();
    probe_math();
    probe_types();
    probe_misc();
    probe_arc(NULL);
    probe_partlist();
    printf("probe done\n");
    /* vfatlib's vendored dosfstools common.c defines its own exit() -- a bare
     * NtTerminateProcess, for the native setup programs -- and this image links it, so
     * the CRT startup's exit() never reaches msvcrt and nothing flushes stdio. */
    fflush(stdout);
    return 0;
}
