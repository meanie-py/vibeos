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

/* fwprintf and _cprintf are NOT ucrtbase exports -- UCRT makes them header inlines
 * over __stdio_common_vfwprintf -- so no rename rule covers the msvcrt Rust bodies'
 * references to them, and the import lib has nothing to offer.
 *
 * They cannot be stubbed as ordinary C: the headers declare fwprintf with a signature
 * this file would conflict with, and conio.h declares _cprintf STATIC, so a non-static
 * definition is not merely redundant but illegal.
 *
 * An explicit assembler name sidesteps both -- the C identifier is distinct, so nothing
 * conflicts, while the object defines exactly the symbol the Rust archive asks for.
 * (lld in PE mode has no --defsym, which was the other way to do this.) The label is
 * taken LITERALLY, so the i386 cdecl underscore has to be spelled here. */
#ifdef _M_IX86
#define UCRTDIFF_SYM(n) "_" n
#else
#define UCRTDIFF_SYM(n) n
#endif

void ucrtdiff_fwprintf_stub(void) __asm__(UCRTDIFF_SYM("fwprintf"));
void ucrtdiff_fwprintf_stub(void) { unreachable_stub("fwprintf"); }

void ucrtdiff_cprintf_stub(void) __asm__(UCRTDIFF_SYM("_cprintf"));
void ucrtdiff_cprintf_stub(void) { unreachable_stub("_cprintf"); }

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

/* ---- generated-case corpora ------------------------------------------- */
/* One looping scalar argument gets the FULL range plus the edges: the ctype and
 * mbctype families live here, and 0x80-0xFF is the half where the narrow table and
 * the codepage actually decide anything. EOF (-1) is included because every ctype
 * and case-conversion function contracts a specific answer for it. */
static const int g_int_full[] = {
    -1, 0, 1, 7, 8, 9, 10, 11, 12, 13, 26, 27, 31, 32, 33, 47, 48, 57, 58, 64, 65, 90,
    91, 96, 97, 122, 123, 126, 127, 128, 129, 159, 160, 161, 191, 192, 223, 224, 254, 255,
};
/* The sweep STOPS at 255. The ctype and mbc families are contracted over EOF and
 * 0..255, so beyond that there is no contract to grade -- and our Rust stops producing
 * output at exactly 256, which has the shape of a 256-entry table indexed without a
 * bounds check (mbcs.rs _ismbcgraph_l -> msvcrt_mbc_to_wc_l). That is worth fixing on
 * its own, but sweeping into it here bought nothing and cost every finding AFTER the
 * one export that tripped it: the run ended at _ismbcgraph_l with 180-odd exports
 * still ungraded. Recorded in the design doc. */
#define G_INT_FULL_N (sizeof(g_int_full) / sizeof(g_int_full[0]))

/* Anything with more than one looping argument uses these: the product is what
 * explodes -- a four-argument shape at the full sweep would be billions of cases. */
static const int g_int_small[] = { -1, 0, 1, 10, 16, 65, 128, 255 };
#define G_INT_SMALL_N (sizeof(g_int_small) / sizeof(g_int_small[0]))

static const char *const g_narrow[] = { "", "a", "abc", "ABC", "12", "\x80\xff", " x " };
#define G_NARROW_N (sizeof(g_narrow) / sizeof(g_narrow[0]))

static const wchar_t *const g_wide[] = { L"", L"a", L"abc", L"ABC", L"12", L"\x00e9", L" x " };
#define G_WIDE_N (sizeof(g_wide) / sizeof(g_wide[0]))

/* mbstring takes unsigned char*; a separate corpus rather than casts, so the
 * generated calls stay type-correct against the real declarations. */
static const unsigned char *const g_mbstr[] = {
    (const unsigned char *)"", (const unsigned char *)"a", (const unsigned char *)"abc",
    (const unsigned char *)"\x81\x40", (const unsigned char *)"\xa1\xa2",
    (const unsigned char *)"12", (const unsigned char *)"\xff",
};
#define G_MBSTR_N (sizeof(g_mbstr) / sizeof(g_mbstr[0]))

static const char *label_idx(size_t i)
{
    static char buf[16];
    buf[0] = 'i'; buf[1] = '=';
    buf[2] = (char)('0' + (i / 100) % 10);
    buf[3] = (char)('0' + (i / 10) % 10);
    buf[4] = (char)('0' + i % 10);
    buf[5] = 0;
    return buf;
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
        if (g_dll_fail <= 300)
            DbgPrint("[UCRTDIFF] DLL-FAIL %s %s dll=%lld ref=%lld\n", family, what, dll, ref);
        else if (g_dll_fail == 301)
            DbgPrint("[UCRTDIFF] (further DLL-FAIL rows suppressed; see the count)\n");
    }
    if (rust == ref) {
        g_rust_pass++;
    } else {
        g_rust_fail++;
        /* A regressed family can fail every case; serial is a shared, slow channel and
         * flooding it would push the [UCRTDIFF] done line out of the drain window. The
         * COUNT is the verdict -- the first rows are only there to make it actionable. */
        if (g_rust_fail <= 300)
            DbgPrint("[UCRTDIFF] RUST-FAIL %s %s rust=%lld ref=%lld\n", family, what, rust, ref);
        else if (g_rust_fail == 301)
            DbgPrint("[UCRTDIFF] (further RUST-FAIL rows suppressed; see the count)\n");
    }
}

/* An export with no Rust body yet (one of the ~200 that need genuinely new code):
 * only the shipping DLL can be graded. Counting a nonexistent body as a pass would
 * inflate the reuse number with exactly the exports the reuse plan does NOT cover. */
static void grade2(const char *family, const char *what, long long dll, long long ref)
{
    g_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        if (g_dll_fail <= 300)
            DbgPrint("[UCRTDIFF] DLL-FAIL %s %s dll=%lld ref=%lld\n", family, what, dll, ref);
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

/* The generated half: 233 exports on i386, declared with __typeof__ of the real
 * prototype and swept by the macros above. Generated by
 * tools/gen_ucrt_tier2_cases.py; build.sh regenerates it before every link. */
#include "generated.inc"

/* The Rust side's per-thread state, which msvcrt.dll's DllMain normally sets up.
 *
 * `msvcrt_get_thread_data()` does `TlsGetValue(msvcrt_tls_index)`, and msvcrt_tls_index
 * is a global that DllMain fills from TlsAlloc(). The harness links a PRIVATE copy of
 * these bodies, for which no DllMain ever runs, so the index is 0 -- and TLS slot 0 is
 * not empty, it belongs to something else. The garbage that comes back is then used as
 * a thread_data_t, which is why every locale-aware family took the process down
 * silently: first the mbc ones, then, as each was excluded, the collation ones.
 *
 * Allocating a real slot here is half the fix; msvcrt_init_locale() is the other half,
 * since thread data is only reachable once MSVCRT_locale exists. Both have to happen
 * before any graded call. This is the "state isolation in practice" item the design doc
 * has listed as unverified from the start. */
extern unsigned long msvcrt_tls_index;
unsigned char __cdecl msvcrt_init_locale(void);

static int init_rust_thread_state(void)
{
    unsigned long idx = TlsAlloc();
    if (idx == TLS_OUT_OF_INDEXES) {
        DbgPrint("[UCRTDIFF] FATAL: TlsAlloc failed, the Rust side has no thread data\n");
        return 0;
    }
    msvcrt_tls_index = idx;
    if (!msvcrt_init_locale()) {
        DbgPrint("[UCRTDIFF] FATAL: msvcrt_init_locale failed; MSVCRT_locale stays null\n");
        return 0;
    }
    return 1;
}

int main(void)
{
    DbgPrint("[UCRTDIFF] start\n");
    if (!init_rust_thread_state()) {
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        return 1;
    }
    run_generated();
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
