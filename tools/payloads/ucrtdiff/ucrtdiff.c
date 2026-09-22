/*
 * ucrtdiff — Tier 2 ucrt difftest harness, graded IN THE GUEST.
 *
 * Every case calls the SAME operation twice:
 *
 *     ref_foo(...)   the genuine Microsoft UCRT C, linked privately from
 *                    libucrt_ref.a (built by tools/ucrt_tier2_refset.py,
 *                    which renames ucrtbase's exports to ref_* with
 *                    llvm-objcopy on the objects the real build produced)
 *     foo(...)       whatever ucrtbase.dll actually ships, through the
 *                    import lib — our Rust once a family is taken over,
 *                    the C before it
 *
 * so a row grades the SHIPPING ARTIFACT, not an archive. Tier 1
 * (sdk/lib/rust/ucrt-difftest) cannot reach any of this: it runs on the host,
 * where `long` is 64 bits and Windows' is 32, which silently mis-grades ~188
 * public declarations including the whole strtol/atol family.
 *
 * Results go to the kernel debug serial via ntdll DbgPrint, the same channel
 * the kmtest sweep uses; tools/payloads/ucrtdiff/payload.yaml grafts this into
 * the All Users Startup folder and holds the boot open until [UCRTDIFF] done.
 *
 * Declarations are written out by hand rather than included: the UCRT headers
 * cannot name the ref_ side, and a `#define foo ref_foo` prelude collapses the
 * C++ overloads that differ only in return type (it fails identically at
 * corecrt_math.h:525 -- that is why this design renames objects, not source).
 */

#include <windows.h>
/* The BARE side is declared by the real headers, so its prototypes cannot drift
 * from the ones every other caller sees. We are -nostdlib, so these resolve
 * from the ucrtbase import lib the link line names, not from the toolchain's. */
#include <ctype.h>
#include <stdlib.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* The renamed genuine-Microsoft bodies. Declared by hand because no header can
 * name them -- which is the point: a `#define foo ref_foo` prelude would rewrite
 * the header declarations too, and collapse the C++ overloads that differ only
 * in return type (corecrt_math.h:525). We rename OBJECTS, not source.
 *
 * Only exports the ucrt objects actually DEFINE are listed. strchr, strrchr and
 * strstr are ucrtbase exports but are implemented in vcruntime, so there is no
 * genuine ucrt body here to grade them against. */
long __cdecl ref_strtol(const char *, char **, int);
unsigned long __cdecl ref_strtoul(const char *, char **, int);
int __cdecl ref_atoi(const char *);
long __cdecl ref_atol(const char *);
int __cdecl ref_tolower(int);
int __cdecl ref_toupper(int);
int __cdecl ref_isalpha(int);
int __cdecl ref_isdigit(int);
int __cdecl ref_isspace(int);
int __cdecl ref_isupper(int);
int __cdecl ref_ispunct(int);
size_t __cdecl ref_strlen(const char *);
int __cdecl ref_strcmp(const char *, const char *);
int __cdecl ref__stricmp(const char *, const char *);
int __cdecl ref_strncmp(const char *, const char *, size_t);
size_t __cdecl ref_strspn(const char *, const char *);
size_t __cdecl ref_strcspn(const char *, const char *);

static int g_pass, g_fail, g_cases;

/* A comparison reports the FAMILY, because that is the unit the burn-down and
 * tools/ucrt-takeover.yaml speak in -- a per-case count would not join to it. */
static void grade(const char *family, const char *what, long long ours, long long ref)
{
    g_cases++;
    if (ours == ref) {
        g_pass++;
    } else {
        g_fail++;
        DbgPrint("[UCRTDIFF] FAIL %s %s ours=%lld ref=%lld\n", family, what, ours, ref);
    }
}

/* strcmp's SIGN is contractual, its magnitude is not: MSVC returns the byte
 * difference where other implementations return -1/0/1, and grading the raw
 * value would report a difference that no caller can observe. */
static long long sgn(long long v) { return v < 0 ? -1 : (v > 0 ? 1 : 0); }

/* A returned pointer is only comparable as an OFFSET: the two sides scan two
 * different copies of the same literal, so the addresses never match. -1 means
 * "not found", which must agree too. */
static long long off(const char *base, const char *p) { return p ? (long long)(p - base) : -1; }

static void run_strtox(void)
{
    static const char *const in[] = {
        "42", "-42", "0", "  \t 17", "+99", "0x1f", "2147483647", "-2147483648",
        "2147483648", "-2147483649", "4294967295", "4294967296", "999999999999999",
        "", "   ", "abc", "12abc", "0x", "-", "+", "0b101", "  -0X7FfF", "007",
    };
    static const int bases[] = { 0, 8, 10, 16, 36, 2 };
    char *e1, *e2;
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        for (unsigned b = 0; b < sizeof(bases) / sizeof(bases[0]); b++) {
            e1 = e2 = 0;
            grade("strtol", in[i], strtol(in[i], &e1, bases[b]), ref_strtol(in[i], &e2, bases[b]));
            /* The end pointer is half the contract and is where a rewrite most
             * often drifts -- a body can return the right number having consumed
             * the wrong number of characters. */
            grade("strtol.end", in[i], off(in[i], e1), off(in[i], e2));
            e1 = e2 = 0;
            grade("strtoul", in[i],
                  (long long)strtoul(in[i], &e1, bases[b]),
                  (long long)ref_strtoul(in[i], &e2, bases[b]));
            grade("strtoul.end", in[i], off(in[i], e1), off(in[i], e2));
        }
        grade("atoi", in[i], atoi(in[i]), ref_atoi(in[i]));
        grade("atol", in[i], atol(in[i]), ref_atol(in[i]));
    }
}

static void run_ctype(void)
{
    /* The full 0..255 range plus EOF: the 0x80-0xFF half is where the narrow
     * ctype table and the locale's codepage actually decide anything, and it is
     * the half a C-locale-only test would never reach. */
    for (int c = -1; c < 256; c++) {
        char w[8];
        w[0] = 'c'; w[1] = '='; w[2] = (char)('0' + (c / 100 + 256) % 10);
        w[3] = (char)('0' + (c / 10 + 256) % 10); w[4] = (char)('0' + (c + 256) % 10); w[5] = 0;
        grade("tolower", w, tolower(c), ref_tolower(c));
        grade("toupper", w, toupper(c), ref_toupper(c));
        /* Normalised: the is* family contracts a NONZERO value, not a specific
         * one, and MSVC returns the raw ctype bitmask. */
        grade("isalpha", w, !!isalpha(c), !!ref_isalpha(c));
        grade("isdigit", w, !!isdigit(c), !!ref_isdigit(c));
        grade("isspace", w, !!isspace(c), !!ref_isspace(c));
        grade("isupper", w, !!isupper(c), !!ref_isupper(c));
        grade("ispunct", w, !!ispunct(c), !!ref_ispunct(c));
    }
}

static void run_string(void)
{
    static const char *const s[] = {
        "", "a", "abc", "ABC", "aBc", "hello world", "  padded  ",
        "\x80\xff mixed", "aaaa", "abcabcabc", "z",
    };
    const unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned i = 0; i < n; i++) {
        grade("strlen", s[i], (long long)strlen(s[i]), (long long)ref_strlen(s[i]));
        for (unsigned j = 0; j < n; j++) {
            grade("strcmp", s[i], sgn(strcmp(s[i], s[j])), sgn(ref_strcmp(s[i], s[j])));
            grade("_stricmp", s[i], sgn(_stricmp(s[i], s[j])), sgn(ref__stricmp(s[i], s[j])));
            grade("strncmp", s[i], sgn(strncmp(s[i], s[j], 3)), sgn(ref_strncmp(s[i], s[j], 3)));
            grade("strspn", s[i], (long long)strspn(s[i], s[j]), (long long)ref_strspn(s[i], s[j]));
            grade("strcspn", s[i],
                  (long long)strcspn(s[i], s[j]), (long long)ref_strcspn(s[i], s[j]));
        }
    }
}

int main(void)
{
    DbgPrint("[UCRTDIFF] start\n");
    run_strtox();
    run_ctype();
    run_string();
    /* A ZERO-case run is a vacuous pass and must read as a failure, the same
     * rule tests/qemu/cli/difftest.py's graded() enforces for the host tier. */
    if (g_cases == 0) {
        DbgPrint("[UCRTDIFF] FATAL: no cases ran\n");
        DbgPrint("[UCRTDIFF] done cases=0 pass=0 fail=1\n");
        return 1;
    }
    DbgPrint("[UCRTDIFF] done cases=%d pass=%d fail=%d\n", g_cases, g_pass, g_fail);
    return g_fail != 0;
}
