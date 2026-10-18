/* fwprobe — does GetSystemFirmwareTable answer for each provider, the way Windows does?
 *
 * 'RSMB': Enumerate lists one table, ID 0; Get returns the 8-byte RawSMBIOSData header
 * followed by Length bytes of SMBIOS structures, which walk cleanly and include type 0 (BIOS).
 * 'ACPI': Enumerate lists every table the XSDT/RSDT names plus the DSDT; Get on each
 * returns the whole table: its own signature, Length equal to the bytes returned, and a
 * byte sum of zero. A FACP must be listed, and so must the DSDT it points at.
 * 'FIRM': the 0xC0000 and 0xE0000 regions, 128 KB each, on x86 (none elsewhere).
 * Verdict: "[FWPROBE] done VERDICT=PASS|FAIL". */
#define _WIN32_WINNT 0x0601
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* DbgPrint for the guest's serial; stdout too, for `vibeos-cli winref run` on Windows. */
static void SAY(const char *Format, ...)
{
    char line[512];
    va_list args;
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    int len = lstrlenA(lstrcpyA(line, "[FWPROBE] "));
    va_start(args, Format);
    wvsprintfA(line + len, Format, args);
    va_end(args);
    DbgPrint("%s", line);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, lstrlenA(line), &n, NULL);
}

/* A table ID is the table's signature bytes in memory order ('FACP' reads as 'PCAF' in C);
 * a provider is a C multichar constant, so 'RSMB' is 0x52534D42 - the opposite byte order. */
#define SIG(a, b, c, d) ((DWORD)(a) | ((DWORD)(b) << 8) | ((DWORD)(c) << 16) | ((DWORD)(d) << 24))
#define PROVIDER(a, b, c, d) SIG(d, c, b, a)

static int failures;

static void FAIL(const char *what, DWORD sig, DWORD got, DWORD want)
{
    char s[5];
    CopyMemory(s, &sig, 4);
    s[4] = 0;
    SAY("FAIL %s %s: got 0x%lx, want 0x%lx\n", what, s, got, want);
    failures++;
}

static BYTE *fetch(DWORD provider, DWORD id, UINT *len)
{
    UINT need = GetSystemFirmwareTable(provider, id, NULL, 0);
    BYTE *buf;
    *len = 0;
    if (!need)
    {
        SAY("size query: 0 bytes, error %lu\n", GetLastError());
        return NULL;
    }
    buf = HeapAlloc(GetProcessHeap(), 0, need);
    if (!buf)
        return NULL;
    *len = GetSystemFirmwareTable(provider, id, buf, need);
    if (*len != need)
    {
        FAIL("refetch length", id, *len, need);
        HeapFree(GetProcessHeap(), 0, buf);
        return NULL;
    }
    return buf;
}

static void probe_rsmb(void)
{
    DWORD ids[16];
    UINT n = EnumSystemFirmwareTables(PROVIDER('R', 'S', 'M', 'B'), ids, sizeof(ids));
    UINT len;
    BYTE *t;
    DWORD body;
    if (n != sizeof(DWORD) || ids[0] != 0)
    {
        SAY("RSMB enumerate: %u bytes, error %lu\n", n, GetLastError());
        FAIL("RSMB enumerate bytes", PROVIDER('R', 'S', 'M', 'B'), n, sizeof(DWORD));
    }
    t = fetch(PROVIDER('R', 'S', 'M', 'B'), 0, &len);
    if (!t)
    {
        FAIL("RSMB get", PROVIDER('R', 'S', 'M', 'B'), GetLastError(), 0);
        return;
    }
    CopyMemory(&body, t + 4, 4);
    SAY("RSMB: %u bytes, SMBIOS %u.%u\n", len, t[1], t[2]);
    if (body + 8 != len)
        FAIL("RSMB header length", PROVIDER('R', 'S', 'M', 'B'), body + 8, len);
    /* Walk the structures: each is a formatted area of at least 4 bytes (type, length, handle)
     * then strings ending in a double NUL. The walk must land exactly on the end, and the BIOS
     * information (type 0) must be among them. A header of the wrong size shifts the first
     * structure and the walk derails. The spec fixes no order: QEMU lists type 1 first. */
    {
        DWORD off = 8, structures = 0;
        int bios = 0, ok = 1;
        while (off + 4 <= len)
        {
            BYTE type = t[off], slen = t[off + 1];
            if (slen < 4 || off + slen > len)
            {
                ok = 0;
                break;
            }
            bios |= type == 0;
            structures++;
            off += slen;
            while (off + 1 < len && (t[off] || t[off + 1]))
                off++;
            off += 2;
            if (type == 127)
                break;
        }
        SAY("RSMB: %lu structures, walk ended at %lu of %u\n", structures, off, len);
        if (!ok || off != len)
            FAIL("RSMB structure walk end", PROVIDER('R', 'S', 'M', 'B'), off, len);
        if (!bios)
            FAIL("RSMB has a type 0", PROVIDER('R', 'S', 'M', 'B'), 0, 1);
    }
    HeapFree(GetProcessHeap(), 0, t);
}

static void probe_acpi(void)
{
    DWORD ids[256];
    UINT bytes = EnumSystemFirmwareTables(PROVIDER('A', 'C', 'P', 'I'), ids, sizeof(ids));
    UINT count = bytes / sizeof(DWORD), i, j;
    int facp = 0, dsdt = 0;
    if (!bytes || bytes % sizeof(DWORD) || bytes > sizeof(ids))
    {
        SAY("ACPI enumerate: %u bytes, error %lu; trying FACP and DSDT directly\n", bytes, GetLastError());
        FAIL("ACPI enumerate bytes", PROVIDER('A', 'C', 'P', 'I'), bytes, 0);
        ids[0] = SIG('F', 'A', 'C', 'P');
        ids[1] = SIG('D', 'S', 'D', 'T');
        count = 2;
    }
    for (i = 0; i < count; i++)
    {
        UINT len;
        BYTE *t = fetch(PROVIDER('A', 'C', 'P', 'I'), ids[i], &len);
        DWORD sig, tlen;
        BYTE sum = 0;
        char s[5];
        CopyMemory(s, &ids[i], 4);
        s[4] = 0;
        if (!t)
        {
            FAIL("ACPI get", ids[i], GetLastError(), 0);
            continue;
        }
        CopyMemory(&sig, t, 4);
        CopyMemory(&tlen, t + 4, 4);
        for (j = 0; j < len; j++)
            sum += t[j];
        SAY("ACPI %s: %u bytes, revision %u, checksum %s\n", s, len, t[8], sum ? "BAD" : "ok");
        if (sig != ids[i])
            FAIL("ACPI signature", ids[i], sig, ids[i]);
        if (tlen != len)
            FAIL("ACPI length", ids[i], tlen, len);
        if (sum)
            FAIL("ACPI checksum", ids[i], sum, 0);
        facp |= ids[i] == SIG('F', 'A', 'C', 'P');
        dsdt |= ids[i] == SIG('D', 'S', 'D', 'T');
        HeapFree(GetProcessHeap(), 0, t);
    }
    if (!facp)
        FAIL("ACPI lists", SIG('F', 'A', 'C', 'P'), 0, 1);
    if (!dsdt)
        FAIL("ACPI lists", SIG('D', 'S', 'D', 'T'), 0, 1);
    if (GetSystemFirmwareTable(PROVIDER('A', 'C', 'P', 'I'), SIG('N', 'O', 'P', 'E'), NULL, 0))
        FAIL("ACPI get of an absent table", SIG('N', 'O', 'P', 'E'), 1, 0);
}

/* 'FIRM': the legacy BIOS regions as raw memory, IDs 0xC0000 and 0xE0000, 128 KB each, on x86
 * only. A BIOS boot keeps its RSDP and SMBIOS anchors in the E/F segment, so finding them (with
 * a valid RSDP checksum) shows the kernel maps the real memory there; a UEFI boot has none. */
static void probe_firm(void)
{
    DWORD ids[8];
    UINT bytes = EnumSystemFirmwareTables(PROVIDER('F', 'I', 'R', 'M'), ids, sizeof(ids));
    UINT i, len, off;
    BYTE *t;
#if defined(__i386__) || defined(__x86_64__)
    if (bytes != 2 * sizeof(DWORD) || ids[0] != 0xC0000 || ids[1] != 0xE0000)
    {
        FAIL("FIRM enumerate bytes", PROVIDER('F', 'I', 'R', 'M'), bytes, 2 * sizeof(DWORD));
        return;
    }
#else
    if (bytes != 0)
        FAIL("FIRM enumerate bytes", PROVIDER('F', 'I', 'R', 'M'), bytes, 0);
    return;
#endif
    for (i = 0; i < 2; i++)
    {
        t = fetch(PROVIDER('F', 'I', 'R', 'M'), ids[i], &len);
        if (!t || len != 0x20000)
        {
            FAIL("FIRM get length", ids[i], len, 0x20000);
            if (t)
                HeapFree(GetProcessHeap(), 0, t);
            continue;
        }
        for (off = 0; off + 20 <= len; off += 16)
        {
            if (!memcmp(t + off, "RSD PTR ", 8))
            {
                BYTE sum = 0;
                UINT k;
                for (k = 0; k < 20; k++)
                    sum += t[off + k];
                SAY("FIRM %05lx: RSD PTR at %05lx, checksum %s\n", ids[i], ids[i] + off, sum ? "BAD" : "ok");
                if (sum)
                    FAIL("FIRM RSDP checksum", ids[i], sum, 0);
            }
            else if (!memcmp(t + off, "_SM3_", 5) || !memcmp(t + off, "_SM_", 4))
                SAY("FIRM %05lx: %.4s anchor at %05lx\n", ids[i], t + off, ids[i] + off);
        }
        {
            /* What the region holds, for comparing against a physical read of the same
             * addresses: the reset vector at 0xFFFF0 and a sample at 0xF52D0. */
            UINT ff = 0;
            for (off = 0; off < len; off++)
                ff += t[off] == 0xff;
            SAY("FIRM %05lx: %u of %u bytes are 0xFF\n", ids[i], ff, len);
            if (ids[i] == 0xE0000)
            {
                const BYTE *r = t + 0x1fff0, *q = t + 0x152d0;
                SAY("FIRM ffff0:  %02x %02x %02x %02x %02x %02x %02x %02x\n", r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
                SAY("FIRM f52d0:  %02x %02x %02x %02x %02x %02x %02x %02x\n", q[0], q[1], q[2], q[3], q[4], q[5], q[6], q[7]);
            }
        }
        HeapFree(GetProcessHeap(), 0, t);
    }
    SAY("FIRM: 2 regions of 128 KB\n");
}

int main(void)
{
    probe_rsmb();
    probe_acpi();
    probe_firm();
    SAY("done VERDICT=%s (%d failures)\n", failures ? "FAIL" : "PASS", failures);
    return failures;
}
