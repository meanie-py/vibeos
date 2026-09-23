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

/* ---- fault isolation --------------------------------------------------
 *
 * WHY THIS IS THE WHOLE POINT. Without it, ONE faulting body ends the run and hides
 * every export after it, so each blocker costs a separate four-minute boot and they
 * are discovered strictly in order. That is how six boots were spent learning six
 * names. Guarded, a faulting body becomes a FAULT row and the sweep carries on, so a
 * single run reports every fault, every DIFF and every MATCH at once.
 *
 * VEH, NOT `__try`. clang's __try does not catch a HARDWARE fault under mingw at all
 * -- it never emits async-exception handler liveness -- which dll/ntdll/seh_poc_user.c
 * records after proving it, and its 32-bit x86 SEH codegen is broken here besides
 * (undefined `L__ehtable$` / `$parent_frame_offset` labels). A vectored handler that
 * redirects the faulting context is the only mechanism that works, and it is plain
 * Win32 on all three arches.
 *
 * IT ONLY FIRES ON arm64, WHICH IS WHY TIER 2 RUNS THERE. RtlDispatchException's
 * RtlCallVectoredExceptionHandlers block is `#ifndef _NTOSKRNL_` and librtl always
 * builds with -D_NTOSKRNL_, so first-chance VEH dispatch is dead code on i386 and
 * amd64 -- rtl-except/src/{i386,amd64}/except.rs say so outright. arm64 has it wired
 * and proved: "[SEH-POC-U] VEH on hardware AV: fired=1 resumed=1 (PASS)".
 *
 * The guard has a POSITIVE CONTROL: a synthetic body that always faults runs first,
 * and the run refuses to grade if it is not caught. A guard that has never fired is a
 * guard nobody knows is connected -- and on the arches above it genuinely is not.
 */
static volatile int g_fault_armed;
static volatile int g_fault_flag;
static volatile unsigned long g_fault_code;
static volatile int g_fault_hits;

/* NO setjmp/longjmp. On arm64 setjmp resolves to the SEH-based __intrinsic_setjmpex,
 * which collides with the renamed reference set AND would need valid unwind data to
 * return into from a context we redirected -- precisely the machinery we cannot lean
 * on while grading it. Instead the handler does what seh_poc_user.c proved works here:
 * ADVANCE THE PC PAST THE FAULTING INSTRUCTION and resume. arm64 instructions are a
 * fixed 4 bytes, so that is exact (it is also why this is arm64-only; x86's
 * variable-length encoding has no such step, and VEH does not fire there anyway).
 *
 * Skipping one instruction does not by itself end the sweep, so the handler also raises
 * g_fault_flag, and every generated loop tests it. The body limps at most to the next
 * case boundary, the export is recorded FAULT, and the run carries on to the next one. */
static LONG CALLBACK ucrtdiff_veh(EXCEPTION_POINTERS *ep)
{
    if (!g_fault_armed)
        return EXCEPTION_CONTINUE_SEARCH;
    g_fault_code = ep->ExceptionRecord->ExceptionCode;
    g_fault_flag = 1;
    if (++g_fault_hits > 4096) {
        /* A body faulting without making progress would spin here forever and cost the
         * whole run. Bail loudly rather than hang the boot. */
        DbgPrint("[UCRTDIFF] FATAL: %d faults without progress; aborting\n", g_fault_hits);
        return EXCEPTION_CONTINUE_SEARCH;
    }
#if defined(__aarch64__) || defined(_M_ARM64)
    ep->ContextRecord->Pc += 4;
    return EXCEPTION_CONTINUE_EXECUTION;
#else
    return EXCEPTION_CONTINUE_SEARCH;
#endif
}

#define UCRTDIFF_GUARD(faulted, stmt)                                                  \
    do {                                                                               \
        g_fault_flag = 0;                                                              \
        g_fault_hits = 0;                                                              \
        g_fault_armed = 1;                                                             \
        stmt;                                                                          \
        g_fault_armed = 0;                                                             \
        (faulted) = g_fault_flag;                                                      \
    } while (0)

/* ---- per-export accounting -------------------------------------------
 *
 * The verdict is reported PER EXPORT, because that is the unit the burn-down and
 * tools/ucrt-takeover.yaml speak in, and the unit a reader acts on. A log of failing
 * cases cannot be joined to either; a table of
 *     EXPORT <name> <MATCH|DIFF n/total|FAULT|NORUST>
 * is a worklist -- MATCH is a body we can take over with evidence behind it, DIFF
 * carries its first failing case, FAULT names something to investigate. */
static int g_ex_cases, g_ex_dll_fail, g_ex_rust_fail;
static const char *g_ex_name = "?";
static char g_ex_first[96];

static void export_begin(const char *name)
{
    g_ex_name = name;
    g_ex_cases = g_ex_dll_fail = g_ex_rust_fail = 0;
    g_ex_first[0] = 0;
}

static void export_end(int faulted, int has_rust)
{
    if (faulted) {
        DbgPrint("[UCRTDIFF] EXPORT %s FAULT code=0x%08lx after=%d\n",
                 g_ex_name, g_fault_code, g_ex_cases);
        return;
    }
    if (g_ex_cases == 0) {
        /* Vacuous: an export that graded nothing must not read as a pass. */
        DbgPrint("[UCRTDIFF] EXPORT %s NOCASES\n", g_ex_name);
        return;
    }
    if (!has_rust) {
        DbgPrint("[UCRTDIFF] EXPORT %s NORUST dll=%s n=%d\n", g_ex_name,
                 g_ex_dll_fail ? "DIFF" : "MATCH", g_ex_cases);
        return;
    }
    if (g_ex_rust_fail == 0 && g_ex_dll_fail == 0) {
        DbgPrint("[UCRTDIFF] EXPORT %s MATCH n=%d\n", g_ex_name, g_ex_cases);
    } else {
        DbgPrint("[UCRTDIFF] EXPORT %s DIFF rust=%d dll=%d of %d first=%s\n",
                 g_ex_name, g_ex_rust_fail, g_ex_dll_fail, g_ex_cases, g_ex_first);
    }
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
/* The FIRST failing case for this export, kept so a DIFF row is actionable without
 * printing every case. Serial is shared and slow; a regressed family can fail
 * thousands of times and flooding it would push the summary out of the drain. */
static void note_first(const char *family, const char *what)
{
    int i = 0, j;
    if (g_ex_first[0])
        return;
    for (j = 0; family[j] && i < (int)sizeof(g_ex_first) - 2; j++)
        g_ex_first[i++] = family[j];
    g_ex_first[i++] = ':';
    for (j = 0; what[j] && i < (int)sizeof(g_ex_first) - 1; j++)
        g_ex_first[i++] = what[j];
    g_ex_first[i] = 0;
}

static void grade(const char *family, const char *what, long long dll, long long rust,
                  long long ref)
{
    g_cases++;
    g_ex_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        g_ex_dll_fail++;
        note_first(family, what);
    }
    if (rust == ref) {
        g_rust_pass++;
    } else {
        g_rust_fail++;
        g_ex_rust_fail++;
        note_first(family, what);
    }
}

/* An export with no Rust body yet (one of the ~200 that need genuinely new code):
 * only the shipping DLL can be graded. Counting a nonexistent body as a pass would
 * inflate the reuse number with exactly the exports the reuse plan does NOT cover. */
static void grade2(const char *family, const char *what, long long dll, long long ref)
{
    g_cases++;
    g_ex_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        g_ex_dll_fail++;
        note_first(family, what);
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

/* POSITIVE CONTROL for the guard. A guard that has never fired is a guard nobody
 * knows is connected -- and a disconnected one turns every FAULT into a dead run
 * again, silently. This faults on purpose; if it is NOT caught the run cannot be
 * trusted and says so instead of grading anything. */
static int guard_selftest(void)
{
    int faulted;
    volatile int *nowhere = (volatile int *)0;
    UCRTDIFF_GUARD(faulted, (void)*nowhere);
    return faulted;
}

/* ---- entry -------------------------------------------------------------
 *
 * NOT mainCRTStartup. An arm64 exe that takes its CRT from ucrtbase.dll dies before
 * main with nothing on serial -- proved with two probes: a bare-entry exe linking only
 * ntdll+kernel32 RUNS, while the same harness through mainCRTStartup does not, and so
 * does a hello-world with ucrtbase+vcstartup and NO reference set. So it is CRT startup
 * itself, not this harness and not the renamed objects. It is consistent with the rest
 * of the tree: kmtest's arm64 payload deliberately links classic msvcrt.dll and starts
 * fine, and ucrtbase_winetest is an i386/amd64 rostest.
 *
 * So we enter bare and run the one part of CRT init the graded bodies actually need:
 * the C initializer table between __xi_a and __xi_z (vcstartup supplies the section
 * bounds). The index is printed as we go, so if one of THOSE is what breaks arm64
 * startup, this run names it instead of dying silently -- which is a bug report the
 * tree wants regardless of Tier 2.
 */
typedef int(__cdecl *ucrtdiff_pifv)(void);
extern ucrtdiff_pifv __xi_a[], __xi_z[];

static int run_c_initializers(void)
{
    ucrtdiff_pifv *p;
    int n = 0;
    for (p = __xi_a; p < __xi_z; p++) {
        if (!*p)
            continue;
        DbgPrint("[UCRTDIFF] init %d\n", n);
        if ((*p)() != 0) {
            DbgPrint("[UCRTDIFF] FATAL: C initializer %d failed\n", n);
            return 0;
        }
        n++;
    }
    DbgPrint("[UCRTDIFF] init ok (%d ran)\n", n);
    return 1;
}

static int ucrtdiff_main(void);

void ucrtdiff_entry(void);
void ucrtdiff_entry(void)
{
    int rc;
    DbgPrint("[UCRTDIFF] entry\n");
    if (!run_c_initializers()) {
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        ExitProcess(1);
    }
    rc = ucrtdiff_main();
    ExitProcess((UINT)rc);
}

static int ucrtdiff_main(void)
{
    DbgPrint("[UCRTDIFF] start\n");
    if (!AddVectoredExceptionHandler(1, ucrtdiff_veh)) {
        DbgPrint("[UCRTDIFF] FATAL: AddVectoredExceptionHandler failed\n");
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        return 1;
    }
    if (!guard_selftest()) {
        /* On i386 and amd64 this is EXPECTED and is why Tier 2 runs on arm64.
         * RtlDispatchException's RtlCallVectoredExceptionHandlers block is
         * `#ifndef _NTOSKRNL_`, and librtl always builds with -D_NTOSKRNL_, so the
         * first-chance VEH dispatch is dead code in the only variant ever built --
         * see sdk/lib/rust/rtl-except/src/{i386,amd64}/except.rs, which say so
         * outright. Only arm64 has it wired (proved by dll/ntdll/seh_poc_user.c:
         * "VEH on hardware AV: fired=1 resumed=1"). Without it one faulting body
         * ends the run and hides every export after it, so refuse to grade rather
         * than produce a table that silently stops partway. */
        DbgPrint("[UCRTDIFF] FATAL: the fault guard did not catch a deliberate fault. "
                 "VEH first-chance dispatch is only wired on arm64 (rtl-except); "
                 "run this payload with --arch arm64\n");
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        return 1;
    }
    DbgPrint("[UCRTDIFF] guard ok (caught 0x%08lx)\n", g_fault_code);
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
