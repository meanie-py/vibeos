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

/* The THIRD side: our existing msvcrt Rust bodies, renamed rust_* from
 * libmsvcrt_crt_cargo.a. These are the 799 exports that already have a Rust body
 * written but never reach ucrtbase's link line -- letting them through under
 * --allow-multiple-definition would silently replace genuine UCRT bodies with
 * MSVCRT-semantics ones (ucrtbase is _MSVCR_VER 140, msvcrt is 0). Grading them
 * HERE answers 'is this body UCRT-correct?' without flipping anything, and
 * without deleting the C that is its only oracle. */
long __cdecl rust_strtol(const char *, char **, int);
unsigned long __cdecl rust_strtoul(const char *, char **, int);
int __cdecl rust_atoi(const char *);
long __cdecl rust_atol(const char *);
int __cdecl rust_tolower(int);
int __cdecl rust_toupper(int);
int __cdecl rust_isalpha(int);
int __cdecl rust_isdigit(int);
int __cdecl rust_isspace(int);
int __cdecl rust_isupper(int);
int __cdecl rust_ispunct(int);
size_t __cdecl rust_strlen(const char *);
int __cdecl rust_strcmp(const char *, const char *);
int __cdecl rust__stricmp(const char *, const char *);
int __cdecl rust_strncmp(const char *, const char *, size_t);
size_t __cdecl rust_strspn(const char *, const char *);
size_t __cdecl rust_strcspn(const char *, const char *);

/* The msvcrt Rust archive is not self-contained: a few of its bodies reach msvcrt
 * internals that ucrtbase does not export, so no rename rule covers them and nothing
 * on this link line defines them. Rust codegen units bundle many functions, so the
 * objects come in whether or not we grade what pulls them.
 *
 * They are satisfied HERE, loudly, rather than by linking libmsvcrt_static.a: that
 * archive defines strtol and friends too, and --allow-multiple-definition would let
 * it answer a graded name silently -- the harness would then grade msvcrt against
 * Microsoft while reporting it as the shipping DLL. Nothing below is reachable from
 * any graded family; if that ever changes, the run says so instead of lying. */
static void unreachable_stub(const char *who)
{
    DbgPrint("[UCRTDIFF] FATAL: a graded body reached %s, which this harness stubs\n", who);
    DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
    ExitProcess(2);
}

int __cdecl _scprintf(const char *fmt, ...);
int __cdecl _scprintf(const char *fmt, ...)
{
    (void)fmt;
    unreachable_stub("_scprintf");
    return 0;
}

void __stdcall __wine_RtlUnwind(void *a, void *b, void *c, void *d);
void __stdcall __wine_RtlUnwind(void *a, void *b, void *c, void *d)
{
    (void)a; (void)b; (void)c; (void)d;
    unreachable_stub("__wine_RtlUnwind");
}

/* fwprintf and cprintf are NOT ucrtbase exports -- UCRT makes them header inlines
 * over __stdio_common_vfwprintf -- so no rename rule covers the msvcrt Rust bodies'
 * references to them, and the import lib has nothing to offer. */
int __cdecl _cprintf(const char *fmt, ...);
int __cdecl _cprintf(const char *fmt, ...)
{
    (void)fmt;
    unreachable_stub("_cprintf");
    return 0;
}

int __cdecl fwprintf(void *stream, const wchar_t *fmt, ...);
int __cdecl fwprintf(void *stream, const wchar_t *fmt, ...)
{
    (void)stream; (void)fmt;
    unreachable_stub("fwprintf");
    return 0;
}

/* msvcrt's C++ EH helpers, which live in libmsvcrt_static.a. */
void __cdecl call_copy_ctor(void *a, void *b, void *c, int d);
void __cdecl call_copy_ctor(void *a, void *b, void *c, int d)
{
    (void)a; (void)b; (void)c; (void)d;
    unreachable_stub("call_copy_ctor");
}

void __cdecl call_dtor(void *a, void *b);
void __cdecl call_dtor(void *a, void *b)
{
    (void)a; (void)b;
    unreachable_stub("call_dtor");
}

static int g_cases;
/* DLL: what ucrtbase.dll ships today. While a family is still answered by the
 * vendored Microsoft C, this compares that C against a private copy of itself --
 * a real positive control for the harness, but NOT a verdict on our code. */
static int g_dll_pass, g_dll_fail;
/* RUST: our existing msvcrt Rust body against Microsoft's. THIS is the reuse
 * verdict, and it is meaningful whether or not the family has been taken over. */
static int g_rust_pass, g_rust_fail;

/* A comparison reports the FAMILY, because that is the unit the burn-down and
 * tools/ucrt-takeover.yaml speak in -- a per-case count would not join to it. */
static void grade(const char *family, const char *what, long long dll, long long rust,
                  long long ref)
{
    g_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        DbgPrint("[UCRTDIFF] DLL-FAIL %s %s dll=%lld ref=%lld\n", family, what, dll, ref);
    }
    if (rust == ref) {
        g_rust_pass++;
    } else {
        g_rust_fail++;
        DbgPrint("[UCRTDIFF] RUST-FAIL %s %s rust=%lld ref=%lld\n", family, what, rust, ref);
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

/* strtol/strtoul are graded at six bases; a FAIL that names only the input cannot
 * be acted on, because the same string behaves differently per base. */
static const char *with_base(const char *family, int base)
{
    static char buf[48];
    int n = 0;
    while (family[n] && n < 32) { buf[n] = family[n]; n++; }
    buf[n++] = ':'; buf[n++] = 'b';
    if (base >= 10) buf[n++] = (char)('0' + base / 10);
    buf[n++] = (char)('0' + base % 10);
    buf[n] = 0;
    return buf;
}

static void run_strtox(void)
{
    static const char *const in[] = {
        "42", "-42", "0", "  \t 17", "+99", "0x1f", "2147483647", "-2147483648",
        "2147483648", "-2147483649", "4294967295", "4294967296", "999999999999999",
        "", "   ", "abc", "12abc", "0x", "-", "+", "0b101", "  -0X7FfF", "007",
    };
    static const int bases[] = { 0, 8, 10, 16, 36, 2 };
    char *e1, *e2, *e3;
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        for (unsigned b = 0; b < sizeof(bases) / sizeof(bases[0]); b++) {
            e1 = e2 = e3 = 0;
            grade(with_base("strtol", bases[b]), in[i], strtol(in[i], &e1, bases[b]),
                  rust_strtol(in[i], &e3, bases[b]), ref_strtol(in[i], &e2, bases[b]));
            /* The end pointer is half the contract and is where a rewrite most
             * often drifts -- a body can return the right number having consumed
             * the wrong number of characters. */
            grade(with_base("strtol.end", bases[b]), in[i], off(in[i], e1), off(in[i], e3), off(in[i], e2));
            e1 = e2 = e3 = 0;
            grade(with_base("strtoul", bases[b]), in[i], (long long)strtoul(in[i], &e1, bases[b]),
                  (long long)rust_strtoul(in[i], &e3, bases[b]),
                  (long long)ref_strtoul(in[i], &e2, bases[b]));
            grade(with_base("strtoul.end", bases[b]), in[i], off(in[i], e1), off(in[i], e3), off(in[i], e2));
        }
        grade("atoi", in[i], atoi(in[i]), rust_atoi(in[i]), ref_atoi(in[i]));
        grade("atol", in[i], atol(in[i]), rust_atol(in[i]), ref_atol(in[i]));
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
        grade("tolower", w, tolower(c), rust_tolower(c), ref_tolower(c));
        grade("toupper", w, toupper(c), rust_toupper(c), ref_toupper(c));
        /* Normalised: the is* family contracts a NONZERO value, not a specific
         * one, and MSVC returns the raw ctype bitmask. */
        grade("isalpha", w, !!isalpha(c), !!rust_isalpha(c), !!ref_isalpha(c));
        grade("isdigit", w, !!isdigit(c), !!rust_isdigit(c), !!ref_isdigit(c));
        grade("isspace", w, !!isspace(c), !!rust_isspace(c), !!ref_isspace(c));
        grade("isupper", w, !!isupper(c), !!rust_isupper(c), !!ref_isupper(c));
        grade("ispunct", w, !!ispunct(c), !!rust_ispunct(c), !!ref_ispunct(c));
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
        grade("strlen", s[i], (long long)strlen(s[i]), (long long)rust_strlen(s[i]),
              (long long)ref_strlen(s[i]));
        for (unsigned j = 0; j < n; j++) {
            grade("strcmp", s[i], sgn(strcmp(s[i], s[j])), sgn(rust_strcmp(s[i], s[j])),
                  sgn(ref_strcmp(s[i], s[j])));
            grade("_stricmp", s[i], sgn(_stricmp(s[i], s[j])), sgn(rust__stricmp(s[i], s[j])),
                  sgn(ref__stricmp(s[i], s[j])));
            grade("strncmp", s[i], sgn(strncmp(s[i], s[j], 3)), sgn(rust_strncmp(s[i], s[j], 3)),
                  sgn(ref_strncmp(s[i], s[j], 3)));
            grade("strspn", s[i], (long long)strspn(s[i], s[j]),
                  (long long)rust_strspn(s[i], s[j]), (long long)ref_strspn(s[i], s[j]));
            grade("strcspn", s[i], (long long)strcspn(s[i], s[j]),
                  (long long)rust_strcspn(s[i], s[j]), (long long)ref_strcspn(s[i], s[j]));
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
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        return 1;
    }
    DbgPrint("[UCRTDIFF] done cases=%d dll_pass=%d dll_fail=%d rust_pass=%d rust_fail=%d\n",
             g_cases, g_dll_pass, g_dll_fail, g_rust_pass, g_rust_fail);
    return (g_dll_fail || g_rust_fail) != 0;
}
