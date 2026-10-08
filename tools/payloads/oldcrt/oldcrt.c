/* oldcrt — do the legacy CRT DLLs load and answer correctly now that they carry Rust?
 *
 * msvcrt20, msvcrt40 and crtdll (i386) are built from the same msvcrt_static C as
 * msvcrt.dll and now link the same Rust cohab archive (dll/win32/msvcrt/
 * msvcrt_rust_cohab.cmake). Nothing in a normal boot loads them, so the smoke gate
 * cannot see a broken one: this loads each by name (running its DllMain, which is
 * Rust), resolves a spread of exports and checks the answers against constants.
 * msvcrt.dll is the control -- if it fails here, the harness is wrong, not the DLLs.
 *
 * The spread is deliberate: a `...` export (sprintf, a va_entry! thunk over the Rust
 * engine), plain Rust bodies (strlen, atoi, strtol, _itoa, qsort, toupper), and the heap
 * (malloc/free/_strdup, which need DllMain's heap setup). Then each legacy DLL's OWN
 * behaviour, which was its dllmain.c / msvcrt20.c and is msvcrt-crt-rust's legacy.rs under
 * prefixed names the spec maps: msvcrt20's by-value `new_mode`, and crtdll's byte-only
 * `_str*`, narrow `struct stat`, -1-on-truncation `_snprintf` and version words. Each is a
 * contract msvcrt.dll's export of the same name does NOT have, so a wrong spec mapping
 * (crtdll answering with msvcrt's body) fails here.
 */
#include <windows.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[OLDCRT] " __VA_ARGS__)

typedef int(__cdecl *sprintf_t)(char *, const char *, ...);
typedef size_t(__cdecl *strlen_t)(const char *);
typedef int(__cdecl *atoi_t)(const char *);
typedef long(__cdecl *strtol_t)(const char *, char **, int);
typedef char *(__cdecl *itoa_t)(int, char *, int);
typedef void *(__cdecl *malloc_t)(size_t);
typedef void(__cdecl *free_t)(void *);
typedef char *(__cdecl *strdup_t)(const char *);
typedef void(__cdecl *qsort_t)(void *, size_t, size_t, int(__cdecl *)(const void *, const void *));
typedef int(__cdecl *toupper_t)(int);
typedef char *(__cdecl *strinc_t)(const char *);
typedef char *(__cdecl *strdec_t)(const char *, const char *);
typedef size_t(__cdecl *strncnt_t)(const char *, size_t);
typedef unsigned int(__cdecl *strnextc_t)(const char *);
typedef char *(__cdecl *strninc_t)(const char *, size_t);
typedef char *(__cdecl *strspnp_t)(const char *, const char *);
typedef unsigned char *(__cdecl *mbsstr_t)(const unsigned char *, const unsigned char *);
typedef int(__cdecl *snprintf_t)(char *, size_t, const char *, ...);
typedef int *(__cdecl *errno_t_)(void);
typedef int(__cdecl *crtdll_stat_t)(const char *, void *);
typedef void(__cdecl *getmainargs_t)(int *, char ***, char ***, int, int);
typedef void(__cdecl *GetMainArgs_t)(int *, char ***, char ***, int);

/* crtdll's `struct stat` (dev_t is a short there, an unsigned int in msvcrt). */
struct crtdll_stat {
    short st_dev;
    unsigned short st_ino;
    unsigned short st_mode;
    short st_nlink, st_uid, st_gid;
    short st_rdev;
    long st_size;
    long st_atime, st_mtime, st_ctime;
};

static int g_fail;

static void check(const char *dll, const char *what, int ok)
{
    if (!ok)
        g_fail++;
    SAY("%s %s %s\n", dll, what, ok ? "ok" : "FAIL");
}

static int __cdecl cmp_int(const void *a, const void *b)
{
    return *(const int *)a - *(const int *)b;
}

#define GET(T, name) ((T)(void *)GetProcAddress(h, name))

static void probe_crtdll(HMODULE h)
{
    static const char dll[] = "crtdll.dll";
    const char *s = "xyz";
    strinc_t p_strinc = GET(strinc_t, "_strinc");
    strdec_t p_strdec = GET(strdec_t, "_strdec");
    strncnt_t p_strncnt = GET(strncnt_t, "_strncnt");
    strnextc_t p_strnextc = GET(strnextc_t, "_strnextc");
    strninc_t p_strninc = GET(strninc_t, "_strninc");
    strspnp_t p_strspnp = GET(strspnp_t, "_strspnp");
    mbsstr_t p_mbsstr = GET(mbsstr_t, "_mbsstr");
    snprintf_t p_snprintf = GET(snprintf_t, "_snprintf");
    errno_t_ p_errno = GET(errno_t_, "_errno");
    crtdll_stat_t p_stat = GET(crtdll_stat_t, "_stat");
    GetMainArgs_t p_GMA = GET(GetMainArgs_t, "__GetMainArgs");

    check(dll, "_strinc", p_strinc && p_strinc(s) == s + 1);
    check(dll, "_strdec", p_strdec && p_strdec(s, s + 2) == s + 1);
    check(dll, "_strncnt", p_strncnt && p_strncnt("hello", 3) == 3 && p_strncnt("hi", 9) == 2);
    /* Byte-only and sign-extending: crtdll's reads a plain char. */
    check(dll, "_strnextc", p_strnextc && p_strnextc("\xe9z") == 0xFFFFFFE9u);
    check(dll, "_strninc", p_strninc && p_strninc(s, 2) == s + 2);
    check(dll, "_strspnp", p_strspnp && p_strspnp("aab", "a") && *p_strspnp("aab", "a") == 'b'
                               && p_strspnp("aa", "a") == NULL);
    /* Empty in empty is NULL in crtdll; msvcrt's returns the haystack. */
    check(dll, "_mbsstr", p_mbsstr && p_mbsstr((const unsigned char *)"", (const unsigned char *)"") == NULL
                              && p_mbsstr((const unsigned char *)"abc", (const unsigned char *)"bc") != NULL);
    if (p_snprintf && p_errno) {
        char buf[8];
        *p_errno() = 77;
        int trunc = p_snprintf(buf, 4, "%s", "hello");
        int e1 = *p_errno();
        int fits = p_snprintf(buf, sizeof(buf), "%d", 42);
        check(dll, "_snprintf -1 on truncation, errno cleared",
              trunc == -1 && e1 == 0 && fits == 2 && strcmp(buf, "42") == 0);
    } else
        check(dll, "_snprintf resolves", 0);
    if (p_stat) {
        char path[MAX_PATH];
        struct crtdll_stat st;
        memset(&st, 0xcc, sizeof(st));
        DWORD n = GetModuleFileNameA(h, path, sizeof(path));
        HANDLE f = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
        DWORD size = f != INVALID_HANDLE_VALUE ? GetFileSize(f, NULL) : 0;
        if (f != INVALID_HANDLE_VALUE)
            CloseHandle(f);
        int r = n ? p_stat(path, &st) : -1;
        /* The narrow layout puts st_size at offset 16; msvcrt's would put it at 20. */
        check(dll, "_stat (crtdll struct)", r == 0 && size && st.st_size == (long)size
                                               && (st.st_mode & 0x8000) && st.st_nlink >= 1);
    } else
        check(dll, "_stat resolves", 0);
    if (p_GMA) {
        int argc = -1;
        char **argv = NULL, **envp = NULL;
        p_GMA(&argc, &argv, &envp, 0);
        check(dll, "__GetMainArgs", argc >= 1 && argv && argv[0]);
    } else
        check(dll, "__GetMainArgs resolves", 0);
    {
        unsigned *base = GET(unsigned *, "_baseversion_dll");
        unsigned *major = GET(unsigned *, "_basemajor_dll");
        unsigned *cpumode = GET(unsigned *, "_cpumode_dll");
        unsigned want = GetVersion() >> 16;
        check(dll, "_baseversion_dll", base && major && cpumode && *base == want
                                           && *major == (want >> 8) && *cpumode == 1);
    }
}

static void probe(const char *dll)
{
    HMODULE h = LoadLibraryA(dll);
    check(dll, "LoadLibrary", h != NULL);
    if (!h)
        return;
    char buf[64];
    sprintf_t p_sprintf = GET(sprintf_t, "sprintf");
    strlen_t p_strlen = GET(strlen_t, "strlen");
    atoi_t p_atoi = GET(atoi_t, "atoi");
    strtol_t p_strtol = GET(strtol_t, "strtol");
    itoa_t p_itoa = GET(itoa_t, "_itoa");
    malloc_t p_malloc = GET(malloc_t, "malloc");
    free_t p_free = GET(free_t, "free");
    strdup_t p_strdup = GET(strdup_t, "_strdup");
    qsort_t p_qsort = GET(qsort_t, "qsort");
    toupper_t p_toupper = GET(toupper_t, "toupper");

    if (p_sprintf) {
        int n = p_sprintf(buf, "%d|%5.2f|%s|%x", -42, 3.14159, "ab", 255);
        static const char want[] = "-42| 3.14|ab|ff";
        check(dll, "sprintf", n == (int)sizeof(want) - 1 && strcmp(buf, want) == 0);
    } else
        check(dll, "sprintf resolves", 0);
    check(dll, "strlen", p_strlen && p_strlen("hello") == 5);
    check(dll, "atoi", p_atoi && p_atoi("  -123x") == -123);
    if (p_strtol) {
        char *end = NULL;
        long v = p_strtol("0x1fZ", &end, 16);
        check(dll, "strtol", v == 31 && end && *end == 'Z');
    } else
        check(dll, "strtol resolves", 0);
    check(dll, "_itoa", p_itoa && strcmp(p_itoa(-255, buf, 16), "ffffff01") == 0);
    if (p_malloc && p_free && p_strdup) {
        char *m = p_malloc(100);
        char *d = p_strdup("dup me");
        check(dll, "malloc/_strdup", m && d && strcmp(d, "dup me") == 0);
        p_free(m);
        p_free(d);
    } else
        check(dll, "heap resolves", 0);
    if (p_qsort) {
        int a[] = { 5, -1, 3, 0, 2 };
        p_qsort(a, 5, sizeof(int), cmp_int);
        check(dll, "qsort", a[0] == -1 && a[1] == 0 && a[2] == 2 && a[3] == 3 && a[4] == 5);
    } else
        check(dll, "qsort resolves", 0);
    check(dll, "toupper", p_toupper && p_toupper('q') == 'Q' && p_toupper('7') == '7');
    {
        /* The CRT's _stricmp. The legacy specs once left this unmapped, so ntdll's import
         * library answered it; both agree on ASCII, so check it resolves to the CRT body. */
        void *p_strcmpi = (void *)GetProcAddress(h, "_strcmpi");
        void *p_stricmp = (void *)GetProcAddress(h, "_stricmp");
        check(dll, "_strcmpi is the CRT's _stricmp", p_strcmpi && p_strcmpi == p_stricmp);
    }

    if (strcmp(dll, "msvcrt20.dll") == 0) {
        /* new_mode by VALUE: 0 here would be a NULL int* to msvcrt's own __getmainargs. */
        getmainargs_t p_gma = GET(getmainargs_t, "__getmainargs");
        int argc = -1;
        char **argv = NULL, **envp = NULL;
        if (p_gma)
            p_gma(&argc, &argv, &envp, 0, 0);
        check(dll, "__getmainargs (new_mode by value)", p_gma && argc >= 1 && argv && argv[0]);
    }

    if (strcmp(dll, "crtdll.dll") == 0)
        probe_crtdll(h);
}

int main(void)
{
    static const char *const dlls[] = { "msvcrt.dll", "msvcrt20.dll", "msvcrt40.dll",
                                        "crtdll.dll" };
    SAY("start\n");
    for (unsigned i = 0; i < sizeof(dlls) / sizeof(dlls[0]); i++)
        probe(dlls[i]);
    SAY("done %s fails=%d\n", g_fail ? "FAIL" : "PASS", g_fail);
    return g_fail != 0;
}
