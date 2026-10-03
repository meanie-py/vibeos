/* oldcrt — do the legacy CRT DLLs load and answer correctly now that they carry Rust?
 *
 * msvcrt20, msvcrt40 and crtdll (i386) are built from the same msvcrt_static C as
 * msvcrt.dll and now link the same Rust cohab archive (dll/win32/msvcrt/
 * msvcrt_rust_cohab.cmake). Nothing in a normal boot loads them, so the smoke gate
 * cannot see a broken one: this loads each by name (running its DllMain, which is
 * Rust), resolves a spread of exports and checks the answers against constants.
 * msvcrt.dll is the control -- if it fails here, the harness is wrong, not the DLLs.
 *
 * The spread is deliberate: a `...` export (sprintf, through the C varargs shims over
 * the Rust engine), plain Rust bodies (strlen, atoi, strtol, _itoa, qsort, toupper),
 * the heap (malloc/free/_strdup, which need DllMain's heap setup), and on crtdll one
 * of its OWN C overrides (_strinc from dllmain.c), which must still win first-match.
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

    if (strcmp(dll, "crtdll.dll") == 0) {
        strinc_t p_strinc = GET(strinc_t, "_strinc");
        const char *s = "xyz";
        check(dll, "_strinc (crtdll's own C)", p_strinc && p_strinc(s) == s + 1);
    }
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
