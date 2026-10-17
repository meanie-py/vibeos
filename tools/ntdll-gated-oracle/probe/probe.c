/* Black-box oracle: call the REAL Windows ntdll exports this staging lane implements and
 * print every observable result (return values, NTSTATUS, and the full byte image of every
 * caller-visible buffer). The host Rust tests (../tests/oracle.rs) replay the SAME cases
 * against the Rust bodies and diff against probe/windows-oracle.txt.
 *
 * Build: x86_64-w64-mingw32-clang -O1 -o probe.exe probe.c ; run from WSL (interop).
 * Nothing here looks inside ntdll: only GetProcAddress + calls, as an apitest does. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

typedef LONG NTSTATUS;
typedef struct { USHORT Length, MaximumLength; PWSTR Buffer; } USTR;
typedef struct { USHORT Group; UCHAR Number; UCHAR Reserved; } PROCNUM;

static HMODULE nt;
#define GET(name) ((void *)GetProcAddress(nt, name))

static void hex(const char *tag, const void *p, size_t n)
{
    const unsigned char *b = p;
    printf("%s=", tag);
    for (size_t i = 0; i < n; i++) printf("%02x", b[i]);
    printf("\n");
}

/* ---- hashes ---------------------------------------------------------------------------- */
/* Messages: "", "abc", the RFC 1321/1320 alphabet, and a 1000-byte pattern fed in odd chunks. */
static unsigned char big[1000];
struct msg { const char *tag; const unsigned char *p; unsigned n; unsigned chunk; };

static void hashes(void)
{
    void (WINAPI *Md5I)(void *) = GET("MD5Init"), (WINAPI *Md5F)(void *) = GET("MD5Final");
    void (WINAPI *Md5U)(void *, const void *, unsigned) = GET("MD5Update");
    void (WINAPI *Md4I)(void *) = GET("MD4Init"), (WINAPI *Md4F)(void *) = GET("MD4Final");
    void (WINAPI *Md4U)(void *, const void *, unsigned) = GET("MD4Update");
    void (WINAPI *ShaI)(void *) = GET("A_SHAInit");
    void (WINAPI *ShaU)(void *, const void *, unsigned) = GET("A_SHAUpdate");
    void (WINAPI *ShaF)(void *, ULONG *) = GET("A_SHAFinal");
    for (int i = 0; i < 1000; i++) big[i] = (unsigned char)(i * 7 + 3);
    static const char alpha[] = "abcdefghijklmnopqrstuvwxyz";
    static const char digits[] =
        "12345678901234567890123456789012345678901234567890123456789012345678901234567890";
    struct msg m[] = {
        { "empty", (const unsigned char *)"", 0, 0 },
        { "abc", (const unsigned char *)"abc", 3, 0 },
        { "alpha", (const unsigned char *)alpha, 26, 0 },
        { "digits", (const unsigned char *)digits, 80, 0 },
        { "big1", big, 1000, 0 },
        { "big7", big, 1000, 7 },
        { "big64", big, 1000, 64 },
        { "b55", big, 55, 0 }, { "b56", big, 56, 0 }, { "b63", big, 63, 0 }, { "b64", big, 64, 0 },
    };
    for (unsigned k = 0; k < sizeof(m) / sizeof(m[0]); k++) {
        unsigned char ctx[256];
        char tag[64];
        /* MD5 */
        memset(ctx, 0xCC, sizeof ctx);
        Md5I(ctx);
        sprintf(tag, "md5.%s.init", m[k].tag); hex(tag, ctx, 104);
        if (m[k].chunk) for (unsigned o = 0; o < m[k].n; o += m[k].chunk)
            Md5U(ctx, m[k].p + o, m[k].n - o < m[k].chunk ? m[k].n - o : m[k].chunk);
        else Md5U(ctx, m[k].p, m[k].n);
        sprintf(tag, "md5.%s.upd", m[k].tag); hex(tag, ctx, 104);
        Md5F(ctx);
        sprintf(tag, "md5.%s.fin", m[k].tag); hex(tag, ctx, 104);
        /* MD4 */
        memset(ctx, 0xCC, sizeof ctx);
        Md4I(ctx);
        sprintf(tag, "md4.%s.init", m[k].tag); hex(tag, ctx, 104);
        if (m[k].chunk) for (unsigned o = 0; o < m[k].n; o += m[k].chunk)
            Md4U(ctx, m[k].p + o, m[k].n - o < m[k].chunk ? m[k].n - o : m[k].chunk);
        else Md4U(ctx, m[k].p, m[k].n);
        sprintf(tag, "md4.%s.upd", m[k].tag); hex(tag, ctx, 104);
        Md4F(ctx);
        sprintf(tag, "md4.%s.fin", m[k].tag); hex(tag, ctx, 104);
        /* SHA-1 */
        ULONG res[8];
        memset(ctx, 0xCC, sizeof ctx);
        memset(res, 0xCC, sizeof res);
        ShaI(ctx);
        sprintf(tag, "sha.%s.init", m[k].tag); hex(tag, ctx, 116);
        if (m[k].chunk) for (unsigned o = 0; o < m[k].n; o += m[k].chunk)
            ShaU(ctx, m[k].p + o, m[k].n - o < m[k].chunk ? m[k].n - o : m[k].chunk);
        else ShaU(ctx, m[k].p, m[k].n);
        sprintf(tag, "sha.%s.upd", m[k].tag); hex(tag, ctx, 116);
        ShaF(ctx, res);
        sprintf(tag, "sha.%s.fin", m[k].tag); hex(tag, ctx, 116);
        sprintf(tag, "sha.%s.res", m[k].tag); hex(tag, res, 32);
    }
}

/* ---- memory/port resource descriptors ------------------------------------------------- */
#define CM_SIZE 20
#define IO_SIZE 32
static const unsigned long long lens[] = {
    0, 0x1000, 0xFFFFFFFFull, 0x100000000ull, 0x100000080ull, 0x100000100ull, 0xFFFFFFFF00ull,
    0xFFFFFFFF01ull, 0x10000000000ull, 0x10000000100ull, 0x10000010000ull, 0xFFFFFFFF0000ull,
    0xFFFFFFFF0001ull, 0x1000000000000ull, 0x1000100000000ull, 0x100000000000100ull,
    0xFFFFFFFF00000000ull, 0xFFFFFFFF00000001ull, 0xFFFFFFFFFFFFFFFFull,
};
static const unsigned long long aligns[] = {
    0, 1, 0x81, 0x1000, 0x10000, 0xFFFFFFFFull, 0x100000000ull, 0x10000000000ull,
    0x8000000000000001ull, 0xFFFFFFFFFFFFFFFFull,
};
static const UCHAR types[] = { 0, 1, 2, 3, 4, 7, 8 };
static const USHORT preflags[] = { 0x0000, 0x5555, 0x0E01, 0x0200 };
#define N(a) (sizeof(a) / sizeof((a)[0]))

static void memres(void)
{
    NTSTATUS (WINAPI *CmEnc)(void *, UCHAR, ULONGLONG, ULONGLONG) = GET("RtlCmEncodeMemIoResource");
    ULONGLONG (WINAPI *CmDec)(void *, ULONGLONG *) = GET("RtlCmDecodeMemIoResource");
    NTSTATUS (WINAPI *IoEnc)(void *, UCHAR, ULONGLONG, ULONGLONG, ULONGLONG, ULONGLONG) =
        GET("RtlIoEncodeMemIoResource");
    ULONGLONG (WINAPI *IoDec)(void *, ULONGLONG *, ULONGLONG *, ULONGLONG *) =
        GET("RtlIoDecodeMemIoResource");
    NTSTATUS (WINAPI *Closest)(ULONGLONG, ULONGLONG *) = GET("RtlFindClosestEncodableLength");
    char tag[96];

    for (unsigned t = 0; t < N(types); t++)
        for (unsigned f = 0; f < N(preflags); f++)
            for (unsigned l = 0; l < N(lens); l++) {
                unsigned char d[64];
                memset(d, 0x55, sizeof d);
                memcpy(d + 2, &preflags[f], 2);
                NTSTATUS s = CmEnc(d, types[t], lens[l], 0x123456789ABCull);
                sprintf(tag, "cmenc.t%u.f%04x.l%016llx", types[t], preflags[f], lens[l]);
                printf("%s.st=%08lx\n", tag, (unsigned long)s);
                strcat(tag, ".d"); hex(tag, d, CM_SIZE);
                ULONGLONG st = 0xEEEEEEEEEEEEEEEEull;
                ULONGLONG r = CmDec(d, &st);
                sprintf(tag, "cmdec.t%u.f%04x.l%016llx", types[t], preflags[f], lens[l]);
                printf("%s=%016llx,%016llx\n", tag, r, st);
                for (unsigned a = 0; a < N(aligns); a++) {
                    unsigned char e[64];
                    memset(e, 0x55, sizeof e);
                    memcpy(e + 4, &preflags[f], 2);
                    s = IoEnc(e, types[t], lens[l], aligns[a], 0x1000, 0xFFFFFFFFFFull);
                    sprintf(tag, "ioenc.t%u.f%04x.l%016llx.a%016llx", types[t], preflags[f], lens[l], aligns[a]);
                    printf("%s.st=%08lx\n", tag, (unsigned long)s);
                    strcat(tag, ".d"); hex(tag, e, IO_SIZE);
                    ULONGLONG al = 1, mn = 2, mx = 3;
                    r = IoDec(e, &al, &mn, &mx);
                    sprintf(tag, "iodec.t%u.f%04x.l%016llx.a%016llx", types[t], preflags[f], lens[l], aligns[a]);
                    printf("%s=%016llx,%016llx,%016llx,%016llx\n", tag, r, al, mn, mx);
                }
            }
    /* Decode of hand-built descriptors: every type x every flag nibble combination. */
    for (unsigned t = 0; t < N(types); t++)
        for (unsigned fl = 0; fl < 16; fl++) {
            unsigned char d[64];
            memset(d, 0, sizeof d);
            d[0] = types[t];
            USHORT flags = (USHORT)(fl << 8);
            memcpy(d + 2, &flags, 2);
            ULONGLONG start = 0xABCD, len32 = 0x01000000;
            memcpy(d + 4, &start, 8);
            memcpy(d + 12, &len32, 4);
            ULONGLONG st = 0;
            ULONGLONG r = CmDec(d, &st);
            printf("cmraw.t%u.fl%x=%016llx,%016llx\n", types[t], fl, r, st);
            unsigned char e[64];
            memset(e, 0, sizeof e);
            e[1] = types[t];
            memcpy(e + 4, &flags, 2);
            ULONG al32 = 0x00020000;
            memcpy(e + 8, &len32, 4);
            memcpy(e + 12, &al32, 4);
            ULONGLONG al = 0, mn = 0, mx = 0;
            r = IoDec(e, &al, NULL, &mx);
            printf("ioraw.t%u.fl%x=%016llx,%016llx,%016llx\n", types[t], fl, r, al, mx);
        }
    for (unsigned l = 0; l < N(lens); l++) {
        ULONGLONG out = 0xEEEEEEEEEEEEEEEEull;
        NTSTATUS s = Closest(lens[l], &out);
        printf("closest.%016llx=%08lx,%016llx\n", lens[l], (unsigned long)s, out);
    }
}

/* ---- RtlIsNameInExpression ------------------------------------------------------------ */
static void names(void)
{
    BOOLEAN (WINAPI *IsName)(USTR *, USTR *, BOOLEAN, PWCH) = GET("RtlIsNameInExpression");
    static const WCHAR *ex[] = {
        L"", L"*", L"**", L"*.*", L"?", L"??", L"*?", L"?*?", L"he*o", L"*.cpl", L"F0_*.*",
        L"F0_*.", L"F0_<\"*", L"<.exe", L"<.EXE", L"ab<exe", L"a>c.exe", L"a>>>exe", L"*.c.d",
        L"ntoskrnl\"exe", L"ntoskrnl.exe\"", L"\"x", L"FI<<<<<<<<", L"<<<<<<<<<.<", L"F<",
        L"*a*ab*abc", L"A*C", L"*Z", L"<", L">", L"\"", L"*.", L".*", L"<.<", L"x<", L"x>",
        L">>>.>>>", L"*\"*", L"ABC", L"abc", L"\xC9", L"*\xE9",
    };
    static const WCHAR *nm[] = {
        L"", L"a", L"ab", L".", L"..", L"abc", L"ABC", L"hello", L"helo", L"x.cpl", L"F0_",
        L"F0_.", L"f0_001.txt", L"F0_001.txt", L"F0_001", L"test.exe", L"TEST.EXE", L"abcd.exe",
        L"abcdexe", L"abc.exe", L"ac.exe", L"a.b.c.d", L"ntoskrnl.exe", L"FILE", L"FILE.TXT",
        L"aabaabcdadabdabc", L"A<>\"C", L"**Z", L"x", L"x.", L"x.y", L"x.y.z", L"abc.def.ghi",
        L"\xE9", L"\xC9", L"a\xE9",
    };
    for (unsigned i = 0; i < N(ex); i++)
        for (unsigned j = 0; j < N(nm); j++)
            for (int ic = 0; ic < 2; ic++) {
                USTR e = { (USHORT)(wcslen(ex[i]) * 2), (USHORT)(wcslen(ex[i]) * 2 + 2), (PWSTR)ex[i] };
                USTR n = { (USHORT)(wcslen(nm[j]) * 2), (USHORT)(wcslen(nm[j]) * 2 + 2), (PWSTR)nm[j] };
                printf("name.%u.%u.%d=%d\n", i, j, ic, IsName(&e, &n, (BOOLEAN)ic, NULL));
            }
}

/* ---- processor number ----------------------------------------------------------------- */
static void procnum(void)
{
    ULONG (WINAPI *Plain)(void) = GET("NtGetCurrentProcessorNumber");
    ULONG (WINAPI *Ex)(PROCNUM *) = GET("NtGetCurrentProcessorNumberEx");
    ULONG (WINAPI *ZwEx)(PROCNUM *) = GET("ZwGetCurrentProcessorNumberEx");
    printf("procnum.zw_is_nt=%d\n", Ex == ZwEx);
    for (int cpu = 0; cpu < 2; cpu++) {
        SetThreadAffinityMask(GetCurrentThread(), (DWORD_PTR)1 << cpu);
        Sleep(1);
        PROCNUM p;
        memset(&p, 0xCC, sizeof p);
        ULONG r = Ex(&p);
        printf("procnum.cpu%d.plain=%lu\n", cpu, Plain());
        printf("procnum.cpu%d.ret=%lu\n", cpu, r);
        printf("procnum.cpu%d.group=%u number=%u reserved=%u\n", cpu, p.Group, p.Number, p.Reserved);
        printf("procnum.cpu%d.null_ret=%lu\n", cpu, Ex(NULL));
    }
    static unsigned char raw[16] __attribute__((aligned(16)));
    memset(raw, 0xCC, sizeof raw);
    printf("procnum.odd_ret=%08lx\n", Ex((PROCNUM *)(raw + 1)));
    printf("procnum.two_ret=%08lx\n", Ex((PROCNUM *)(raw + 6)));
    hex("procnum.raw", raw, 16);
    printf("procnum.kernel_ret=%08lx\n", Ex((PROCNUM *)(ULONG_PTR)0xFFFF800000000000ull));
    printf("procnum.lowpage_ret=%08lx\n", Ex((PROCNUM *)(ULONG_PTR)0x1000));
}

/* ---- SList push-list ------------------------------------------------------------------ */
static void slist(void)
{
    PSLIST_ENTRY (WINAPI *PushEx)(PSLIST_HEADER, PSLIST_ENTRY, PSLIST_ENTRY, ULONG) =
        GET("RtlInterlockedPushListSListEx");
    static SLIST_HEADER h;
    static SLIST_ENTRY e[5] __attribute__((aligned(16)));
    InitializeSListHead(&h);
    InterlockedPushEntrySList(&h, &e[0]);
    e[1].Next = &e[2]; e[2].Next = &e[3]; e[3].Next = (PSLIST_ENTRY)0x1;
    PSLIST_ENTRY r = PushEx(&h, &e[1], &e[3], 3);
    printf("slist.ret_is_e0=%d\n", r == &e[0]);
    printf("slist.depth=%u\n", QueryDepthSList(&h));
    printf("slist.e3next_is_e0=%d\n", e[3].Next == &e[0]);
    printf("slist.first_is_e1=%d\n", InterlockedPopEntrySList(&h) == &e[1]);
    /* empty list: returns NULL, tail gets NULL */
    InitializeSListHead(&h);
    e[4].Next = (PSLIST_ENTRY)0x1;
    r = PushEx(&h, &e[4], &e[4], 1);
    printf("slist.empty_ret_null=%d tail_null=%d depth=%u\n", r == NULL, e[4].Next == NULL, QueryDepthSList(&h));
}

/* ---- locale ---------------------------------------------------------------------------- */
static void locale(void)
{
    BOOLEAN (WINAPI *Valid)(PCWSTR, ULONG) = GET("RtlIsValidLocaleName");
    NTSTATUS (WINAPI *Conv)(LCID, ULONG, ULONG, PWSTR, ULONG) = GET("RtlConvertLCIDToString");
    static const WCHAR *ln[] = {
        L"", L"en-US", L"EN-us", L"en", L"de-DE", L"zh-Hant", L"zh-HK_radstr", L"x-IV",
        L"en-XX", L"qps-ploc", L"sr-Latn-RS", L"en-US-", L"en_US", L"iv", L"invariant",
        L"de", L"zh", L"zh-CHS", L"fr-FR", L"ja-JP_radstr",
    };
    static const ULONG fl[] = { 0, 1, 2, 3, 4, 0x80000000u };
    for (unsigned i = 0; i < N(ln); i++)
        for (unsigned f = 0; f < N(fl); f++)
            printf("valid.%u.%lx=%d\n", i, (unsigned long)fl[f], Valid(ln[i], fl[f]));
    NTSTATUS (WINAPI *N2L)(PCWSTR, LCID *, ULONG) = GET("RtlLocaleNameToLcid");
    static const WCHAR *ln2[] = {
        L"en_US", L"de_DE", L"en_us", L"ja_JP_radstr", L"ja-JP-radstr", L"_", L"en-US_",
        L"zh_Hant", L"sr_Latn_RS", L"en-US", L"en", L"", L"x-IV", L"zh-CHS",
    };
    for (unsigned i = 0; i < N(ln2); i++)
        for (unsigned f = 0; f < 4; f++) {
            LCID l = 0xDEADBEEF;
            NTSTATUS st = N2L(ln2[i], &l, f);
            printf("n2l.%u.%u=%08lx,%08lx valid=%d\n", i, f, (unsigned long)st, (unsigned long)l, Valid(ln2[i], f));
        }
    static const LCID lc[] = { 0, 0x409, 0x7F, 0x10407, 0xFFFFFFFF, 0x12345 };
    static const ULONG base[] = { 0, 2, 8, 10, 16, 17 };
    static const ULONG pad[] = { 0, 1, 4, 8, 9, 20 };
    static const ULONG size[] = { 0, 1, 4, 5, 8, 9, 10, 64 };
    for (unsigned a = 0; a < N(lc); a++)
        for (unsigned b = 0; b < N(base); b++)
            for (unsigned p = 0; p < N(pad); p++)
                for (unsigned s = 0; s < N(size); s++) {
                    WCHAR buf[80];
                    for (int k = 0; k < 80; k++) buf[k] = L'#';
                    NTSTATUS st = Conv(lc[a], base[b], pad[p], buf, size[s]);
                    char tag[96];
                    sprintf(tag, "conv.%lx.b%lu.p%lu.s%lu", (unsigned long)lc[a], base[b], pad[p], size[s]);
                    printf("%s.st=%08lx\n", tag, (unsigned long)st);
                    strcat(tag, ".buf"); hex(tag, buf, 70 * 2);
                }
}

int main(void)
{
    nt = GetModuleHandleW(L"ntdll.dll");
    setvbuf(stdout, NULL, _IOFBF, 1 << 20);
    hashes();
    memres();
    names();
    procnum();
    slist();
    locale();
    return 0;
}
