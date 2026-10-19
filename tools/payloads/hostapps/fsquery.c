/* fsquery — dump what an archiver may ask about its output file and volume, in a form
 * that diffs between vibeos and winref (Windows 11). Written for 7-Zip and libarchive's
 * "out of memory" on archive creation (2026-10-18): no syscall failed with NO_MEMORY, so
 * the size must come from a query that succeeds with different data.
 *   fsquery.exe <directory to create a file in> [<existing file> ...]
 */
#include <windows.h>
#include <winternl.h>
#include <winioctl.h>
#include <stdio.h>

static void hex(const char *tag, const void *p, ULONG n)
{
    const BYTE *b = p;
    printf("%s:", tag);
    for (ULONG i = 0; i < n && i < 64; i++)
        printf(" %02x", b[i]);
    printf("\n");
}

static void file_queries(const wchar_t *path, HANDLE h)
{
    BY_HANDLE_FILE_INFORMATION bh;
    LARGE_INTEGER sz;
    printf("== %ls type=%lu\n", path, GetFileType(h));
    if (GetFileInformationByHandle(h, &bh))
        printf("BHFI attr=%08lx vol=%08lx size=%lu:%lu links=%lu\n", bh.dwFileAttributes,
               bh.dwVolumeSerialNumber, bh.nFileSizeHigh, bh.nFileSizeLow, bh.nNumberOfLinks);
    else
        printf("BHFI err=%lu\n", GetLastError());
    if (GetFileSizeEx(h, &sz))
        printf("size=%lld\n", sz.QuadPart);
    for (int c = 0; c <= 0x18; c++) {
        BYTE buf[1024] = {0};
        char tag[32];
        sprintf(tag, "GFIBHEx %02x", c);
        if (GetFileInformationByHandleEx(h, (FILE_INFO_BY_HANDLE_CLASS)c, buf, sizeof(buf)))
            hex(tag, buf, 48);
        else
            printf("%s: err=%lu\n", tag, GetLastError());
    }
    for (ULONG c = 1; c <= 14; c++) {
        BYTE buf[1024] = {0};
        IO_STATUS_BLOCK io = {0};
        char tag[32];
        NTSTATUS st = NtQueryVolumeInformationFile(h, &io, buf, sizeof(buf), (FS_INFORMATION_CLASS)c);
        sprintf(tag, "NtQVIF %02lu st=%08lx n=%3lu", c, (ULONG)st, (ULONG)io.Information);
        hex(tag, buf, st < 0 ? 0 : 48);
    }
}

int wmain(int argc, wchar_t **argv)
{
    MEMORYSTATUSEX ms = { sizeof(ms) };
    SYSTEM_INFO si;
    if (argc < 2)
        return 2;
    GlobalMemoryStatusEx(&ms);
    GetSystemInfo(&si);
    printf("mem load=%lu phys=%llu avail=%llu pagefile=%llu virt=%llu availvirt=%llu\n",
           ms.dwMemoryLoad, ms.ullTotalPhys, ms.ullAvailPhys, ms.ullTotalPageFile,
           ms.ullTotalVirtual, ms.ullAvailVirtual);
    printf("sysinfo page=%lu gran=%lu cpus=%lu largepage=%llu\n", si.dwPageSize,
           si.dwAllocationGranularity, si.dwNumberOfProcessors, (ULONGLONG)GetLargePageMinimum());
    {
        wchar_t root[4] = { argv[1][0], L':', L'\\', 0 }, fs[64] = {0}, dev[8] = L"\\\\.\\X:";
        DWORD spc = 0, bps = 0, fc = 0, tc = 0, serial = 0, maxc = 0, flags = 0;
        ULARGE_INTEGER a, t, f;
        if (GetDiskFreeSpaceW(root, &spc, &bps, &fc, &tc))
            printf("GDFS spc=%lu bps=%lu free=%lu total=%lu\n", spc, bps, fc, tc);
        else
            printf("GDFS err=%lu\n", GetLastError());
        if (GetDiskFreeSpaceExW(root, &a, &t, &f))
            printf("GDFSEx avail=%llu total=%llu free=%llu\n", a.QuadPart, t.QuadPart, f.QuadPart);
        else
            printf("GDFSEx err=%lu\n", GetLastError());
        if (GetVolumeInformationW(root, NULL, 0, &serial, &maxc, &flags, fs, 64))
            printf("GVI fs=%ls maxcomp=%lu flags=%08lx\n", fs, maxc, flags);
        else
            printf("GVI err=%lu\n", GetLastError());
        dev[4] = argv[1][0];
        HANDLE v = CreateFileW(dev, 0, FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING, 0, NULL);
        if (v != INVALID_HANDLE_VALUE) {
            STORAGE_PROPERTY_QUERY q = { StorageAccessAlignmentProperty, PropertyStandardQuery };
            STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR d = {0};
            DISK_GEOMETRY g = {0};
            DWORD n = 0;
            if (DeviceIoControl(v, IOCTL_STORAGE_QUERY_PROPERTY, &q, sizeof(q), &d, sizeof(d), &n, NULL))
                printf("align logical=%lu physical=%lu\n", d.BytesPerLogicalSector, d.BytesPerPhysicalSector);
            else
                printf("align err=%lu\n", GetLastError());
            if (DeviceIoControl(v, IOCTL_DISK_GET_DRIVE_GEOMETRY, NULL, 0, &g, sizeof(g), &n, NULL))
                printf("geometry bps=%lu media=%d\n", g.BytesPerSector, g.MediaType);
            else
                printf("geometry err=%lu\n", GetLastError());
            CloseHandle(v);
        } else
            printf("volume open err=%lu\n", GetLastError());
    }
    {
        wchar_t out[MAX_PATH];
        lstrcpynW(out, argv[1], MAX_PATH - 16);
        lstrcatW(out, L"\\fsquery.tmp");
        HANDLE h = CreateFileW(out, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ, NULL, CREATE_ALWAYS,
                               FILE_ATTRIBUTE_NORMAL, NULL);
        if (h == INVALID_HANDLE_VALUE)
            printf("create %ls err=%lu\n", out, GetLastError());
        else {
            DWORD n;
            WriteFile(h, "fsquery\n", 8, &n, NULL);
            file_queries(out, h);
            CloseHandle(h);
            DeleteFileW(out);
        }
    }
    for (int i = 2; i < argc; i++) {
        HANDLE h = CreateFileW(argv[i], GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
                               OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL);
        if (h == INVALID_HANDLE_VALUE)
            printf("open %ls err=%lu\n", argv[i], GetLastError());
        else {
            file_queries(argv[i], h);
            CloseHandle(h);
        }
    }
    printf("fsquery done\n");
    return 0;
}
