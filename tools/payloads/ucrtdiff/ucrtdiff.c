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
#include "generated-refs.h" /* UCRTDIFF_REF_HAS_*: see HAND_GRADED in the generator */
/* The BARE side is declared by the real headers, so its prototypes cannot drift
 * from the ones every other caller sees. We are -nostdlib, so these resolve
 * from the ucrtbase import lib the link line names, not from the toolchain's. */
#include <ctype.h>
#include <stdlib.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* Locale entry points on the two renamed sides, so all three can be put into the same
 * state before grading. Declared by hand: these are not graded exports, so the
 * generator does not emit __typeof__ declarations for them. */
char *__cdecl ref_setlocale(int, const char *);
char *__cdecl rust_setlocale(int, const char *);
int __cdecl ref__setmbcp(int);
int __cdecl rust__setmbcp(int);
int __cdecl rust__getmbcp(void);
int __cdecl ref__getmbcp(void);

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
/* The rest of the strtox family (convert/strtox.cpp), graded two-sided by
 * run_strtox_family. Written with the widths Windows gives them: long is 32 bits,
 * __int64/intmax_t are long long, wchar_t is 16 bits. */
int *__cdecl ref__errno(void);
long __cdecl ref__strtol_l(const char *, char **, int, _locale_t);
unsigned long __cdecl ref__strtoul_l(const char *, char **, int, _locale_t);
long long __cdecl ref__strtoi64(const char *, char **, int);
long long __cdecl ref__strtoi64_l(const char *, char **, int, _locale_t);
long long __cdecl ref_strtoll(const char *, char **, int);
long long __cdecl ref__strtoll_l(const char *, char **, int, _locale_t);
long long __cdecl ref_strtoimax(const char *, char **, int);
long long __cdecl ref__strtoimax_l(const char *, char **, int, _locale_t);
unsigned long long __cdecl ref__strtoui64(const char *, char **, int);
unsigned long long __cdecl ref__strtoui64_l(const char *, char **, int, _locale_t);
unsigned long long __cdecl ref_strtoull(const char *, char **, int);
unsigned long long __cdecl ref__strtoull_l(const char *, char **, int, _locale_t);
unsigned long long __cdecl ref_strtoumax(const char *, char **, int);
unsigned long long __cdecl ref__strtoumax_l(const char *, char **, int, _locale_t);
long __cdecl ref_wcstol(const wchar_t *, wchar_t **, int);
long __cdecl ref__wcstol_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long __cdecl ref_wcstoul(const wchar_t *, wchar_t **, int);
unsigned long __cdecl ref__wcstoul_l(const wchar_t *, wchar_t **, int, _locale_t);
long long __cdecl ref__wcstoi64(const wchar_t *, wchar_t **, int);
long long __cdecl ref__wcstoi64_l(const wchar_t *, wchar_t **, int, _locale_t);
long long __cdecl ref_wcstoll(const wchar_t *, wchar_t **, int);
long long __cdecl ref__wcstoll_l(const wchar_t *, wchar_t **, int, _locale_t);
long long __cdecl ref_wcstoimax(const wchar_t *, wchar_t **, int);
long long __cdecl ref__wcstoimax_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long long __cdecl ref__wcstoui64(const wchar_t *, wchar_t **, int);
unsigned long long __cdecl ref__wcstoui64_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long long __cdecl ref_wcstoull(const wchar_t *, wchar_t **, int);
unsigned long long __cdecl ref__wcstoull_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long long __cdecl ref_wcstoumax(const wchar_t *, wchar_t **, int);
unsigned long long __cdecl ref__wcstoumax_l(const wchar_t *, wchar_t **, int, _locale_t);
/* The shipping side's spellings the toolchain headers may not declare. */
long long __cdecl _strtoimax_l(const char *, char **, int, _locale_t);
unsigned long long __cdecl _strtoumax_l(const char *, char **, int, _locale_t);
long long __cdecl _wcstoimax_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long long __cdecl _wcstoumax_l(const wchar_t *, wchar_t **, int, _locale_t);
long long __cdecl _strtoll_l(const char *, char **, int, _locale_t);
unsigned long long __cdecl _strtoull_l(const char *, char **, int, _locale_t);
long long __cdecl wcstoll(const wchar_t *, wchar_t **, int);
long long __cdecl _wcstoll_l(const wchar_t *, wchar_t **, int, _locale_t);
unsigned long long __cdecl wcstoull(const wchar_t *, wchar_t **, int);
unsigned long long __cdecl _wcstoull_l(const wchar_t *, wchar_t **, int, _locale_t);
long long __cdecl strtoimax(const char *, char **, int);
unsigned long long __cdecl strtoumax(const char *, char **, int);
long long __cdecl wcstoimax(const wchar_t *, wchar_t **, int);
unsigned long long __cdecl wcstoumax(const wchar_t *, wchar_t **, int);
/* convert/atox.cpp and convert/xtoa.cpp, graded two-sided by run_atox_family and
 * run_xtoa_family. The shipping-side spellings are declared too: the toolchain's
 * headers lack some (_wtoll, _atoll_l, ...), and a missing declaration is a build
 * error rather than a quietly ungraded row. */
typedef void(__cdecl *ucrtdiff_iph_t)(const wchar_t *, const wchar_t *, const wchar_t *,
                                      unsigned, uintptr_t);
ucrtdiff_iph_t __cdecl ref__set_invalid_parameter_handler(ucrtdiff_iph_t);
long long __cdecl _atoll_l(const char *, _locale_t);
long long __cdecl _wtoll(const wchar_t *);
long long __cdecl _wtoll_l(const wchar_t *, _locale_t);
int __cdecl ref__atoi_l(const char *, _locale_t);
long __cdecl ref__atol_l(const char *, _locale_t);
long long __cdecl ref_atoll(const char *);
long long __cdecl ref__atoll_l(const char *, _locale_t);
long long __cdecl ref__atoi64(const char *);
long long __cdecl ref__atoi64_l(const char *, _locale_t);
int __cdecl ref__wtoi(const wchar_t *);
int __cdecl ref__wtoi_l(const wchar_t *, _locale_t);
long __cdecl ref__wtol(const wchar_t *);
long __cdecl ref__wtol_l(const wchar_t *, _locale_t);
long long __cdecl ref__wtoll(const wchar_t *);
long long __cdecl ref__wtoll_l(const wchar_t *, _locale_t);
long long __cdecl ref__wtoi64(const wchar_t *);
long long __cdecl ref__wtoi64_l(const wchar_t *, _locale_t);
char *__cdecl ref__itoa(int, char *, int);
int __cdecl ref__itoa_s(int, char *, size_t, int);
char *__cdecl ref__ltoa(long, char *, int);
int __cdecl ref__ltoa_s(long, char *, size_t, int);
char *__cdecl ref__ultoa(unsigned long, char *, int);
int __cdecl ref__ultoa_s(unsigned long, char *, size_t, int);
char *__cdecl ref__i64toa(long long, char *, int);
int __cdecl ref__i64toa_s(long long, char *, size_t, int);
char *__cdecl ref__ui64toa(unsigned long long, char *, int);
int __cdecl ref__ui64toa_s(unsigned long long, char *, size_t, int);
wchar_t *__cdecl ref__itow(int, wchar_t *, int);
int __cdecl ref__itow_s(int, wchar_t *, size_t, int);
wchar_t *__cdecl ref__ltow(long, wchar_t *, int);
int __cdecl ref__ltow_s(long, wchar_t *, size_t, int);
wchar_t *__cdecl ref__ultow(unsigned long, wchar_t *, int);
int __cdecl ref__ultow_s(unsigned long, wchar_t *, size_t, int);
wchar_t *__cdecl ref__i64tow(long long, wchar_t *, int);
int __cdecl ref__i64tow_s(long long, wchar_t *, size_t, int);
wchar_t *__cdecl ref__ui64tow(unsigned long long, wchar_t *, int);
int __cdecl ref__ui64tow_s(unsigned long long, wchar_t *, size_t, int);
/* filesystem/findfile.cpp, graded two-sided against a real directory by
 * run_findfile_family. */
#include <io.h>
intptr_t __cdecl ref__findfirst32(const char *, struct _finddata32_t *);
intptr_t __cdecl ref__findfirst32i64(const char *, struct _finddata32i64_t *);
intptr_t __cdecl ref__findfirst64i32(const char *, struct _finddata64i32_t *);
intptr_t __cdecl ref__findfirst64(const char *, struct __finddata64_t *);
intptr_t __cdecl ref__wfindfirst32(const wchar_t *, struct _wfinddata32_t *);
intptr_t __cdecl ref__wfindfirst32i64(const wchar_t *, struct _wfinddata32i64_t *);
intptr_t __cdecl ref__wfindfirst64i32(const wchar_t *, struct _wfinddata64i32_t *);
intptr_t __cdecl ref__wfindfirst64(const wchar_t *, struct _wfinddata64_t *);
int __cdecl ref__findnext32(intptr_t, struct _finddata32_t *);
int __cdecl ref__findnext32i64(intptr_t, struct _finddata32i64_t *);
int __cdecl ref__findnext64i32(intptr_t, struct _finddata64i32_t *);
int __cdecl ref__findnext64(intptr_t, struct __finddata64_t *);
int __cdecl ref__wfindnext32(intptr_t, struct _wfinddata32_t *);
int __cdecl ref__wfindnext32i64(intptr_t, struct _wfinddata32i64_t *);
int __cdecl ref__wfindnext64i32(intptr_t, struct _wfinddata64i32_t *);
int __cdecl ref__wfindnext64(intptr_t, struct _wfinddata64_t *);
int __cdecl ref__findclose(intptr_t);
/* The SHIPPING side, called through its IMPORT POINTER. The toolchain's io.h
 * defines _findfirst64i32/_findnext64i32 as __CRT_INLINE wrappers over
 * _findfirst64, so a call by name grades mingw's inline, not ucrtbase -- pecheck's
 * import check caught it, and an assembler label on the name did not help, since
 * the label IS the inline's symbol and clang inlined it anyway. `__imp_<sym>`
 * exists only in the import table: there is no body to inline, so the call is
 * provably ucrtbase's. __USER_LABEL_PREFIX__ supplies i386's leading underscore. */
#define UD_STR2(x) #x
#define UD_STR(x) UD_STR2(x)
#define UD_IMP(n) "__imp_" UD_STR(__USER_LABEL_PREFIX__) #n
extern intptr_t(__cdecl *const ud__findfirst32)(const char *, struct _finddata32_t *) __asm__(UD_IMP(_findfirst32));
extern intptr_t(__cdecl *const ud__findfirst32i64)(const char *, struct _finddata32i64_t *) __asm__(UD_IMP(_findfirst32i64));
extern intptr_t(__cdecl *const ud__findfirst64i32)(const char *, struct _finddata64i32_t *) __asm__(UD_IMP(_findfirst64i32));
extern intptr_t(__cdecl *const ud__findfirst64)(const char *, struct __finddata64_t *) __asm__(UD_IMP(_findfirst64));
extern intptr_t(__cdecl *const ud__wfindfirst32)(const wchar_t *, struct _wfinddata32_t *) __asm__(UD_IMP(_wfindfirst32));
extern intptr_t(__cdecl *const ud__wfindfirst32i64)(const wchar_t *, struct _wfinddata32i64_t *) __asm__(UD_IMP(_wfindfirst32i64));
extern intptr_t(__cdecl *const ud__wfindfirst64i32)(const wchar_t *, struct _wfinddata64i32_t *) __asm__(UD_IMP(_wfindfirst64i32));
extern intptr_t(__cdecl *const ud__wfindfirst64)(const wchar_t *, struct _wfinddata64_t *) __asm__(UD_IMP(_wfindfirst64));
extern int(__cdecl *const ud__findnext32)(intptr_t, struct _finddata32_t *) __asm__(UD_IMP(_findnext32));
extern int(__cdecl *const ud__findnext32i64)(intptr_t, struct _finddata32i64_t *) __asm__(UD_IMP(_findnext32i64));
extern int(__cdecl *const ud__findnext64i32)(intptr_t, struct _finddata64i32_t *) __asm__(UD_IMP(_findnext64i32));
extern int(__cdecl *const ud__findnext64)(intptr_t, struct __finddata64_t *) __asm__(UD_IMP(_findnext64));
extern int(__cdecl *const ud__wfindnext32)(intptr_t, struct _wfinddata32_t *) __asm__(UD_IMP(_wfindnext32));
extern int(__cdecl *const ud__wfindnext32i64)(intptr_t, struct _wfinddata32i64_t *) __asm__(UD_IMP(_wfindnext32i64));
extern int(__cdecl *const ud__wfindnext64i32)(intptr_t, struct _wfinddata64i32_t *) __asm__(UD_IMP(_wfindnext64i32));
extern int(__cdecl *const ud__wfindnext64)(intptr_t, struct _wfinddata64_t *) __asm__(UD_IMP(_wfindnext64));
int __cdecl ref_atoi(const char *);
long __cdecl ref_atol(const char *);
int __cdecl ref_tolower(int);
int __cdecl ref_toupper(int);
int __cdecl ref_isalpha(int);
int __cdecl ref_isdigit(int);
int __cdecl ref_isspace(int);
int __cdecl ref_isupper(int);
int __cdecl ref_ispunct(int);
int __cdecl ref_strcmp(const char *, const char *);
char *__cdecl ref_strcat(char *, const char *);
char *__cdecl ref_strcpy(char *, const char *);
char *__cdecl ref_strncat(char *, const char *, size_t);
char *__cdecl ref_strncpy(char *, const char *, size_t);
char *__cdecl ref_strpbrk(const char *, const char *);
char *__cdecl rust_strcat(char *, const char *);
char *__cdecl rust_strcpy(char *, const char *);
char *__cdecl rust_strncat(char *, const char *, size_t);
char *__cdecl rust_strncpy(char *, const char *, size_t);
char *__cdecl rust_strpbrk(const char *, const char *);
int __cdecl ref__stricmp(const char *, const char *);
int __cdecl ref_strncmp(const char *, const char *, size_t);

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
int __cdecl rust_strcmp(const char *, const char *);
int __cdecl rust__stricmp(const char *, const char *);
int __cdecl rust_strncmp(const char *, const char *, size_t);

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

/* ---- write fencing for ABANDONED threads ------------------------------
 *
 * run_guarded abandons a hung body's thread rather than TerminateThread'ing it
 * (the hang-isolation note below says why, and that reasoning still holds). What
 * that note got WRONG was the cost: it claimed the worst case was "one spurious
 * FAULT, never a silent wrong verdict". In fact an abandoned thread keeps running
 * its grade loop and keeps writing the SHARED accounting -- g_ex_cases,
 * g_ex_dll_fail, g_ex_rust_fail, g_ex_first, g_cases and the pass/fail totals --
 * all of which export_begin resets for the NEXT export. So every row after the
 * first hang was racing a zombie, and which rows those were moved with emission
 * order. That is the instability, not a spurious FAULT.
 *
 * The fault budget was shared the same way. g_fault_hits is one global counter, so
 * a zombie faulting in a loop spent the allowance belonging to the export actually
 * being graded; at 4096 the handler returns CONTINUE_SEARCH, the exception goes
 * unhandled, and the process dies with the serial log simply STOPPING -- no FAULT
 * row, no HANG row, no `done`. That is exactly the signature of the wall.
 *
 * Fix: exactly one thread OWNS the accounting at a time, and writes from any other
 * thread are dropped and counted. The abandon design is kept intact (no CRT lock is
 * ever leaked); a zombie simply loses the ability to corrupt a verdict or end the
 * run. */
static volatile DWORD g_owner_tid;
static volatile int g_zombie_faults;  /* faults taken by abandoned threads */
static volatile int g_dropped_writes; /* grade() calls dropped from a zombie */

static int ucrtdiff_is_owner(void)
{
    return GetCurrentThreadId() == g_owner_tid;
}

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
    /* An ABANDONED thread must not spend the graded export's fault budget and must
     * not be able to end the run. Keep it limping, off the shared counters. */
    if (!ucrtdiff_is_owner()) {
        g_zombie_faults++;
#if defined(__aarch64__) || defined(_M_ARM64)
        ep->ContextRecord->Pc += 4;
        return EXCEPTION_CONTINUE_EXECUTION;
#else
        return EXCEPTION_CONTINUE_SEARCH;
#endif
    }
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

/* ---- hang isolation ---------------------------------------------------
 *
 * The fault guard handles a body that CRASHES. It cannot help with one that LOOPS,
 * and that is not hypothetical: `_mbsnbcoll` spins forever and stopped the first
 * complete arm64 sweep at export 114 of 224, hiding the other 110 exactly the way a
 * crash used to.
 *
 * So each export runs on its own thread and the driver waits with a timeout. On
 * timeout the thread is ABANDONED, never TerminateThread'd -- terminating a thread
 * mid-CRT leaks whatever locks it held, and the next export would then deadlock on
 * them and look like a second hang. A spinning thread that holds nothing costs only
 * its stack, and there are few of them.
 *
 * Two honest limits. If the hung body DOES hold a lock a later export needs, that
 * export hangs too and is reported HANG -- correctly, but the cause is upstream. And
 * an abandoned thread can still set g_fault_flag afterwards; the flag is cleared at
 * the start of every export, so the worst case is one spurious FAULT, never a silent
 * wrong verdict.
 */
#define UCRTDIFF_EXPORT_TIMEOUT_MS 8000

/* ---- sharding and the harness's own deadline --------------------------
 *
 * One boot used to be one shot at all 220 exports, so anything that stopped the
 * run cost every export after it -- and because the stopping point moved with
 * emission order (114 forward, 53 reversed; NOT a fixed export count and, measured
 * from the corpus products, not a fixed case count either: ~5,263 vs ~2,182), each
 * table we produced was some PREFIX of the truth and no two prefixes agreed.
 *
 * Sharding makes that structural problem go away without first having to name the
 * cause: shard k of N grades exports k, k+N, k+2N..., so a shard is a small run
 * that finishes well inside any of the observed walls, and N boots cover the set.
 * Interleaving rather than blocking is deliberate -- a contiguous block would put
 * all of one family (and so all of one failure mode) in one shard.
 *
 * The deadline is the other half, and it is the FAIL-LOUD half. Whatever ends a
 * long run today does so by killing the guest, which is indistinguishable on serial
 * from a wedge: the log just stops, with no `done`. Owning our own deadline means a
 * truncated sweep prints a `done` line that SAYS it was truncated and how far it
 * got, so a short run is a parseable partial result instead of silence. */
#ifndef UCRTDIFF_SHARD_K
#define UCRTDIFF_SHARD_K 0
#endif
#ifndef UCRTDIFF_SHARD_N
#define UCRTDIFF_SHARD_N 1
#endif
#ifndef UCRTDIFF_BUDGET_MS
#define UCRTDIFF_BUDGET_MS 600000 /* 10 min of grading, well inside the VM's life */
#endif

#define UCRTDIFF_OK 0
#define UCRTDIFF_FAULT 1
#define UCRTDIFF_HANG 2

static DWORD WINAPI ucrtdiff_job(LPVOID p)
{
    void (*fn)(void) = (void (*)(void))p;
    g_owner_tid = GetCurrentThreadId();
    g_fault_flag = 0;
    g_fault_hits = 0;
    g_fault_armed = 1;
    fn();
    g_fault_armed = 0;
    return 0;
}

static int run_guarded(void (*fn)(void))
{
    HANDLE h;
    DWORD r;
    g_fault_flag = 0;
    g_fault_hits = 0;
    h = CreateThread(NULL, 0, ucrtdiff_job, (LPVOID)fn, 0, NULL);
    if (!h) {
        DbgPrint("[UCRTDIFF] FATAL: CreateThread failed err=%lu\n", GetLastError());
        return UCRTDIFF_FAULT;
    }
    r = WaitForSingleObject(h, UCRTDIFF_EXPORT_TIMEOUT_MS);
    CloseHandle(h);
    /* Take the accounting back on the driver thread. If the job TIMED OUT it is
     * still running and still thinks it owns the counters -- this one store is what
     * stops it corrupting every export that follows. */
    g_owner_tid = GetCurrentThreadId();
    if (r == WAIT_TIMEOUT)
        return UCRTDIFF_HANG;
    return g_fault_flag ? UCRTDIFF_FAULT : UCRTDIFF_OK;
}

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

/* Position in the FULL export list, counted for EVERY export whether this shard
 * grades it or not -- the shard split must not depend on what ran. */
static int g_ex_index;
static int g_ran_exports, g_skip_shard, g_skip_budget;
static DWORD g_deadline_tick;

/* Returns 0 when this export is not ours to run; the generated driver then skips
 * straight past it. Both reasons are counted so the `done` line can distinguish a
 * shard that finished from a run that was cut short. */
static int export_begin(const char *name)
{
    int idx = g_ex_index++;
    if ((idx % UCRTDIFF_SHARD_N) != UCRTDIFF_SHARD_K) {
        g_skip_shard++;
        return 0;
    }
    /* GetTickCount, not GetTickCount64: the 49-day wrap cannot be reached inside a
     * boot, and it is present on every arch's kernel32 here. */
    if ((long)(GetTickCount() - g_deadline_tick) >= 0) {
        g_skip_budget++;
        return 0;
    }
    g_ex_name = name;
    g_ex_cases = g_ex_dll_fail = g_ex_rust_fail = 0;
    g_ex_first[0] = 0;
    g_ran_exports++;
    return 1;
}

static void export_end(int outcome, int has_rust)
{
    if (outcome == UCRTDIFF_HANG) {
        DbgPrint("[UCRTDIFF] EXPORT %s HANG after=%d (%dms)\n",
                 g_ex_name, g_ex_cases, UCRTDIFF_EXPORT_TIMEOUT_MS);
        return;
    }
    if (outcome == UCRTDIFF_FAULT) {
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
    if (!ucrtdiff_is_owner()) { g_dropped_writes++; return; }
    g_cases++;
    g_ex_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        g_ex_dll_fail++;
        note_first(family, what);
        /* The verdict line names only the first failing case; the values are what
         * make a DIFF actionable. A few per export, so a broken row cannot flood. */
        if (g_ex_dll_fail <= 6)
            DbgPrint("[UCRTDIFF]   diff %s %s dll=%lld ref=%lld\n", family, what, dll, ref);
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
    if (!ucrtdiff_is_owner()) { g_dropped_writes++; return; }
    g_cases++;
    g_ex_cases++;
    if (dll == ref) {
        g_dll_pass++;
    } else {
        g_dll_fail++;
        g_ex_dll_fail++;
        note_first(family, what);
        if (g_ex_dll_fail <= 6)
            DbgPrint("[UCRTDIFF]   diff %s %s dll=%lld ref=%lld\n", family, what, dll, ref);
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
#if UCRTDIFF_REF_HAS_strtol
            e1 = e2 = e3 = 0;
            grade(with_base("strtol", bases[b]), in[i], strtol(in[i], &e1, bases[b]),
                  rust_strtol(in[i], &e3, bases[b]), ref_strtol(in[i], &e2, bases[b]));
            /* The end pointer is half the contract and is where a rewrite most
             * often drifts -- a body can return the right number having consumed
             * the wrong number of characters. */
            grade(with_base("strtol.end", bases[b]), in[i], off(in[i], e1), off(in[i], e3), off(in[i], e2));
#endif
#if UCRTDIFF_REF_HAS_strtoul
            e1 = e2 = e3 = 0;
            grade(with_base("strtoul", bases[b]), in[i], (long long)strtoul(in[i], &e1, bases[b]),
                  (long long)rust_strtoul(in[i], &e3, bases[b]),
                  (long long)ref_strtoul(in[i], &e2, bases[b]));
            grade(with_base("strtoul.end", bases[b]), in[i], off(in[i], e1), off(in[i], e3), off(in[i], e2));
#endif
        }
#if UCRTDIFF_REF_HAS_atoi
        grade("atoi", in[i], atoi(in[i]), rust_atoi(in[i]), ref_atoi(in[i]));
#endif
#if UCRTDIFF_REF_HAS_atol
        grade("atol", in[i], atol(in[i]), rust_atol(in[i]), ref_atol(in[i]));
#endif
    }
}

/* The WHOLE strtox family, two-sided (shipping DLL vs Microsoft): value, end
 * offset and errno, at the same six bases, over the legacy inputs plus the edges
 * of the 64-bit types. The wide spellings get the same strings widened plus the
 * cases only a wide parser has: Unicode decimal digits (fullwidth, Arabic-Indic),
 * a fullwidth LETTER that must not parse, and U+3000 as leading whitespace.
 * The _l forms are called with a NULL locale -- the current one. */
static const char *const g_strtox_in[] = {
    "42", "-42", "0", "  \t 17", "+99", "0x1f", "2147483647", "-2147483648",
    "2147483648", "-2147483649", "4294967295", "4294967296", "-4294967295",
    "999999999999999", "", "   ", "abc", "12abc", "0x", "0X", "-", "+", "0b101",
    "  -0X7FfF", "007", "z", "Zz", "1_000", "\t\n\v\f\r 5", "-0",
    "9223372036854775807", "-9223372036854775808", "9223372036854775808",
    "-9223372036854775809", "18446744073709551615", "18446744073709551616",
    "-18446744073709551615", "0xFFFFFFFFFFFFFFFF", "0x10000000000000000",
};
static const wchar_t *const g_strtox_win[] = {
    L"\xFF11\xFF12", L"-\xFF19", L"\x0663\x0664", L"\x06F7", L"\x3000 42", L"\xFF21",
    L"\x00B2", L"4\xFF15", L"0\xFF58" L"1",
};
#define STRTOX_N (sizeof(g_strtox_in) / sizeof(g_strtox_in[0]))
#define STRTOX_WN (sizeof(g_strtox_win) / sizeof(g_strtox_win[0]))

#define STRTOX_ROW(fn, CH, s, lbl, base, ...) do {                                  \
        CH *e1 = 0, *e2 = 0;                                                        \
        *_errno() = 0; *ref__errno() = 0;                                           \
        long long v1 = (long long)fn(s, &e1, base, ##__VA_ARGS__);                  \
        long long v2 = (long long)ref_##fn(s, &e2, base, ##__VA_ARGS__);            \
        int n1 = *_errno(), n2 = *ref__errno();                                     \
        grade2(with_base(#fn, base), lbl, v1, v2);                                  \
        grade2(with_base(#fn ".end", base), lbl, e1 ? (long long)(e1 - s) : -1,     \
               e2 ? (long long)(e2 - s) : -1);                                      \
        grade2(with_base(#fn ".errno", base), lbl, n1, n2);                         \
    } while (0)

#define STRTOX_NARROW(fn, ...)                                                       \
    for (unsigned i = 0; i < STRTOX_N; i++)                                          \
        for (unsigned b = 0; b < 6; b++)                                             \
            STRTOX_ROW(fn, char, g_strtox_in[i], g_strtox_in[i], bases[b], ##__VA_ARGS__)

#define STRTOX_WIDE(fn, ...)                                                         \
    for (unsigned i = 0; i < STRTOX_N + STRTOX_WN; i++) {                            \
        wchar_t w[48]; const wchar_t *ws;                                            \
        if (i < STRTOX_N) {                                                          \
            unsigned k = 0;                                                          \
            for (; g_strtox_in[i][k] && k < 47; k++) w[k] = (unsigned char)g_strtox_in[i][k]; \
            w[k] = 0; ws = w;                                                        \
        } else {                                                                     \
            ws = g_strtox_win[i - STRTOX_N];                                         \
        }                                                                            \
        for (unsigned b = 0; b < 6; b++)                                             \
            STRTOX_ROW(fn, wchar_t, ws, label_idx(i), bases[b], ##__VA_ARGS__);      \
    }

static void run_strtox_family(void)
{
    static const int bases[] = { 0, 8, 10, 16, 36, 2 };
    (void)bases;
#if UCRTDIFF_REF_HAS__strtol_l
    STRTOX_NARROW(_strtol_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__strtoul_l
    STRTOX_NARROW(_strtoul_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__strtoi64
    STRTOX_NARROW(_strtoi64);
#endif
#if UCRTDIFF_REF_HAS__strtoi64_l
    STRTOX_NARROW(_strtoi64_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_strtoll
    STRTOX_NARROW(strtoll);
#endif
#if UCRTDIFF_REF_HAS__strtoll_l
    STRTOX_NARROW(_strtoll_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_strtoimax
    STRTOX_NARROW(strtoimax);
#endif
#if UCRTDIFF_REF_HAS__strtoimax_l
    STRTOX_NARROW(_strtoimax_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__strtoui64
    STRTOX_NARROW(_strtoui64);
#endif
#if UCRTDIFF_REF_HAS__strtoui64_l
    STRTOX_NARROW(_strtoui64_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_strtoull
    STRTOX_NARROW(strtoull);
#endif
#if UCRTDIFF_REF_HAS__strtoull_l
    STRTOX_NARROW(_strtoull_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_strtoumax
    STRTOX_NARROW(strtoumax);
#endif
#if UCRTDIFF_REF_HAS__strtoumax_l
    STRTOX_NARROW(_strtoumax_l, NULL);
#endif
}

/* Split from the narrow half only to keep each legacy export well inside the
 * per-export hang timeout (UCRTDIFF_EXPORT_TIMEOUT_MS). */
static void run_wcstox_family(void)
{
    static const int bases[] = { 0, 8, 10, 16, 36, 2 };
    (void)bases;
#if UCRTDIFF_REF_HAS_wcstol
    STRTOX_WIDE(wcstol);
#endif
#if UCRTDIFF_REF_HAS__wcstol_l
    STRTOX_WIDE(_wcstol_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_wcstoul
    STRTOX_WIDE(wcstoul);
#endif
#if UCRTDIFF_REF_HAS__wcstoul_l
    STRTOX_WIDE(_wcstoul_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wcstoi64
    STRTOX_WIDE(_wcstoi64);
#endif
#if UCRTDIFF_REF_HAS__wcstoi64_l
    STRTOX_WIDE(_wcstoi64_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_wcstoll
    STRTOX_WIDE(wcstoll);
#endif
#if UCRTDIFF_REF_HAS__wcstoll_l
    STRTOX_WIDE(_wcstoll_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_wcstoimax
    STRTOX_WIDE(wcstoimax);
#endif
#if UCRTDIFF_REF_HAS__wcstoimax_l
    STRTOX_WIDE(_wcstoimax_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wcstoui64
    STRTOX_WIDE(_wcstoui64);
#endif
#if UCRTDIFF_REF_HAS__wcstoui64_l
    STRTOX_WIDE(_wcstoui64_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_wcstoull
    STRTOX_WIDE(wcstoull);
#endif
#if UCRTDIFF_REF_HAS__wcstoull_l
    STRTOX_WIDE(_wcstoull_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_wcstoumax
    STRTOX_WIDE(wcstoumax);
#endif
#if UCRTDIFF_REF_HAS__wcstoumax_l
    STRTOX_WIDE(_wcstoumax_l, NULL);
#endif
}

/* ---- atox / xtoa ------------------------------------------------------- */

/* The _s forms' error paths call the invalid-parameter handler, and the default
 * one fast-fails the process. Both CRT instances -- the shipping DLL and the
 * reference set's private copy -- get a handler that only COUNTS, and the count is
 * graded like any other output: whether a call raises the handler is contract. */
static volatile long g_iph_dll, g_iph_ref;
static void __cdecl iph_dll(const wchar_t *e, const wchar_t *f, const wchar_t *fl, unsigned l,
                            uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_iph_dll++; }
static void __cdecl iph_ref(const wchar_t *e, const wchar_t *f, const wchar_t *fl, unsigned l,
                            uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_iph_ref++; }
static void install_counting_iph(void)
{
    _set_invalid_parameter_handler(iph_dll);
    ref__set_invalid_parameter_handler(iph_ref);
}

#define ATOX_ROW(fn, s, lbl, ...) do {                                              \
        *_errno() = 0; *ref__errno() = 0;                                           \
        long long v1 = (long long)fn(s, ##__VA_ARGS__);                             \
        long long v2 = (long long)ref_##fn(s, ##__VA_ARGS__);                       \
        int n1 = *_errno(), n2 = *ref__errno();                                     \
        grade2(#fn, lbl, v1, v2);                                                   \
        grade2(#fn ".errno", lbl, n1, n2);                                          \
    } while (0)

#define ATOX_NARROW(fn, ...)                                                         \
    for (unsigned i = 0; i < STRTOX_N; i++)                                          \
        ATOX_ROW(fn, g_strtox_in[i], g_strtox_in[i], ##__VA_ARGS__)

#define ATOX_WIDE(fn, ...)                                                           \
    for (unsigned i = 0; i < STRTOX_N + STRTOX_WN; i++) {                            \
        wchar_t w[48]; const wchar_t *ws;                                            \
        if (i < STRTOX_N) {                                                          \
            unsigned k = 0;                                                          \
            for (; g_strtox_in[i][k] && k < 47; k++) w[k] = (unsigned char)g_strtox_in[i][k]; \
            w[k] = 0; ws = w;                                                        \
        } else {                                                                     \
            ws = g_strtox_win[i - STRTOX_N];                                         \
        }                                                                            \
        ATOX_ROW(fn, ws, label_idx(i), ##__VA_ARGS__);                               \
    }

static void run_atox_family(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_HAS_atoi
    ATOX_NARROW(atoi);
#endif
#if UCRTDIFF_REF_HAS__atoi_l
    ATOX_NARROW(_atoi_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_atol
    ATOX_NARROW(atol);
#endif
#if UCRTDIFF_REF_HAS__atol_l
    ATOX_NARROW(_atol_l, NULL);
#endif
#if UCRTDIFF_REF_HAS_atoll
    ATOX_NARROW(atoll);
#endif
#if UCRTDIFF_REF_HAS__atoll_l
    ATOX_NARROW(_atoll_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__atoi64
    ATOX_NARROW(_atoi64);
#endif
#if UCRTDIFF_REF_HAS__atoi64_l
    ATOX_NARROW(_atoi64_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wtoi
    ATOX_WIDE(_wtoi);
#endif
#if UCRTDIFF_REF_HAS__wtoi_l
    ATOX_WIDE(_wtoi_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wtol
    ATOX_WIDE(_wtol);
#endif
#if UCRTDIFF_REF_HAS__wtol_l
    ATOX_WIDE(_wtol_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wtoll
    ATOX_WIDE(_wtoll);
#endif
#if UCRTDIFF_REF_HAS__wtoll_l
    ATOX_WIDE(_wtoll_l, NULL);
#endif
#if UCRTDIFF_REF_HAS__wtoi64
    ATOX_WIDE(_wtoi64);
#endif
#if UCRTDIFF_REF_HAS__wtoi64_l
    ATOX_WIDE(_wtoi64_l, NULL);
#endif
    /* NULL: the invalid-parameter path, EINVAL, and 0. */
#if UCRTDIFF_REF_HAS_atoi
    g_iph_dll = g_iph_ref = 0;
    ATOX_ROW(atoi, (const char *)0, "NULL");
    grade2("atoi.iph", "NULL", g_iph_dll, g_iph_ref);
#endif
#if UCRTDIFF_REF_HAS__wtoi64
    g_iph_dll = g_iph_ref = 0;
    ATOX_ROW(_wtoi64, (const wchar_t *)0, "NULL");
    grade2("_wtoi64.iph", "NULL", g_iph_dll, g_iph_ref);
#endif
}

/* FNV-1a over the WHOLE buffer, not just up to the NUL: a too-small _s call leaves
 * digits after the cleared buffer[0], and that is observable. */
static long long fnv(const void *p, size_t n)
{
    const unsigned char *b = (const unsigned char *)p;
    unsigned long long h = 1469598103934665603ULL;
    while (n--) { h ^= *b++; h *= 1099511628211ULL; }
    return (long long)h;
}

static const long long g_xtoa_vals[] = {
    0, 1, -1, 7, 9, 10, 35, 36, 255, -255, 2147483647LL, -2147483647LL - 1,
    2147483648LL, 4294967295LL, 1234567890, -1234567890, 0x7FFFFFFFFFFFFFFFLL,
    -0x7FFFFFFFFFFFFFFFLL - 1, (long long)0xDEADBEEFCAFEBABEULL, 1000000007LL,
};
#define XTOA_VN (sizeof(g_xtoa_vals) / sizeof(g_xtoa_vals[0]))
static const int g_xtoa_radix[] = { 2, 8, 10, 16, 36, 3 };
/* _s only: invalid radices are part of its contract. The plain forms never check
 * the radix -- 0 divides by zero -- so they are not asked. */
static const int g_xtoa_radix_s[] = { 2, 10, 16, 36, 0, 1, 37 };
static const size_t g_xtoa_counts[] = { 0, 1, 2, 3, 5, 9, 11, 33, 66 };

#define XTOA_PLAIN(ch, T, fn) do {                                                  \
        for (unsigned i = 0; i < XTOA_VN; i++)                                      \
            for (unsigned r = 0; r < 6; r++) {                                      \
                ch b1[80], b2[80];                                                  \
                for (unsigned k = 0; k < 80; k++) b1[k] = b2[k] = (ch)'X';          \
                ch *p1 = fn((T)g_xtoa_vals[i], b1, g_xtoa_radix[r]);                \
                ch *p2 = ref_##fn((T)g_xtoa_vals[i], b2, g_xtoa_radix[r]);          \
                grade2(with_base(#fn, g_xtoa_radix[r]), label_idx(i),               \
                       fnv(b1, sizeof b1), fnv(b2, sizeof b2));                     \
                grade2(with_base(#fn ".ret", g_xtoa_radix[r]), label_idx(i),        \
                       p1 == b1, p2 == b2);                                         \
            }                                                                       \
    } while (0)

#define XTOA_S(ch, T, fn) do {                                                      \
        for (unsigned i = 0; i < XTOA_VN; i++)                                      \
            for (unsigned r = 0; r < 7; r++)                                        \
                for (unsigned c = 0; c < 9; c++) {                                  \
                    ch b1[80], b2[80];                                              \
                    for (unsigned k = 0; k < 80; k++) b1[k] = b2[k] = (ch)'X';      \
                    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;    \
                    int r1 = fn((T)g_xtoa_vals[i], b1, g_xtoa_counts[c], g_xtoa_radix_s[r]); \
                    int r2 = ref_##fn((T)g_xtoa_vals[i], b2, g_xtoa_counts[c], g_xtoa_radix_s[r]); \
                    const char *lbl = label_idx(i * 10 + c);                       \
                    grade2(with_base(#fn, g_xtoa_radix_s[r]), lbl,                   \
                           fnv(b1, sizeof b1), fnv(b2, sizeof b2));                 \
                    grade2(with_base(#fn ".ret", g_xtoa_radix_s[r]), lbl, r1, r2);  \
                    grade2(with_base(#fn ".errno", g_xtoa_radix_s[r]), lbl,         \
                           *_errno(), *ref__errno());                               \
                    grade2(with_base(#fn ".iph", g_xtoa_radix_s[r]), lbl,           \
                           g_iph_dll, g_iph_ref);                                   \
                }                                                                   \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;                \
        int r1 = fn((T)1, (ch *)0, 10, 10);                                         \
        int r2 = ref_##fn((T)1, (ch *)0, 10, 10);                                   \
        grade2(#fn ".null.ret", "NULL", r1, r2);                                    \
        grade2(#fn ".null.errno", "NULL", *_errno(), *ref__errno());                \
        grade2(#fn ".null.iph", "NULL", g_iph_dll, g_iph_ref);                      \
    } while (0)

static void run_xtoa_family(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_HAS__itoa
    XTOA_PLAIN(char, int, _itoa);
#endif
#if UCRTDIFF_REF_HAS__ltoa
    XTOA_PLAIN(char, long, _ltoa);
#endif
#if UCRTDIFF_REF_HAS__ultoa
    XTOA_PLAIN(char, unsigned long, _ultoa);
#endif
#if UCRTDIFF_REF_HAS__i64toa
    XTOA_PLAIN(char, long long, _i64toa);
#endif
#if UCRTDIFF_REF_HAS__ui64toa
    XTOA_PLAIN(char, unsigned long long, _ui64toa);
#endif
#if UCRTDIFF_REF_HAS__itow
    XTOA_PLAIN(wchar_t, int, _itow);
#endif
#if UCRTDIFF_REF_HAS__ltow
    XTOA_PLAIN(wchar_t, long, _ltow);
#endif
#if UCRTDIFF_REF_HAS__ultow
    XTOA_PLAIN(wchar_t, unsigned long, _ultow);
#endif
#if UCRTDIFF_REF_HAS__i64tow
    XTOA_PLAIN(wchar_t, long long, _i64tow);
#endif
#if UCRTDIFF_REF_HAS__ui64tow
    XTOA_PLAIN(wchar_t, unsigned long long, _ui64tow);
#endif
}

/* Split from the plain forms only to keep each legacy export well inside
 * UCRTDIFF_EXPORT_TIMEOUT_MS: 10 exports x 20 values x 7 radices x 9 counts. */
static void run_xtoa_s_family(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_HAS__itoa_s
    XTOA_S(char, int, _itoa_s);
#endif
#if UCRTDIFF_REF_HAS__ltoa_s
    XTOA_S(char, long, _ltoa_s);
#endif
#if UCRTDIFF_REF_HAS__ultoa_s
    XTOA_S(char, unsigned long, _ultoa_s);
#endif
#if UCRTDIFF_REF_HAS__i64toa_s
    XTOA_S(char, long long, _i64toa_s);
#endif
#if UCRTDIFF_REF_HAS__ui64toa_s
    XTOA_S(char, unsigned long long, _ui64toa_s);
#endif
#if UCRTDIFF_REF_HAS__itow_s
    XTOA_S(wchar_t, int, _itow_s);
#endif
#if UCRTDIFF_REF_HAS__ltow_s
    XTOA_S(wchar_t, long, _ltow_s);
#endif
#if UCRTDIFF_REF_HAS__ultow_s
    XTOA_S(wchar_t, unsigned long, _ultow_s);
#endif
#if UCRTDIFF_REF_HAS__i64tow_s
    XTOA_S(wchar_t, long long, _i64tow_s);
#endif
#if UCRTDIFF_REF_HAS__ui64tow_s
    XTOA_S(wchar_t, unsigned long long, _ui64tow_s);
#endif
}

/* ---- findfile ------------------------------------------------------------
 *
 * A real directory, built fresh: an empty file, a 1234-byte file dated 2001, a
 * read-only file, a hidden one with spaces in its name, a subdirectory, and a file
 * dated 2040 -- past the 32-bit time_t, so the __time32_t forms must answer -1
 * for it while the 64-bit forms answer the date. Both sides enumerate the same
 * directory; each record is compared byte for byte (both prefilled 'X', so bytes
 * the C leaves alone must be left alone), and each side closes its OWN handles
 * with its own _findclose. A fixture that cannot be built is a failing row, never
 * a family that silently grades nothing. */
static wchar_t g_ff_dir[MAX_PATH];

static int ff_make_file(const wchar_t *name, DWORD size, int year, DWORD attrs)
{
    wchar_t path[MAX_PATH];
    lstrcpyW(path, g_ff_dir); lstrcatW(path, L"\\"); lstrcatW(path, name);
    SetFileAttributesW(path, FILE_ATTRIBUTE_NORMAL); /* a previous run's read-only */
    HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) {
        DbgPrint("[UCRTDIFF] findfile fixture: CreateFileW(%ls) failed, error %lu\n", path,
                 GetLastError());
        return 0;
    }
    static char zeros[1234];
    DWORD wrote = 0;
    if (size && !WriteFile(h, zeros, size, &wrote, NULL)) {
        DbgPrint("[UCRTDIFF] findfile fixture: WriteFile(%ls) failed, error %lu\n", path,
                 GetLastError());
        CloseHandle(h);
        return 0;
    }
    if (year) {
        SYSTEMTIME st = { (WORD)year, 6, 0, 15, 12, 30, 44, 0 };
        FILETIME ft;
        SystemTimeToFileTime(&st, &ft);
        if (!SetFileTime(h, &ft, &ft, &ft))
            DbgPrint("[UCRTDIFF] findfile fixture: SetFileTime(%ls) failed, error %lu (graded anyway)\n",
                     path, GetLastError());
    }
    CloseHandle(h);
    if (attrs != FILE_ATTRIBUTE_NORMAL)
        SetFileAttributesW(path, attrs);
    return 1;
}

/* Build the fixture under the first base directory that takes it: %TEMP%, the
 * root of C:, the directory holding this exe. Each failure is logged with its
 * Win32 error, so a fixture that cannot be built says WHY on serial. */
static int ff_try_base(const wchar_t *base)
{
    lstrcpyW(g_ff_dir, base);
    size_t n = (size_t)lstrlenW(g_ff_dir);
    if (n && g_ff_dir[n - 1] != L'\\')
        lstrcatW(g_ff_dir, L"\\");
    lstrcatW(g_ff_dir, L"ucrtdiff_ff");
    if (!CreateDirectoryW(g_ff_dir, NULL) && GetLastError() != ERROR_ALREADY_EXISTS) {
        DbgPrint("[UCRTDIFF] findfile fixture: CreateDirectoryW(%ls) failed, error %lu\n",
                 g_ff_dir, GetLastError());
        return 0;
    }
    wchar_t sub[MAX_PATH];
    lstrcpyW(sub, g_ff_dir); lstrcatW(sub, L"\\sub");
    CreateDirectoryW(sub, NULL);
    int ok = ff_make_file(L"a.txt", 0, 0, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"b.bin", 1234, 2001, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"c.txt", 7, 0, FILE_ATTRIBUTE_READONLY)
        && ff_make_file(L"e with space.txt", 3, 0, FILE_ATTRIBUTE_HIDDEN)
        && ff_make_file(L"f_2040.dat", 1, 2040, FILE_ATTRIBUTE_NORMAL);
    if (ok)
        DbgPrint("[UCRTDIFF] findfile fixture: built under %ls\n", g_ff_dir);
    return ok;
}

static int ff_setup(void)
{
    wchar_t base[MAX_PATH];
    DWORD n = GetTempPathW(MAX_PATH - 32, base);
    if (n && n < MAX_PATH - 32 && ff_try_base(base))
        return 1;
    if (ff_try_base(L"C:\\"))
        return 1;
    n = GetModuleFileNameW(NULL, base, MAX_PATH - 32);
    if (n && n < MAX_PATH - 32) {
        wchar_t *slash = NULL;
        for (wchar_t *p = base; *p; p++)
            if (*p == L'\\')
                slash = p;
        if (slash) {
            *slash = 0;
            if (ff_try_base(base))
                return 1;
        }
    }
    return 0;
}

#define FF_RECORD(pfx, lbl, ok1, ok2, d1, d2) do {                                 \
        grade2(pfx ".ok", lbl, ok1, ok2);                                          \
        grade2(pfx ".errno", lbl, *_errno(), *ref__errno());                       \
        grade2(pfx ".iph", lbl, g_iph_dll, g_iph_ref);                             \
        if ((ok1) && (ok2))                                                        \
            grade2(pfx ".rec", lbl, fnv(&(d1), sizeof(d1)), fnv(&(d2), sizeof(d2))); \
    } while (0)

#define FF_RESET(d1, d2) do {                                                      \
        memset(&(d1), 'X', sizeof(d1)); memset(&(d2), 'X', sizeof(d2));            \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;               \
    } while (0)

/* One pattern through one first/next pair: first, then next until either side
 * stops, then close. */
#define FF_WALK(T, first, next, pat, lbl) do {                                     \
        struct T d1, d2;                                                           \
        FF_RESET(d1, d2);                                                          \
        intptr_t h1 = ud_##first(pat, &d1), h2 = ref_##first(pat, &d2);                 \
        FF_RECORD(#first, lbl, h1 != -1, h2 != -1, d1, d2);                        \
        for (int step = 0; h1 != -1 && h2 != -1 && step < 32; step++) {            \
            FF_RESET(d1, d2);                                                      \
            int r1 = ud_##next(h1, &d1), r2 = ref_##next(h2, &d2);                      \
            FF_RECORD(#next, lbl, r1 == 0, r2 == 0, d1, d2);                       \
            if (r1 != 0 || r2 != 0)                                                \
                break;                                                             \
        }                                                                          \
        *_errno() = 0; *ref__errno() = 0;                                          \
        int c1 = h1 != -1 ? _findclose(h1) : -2;                                   \
        int c2 = h2 != -1 ? ref__findclose(h2) : -2;                               \
        grade2("_findclose", lbl, c1, c2);                                         \
        grade2("_findclose.errno", lbl, *_errno(), *ref__errno());                 \
    } while (0)

/* The argument-validation paths: NULL record, and _findnext on handles 0/-1. */
#define FF_INVALID(T, first, next, pat) do {                                       \
        struct T d1, d2;                                                           \
        FF_RESET(d1, d2);                                                          \
        intptr_t h1 = ud_##first(pat, (struct T *)0), h2 = ref_##first(pat, (struct T *)0); \
        FF_RECORD(#first ".nullrec", "-", h1 != -1, h2 != -1, d1, d2);             \
        FF_RESET(d1, d2);                                                          \
        int r1 = ud_##next(0, &d1), r2 = ref_##next(0, &d2);                            \
        FF_RECORD(#next ".h0", "-", r1 == 0, r2 == 0, d1, d2);                     \
        FF_RESET(d1, d2);                                                          \
        r1 = ud_##next(-1, &d1); r2 = ref_##next(-1, &d2);                              \
        FF_RECORD(#next ".hm1", "-", r1 == 0, r2 == 0, d1, d2);                    \
    } while (0)

static void run_findfile_family(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_HAS__findfirst32
    if (!ff_setup()) {
        grade2("findfile.fixture", "setup", 0, 1); /* fail loud */
        return;
    }
    static const wchar_t *const tails[] = {
        L"\\*", L"\\*.txt", L"\\b.bin", L"\\f_2040.dat", L"\\nomatch*", L"\\sub",
    };
    enum { NT = sizeof(tails) / sizeof(tails[0]) };
    for (int t = 0; t < NT + 3; t++) {
        wchar_t wp[MAX_PATH];
        char np[MAX_PATH * 2];
        const wchar_t *w = wp;
        const char *n = np;
        if (t < NT) {
            lstrcpyW(wp, g_ff_dir); lstrcatW(wp, tails[t]);
        } else if (t == NT) {
            lstrcpyW(wp, L"C:\\ucrtdiff_no_such_dir\\*");
        } else if (t == NT + 1) {
            wp[0] = 0;
        } else {
            w = 0; /* NULL pattern */
        }
        if (w)
            WideCharToMultiByte(CP_ACP, 0, w, -1, np, sizeof np, NULL, NULL);
        else
            n = 0;
        const char *lbl = label_idx((size_t)t);
        FF_WALK(_finddata32_t, _findfirst32, _findnext32, n, lbl);
        FF_WALK(_finddata32i64_t, _findfirst32i64, _findnext32i64, n, lbl);
        FF_WALK(_finddata64i32_t, _findfirst64i32, _findnext64i32, n, lbl);
        FF_WALK(__finddata64_t, _findfirst64, _findnext64, n, lbl);
        FF_WALK(_wfinddata32_t, _wfindfirst32, _wfindnext32, w, lbl);
        FF_WALK(_wfinddata32i64_t, _wfindfirst32i64, _wfindnext32i64, w, lbl);
        FF_WALK(_wfinddata64i32_t, _wfindfirst64i32, _wfindnext64i32, w, lbl);
        FF_WALK(_wfinddata64_t, _wfindfirst64, _wfindnext64, w, lbl);
    }
    {
        char np[MAX_PATH * 2];
        wchar_t wp[MAX_PATH];
        lstrcpyW(wp, g_ff_dir); lstrcatW(wp, L"\\*");
        WideCharToMultiByte(CP_ACP, 0, wp, -1, np, sizeof np, NULL, NULL);
        FF_INVALID(_finddata32_t, _findfirst32, _findnext32, np);
        FF_INVALID(_finddata32i64_t, _findfirst32i64, _findnext32i64, np);
        FF_INVALID(_finddata64i32_t, _findfirst64i32, _findnext64i32, np);
        FF_INVALID(__finddata64_t, _findfirst64, _findnext64, np);
        FF_INVALID(_wfinddata32_t, _wfindfirst32, _wfindnext32, wp);
        FF_INVALID(_wfinddata32i64_t, _wfindfirst32i64, _wfindnext32i64, wp);
        FF_INVALID(_wfinddata64i32_t, _wfindfirst64i32, _wfindnext64i32, wp);
        FF_INVALID(_wfinddata64_t, _wfindfirst64, _wfindnext64, wp);
    }
#endif
}

/* ---- output (stdio/output.cpp) -----------------------------------------
 *
 * The fourteen __stdio_common_v*printf* entry points, one runner each so each
 * gets its own table row. Every case goes through a pair of variadic trampolines,
 * one per side, so both receive the same arguments in a real va_list. The shared
 * corpus below (PF_CORPUS) is run through every function, narrow and wide, and the
 * positional one (PF_POSITIONAL) too: the plain and _s functions must print or
 * reject those formats exactly as Microsoft's do.
 *
 * Graded per call: the return, the WHOLE buffer (prefilled, so a stray write or a
 * missing terminator shows), errno (from a sentinel, so a RESTORED errno shows) and
 * the invalid-parameter count. A case may only consume arguments it is given in
 * EVERY mode the functions run it in -- legacy options, _s validation, positional
 * scans -- which is why the string cases pass g_pf_dual, valid read either narrow
 * or wide.
 *
 * The stream functions write a real file per side and grade the return, errno, the
 * position after each call and, at the end, the whole file. */
#include <stdio.h>
#include <float.h>
#include <math.h>

/* All fourteen come from the one object, so one guard: once it is taken over and a
 * plain sweep has no reference for it, the runners AND their helpers compile out
 * (pecheck exempts the whole block, helpers included). */
#if UCRTDIFF_REF_HAS___stdio_common_vsprintf
int __cdecl ref___stdio_common_vsprintf(unsigned long long, char *, size_t, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vswprintf(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vsprintf_s(unsigned long long, char *, size_t, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vswprintf_s(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vsnprintf_s(unsigned long long, char *, size_t, size_t, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vsnwprintf_s(unsigned long long, wchar_t *, size_t, size_t, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vsprintf_p(unsigned long long, char *, size_t, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vswprintf_p(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfprintf(unsigned long long, FILE *, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfwprintf(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfprintf_s(unsigned long long, FILE *, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfwprintf_s(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfprintf_p(unsigned long long, FILE *, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfwprintf_p(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fflush(FILE *);
int __cdecl ref_fseek(FILE *, long, int);
long __cdecl ref_ftell(FILE *);
size_t __cdecl ref_fread(void *, size_t, size_t, FILE *);
int __cdecl ref__set_printf_count_output(int);

/* The shipping side through its import pointers, as for findfile: mingw's stdio.h
 * wraps these names in inlines of its own. */
extern int(__cdecl *const ud___stdio_common_vsprintf)(unsigned long long, char *, size_t, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsprintf));
extern int(__cdecl *const ud___stdio_common_vswprintf)(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vswprintf));
extern int(__cdecl *const ud___stdio_common_vsprintf_s)(unsigned long long, char *, size_t, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsprintf_s));
extern int(__cdecl *const ud___stdio_common_vswprintf_s)(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vswprintf_s));
extern int(__cdecl *const ud___stdio_common_vsnprintf_s)(unsigned long long, char *, size_t, size_t, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsnprintf_s));
extern int(__cdecl *const ud___stdio_common_vsnwprintf_s)(unsigned long long, wchar_t *, size_t, size_t, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsnwprintf_s));
extern int(__cdecl *const ud___stdio_common_vsprintf_p)(unsigned long long, char *, size_t, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsprintf_p));
extern int(__cdecl *const ud___stdio_common_vswprintf_p)(unsigned long long, wchar_t *, size_t, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vswprintf_p));
extern int(__cdecl *const ud___stdio_common_vfprintf)(unsigned long long, FILE *, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfprintf));
extern int(__cdecl *const ud___stdio_common_vfwprintf)(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfwprintf));
extern int(__cdecl *const ud___stdio_common_vfprintf_s)(unsigned long long, FILE *, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfprintf_s));
extern int(__cdecl *const ud___stdio_common_vfwprintf_s)(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfwprintf_s));
extern int(__cdecl *const ud___stdio_common_vfprintf_p)(unsigned long long, FILE *, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfprintf_p));
extern int(__cdecl *const ud___stdio_common_vfwprintf_p)(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfwprintf_p));

/* A varargs trampoline per side: the only way to hand both a genuine va_list. */
#define PF_TRAMP_BUF(name, ch)                                                            \
    static int pfd_##name(unsigned long long o, ch *b, size_t n, const ch *f, ...)        \
    { va_list ap; va_start(ap, f); int r = ud_##name(o, b, n, f, NULL, ap); va_end(ap); return r; } \
    static int pfr_##name(unsigned long long o, ch *b, size_t n, const ch *f, ...)        \
    { va_list ap; va_start(ap, f); int r = ref_##name(o, b, n, f, NULL, ap); va_end(ap); return r; }
#define PF_TRAMP_SN(name, ch)                                                             \
    static int pfd_##name(unsigned long long o, ch *b, size_t n, size_t m, const ch *f, ...) \
    { va_list ap; va_start(ap, f); int r = ud_##name(o, b, n, m, f, NULL, ap); va_end(ap); return r; } \
    static int pfr_##name(unsigned long long o, ch *b, size_t n, size_t m, const ch *f, ...) \
    { va_list ap; va_start(ap, f); int r = ref_##name(o, b, n, m, f, NULL, ap); va_end(ap); return r; }
#define PF_TRAMP_F(name, ch)                                                              \
    static int pfd_##name(unsigned long long o, FILE *s, const ch *f, ...)                \
    { va_list ap; va_start(ap, f); int r = ud_##name(o, s, f, NULL, ap); va_end(ap); return r; } \
    static int pfr_##name(unsigned long long o, FILE *s, const ch *f, ...)                \
    { va_list ap; va_start(ap, f); int r = ref_##name(o, s, f, NULL, ap); va_end(ap); return r; }

PF_TRAMP_BUF(__stdio_common_vsprintf, char)
PF_TRAMP_BUF(__stdio_common_vswprintf, wchar_t)
PF_TRAMP_BUF(__stdio_common_vsprintf_s, char)
PF_TRAMP_BUF(__stdio_common_vswprintf_s, wchar_t)
PF_TRAMP_SN(__stdio_common_vsnprintf_s, char)
PF_TRAMP_SN(__stdio_common_vsnwprintf_s, wchar_t)
PF_TRAMP_BUF(__stdio_common_vsprintf_p, char)
PF_TRAMP_BUF(__stdio_common_vswprintf_p, wchar_t)
PF_TRAMP_F(__stdio_common_vfprintf, char)
PF_TRAMP_F(__stdio_common_vfwprintf, wchar_t)
PF_TRAMP_F(__stdio_common_vfprintf_s, char)
PF_TRAMP_F(__stdio_common_vfwprintf_s, wchar_t)
PF_TRAMP_F(__stdio_common_vfprintf_p, char)
PF_TRAMP_F(__stdio_common_vfwprintf_p, wchar_t)

/* Valid read narrow ("hi!") AND wide (U+6968 U+0021): %s is narrow or wide in a
 * wide function depending on _CRT_INTERNAL_PRINTF_LEGACY_WIDE_SPECIFIERS. */
static const unsigned short g_pf_dual_w[] = { 0x6968, 0x0021, 0, 0 };
#define g_pf_dual ((const void *)g_pf_dual_w)
/* %Z's counted strings: 4 bytes of the dual buffer, whichever way it is read. */
static const struct { unsigned short len, max; const void *buf; } g_pf_z = { 4, 6, g_pf_dual_w };
static const struct { unsigned short len, max; const void *buf; } g_pf_z_nobuf = { 4, 6, 0 };

#define PF_CORPUS(X)                                                                    \
    X("") X("hello") X("%%") X("a%%b") X("100%") X("%") X("%q") X("%5%") X("%-q")      \
    X("%5.3q") X("%hq") X("%lq") X("%N") X("%Nd", 1) X("%Fd", 1.5) X("%F", 1.5)         \
    X("%I") X("%Iq") X("caf\xe9 %d", 1) X("\x81\x40%d", 2)                               \
    X("%d", 0) X("%d", -2147483647 - 1) X("%5d", 42) X("%-5d|", 42) X("%05d", -42)      \
    X("%+d", 42) X("% d", 42) X("%+ d", 42) X("%.0d", 0) X("%.3d", 7) X("%8.3d", -7)    \
    X("%-08d|", 7) X("%x", 0xBEEF) X("%#x", 0) X("%#x", 255) X("%#X", 255) X("%#o", 8)  \
    X("%#o", 0) X("%o", 8) X("%u", -1) X("%i", 12) X("%*d", 5, 42) X("%-*d|", -5, 42)   \
    X("%.*d", 4, 3) X("%.*d", -1, 3) X("%*.*d", 8, 3, 5) X("%99999999999d", 1)          \
    X("%.99999999999d", 1) X("%300d", 1) X("%hhd", 300) X("%hd", 70000) X("%hu", -1)   \
    X("%ld", -5L) X("%lld", -1LL) X("%llx", 0x123456789ABCDEFLL) X("%I64d", 5LL)          \
    X("%I32d", 5) X("%Id", (intptr_t)-5) X("%Ix", (intptr_t)255) X("%zd", (size_t)42)   \
    X("%zu", (size_t)-1) X("%td", (ptrdiff_t)-3) X("%jd", (long long)-9) X("%Ld", 1)    \
    X("%wd", 1) X("%hld", 1) X("%lhd", 1) X("%I64", 1LL) X("%I32", 1)                   \
    X("%c", 'A') X("%c", 0) X("%5c", 'B') X("%-3c|", 'C') X("%hc", 'D') X("%lc", 0x263A) \
    X("%lc", 'E') X("%C", 'F') X("%wc", 'G') X("%c", 0x1E9E) X("%hc", 0xE9)             \
    X("%s", g_pf_dual) X("%s", (char *)0) X("%10s", g_pf_dual) X("%-10s|", g_pf_dual)  \
    X("%.1s", g_pf_dual) X("%.0s", g_pf_dual) X("%.*s", 1, g_pf_dual)                    \
    X("%hs", "narrow") X("%ls", L"wide") X("%S", L"wide") X("%ws", L"wide")              \
    X("%ls", (wchar_t *)0) X("%hs", (char *)0) X("%Z", &g_pf_z) X("%Z", (void *)0)       \
    X("%Z", &g_pf_z_nobuf) X("%wZ", &g_pf_z) X("%hZ", &g_pf_z)                          \
    X("%p", (void *)0x1234) X("%p", (void *)0) X("%20p", (void *)0xABC) X("%-20p|", (void *)1) \
    X("%f", 0.0) X("%f", -0.0) X("%f", 1.5) X("%f", 1e300) X("%e", 12345.678)            \
    X("%E", 1e-10) X("%g", 0.0001) X("%g", 123456789.0) X("%G", 1e-5) X("%#g", 1.0)      \
    X("%#.0f", 3.0) X("%#.0e", 3.0) X("%.0f", 2.5) X("%.0f", 3.5) X("%.1f", 0.05)        \
    X("%.20f", 0.1) X("%.200f", 1.0 / 3) X("%a", 1.0) X("%A", -0.5) X("%.3a", 3.14)      \
    X("%.0a", 1.5) X("%#.0a", 1.5) X("%f", HUGE_VAL) X("%f", -HUGE_VAL) X("%f", NAN)     \
    X("%010f", HUGE_VAL) X("%-10f|", NAN) X("%+e", 0.0) X("% g", 5.0)                     \
    X("%010.3f", -1.5) X("%Lf", 1.5) X("%lf", 2.5) X("%.*f", 2, 3.14159) X("%e", DBL_MAX) \
    X("%e", DBL_MIN) X("%g", 4.9e-324) X("%#x %#o %d %s|", 1, 1, 1, g_pf_dual)           \
    X("%-+#012.4e", 6.02e23) X("%.3g", 0.0001234) X("%g", 100000.0) X("%g", 1000000.0)

#define PF_POSITIONAL(X)                                                                \
    X("%1$d %2$d", 1, 2) X("%2$d %1$d", 1, 2) X("%1$d %1$d", 7) X("%1$s %2$d", g_pf_dual, 3) \
    X("%2$s %1$f", 1.5, g_pf_dual) X("%1$*2$d|", 5, 8) X("%1$.*2$f", 3.14159, 2)        \
    X("%2$d", 1, 2) X("%1$d %d", 1, 2) X("%d %1$d", 1, 2) X("%0$d", 1) X("%101$d", 1)   \
    X("%1$d %1$s", 1) X("%1$lld %2$d", 1LL, 2) X("%1$c%2$c", 'a', 'b') X("abc %1$d def", 5) \
    X("%%%1$d", 5) X("%1$p", (void *)1) X("%1$d %1$x", 255) X("%1$d %1$lld", 1LL)          \
    X("%1$*1$d", 3) X("%1$", 1) X("%1$-5d|", 4) X("%1$n", (int *)0) X("%3$d %1$d", 1, 2, 3) \
    X("%1$* 2$d|", 5, 8) X("%1$*+2$d|", 5, 8) X("% 1$d", 5) X("%1$.* 2$f", 1.5, 2)

static const struct { unsigned long long opt; int size; } g_pf_cfg[] = {
    { 0, -1 }, { 0, -2 }, { 0, 0 }, { 0, 1 }, { 0, 4 }, { 0, 12 }, { 0, 256 },
    { 1, 256 }, { 2, 256 }, { 4, 256 }, { 8, 256 }, { 0x10, 256 }, { 0x20, 256 }, { 0x24, 256 },
    { 1, 0 }, { 1, 1 }, { 1, 4 }, { 1, 12 }, { 2, -1 }, { 2, 0 }, { 2, 1 }, { 2, 4 },
};
#define PF_NCFG (sizeof(g_pf_cfg) / sizeof(g_pf_cfg[0]))
/* The _s and _p functions: fewer option mixes, the same sizes. */
static const struct { unsigned long long opt; int size; } g_pf_cfg_s[] = {
    { 0, -1 }, { 0, -2 }, { 0, 0 }, { 0, 1 }, { 0, 4 }, { 0, 12 }, { 0, 256 }, { 4, 256 }, { 8, 256 },
};
#define PF_NCFG_S (sizeof(g_pf_cfg_s) / sizeof(g_pf_cfg_s[0]))
/* _vsnprintf_s: (buffer count, max count); -1 is a NULL buffer. */
static const struct { int size; size_t max; } g_pf_sn[] = {
    { -1, 0 }, { -1, 5 }, { 256, (size_t)-1 }, { 4, (size_t)-1 }, { 1, (size_t)-1 },
    { 4, 2 }, { 4, 3 }, { 4, 10 }, { 12, 5 }, { 256, 0 }, { 256, 300 },
};
#define PF_NSN (sizeof(g_pf_sn) / sizeof(g_pf_sn[0]))

#define PF_ERRNO_SENTINEL 777

static const char *pf_label(const char *fmt, unsigned cfg)
{
    static char buf[48];
    int n = 0;
    for (; fmt[n] && n < 36; n++)
        buf[n] = fmt[n];
    buf[n++] = '#';
    buf[n++] = (char)('0' + cfg / 10 % 10);
    buf[n++] = (char)('0' + cfg % 10);
    buf[n] = 0;
    return buf;
}

#define PF_RESET() do {                                                                 \
        *_errno() = PF_ERRNO_SENTINEL; *ref__errno() = PF_ERRNO_SENTINEL;               \
        g_iph_dll = g_iph_ref = 0;                                                      \
    } while (0)

#define PF_GRADE(fam, lbl, r1, r2, b1, b2) do {                                         \
        grade2(fam ".ret", lbl, r1, r2);                                                \
        grade2(fam ".buf", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));                \
        grade2(fam ".errno", lbl, *_errno(), *ref__errno());                            \
        grade2(fam ".iph", lbl, g_iph_dll, g_iph_ref);                                  \
    } while (0)

/* A buffer function over one format: every configuration in `cfgs`. */
#define PF_BUF_CASE(name, fam, ch, lit, cfgs, ncfg, fmt, ...)                           \
    for (unsigned c = 0; c < ncfg; c++) {                                               \
        ch b1[260], b2[260];                                                            \
        for (unsigned k = 0; k < 260; k++) b1[k] = b2[k] = (ch)'X';                     \
        int sz = cfgs[c].size;                                                          \
        ch *p1 = sz < 0 ? (ch *)0 : b1, *p2 = sz < 0 ? (ch *)0 : b2;                    \
        size_t n = sz == -2 ? 12 : sz < 0 ? 0 : (size_t)sz;                             \
        PF_RESET();                                                                     \
        int r1 = pfd_##name(cfgs[c].opt, p1, n, lit fmt, ##__VA_ARGS__);                \
        int r2 = pfr_##name(cfgs[c].opt, p2, n, lit fmt, ##__VA_ARGS__);                \
        PF_GRADE(fam, pf_label(fmt, c), r1, r2, b1, b2);                                \
    }

#define PFX_VSPRINTF(fmt, ...)   { PF_BUF_CASE(__stdio_common_vsprintf, "vsprintf", char, , g_pf_cfg, PF_NCFG, fmt, ##__VA_ARGS__) }
#define PFX_VSWPRINTF(fmt, ...)  { PF_BUF_CASE(__stdio_common_vswprintf, "vswprintf", wchar_t, L"", g_pf_cfg, PF_NCFG, fmt, ##__VA_ARGS__) }
#define PFX_VSPRINTF_S(fmt, ...) { PF_BUF_CASE(__stdio_common_vsprintf_s, "vsprintf_s", char, , g_pf_cfg_s, PF_NCFG_S, fmt, ##__VA_ARGS__) }
#define PFX_VSWPRINTF_S(fmt, ...) { PF_BUF_CASE(__stdio_common_vswprintf_s, "vswprintf_s", wchar_t, L"", g_pf_cfg_s, PF_NCFG_S, fmt, ##__VA_ARGS__) }
#define PFX_VSPRINTF_P(fmt, ...) { PF_BUF_CASE(__stdio_common_vsprintf_p, "vsprintf_p", char, , g_pf_cfg_s, PF_NCFG_S, fmt, ##__VA_ARGS__) }
#define PFX_VSWPRINTF_P(fmt, ...) { PF_BUF_CASE(__stdio_common_vswprintf_p, "vswprintf_p", wchar_t, L"", g_pf_cfg_s, PF_NCFG_S, fmt, ##__VA_ARGS__) }

#define PF_SN_CASE(name, fam, ch, lit, fmt, ...)                                        \
    for (unsigned c = 0; c < PF_NSN; c++) {                                             \
        ch b1[260], b2[260];                                                            \
        for (unsigned k = 0; k < 260; k++) b1[k] = b2[k] = (ch)'X';                     \
        int sz = g_pf_sn[c].size;                                                       \
        ch *p1 = sz < 0 ? (ch *)0 : b1, *p2 = sz < 0 ? (ch *)0 : b2;                    \
        size_t n = sz < 0 ? 0 : (size_t)sz;                                             \
        PF_RESET();                                                                     \
        int r1 = pfd_##name(0, p1, n, g_pf_sn[c].max, lit fmt, ##__VA_ARGS__);          \
        int r2 = pfr_##name(0, p2, n, g_pf_sn[c].max, lit fmt, ##__VA_ARGS__);          \
        PF_GRADE(fam, pf_label(fmt, c), r1, r2, b1, b2);                                \
    }
#define PFX_VSNPRINTF_S(fmt, ...)  { PF_SN_CASE(__stdio_common_vsnprintf_s, "vsnprintf_s", char, , fmt, ##__VA_ARGS__) }
#define PFX_VSNWPRINTF_S(fmt, ...) { PF_SN_CASE(__stdio_common_vsnwprintf_s, "vsnwprintf_s", wchar_t, L"", fmt, ##__VA_ARGS__) }

/* %n needs a DIFFERENT target per side, so it cannot ride the shared corpus: both
 * with the count output disabled (the default: the invalid-parameter path) and
 * enabled, at each integer width. */
#define PF_N_CASES(name, fam, ch, lit)                                                  \
    for (int enable = 0; enable < 2; enable++) {                                        \
        _set_printf_count_output(enable); ref__set_printf_count_output(enable);         \
        long long n1, n2; ch b1[64], b2[64];                                            \
        static const char *const fmts[] = { "ab%n", "ab%hhn", "ab%hn", "abc%lln", "%5d%n", "%Ln" }; \
        for (unsigned f = 0; f < 6; f++) {                                              \
            ch wf[16]; unsigned k = 0;                                                  \
            for (; fmts[f][k]; k++) wf[k] = (ch)fmts[f][k];                              \
            wf[k] = 0;                                                                  \
            for (k = 0; k < 64; k++) b1[k] = b2[k] = (ch)'X';                           \
            n1 = n2 = -12345; PF_RESET();                                               \
            int r1, r2;                                                                 \
            if (f == 4) {                                                               \
                r1 = pfd_##name(0, b1, 64, wf, 7, &n1);                                 \
                r2 = pfr_##name(0, b2, 64, wf, 7, &n2);                                 \
            } else {                                                                    \
                r1 = pfd_##name(0, b1, 64, wf, &n1);                                    \
                r2 = pfr_##name(0, b2, 64, wf, &n2);                                    \
            }                                                                           \
            const char *lbl = pf_label(fmts[f], (unsigned)enable);                      \
            PF_GRADE(fam ".n", lbl, r1, r2, b1, b2);                                    \
            grade2(fam ".n.value", lbl, n1, n2);                                        \
        }                                                                               \
    }                                                                                   \
    _set_printf_count_output(0); ref__set_printf_count_output(0);

static void run_pf_vsprintf(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSPRINTF)
    PF_POSITIONAL(PFX_VSPRINTF)
    PF_N_CASES(__stdio_common_vsprintf, "vsprintf", char, )
}
static void run_pf_vswprintf(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSWPRINTF)
    PF_POSITIONAL(PFX_VSWPRINTF)
    PF_N_CASES(__stdio_common_vswprintf, "vswprintf", wchar_t, L"")
}
static void run_pf_vsprintf_s(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSPRINTF_S)
    PF_POSITIONAL(PFX_VSPRINTF_S)
    PF_N_CASES(__stdio_common_vsprintf_s, "vsprintf_s", char, )
}
static void run_pf_vswprintf_s(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSWPRINTF_S)
    PF_POSITIONAL(PFX_VSWPRINTF_S)
    PF_N_CASES(__stdio_common_vswprintf_s, "vswprintf_s", wchar_t, L"")
}
static void run_pf_vsnprintf_s(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSNPRINTF_S)
    PF_POSITIONAL(PFX_VSNPRINTF_S)
}
static void run_pf_vsnwprintf_s(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSNWPRINTF_S)
    PF_POSITIONAL(PFX_VSNWPRINTF_S)
}
static void run_pf_vsprintf_p(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSPRINTF_P)
    PF_POSITIONAL(PFX_VSPRINTF_P)
    PF_N_CASES(__stdio_common_vsprintf_p, "vsprintf_p", char, )
}
static void run_pf_vswprintf_p(void)
{
    install_counting_iph();
    PF_CORPUS(PFX_VSWPRINTF_P)
    PF_POSITIONAL(PFX_VSWPRINTF_P)
    PF_N_CASES(__stdio_common_vswprintf_p, "vswprintf_p", wchar_t, L"")
}

/* ---- the stream functions ---- */

static char g_pf_dir[MAX_PATH];

/* A directory both sides can create files in: %TEMP%, else C:\, as ff_setup. */
static int pf_dir(void)
{
    if (g_pf_dir[0])
        return 1;
    char probe[MAX_PATH + 32];
    DWORD n = GetTempPathA(MAX_PATH - 32, g_pf_dir);
    if (n && n < MAX_PATH - 32) {
        lstrcpyA(probe, g_pf_dir); lstrcatA(probe, "ucrtdiff_pf.probe");
        HANDLE h = CreateFileA(probe, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
        if (h != INVALID_HANDLE_VALUE) {
            CloseHandle(h);
            return 1;
        }
    }
    lstrcpyA(g_pf_dir, "C:\\");
    return 1;
}

static const char *pf_path(const char *leaf, int side)
{
    static char buf[2][MAX_PATH + 48];
    lstrcpyA(buf[side], g_pf_dir);
    lstrcatA(buf[side], side ? "ucrtdiff_pf_ref_" : "ucrtdiff_pf_dll_");
    lstrcatA(buf[side], leaf);
    return buf[side];
}

/* One stream case: the return, errno, iph and where the stream stands afterwards. */
#define PF_F_CASE(name, fam, lit, f1, f2, fmt, ...) {                                   \
        PF_RESET();                                                                     \
        int r1 = pfd_##name(0, f1, lit fmt, ##__VA_ARGS__);                             \
        int r2 = pfr_##name(0, f2, lit fmt, ##__VA_ARGS__);                             \
        const char *lbl = pf_label(fmt, 0);                                             \
        grade2(fam ".ret", lbl, r1, r2);                                                \
        grade2(fam ".errno", lbl, *_errno(), *ref__errno());                            \
        grade2(fam ".iph", lbl, g_iph_dll, g_iph_ref);                                  \
        grade2(fam ".tell", lbl, ftell(f1), ref_ftell(f2));                             \
    }

/* The whole file, read back by each side through its own CRT. */
static unsigned char g_pf_d1[65536], g_pf_d2[65536];
#define PF_GRADE_FILE(fam, mode, f1, f2) do {                                           \
        fflush(f1); ref_fflush(f2);                                                     \
        fseek(f1, 0, SEEK_SET); ref_fseek(f2, 0, SEEK_SET);                             \
        size_t n1 = fread(g_pf_d1, 1, sizeof g_pf_d1, f1);                              \
        size_t n2 = ref_fread(g_pf_d2, 1, sizeof g_pf_d2, f2);                          \
        grade2(fam ".file.len", mode, (long long)n1, (long long)n2);                    \
        grade2(fam ".file", mode, fnv(g_pf_d1, n1), fnv(g_pf_d2, n2));                  \
    } while (0)

/* Each side opens its own file in `mode`, runs the corpus through it, and the two
 * files are compared. The Unicode-mode stream is where the narrow functions must
 * refuse (EINVAL) and the wide ones write UTF-16; the text-mode one is where a wide
 * function converts through the locale, character by character. */
#define PF_STREAM_RUN(name, fam, lit, X_CORPUS, leaf, mode) do {                        \
        FILE *f1 = fopen(pf_path(leaf, 0), mode);                                      \
        FILE *f2 = ref_fopen(pf_path(leaf, 1), mode);                                  \
        grade2(fam ".open", mode, f1 != 0, f2 != 0);                                   \
        if (f1 && f2) {                                                                \
            X_CORPUS(PFX_F_##name)                                                     \
            PF_GRADE_FILE(fam, mode, f1, f2);                                          \
        }                                                                              \
        if (f1) fclose(f1);                                                            \
        if (f2) ref_fclose(f2);                                                        \
    } while (0)

#define PFX_F___stdio_common_vfprintf(fmt, ...)    PF_F_CASE(__stdio_common_vfprintf, "vfprintf", , f1, f2, fmt, ##__VA_ARGS__)
#define PFX_F___stdio_common_vfwprintf(fmt, ...)   PF_F_CASE(__stdio_common_vfwprintf, "vfwprintf", L"", f1, f2, fmt, ##__VA_ARGS__)
#define PFX_F___stdio_common_vfprintf_s(fmt, ...)  PF_F_CASE(__stdio_common_vfprintf_s, "vfprintf_s", , f1, f2, fmt, ##__VA_ARGS__)
#define PFX_F___stdio_common_vfwprintf_s(fmt, ...) PF_F_CASE(__stdio_common_vfwprintf_s, "vfwprintf_s", L"", f1, f2, fmt, ##__VA_ARGS__)
#define PFX_F___stdio_common_vfprintf_p(fmt, ...)  PF_F_CASE(__stdio_common_vfprintf_p, "vfprintf_p", , f1, f2, fmt, ##__VA_ARGS__)
#define PFX_F___stdio_common_vfwprintf_p(fmt, ...) PF_F_CASE(__stdio_common_vfwprintf_p, "vfwprintf_p", L"", f1, f2, fmt, ##__VA_ARGS__)

/* The NULL-stream and NULL-format rejections. */
#define PF_F_NULLS(name, fam, lit) do {                                                 \
        PF_RESET();                                                                     \
        int r1 = pfd_##name(0, (FILE *)0, lit "x"), r2 = pfr_##name(0, (FILE *)0, lit "x"); \
        grade2(fam ".nullstream", "-", r1, r2);                                         \
        grade2(fam ".nullstream.errno", "-", *_errno(), *ref__errno());                 \
        grade2(fam ".nullstream.iph", "-", g_iph_dll, g_iph_ref);                       \
    } while (0)

#define PF_STREAM_FAMILY(name, fam, lit, X_EXTRA) do {                                  \
        install_counting_iph();                                                         \
        pf_dir();                                                                       \
        PF_STREAM_RUN(name, fam, lit, PF_CORPUS, #name ".b", "w+b");                    \
        PF_STREAM_RUN(name, fam, lit, PF_CORPUS, #name ".t", "w+t");                    \
        PF_STREAM_RUN(name, fam, lit, X_EXTRA, #name ".u", "w+, ccs=UTF-16LE");          \
        PF_F_NULLS(name, fam, lit);                                                     \
    } while (0)

static void run_pf_vfprintf(void)     { PF_STREAM_FAMILY(__stdio_common_vfprintf, "vfprintf", , PF_POSITIONAL); }
static void run_pf_vfwprintf(void)    { PF_STREAM_FAMILY(__stdio_common_vfwprintf, "vfwprintf", L"", PF_POSITIONAL); }
static void run_pf_vfprintf_s(void)   { PF_STREAM_FAMILY(__stdio_common_vfprintf_s, "vfprintf_s", , PF_POSITIONAL); }
static void run_pf_vfwprintf_s(void)  { PF_STREAM_FAMILY(__stdio_common_vfwprintf_s, "vfwprintf_s", L"", PF_POSITIONAL); }
static void run_pf_vfprintf_p(void)   { PF_STREAM_FAMILY(__stdio_common_vfprintf_p, "vfprintf_p", , PF_POSITIONAL); }
static void run_pf_vfwprintf_p(void)  { PF_STREAM_FAMILY(__stdio_common_vfwprintf_p, "vfwprintf_p", L"", PF_POSITIONAL); }
#endif /* UCRTDIFF_REF_HAS___stdio_common_vsprintf */

/* ---- strtod (convert/strtod.cpp) ----------------------------------------
 *
 * One runner per export. Each converts the corpus on both sides and grades the
 * result's BITS (a -0.0, a NaN payload or a one-ulp miss all show), the end
 * offset, errno from a sentinel, and the invalid-parameter count. The wide runners
 * add strings only a wide parser can see: Unicode decimal digits, which UCRT's
 * wide digit parser accepts. */
#if UCRTDIFF_REF_HAS_strtod
float __cdecl ref_strtof(const char *, char **);
float __cdecl ref__strtof_l(const char *, char **, _locale_t);
double __cdecl ref_strtod(const char *, char **);
double __cdecl ref__strtod_l(const char *, char **, _locale_t);
long double __cdecl ref_strtold(const char *, char **);
long double __cdecl ref__strtold_l(const char *, char **, _locale_t);
float __cdecl ref_wcstof(const wchar_t *, wchar_t **);
float __cdecl ref__wcstof_l(const wchar_t *, wchar_t **, _locale_t);
double __cdecl ref_wcstod(const wchar_t *, wchar_t **);
double __cdecl ref__wcstod_l(const wchar_t *, wchar_t **, _locale_t);
long double __cdecl ref_wcstold(const wchar_t *, wchar_t **);
long double __cdecl ref__wcstold_l(const wchar_t *, wchar_t **, _locale_t);
/* The shipping side through its import pointers: the toolchain headers declare
 * some of these inline or with a different long double. */
extern float(__cdecl *const ud_strtof)(const char *, char **) __asm__(UD_IMP(strtof));
extern float(__cdecl *const ud__strtof_l)(const char *, char **, _locale_t) __asm__(UD_IMP(_strtof_l));
extern double(__cdecl *const ud_strtod)(const char *, char **) __asm__(UD_IMP(strtod));
extern double(__cdecl *const ud__strtod_l)(const char *, char **, _locale_t) __asm__(UD_IMP(_strtod_l));
extern long double(__cdecl *const ud_strtold)(const char *, char **) __asm__(UD_IMP(strtold));
extern long double(__cdecl *const ud__strtold_l)(const char *, char **, _locale_t) __asm__(UD_IMP(_strtold_l));
extern float(__cdecl *const ud_wcstof)(const wchar_t *, wchar_t **) __asm__(UD_IMP(wcstof));
extern float(__cdecl *const ud__wcstof_l)(const wchar_t *, wchar_t **, _locale_t) __asm__(UD_IMP(_wcstof_l));
extern double(__cdecl *const ud_wcstod)(const wchar_t *, wchar_t **) __asm__(UD_IMP(wcstod));
extern double(__cdecl *const ud__wcstod_l)(const wchar_t *, wchar_t **, _locale_t) __asm__(UD_IMP(_wcstod_l));
extern long double(__cdecl *const ud_wcstold)(const wchar_t *, wchar_t **) __asm__(UD_IMP(wcstold));
extern long double(__cdecl *const ud__wcstold_l)(const wchar_t *, wchar_t **, _locale_t) __asm__(UD_IMP(_wcstold_l));

static const char *const g_fp_in[] = {
    "0", "-0", "+0", "1", "-1", "0.1", "0.30000000000000004", "1e308", "1e309", "-1e309",
    "1.7976931348623157e308", "1.7976931348623158e308", "1.7976931348623159e308",
    "2.2250738585072011e-308", "2.2250738585072014e-308", "4.9406564584124654e-324",
    "2.4703282292062327e-324", "2.4703282292062328e-324", "1e-324", "1e-400", "-1e-400",
    "3.4028235e38", "3.4028236e38", "3.40282357e38", "1.17549435e-38", "1.4e-45", "7e-46", "7.1e-46",
    "9007199254740993", "9007199254740992.5", "9007199254740993.0000000000000000001",
    "123456789012345678901234567890", "0.000001", "000000.000000123e+0010",
    "1e99999", "1e-99999", "0e99999", "1e5200", "1e5201", "0.0000000001e5205",
    "  \t\n12.5xyz", "+.5", "-.5e1", ".", "..5", "e5", "1e", "1e+", "1e-", "1E3", "1ee3",
    "inf", "INF", "-Inf", "infinity", "INFINITY", "infinit", "infx", "in", "i",
    "nan", "-NaN", "nan(", "nan()", "nan(123abc_)", "nan(1-2)", "nan(snan)", "NAN(SNAN)",
    "nan(ind)", "-nan(ind)", "nan(inda)", "nan(sna)", "nanq",
    "0x", "0X", "0x1", "0x1p", "0x1p-2", "0x1.8p1", "0xAbC.dEfP+4", "0x.8p1", "0x1p1024",
    "0x1p-1074", "0x1p-1075", "0x1.00000000000008p0", "0x1.00000000000018p0",
    "0x1.fffffffffffff8p1023", "0x1.000001p0", "0x1.0000011p0", "0x0p5", "0x0000.0000p9", "0xg",
    "0.5e", "5.", "05", "-0x1p3", "0x-1", "x1", "", " ", "-", "+",
    "1.00000000000000011102230246251565404236316680908203125",
    "1.000000000000000111022302462515654042363166809082031251",
    "1.000000000000000111022302462515654042363166809082031249",
    "4503599627370496.5", "4503599627370497.5", "2.5e-324", "123.456e-2", "1,5",
    "\x80\xff 1", "1\xa0",
};
#define FP_N (sizeof(g_fp_in) / sizeof(g_fp_in[0]))

/* A long mantissa: past the parser's 768 kept digits, with the deciding digit
 * after them. */
static char g_fp_long[1100];
static void fp_make_long(void)
{
    int k = 0;
    g_fp_long[k++] = '0'; g_fp_long[k++] = '.';
    for (int i = 0; i < 322; i++) g_fp_long[k++] = '0';
    g_fp_long[k++] = '2'; g_fp_long[k++] = '4'; g_fp_long[k++] = '7';
    for (int i = 0; i < 700; i++) g_fp_long[k++] = (char)('0' + i * 7 % 10);
    g_fp_long[k++] = '1';
    g_fp_long[k] = 0;
}

static const wchar_t *const g_fp_win[] = {
    L"\x0661\x0662.\x0665", L"\xff11\xff10" L"e\xff12", L"\x0966.5", L"1\x2003", L"\x3000 3.5",
    L"1\x0e50\x0e50", L"\x17e0x1p1",
};
#define FP_WN (sizeof(g_fp_win) / sizeof(g_fp_win[0]))

#define FP_BITS(v) _Generic((v), float: (long long)fp_bits32(v), default: (long long)fp_bits64((double)(v)))
static unsigned fp_bits32(float f) { unsigned u; memcpy(&u, &f, 4); return u; }
static unsigned long long fp_bits64(double d) { unsigned long long u; memcpy(&u, &d, 8); return u; }

#define FP_ROW(fn, T, s, lbl, ...) do {                                              \
        T *e1 = (T *)1, *e2 = (T *)1;                                                \
        *_errno() = PF_ERRNO_SENTINEL; *ref__errno() = PF_ERRNO_SENTINEL;            \
        g_iph_dll = g_iph_ref = 0;                                                   \
        long long v1 = FP_BITS(ud_##fn(s, &e1, ##__VA_ARGS__));                      \
        long long v2 = FP_BITS(ref_##fn(s, &e2, ##__VA_ARGS__));                     \
        grade2(#fn, lbl, v1, v2);                                                    \
        grade2(#fn ".end", lbl, s ? e1 - s : (e1 == 0) - 2, s ? e2 - s : (e2 == 0) - 2); \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                         \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                               \
    } while (0)

#define FP_NARROW(fn, ...) do {                                                      \
        install_counting_iph();                                                      \
        fp_make_long();                                                              \
        for (unsigned i = 0; i < FP_N; i++)                                          \
            FP_ROW(fn, char, g_fp_in[i], g_fp_in[i], ##__VA_ARGS__);                 \
        FP_ROW(fn, char, g_fp_long, "long", ##__VA_ARGS__);                          \
        FP_ROW(fn, char, (const char *)0, "NULL", ##__VA_ARGS__);                    \
    } while (0)

#define FP_WIDE(fn, ...) do {                                                        \
        install_counting_iph();                                                      \
        fp_make_long();                                                              \
        static wchar_t w[1100];                                                      \
        for (unsigned i = 0; i < FP_N + 1 + FP_WN; i++) {                            \
            const wchar_t *ws = w;                                                   \
            if (i < FP_N + 1) {                                                      \
                const char *n = i < FP_N ? g_fp_in[i] : g_fp_long;                   \
                unsigned k = 0;                                                      \
                for (; n[k]; k++) w[k] = (unsigned char)n[k];                        \
                w[k] = 0;                                                            \
            } else {                                                                 \
                ws = g_fp_win[i - FP_N - 1];                                         \
            }                                                                        \
            FP_ROW(fn, wchar_t, ws, label_idx(i), ##__VA_ARGS__);                    \
        }                                                                            \
        FP_ROW(fn, wchar_t, (const wchar_t *)0, "NULL", ##__VA_ARGS__);              \
    } while (0)

static void run_fp_strtof(void)    { FP_NARROW(strtof); }
static void run_fp__strtof_l(void) { FP_NARROW(_strtof_l, NULL); }
static void run_fp_strtod(void)    { FP_NARROW(strtod); }
static void run_fp__strtod_l(void) { FP_NARROW(_strtod_l, NULL); }
static void run_fp_strtold(void)   { FP_NARROW(strtold); }
static void run_fp__strtold_l(void){ FP_NARROW(_strtold_l, NULL); }
static void run_fp_wcstof(void)    { FP_WIDE(wcstof); }
static void run_fp__wcstof_l(void) { FP_WIDE(_wcstof_l, NULL); }
static void run_fp_wcstod(void)    { FP_WIDE(wcstod); }
static void run_fp__wcstod_l(void) { FP_WIDE(_wcstod_l, NULL); }
static void run_fp_wcstold(void)   { FP_WIDE(wcstold); }
static void run_fp__wcstold_l(void){ FP_WIDE(_wcstold_l, NULL); }
#endif /* UCRTDIFF_REF_HAS_strtod */

/* ---- stat (filesystem/stat.cpp) ------------------------------------------
 *
 * The twelve _stat/_wstat/_fstat entry points, one runner each. Every path in
 * ST_PATHS goes through every path function, narrow and wide: files in a fixture
 * directory (read-only, executable extensions, a 2040 timestamp that a 32-bit
 * time cannot hold), the directory itself with and without a slash, missing
 * files, roots that cannot be opened and are fabricated, a bare "C:", devices.
 * Graded per call: the return, errno, _doserrno, the handler count and the WHOLE
 * record, prefilled with 'X' so padding and untouched bytes show.
 *
 * _fstat gets a descriptor per side from that side's own lowio table: a disk
 * file, NUL, a pipe with and without data, the standard handles, a closed one and
 * the out-of-range and _NO_CONSOLE_FILENO values. */
#include <sys/stat.h>
#include <fcntl.h>

#if UCRTDIFF_REF_HAS__stat64
int __cdecl ref__stat32(const char *, struct _stat32 *);
int __cdecl ref__stat32i64(const char *, struct _stat32i64 *);
int __cdecl ref__stat64i32(const char *, struct _stat64i32 *);
int __cdecl ref__stat64(const char *, struct _stat64 *);
int __cdecl ref__wstat32(const wchar_t *, struct _stat32 *);
int __cdecl ref__wstat32i64(const wchar_t *, struct _stat32i64 *);
int __cdecl ref__wstat64i32(const wchar_t *, struct _stat64i32 *);
int __cdecl ref__wstat64(const wchar_t *, struct _stat64 *);
int __cdecl ref__fstat32(int, struct _stat32 *);
int __cdecl ref__fstat32i64(int, struct _stat32i64 *);
int __cdecl ref__fstat64i32(int, struct _stat64i32 *);
int __cdecl ref__fstat64(int, struct _stat64 *);
unsigned long *__cdecl ref___doserrno(void);
int __cdecl ref__wopen(const wchar_t *, int, ...);
int __cdecl ref__close(int);
int __cdecl ref__pipe(int *, unsigned, int);
int __cdecl ref__write(int, const void *, unsigned);

/* Through the import pointers: mingw's sys/stat.h maps several of these names
 * onto inlines and onto each other. */
extern int(__cdecl *const ud__stat32)(const char *, struct _stat32 *) __asm__(UD_IMP(_stat32));
extern int(__cdecl *const ud__stat32i64)(const char *, struct _stat32i64 *) __asm__(UD_IMP(_stat32i64));
extern int(__cdecl *const ud__stat64i32)(const char *, struct _stat64i32 *) __asm__(UD_IMP(_stat64i32));
extern int(__cdecl *const ud__stat64)(const char *, struct _stat64 *) __asm__(UD_IMP(_stat64));
extern int(__cdecl *const ud__wstat32)(const wchar_t *, struct _stat32 *) __asm__(UD_IMP(_wstat32));
extern int(__cdecl *const ud__wstat32i64)(const wchar_t *, struct _stat32i64 *) __asm__(UD_IMP(_wstat32i64));
extern int(__cdecl *const ud__wstat64i32)(const wchar_t *, struct _stat64i32 *) __asm__(UD_IMP(_wstat64i32));
extern int(__cdecl *const ud__wstat64)(const wchar_t *, struct _stat64 *) __asm__(UD_IMP(_wstat64));
extern int(__cdecl *const ud__fstat32)(int, struct _stat32 *) __asm__(UD_IMP(_fstat32));
extern int(__cdecl *const ud__fstat32i64)(int, struct _stat32i64 *) __asm__(UD_IMP(_fstat32i64));
extern int(__cdecl *const ud__fstat64i32)(int, struct _stat64i32 *) __asm__(UD_IMP(_fstat64i32));
extern int(__cdecl *const ud__fstat64)(int, struct _stat64 *) __asm__(UD_IMP(_fstat64));

static wchar_t g_st_dir[MAX_PATH];

/* The fixture, under the first base that takes it (as findfile's): a sibling
 * directory built with findfile's file maker. */
static int st_try_base(const wchar_t *base)
{
    lstrcpyW(g_ff_dir, base);
    size_t n = (size_t)lstrlenW(g_ff_dir);
    if (n && g_ff_dir[n - 1] != L'\\')
        lstrcatW(g_ff_dir, L"\\");
    lstrcatW(g_ff_dir, L"ucrtdiff_st");
    if (!CreateDirectoryW(g_ff_dir, NULL) && GetLastError() != ERROR_ALREADY_EXISTS) {
        DbgPrint("[UCRTDIFF] stat fixture: CreateDirectoryW(%ls) failed, error %lu\n",
                 g_ff_dir, GetLastError());
        return 0;
    }
    wchar_t sub[MAX_PATH];
    lstrcpyW(sub, g_ff_dir); lstrcatW(sub, L"\\sub");
    CreateDirectoryW(sub, NULL);
    int ok = ff_make_file(L"plain.txt", 5, 2001, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"ro.txt", 7, 2001, FILE_ATTRIBUTE_READONLY)
        && ff_make_file(L"prog.EXE", 1, 2001, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"run.cmd", 1, 2001, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"x.bat.txt", 1, 2001, FILE_ATTRIBUTE_NORMAL)
        && ff_make_file(L"t2040.com", 1234, 2040, FILE_ATTRIBUTE_NORMAL);
    if (ok) {
        lstrcpyW(g_st_dir, g_ff_dir);
        DbgPrint("[UCRTDIFF] stat fixture: built under %ls\n", g_st_dir);
    }
    return ok;
}

static int st_setup(void)
{
    wchar_t base[MAX_PATH];
    DWORD n = GetTempPathW(MAX_PATH - 32, base);
    if (n && n < MAX_PATH - 32 && st_try_base(base))
        return 1;
    return st_try_base(L"C:\\");
}

/* Relative to the fixture when they start with '|', else as written. */
static const wchar_t *const g_st_paths[] = {
    L"|", L"|\\", L"|/", L"|\\plain.txt", L"|/plain.txt", L"|\\ro.txt", L"|\\prog.EXE",
    L"|\\run.cmd", L"|\\x.bat.txt", L"|\\t2040.com", L"|\\sub", L"|\\sub\\",
    L"|\\sub\\.", L"|\\sub\\..", L"|\\nothere", L"|\\nothere\\x", L"|\\plain.txt\\",
    L"C:\\", L"C:/", L"c:\\", L"C:", L"C:.", L"\\", L"/", L".", L"..", L"", L"Z:\\", L"Z:",
    L"NUL", L"CON", L"nothere", L"//", L"\\\\", L"\\\\x", L"a.exe",
};
enum { ST_NPATHS = sizeof(g_st_paths) / sizeof(g_st_paths[0]) };

static void st_path(size_t i, wchar_t *w, char *n)
{
    const wchar_t *p = g_st_paths[i];
    if (p[0] == L'|') {
        lstrcpyW(w, g_st_dir);
        lstrcatW(w, p + 1);
    } else {
        lstrcpyW(w, p);
    }
    WideCharToMultiByte(CP_ACP, 0, w, -1, n, MAX_PATH * 2, NULL, NULL);
}

#define ST_ROW(fn, lbl, T, CALL_DLL, CALL_REF) do {                                  \
        struct T d1, d2;                                                             \
        memset(&d1, 'X', sizeof d1); memset(&d2, 'X', sizeof d2);                    \
        *_errno() = 0; *ref__errno() = 0;                                            \
        *__doserrno() = 0; *ref___doserrno() = 0;                                    \
        g_iph_dll = g_iph_ref = 0;                                                   \
        int r1 = CALL_DLL, r2 = CALL_REF;                                            \
        grade2(#fn, lbl, r1, r2);                                                    \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                         \
        grade2(#fn ".doserrno", lbl, (long long)*__doserrno(), (long long)*ref___doserrno()); \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                               \
        grade2(#fn ".rec", lbl, fnv(&d1, sizeof d1), fnv(&d2, sizeof d2));           \
    } while (0)

/* NULL path and NULL record, then every path. `P` is the path of the right width. */
#define ST_PATH_RUN(fn, T, ch, P) do {                                               \
        if (!st_setup()) {                                                           \
            grade2(#fn ".fixture", "setup", 0, 1); /* fail loud */                   \
            return;                                                                  \
        }                                                                            \
        wchar_t wp[MAX_PATH];                                                        \
        char np[MAX_PATH * 2];                                                       \
        ST_ROW(fn, "NULL", T, ud_##fn((const ch *)0, &d1), ref_##fn((const ch *)0, &d2)); \
        st_path(3, wp, np);                                                          \
        ST_ROW(fn, "nullrec", T, ud_##fn(P, (struct T *)0), ref_##fn(P, (struct T *)0)); \
        ST_ROW(fn, "NULL+nullrec", T, ud_##fn((const ch *)0, (struct T *)0),         \
               ref_##fn((const ch *)0, (struct T *)0));                              \
        for (size_t i = 0; i < ST_NPATHS; i++) {                                     \
            st_path(i, wp, np);                                                      \
            ST_ROW(fn, label_idx(i), T, ud_##fn(P, &d1), ref_##fn(P, &d2));          \
        }                                                                            \
    } while (0)

#define ST_NARROW(fn, T) ST_PATH_RUN(fn, T, char, np)
#define ST_WIDE(fn, T) ST_PATH_RUN(fn, T, wchar_t, wp)

static void run_st__stat32(void)    { install_counting_iph(); ST_NARROW(_stat32, _stat32); }
static void run_st__stat32i64(void) { install_counting_iph(); ST_NARROW(_stat32i64, _stat32i64); }
static void run_st__stat64i32(void) { install_counting_iph(); ST_NARROW(_stat64i32, _stat64i32); }
static void run_st__stat64(void)    { install_counting_iph(); ST_NARROW(_stat64, _stat64);
    /* Narrow only: a byte CP_ACP has no mapping for fails the conversion, and
     * the record is left as it was. */
    ST_ROW(_stat64, "unmapped", _stat64, ud__stat64("\x81\x8d\x8f\x90\x9d", &d1),
           ref__stat64("\x81\x8d\x8f\x90\x9d", &d2));
}
static void run_st__wstat32(void)    { install_counting_iph(); ST_WIDE(_wstat32, _stat32); }
static void run_st__wstat32i64(void) { install_counting_iph(); ST_WIDE(_wstat32i64, _stat32i64); }
static void run_st__wstat64i32(void) { install_counting_iph(); ST_WIDE(_wstat64i32, _stat64i32); }
static void run_st__wstat64(void)    { install_counting_iph(); ST_WIDE(_wstat64, _stat64); }

/* One descriptor kind per side. Each side opens from its own lowio table, so the
 * two descriptors are graded as numbers too: _fstat reports a device's or pipe's
 * descriptor in st_dev. */
enum { STF_FILE, STF_NUL, STF_PIPE_DATA, STF_PIPE_EMPTY, STF_CLOSED, STF_N_OPENED };

static int stf_open(int kind, int ref, int *other)
{
    wchar_t path[MAX_PATH];
    int fds[2];
    int fd = -1;
    *other = -1;
    switch (kind) {
    case STF_FILE:
    case STF_CLOSED:
        lstrcpyW(path, g_st_dir); lstrcatW(path, L"\\t2040.com");
        fd = ref ? ref__wopen(path, _O_RDONLY) : _wopen(path, _O_RDONLY);
        if (kind == STF_CLOSED && fd >= 0) {
            if (ref) ref__close(fd); else _close(fd);
        }
        return fd;
    case STF_NUL:
        return ref ? ref__wopen(L"NUL", _O_RDWR) : _wopen(L"NUL", _O_RDWR);
    case STF_PIPE_DATA:
    case STF_PIPE_EMPTY:
        if ((ref ? ref__pipe(fds, 256, _O_BINARY) : _pipe(fds, 256, _O_BINARY)) != 0)
            return -1;
        if (kind == STF_PIPE_DATA) {
            if (ref) ref__write(fds[1], "abc", 3); else _write(fds[1], "abc", 3);
        }
        *other = fds[1];
        return fds[0];
    }
    return fd;
}

static void stf_close(int fd, int other, int ref)
{
    if (ref) {
        if (fd >= 0) ref__close(fd);
        if (other >= 0) ref__close(other);
    } else {
        if (fd >= 0) _close(fd);
        if (other >= 0) _close(other);
    }
}

#define ST_FSTAT_RUN(fn, T) do {                                                     \
        if (!st_setup()) {                                                           \
            grade2(#fn ".fixture", "setup", 0, 1);                                   \
            return;                                                                  \
        }                                                                            \
        static const char *const kinds[] = { "file", "nul", "pipe+data", "pipe", "closed" }; \
        for (int k = 0; k < STF_N_OPENED; k++) {                                     \
            int o1, o2;                                                              \
            int f1 = stf_open(k, 0, &o1), f2 = stf_open(k, 1, &o2);                  \
            grade2(#fn ".fd", kinds[k], f1, f2);                                     \
            ST_ROW(fn, kinds[k], T, ud_##fn(f1, &d1), ref_##fn(f2, &d2));            \
            if (k != STF_CLOSED) {                                                   \
                ST_ROW(fn, "nullrec", T, ud_##fn(f1, (struct T *)0), ref_##fn(f2, (struct T *)0)); \
                stf_close(f1, o1, 0); stf_close(f2, o2, 1);                          \
            }                                                                        \
        }                                                                            \
        static const int fixed[] = { 0, 1, 2, -1, -2, -3, 2047, 8191, 8192, 100000 }; \
        for (size_t i = 0; i < sizeof fixed / sizeof fixed[0]; i++)                  \
            ST_ROW(fn, label_idx(i), T, ud_##fn(fixed[i], &d1), ref_##fn(fixed[i], &d2)); \
        ST_ROW(fn, "-2+nullrec", T, ud_##fn(-2, (struct T *)0), ref_##fn(-2, (struct T *)0)); \
    } while (0)

static void run_st__fstat32(void)    { install_counting_iph(); ST_FSTAT_RUN(_fstat32, _stat32); }
static void run_st__fstat32i64(void) { install_counting_iph(); ST_FSTAT_RUN(_fstat32i64, _stat32i64); }
static void run_st__fstat64i32(void) { install_counting_iph(); ST_FSTAT_RUN(_fstat64i32, _stat64i32); }
static void run_st__fstat64(void)    { install_counting_iph(); ST_FSTAT_RUN(_fstat64, _stat64); }
#endif /* UCRTDIFF_REF_HAS__stat64 */

/* ---- locale (the locale/ directory and mbstring/mbctype.cpp) ---------------
 *
 * The locale takeover is graded as ONE family: its pieces call each other's
 * internals, so it is deleted and checked together. Every runner walks a corpus
 * of locale names through BOTH sides' setlocale -- legacy names, BCP-47 tags with
 * and without code pages, bare code pages, compound LC_ strings, junk -- and after
 * each grades what its own export reports: the return, errno and handler count
 * for setlocale itself; the lconv, code pages, locale names, ctype tables and
 * multibyte tables for the accessors. strftime/wcsftime/strcoll/_stricoll/_strlwr
 * ride along under setlocale's row: they are still Microsoft's on both sides, but
 * they read the locale data each side's setlocale built, so they grade the
 * initializers end to end. Each runner leaves both sides back in "C". */
#include <locale.h>
#include <mbctype.h>
#include <time.h>

#if UCRTDIFF_REF_HAS_setlocale
char *__cdecl ref_setlocale(int, const char *);
wchar_t *__cdecl ref__wsetlocale(int, const wchar_t *);
_locale_t __cdecl ref__create_locale(int, const char *);
_locale_t __cdecl ref__wcreate_locale(int, const wchar_t *);
void __cdecl ref__free_locale(_locale_t);
_locale_t __cdecl ref__get_current_locale(void);
int __cdecl ref__configthreadlocale(int);
struct lconv *__cdecl ref_localeconv(void);
unsigned int __cdecl ref____lc_codepage_func(void);
unsigned int __cdecl ref____lc_collate_cp_func(void);
wchar_t **__cdecl ref____lc_locale_name_func(void);
int __cdecl ref____mb_cur_max_func(void);
int __cdecl ref____mb_cur_max_l_func(_locale_t);
const unsigned short *__cdecl ref___pctype_func(void);
const unsigned short *__cdecl ref___pwctype_func(void);
int __cdecl ref__getmbcp(void);
int __cdecl ref__setmbcp(int);
unsigned char *__cdecl ref___p__mbctype(void);
unsigned char *__cdecl ref___p__mbcasemap(void);
int __cdecl ref___initialize_lconv_for_unsigned_char(void);
extern const unsigned short ref__wctype[];
size_t __cdecl ref_strftime(char *, size_t, const char *, const struct tm *);
size_t __cdecl ref_wcsftime(wchar_t *, size_t, const wchar_t *, const struct tm *);
int __cdecl ref_strcoll(const char *, const char *);
int __cdecl ref__stricoll(const char *, const char *);
char *__cdecl ref__strlwr(char *);
unsigned char *__cdecl ref__mbsupr(unsigned char *);

extern char *(__cdecl *const ud_setlocale)(int, const char *) __asm__(UD_IMP(setlocale));
extern wchar_t *(__cdecl *const ud__wsetlocale)(int, const wchar_t *) __asm__(UD_IMP(_wsetlocale));
extern _locale_t(__cdecl *const ud__create_locale)(int, const char *) __asm__(UD_IMP(_create_locale));
extern _locale_t(__cdecl *const ud__wcreate_locale)(int, const wchar_t *) __asm__(UD_IMP(_wcreate_locale));
extern void(__cdecl *const ud__free_locale)(_locale_t) __asm__(UD_IMP(_free_locale));
extern _locale_t(__cdecl *const ud__get_current_locale)(void) __asm__(UD_IMP(_get_current_locale));
extern int(__cdecl *const ud__configthreadlocale)(int) __asm__(UD_IMP(_configthreadlocale));
extern struct lconv *(__cdecl *const ud_localeconv)(void) __asm__(UD_IMP(localeconv));
extern unsigned int(__cdecl *const ud____lc_codepage_func)(void) __asm__(UD_IMP(___lc_codepage_func));
extern unsigned int(__cdecl *const ud____lc_collate_cp_func)(void) __asm__(UD_IMP(___lc_collate_cp_func));
extern wchar_t **(__cdecl *const ud____lc_locale_name_func)(void) __asm__(UD_IMP(___lc_locale_name_func));
extern int(__cdecl *const ud____mb_cur_max_func)(void) __asm__(UD_IMP(___mb_cur_max_func));
extern int(__cdecl *const ud____mb_cur_max_l_func)(_locale_t) __asm__(UD_IMP(___mb_cur_max_l_func));
extern const unsigned short *(__cdecl *const ud___pctype_func)(void) __asm__(UD_IMP(__pctype_func));
extern const unsigned short *(__cdecl *const ud___pwctype_func)(void) __asm__(UD_IMP(__pwctype_func));
extern int(__cdecl *const ud__getmbcp)(void) __asm__(UD_IMP(_getmbcp));
extern int(__cdecl *const ud__setmbcp)(int) __asm__(UD_IMP(_setmbcp));
extern unsigned char *(__cdecl *const ud___p__mbctype)(void) __asm__(UD_IMP(__p__mbctype));
extern unsigned char *(__cdecl *const ud___p__mbcasemap)(void) __asm__(UD_IMP(__p__mbcasemap));
extern int(__cdecl *const ud___initialize_lconv_for_unsigned_char)(void) __asm__(UD_IMP(__initialize_lconv_for_unsigned_char));
/* A DATA export: the import slot holds the table's address. */
extern const unsigned short *const ud__wctype __asm__(UD_IMP(_wctype));
extern size_t(__cdecl *const ud_strftime)(char *, size_t, const char *, const struct tm *) __asm__(UD_IMP(strftime));
extern size_t(__cdecl *const ud_wcsftime)(wchar_t *, size_t, const wchar_t *, const struct tm *) __asm__(UD_IMP(wcsftime));
extern int(__cdecl *const ud_strcoll)(const char *, const char *) __asm__(UD_IMP(strcoll));
extern int(__cdecl *const ud__stricoll)(const char *, const char *) __asm__(UD_IMP(_stricoll));
extern char *(__cdecl *const ud__strlwr)(char *) __asm__(UD_IMP(_strlwr));
extern unsigned char *(__cdecl *const ud__mbsupr)(unsigned char *) __asm__(UD_IMP(_mbsupr));

static const char *const g_loc_names[] = {
    "C", "", "English_United States.1252", "English", "american", "american_usa", "ENU",
    "German_Germany.1252", "german", "de-DE", "de-DE.utf8", "en-US.UTF-8", "en-US.utf-8",
    "en-US.xyz-8", ".utf8", ".UTF-8", ".1252", ".932", ".65001", ".", "ja-JP",
    "Japanese_Japan.932", "chs", "zh-Hans-CN", "zh-CN.936", "ko-KR", "fr-FR", "French_France",
    ".ACP", ".OCP", ".acp", "_Germany", "english-us", "English_United States.utf8",
    "french-canadian", "spanish-modern", "swiss", "French_Belgium", "english.65000",
    "english.99999", "tr-TR", "Turkish_Turkey.1254", "ar-SA", "th-TH", "zh-TW", "xx-XX", "junk", "english_united-states", "de_DE", "C.UTF-8",
    "LC_COLLATE=C;LC_CTYPE=German_Germany.1252;LC_MONETARY=C;LC_NUMERIC=C;LC_TIME=C",
    "LC_CTYPE=de-DE;LC_BOGUS=x", "LC_NUMERIC=fr-FR", "LC_MONETARY=de-DE;LC_NUMERIC=C", "LC_=;", "LC_CTYPE=", "LC_TIME;x",
};
enum { LOC_NNAMES = sizeof(g_loc_names) / sizeof(g_loc_names[0]) };

static long long loc_hs(const char *s) { return s ? fnv(s, strlen(s) + 1) : -1; }
static long long loc_hw(const wchar_t *s) { return s ? fnv(s, (wcslen(s) + 1) * sizeof(wchar_t)) : -1; }

/* What one side's accessors report, component by component. */
enum {
    LS_LCONV, LS_CP, LS_COLLATE_CP, LS_NAMES, LS_MB_CUR_MAX, LS_PCTYPE, LS_PWCTYPE,
    LS_MBCP, LS_MBCTYPE, LS_MBCASEMAP, LS_STRFTIME, LS_WCSFTIME, LS_STRCOLL, LS_STRLWR,
    LS_MBSUPR, LS_N
};
static const char *const g_ls_names[LS_N] = {
    "lconv", "codepage", "collate_cp", "locale_names", "mb_cur_max", "pctype", "pwctype",
    "mbcp", "mbctype", "mbcasemap", "strftime", "wcsftime", "strcoll", "strlwr", "mbsupr",
};

#define LOC_DEFINE_SNAP(side)                                                              \
    static void loc_snap_##side(long long *h)                                              \
    {                                                                                      \
        struct lconv *l = side##localeconv();                                              \
        long long a = 0;                                                                   \
        if (l) {                                                                           \
            const char *n[] = { l->decimal_point, l->thousands_sep, l->grouping,          \
                l->int_curr_symbol, l->currency_symbol, l->mon_decimal_point,              \
                l->mon_thousands_sep, l->mon_grouping, l->positive_sign, l->negative_sign }; \
            const wchar_t *w[] = { l->_W_decimal_point, l->_W_thousands_sep,             \
                l->_W_int_curr_symbol, l->_W_currency_symbol, l->_W_mon_decimal_point,     \
                l->_W_mon_thousands_sep, l->_W_positive_sign, l->_W_negative_sign };       \
            for (int i = 0; i < 10; i++) a = a * 31 + loc_hs(n[i]);                        \
            for (int i = 0; i < 8; i++) a = a * 31 + loc_hw(w[i]);                         \
            a = a * 31 + fnv(&l->int_frac_digits, 8);                                      \
        }                                                                                  \
        h[LS_LCONV] = l ? a : -1;                                                          \
        h[LS_CP] = side##___lc_codepage_func();                                            \
        h[LS_COLLATE_CP] = side##___lc_collate_cp_func();                                  \
        wchar_t **names = side##___lc_locale_name_func();                                  \
        a = 0;                                                                             \
        for (int i = 0; i < 6; i++) a = a * 31 + loc_hw(names ? names[i] : 0);             \
        h[LS_NAMES] = a;                                                                   \
        h[LS_MB_CUR_MAX] = side##___mb_cur_max_func();                                     \
        const unsigned short *pc = side##__pctype_func();                                  \
        h[LS_PCTYPE] = pc ? fnv(pc - 127, 383 * sizeof(unsigned short)) : -1;              \
        const unsigned short *pw = side##__pwctype_func();                                 \
        h[LS_PWCTYPE] = pw ? fnv(pw, 256 * sizeof(unsigned short)) : -1;                   \
        h[LS_MBCP] = side##_getmbcp();                                                     \
        h[LS_MBCTYPE] = fnv(side##__p__mbctype(), 257);                                    \
        h[LS_MBCASEMAP] = fnv(side##__p__mbcasemap(), 256);                                \
        static const struct tm t = { 5, 4, 15, 6, 8, 126, 0, 248, 0 };                     \
        static const char *const fmts[] = { "%a|%A|%b|%B|%p", "%c", "%x", "%X", "%#x",     \
                                            "%#c", "%Ec|%Ex" };                            \
        char buf[512];                                                                     \
        a = 0;                                                                             \
        for (int i = 0; i < 7; i++) {                                                      \
            memset(buf, 'X', sizeof buf);                                                  \
            size_t r = side##strftime(buf, sizeof buf, fmts[i], &t);                       \
            a = a * 31 + (long long)r;                                                     \
            a = a * 31 + fnv(buf, r + 1);                                                  \
        }                                                                                  \
        h[LS_STRFTIME] = a;                                                                \
        wchar_t wbuf[256];                                                                 \
        for (int k = 0; k < 256; k++) wbuf[k] = L'X';                                     \
        size_t wr = side##wcsftime(wbuf, 256, L"%a|%A|%b|%B|%p|%c|%x|%X", &t);             \
        h[LS_WCSFTIME] = (long long)wr * 31 + fnv(wbuf, (wr + 1) * sizeof(wchar_t));       \
        static const char *const coll[][2] = { { "a", "B" }, { "apple", "Apple" },         \
            { "\xe4", "b" }, { "co-op", "coop" }, { "", "" }, { "\xe9t\xe9", "ete" },      \
            { "\x81\x40", "\x82\xa0" } };                                                  \
        a = 0;                                                                             \
        for (int i = 0; i < 7; i++) {                                                      \
            int c = side##strcoll(coll[i][0], coll[i][1]);                                 \
            int d = side##_stricoll(coll[i][0], coll[i][1]);                               \
            a = a * 9 + (c > 0) - (c < 0);                                                 \
            a = a * 9 + (d > 0) - (d < 0);                                                 \
        }                                                                                  \
        h[LS_STRCOLL] = a;                                                                 \
        char low[] = "ABC\xc0\xc4\xc9\xd6\xdc\xdd\xde\xff\x8a\x8c";                        \
        side##_strlwr(low);                                                                \
        h[LS_STRLWR] = fnv(low, sizeof low);                                               \
        unsigned char up[] = "abc\x82\xa0\x82\xa1xyz\x81\x40\xe0\xe0";                     \
        side##_mbsupr(up);                                                                 \
        h[LS_MBSUPR] = fnv(up, sizeof up);                                                 \
    }
LOC_DEFINE_SNAP(ud_)
LOC_DEFINE_SNAP(ref_)

/* Put both sides into `name` for `cat`; grade the returns when `row` is set. */
static void loc_set_both(const char *row, int cat, const char *name, const char *lbl)
{
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    char *r1 = ud_setlocale(cat, name), *r2 = ref_setlocale(cat, name);
    if (row) {
        char what[96];
        lstrcpyA(what, row);
        grade2(what, lbl, loc_hs(r1), loc_hs(r2));
        lstrcatA(what, ".errno");
        grade2(what, lbl, *_errno(), *ref__errno());
        lstrcpyA(what, row); lstrcatA(what, ".iph");
        grade2(what, lbl, g_iph_dll, g_iph_ref);
    }
}

static void loc_reset(void)
{
    ud_setlocale(LC_ALL, "C"); ref_setlocale(LC_ALL, "C");
}

static const char *loc_label(int cat, size_t i)
{
    static char buf[24];
    lstrcpyA(buf, label_idx(i));
    buf[5] = '/'; buf[6] = (char)('0' + (cat + 10) % 10); buf[7] = 0;
    if (cat < 0) buf[6] = 'm';
    return buf;
}

/* Every component after every LC_ALL setting, under `row`: the accessor rows
 * grade their own component, setlocale grades them all. */
static void loc_walk(const char *row, int only)
{
    for (size_t i = 0; i < LOC_NNAMES; i++) {
        loc_set_both(0, LC_ALL, g_loc_names[i], 0);
        /* What each name resolved to, on serial: a corpus the guest's NLS cannot
         * resolve would MATCH as NULL on both sides and prove nothing, so the table
         * must never be read without this. */
        if (only < 0)
            DbgPrint("[UCRTDIFF] locale %s: \"%s\" -> \"%s\" cp=%u mb=%d\n", label_idx(i),
                     g_loc_names[i], ud_setlocale(LC_ALL, NULL) ? ud_setlocale(LC_ALL, NULL) : "(null)",
                     ud____lc_codepage_func(), ud____mb_cur_max_func());
        long long h1[LS_N], h2[LS_N];
        loc_snap_ud_(h1);
        loc_snap_ref_(h2);
        for (int c = 0; c < LS_N; c++) {
            if (only >= 0 && c != only)
                continue;
            char what[96];
            lstrcpyA(what, row); lstrcatA(what, "."); lstrcatA(what, g_ls_names[c]);
            grade2(what, label_idx(i), h1[c], h2[c]);
        }
    }
    loc_reset();
}

static void run_loc_setlocale(void)
{
    install_counting_iph();
    for (int cat = -1; cat <= LC_MAX + 1; cat++) {
        for (size_t i = 0; i < LOC_NNAMES; i++)
            loc_set_both("setlocale", cat, g_loc_names[i], loc_label(cat, i));
        loc_set_both("setlocale", cat, NULL, loc_label(cat, LOC_NNAMES));
        loc_reset();
    }
    /* Query after a mixed setting: the compound string, and setting it back. */
    loc_set_both(0, LC_CTYPE, "de-DE", 0);
    loc_set_both(0, LC_TIME, "ja-JP", 0);
    loc_set_both("setlocale", LC_ALL, NULL, "mixed");
    {
        char saved[1024];
        char *q = ud_setlocale(LC_ALL, NULL);
        lstrcpynA(saved, q ? q : "", sizeof saved);
        loc_reset();
        loc_set_both("setlocale", LC_ALL, saved, "restore");
        loc_set_both("setlocale", LC_ALL, NULL, "restored");
    }
    loc_reset();
    loc_walk("setlocale", -1);
}

static void run_loc__wsetlocale(void)
{
    install_counting_iph();
    for (int cat = -1; cat <= LC_MAX + 1; cat++) {
        for (size_t i = 0; i <= LOC_NNAMES; i++) {
            wchar_t w[256];
            const wchar_t *wp = w;
            if (i < LOC_NNAMES)
                MultiByteToWideChar(CP_ACP, 0, g_loc_names[i], -1, w, 256);
            else
                wp = 0;
            *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
            wchar_t *r1 = ud__wsetlocale(cat, wp), *r2 = ref__wsetlocale(cat, wp);
            const char *lbl = loc_label(cat, i);
            grade2("_wsetlocale", lbl, loc_hw(r1), loc_hw(r2));
            grade2("_wsetlocale.errno", lbl, *_errno(), *ref__errno());
            grade2("_wsetlocale.iph", lbl, g_iph_dll, g_iph_ref);
        }
        loc_reset();
    }
}

/* A _locale_t's contents, read through the pinned layout: the public head, the
 * six category strings, and the ctype table. */
static long long loc_hash_locale_t(_locale_t l)
{
    if (!l)
        return -1;
    const char *li = (const char *)l->locinfo;
    const size_t P = sizeof(void *);
    const size_t cat0 = P == 8 ? 32 : 28;
    long long a = 0;
    for (int i = 0; i < 6; i++)
        a = a * 31 + loc_hw(*(wchar_t *const *)(li + cat0 + (size_t)i * 4 * P + P));
    const unsigned short *pc = *(const unsigned short *const *)li;
    a = a * 31 + (pc ? fnv(pc - 127, 383 * sizeof(unsigned short)) : -1);
    a = a * 31 + fnv(li + P, 8); /* _locale_mb_cur_max, _locale_lc_codepage */
    const unsigned char *mb = (const unsigned char *)l->mbcinfo;
    a = a * 31 + (mb ? fnv(mb + 4, 4 + 4 + 12 + 257 + 256) : -1);
    return a;
}

static void run_loc__create_locale(void)
{
    install_counting_iph();
    for (int cat = -1; cat <= LC_MAX + 1; cat++) {
        for (size_t i = 0; i <= LOC_NNAMES; i++) {
            const char *n = i < LOC_NNAMES ? g_loc_names[i] : 0;
            const char *lbl = loc_label(cat, i);
            *_errno() = 0; *ref__errno() = 0;
            _locale_t l1 = ud__create_locale(cat, n), l2 = ref__create_locale(cat, n);
            grade2("_create_locale", lbl, loc_hash_locale_t(l1), loc_hash_locale_t(l2));
            grade2("_create_locale.errno", lbl, *_errno(), *ref__errno());
            if (l1 && l2)
                grade2("_create_locale.mb_cur_max_l", lbl, ud____mb_cur_max_l_func(l1),
                       ref____mb_cur_max_l_func(l2));
            ud__free_locale(l1);
            ref__free_locale(l2);
        }
    }
}

static void run_loc__wcreate_locale(void)
{
    install_counting_iph();
    for (int cat = -1; cat <= LC_MAX + 1; cat++) {
        for (size_t i = 0; i <= LOC_NNAMES; i++) {
            wchar_t w[256];
            const wchar_t *wp = w;
            if (i < LOC_NNAMES)
                MultiByteToWideChar(CP_ACP, 0, g_loc_names[i], -1, w, 256);
            else
                wp = 0;
            const char *lbl = loc_label(cat, i);
            *_errno() = 0; *ref__errno() = 0;
            _locale_t l1 = ud__wcreate_locale(cat, wp), l2 = ref__wcreate_locale(cat, wp);
            grade2("_wcreate_locale", lbl, loc_hash_locale_t(l1), loc_hash_locale_t(l2));
            grade2("_wcreate_locale.errno", lbl, *_errno(), *ref__errno());
            ud__free_locale(l1);
            ref__free_locale(l2);
        }
    }
}

static void run_loc__get_current_locale(void)
{
    install_counting_iph();
    for (size_t i = 0; i < LOC_NNAMES; i++) {
        loc_set_both(0, LC_ALL, g_loc_names[i], 0);
        _locale_t l1 = ud__get_current_locale(), l2 = ref__get_current_locale();
        grade2("_get_current_locale", label_idx(i), loc_hash_locale_t(l1), loc_hash_locale_t(l2));
        /* Still valid after the global locale moves on. */
        loc_reset();
        grade2("_get_current_locale.after", label_idx(i), loc_hash_locale_t(l1),
               loc_hash_locale_t(l2));
        grade2("___mb_cur_max_l_func", label_idx(i), ud____mb_cur_max_l_func(l1),
               ref____mb_cur_max_l_func(l2));
        ud__free_locale(l1);
        ref__free_locale(l2);
    }
    ud__free_locale(0);
    ref__free_locale(0);
    grade2("_free_locale.null", "-", 0, 0);
}

static void run_loc__configthreadlocale(void)
{
    install_counting_iph();
    static const int args[] = { 0, 1, 0, 2, 0, 3, 0, -2, 0, 1, 1, 2, 2, 0 };
    for (size_t i = 0; i < sizeof args / sizeof args[0]; i++) {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        int r1 = ud__configthreadlocale(args[i]), r2 = ref__configthreadlocale(args[i]);
        grade2("_configthreadlocale", label_idx(i), r1, r2);
        grade2("_configthreadlocale.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("_configthreadlocale.iph", label_idx(i), g_iph_dll, g_iph_ref);
    }
    /* A per-thread locale: this thread changes, the global copy does not. */
    ud__configthreadlocale(1); ref__configthreadlocale(1);
    loc_set_both("_configthreadlocale.setlocale", LC_ALL, "de-DE", "perthread");
    ud__configthreadlocale(2); ref__configthreadlocale(2);
    grade2("_configthreadlocale.global_cp", "-", ud____lc_codepage_func(), ref____lc_codepage_func());
    loc_reset();
}

static void run_loc_localeconv(void) { install_counting_iph(); loc_walk("localeconv", LS_LCONV); }
static void run_loc____lc_codepage_func(void) { loc_walk("___lc_codepage_func", LS_CP); }
static void run_loc____lc_collate_cp_func(void) { loc_walk("___lc_collate_cp_func", LS_COLLATE_CP); }
static void run_loc____lc_locale_name_func(void) { loc_walk("___lc_locale_name_func", LS_NAMES); }
static void run_loc____mb_cur_max_func(void) { loc_walk("___mb_cur_max_func", LS_MB_CUR_MAX); }
static void run_loc___pctype_func(void) { loc_walk("__pctype_func", LS_PCTYPE); }
static void run_loc___pwctype_func(void) { loc_walk("__pwctype_func", LS_PWCTYPE); }
static void run_loc___p__mbctype(void) { loc_walk("__p__mbctype", LS_MBCTYPE); }
static void run_loc___p__mbcasemap(void) { loc_walk("__p__mbcasemap", LS_MBCASEMAP); }

static void run_loc__wctype(void)
{
    grade2("_wctype", "table", fnv(ud__wctype, 258 * sizeof(unsigned short)),
           fnv(ref__wctype, 258 * sizeof(unsigned short)));
}

/* _setmbcp's own row (the generated one grades only its return): every code page
 * kind, then what the multibyte accessors see. */
static void run_loc__setmbcp(void)
{
    install_counting_iph();
    /* 932 then 65001 keeps 932's lead flags (the UTF-8 path clears only the first
     * few entries); 1252 twice takes the no-op path. */
    static const int cps[] = { 0, -2, -3, -4, 932, 936, 949, 950, 1252, 1250, 65001, 20127,
                               437, 12345, -5, -1, 1361, 65000, 65536 + 1252, 932, 65001,
                               1252, 1252, 0 };
    int orig1 = ud__getmbcp(), orig2 = ref__getmbcp();
    grade2("_setmbcp+tables.orig", "-", orig1, orig2);
    for (size_t i = 0; i < sizeof cps / sizeof cps[0]; i++) {
        const char *lbl = label_idx(i);
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        int r1 = ud__setmbcp(cps[i]), r2 = ref__setmbcp(cps[i]);
        grade2("_setmbcp+tables", lbl, r1, r2);
        grade2("_setmbcp+tables.errno", lbl, *_errno(), *ref__errno());
        grade2("_setmbcp+tables.iph", lbl, g_iph_dll, g_iph_ref);
        grade2("_setmbcp+tables.getmbcp", lbl, ud__getmbcp(), ref__getmbcp());
        grade2("_setmbcp+tables.mbctype", lbl, fnv(ud___p__mbctype(), 257), fnv(ref___p__mbctype(), 257));
        grade2("_setmbcp+tables.mbcasemap", lbl, fnv(ud___p__mbcasemap(), 256),
               fnv(ref___p__mbcasemap(), 256));
        unsigned char up1[] = "abc\x82\xa0\x82\xa1xyz\x81\x40\xe0\xe0\xa4", up2[sizeof up1];
        memcpy(up2, up1, sizeof up1);
        ud__mbsupr(up1); ref__mbsupr(up2);
        grade2("_setmbcp+tables.mbsupr", lbl, fnv(up1, sizeof up1), fnv(up2, sizeof up2));
    }
    ud__setmbcp(orig1); ref__setmbcp(orig2);
}

static void run_loc___initialize_lconv_for_unsigned_char(void)
{
    grade2("__initialize_lconv_for_unsigned_char", "ret", ud___initialize_lconv_for_unsigned_char(),
           ref___initialize_lconv_for_unsigned_char());
    loc_walk("__initialize_lconv_for_unsigned_char", LS_LCONV);
}
#endif /* UCRTDIFF_REF_HAS_setlocale */

/* Locale DRIVERS for the blocks below, once setlocale itself is no longer graded.
 * After the locale takeover the reference set has no ref_setlocale; the reference
 * instance's locale is our Rust copy (ref_setlocale is an alias of it, see
 * build_internals_set). Driving it is still what a block needs to grade Microsoft's
 * strcoll or _mbschr under a real locale on both sides -- so UCRTDIFF_REF_CALLABLE_X,
 * not UCRTDIFF_REF_HAS_X, which asks whether X itself can be graded. */
#if !UCRTDIFF_REF_HAS_setlocale && UCRTDIFF_REF_CALLABLE_setlocale
extern char *(__cdecl *const ud_setlocale)(int, const char *) __asm__(UD_IMP(setlocale));
static long long loc_hs(const char *s) { return s ? fnv(s, strlen(s) + 1) : -1; }
static void loc_set_both(const char *row, int cat, const char *name, const char *lbl)
{
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    char *r1 = ud_setlocale(cat, name), *r2 = ref_setlocale(cat, name);
    if (row) {
        char what[96];
        lstrcpyA(what, row);
        grade2(what, lbl, loc_hs(r1), loc_hs(r2));
        lstrcatA(what, ".errno");
        grade2(what, lbl, *_errno(), *ref__errno());
        lstrcpyA(what, row); lstrcatA(what, ".iph");
        grade2(what, lbl, g_iph_dll, g_iph_ref);
    }
}
static void loc_reset(void)
{
    ud_setlocale(LC_ALL, "C"); ref_setlocale(LC_ALL, "C");
}
#endif

static void run_ctype(void)
{
    /* The full 0..255 range plus EOF: the 0x80-0xFF half is where the narrow
     * ctype table and the locale's codepage actually decide anything, and it is
     * the half a C-locale-only test would never reach. */
    for (int c = -1; c < 256; c++) {
        char w[8];
        w[0] = 'c'; w[1] = '='; w[2] = (char)('0' + (c / 100 + 256) % 10);
        w[3] = (char)('0' + (c / 10 + 256) % 10); w[4] = (char)('0' + (c + 256) % 10); w[5] = 0;
#if UCRTDIFF_REF_HAS_tolower
        grade("tolower", w, tolower(c), rust_tolower(c), ref_tolower(c));
#endif
#if UCRTDIFF_REF_HAS_toupper
        grade("toupper", w, toupper(c), rust_toupper(c), ref_toupper(c));
#endif
        /* Normalised: the is* family contracts a NONZERO value, not a specific
         * one, and MSVC returns the raw ctype bitmask. */
        /* Each is* row only while its Microsoft reference still exists: the ctype
         * takeover deleted them, and a plain sweep then failed to LINK. Their evidence
         * is the MATCH rows graded by --takeover-check, as for the string family. */
#if UCRTDIFF_REF_HAS_isalpha
        grade("isalpha", w, !!isalpha(c), !!rust_isalpha(c), !!ref_isalpha(c));
#endif
#if UCRTDIFF_REF_HAS_isdigit
        grade("isdigit", w, !!isdigit(c), !!rust_isdigit(c), !!ref_isdigit(c));
#endif
#if UCRTDIFF_REF_HAS_isspace
        grade("isspace", w, !!isspace(c), !!rust_isspace(c), !!ref_isspace(c));
#endif
#if UCRTDIFF_REF_HAS_isupper
        grade("isupper", w, !!isupper(c), !!rust_isupper(c), !!ref_isupper(c));
#endif
#if UCRTDIFF_REF_HAS_ispunct
        grade("ispunct", w, !!ispunct(c), !!rust_ispunct(c), !!ref_ispunct(c));
#endif
    }
}

static void run_string(void)
{
    static const char *const s[] = {
        "", "a", "abc", "ABC", "aBc", "hello world", "  padded  ",
        "\x80\xff mixed", "aaaa", "abcabcabc", "z",
    };
    const unsigned n = sizeof(s) / sizeof(s[0]);
    /* strlen, strspn and strcspn used to be graded here too. They are now TAKEN
     * OVER (the `string` family in tools/ucrt-takeover.yaml): the DLL links the
     * same crt-portable-rust bodies as rust_, and their Microsoft objects are
     * deleted, so there is no reference left to grade against and a DLL-vs-rust_
     * row would compare the code with itself. Their evidence is the MATCH rows in
     * docs/engineering/active-lanes/ucrt-tier2-table.tsv, stamped with the commit
     * they were graded at. */
    for (unsigned i = 0; i < n; i++) {
        for (unsigned j = 0; j < n; j++) {
            grade("strcmp", s[i], sgn(strcmp(s[i], s[j])), sgn(rust_strcmp(s[i], s[j])),
                  sgn(ref_strcmp(s[i], s[j])));
            grade("_stricmp", s[i], sgn(_stricmp(s[i], s[j])), sgn(rust__stricmp(s[i], s[j])),
                  sgn(ref__stricmp(s[i], s[j])));
            grade("strncmp", s[i], sgn(strncmp(s[i], s[j], 3)), sgn(rust_strncmp(s[i], s[j], 3)),
                  sgn(ref_strncmp(s[i], s[j], 3)));
        }
    }
}

/* The string WRITERS and the pointer-returning searcher, which the generator has no
 * driver for (it only compares integer returns). Hand-written for the same reason
 * run_string is. Each side writes into its OWN buffer, pre-filled with the same
 * sentinel pattern, and the graded value folds together the returned pointer's
 * OFFSET into that side's own buffer and an FNV-1a hash of the WHOLE buffer -- so a
 * wrong return, a missing terminator, strncpy's zero padding, or a stray write past
 * the end all change it. Comparing raw pointers would be meaningless: three buffers. */
#define WBUF 48
static long long wfold(const char *buf, const char *ret)
{
    unsigned long long h = 1469598103934665603ULL;
    for (int k = 0; k < WBUF; k++) {
        h ^= (unsigned char)buf[k];
        h *= 1099511628211ULL;
    }
    long long off = ret ? (long long)(ret - buf) : -1;
    return (long long)(h ^ ((unsigned long long)(off + 2) << 56));
}
static void wfill(char *b, const char *init)
{
    for (int k = 0; k < WBUF; k++)
        b[k] = (char)(0xA5 ^ k);
    if (init)
        memcpy(b, init, strlen(init) + 1);
}

static long long poff(const char *base, const char *ret)
{
    return ret ? (long long)(ret - base) : -1;
}

static const char *const wcorp[] = {
    "", "a", "abc", "ABC", "hello world", "\x80\xff mixed", "abcabcabc", "z",
};
static const size_t wns[] = { 0, 1, 2, 3, 5, 11, 20 };
#define WN (sizeof(wcorp) / sizeof(wcorp[0]))
#define WNS (sizeof(wns) / sizeof(wns[0]))

/* Each runner below is compiled only while the reference set still defines its
 * ref_ body (generated-refs.h, written from the archive by the generator). Once a
 * takeover deletes Microsoft's object the reference has none, a reference to it would
 * stop the harness linking, and grading dll-vs-rust_ alone would compare the
 * taken-over code with itself. So a missing reference skips the export with NO row --
 * and the durable table, a union, keeps the real MATCH row it earned, stamped with the
 * commit it was graded at. (Not weak symbols: a weak undefined never pulls an archive
 * member, so it read NULL even while the object was in the archive.)
 *
 * strnlen is deliberately absent: on arm64 -- the only arch Tier 2 runs on -- Microsoft's
 * strnlen.cpp defines nothing (the export is UNBACKED there), so there is no ref_strnlen
 * to grade against. Its in-guest evidence is owed, not assumed.
 *
 * One runner per export, so each gets its own table row keyed by its real name --
 * that row is the evidence a takeover cites. Every corpus pair fits in WBUF. */
#if UCRTDIFF_REF_HAS_strcat
static void run_strcat(void)
{
    char a[WBUF], b[WBUF], c[WBUF];
    for (unsigned i = 0; i < WN; i++)
        for (unsigned j = 0; j < WN; j++) {
            wfill(a, wcorp[i]); wfill(b, wcorp[i]); wfill(c, wcorp[i]);
            grade("strcat", wcorp[i], wfold(a, strcat(a, wcorp[j])),
                  wfold(b, rust_strcat(b, wcorp[j])), wfold(c, ref_strcat(c, wcorp[j])));
        }
}
#endif
#if UCRTDIFF_REF_HAS_strcpy
static void run_strcpy(void)
{
    char a[WBUF], b[WBUF], c[WBUF];
    for (unsigned j = 0; j < WN; j++) {
        wfill(a, 0); wfill(b, 0); wfill(c, 0);
        grade("strcpy", wcorp[j], wfold(a, strcpy(a, wcorp[j])),
              wfold(b, rust_strcpy(b, wcorp[j])), wfold(c, ref_strcpy(c, wcorp[j])));
    }
}
#endif
#if UCRTDIFF_REF_HAS_strncat
static void run_strncat(void)
{
    char a[WBUF], b[WBUF], c[WBUF];
    for (unsigned i = 0; i < WN; i++)
        for (unsigned j = 0; j < WN; j++)
            for (unsigned k = 0; k < WNS; k++) {
                size_t m = wns[k];
                wfill(a, wcorp[i]); wfill(b, wcorp[i]); wfill(c, wcorp[i]);
                grade("strncat", wcorp[i], wfold(a, strncat(a, wcorp[j], m)),
                      wfold(b, rust_strncat(b, wcorp[j], m)),
                      wfold(c, ref_strncat(c, wcorp[j], m)));
            }
}
#endif
#if UCRTDIFF_REF_HAS_strncpy
static void run_strncpy(void)
{
    char a[WBUF], b[WBUF], c[WBUF];
    for (unsigned j = 0; j < WN; j++)
        for (unsigned k = 0; k < WNS; k++) {
            size_t m = wns[k];
            wfill(a, 0); wfill(b, 0); wfill(c, 0);
            grade("strncpy", wcorp[j], wfold(a, strncpy(a, wcorp[j], m)),
                  wfold(b, rust_strncpy(b, wcorp[j], m)), wfold(c, ref_strncpy(c, wcorp[j], m)));
        }
}
#endif
#if UCRTDIFF_REF_HAS_strpbrk
static void run_strpbrk(void)
{
    /* Offset into the SAME input on every side: comparing it is exact, and it never
     * reads past the string the way hashing a fixed window would. */
    for (unsigned i = 0; i < WN; i++)
        for (unsigned j = 0; j < WN; j++)
            grade("strpbrk", wcorp[i], poff(wcorp[i], strpbrk(wcorp[i], wcorp[j])),
                  poff(wcorp[i], rust_strpbrk(wcorp[i], wcorp[j])),
                  poff(wcorp[i], ref_strpbrk(wcorp[i], wcorp[j])));
}
#endif

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

/* The REFERENCE side's CRT startup. ref_ is a private copy of Microsoft's UCRT
 * objects with only the EXPORTS renamed, so its internals -- including this, the
 * table ucrtbase's own DllMain runs -- keep their names and resolve to the private
 * copy. Nothing called it before, so ref_ ran with a zeroed lock table: a zeroed
 * CRITICAL_SECTION reads as HELD (LockCount 0; unlocked is -1), and the first
 * ref_ call that takes __acrt_lock -- ref_setlocale -- waited on an owner that
 * does not exist until ntdll raised STATUS_POSSIBLE_DEADLOCK (0xC0000194). It is
 * also why the ref_ side's multibyte state never matched the DLL's: no
 * __acrt_initialize_multibyte either. */
unsigned char __cdecl __acrt_initialize(void); /* C++ `bool`: one byte, same ABI */

static int init_ref_crt(void)
{
    if (!__acrt_initialize()) {
        DbgPrint("[UCRTDIFF] FATAL: the reference UCRT's __acrt_initialize failed\n");
        return 0;
    }
    DbgPrint("[UCRTDIFF] ref CRT initialized\n");
    return 1;
}

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

/* The three hand-written runners predate the generator, and they were the last code in
 * the harness still calling grade() UNGUARDED on the main thread: no fault guard, no
 * hang timeout, no shard gate and no deadline check. That gap is not theoretical --
 * shard 0/8 graded all 25 of its generated exports cleanly and then the GUEST died in
 * here, with no row naming what did it and no `done` line to say the run was cut short.
 *
 * Routing them through export_begin/run_guarded/export_end gives them exactly what every
 * generated export gets, and gates them by shard too, so they run in ONE shard instead
 * of being repeated in all of them. */

/* ==== staging/convtime.c, inlined ==== */
/*
 * convtime -- guest grader for the "ctime" batch: convert/atof.cpp,
 * time/ctime.cpp, time/timeset.cpp, startup/argv_data.cpp.
 *
 * Included verbatim into ucrtdiff.c just before `run_legacy`, exactly like the
 * FP_HAND/ST_HAND/LOC_HAND blocks above it: every helper this file uses
 * (grade2, install_counting_iph, label_idx, fnv, g_iph_dll/g_iph_ref, _errno,
 * ref__errno, UD_IMP) is already in scope by inclusion point, so none of it is
 * redeclared here.
 *
 * Four separate guards, one per C file that may be deleted independently:
 *   UCRTDIFF_REF_HAS_atof         -- convert/atof.cpp
 *   UCRTDIFF_REF_HAS__ctime32     -- time/ctime.cpp
 *   UCRTDIFF_REF_HAS__get_daylight-- time/timeset.cpp
 *   UCRTDIFF_REF_HAS___p___argc   -- startup/argv_data.cpp
 *
 * NONE of these four macros exist in tools/gen_ucrt_tier2_cases.py's
 * HAND_GRADED tuple yet (this is a brand-new batch of hand-written runners,
 * like FP_HAND/ST_HAND/LOC_HAND were when they were added) -- the integrator
 * must add the four representative names above to HAND_GRADED and regenerate
 * generated-refs.h for all three arches before these guards read as 1. Until
 * then they read as 0 (an undefined macro in `#if` is 0), which is exactly
 * the "guards=0" case this file was syntax-checked under.
 *
 * All four `ref_<name>` symbols this file declares were confirmed present in
 * build-{i386,amd64,arm64}/libucrt_ref.a with llvm-nm (i386 carries the extra
 * leading underscore llvm-nm shows separately from the bare name; all three
 * archives were checked). None of atof/ctime's or timeset's/argv-data's DLL
 * side spellings are wrapped in a mingw __CRT_INLINE (checked against
 * generic-w64-mingw32/include/{stdlib,time,corecrt_startup}.h), so every
 * DLL-side call below is a PLAIN name, not a ud_ import pointer -- there is no
 * inline body for the compiler to substitute instead.
 *
 * __wide_tzname, __acrt_initialize_timeset, __acrt_initialize_command_line and
 * __acrt_uninitialize_command_line are NOT in ucrtbase.spec (no export, hence
 * no ud_ or ref_ symbol either), so none of the four is graded here; they are
 * exercised only indirectly (the first through __tzname's pointee, the other
 * three through argv-data's globals already having run once by the time
 * ucrtdiff_entry starts -- see run_convtime___p__acmdln's comment).
 */

/* ==== atof (convert/atof.cpp) ============================================ */
#if UCRTDIFF_REF_HAS_atof
double __cdecl ref_atof(const char *);
double __cdecl ref__atof_l(const char *, _locale_t);
double __cdecl ref__wtof(const wchar_t *);
double __cdecl ref__wtof_l(const wchar_t *, _locale_t);
int __cdecl ref__atoflt(_CRT_FLOAT *, const char *);
int __cdecl ref__atoflt_l(_CRT_FLOAT *, const char *, _locale_t);
int __cdecl ref__atodbl(_CRT_DOUBLE *, char *);
int __cdecl ref__atodbl_l(_CRT_DOUBLE *, char *, _locale_t);

static const char *const g_atof_in[] = {
    "0", "-0", "3.14159", "  \t 2.5e10xyz", "1e400", "-1e400", "1e-400",
    "4.9406564584124654e-324", "abc", "", "inf", "-infinity", "nan", "-nan(ind)",
    "007.5", "0x1.8p3", "  ",
};
#define ATOF_N (sizeof(g_atof_in) / sizeof(g_atof_in[0]))

static const wchar_t *const g_atof_win[] = {
    L"0", L"-0", L"3.14159", L"  \t 2.5e10xyz", L"1e400", L"-1e400", L"1e-400",
    L"4.9406564584124654e-324", L"abc", L"", L"inf", L"-infinity", L"nan", L"-nan(ind)",
    L"007.5", L"0x1.8p3", L"  ",
};
#define ATOF_WN (sizeof(g_atof_win) / sizeof(g_atof_win[0]))

/* `fp_bits32`/`fp_bits64`/`FP_BITS` (defined for the strtod block, above) are
 * NOT reusable: convert/strtod.cpp is already taken over and deleted, so
 * `UCRTDIFF_REF_HAS_strtod` -- and everything inside that `#if`, including
 * those three -- currently reads 0 in every arch's generated-refs.h. Local,
 * independently-guarded equivalents instead. */
static unsigned convtime_fp_bits32(float f) { unsigned u; memcpy(&u, &f, 4); return u; }
static unsigned long long convtime_fp_bits64(double d) { unsigned long long u; memcpy(&u, &d, 8); return u; }
#define CONVTIME_FP_BITS(v) _Generic((v), float: (long long)convtime_fp_bits32(v), default: (long long)convtime_fp_bits64((double)(v)))

/* No end pointer and no traits argument (unlike FP_ROW) -- atof/_wtof discard
 * the parser's status entirely, so only the value, errno and handler count are
 * comparable. */
#define ATOF_ROW(fn, lbl, s, ...) do {                                         \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        long long v1 = CONVTIME_FP_BITS(fn(s, ##__VA_ARGS__));                 \
        long long v2 = CONVTIME_FP_BITS(ref_##fn(s, ##__VA_ARGS__));           \
        grade2(#fn, lbl, v1, v2);                                              \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

static void run_convtime_atof(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATOF_ROW(atof, g_atof_in[i], g_atof_in[i]);
    ATOF_ROW(atof, "NULL", (const char *)0);
}

static void run_convtime__atof_l(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATOF_ROW(_atof_l, g_atof_in[i], g_atof_in[i], NULL);
    ATOF_ROW(_atof_l, "NULL", (const char *)0, NULL);
}

static void run_convtime__wtof(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_WN; i++)
        ATOF_ROW(_wtof, label_idx(i), g_atof_win[i]);
    ATOF_ROW(_wtof, "NULL", (const wchar_t *)0);
}

static void run_convtime__wtof_l(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_WN; i++)
        ATOF_ROW(_wtof_l, label_idx(i), g_atof_win[i], NULL);
    ATOF_ROW(_wtof_l, "NULL", (const wchar_t *)0, NULL);
}

/* _atoflt/_atodbl grade the out-struct's bytes AND the returned code (0 /
 * _OVERFLOW=3 / _UNDERFLOW=4 / _DOMAIN=1). Each struct is sentinel-prefilled
 * (-1.0) so an untouched-on-failure struct would show. Deliberately NOT
 * tested with a NULL string: common_atodbl_l (atof.cpp) validates only
 * `result`, never `string` -- passing a NULL string dereferences null on
 * BOTH sides (a genuine Microsoft quirk, kept faithfully in
 * ucrt-rust/src/atof.rs's port) and there is no safe way to grade a call that
 * is expected to crash. */
/* Cast `s` to `char *` at the call, same as ATODBL_ROW: Microsoft's own
 * `_atoflt`/`_atoflt_l` take `char const*` (stdlib.h), but the mingw header
 * this file's DLL-side calls resolve through declares them with `char *_Str`
 * instead -- a mingw header quirk, not a UCRT one, and the cast is needed
 * only to keep that header happy. */
#define ATOFLT_ROW(fn, lbl, s, ...) do {                                       \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        _CRT_FLOAT r1, r2;                                                     \
        r1.f = -1.0f; r2.f = -1.0f;                                            \
        int c1 = fn(&r1, (char *)(s), ##__VA_ARGS__);                          \
        int c2 = ref_##fn(&r2, (char *)(s), ##__VA_ARGS__);                    \
        grade2(#fn, lbl, c1, c2);                                              \
        grade2(#fn ".val", lbl, (long long)convtime_fp_bits32(r1.f), (long long)convtime_fp_bits32(r2.f)); \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

#define ATODBL_ROW(fn, lbl, s, ...) do {                                       \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        _CRT_DOUBLE r1, r2;                                                    \
        r1.x = -1.0; r2.x = -1.0;                                              \
        int c1 = fn(&r1, (char *)(s), ##__VA_ARGS__);                          \
        int c2 = ref_##fn(&r2, (char *)(s), ##__VA_ARGS__);                    \
        grade2(#fn, lbl, c1, c2);                                              \
        grade2(#fn ".val", lbl, (long long)convtime_fp_bits64(r1.x), (long long)convtime_fp_bits64(r2.x)); \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

/* `result == NULL` is validated FIRST in `common_atodbl_l` (`_VALIDATE_RETURN`,
 * handler-calling) and short-circuits before `string` is ever touched, so it
 * is the one row of this family safe to run with a literal NULL pointer where
 * ATOFLT_ROW/ATODBL_ROW pass `&r1`/`&r2`. Written out directly rather than
 * routed through those macros, which always take the struct's address. */
#define ATOF_NULLRESULT_ROW(kind, fn, ...) do {                                \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        int c1 = fn((kind *)0, "3.5", ##__VA_ARGS__);                          \
        int c2 = ref_##fn((kind *)0, "3.5", ##__VA_ARGS__);                    \
        grade2(#fn ".nullresult", "-", c1, c2);                                \
        grade2(#fn ".nullresult.errno", "-", *_errno(), *ref__errno());        \
        grade2(#fn ".nullresult.iph", "-", g_iph_dll, g_iph_ref);              \
    } while (0)

static void run_convtime__atoflt(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATOFLT_ROW(_atoflt, g_atof_in[i], g_atof_in[i]);
    ATOF_NULLRESULT_ROW(_CRT_FLOAT, _atoflt);
}

static void run_convtime__atoflt_l(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATOFLT_ROW(_atoflt_l, g_atof_in[i], g_atof_in[i], NULL);
    ATOF_NULLRESULT_ROW(_CRT_FLOAT, _atoflt_l, NULL);
}

static void run_convtime__atodbl(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATODBL_ROW(_atodbl, g_atof_in[i], g_atof_in[i]);
    ATOF_NULLRESULT_ROW(_CRT_DOUBLE, _atodbl);
}

static void run_convtime__atodbl_l(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < ATOF_N; i++)
        ATODBL_ROW(_atodbl_l, g_atof_in[i], g_atof_in[i], NULL);
    ATOF_NULLRESULT_ROW(_CRT_DOUBLE, _atodbl_l, NULL);
}
#endif /* UCRTDIFF_REF_HAS_atof */

/* ==== ctime (time/ctime.cpp) ============================================= */
#if UCRTDIFF_REF_HAS__ctime32
errno_t __cdecl ref__ctime32_s(char *, size_t, const __time32_t *);
errno_t __cdecl ref__wctime32_s(wchar_t *, size_t, const __time32_t *);
errno_t __cdecl ref__ctime64_s(char *, size_t, const __time64_t *);
errno_t __cdecl ref__wctime64_s(wchar_t *, size_t, const __time64_t *);
char *__cdecl ref__ctime32(const __time32_t *);
wchar_t *__cdecl ref__wctime32(const __time32_t *);
char *__cdecl ref__ctime64(const __time64_t *);
wchar_t *__cdecl ref__wctime64(const __time64_t *);

/* 0, one day in, a mid-range value, __crt_time_time_t_traits<__time32_t>::max_time_t
 * (0x7fffd27f -- 23:59:59 01/18/2038 UTC), and -1 (the one validated-without-a-
 * handler case, `_VALIDATE_RETURN_ERRCODE_NOEXC(*time_t_value >= 0, EINVAL)`). */
static const __time32_t g_ctime32_vals[] = { 0, 86400, 1000000000L, 0x7fffd27fL, -1 };
#define CTIME32_N (sizeof(g_ctime32_vals) / sizeof(g_ctime32_vals[0]))
/* Same shape, plus a value only a 64-bit time_t can hold (past 2^31) and
 * __crt_time_time_t_traits<__time64_t>::max_time_t (0x793582aff). */
static const __time64_t g_ctime64_vals[] = {
    0, 86400, 1000000000LL, 2147483648LL, 0x793582affLL, -1,
};
#define CTIME64_N (sizeof(g_ctime64_vals) / sizeof(g_ctime64_vals[0]))

/* Every buffer is prefilled with 'X' so an untouched tail, or the release-build
 * `_RESET_STRING`'s single leading NUL, both show plainly. 32 narrow / 32 wide
 * elements is comfortably over the 26-char contract with room for a too-small
 * probe at size 10. */
#define CT_ROW_S(fn, T, CH, val, size) do {                                    \
        CH b1[32], b2[32];                                                     \
        for (int _k = 0; _k < 32; _k++) { b1[_k] = (CH)'X'; b2[_k] = (CH)'X'; } \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        T tv = (val);                                                          \
        errno_t c1 = fn(b1, (size), &tv);                                      \
        errno_t c2 = ref_##fn(b2, (size), &tv);                                \
        grade2(#fn, label_idx((unsigned)(val) & 0xff), c1, c2);                \
        grade2(#fn ".val", label_idx((unsigned)(val) & 0xff), fnv(b1, 32 * sizeof(CH)), fnv(b2, 32 * sizeof(CH))); \
        grade2(#fn ".errno", label_idx((unsigned)(val) & 0xff), *_errno(), *ref__errno()); \
        grade2(#fn ".iph", label_idx((unsigned)(val) & 0xff), g_iph_dll, g_iph_ref); \
    } while (0)

static void run_convtime__ctime32_s(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME32_N; i++)
        CT_ROW_S(_ctime32_s, __time32_t, char, g_ctime32_vals[i], 32);
    /* buffer too small (>0 but <26): still NUL-reset at [0] (release
     * `_RESET_STRING`), then EINVAL. */
    {
        char b1[32], b2[32];
        for (int k = 0; k < 32; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        __time32_t tv = 0;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _ctime32_s(b1, 10, &tv);
        errno_t c2 = ref__ctime32_s(b2, 10, &tv);
        grade2("_ctime32_s.small", "-", c1, c2);
        grade2("_ctime32_s.small.val", "-", fnv(b1, 32), fnv(b2, 32));
        grade2("_ctime32_s.small.errno", "-", *_errno(), *ref__errno());
    }
    /* NULL buffer, NULL time, zero size -- each its own handler-calling
     * validation in `common_ctime_s`. */
    {
        __time32_t tv = 0;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _ctime32_s((char *)0, 32, &tv);
        errno_t c2 = ref__ctime32_s((char *)0, 32, &tv);
        grade2("_ctime32_s.nullbuf", "-", c1, c2);
        grade2("_ctime32_s.nullbuf.errno", "-", *_errno(), *ref__errno());
        grade2("_ctime32_s.nullbuf.iph", "-", g_iph_dll, g_iph_ref);

        char b1[32], b2[32];
        for (int k = 0; k < 32; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        c1 = _ctime32_s(b1, 32, (const __time32_t *)0);
        c2 = ref__ctime32_s(b2, 32, (const __time32_t *)0);
        grade2("_ctime32_s.nulltime", "-", c1, c2);
        grade2("_ctime32_s.nulltime.val", "-", fnv(b1, 32), fnv(b2, 32));
        grade2("_ctime32_s.nulltime.errno", "-", *_errno(), *ref__errno());

        for (int k = 0; k < 32; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        c1 = _ctime32_s(b1, 0, &tv);
        c2 = ref__ctime32_s(b2, 0, &tv);
        grade2("_ctime32_s.zerosize", "-", c1, c2);
        grade2("_ctime32_s.zerosize.val", "-", fnv(b1, 32), fnv(b2, 32));
        grade2("_ctime32_s.zerosize.errno", "-", *_errno(), *ref__errno());
    }
}

static void run_convtime__wctime32_s(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME32_N; i++)
        CT_ROW_S(_wctime32_s, __time32_t, wchar_t, g_ctime32_vals[i], 32);
}

static void run_convtime__ctime64_s(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME64_N; i++)
        CT_ROW_S(_ctime64_s, __time64_t, char, g_ctime64_vals[i], 32);
}

static void run_convtime__wctime64_s(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME64_N; i++)
        CT_ROW_S(_wctime64_s, __time64_t, wchar_t, g_ctime64_vals[i], 32);
}

/* The pointer-returning forms: grade the STRING CONTENT at the returned
 * pointer (never the pointer itself -- ctime.rs's buffer and asctime.cpp's
 * thread-local buffer are two different addresses on the two sides), the
 * null-ness, and errno/iph for the null-time and negative-time cases. */
#define CT_PTR_ROW(fn, T, CH, WLEN, val) do {                                  \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        T tv = (val);                                                         \
        CH *p1 = fn(&tv);                                                     \
        CH *p2 = ref_##fn(&tv);                                               \
        grade2(#fn ".isnull", label_idx((unsigned)(val) & 0xff), p1 == 0, p2 == 0); \
        if (p1 && p2)                                                         \
            grade2(#fn ".val", label_idx((unsigned)(val) & 0xff),             \
                   fnv(p1, (WLEN(p1) + 1) * sizeof(CH)), fnv(p2, (WLEN(p2) + 1) * sizeof(CH))); \
        grade2(#fn ".errno", label_idx((unsigned)(val) & 0xff), *_errno(), *ref__errno()); \
        grade2(#fn ".iph", label_idx((unsigned)(val) & 0xff), g_iph_dll, g_iph_ref); \
    } while (0)
#define CT_STRLEN(p) strlen(p)
#define CT_WCSLEN(p) wcslen(p)

static void run_convtime__ctime32(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME32_N; i++)
        CT_PTR_ROW(_ctime32, __time32_t, char, CT_STRLEN, g_ctime32_vals[i]);
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        char *p1 = _ctime32((const __time32_t *)0);
        char *p2 = ref__ctime32((const __time32_t *)0);
        grade2("_ctime32.nulltime", "-", p1 == 0, p2 == 0);
        grade2("_ctime32.nulltime.errno", "-", *_errno(), *ref__errno());
        grade2("_ctime32.nulltime.iph", "-", g_iph_dll, g_iph_ref);
    }
}

static void run_convtime__wctime32(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME32_N; i++)
        CT_PTR_ROW(_wctime32, __time32_t, wchar_t, CT_WCSLEN, g_ctime32_vals[i]);
}

static void run_convtime__ctime64(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME64_N; i++)
        CT_PTR_ROW(_ctime64, __time64_t, char, CT_STRLEN, g_ctime64_vals[i]);
}

static void run_convtime__wctime64(void)
{
    install_counting_iph();
    for (unsigned i = 0; i < CTIME64_N; i++)
        CT_PTR_ROW(_wctime64, __time64_t, wchar_t, CT_WCSLEN, g_ctime64_vals[i]);
}
#endif /* UCRTDIFF_REF_HAS__ctime32 */

/* ==== timeset (time/timeset.cpp) ========================================= */
#if UCRTDIFF_REF_HAS__get_daylight
errno_t __cdecl ref__get_daylight(int *);
errno_t __cdecl ref__get_dstbias(long *);
errno_t __cdecl ref__get_timezone(long *);
errno_t __cdecl ref__get_tzname(size_t *, char *, size_t, int);
int *__cdecl ref___daylight(void);
long *__cdecl ref___dstbias(void);
long *__cdecl ref___timezone(void);
char **__cdecl ref___tzname(void);

static void run_convtime__get_daylight(void)
{
    install_counting_iph();
    int v1 = -999, v2 = -999;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    errno_t c1 = _get_daylight(&v1);
    errno_t c2 = ref__get_daylight(&v2);
    grade2("_get_daylight", "-", c1, c2);
    grade2("_get_daylight.val", "-", v1, v2);
    grade2("_get_daylight.errno", "-", *_errno(), *ref__errno());

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _get_daylight((int *)0);
    c2 = ref__get_daylight((int *)0);
    grade2("_get_daylight.null", "-", c1, c2);
    grade2("_get_daylight.null.errno", "-", *_errno(), *ref__errno());
    grade2("_get_daylight.null.iph", "-", g_iph_dll, g_iph_ref);
}

static void run_convtime__get_dstbias(void)
{
    install_counting_iph();
    long v1 = -999, v2 = -999;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    errno_t c1 = _get_dstbias(&v1);
    errno_t c2 = ref__get_dstbias(&v2);
    grade2("_get_dstbias", "-", c1, c2);
    grade2("_get_dstbias.val", "-", v1, v2);
    grade2("_get_dstbias.errno", "-", *_errno(), *ref__errno());

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _get_dstbias((long *)0);
    c2 = ref__get_dstbias((long *)0);
    grade2("_get_dstbias.null", "-", c1, c2);
    grade2("_get_dstbias.null.errno", "-", *_errno(), *ref__errno());
    grade2("_get_dstbias.null.iph", "-", g_iph_dll, g_iph_ref);
}

static void run_convtime__get_timezone(void)
{
    install_counting_iph();
    long v1 = -999, v2 = -999;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    errno_t c1 = _get_timezone(&v1);
    errno_t c2 = ref__get_timezone(&v2);
    grade2("_get_timezone", "-", c1, c2);
    grade2("_get_timezone.val", "-", v1, v2);
    grade2("_get_timezone.errno", "-", *_errno(), *ref__errno());

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _get_timezone((long *)0);
    c2 = ref__get_timezone((long *)0);
    grade2("_get_timezone.null", "-", c1, c2);
    grade2("_get_timezone.null.errno", "-", *_errno(), *ref__errno());
    grade2("_get_timezone.null.iph", "-", g_iph_dll, g_iph_ref);
}

static void run_convtime__get_tzname(void)
{
    install_counting_iph();
    char b1[80], b2[80];
    for (int idx = 0; idx <= 1; idx++) {
        size_t n1 = (size_t)-1, n2 = (size_t)-1;
        for (int k = 0; k < 80; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _get_tzname(&n1, b1, sizeof(b1), idx);
        errno_t c2 = ref__get_tzname(&n2, b2, sizeof(b2), idx);
        grade2("_get_tzname", label_idx((unsigned)idx), c1, c2);
        grade2("_get_tzname.len", label_idx((unsigned)idx), (long long)n1, (long long)n2);
        grade2("_get_tzname.val", label_idx((unsigned)idx),
               fnv(b1, n1 < sizeof(b1) ? n1 : sizeof(b1)), fnv(b2, n2 < sizeof(b2) ? n2 : sizeof(b2)));
        grade2("_get_tzname.errno", label_idx((unsigned)idx), *_errno(), *ref__errno());
    }
    /* size query: buffer NULL, size 0 -- fills *length, returns 0, never
     * touches strcpy_s. */
    {
        size_t n1 = (size_t)-1, n2 = (size_t)-1;
        *_errno() = 0; *ref__errno() = 0;
        errno_t c1 = _get_tzname(&n1, (char *)0, 0, 0);
        errno_t c2 = ref__get_tzname(&n2, (char *)0, 0, 0);
        grade2("_get_tzname.sizequery", "-", c1, c2);
        grade2("_get_tzname.sizequery.len", "-", (long long)n1, (long long)n2);
    }
    /* bad index (handler-calling). */
    {
        size_t n1 = (size_t)-1, n2 = (size_t)-1;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _get_tzname(&n1, b1, sizeof(b1), 2);
        errno_t c2 = ref__get_tzname(&n2, b2, sizeof(b2), 2);
        grade2("_get_tzname.badindex", "-", c1, c2);
        grade2("_get_tzname.badindex.errno", "-", *_errno(), *ref__errno());
        grade2("_get_tzname.badindex.iph", "-", g_iph_dll, g_iph_ref);
    }
    /* NULL length. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _get_tzname((size_t *)0, b1, sizeof(b1), 0);
        errno_t c2 = ref__get_tzname((size_t *)0, b2, sizeof(b2), 0);
        grade2("_get_tzname.nulllen", "-", c1, c2);
        grade2("_get_tzname.nulllen.errno", "-", *_errno(), *ref__errno());
        grade2("_get_tzname.nulllen.iph", "-", g_iph_dll, g_iph_ref);
    }
    /* buffer!=NULL but size==0 -- the first validation's "either both or
     * neither" contract. */
    {
        size_t n1 = (size_t)-1, n2 = (size_t)-1;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _get_tzname(&n1, b1, 0, 0);
        errno_t c2 = ref__get_tzname(&n2, b2, 0, 0);
        grade2("_get_tzname.badsize", "-", c1, c2);
        grade2("_get_tzname.badsize.errno", "-", *_errno(), *ref__errno());
    }
    /* Too-small buffer -> ERANGE, buffer[0] already reset to NUL first. */
    {
        size_t n1 = (size_t)-1, n2 = (size_t)-1;
        for (int k = 0; k < 80; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0;
        errno_t c1 = _get_tzname(&n1, b1, 1, 0);
        errno_t c2 = ref__get_tzname(&n2, b2, 1, 0);
        grade2("_get_tzname.small", "-", c1, c2);
        grade2("_get_tzname.small.val", "-", fnv(b1, 1), fnv(b2, 1));
    }
}

/* Pointer-returning accessors: only the POINTEE is comparable (the addresses
 * are two different processes' worth of static storage). */
static void run_convtime___daylight(void)
{
    int *p1 = __daylight();
    int *p2 = ref___daylight();
    grade2("__daylight.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__daylight.val", "-", *p1, *p2);
}

static void run_convtime___dstbias(void)
{
    long *p1 = __dstbias();
    long *p2 = ref___dstbias();
    grade2("__dstbias.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__dstbias.val", "-", *p1, *p2);
}

static void run_convtime___timezone(void)
{
    long *p1 = __timezone();
    long *p2 = ref___timezone();
    grade2("__timezone.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__timezone.val", "-", *p1, *p2);
}

/* __tzname()[0]/[1] are "PST"/"PDT" by default (both sides start from the
 * same compiled-in default and neither side ever calls tzset() here). */
static void run_convtime___tzname(void)
{
    char **p1 = __tzname();
    char **p2 = ref___tzname();
    grade2("__tzname.isnull", "-", p1 == 0, p2 == 0);
    if (!p1 || !p2)
        return;
    grade2("__tzname.0", "-", fnv(p1[0], strlen(p1[0]) + 1), fnv(p2[0], strlen(p2[0]) + 1));
    grade2("__tzname.1", "-", fnv(p1[1], strlen(p1[1]) + 1), fnv(p2[1], strlen(p2[1]) + 1));
}
#endif /* UCRTDIFF_REF_HAS__get_daylight */

/* ==== argv-data (startup/argv_data.cpp) ================================== */
#if UCRTDIFF_REF_HAS___p___argc
int *__cdecl ref___p___argc(void);
char ***__cdecl ref___p___argv(void);
wchar_t ***__cdecl ref___p___wargv(void);
char **__cdecl ref___p__pgmptr(void);
wchar_t **__cdecl ref___p__wpgmptr(void);
char **__cdecl ref___p__acmdln(void);
wchar_t **__cdecl ref___p__wcmdln(void);
errno_t __cdecl ref__get_pgmptr(char **);
errno_t __cdecl ref__get_wpgmptr(wchar_t **);

/* This harness is bare-entry, NOT mainCRTStartup (see ucrtdiff_entry's own
 * comment): __argc/__argv/__wargv/_pgmptr/_wpgmptr are set by an EXE's own
 * CRT startup (exe_common.inl's _configure_narrow_argv & co via
 * startup/argv_parsing.cpp), which never runs here, on EITHER side -- so both
 * are legitimately still at their zero-initialised state, and that agreement
 * (not a hardcoded expected value) is what these rows grade. */
static void run_convtime___p___argc(void)
{
    int *p1 = __p___argc();
    int *p2 = ref___p___argc();
    grade2("__p___argc.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__p___argc.val", "-", *p1, *p2);
}

static void run_convtime___p___argv(void)
{
    char ***p1 = __p___argv();
    char ***p2 = ref___p___argv();
    grade2("__p___argv.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__p___argv.val.isnull", "-", *p1 == 0, *p2 == 0);
}

static void run_convtime___p___wargv(void)
{
    wchar_t ***p1 = __p___wargv();
    wchar_t ***p2 = ref___p___wargv();
    grade2("__p___wargv.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__p___wargv.val.isnull", "-", *p1 == 0, *p2 == 0);
}

/* __acrt_initialize_command_line (still C, unrenamed name, called from BOTH
 * sides' own __acrt_initialize() before ucrtdiff_entry runs) sets
 * _acmdln/_wcmdln from GetCommandLineA/W -- and since dll and ref live in the
 * SAME process, that is the SAME string on both sides: content equality is a
 * meaningful, exact expectation here, not just cross-side agreement. */
static void run_convtime___p__acmdln(void)
{
    char **p1 = __p__acmdln();
    char **p2 = ref___p__acmdln();
    grade2("__p__acmdln.isnull", "-", (p1 && *p1) == 0, (p2 && *p2) == 0);
    if (p1 && *p1 && p2 && *p2)
        grade2("__p__acmdln.val", "-", fnv(*p1, strlen(*p1) + 1), fnv(*p2, strlen(*p2) + 1));
}

static void run_convtime___p__wcmdln(void)
{
    wchar_t **p1 = __p__wcmdln();
    wchar_t **p2 = ref___p__wcmdln();
    grade2("__p__wcmdln.isnull", "-", (p1 && *p1) == 0, (p2 && *p2) == 0);
    if (p1 && *p1 && p2 && *p2)
        grade2("__p__wcmdln.val", "-", fnv(*p1, (wcslen(*p1) + 1) * sizeof(wchar_t)),
               fnv(*p2, (wcslen(*p2) + 1) * sizeof(wchar_t)));
}

static void run_convtime___p__pgmptr(void)
{
    char **p1 = __p__pgmptr();
    char **p2 = ref___p__pgmptr();
    grade2("__p__pgmptr.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__p__pgmptr.val.isnull", "-", *p1 == 0, *p2 == 0);
}

static void run_convtime___p__wpgmptr(void)
{
    wchar_t **p1 = __p__wpgmptr();
    wchar_t **p2 = ref___p__wpgmptr();
    grade2("__p__wpgmptr.isnull", "-", p1 == 0, p2 == 0);
    if (p1 && p2)
        grade2("__p__wpgmptr.val.isnull", "-", *p1 == 0, *p2 == 0);
}

/* _pgmptr/_wpgmptr are never configured here (see the comment above), so both
 * sides must validate-fail with EINVAL -- exercised through the handler path,
 * not the happy path (there is no happy path reachable in this harness). */
static void run_convtime__get_pgmptr(void)
{
    install_counting_iph();
    char *r1 = (char *)1, *r2 = (char *)1;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    errno_t c1 = _get_pgmptr(&r1);
    errno_t c2 = ref__get_pgmptr(&r2);
    grade2("_get_pgmptr", "-", c1, c2);
    grade2("_get_pgmptr.errno", "-", *_errno(), *ref__errno());
    grade2("_get_pgmptr.iph", "-", g_iph_dll, g_iph_ref);

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _get_pgmptr((char **)0);
    c2 = ref__get_pgmptr((char **)0);
    grade2("_get_pgmptr.nullresult", "-", c1, c2);
    grade2("_get_pgmptr.nullresult.errno", "-", *_errno(), *ref__errno());
    grade2("_get_pgmptr.nullresult.iph", "-", g_iph_dll, g_iph_ref);
}

static void run_convtime__get_wpgmptr(void)
{
    install_counting_iph();
    wchar_t *r1 = (wchar_t *)1, *r2 = (wchar_t *)1;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    errno_t c1 = _get_wpgmptr(&r1);
    errno_t c2 = ref__get_wpgmptr(&r2);
    grade2("_get_wpgmptr", "-", c1, c2);
    grade2("_get_wpgmptr.errno", "-", *_errno(), *ref__errno());
    grade2("_get_wpgmptr.iph", "-", g_iph_dll, g_iph_ref);

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _get_wpgmptr((wchar_t **)0);
    c2 = ref__get_wpgmptr((wchar_t **)0);
    grade2("_get_wpgmptr.nullresult", "-", c1, c2);
    grade2("_get_wpgmptr.nullresult.errno", "-", *_errno(), *ref__errno());
    grade2("_get_wpgmptr.nullresult.iph", "-", g_iph_dll, g_iph_ref);
}
#endif /* UCRTDIFF_REF_HAS___p___argc */

/*
 * ==== registry ============================================================
 * Paste into run_legacy()'s `legacy[]` table, right after the ST_HAND rows
 * (or anywhere else in that table -- order does not matter, `export_begin`
 * shards by name). One #define block per guard, exactly like FP_HAND/ST_HAND
 * above; each expands to `{ 0 }` rows when its guard is 0, so the table
 * always compiles, guard on or off.
 */
#if UCRTDIFF_REF_HAS_atof
#define CONVTIME_ATOF_HAND(name) { #name, run_convtime_##name },
#else
#define CONVTIME_ATOF_HAND(name) { #name, 0 },
#endif
#if UCRTDIFF_REF_HAS__ctime32
#define CONVTIME_CTIME_HAND(name) { #name, run_convtime_##name },
#else
#define CONVTIME_CTIME_HAND(name) { #name, 0 },
#endif
#if UCRTDIFF_REF_HAS__get_daylight
#define CONVTIME_TIMESET_HAND(name) { #name, run_convtime_##name },
#else
#define CONVTIME_TIMESET_HAND(name) { #name, 0 },
#endif
#if UCRTDIFF_REF_HAS___p___argc
#define CONVTIME_ARGV_HAND(name) { #name, run_convtime_##name },
#else
#define CONVTIME_ARGV_HAND(name) { #name, 0 },
#endif

#define CONVTIME_HAND_ENTRIES                                                  \
    CONVTIME_ATOF_HAND(atof) CONVTIME_ATOF_HAND(_atof_l)                       \
    CONVTIME_ATOF_HAND(_wtof) CONVTIME_ATOF_HAND(_wtof_l)                      \
    CONVTIME_ATOF_HAND(_atoflt) CONVTIME_ATOF_HAND(_atoflt_l)                  \
    CONVTIME_ATOF_HAND(_atodbl) CONVTIME_ATOF_HAND(_atodbl_l)                  \
    CONVTIME_CTIME_HAND(_ctime32_s) CONVTIME_CTIME_HAND(_wctime32_s)           \
    CONVTIME_CTIME_HAND(_ctime64_s) CONVTIME_CTIME_HAND(_wctime64_s)           \
    CONVTIME_CTIME_HAND(_ctime32) CONVTIME_CTIME_HAND(_wctime32)               \
    CONVTIME_CTIME_HAND(_ctime64) CONVTIME_CTIME_HAND(_wctime64)               \
    CONVTIME_TIMESET_HAND(_get_daylight) CONVTIME_TIMESET_HAND(_get_dstbias)   \
    CONVTIME_TIMESET_HAND(_get_timezone) CONVTIME_TIMESET_HAND(_get_tzname)    \
    CONVTIME_TIMESET_HAND(__daylight) CONVTIME_TIMESET_HAND(__dstbias)         \
    CONVTIME_TIMESET_HAND(__timezone) CONVTIME_TIMESET_HAND(__tzname)         \
    CONVTIME_ARGV_HAND(__p___argc) CONVTIME_ARGV_HAND(__p___argv)              \
    CONVTIME_ARGV_HAND(__p___wargv) CONVTIME_ARGV_HAND(__p__acmdln)            \
    CONVTIME_ARGV_HAND(__p__wcmdln) CONVTIME_ARGV_HAND(__p__pgmptr)            \
    CONVTIME_ARGV_HAND(__p__wpgmptr) CONVTIME_ARGV_HAND(_get_pgmptr)           \
    CONVTIME_ARGV_HAND(_get_wpgmptr)


/* ==== staging/heapio.c, inlined ==== */
/*
 * heapio.c -- guest grader for the "heapio" batch: align.cpp, invalid_parameter.cpp,
 * open.cpp (all eight exports: _sopen_dispatch, _wsopen_dispatch, _sopen_s, _wsopen_s,
 * plus the four variadic entry points _open/_wopen/_sopen/_wsopen, which now route
 * through the varargs-rust shim's `_v` twins rather than being unported; their shared
 * `_wsopen_nolock`/`_sopen_nolock` internals are exercised indirectly through all of
 * them, never called directly since they are not exported).
 *
 * #included into a copy of ucrtdiff.c, just before `run_legacy()`, so every symbol
 * ucrtdiff.c itself defines above that point (grade2, fnv, label_idx, _errno,
 * ref__errno, __doserrno, install_counting_iph, g_iph_dll/g_iph_ref, UD_IMP, MAX_PATH
 * via <windows.h>) is already in scope. Nothing here reuses the STAT block's fixture
 * helpers (`st_setup`/`g_st_dir`) -- those live inside `#if UCRTDIFF_REF_HAS__stat64`,
 * which this file must survive stat.cpp's deletion independent of.
 *
 * Guard names used (one per C file, per the brief): `UCRTDIFF_REF_HAS__aligned_malloc`
 * (align.cpp), `UCRTDIFF_REF_HAS__invalid_parameter` (invalid_parameter.cpp),
 * `UCRTDIFF_REF_HAS__sopen_dispatch` (open.cpp). None of the three appear yet in
 * generated-refs.h (that generator does not know about this batch), so each is
 * defined here with `#ifndef` -- confirmed present in the reference archive via
 * `llvm-nm --defined-only build-arm64/libucrt_ref.a | grep ' ref_<name>$'` (all three,
 * and every other ref_ symbol this file declares, returned exactly one match). If the
 * generator later grows entries for these names, its `#define` (processed first, since
 * generated-refs.h is `#include`d at the top of ucrtdiff.c) wins over the `#ifndef`
 * fallback here.
 */


/* ==========================================================================
 * ALIGN -- _aligned_malloc, _aligned_free, _aligned_msize, _aligned_offset_malloc,
 * _aligned_offset_realloc, _aligned_offset_recalloc, _aligned_realloc,
 * _aligned_recalloc.
 * ========================================================================== */
#if UCRTDIFF_REF_HAS__aligned_malloc

void *__cdecl ref__aligned_malloc(size_t, size_t);
void __cdecl ref__aligned_free(void *);
size_t __cdecl ref__aligned_msize(void *, size_t, size_t);
void *__cdecl ref__aligned_offset_malloc(size_t, size_t, size_t);
void *__cdecl ref__aligned_offset_realloc(void *, size_t, size_t, size_t);
void *__cdecl ref__aligned_offset_recalloc(void *, size_t, size_t, size_t, size_t);
void *__cdecl ref__aligned_realloc(void *, size_t, size_t);
void *__cdecl ref__aligned_recalloc(void *, size_t, size_t, size_t);

extern void *(__cdecl *const ud__aligned_malloc)(size_t, size_t) __asm__(UD_IMP(_aligned_malloc));
extern void(__cdecl *const ud__aligned_free)(void *) __asm__(UD_IMP(_aligned_free));
extern size_t(__cdecl *const ud__aligned_msize)(void *, size_t, size_t) __asm__(UD_IMP(_aligned_msize));
extern void *(__cdecl *const ud__aligned_offset_malloc)(size_t, size_t, size_t)
    __asm__(UD_IMP(_aligned_offset_malloc));
extern void *(__cdecl *const ud__aligned_offset_realloc)(void *, size_t, size_t, size_t)
    __asm__(UD_IMP(_aligned_offset_realloc));
extern void *(__cdecl *const ud__aligned_offset_recalloc)(void *, size_t, size_t, size_t, size_t)
    __asm__(UD_IMP(_aligned_offset_recalloc));
extern void *(__cdecl *const ud__aligned_realloc)(void *, size_t, size_t)
    __asm__(UD_IMP(_aligned_realloc));
extern void *(__cdecl *const ud__aligned_recalloc)(void *, size_t, size_t, size_t)
    __asm__(UD_IMP(_aligned_recalloc));

/* Grades a freshly-allocated pair: ALIGNMENT (not the raw address), CONTENTS
 * (after stamping a pattern through both), and `_aligned_msize`'s report --
 * never the pointer values themselves, which can never match across two
 * independent heaps. */
static void align_grade_block(const char *family, const char *lbl, void *p1, void *p2,
                               size_t size, size_t alignment, size_t offset)
{
    grade2(family, lbl, (p1 != 0), (p2 != 0));
    if (!p1 || !p2)
        return;
    grade2(family, lbl, (long long)((uintptr_t)p1 % alignment), (long long)((uintptr_t)p2 % alignment));
    if (size) {
        memset(p1, 0x5a, size);
        memset(p2, 0x5a, size);
    }
    grade2(family, lbl, fnv(p1, size), fnv(p2, size));
    grade2(family, lbl, (long long)ud__aligned_msize(p1, alignment, offset),
           (long long)ref__aligned_msize(p2, alignment, offset));
}

static void run_align__aligned_malloc(void)
{
    install_counting_iph();
    static const size_t sizes[] = { 0, 1, 7, 64, 4096 };
    static const size_t aligns[] = { 8, 16, 32, 64, 4096 };
    size_t i, j;
    for (i = 0; i < sizeof(sizes) / sizeof(sizes[0]); i++) {
        for (j = 0; j < sizeof(aligns) / sizeof(aligns[0]); j++) {
            void *p1 = ud__aligned_malloc(sizes[i], aligns[j]);
            void *p2 = ref__aligned_malloc(sizes[i], aligns[j]);
            align_grade_block("_aligned_malloc", label_idx(i * 8 + j), p1, p2, sizes[i], aligns[j], 0);
            ud__aligned_free(p1);
            ref__aligned_free(p2);
        }
    }

    /* Invalid (non-power-of-two, and zero) alignment: EINVAL, NULL, iph fires once
     * per side. */
    {
        static const size_t bad[] = { 0, 3, 5, 100 };
        for (i = 0; i < sizeof(bad) / sizeof(bad[0]); i++) {
            *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
            void *p1 = ud__aligned_malloc(16, bad[i]);
            void *p2 = ref__aligned_malloc(16, bad[i]);
            grade2("_aligned_malloc.badalign", label_idx(i), (p1 == 0), (p2 == 0));
            grade2("_aligned_malloc.badalign.errno", label_idx(i), *_errno(), *ref__errno());
            grade2("_aligned_malloc.badalign.iph", label_idx(i), g_iph_dll, g_iph_ref);
            ud__aligned_free(p1);
            ref__aligned_free(p2);
        }
    }
}

static void run_align__aligned_free(void)
{
    install_counting_iph();
    /* free(NULL) must not fault/hang -- the fault-isolation guard covers the
     * class; here we just confirm both sides return (implicitly, by reaching
     * this grade2 at all) and touch no shared accounting on their own. */
    ud__aligned_free(0);
    ref__aligned_free(0);
    grade2("_aligned_free.null", "ok", 1, 1);

    void *p1 = ud__aligned_malloc(32, 16);
    void *p2 = ref__aligned_malloc(32, 16);
    ud__aligned_free(p1);
    ref__aligned_free(p2);
    grade2("_aligned_free.roundtrip", "ok", 1, 1);
}

static void run_align__aligned_msize(void)
{
    install_counting_iph();
    /* NULL -> EINVAL, (size_t)-1. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    grade2("_aligned_msize.null", "ok", (long long)ud__aligned_msize(0, 16, 0),
           (long long)ref__aligned_msize(0, 16, 0));
    grade2("_aligned_msize.null.errno", "ok", *_errno(), *ref__errno());
    grade2("_aligned_msize.null.iph", "ok", g_iph_dll, g_iph_ref);

    static const size_t sizes[] = { 1, 100, 4096 };
    static const size_t aligns[] = { 8, 64 };
    size_t i, j;
    for (i = 0; i < sizeof(sizes) / sizeof(sizes[0]); i++) {
        for (j = 0; j < sizeof(aligns) / sizeof(aligns[0]); j++) {
            void *p1 = ud__aligned_malloc(sizes[i], aligns[j]);
            void *p2 = ref__aligned_malloc(sizes[i], aligns[j]);
            grade2("_aligned_msize", label_idx(i * 2 + j), (long long)ud__aligned_msize(p1, aligns[j], 0),
                   (long long)ref__aligned_msize(p2, aligns[j], 0));
            ud__aligned_free(p1);
            ref__aligned_free(p2);
        }
    }
}

static void run_align__aligned_offset_malloc(void)
{
    install_counting_iph();
    static const size_t size = 200;
    static const size_t aligns[] = { 8, 32, 64 };
    size_t i;
    for (i = 0; i < sizeof(aligns) / sizeof(aligns[0]); i++) {
        size_t offset = aligns[i] > 4 ? aligns[i] / 4 : 1;
        void *p1 = ud__aligned_offset_malloc(size, aligns[i], offset);
        void *p2 = ref__aligned_offset_malloc(size, aligns[i], offset);
        align_grade_block("_aligned_offset_malloc", label_idx(i), p1, p2, size, aligns[i], offset);
        ud__aligned_free(p1);
        ref__aligned_free(p2);
    }

    /* offset == size (only offset < size or offset == 0 is legal) -> EINVAL. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    void *p1 = ud__aligned_offset_malloc(64, 16, 64);
    void *p2 = ref__aligned_offset_malloc(64, 16, 64);
    grade2("_aligned_offset_malloc.offset_eq_size", "ok", (p1 == 0), (p2 == 0));
    grade2("_aligned_offset_malloc.offset_eq_size.errno", "ok", *_errno(), *ref__errno());
    grade2("_aligned_offset_malloc.offset_eq_size.iph", "ok", g_iph_dll, g_iph_ref);
    ud__aligned_free(p1);
    ref__aligned_free(p2);

    /* offset > size -> EINVAL too. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    p1 = ud__aligned_offset_malloc(64, 16, 65);
    p2 = ref__aligned_offset_malloc(64, 16, 65);
    grade2("_aligned_offset_malloc.offset_gt_size", "ok", (p1 == 0), (p2 == 0));
    grade2("_aligned_offset_malloc.offset_gt_size.errno", "ok", *_errno(), *ref__errno());
    ud__aligned_free(p1);
    ref__aligned_free(p2);
}

static void run_align__aligned_realloc(void)
{
    install_counting_iph();
    static const size_t alignment = 32;
    static const size_t grow_from = 40, grow_to = 4096;
    static const size_t shrink_from = 4096, shrink_to = 16;

    /* Growth: contents up to the old size must survive. */
    {
        void *p1 = ud__aligned_malloc(grow_from, alignment);
        void *p2 = ref__aligned_malloc(grow_from, alignment);
        memset(p1, 0xc3, grow_from);
        memset(p2, 0xc3, grow_from);
        void *r1 = ud__aligned_realloc(p1, grow_to, alignment);
        void *r2 = ref__aligned_realloc(p2, grow_to, alignment);
        grade2("_aligned_realloc.grow.align", "ok", (long long)((uintptr_t)r1 % alignment),
               (long long)((uintptr_t)r2 % alignment));
        grade2("_aligned_realloc.grow.prefix", "ok", fnv(r1, grow_from), fnv(r2, grow_from));
        ud__aligned_free(r1);
        ref__aligned_free(r2);
    }

    /* Shrink: the surviving prefix is the NEW (smaller) size. */
    {
        void *p1 = ud__aligned_malloc(shrink_from, alignment);
        void *p2 = ref__aligned_malloc(shrink_from, alignment);
        memset(p1, 0x7e, shrink_from);
        memset(p2, 0x7e, shrink_from);
        void *r1 = ud__aligned_realloc(p1, shrink_to, alignment);
        void *r2 = ref__aligned_realloc(p2, shrink_to, alignment);
        grade2("_aligned_realloc.shrink.align", "ok", (long long)((uintptr_t)r1 % alignment),
               (long long)((uintptr_t)r2 % alignment));
        grade2("_aligned_realloc.shrink.prefix", "ok", fnv(r1, shrink_to), fnv(r2, shrink_to));
        ud__aligned_free(r1);
        ref__aligned_free(r2);
    }

    /* NULL block -> forwards to malloc. */
    {
        void *r1 = ud__aligned_realloc(0, 64, alignment);
        void *r2 = ref__aligned_realloc(0, 64, alignment);
        grade2("_aligned_realloc.from_null", "ok", (r1 != 0), (r2 != 0));
        ud__aligned_free(r1);
        ref__aligned_free(r2);
    }

    /* size == 0 -> frees and returns NULL. */
    {
        void *p1 = ud__aligned_malloc(64, alignment);
        void *p2 = ref__aligned_malloc(64, alignment);
        void *r1 = ud__aligned_realloc(p1, 0, alignment);
        void *r2 = ref__aligned_realloc(p2, 0, alignment);
        grade2("_aligned_realloc.to_zero", "ok", (r1 == 0), (r2 == 0));
    }
}

static void run_align__aligned_offset_realloc(void)
{
    install_counting_iph();
    static const size_t alignment = 16, offset = 4;
    void *p1 = ud__aligned_offset_malloc(100, alignment, offset);
    void *p2 = ref__aligned_offset_malloc(100, alignment, offset);
    memset(p1, 0x11, 100);
    memset(p2, 0x11, 100);
    void *r1 = ud__aligned_offset_realloc(p1, 500, alignment, offset);
    void *r2 = ref__aligned_offset_realloc(p2, 500, alignment, offset);
    grade2("_aligned_offset_realloc.align", "ok", (long long)(((uintptr_t)r1 + offset) % alignment),
           (long long)(((uintptr_t)r2 + offset) % alignment));
    grade2("_aligned_offset_realloc.prefix", "ok", fnv(r1, 100), fnv(r2, 100));
    ud__aligned_free(r1);
    ref__aligned_free(r2);
}

static void run_align__aligned_recalloc(void)
{
    install_counting_iph();
    static const size_t alignment = 32;
    void *p1 = ud__aligned_recalloc(0, 10, 40, alignment);
    void *p2 = ref__aligned_recalloc(0, 10, 40, alignment);
    grade2("_aligned_recalloc.fresh", "ok", (p1 != 0), (p2 != 0));
    if (p1 && p2) {
        size_t i, nz1 = 0, nz2 = 0;
        for (i = 0; i < 400; i++) {
            if (((unsigned char *)p1)[i]) nz1++;
            if (((unsigned char *)p2)[i]) nz2++;
        }
        grade2("_aligned_recalloc.zeroed", "ok", (long long)nz1, (long long)nz2);
    }

    /* Grow via recalloc: the ORIGINAL bytes survive, only the tail is zeroed. */
    memset(p1, 0x9a, 400);
    memset(p2, 0x9a, 400);
    void *r1 = ud__aligned_recalloc(p1, 20, 40, alignment);
    void *r2 = ref__aligned_recalloc(p2, 20, 40, alignment);
    grade2("_aligned_recalloc.grow.prefix", "ok", fnv(r1, 400), fnv(r2, 400));
    {
        size_t i, nz1 = 0, nz2 = 0;
        for (i = 400; i < 800; i++) {
            if (((unsigned char *)r1)[i]) nz1++;
            if (((unsigned char *)r2)[i]) nz2++;
        }
        grade2("_aligned_recalloc.grow.tail_zeroed", "ok", (long long)nz1, (long long)nz2);
    }
    ud__aligned_free(r1);
    ref__aligned_free(r2);

    /* count * size overflow -> ENOMEM, NULL. */
    *_errno() = 0; *ref__errno() = 0;
    void *o1 = ud__aligned_recalloc(0, (size_t)-1, 2, alignment);
    void *o2 = ref__aligned_recalloc(0, (size_t)-1, 2, alignment);
    grade2("_aligned_recalloc.overflow", "ok", (o1 == 0), (o2 == 0));
    grade2("_aligned_recalloc.overflow.errno", "ok", *_errno(), *ref__errno());
}

static void run_align__aligned_offset_recalloc(void)
{
    install_counting_iph();
    static const size_t alignment = 16, offset = 8;
    void *p1 = ud__aligned_offset_recalloc(0, 5, 32, alignment, offset);
    void *p2 = ref__aligned_offset_recalloc(0, 5, 32, alignment, offset);
    grade2("_aligned_offset_recalloc.fresh", "ok", (p1 != 0), (p2 != 0));
    if (p1 && p2) {
        size_t i, nz1 = 0, nz2 = 0;
        for (i = 0; i < 160; i++) {
            if (((unsigned char *)p1)[i]) nz1++;
            if (((unsigned char *)p2)[i]) nz2++;
        }
        grade2("_aligned_offset_recalloc.zeroed", "ok", (long long)nz1, (long long)nz2);
    }
    ud__aligned_free(p1);
    ref__aligned_free(p2);
}

#define HEAPIO_ALIGN_HAND(name) { "_aligned_" #name, run_align__aligned_##name },
#else
#define HEAPIO_ALIGN_HAND(name) { "_aligned_" #name, 0 },
#endif /* UCRTDIFF_REF_HAS__aligned_malloc */

#define HEAPIO_ALIGN_HAND_ENTRIES \
    HEAPIO_ALIGN_HAND(malloc) HEAPIO_ALIGN_HAND(free) HEAPIO_ALIGN_HAND(msize) \
    HEAPIO_ALIGN_HAND(offset_malloc) HEAPIO_ALIGN_HAND(realloc) \
    HEAPIO_ALIGN_HAND(offset_realloc) HEAPIO_ALIGN_HAND(recalloc) \
    HEAPIO_ALIGN_HAND(offset_recalloc)

/* ==========================================================================
 * INVALID PARAMETER -- _invalid_parameter, _invalid_parameter_noinfo,
 * _set_invalid_parameter_handler, _get_invalid_parameter_handler,
 * _set_thread_local_invalid_parameter_handler,
 * _get_thread_local_invalid_parameter_handler.
 *
 * NEVER exercised: _invalid_parameter_noinfo_noreturn, _invoke_watson (both
 * terminate the process when no handler intercepts them -- and the whole point
 * of this block is to always have ONE installed before anything can reach
 * Watson).
 * ========================================================================== */
#if UCRTDIFF_REF_HAS__invalid_parameter

typedef void(__cdecl *heapio_iph_t)(const wchar_t *, const wchar_t *, const wchar_t *, unsigned, uintptr_t);

void __cdecl ref__invalid_parameter(const wchar_t *, const wchar_t *, const wchar_t *, unsigned, uintptr_t);
void __cdecl ref__invalid_parameter_noinfo(void);
heapio_iph_t __cdecl ref__set_invalid_parameter_handler(heapio_iph_t);
heapio_iph_t __cdecl ref__get_invalid_parameter_handler(void);
heapio_iph_t __cdecl ref__set_thread_local_invalid_parameter_handler(heapio_iph_t);
heapio_iph_t __cdecl ref__get_thread_local_invalid_parameter_handler(void);

extern void(__cdecl *const ud__invalid_parameter)(const wchar_t *, const wchar_t *, const wchar_t *,
                                                   unsigned, uintptr_t) __asm__(UD_IMP(_invalid_parameter));
extern void(__cdecl *const ud__invalid_parameter_noinfo)(void) __asm__(UD_IMP(_invalid_parameter_noinfo));
extern heapio_iph_t(__cdecl *const ud__set_invalid_parameter_handler)(heapio_iph_t)
    __asm__(UD_IMP(_set_invalid_parameter_handler));
extern heapio_iph_t(__cdecl *const ud__get_invalid_parameter_handler)(void)
    __asm__(UD_IMP(_get_invalid_parameter_handler));
extern heapio_iph_t(__cdecl *const ud__set_thread_local_invalid_parameter_handler)(heapio_iph_t)
    __asm__(UD_IMP(_set_thread_local_invalid_parameter_handler));
extern heapio_iph_t(__cdecl *const ud__get_thread_local_invalid_parameter_handler)(void)
    __asm__(UD_IMP(_get_thread_local_invalid_parameter_handler));

/* Two DISTINCT counting handlers per side (four total), so a precedence test can
 * tell "the ref handler ran" from "the global handler ran" instead of both
 * incrementing the same counter. */
static volatile long g_heapio_dll_ref, g_heapio_dll_tl, g_heapio_ref_ref, g_heapio_ref_tl;
static void __cdecl heapio_dll_ref_iph(const wchar_t *e, const wchar_t *f, const wchar_t *fl,
                                        unsigned l, uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_heapio_dll_ref++; }
static void __cdecl heapio_dll_tl_iph(const wchar_t *e, const wchar_t *f, const wchar_t *fl,
                                       unsigned l, uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_heapio_dll_tl++; }
static void __cdecl heapio_ref_ref_iph(const wchar_t *e, const wchar_t *f, const wchar_t *fl,
                                        unsigned l, uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_heapio_ref_ref++; }
static void __cdecl heapio_ref_tl_iph(const wchar_t *e, const wchar_t *f, const wchar_t *fl,
                                       unsigned l, uintptr_t r)
{ (void)e; (void)f; (void)fl; (void)l; (void)r; g_heapio_ref_tl++; }

/* Restores BOTH handler slots to what `install_counting_iph()` set up, on
 * BOTH sides, at exit: this block must not leave a handler installed that a
 * LATER export (which shares this process) does not expect. */
static void heapio_iph_restore(void)
{
    ud__set_invalid_parameter_handler((heapio_iph_t)ud__get_invalid_parameter_handler);
    ud__set_thread_local_invalid_parameter_handler(0);
    ref__set_thread_local_invalid_parameter_handler(0);
    install_counting_iph();
}

static void run_iph__set_get_invalid_parameter_handler(void)
{
    install_counting_iph();
    /* Round trip: install, read back, must be the SAME pointer -- this is one
     * of the few places a raw function-pointer comparison IS the contract
     * (the getter's job is literally "return what I was given"), not an
     * address that differs across the two heaps. */
    heapio_iph_t old_dll = ud__set_invalid_parameter_handler(heapio_dll_ref_iph);
    heapio_iph_t old_ref = ref__set_invalid_parameter_handler(heapio_ref_ref_iph);
    grade2("_set_invalid_parameter_handler.roundtrip", "ok",
           (ud__get_invalid_parameter_handler() == heapio_dll_ref_iph),
           (ref__get_invalid_parameter_handler() == heapio_ref_ref_iph));
    /* Old handler returned was the counting iph `install_counting_iph()`
     * installed -- both sides had ONE installed, so both "old" values are
     * non-NULL. */
    grade2("_set_invalid_parameter_handler.old_nonnull", "ok", (old_dll != 0), (old_ref != 0));

    /* NULL resets to "no handler" -- observable only via a call that reaches
     * Watson if unhandled, so this checks the GETTER, never triggers that
     * path. */
    ud__set_invalid_parameter_handler(0);
    ref__set_invalid_parameter_handler(0);
    grade2("_set_invalid_parameter_handler.null_reset", "ok",
           (ud__get_invalid_parameter_handler() == 0), (ref__get_invalid_parameter_handler() == 0));

    heapio_iph_restore();
}

static void run_iph__set_get_thread_local_invalid_parameter_handler(void)
{
    install_counting_iph();
    grade2("_get_thread_local_invalid_parameter_handler.initial", "ok",
           (ud__get_thread_local_invalid_parameter_handler() == 0),
           (ref__get_thread_local_invalid_parameter_handler() == 0));

    heapio_iph_t old_dll = ud__set_thread_local_invalid_parameter_handler(heapio_dll_tl_iph);
    heapio_iph_t old_ref = ref__set_thread_local_invalid_parameter_handler(heapio_ref_tl_iph);
    grade2("_set_thread_local_invalid_parameter_handler.old_was_null", "ok", (old_dll == 0), (old_ref == 0));
    grade2("_get_thread_local_invalid_parameter_handler.roundtrip", "ok",
           (ud__get_thread_local_invalid_parameter_handler() == heapio_dll_tl_iph),
           (ref__get_thread_local_invalid_parameter_handler() == heapio_ref_tl_iph));

    ud__set_thread_local_invalid_parameter_handler(0);
    ref__set_thread_local_invalid_parameter_handler(0);
    grade2("_get_thread_local_invalid_parameter_handler.reset", "ok",
           (ud__get_thread_local_invalid_parameter_handler() == 0),
           (ref__get_thread_local_invalid_parameter_handler() == 0));

    heapio_iph_restore();
}

/* Precedence: thread-local beats global when BOTH are installed, and a call
 * through `_invalid_parameter` (which builds its own fresh
 * `__crt_cached_ptd_host`/`PtdHost`) is what exercises the lookup -- not a
 * direct call to the internal, which is not part of the public surface. */
static void run_iph__precedence(void)
{
    install_counting_iph();
    ud__set_invalid_parameter_handler(heapio_dll_ref_iph);
    ref__set_invalid_parameter_handler(heapio_ref_ref_iph);
    ud__set_thread_local_invalid_parameter_handler(heapio_dll_tl_iph);
    ref__set_thread_local_invalid_parameter_handler(heapio_ref_tl_iph);

    g_heapio_dll_ref = g_heapio_dll_tl = g_heapio_ref_ref = g_heapio_ref_tl = 0;
    ud__invalid_parameter_noinfo();
    ref__invalid_parameter_noinfo();
    /* Thread-local ran, global did NOT, on both sides. */
    grade2("_invalid_parameter.precedence.tl_ran", "ok", g_heapio_dll_tl, g_heapio_ref_tl);
    grade2("_invalid_parameter.precedence.global_silent", "ok", g_heapio_dll_ref, g_heapio_ref_ref);

    /* Remove the thread-local handler: now the GLOBAL one must run. */
    ud__set_thread_local_invalid_parameter_handler(0);
    ref__set_thread_local_invalid_parameter_handler(0);
    g_heapio_dll_ref = g_heapio_dll_tl = g_heapio_ref_ref = g_heapio_ref_tl = 0;
    ud__invalid_parameter(0, 0, 0, 0, 0);
    ref__invalid_parameter(0, 0, 0, 0, 0);
    grade2("_invalid_parameter.precedence.global_ran", "ok", g_heapio_dll_ref, g_heapio_ref_ref);

    heapio_iph_restore();
}

/* The self-check the brief asks for: a real CRT export -- from THIS SAME
 * batch, `_aligned_malloc` with a non-power-of-two alignment -- must
 * increment exactly the installed handler, on both sides, tying the iph
 * machinery to a caller outside this file rather than only to direct calls
 * to the setters/getters above. */
static void run_iph__real_caller(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_HAS__aligned_malloc
    g_iph_dll = g_iph_ref = 0;
    void *p1 = ud__aligned_malloc(16, 5 /* not a power of two */);
    void *p2 = ref__aligned_malloc(16, 5);
    grade2("_invalid_parameter.real_caller.fired", "ok", g_iph_dll, g_iph_ref);
    ud__aligned_free(p1);
    ref__aligned_free(p2);
#else
    grade2("_invalid_parameter.real_caller", "skipped-no-align-ref", 1, 1);
#endif
}

#define HEAPIO_IPH_HAND(name, fn) { name, run_iph__##fn },
#else
#define HEAPIO_IPH_HAND(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS__invalid_parameter */

#define HEAPIO_IPH_HAND_ENTRIES \
    HEAPIO_IPH_HAND("_set_invalid_parameter_handler+_get", set_get_invalid_parameter_handler) \
    HEAPIO_IPH_HAND("_set_get_thread_local_invalid_parameter_handler", set_get_thread_local_invalid_parameter_handler) \
    HEAPIO_IPH_HAND("_invalid_parameter.precedence", precedence) \
    HEAPIO_IPH_HAND("_invalid_parameter.real_caller", real_caller)

/* ==========================================================================
 * OPEN -- the non-variadic five: _sopen_dispatch, _wsopen_dispatch, _sopen_s,
 * _wsopen_s (their shared `_wsopen_nolock`/`_sopen_nolock` internals are
 * exercised through them, never called directly -- they are not exported).
 * ========================================================================== */
#if UCRTDIFF_REF_HAS__sopen_dispatch

int __cdecl ref__sopen_dispatch(const char *, int, int, int, int *, int);
int __cdecl ref__wsopen_dispatch(const wchar_t *, int, int, int, int *, int);
int __cdecl ref__sopen_s(int *, const char *, int, int, int);
int __cdecl ref__wsopen_s(int *, const wchar_t *, int, int, int);
int __cdecl ref__close(int);
int __cdecl ref__write(int, const void *, unsigned);

extern int(__cdecl *const ud__sopen_dispatch)(const char *, int, int, int, int *, int)
    __asm__(UD_IMP(_sopen_dispatch));
extern int(__cdecl *const ud__wsopen_dispatch)(const wchar_t *, int, int, int, int *, int)
    __asm__(UD_IMP(_wsopen_dispatch));
extern int(__cdecl *const ud__sopen_s)(int *, const char *, int, int, int) __asm__(UD_IMP(_sopen_s));
extern int(__cdecl *const ud__wsopen_s)(int *, const wchar_t *, int, int, int) __asm__(UD_IMP(_wsopen_s));

/* Self-contained fixture directory -- deliberately NOT the stat block's
 * `st_setup`/`g_st_dir`, which live inside `#if UCRTDIFF_REF_HAS__stat64` and
 * must not be a dependency of this file surviving stat.cpp's deletion. */
#define SH_DENYNO_LOCAL 0x40   /* _SH_DENYNO */
#define S_IREAD_LOCAL 0x0100   /* _S_IREAD */
#define S_IWRITE_LOCAL 0x0080  /* _S_IWRITE */
/* `_O_*` (fcntl.h), used as-is below: this file is included into a TU that
 * already has them from real headers (WRONLY/CREAT/TRUNC/EXCL/RDONLY/BINARY/
 * WTEXT/U16TEXT/U8TEXT/TEXT), confirmed present via <fcntl.h> already
 * included by ucrtdiff.c's own headers. Only the three internal-looking
 * `_umaskval`-adjacent ones above (SH_DENYNO/S_IREAD/S_IWRITE) are missing
 * from the DLL-facing headers ucrtdiff.c includes, hence the local spellings.
 */

static wchar_t g_heapio_dir[MAX_PATH];
static int heapio_dir_ready;

static int heapio_setup_dir(void)
{
    if (heapio_dir_ready)
        return 1;
    wchar_t base[MAX_PATH];
    DWORD n = GetTempPathW(MAX_PATH - 32, base);
    if (!n || n >= MAX_PATH - 32)
        return 0;
    lstrcpyW(g_heapio_dir, base);
    lstrcatW(g_heapio_dir, L"ucrtdiff_heapio\\");
    if (!CreateDirectoryW(g_heapio_dir, NULL) && GetLastError() != ERROR_ALREADY_EXISTS)
        return 0;
    heapio_dir_ready = 1;
    return 1;
}

static void heapio_path(const wchar_t *leaf, wchar_t *out)
{
    lstrcpyW(out, g_heapio_dir);
    lstrcatW(out, leaf);
}

/* Reads back what was actually written, from OUTSIDE the CRT entirely (raw
 * Win32), so a BOM-writing bug cannot grade itself as passing. */
static long long heapio_read_back(const wchar_t *path)
{
    HANDLE h = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
                            OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
        return -1;
    unsigned char buf[64];
    DWORD got = 0;
    BOOL ok = ReadFile(h, buf, sizeof buf, &got, NULL);
    CloseHandle(h);
    if (!ok)
        return -1;
    return fnv(buf, got);
}

/* `_O_CREAT|_O_TRUNC|_O_WRONLY` with each text mode, both narrow and wide
 * entry points; grades the fd's validity/errno/iph AND the BOM bytes actually
 * on disk (read back independently). */
static void heapio_bom_case(const char *lbl, int oflag, wchar_t *path, const char *narrow_path,
                             int wide)
{
    int fh1 = -1, fh2 = -1;
    int rc1, rc2;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    if (wide) {
        DbgPrint("[UCRTDIFF] bom %s oflag=0x%x: dll ->\n", lbl, oflag);
        rc1 = ud__wsopen_s(&fh1, path, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        DbgPrint("[UCRTDIFF] bom %s: dll rc=%d fh=%d, ref ->\n", lbl, rc1, fh1);
        rc2 = ref__wsopen_s(&fh2, path, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        DbgPrint("[UCRTDIFF] bom %s: ref rc=%d fh=%d\n", lbl, rc2, fh2);
    } else {
        rc1 = ud__sopen_s(&fh1, narrow_path, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        rc2 = ref__sopen_s(&fh2, narrow_path, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
    }
    grade2("_open.bom.rc", lbl, rc1, rc2);
    grade2("_open.bom.fh_valid", lbl, (fh1 >= 0), (fh2 >= 0));
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    long long bytes = heapio_read_back(path);
    grade2("_open.bom.bytes", lbl, bytes, bytes); /* recorded, not cross-graded: each
        run only has ONE file on disk (the DLL and ref sides share the same path,
        opened and closed in turn), so this is a sanity echo, not a real
        cross-check -- see the two-path variant below for the real BOM grade. */
}

/* The real cross-check: open the SAME logical file with the DLL and the ref
 * side at two DIFFERENT paths, then diff what landed on disk byte-for-byte. */
static void heapio_bom_pair(const char *lbl, int oflag)
{
    wchar_t p1[MAX_PATH], p2[MAX_PATH];
    heapio_path(L"dll.bom", p1);
    heapio_path(L"ref.bom", p2);
    DeleteFileW(p1);
    DeleteFileW(p2);

    int fh1 = -1, fh2 = -1;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    /* One serial line per side: the u16/wtext cases have taken the guest KERNEL
     * down (fastfat, an uncaught STATUS_FILE_INVALID), and only these say which
     * side's calls reached it. */
    DbgPrint("[UCRTDIFF] bom %s oflag=0x%x: dll ->\n", lbl, oflag);
    int rc1 = ud__wsopen_s(&fh1, p1, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
    DbgPrint("[UCRTDIFF] bom %s: dll rc=%d fh=%d, ref ->\n", lbl, rc1, fh1);
    int rc2 = ref__wsopen_s(&fh2, p2, oflag, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
    DbgPrint("[UCRTDIFF] bom %s: ref rc=%d fh=%d\n", lbl, rc2, fh2);
    grade2("_wsopen_s.bom.rc", lbl, rc1, rc2);
    grade2("_wsopen_s.bom.fh_valid", lbl, (fh1 >= 0), (fh2 >= 0));
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    long long b1 = heapio_read_back(p1);
    long long b2 = heapio_read_back(p2);
    grade2("_wsopen_s.bom.on_disk", lbl, b1, b2);
}

static void run_open__bom(void)
{
    install_counting_iph();
    if (!heapio_setup_dir()) {
        grade2("_open.bom.fixture", "setup", 0, 1);
        return;
    }

    static const struct { const char *lbl; int oflag; } cases[] = {
        { "u8", _O_CREAT | _O_TRUNC | _O_WRONLY | _O_U8TEXT },
        { "u16", _O_CREAT | _O_TRUNC | _O_WRONLY | _O_U16TEXT },
        { "wtext", _O_CREAT | _O_TRUNC | _O_WRONLY | _O_WTEXT },
        { "text", _O_CREAT | _O_TRUNC | _O_WRONLY | _O_TEXT },
        { "binary", _O_CREAT | _O_TRUNC | _O_WRONLY | _O_BINARY },
    };
    size_t i;
    for (i = 0; i < sizeof(cases) / sizeof(cases[0]); i++)
        heapio_bom_pair(cases[i].lbl, cases[i].oflag);
}

static void run_open__excl_and_bad_flags(void)
{
    install_counting_iph();
    if (!heapio_setup_dir()) {
        grade2("_open.excl.fixture", "setup", 0, 1);
        return;
    }
    wchar_t p[MAX_PATH];
    heapio_path(L"excl.txt", p);
    DeleteFileW(p);

    /* First create it... */
    int fh1 = -1, fh2 = -1;
    wchar_t p2path[MAX_PATH];
    heapio_path(L"excl_ref.txt", p2path);
    DeleteFileW(p2path);
    ud__wsopen_s(&fh1, p, _O_CREAT | _O_WRONLY, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
    ref__wsopen_s(&fh2, p2path, _O_CREAT | _O_WRONLY, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    /* ... then _O_CREAT|_O_EXCL on the now-existing file must FAIL on both. */
    *_errno() = 0; *ref__errno() = 0;
    int rc1 = ud__wsopen_s(&fh1, p, _O_CREAT | _O_EXCL | _O_WRONLY, SH_DENYNO_LOCAL,
                            S_IREAD_LOCAL | S_IWRITE_LOCAL);
    int rc2 = ref__wsopen_s(&fh2, p2path, _O_CREAT | _O_EXCL | _O_WRONLY, SH_DENYNO_LOCAL,
                             S_IREAD_LOCAL | S_IWRITE_LOCAL);
    grade2("_wsopen_s.excl_existing.rc_nonzero", "ok", (rc1 != 0), (rc2 != 0));
    grade2("_wsopen_s.excl_existing.fh", "ok", fh1, fh2);
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    /* Bad shflag (e.g. _SH_COMPAT, 0x00 -- no case in decode_sharing_flags). */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_s(&fh1, p, _O_RDONLY, 0 /* _SH_COMPAT */, 0);
    rc2 = ref__wsopen_s(&fh2, p2path, _O_RDONLY, 0, 0);
    grade2("_wsopen_s.bad_shflag.rc", "ok", rc1, rc2);
    grade2("_wsopen_s.bad_shflag.errno", "ok", *_errno(), *ref__errno());
    grade2("_wsopen_s.bad_shflag.iph", "ok", g_iph_dll, g_iph_ref);
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    /* Bad pmode for the `_s` (secure) forms only: any bit outside
     * _S_IREAD|_S_IWRITE. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_s(&fh1, p, _O_CREAT | _O_WRONLY, SH_DENYNO_LOCAL, 0x1);
    rc2 = ref__wsopen_s(&fh2, p2path, _O_CREAT | _O_WRONLY, SH_DENYNO_LOCAL, 0x1);
    grade2("_wsopen_s.bad_pmode.rc", "ok", rc1, rc2);
    grade2("_wsopen_s.bad_pmode.errno", "ok", *_errno(), *ref__errno());
    grade2("_wsopen_s.bad_pmode.iph", "ok", g_iph_dll, g_iph_ref);
    if (fh1 >= 0) _close(fh1);
    if (fh2 >= 0) ref__close(fh2);

    /* NULL name / NULL pfh for the `_s` forms. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_s(&fh1, 0, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    rc2 = ref__wsopen_s(&fh2, 0, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    grade2("_wsopen_s.null_name.rc", "ok", rc1, rc2);
    grade2("_wsopen_s.null_name.fh", "ok", fh1, fh2);

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_s(0, p, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    rc2 = ref__wsopen_s(0, p2path, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    grade2("_wsopen_s.null_pfh.rc", "ok", rc1, rc2);
}

/* _sopen_s (narrow) and _wsopen_dispatch (wide), which no other runner reaches: a
 * create/write round trip read back from outside, then the NULL-argument paths. */
static void run_open__sopen_s_wsopen_dispatch(void)
{
    install_counting_iph();
    if (!heapio_setup_dir()) {
        grade2("_sopen_s.fixture", "setup", 0, 1);
        return;
    }
    wchar_t wp1[MAX_PATH], wp2[MAX_PATH];
    char np1[MAX_PATH * 2], np2[MAX_PATH * 2];
    int fh1, fh2, rc1, rc2;

    heapio_path(L"sopen_s1.txt", wp1);
    heapio_path(L"sopen_s2.txt", wp2);
    DeleteFileW(wp1);
    DeleteFileW(wp2);
    WideCharToMultiByte(CP_ACP, 0, wp1, -1, np1, sizeof np1, NULL, NULL);
    WideCharToMultiByte(CP_ACP, 0, wp2, -1, np2, sizeof np2, NULL, NULL);
    fh1 = fh2 = -1;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__sopen_s(&fh1, np1, _O_CREAT | _O_WRONLY | _O_TEXT, SH_DENYNO_LOCAL,
                      S_IREAD_LOCAL | S_IWRITE_LOCAL);
    rc2 = ref__sopen_s(&fh2, np2, _O_CREAT | _O_WRONLY | _O_TEXT, SH_DENYNO_LOCAL,
                       S_IREAD_LOCAL | S_IWRITE_LOCAL);
    grade2("_sopen_s.rc", "create", rc1, rc2);
    grade2("_sopen_s.fh_valid", "create", (fh1 >= 0), (fh2 >= 0));
    if (fh1 >= 0) { _write(fh1, "a\nb", 3); _close(fh1); }
    if (fh2 >= 0) { ref__write(fh2, "a\nb", 3); ref__close(fh2); }
    grade2("_sopen_s.on_disk", "create", heapio_read_back(wp1), heapio_read_back(wp2));
    fh1 = fh2 = 7;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__sopen_s(&fh1, 0, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    rc2 = ref__sopen_s(&fh2, 0, _O_RDONLY, SH_DENYNO_LOCAL, 0);
    grade2("_sopen_s.rc", "null_path", rc1, rc2);
    grade2("_sopen_s.fh", "null_path", fh1, fh2);
    grade2("_sopen_s.errno", "null_path", *_errno(), *ref__errno());
    grade2("_sopen_s.iph", "null_path", g_iph_dll, g_iph_ref);

    heapio_path(L"wdispatch1.txt", wp1);
    heapio_path(L"wdispatch2.txt", wp2);
    DeleteFileW(wp1);
    DeleteFileW(wp2);
    fh1 = fh2 = -1;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_dispatch(wp1, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                              S_IREAD_LOCAL | S_IWRITE_LOCAL, &fh1, 1);
    rc2 = ref__wsopen_dispatch(wp2, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                               S_IREAD_LOCAL | S_IWRITE_LOCAL, &fh2, 1);
    grade2("_wsopen_dispatch.rc", "create", rc1, rc2);
    grade2("_wsopen_dispatch.fh_valid", "create", (fh1 >= 0), (fh2 >= 0));
    if (fh1 >= 0) { _write(fh1, "xyz", 3); _close(fh1); }
    if (fh2 >= 0) { ref__write(fh2, "xyz", 3); ref__close(fh2); }
    grade2("_wsopen_dispatch.on_disk", "create", heapio_read_back(wp1), heapio_read_back(wp2));
    fh1 = fh2 = 7;
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_dispatch(0, _O_RDONLY, SH_DENYNO_LOCAL, 0, &fh1, 1);
    rc2 = ref__wsopen_dispatch(0, _O_RDONLY, SH_DENYNO_LOCAL, 0, &fh2, 1);
    grade2("_wsopen_dispatch.rc", "null_path", rc1, rc2);
    grade2("_wsopen_dispatch.fh", "null_path", fh1, fh2);
    grade2("_wsopen_dispatch.errno", "null_path", *_errno(), *ref__errno());
    /* A bad pmode under the secure form is EINVAL through the handler. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__wsopen_dispatch(wp1, _O_CREAT | _O_RDWR, SH_DENYNO_LOCAL, 0x1234, &fh1, 1);
    rc2 = ref__wsopen_dispatch(wp2, _O_CREAT | _O_RDWR, SH_DENYNO_LOCAL, 0x1234, &fh2, 1);
    grade2("_wsopen_dispatch.rc", "bad_pmode", rc1, rc2);
    grade2("_wsopen_dispatch.iph", "bad_pmode", g_iph_dll, g_iph_ref);
    if (rc1 == 0 && fh1 >= 0) _close(fh1);
    if (rc2 == 0 && fh2 >= 0) ref__close(fh2);
}

static void run_open__dispatch(void)
{
    install_counting_iph();
    if (!heapio_setup_dir()) {
        grade2("_sopen_dispatch.fixture", "setup", 0, 1);
        return;
    }
    wchar_t wp1[MAX_PATH], wp2[MAX_PATH];
    heapio_path(L"dispatch1.txt", wp1);
    heapio_path(L"dispatch2.txt", wp2);
    DeleteFileW(wp1);
    DeleteFileW(wp2);
    char np1[MAX_PATH * 2], np2[MAX_PATH * 2];
    WideCharToMultiByte(CP_ACP, 0, wp1, -1, np1, sizeof np1, NULL, NULL);
    WideCharToMultiByte(CP_ACP, 0, wp2, -1, np2, sizeof np2, NULL, NULL);

    int fh1 = -1, fh2 = -1;
    int rc1 = ud__sopen_dispatch(np1, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                                  S_IREAD_LOCAL | S_IWRITE_LOCAL, &fh1, 0 /* not secure: pmode unchecked */);
    int rc2 = ref__sopen_dispatch(np2, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                                   S_IREAD_LOCAL | S_IWRITE_LOCAL, &fh2, 0);
    grade2("_sopen_dispatch.rc", "ok", rc1, rc2);
    grade2("_sopen_dispatch.fh_valid", "ok", (fh1 >= 0), (fh2 >= 0));
    if (fh1 >= 0) { _write(fh1, "hi", 2); _close(fh1); }
    if (fh2 >= 0) { ref__write(fh2, "hi", 2); ref__close(fh2); }
    grade2("_sopen_dispatch.on_disk", "ok", heapio_read_back(wp1), heapio_read_back(wp2));

    /* NULL pfh -> EINVAL, never touches *pfh. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__sopen_dispatch(np1, _O_RDONLY, SH_DENYNO_LOCAL, 0, 0, 0);
    rc2 = ref__sopen_dispatch(np2, _O_RDONLY, SH_DENYNO_LOCAL, 0, 0, 0);
    grade2("_sopen_dispatch.null_pfh.rc", "ok", rc1, rc2);
    grade2("_sopen_dispatch.null_pfh.errno", "ok", *_errno(), *ref__errno());

    /* NULL path -> EINVAL, *pfh set to -1. */
    *_errno() = 0; *ref__errno() = 0;
    rc1 = ud__sopen_dispatch(0, _O_RDONLY, SH_DENYNO_LOCAL, 0, &fh1, 0);
    rc2 = ref__sopen_dispatch(0, _O_RDONLY, SH_DENYNO_LOCAL, 0, &fh2, 0);
    grade2("_sopen_dispatch.null_path.rc", "ok", rc1, rc2);
    grade2("_sopen_dispatch.null_path.fh", "ok", fh1, fh2);

    /* secure=1 with a bad pmode -> EINVAL, before any I/O. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    rc1 = ud__sopen_dispatch(np1, _O_RDONLY, SH_DENYNO_LOCAL, 0x4, &fh1, 1);
    rc2 = ref__sopen_dispatch(np2, _O_RDONLY, SH_DENYNO_LOCAL, 0x4, &fh2, 1);
    grade2("_sopen_dispatch.secure_bad_pmode.rc", "ok", rc1, rc2);
    grade2("_sopen_dispatch.secure_bad_pmode.errno", "ok", *_errno(), *ref__errno());
}

/* The four VARIADIC entry points themselves: `_open`, `_wopen`, `_sopen`,
 * `_wsopen`. Both sides declare them exactly as the real headers do (`...`),
 * so an ordinary call site passes `pmode` the same way on both -- no `ud_`
 * import-pointer trick needed here (unlike `_stat`-family names, `_open` and
 * friends are not remapped to inline twins by mingw's headers; confirmed by
 * grep over `sdk/include/crt/io.h` finding no `__CRT_INLINE`/`#define` for
 * any of the four). The DLL side is called by its plain name (as the STAT
 * block already does for `_wopen`, `ref__wopen`, at ucrtdiff.c's fstat
 * helper) and resolves through the import lib ucrtdiff links against. */
int __cdecl ref__open(const char *, int, ...);
int __cdecl ref__sopen(const char *, int, int, ...);
int __cdecl ref__wsopen(const wchar_t *, int, int, ...);
/* `ref__wopen` is declared once already, inside the STAT block's `#if
 * UCRTDIFF_REF_HAS__stat64` guard -- redeclared here too since this file
 * must not depend on that guard being 1. A repeated `extern` declaration
 * with an identical signature is legal C. */
int __cdecl ref__wopen(const wchar_t *, int, ...);

static void run_open__variadic(void)
{
    install_counting_iph();
    if (!heapio_setup_dir()) {
        grade2("_open.variadic.fixture", "setup", 0, 1);
        return;
    }

    /* `_open`/narrow. */
    {
        wchar_t wp1[MAX_PATH], wp2[MAX_PATH];
        heapio_path(L"variadic_open1.txt", wp1);
        heapio_path(L"variadic_open2.txt", wp2);
        DeleteFileW(wp1);
        DeleteFileW(wp2);
        char np1[MAX_PATH * 2], np2[MAX_PATH * 2];
        WideCharToMultiByte(CP_ACP, 0, wp1, -1, np1, sizeof np1, NULL, NULL);
        WideCharToMultiByte(CP_ACP, 0, wp2, -1, np2, sizeof np2, NULL, NULL);

        int fh1 = _open(np1, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        int fh2 = ref__open(np2, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        grade2("_open.variadic.fh_valid", "ok", (fh1 >= 0), (fh2 >= 0));
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);

        /* pmode's READONLY bit takes effect even without _O_CREAT|_O_EXCL:
         * open with pmode lacking _S_IWRITE, then check the attribute landed
         * on disk (raw Win32, outside either CRT). This is the case that
         * actually exercises "pmode read unconditionally, same as the C". */
        DeleteFileW(wp1); DeleteFileW(wp2);
        fh1 = _open(np1, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL /* no write */);
        fh2 = ref__open(np2, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL);
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);
        DWORD a1 = GetFileAttributesW(wp1), a2 = GetFileAttributesW(wp2);
        grade2("_open.variadic.readonly_attr", "ok",
               (a1 != INVALID_FILE_ATTRIBUTES) && (a1 & FILE_ATTRIBUTE_READONLY) != 0,
               (a2 != INVALID_FILE_ATTRIBUTES) && (a2 & FILE_ATTRIBUTE_READONLY) != 0);
        SetFileAttributesW(wp1, FILE_ATTRIBUTE_NORMAL);
        SetFileAttributesW(wp2, FILE_ATTRIBUTE_NORMAL);
    }

    /* `_wopen`/wide. */
    {
        wchar_t wp1[MAX_PATH], wp2[MAX_PATH];
        heapio_path(L"variadic_wopen1.txt", wp1);
        heapio_path(L"variadic_wopen2.txt", wp2);
        DeleteFileW(wp1);
        DeleteFileW(wp2);
        int fh1 = _wopen(wp1, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        int fh2 = ref__wopen(wp2, _O_CREAT | _O_WRONLY | _O_BINARY, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        grade2("_wopen.variadic.fh_valid", "ok", (fh1 >= 0), (fh2 >= 0));
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);
    }

    /* `_sopen`/narrow, `_wsopen`/wide -- both return the fd directly (or -1),
     * never `_sopen_dispatch`'s own errno_t. */
    {
        wchar_t wp1[MAX_PATH], wp2[MAX_PATH];
        heapio_path(L"variadic_sopen1.txt", wp1);
        heapio_path(L"variadic_sopen2.txt", wp2);
        DeleteFileW(wp1);
        DeleteFileW(wp2);
        char np1[MAX_PATH * 2], np2[MAX_PATH * 2];
        WideCharToMultiByte(CP_ACP, 0, wp1, -1, np1, sizeof np1, NULL, NULL);
        WideCharToMultiByte(CP_ACP, 0, wp2, -1, np2, sizeof np2, NULL, NULL);

        int fh1 = _sopen(np1, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                          S_IREAD_LOCAL | S_IWRITE_LOCAL);
        int fh2 = ref__sopen(np2, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL,
                              S_IREAD_LOCAL | S_IWRITE_LOCAL);
        grade2("_sopen.variadic.fh_valid", "ok", (fh1 >= 0), (fh2 >= 0));
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);

        heapio_path(L"variadic_wsopen1.txt", wp1);
        heapio_path(L"variadic_wsopen2.txt", wp2);
        DeleteFileW(wp1);
        DeleteFileW(wp2);
        fh1 = _wsopen(wp1, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        fh2 = ref__wsopen(wp2, _O_CREAT | _O_WRONLY | _O_BINARY, SH_DENYNO_LOCAL, S_IREAD_LOCAL | S_IWRITE_LOCAL);
        grade2("_wsopen.variadic.fh_valid", "ok", (fh1 >= 0), (fh2 >= 0));
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);

        /* Bad shflag through `_wsopen` (not `_wsopen_s`): the non-`_s` forms
         * never validate pmode (secure=FALSE), but shflag decoding is the
         * SAME `decode_sharing_flags` either way, so `_SH_COMPAT` (0) must
         * still fail here too. */
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        fh1 = _wsopen(wp1, _O_RDONLY, 0 /* _SH_COMPAT */, 0);
        fh2 = ref__wsopen(wp2, _O_RDONLY, 0, 0);
        grade2("_wsopen.variadic.bad_shflag.fh", "ok", fh1, fh2);
        grade2("_wsopen.variadic.bad_shflag.errno", "ok", *_errno(), *ref__errno());
        if (fh1 >= 0) _close(fh1);
        if (fh2 >= 0) ref__close(fh2);
    }
}

#define HEAPIO_OPEN_HAND(name, fn) { name, run_open__##fn },
#else
#define HEAPIO_OPEN_HAND(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS__sopen_dispatch */

#define HEAPIO_OPEN_HAND_ENTRIES \
    HEAPIO_OPEN_HAND("_wsopen_s.bom", bom) \
    HEAPIO_OPEN_HAND("_wsopen_s.excl_and_bad_flags", excl_and_bad_flags) \
    HEAPIO_OPEN_HAND("_sopen_dispatch", dispatch) \
    HEAPIO_OPEN_HAND("_sopen_s+_wsopen_dispatch", sopen_s_wsopen_dispatch) \
    HEAPIO_OPEN_HAND("_open+_wopen+_sopen+_wsopen", variadic)

/* ==========================================================================
 * Registry -- paste HEAPIO_ALIGN_HAND_ENTRIES, HEAPIO_IPH_HAND_ENTRIES and
 * HEAPIO_OPEN_HAND_ENTRIES into `run_legacy()`'s `legacy[]` table, the same
 * way the existing ST_HAND/FP_HAND/LOC_HAND groups already are.
 * ========================================================================== */


/* ==== staging/exec.c, inlined ==== */
/* ucrtdiff staging snippet -- batch `exec` (sdk/lib/rust/ucrt-rust/src/exec.rs).
 *
 * Grades the 16 `-v`/`-ve`/`-vp`/`-vpe` `_exec*`/`_spawn*` exports (narrow and
 * wide) that `src/exec.rs` ports out of `sdk/lib/ucrt/exec/spawnv.cpp` and
 * `spawnvp.cpp`. NOT covered here (see exec.rs's module docs for why): the
 * 16 variadic `-l`/`-le`/`-lp`/`-lpe` exports (spawnl.cpp/spawnlp.cpp, still
 * C), and `cenvarg.cpp`'s `__acrt_capture_{narrow,wide}_argv` (also still C;
 * its other two symbols, the `__acrt_pack_*_command_line_and_environment`
 * pair, are exercised transitively by every spawn row below since
 * `execute_command` calls them on every call that reaches it -- they are not
 * independently gradable through ud_/ref_ import pointers because they are
 * internal helpers, not ucrtbase exports).
 *
 * SAFETY (read before editing): this snippet runs IN a process whose own CRT
 * is the DLL side under test.
 *   - `_exec*`/`_wexec*` (all 8) are graded ONLY on their NULL-argument
 *     validation failures, which return -1 BEFORE any process work. NEVER
 *     call them with valid arguments -- on success they `_exit(0)` (via
 *     `execute_command`'s `_P_OVERLAY` branch) and terminate this process.
 *   - `_spawn*`/`_wspawn*` are graded with `_P_WAIT` (mode 0) ONLY for their
 *     successful-execution rows. NEVER pass `_P_OVERLAY` (mode 2) to a
 *     `_spawn*` function here either -- `execute_command` takes the exact
 *     same `_exit(0)` branch regardless of whether `_P_OVERLAY` arrived via
 *     `_execv` or via an explicit `_spawnv(..., 2, ...)` call. The
 *     "invalid mode" row below uses 999 specifically because it is invalid
 *     (caught by validation before `execute_command` runs at all), not
 *     because a valid-but-untested mode was avoided by omission.
 *   - `_P_NOWAIT`/`_P_NOWAITO`/`_P_DETACH` are not exercised: they leave a
 *     real child process outstanding (detached, or a handle the grader would
 *     need to reap) with no payoff over `_P_WAIT` for parity grading.
 *
 * Fixture: `%SystemRoot%\System32\cmd.exe`, invoked as `cmd.exe /c exit 7`
 * via argv (ReactOS ships cmd.exe there). Every row calls the DLL side and
 * the REF side with IDENTICAL arguments, so grading is a differential
 * DLL-vs-Microsoft comparison, not a check against a hardcoded expectation --
 * robust even if an environmental assumption here (e.g. System32 being on
 * %PATH%) turns out wrong, since both sides would then fail identically too.
 */

#if UCRTDIFF_REF_HAS__spawnv || UCRTDIFF_REF_HAS__spawnvp || \
    UCRTDIFF_REF_HAS__spawnl || UCRTDIFF_REF_HAS__spawnlp

#include <process.h>

/* ---- ref_ declarations (hand-written: no header can name the ref_ side) ---- */

intptr_t __cdecl ref__execv(const char *, const char *const *);
intptr_t __cdecl ref__execve(const char *, const char *const *, const char *const *);
intptr_t __cdecl ref__spawnv(int, const char *, const char *const *);
intptr_t __cdecl ref__spawnve(int, const char *, const char *const *, const char *const *);
intptr_t __cdecl ref__execvp(const char *, const char *const *);
intptr_t __cdecl ref__execvpe(const char *, const char *const *, const char *const *);
intptr_t __cdecl ref__spawnvp(int, const char *, const char *const *);
intptr_t __cdecl ref__spawnvpe(int, const char *, const char *const *, const char *const *);

intptr_t __cdecl ref__wexecv(const wchar_t *, const wchar_t *const *);
intptr_t __cdecl ref__wexecve(const wchar_t *, const wchar_t *const *, const wchar_t *const *);
intptr_t __cdecl ref__wspawnv(int, const wchar_t *, const wchar_t *const *);
intptr_t __cdecl ref__wspawnve(int, const wchar_t *, const wchar_t *const *, const wchar_t *const *);
intptr_t __cdecl ref__wexecvp(const wchar_t *, const wchar_t *const *);
intptr_t __cdecl ref__wexecvpe(const wchar_t *, const wchar_t *const *, const wchar_t *const *);
intptr_t __cdecl ref__wspawnvp(int, const wchar_t *, const wchar_t *const *);
intptr_t __cdecl ref__wspawnvpe(int, const wchar_t *, const wchar_t *const *, const wchar_t *const *);

/* ---- ud_ import pointers (mingw headers inline/alias several CRT names,
 * so the DLL side is never called by plain name -- see ST_ROW's block for
 * the same rule). */

extern intptr_t(__cdecl *const ud__execv)(const char *, const char *const *) __asm__(UD_IMP(_execv));
extern intptr_t(__cdecl *const ud__execve)(const char *, const char *const *, const char *const *) __asm__(UD_IMP(_execve));
extern intptr_t(__cdecl *const ud__spawnv)(int, const char *, const char *const *) __asm__(UD_IMP(_spawnv));
extern intptr_t(__cdecl *const ud__spawnve)(int, const char *, const char *const *, const char *const *) __asm__(UD_IMP(_spawnve));
extern intptr_t(__cdecl *const ud__execvp)(const char *, const char *const *) __asm__(UD_IMP(_execvp));
extern intptr_t(__cdecl *const ud__execvpe)(const char *, const char *const *, const char *const *) __asm__(UD_IMP(_execvpe));
extern intptr_t(__cdecl *const ud__spawnvp)(int, const char *, const char *const *) __asm__(UD_IMP(_spawnvp));
extern intptr_t(__cdecl *const ud__spawnvpe)(int, const char *, const char *const *, const char *const *) __asm__(UD_IMP(_spawnvpe));

extern intptr_t(__cdecl *const ud__wexecv)(const wchar_t *, const wchar_t *const *) __asm__(UD_IMP(_wexecv));
extern intptr_t(__cdecl *const ud__wexecve)(const wchar_t *, const wchar_t *const *, const wchar_t *const *) __asm__(UD_IMP(_wexecve));
extern intptr_t(__cdecl *const ud__wspawnv)(int, const wchar_t *, const wchar_t *const *) __asm__(UD_IMP(_wspawnv));
extern intptr_t(__cdecl *const ud__wspawnve)(int, const wchar_t *, const wchar_t *const *, const wchar_t *const *) __asm__(UD_IMP(_wspawnve));
extern intptr_t(__cdecl *const ud__wexecvp)(const wchar_t *, const wchar_t *const *) __asm__(UD_IMP(_wexecvp));
extern intptr_t(__cdecl *const ud__wexecvpe)(const wchar_t *, const wchar_t *const *, const wchar_t *const *) __asm__(UD_IMP(_wexecvpe));
extern intptr_t(__cdecl *const ud__wspawnvp)(int, const wchar_t *, const wchar_t *const *) __asm__(UD_IMP(_wspawnvp));
extern intptr_t(__cdecl *const ud__wspawnvpe)(int, const wchar_t *, const wchar_t *const *, const wchar_t *const *) __asm__(UD_IMP(_wspawnvpe));

unsigned long *__cdecl ref___doserrno(void);

/* ---- fixture: %SystemRoot%\System32\cmd(.exe), narrow and wide ---------- */

static wchar_t g_spv_dir_w[MAX_PATH];
static wchar_t g_spv_full_w[MAX_PATH];
static wchar_t g_spv_noext_w[MAX_PATH];
static char g_spv_full_n[MAX_PATH * 2];
static char g_spv_noext_n[MAX_PATH * 2];
static int g_spv_ready;

static int spv_setup(void)
{
    if (g_spv_ready)
        return g_spv_ready > 0;
    DWORD n = GetEnvironmentVariableW(L"SystemRoot", g_spv_dir_w, MAX_PATH - 32);
    if (!n || n >= MAX_PATH - 32) {
        g_spv_ready = -1;
        return 0;
    }
    lstrcpyW(g_spv_full_w, g_spv_dir_w);
    lstrcatW(g_spv_full_w, L"\\System32\\cmd.exe");
    lstrcpyW(g_spv_noext_w, g_spv_dir_w);
    lstrcatW(g_spv_noext_w, L"\\System32\\cmd");
    WideCharToMultiByte(CP_ACP, 0, g_spv_full_w, -1, g_spv_full_n, (int)sizeof g_spv_full_n, NULL, NULL);
    WideCharToMultiByte(CP_ACP, 0, g_spv_noext_w, -1, g_spv_noext_n, (int)sizeof g_spv_noext_n, NULL, NULL);
    /* The `e` forms copy the per-drive "=X:=" variables out of the OS environment,
     * and Microsoft's scan for the first one has no stop at the block's end: with
     * none present it reads past the block until something starts with '='
     * (ours stops -- see exec.rs). Windows always has one; ReactOS may not. Give
     * the process one, as Windows would, so both sides take the defined path and
     * the grade does not depend on heap layout. */
    SetEnvironmentVariableW(L"=C:", L"C:\\");
    g_spv_ready = 1;
    return 1;
}

/* argv/envp fixtures. `PATH`-search (the `p` forms) and extension-probing use
 * a bare/no-extension name; explicit-path rows use the full `...\cmd.exe`. */
static const char *const g_spv_argv_n[] = { "cmd.exe", "/c", "exit", "7", 0 };
static const wchar_t *const g_spv_argv_w[] = { L"cmd.exe", L"/c", L"exit", L"7", 0 };
static const char *const g_spv_argv_empty_n[] = { 0 };
static const wchar_t *const g_spv_argv_empty_w[] = { 0 };
static const char *const g_spv_argv_emptystr_n[] = { "", 0 };
static const wchar_t *const g_spv_argv_emptystr_w[] = { L"", 0 };
static const char *const g_spv_env_n[] = { "UCRTDIFF_EXEC_VAR=1", 0 };
static const wchar_t *const g_spv_env_w[] = { L"UCRTDIFF_EXEC_VAR=1", 0 };
static const char g_spv_missing_n[] = "Z:\\ucrtdiff_missing_9821.exe";
static const wchar_t g_spv_missing_w[] = L"Z:\\ucrtdiff_missing_9821.exe";
/* Bare name (PATH search / extension probing via the `p` forms): the file
 * itself is never opened relative to a directory we control, so this also
 * exercises `_access_s`/`_waccess_s` against the current directory first. */
static const char g_spv_bare_n[] = "cmd.exe";
static const wchar_t g_spv_bare_w[] = L"cmd.exe";

/* ---- grading -------------------------------------------------------------
 * Resets errno/doserrno/iph before EACH side's call (not once for the pair),
 * matching ST_ROW: a call must not see state left behind by its sibling. */
/* A spawn can block on its child, and the per-export timeout reports only HANG:
 * one serial line per side says WHICH side never came back. */
#define SPV_ROW(fam, lbl, CALL_DLL, CALL_REF) do {                          \
        *_errno() = 0; *__doserrno() = 0; g_iph_dll = g_iph_ref = 0;        \
        DbgPrint("[UCRTDIFF] spv %s %s: dll ->\n", fam, lbl);               \
        long long r1 = (long long)(intptr_t)(CALL_DLL);                    \
        long long n1 = (long long)*_errno(), d1 = (long long)*__doserrno(); \
        long long i1 = g_iph_dll;                                          \
        *ref__errno() = 0; *ref___doserrno() = 0; g_iph_dll = g_iph_ref = 0;\
        DbgPrint("[UCRTDIFF] spv %s %s: dll=%lld, ref ->\n", fam, lbl, r1); \
        long long r2 = (long long)(intptr_t)(CALL_REF);                    \
        DbgPrint("[UCRTDIFF] spv %s %s: ref=%lld\n", fam, lbl, r2);         \
        long long n2 = (long long)*ref__errno(), d2 = (long long)*ref___doserrno(); \
        long long i2 = g_iph_ref;                                          \
        grade2(fam, lbl, r1, r2);                                          \
        grade2(fam ".errno", lbl, n1, n2);                                 \
        grade2(fam ".doserrno", lbl, d1, d2);                              \
        grade2(fam ".iph", lbl, i1, i2);                                   \
    } while (0)

#define SPV_ROW2(fam, lbl, fn, a, b)       SPV_ROW(fam, lbl, ud_##fn(a, b), ref_##fn(a, b))
#define SPV_ROW3(fam, lbl, fn, a, b, c)    SPV_ROW(fam, lbl, ud_##fn(a, b, c), ref_##fn(a, b, c))
#define SPV_ROW4(fam, lbl, fn, a, b, c, d) SPV_ROW(fam, lbl, ud_##fn(a, b, c, d), ref_##fn(a, b, c, d))

/* Placeholder-argument runs for the `-l`/`-lp` heap-growth path (content is
 * irrelevant -- only count and NUL-termination matter for exercising
 * `common_capture_argv`). Shared by both the spawnl.cpp and spawnlp.cpp
 * sections below, which may be guarded independently. */
#define SPV_X4_N  "x", "x", "x", "x"
#define SPV_X16_N SPV_X4_N, SPV_X4_N, SPV_X4_N, SPV_X4_N
#define SPV_X64_N SPV_X16_N, SPV_X16_N, SPV_X16_N, SPV_X16_N
#define SPV_X4_W  L"x", L"x", L"x", L"x"
#define SPV_X16_W SPV_X4_W, SPV_X4_W, SPV_X4_W, SPV_X4_W
#define SPV_X64_W SPV_X16_W, SPV_X16_W, SPV_X16_W, SPV_X16_W

#endif /* UCRTDIFF_REF_HAS__spawnv || UCRTDIFF_REF_HAS__spawnvp ||
        * UCRTDIFF_REF_HAS__spawnl || UCRTDIFF_REF_HAS__spawnlp */

/* ========================================================================
 * spawnv.cpp: _execv/_execve/_spawnv/_spawnve, narrow and wide.
 * ======================================================================== */
#if UCRTDIFF_REF_HAS__spawnv

/* -- _execv/_execve/_wexecv/_wexecve: validation failures ONLY (see the
 * SAFETY note above -- a success here would `_exit(0)` this process). */

static void run_spv__execv(void)
{
    install_counting_iph();
    SPV_ROW2("_execv", "NULL-name", _execv, (const char *)0, g_spv_argv_n);
    SPV_ROW2("_execv", "empty-name", _execv, "", g_spv_argv_n);
    SPV_ROW2("_execv", "NULL-argv", _execv, g_spv_full_n, (const char *const *)0);
    SPV_ROW2("_execv", "empty-argv", _execv, g_spv_full_n, g_spv_argv_empty_n);
    SPV_ROW2("_execv", "empty-argv0-str", _execv, g_spv_full_n, g_spv_argv_emptystr_n);
}

static void run_spv__execve(void)
{
    install_counting_iph();
    SPV_ROW3("_execve", "NULL-name", _execve, (const char *)0, g_spv_argv_n, g_spv_env_n);
    SPV_ROW3("_execve", "empty-name", _execve, "", g_spv_argv_n, g_spv_env_n);
    SPV_ROW3("_execve", "NULL-argv", _execve, g_spv_full_n, (const char *const *)0, g_spv_env_n);
    SPV_ROW3("_execve", "empty-argv", _execve, g_spv_full_n, g_spv_argv_empty_n, g_spv_env_n);
    SPV_ROW3("_execve", "empty-argv0-str", _execve, g_spv_full_n, g_spv_argv_emptystr_n, g_spv_env_n);
}

static void run_spv__wexecv(void)
{
    install_counting_iph();
    SPV_ROW2("_wexecv", "NULL-name", _wexecv, (const wchar_t *)0, g_spv_argv_w);
    SPV_ROW2("_wexecv", "empty-name", _wexecv, L"", g_spv_argv_w);
    SPV_ROW2("_wexecv", "NULL-argv", _wexecv, g_spv_full_w, (const wchar_t *const *)0);
    SPV_ROW2("_wexecv", "empty-argv", _wexecv, g_spv_full_w, g_spv_argv_empty_w);
    SPV_ROW2("_wexecv", "empty-argv0-str", _wexecv, g_spv_full_w, g_spv_argv_emptystr_w);
}

static void run_spv__wexecve(void)
{
    install_counting_iph();
    SPV_ROW3("_wexecve", "NULL-name", _wexecve, (const wchar_t *)0, g_spv_argv_w, g_spv_env_w);
    SPV_ROW3("_wexecve", "empty-name", _wexecve, L"", g_spv_argv_w, g_spv_env_w);
    SPV_ROW3("_wexecve", "NULL-argv", _wexecve, g_spv_full_w, (const wchar_t *const *)0, g_spv_env_w);
    SPV_ROW3("_wexecve", "empty-argv", _wexecve, g_spv_full_w, g_spv_argv_empty_w, g_spv_env_w);
    SPV_ROW3("_wexecve", "empty-argv0-str", _wexecve, g_spv_full_w, g_spv_argv_emptystr_w, g_spv_env_w);
}

/* -- _spawnv/_spawnve/_wspawnv/_wspawnve: full matrix (mode is always
 * _P_WAIT=0 for anything that reaches `execute_command`; see SAFETY). */

static void run_spv__spawnv(void)
{
    install_counting_iph();
    SPV_ROW3("_spawnv", "NULL-name", _spawnv, 0, (const char *)0, g_spv_argv_n);
    SPV_ROW3("_spawnv", "empty-name", _spawnv, 0, "", g_spv_argv_n);
    SPV_ROW3("_spawnv", "NULL-argv", _spawnv, 0, g_spv_full_n, (const char *const *)0);
    SPV_ROW3("_spawnv", "empty-argv", _spawnv, 0, g_spv_full_n, g_spv_argv_empty_n);
    SPV_ROW3("_spawnv", "empty-argv0-str", _spawnv, 0, g_spv_full_n, g_spv_argv_emptystr_n);
    SPV_ROW3("_spawnv", "invalid-mode", _spawnv, 999, g_spv_full_n, g_spv_argv_n);
    if (spv_setup()) {
        SPV_ROW3("_spawnv", "nonexistent", _spawnv, 0, g_spv_missing_n, g_spv_argv_n);
        SPV_ROW3("_spawnv", "wait-fullpath", _spawnv, 0, g_spv_full_n, g_spv_argv_n);
        SPV_ROW3("_spawnv", "extension-probe", _spawnv, 0, g_spv_noext_n, g_spv_argv_n);
    } else {
        grade2("_spawnv.fixture", "setup", 0, 1); /* fail loud */
    }
}

static void run_spv__spawnve(void)
{
    install_counting_iph();
    SPV_ROW4("_spawnve", "NULL-name", _spawnve, 0, (const char *)0, g_spv_argv_n, g_spv_env_n);
    SPV_ROW4("_spawnve", "empty-name", _spawnve, 0, "", g_spv_argv_n, g_spv_env_n);
    SPV_ROW4("_spawnve", "NULL-argv", _spawnve, 0, g_spv_full_n, (const char *const *)0, g_spv_env_n);
    SPV_ROW4("_spawnve", "empty-argv", _spawnve, 0, g_spv_full_n, g_spv_argv_empty_n, g_spv_env_n);
    SPV_ROW4("_spawnve", "invalid-mode", _spawnve, 999, g_spv_full_n, g_spv_argv_n, g_spv_env_n);
    if (spv_setup()) {
        SPV_ROW4("_spawnve", "nonexistent", _spawnve, 0, g_spv_missing_n, g_spv_argv_n, g_spv_env_n);
        SPV_ROW4("_spawnve", "wait-fullpath-NULLenv", _spawnve, 0, g_spv_full_n, g_spv_argv_n, (const char *const *)0);
        SPV_ROW4("_spawnve", "wait-fullpath-customenv", _spawnve, 0, g_spv_full_n, g_spv_argv_n, g_spv_env_n);
        SPV_ROW4("_spawnve", "extension-probe", _spawnve, 0, g_spv_noext_n, g_spv_argv_n, g_spv_env_n);
    } else {
        grade2("_spawnve.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnv(void)
{
    install_counting_iph();
    SPV_ROW3("_wspawnv", "NULL-name", _wspawnv, 0, (const wchar_t *)0, g_spv_argv_w);
    SPV_ROW3("_wspawnv", "empty-name", _wspawnv, 0, L"", g_spv_argv_w);
    SPV_ROW3("_wspawnv", "NULL-argv", _wspawnv, 0, g_spv_full_w, (const wchar_t *const *)0);
    SPV_ROW3("_wspawnv", "empty-argv", _wspawnv, 0, g_spv_full_w, g_spv_argv_empty_w);
    SPV_ROW3("_wspawnv", "empty-argv0-str", _wspawnv, 0, g_spv_full_w, g_spv_argv_emptystr_w);
    SPV_ROW3("_wspawnv", "invalid-mode", _wspawnv, 999, g_spv_full_w, g_spv_argv_w);
    if (spv_setup()) {
        SPV_ROW3("_wspawnv", "nonexistent", _wspawnv, 0, g_spv_missing_w, g_spv_argv_w);
        SPV_ROW3("_wspawnv", "wait-fullpath", _wspawnv, 0, g_spv_full_w, g_spv_argv_w);
        SPV_ROW3("_wspawnv", "extension-probe", _wspawnv, 0, g_spv_noext_w, g_spv_argv_w);
    } else {
        grade2("_wspawnv.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnve(void)
{
    install_counting_iph();
    SPV_ROW4("_wspawnve", "NULL-name", _wspawnve, 0, (const wchar_t *)0, g_spv_argv_w, g_spv_env_w);
    SPV_ROW4("_wspawnve", "empty-name", _wspawnve, 0, L"", g_spv_argv_w, g_spv_env_w);
    SPV_ROW4("_wspawnve", "NULL-argv", _wspawnve, 0, g_spv_full_w, (const wchar_t *const *)0, g_spv_env_w);
    SPV_ROW4("_wspawnve", "empty-argv", _wspawnve, 0, g_spv_full_w, g_spv_argv_empty_w, g_spv_env_w);
    SPV_ROW4("_wspawnve", "invalid-mode", _wspawnve, 999, g_spv_full_w, g_spv_argv_w, g_spv_env_w);
    if (spv_setup()) {
        SPV_ROW4("_wspawnve", "nonexistent", _wspawnve, 0, g_spv_missing_w, g_spv_argv_w, g_spv_env_w);
        SPV_ROW4("_wspawnve", "wait-fullpath-NULLenv", _wspawnve, 0, g_spv_full_w, g_spv_argv_w, (const wchar_t *const *)0);
        SPV_ROW4("_wspawnve", "wait-fullpath-customenv", _wspawnve, 0, g_spv_full_w, g_spv_argv_w, g_spv_env_w);
        SPV_ROW4("_wspawnve", "extension-probe", _wspawnve, 0, g_spv_noext_w, g_spv_argv_w, g_spv_env_w);
    } else {
        grade2("_wspawnve.fixture", "setup", 0, 1);
    }
}

#endif /* UCRTDIFF_REF_HAS__spawnv */

/* ========================================================================
 * spawnvp.cpp: _execvp/_execvpe/_spawnvp/_spawnvpe, narrow and wide.
 * ======================================================================== */
#if UCRTDIFF_REF_HAS__spawnvp

static void run_spv__execvp(void)
{
    install_counting_iph();
    SPV_ROW2("_execvp", "NULL-name", _execvp, (const char *)0, g_spv_argv_n);
    SPV_ROW2("_execvp", "empty-name", _execvp, "", g_spv_argv_n);
    SPV_ROW2("_execvp", "NULL-argv", _execvp, g_spv_bare_n, (const char *const *)0);
    SPV_ROW2("_execvp", "empty-argv", _execvp, g_spv_bare_n, g_spv_argv_empty_n);
}

static void run_spv__execvpe(void)
{
    install_counting_iph();
    SPV_ROW3("_execvpe", "NULL-name", _execvpe, (const char *)0, g_spv_argv_n, g_spv_env_n);
    SPV_ROW3("_execvpe", "empty-name", _execvpe, "", g_spv_argv_n, g_spv_env_n);
    SPV_ROW3("_execvpe", "NULL-argv", _execvpe, g_spv_bare_n, (const char *const *)0, g_spv_env_n);
    SPV_ROW3("_execvpe", "empty-argv", _execvpe, g_spv_bare_n, g_spv_argv_empty_n, g_spv_env_n);
}

static void run_spv__wexecvp(void)
{
    install_counting_iph();
    SPV_ROW2("_wexecvp", "NULL-name", _wexecvp, (const wchar_t *)0, g_spv_argv_w);
    SPV_ROW2("_wexecvp", "empty-name", _wexecvp, L"", g_spv_argv_w);
    SPV_ROW2("_wexecvp", "NULL-argv", _wexecvp, g_spv_bare_w, (const wchar_t *const *)0);
    SPV_ROW2("_wexecvp", "empty-argv", _wexecvp, g_spv_bare_w, g_spv_argv_empty_w);
}

static void run_spv__wexecvpe(void)
{
    install_counting_iph();
    SPV_ROW3("_wexecvpe", "NULL-name", _wexecvpe, (const wchar_t *)0, g_spv_argv_w, g_spv_env_w);
    SPV_ROW3("_wexecvpe", "empty-name", _wexecvpe, L"", g_spv_argv_w, g_spv_env_w);
    SPV_ROW3("_wexecvpe", "NULL-argv", _wexecvpe, g_spv_bare_w, (const wchar_t *const *)0, g_spv_env_w);
    SPV_ROW3("_wexecvpe", "empty-argv", _wexecvpe, g_spv_bare_w, g_spv_argv_empty_w, g_spv_env_w);
}

static void run_spv__spawnvp(void)
{
    install_counting_iph();
    SPV_ROW3("_spawnvp", "NULL-name", _spawnvp, 0, (const char *)0, g_spv_argv_n);
    SPV_ROW3("_spawnvp", "empty-name", _spawnvp, 0, "", g_spv_argv_n);
    SPV_ROW3("_spawnvp", "NULL-argv", _spawnvp, 0, g_spv_bare_n, (const char *const *)0);
    SPV_ROW3("_spawnvp", "empty-argv", _spawnvp, 0, g_spv_bare_n, g_spv_argv_empty_n);
    SPV_ROW3("_spawnvp", "invalid-mode", _spawnvp, 999, g_spv_bare_n, g_spv_argv_n);
    /* Absolute path with no `p` in play: matches non-`p` "nonexistent". */
    SPV_ROW3("_spawnvp", "nonexistent-abs", _spawnvp, 0, g_spv_missing_n, g_spv_argv_n);
    if (spv_setup()) {
        SPV_ROW3("_spawnvp", "path-search", _spawnvp, 0, g_spv_bare_n, g_spv_argv_n);
    } else {
        grade2("_spawnvp.fixture", "setup", 0, 1);
    }
}

static void run_spv__spawnvpe(void)
{
    install_counting_iph();
    SPV_ROW4("_spawnvpe", "NULL-name", _spawnvpe, 0, (const char *)0, g_spv_argv_n, g_spv_env_n);
    SPV_ROW4("_spawnvpe", "empty-name", _spawnvpe, 0, "", g_spv_argv_n, g_spv_env_n);
    SPV_ROW4("_spawnvpe", "NULL-argv", _spawnvpe, 0, g_spv_bare_n, (const char *const *)0, g_spv_env_n);
    SPV_ROW4("_spawnvpe", "empty-argv", _spawnvpe, 0, g_spv_bare_n, g_spv_argv_empty_n, g_spv_env_n);
    SPV_ROW4("_spawnvpe", "invalid-mode", _spawnvpe, 999, g_spv_bare_n, g_spv_argv_n, g_spv_env_n);
    SPV_ROW4("_spawnvpe", "nonexistent-abs", _spawnvpe, 0, g_spv_missing_n, g_spv_argv_n, g_spv_env_n);
    if (spv_setup()) {
        SPV_ROW4("_spawnvpe", "path-search-NULLenv", _spawnvpe, 0, g_spv_bare_n, g_spv_argv_n, (const char *const *)0);
        SPV_ROW4("_spawnvpe", "path-search-customenv", _spawnvpe, 0, g_spv_bare_n, g_spv_argv_n, g_spv_env_n);
    } else {
        grade2("_spawnvpe.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnvp(void)
{
    install_counting_iph();
    SPV_ROW3("_wspawnvp", "NULL-name", _wspawnvp, 0, (const wchar_t *)0, g_spv_argv_w);
    SPV_ROW3("_wspawnvp", "empty-name", _wspawnvp, 0, L"", g_spv_argv_w);
    SPV_ROW3("_wspawnvp", "NULL-argv", _wspawnvp, 0, g_spv_bare_w, (const wchar_t *const *)0);
    SPV_ROW3("_wspawnvp", "empty-argv", _wspawnvp, 0, g_spv_bare_w, g_spv_argv_empty_w);
    SPV_ROW3("_wspawnvp", "invalid-mode", _wspawnvp, 999, g_spv_bare_w, g_spv_argv_w);
    SPV_ROW3("_wspawnvp", "nonexistent-abs", _wspawnvp, 0, g_spv_missing_w, g_spv_argv_w);
    if (spv_setup()) {
        SPV_ROW3("_wspawnvp", "path-search", _wspawnvp, 0, g_spv_bare_w, g_spv_argv_w);
    } else {
        grade2("_wspawnvp.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnvpe(void)
{
    install_counting_iph();
    SPV_ROW4("_wspawnvpe", "NULL-name", _wspawnvpe, 0, (const wchar_t *)0, g_spv_argv_w, g_spv_env_w);
    SPV_ROW4("_wspawnvpe", "empty-name", _wspawnvpe, 0, L"", g_spv_argv_w, g_spv_env_w);
    SPV_ROW4("_wspawnvpe", "NULL-argv", _wspawnvpe, 0, g_spv_bare_w, (const wchar_t *const *)0, g_spv_env_w);
    SPV_ROW4("_wspawnvpe", "empty-argv", _wspawnvpe, 0, g_spv_bare_w, g_spv_argv_empty_w, g_spv_env_w);
    SPV_ROW4("_wspawnvpe", "invalid-mode", _wspawnvpe, 999, g_spv_bare_w, g_spv_argv_w, g_spv_env_w);
    SPV_ROW4("_wspawnvpe", "nonexistent-abs", _wspawnvpe, 0, g_spv_missing_w, g_spv_argv_w, g_spv_env_w);
    if (spv_setup()) {
        SPV_ROW4("_wspawnvpe", "path-search-NULLenv", _wspawnvpe, 0, g_spv_bare_w, g_spv_argv_w, (const wchar_t *const *)0);
        SPV_ROW4("_wspawnvpe", "path-search-customenv", _wspawnvpe, 0, g_spv_bare_w, g_spv_argv_w, g_spv_env_w);
    } else {
        grade2("_wspawnvpe.fixture", "setup", 0, 1);
    }
}

#endif /* UCRTDIFF_REF_HAS__spawnvp */

/* ========================================================================
 * spawnl.cpp: _execl/_execle/_spawnl/_spawnle, narrow and wide. These are
 * genuinely C-variadic exports, so they are called here with real "..."
 * syntax (unlike the -v forms above, no wrapper is needed for the call
 * itself -- only the shared SPV_ROW grading macro is reused).
 *
 * "Several lengths, including more than the caller-array size" (64): SPV_X16
 * gives 17 total captured pointers (well inside 64, no growth); SPV_X64
 * gives 65 (one MORE than 64), which is exactly the point where
 * `common_capture_argv`'s FIRST heap growth -- and its `memcpy_s` byte/element
 * quirk (see exec.rs's module docs) -- fires. The many-argument row targets
 * `g_spv_missing_n`/`_w` (an absolute, nonexistent path) rather than a real
 * cmd.exe: this exercises the FULL variadic-capture-and-dispatch path (the
 * capture happens before `common_spawnv`/`common_spawnvp` ever runs) while
 * guaranteeing `_access_s`/`_waccess_s` rejects the file before any process
 * is ever created -- no real subprocess, no dependence on what a real
 * process would do with garbage arguments.
 * ======================================================================== */
#if UCRTDIFF_REF_HAS__spawnl

intptr_t __cdecl ref__execl(const char *, const char *, ...);
intptr_t __cdecl ref__execle(const char *, const char *, ...);
intptr_t __cdecl ref__spawnl(int, const char *, const char *, ...);
intptr_t __cdecl ref__spawnle(int, const char *, const char *, ...);
intptr_t __cdecl ref__wexecl(const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wexecle(const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wspawnl(int, const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wspawnle(int, const wchar_t *, const wchar_t *, ...);

extern intptr_t(__cdecl *const ud__execl)(const char *, const char *, ...) __asm__(UD_IMP(_execl));
extern intptr_t(__cdecl *const ud__execle)(const char *, const char *, ...) __asm__(UD_IMP(_execle));
extern intptr_t(__cdecl *const ud__spawnl)(int, const char *, const char *, ...) __asm__(UD_IMP(_spawnl));
extern intptr_t(__cdecl *const ud__spawnle)(int, const char *, const char *, ...) __asm__(UD_IMP(_spawnle));
extern intptr_t(__cdecl *const ud__wexecl)(const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wexecl));
extern intptr_t(__cdecl *const ud__wexecle)(const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wexecle));
extern intptr_t(__cdecl *const ud__wspawnl)(int, const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wspawnl));
extern intptr_t(__cdecl *const ud__wspawnle)(int, const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wspawnle));

static void run_spv__execl(void)
{
    install_counting_iph();
    SPV_ROW("_execl", "NULL-name", ud__execl((const char *)0, "a", (char *)0),
            ref__execl((const char *)0, "a", (char *)0));
    SPV_ROW("_execl", "empty-name", ud__execl("", "a", (char *)0),
            ref__execl("", "a", (char *)0));
    SPV_ROW("_execl", "empty-args", ud__execl(g_spv_full_n, "", (char *)0),
            ref__execl(g_spv_full_n, "", (char *)0));
}

static void run_spv__execle(void)
{
    install_counting_iph();
    SPV_ROW("_execle", "NULL-name", ud__execle((const char *)0, "a", (char *)0, g_spv_env_n),
            ref__execle((const char *)0, "a", (char *)0, g_spv_env_n));
    SPV_ROW("_execle", "empty-name", ud__execle("", "a", (char *)0, g_spv_env_n),
            ref__execle("", "a", (char *)0, g_spv_env_n));
    SPV_ROW("_execle", "empty-args", ud__execle(g_spv_full_n, "", (char *)0, g_spv_env_n),
            ref__execle(g_spv_full_n, "", (char *)0, g_spv_env_n));
}

static void run_spv__wexecl(void)
{
    install_counting_iph();
    SPV_ROW("_wexecl", "NULL-name", ud__wexecl((const wchar_t *)0, L"a", (wchar_t *)0),
            ref__wexecl((const wchar_t *)0, L"a", (wchar_t *)0));
    SPV_ROW("_wexecl", "empty-name", ud__wexecl(L"", L"a", (wchar_t *)0),
            ref__wexecl(L"", L"a", (wchar_t *)0));
    SPV_ROW("_wexecl", "empty-args", ud__wexecl(g_spv_full_w, L"", (wchar_t *)0),
            ref__wexecl(g_spv_full_w, L"", (wchar_t *)0));
}

static void run_spv__wexecle(void)
{
    install_counting_iph();
    SPV_ROW("_wexecle", "NULL-name", ud__wexecle((const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w),
            ref__wexecle((const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wexecle", "empty-name", ud__wexecle(L"", L"a", (wchar_t *)0, g_spv_env_w),
            ref__wexecle(L"", L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wexecle", "empty-args", ud__wexecle(g_spv_full_w, L"", (wchar_t *)0, g_spv_env_w),
            ref__wexecle(g_spv_full_w, L"", (wchar_t *)0, g_spv_env_w));
}

static void run_spv__spawnl(void)
{
    install_counting_iph();
    SPV_ROW("_spawnl", "NULL-name", ud__spawnl(0, (const char *)0, "a", (char *)0),
            ref__spawnl(0, (const char *)0, "a", (char *)0));
    SPV_ROW("_spawnl", "empty-name", ud__spawnl(0, "", "a", (char *)0),
            ref__spawnl(0, "", "a", (char *)0));
    SPV_ROW("_spawnl", "empty-args", ud__spawnl(0, g_spv_full_n, "", (char *)0),
            ref__spawnl(0, g_spv_full_n, "", (char *)0));
    SPV_ROW("_spawnl", "invalid-mode", ud__spawnl(999, g_spv_full_n, "cmd.exe", (char *)0),
            ref__spawnl(999, g_spv_full_n, "cmd.exe", (char *)0));
    if (spv_setup()) {
        SPV_ROW("_spawnl", "wait-short", ud__spawnl(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0),
                ref__spawnl(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0));
        SPV_ROW("_spawnl", "medium-17-nogrowth",
                ud__spawnl(0, g_spv_missing_n, "cmd.exe", SPV_X16_N, (char *)0),
                ref__spawnl(0, g_spv_missing_n, "cmd.exe", SPV_X16_N, (char *)0));
        SPV_ROW("_spawnl", "many-65-growth",
                ud__spawnl(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0),
                ref__spawnl(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0));
    } else {
        grade2("_spawnl.fixture", "setup", 0, 1);
    }
}

static void run_spv__spawnle(void)
{
    install_counting_iph();
    SPV_ROW("_spawnle", "NULL-name", ud__spawnle(0, (const char *)0, "a", (char *)0, g_spv_env_n),
            ref__spawnle(0, (const char *)0, "a", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnle", "empty-name", ud__spawnle(0, "", "a", (char *)0, g_spv_env_n),
            ref__spawnle(0, "", "a", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnle", "empty-args", ud__spawnle(0, g_spv_full_n, "", (char *)0, g_spv_env_n),
            ref__spawnle(0, g_spv_full_n, "", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnle", "invalid-mode", ud__spawnle(999, g_spv_full_n, "cmd.exe", (char *)0, g_spv_env_n),
            ref__spawnle(999, g_spv_full_n, "cmd.exe", (char *)0, g_spv_env_n));
    if (spv_setup()) {
        SPV_ROW("_spawnle", "wait-short-customenv",
                ud__spawnle(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0, g_spv_env_n),
                ref__spawnle(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0, g_spv_env_n));
        SPV_ROW("_spawnle", "wait-short-NULLenv",
                ud__spawnle(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0, (const char *const *)0),
                ref__spawnle(0, g_spv_full_n, "cmd.exe", "/c", "exit", "7", (char *)0, (const char *const *)0));
        SPV_ROW("_spawnle", "many-65-growth",
                ud__spawnle(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0, g_spv_env_n),
                ref__spawnle(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0, g_spv_env_n));
    } else {
        grade2("_spawnle.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnl(void)
{
    install_counting_iph();
    SPV_ROW("_wspawnl", "NULL-name", ud__wspawnl(0, (const wchar_t *)0, L"a", (wchar_t *)0),
            ref__wspawnl(0, (const wchar_t *)0, L"a", (wchar_t *)0));
    SPV_ROW("_wspawnl", "empty-name", ud__wspawnl(0, L"", L"a", (wchar_t *)0),
            ref__wspawnl(0, L"", L"a", (wchar_t *)0));
    SPV_ROW("_wspawnl", "empty-args", ud__wspawnl(0, g_spv_full_w, L"", (wchar_t *)0),
            ref__wspawnl(0, g_spv_full_w, L"", (wchar_t *)0));
    SPV_ROW("_wspawnl", "invalid-mode", ud__wspawnl(999, g_spv_full_w, L"cmd.exe", (wchar_t *)0),
            ref__wspawnl(999, g_spv_full_w, L"cmd.exe", (wchar_t *)0));
    if (spv_setup()) {
        SPV_ROW("_wspawnl", "wait-short", ud__wspawnl(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0),
                ref__wspawnl(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0));
        SPV_ROW("_wspawnl", "medium-17-nogrowth",
                ud__wspawnl(0, g_spv_missing_w, L"cmd.exe", SPV_X16_W, (wchar_t *)0),
                ref__wspawnl(0, g_spv_missing_w, L"cmd.exe", SPV_X16_W, (wchar_t *)0));
        SPV_ROW("_wspawnl", "many-65-growth",
                ud__wspawnl(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0),
                ref__wspawnl(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0));
    } else {
        grade2("_wspawnl.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnle(void)
{
    install_counting_iph();
    SPV_ROW("_wspawnle", "NULL-name", ud__wspawnle(0, (const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w),
            ref__wspawnle(0, (const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnle", "empty-name", ud__wspawnle(0, L"", L"a", (wchar_t *)0, g_spv_env_w),
            ref__wspawnle(0, L"", L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnle", "empty-args", ud__wspawnle(0, g_spv_full_w, L"", (wchar_t *)0, g_spv_env_w),
            ref__wspawnle(0, g_spv_full_w, L"", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnle", "invalid-mode", ud__wspawnle(999, g_spv_full_w, L"cmd.exe", (wchar_t *)0, g_spv_env_w),
            ref__wspawnle(999, g_spv_full_w, L"cmd.exe", (wchar_t *)0, g_spv_env_w));
    if (spv_setup()) {
        SPV_ROW("_wspawnle", "wait-short-customenv",
                ud__wspawnle(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, g_spv_env_w),
                ref__wspawnle(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, g_spv_env_w));
        SPV_ROW("_wspawnle", "wait-short-NULLenv",
                ud__wspawnle(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, (const wchar_t *const *)0),
                ref__wspawnle(0, g_spv_full_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, (const wchar_t *const *)0));
        SPV_ROW("_wspawnle", "many-65-growth",
                ud__wspawnle(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0, g_spv_env_w),
                ref__wspawnle(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0, g_spv_env_w));
    } else {
        grade2("_wspawnle.fixture", "setup", 0, 1);
    }
}

#endif /* UCRTDIFF_REF_HAS__spawnl */

/* ========================================================================
 * spawnlp.cpp: _execlp/_execlpe/_spawnlp/_spawnlpe, narrow and wide.
 * ======================================================================== */
#if UCRTDIFF_REF_HAS__spawnlp

intptr_t __cdecl ref__execlp(const char *, const char *, ...);
intptr_t __cdecl ref__execlpe(const char *, const char *, ...);
intptr_t __cdecl ref__spawnlp(int, const char *, const char *, ...);
intptr_t __cdecl ref__spawnlpe(int, const char *, const char *, ...);
intptr_t __cdecl ref__wexeclp(const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wexeclpe(const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wspawnlp(int, const wchar_t *, const wchar_t *, ...);
intptr_t __cdecl ref__wspawnlpe(int, const wchar_t *, const wchar_t *, ...);

extern intptr_t(__cdecl *const ud__execlp)(const char *, const char *, ...) __asm__(UD_IMP(_execlp));
extern intptr_t(__cdecl *const ud__execlpe)(const char *, const char *, ...) __asm__(UD_IMP(_execlpe));
extern intptr_t(__cdecl *const ud__spawnlp)(int, const char *, const char *, ...) __asm__(UD_IMP(_spawnlp));
extern intptr_t(__cdecl *const ud__spawnlpe)(int, const char *, const char *, ...) __asm__(UD_IMP(_spawnlpe));
extern intptr_t(__cdecl *const ud__wexeclp)(const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wexeclp));
extern intptr_t(__cdecl *const ud__wexeclpe)(const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wexeclpe));
extern intptr_t(__cdecl *const ud__wspawnlp)(int, const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wspawnlp));
extern intptr_t(__cdecl *const ud__wspawnlpe)(int, const wchar_t *, const wchar_t *, ...) __asm__(UD_IMP(_wspawnlpe));

static void run_spv__execlp(void)
{
    install_counting_iph();
    SPV_ROW("_execlp", "NULL-name", ud__execlp((const char *)0, "a", (char *)0),
            ref__execlp((const char *)0, "a", (char *)0));
    SPV_ROW("_execlp", "empty-name", ud__execlp("", "a", (char *)0),
            ref__execlp("", "a", (char *)0));
    SPV_ROW("_execlp", "empty-args", ud__execlp(g_spv_bare_n, "", (char *)0),
            ref__execlp(g_spv_bare_n, "", (char *)0));
}

static void run_spv__execlpe(void)
{
    install_counting_iph();
    SPV_ROW("_execlpe", "NULL-name", ud__execlpe((const char *)0, "a", (char *)0, g_spv_env_n),
            ref__execlpe((const char *)0, "a", (char *)0, g_spv_env_n));
    SPV_ROW("_execlpe", "empty-name", ud__execlpe("", "a", (char *)0, g_spv_env_n),
            ref__execlpe("", "a", (char *)0, g_spv_env_n));
    SPV_ROW("_execlpe", "empty-args", ud__execlpe(g_spv_bare_n, "", (char *)0, g_spv_env_n),
            ref__execlpe(g_spv_bare_n, "", (char *)0, g_spv_env_n));
}

static void run_spv__wexeclp(void)
{
    install_counting_iph();
    SPV_ROW("_wexeclp", "NULL-name", ud__wexeclp((const wchar_t *)0, L"a", (wchar_t *)0),
            ref__wexeclp((const wchar_t *)0, L"a", (wchar_t *)0));
    SPV_ROW("_wexeclp", "empty-name", ud__wexeclp(L"", L"a", (wchar_t *)0),
            ref__wexeclp(L"", L"a", (wchar_t *)0));
    SPV_ROW("_wexeclp", "empty-args", ud__wexeclp(g_spv_bare_w, L"", (wchar_t *)0),
            ref__wexeclp(g_spv_bare_w, L"", (wchar_t *)0));
}

static void run_spv__wexeclpe(void)
{
    install_counting_iph();
    SPV_ROW("_wexeclpe", "NULL-name", ud__wexeclpe((const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w),
            ref__wexeclpe((const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wexeclpe", "empty-name", ud__wexeclpe(L"", L"a", (wchar_t *)0, g_spv_env_w),
            ref__wexeclpe(L"", L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wexeclpe", "empty-args", ud__wexeclpe(g_spv_bare_w, L"", (wchar_t *)0, g_spv_env_w),
            ref__wexeclpe(g_spv_bare_w, L"", (wchar_t *)0, g_spv_env_w));
}

static void run_spv__spawnlp(void)
{
    install_counting_iph();
    SPV_ROW("_spawnlp", "NULL-name", ud__spawnlp(0, (const char *)0, "a", (char *)0),
            ref__spawnlp(0, (const char *)0, "a", (char *)0));
    SPV_ROW("_spawnlp", "empty-name", ud__spawnlp(0, "", "a", (char *)0),
            ref__spawnlp(0, "", "a", (char *)0));
    SPV_ROW("_spawnlp", "empty-args", ud__spawnlp(0, g_spv_bare_n, "", (char *)0),
            ref__spawnlp(0, g_spv_bare_n, "", (char *)0));
    SPV_ROW("_spawnlp", "invalid-mode", ud__spawnlp(999, g_spv_bare_n, "cmd.exe", (char *)0),
            ref__spawnlp(999, g_spv_bare_n, "cmd.exe", (char *)0));
    SPV_ROW("_spawnlp", "nonexistent-abs", ud__spawnlp(0, g_spv_missing_n, "cmd.exe", (char *)0),
            ref__spawnlp(0, g_spv_missing_n, "cmd.exe", (char *)0));
    if (spv_setup()) {
        SPV_ROW("_spawnlp", "path-search-short",
                ud__spawnlp(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0),
                ref__spawnlp(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0));
        SPV_ROW("_spawnlp", "medium-17-nogrowth",
                ud__spawnlp(0, g_spv_missing_n, "cmd.exe", SPV_X16_N, (char *)0),
                ref__spawnlp(0, g_spv_missing_n, "cmd.exe", SPV_X16_N, (char *)0));
        SPV_ROW("_spawnlp", "many-65-growth",
                ud__spawnlp(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0),
                ref__spawnlp(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0));
    } else {
        grade2("_spawnlp.fixture", "setup", 0, 1);
    }
}

static void run_spv__spawnlpe(void)
{
    install_counting_iph();
    SPV_ROW("_spawnlpe", "NULL-name", ud__spawnlpe(0, (const char *)0, "a", (char *)0, g_spv_env_n),
            ref__spawnlpe(0, (const char *)0, "a", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnlpe", "empty-name", ud__spawnlpe(0, "", "a", (char *)0, g_spv_env_n),
            ref__spawnlpe(0, "", "a", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnlpe", "empty-args", ud__spawnlpe(0, g_spv_bare_n, "", (char *)0, g_spv_env_n),
            ref__spawnlpe(0, g_spv_bare_n, "", (char *)0, g_spv_env_n));
    SPV_ROW("_spawnlpe", "invalid-mode", ud__spawnlpe(999, g_spv_bare_n, "cmd.exe", (char *)0, g_spv_env_n),
            ref__spawnlpe(999, g_spv_bare_n, "cmd.exe", (char *)0, g_spv_env_n));
    if (spv_setup()) {
        SPV_ROW("_spawnlpe", "path-search-customenv",
                ud__spawnlpe(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0, g_spv_env_n),
                ref__spawnlpe(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0, g_spv_env_n));
        SPV_ROW("_spawnlpe", "path-search-NULLenv",
                ud__spawnlpe(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0, (const char *const *)0),
                ref__spawnlpe(0, g_spv_bare_n, "cmd.exe", "/c", "exit", "7", (char *)0, (const char *const *)0));
        SPV_ROW("_spawnlpe", "many-65-growth",
                ud__spawnlpe(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0, g_spv_env_n),
                ref__spawnlpe(0, g_spv_missing_n, "cmd.exe", SPV_X64_N, (char *)0, g_spv_env_n));
    } else {
        grade2("_spawnlpe.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnlp(void)
{
    install_counting_iph();
    SPV_ROW("_wspawnlp", "NULL-name", ud__wspawnlp(0, (const wchar_t *)0, L"a", (wchar_t *)0),
            ref__wspawnlp(0, (const wchar_t *)0, L"a", (wchar_t *)0));
    SPV_ROW("_wspawnlp", "empty-name", ud__wspawnlp(0, L"", L"a", (wchar_t *)0),
            ref__wspawnlp(0, L"", L"a", (wchar_t *)0));
    SPV_ROW("_wspawnlp", "empty-args", ud__wspawnlp(0, g_spv_bare_w, L"", (wchar_t *)0),
            ref__wspawnlp(0, g_spv_bare_w, L"", (wchar_t *)0));
    SPV_ROW("_wspawnlp", "invalid-mode", ud__wspawnlp(999, g_spv_bare_w, L"cmd.exe", (wchar_t *)0),
            ref__wspawnlp(999, g_spv_bare_w, L"cmd.exe", (wchar_t *)0));
    SPV_ROW("_wspawnlp", "nonexistent-abs", ud__wspawnlp(0, g_spv_missing_w, L"cmd.exe", (wchar_t *)0),
            ref__wspawnlp(0, g_spv_missing_w, L"cmd.exe", (wchar_t *)0));
    if (spv_setup()) {
        SPV_ROW("_wspawnlp", "path-search-short",
                ud__wspawnlp(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0),
                ref__wspawnlp(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0));
        SPV_ROW("_wspawnlp", "medium-17-nogrowth",
                ud__wspawnlp(0, g_spv_missing_w, L"cmd.exe", SPV_X16_W, (wchar_t *)0),
                ref__wspawnlp(0, g_spv_missing_w, L"cmd.exe", SPV_X16_W, (wchar_t *)0));
        SPV_ROW("_wspawnlp", "many-65-growth",
                ud__wspawnlp(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0),
                ref__wspawnlp(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0));
    } else {
        grade2("_wspawnlp.fixture", "setup", 0, 1);
    }
}

static void run_spv__wspawnlpe(void)
{
    install_counting_iph();
    SPV_ROW("_wspawnlpe", "NULL-name", ud__wspawnlpe(0, (const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w),
            ref__wspawnlpe(0, (const wchar_t *)0, L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnlpe", "empty-name", ud__wspawnlpe(0, L"", L"a", (wchar_t *)0, g_spv_env_w),
            ref__wspawnlpe(0, L"", L"a", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnlpe", "empty-args", ud__wspawnlpe(0, g_spv_bare_w, L"", (wchar_t *)0, g_spv_env_w),
            ref__wspawnlpe(0, g_spv_bare_w, L"", (wchar_t *)0, g_spv_env_w));
    SPV_ROW("_wspawnlpe", "invalid-mode", ud__wspawnlpe(999, g_spv_bare_w, L"cmd.exe", (wchar_t *)0, g_spv_env_w),
            ref__wspawnlpe(999, g_spv_bare_w, L"cmd.exe", (wchar_t *)0, g_spv_env_w));
    if (spv_setup()) {
        SPV_ROW("_wspawnlpe", "path-search-customenv",
                ud__wspawnlpe(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, g_spv_env_w),
                ref__wspawnlpe(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, g_spv_env_w));
        SPV_ROW("_wspawnlpe", "path-search-NULLenv",
                ud__wspawnlpe(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, (const wchar_t *const *)0),
                ref__wspawnlpe(0, g_spv_bare_w, L"cmd.exe", L"/c", L"exit", L"7", (wchar_t *)0, (const wchar_t *const *)0));
        SPV_ROW("_wspawnlpe", "many-65-growth",
                ud__wspawnlpe(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0, g_spv_env_w),
                ref__wspawnlpe(0, g_spv_missing_w, L"cmd.exe", SPV_X64_W, (wchar_t *)0, g_spv_env_w));
    } else {
        grade2("_wspawnlpe.fixture", "setup", 0, 1);
    }
}

#endif /* UCRTDIFF_REF_HAS__spawnlp */

/* ---- registry macro: paste into run_legacy()'s table -------------------- */
#if UCRTDIFF_REF_HAS__spawnv
#define SPV_HAND_ENTRIES_SPAWNV \
    { "_execv", run_spv__execv }, { "_execve", run_spv__execve }, \
    { "_wexecv", run_spv__wexecv }, { "_wexecve", run_spv__wexecve }, \
    { "_spawnv", run_spv__spawnv }, { "_spawnve", run_spv__spawnve }, \
    { "_wspawnv", run_spv__wspawnv }, { "_wspawnve", run_spv__wspawnve },
#else
#define SPV_HAND_ENTRIES_SPAWNV \
    { "_execv", 0 }, { "_execve", 0 }, { "_wexecv", 0 }, { "_wexecve", 0 }, \
    { "_spawnv", 0 }, { "_spawnve", 0 }, { "_wspawnv", 0 }, { "_wspawnve", 0 },
#endif

#if UCRTDIFF_REF_HAS__spawnvp
#define SPV_HAND_ENTRIES_SPAWNVP \
    { "_execvp", run_spv__execvp }, { "_execvpe", run_spv__execvpe }, \
    { "_wexecvp", run_spv__wexecvp }, { "_wexecvpe", run_spv__wexecvpe }, \
    { "_spawnvp", run_spv__spawnvp }, { "_spawnvpe", run_spv__spawnvpe }, \
    { "_wspawnvp", run_spv__wspawnvp }, { "_wspawnvpe", run_spv__wspawnvpe },
#else
#define SPV_HAND_ENTRIES_SPAWNVP \
    { "_execvp", 0 }, { "_execvpe", 0 }, { "_wexecvp", 0 }, { "_wexecvpe", 0 }, \
    { "_spawnvp", 0 }, { "_spawnvpe", 0 }, { "_wspawnvp", 0 }, { "_wspawnvpe", 0 },
#endif

#if UCRTDIFF_REF_HAS__spawnl
#define SPV_HAND_ENTRIES_SPAWNL \
    { "_execl", run_spv__execl }, { "_execle", run_spv__execle }, \
    { "_wexecl", run_spv__wexecl }, { "_wexecle", run_spv__wexecle }, \
    { "_spawnl", run_spv__spawnl }, { "_spawnle", run_spv__spawnle }, \
    { "_wspawnl", run_spv__wspawnl }, { "_wspawnle", run_spv__wspawnle },
#else
#define SPV_HAND_ENTRIES_SPAWNL \
    { "_execl", 0 }, { "_execle", 0 }, { "_wexecl", 0 }, { "_wexecle", 0 }, \
    { "_spawnl", 0 }, { "_spawnle", 0 }, { "_wspawnl", 0 }, { "_wspawnle", 0 },
#endif

#if UCRTDIFF_REF_HAS__spawnlp
#define SPV_HAND_ENTRIES_SPAWNLP \
    { "_execlp", run_spv__execlp }, { "_execlpe", run_spv__execlpe }, \
    { "_wexeclp", run_spv__wexeclp }, { "_wexeclpe", run_spv__wexeclpe }, \
    { "_spawnlp", run_spv__spawnlp }, { "_spawnlpe", run_spv__spawnlpe }, \
    { "_wspawnlp", run_spv__wspawnlp }, { "_wspawnlpe", run_spv__wspawnlpe },
#else
#define SPV_HAND_ENTRIES_SPAWNLP \
    { "_execlp", 0 }, { "_execlpe", 0 }, { "_wexeclp", 0 }, { "_wexeclpe", 0 }, \
    { "_spawnlp", 0 }, { "_spawnlpe", 0 }, { "_wspawnlp", 0 }, { "_wspawnlpe", 0 },
#endif

#define SPV_HAND_ENTRIES \
    SPV_HAND_ENTRIES_SPAWNV SPV_HAND_ENTRIES_SPAWNVP \
    SPV_HAND_ENTRIES_SPAWNL SPV_HAND_ENTRIES_SPAWNLP


/* ==== staging/mbstring.c, inlined ==== */
/* mbstring batch grader -- graded exports from sdk/lib/rust/ucrt-rust/src/mbstring/.
 *
 * Included by the lead into ucrtdiff.c just before `run_legacy()`, so everything
 * declared earlier in that file (grade2, fnv, label_idx, install_counting_iph,
 * g_iph_dll/g_iph_ref, _errno/ref__errno, __doserrno/ref___doserrno, UD_IMP,
 * g_mbstr/G_MBSTR_N, ud__setmbcp/ud__getmbcp, ref__setmbcp/ref__getmbcp) is in
 * scope here and reused rather than redeclared.
 *
 * SCOPE: one runner per C FILE (not per export) -- with 46 files / ~98 exports
 * in this batch, per-export runners would make this file unreviewable. Each
 * runner grades the file's plain form AND its `_l` form (locale NULL, which
 * must equal plain) over a shared corpus, plus the boundary/NULL cases each
 * file's own contract calls out. Every guard below is named after ONE export
 * of its file, per file, since files may be deleted independently of each
 * other.
 *
 * CODEPAGE: `_setmbcp(932)` is exercised once, explicitly, via `run_mbstring_cp932`
 * (guarded on `_mbschr` + `_setmbcp` both existing) which switches BOTH sides,
 * regrades a Shift-JIS-heavy subset of the corpus, and restores the original
 * codepage on both sides before returning -- see that function for exactly
 * which runners it re-invokes. Every other runner below runs at whatever
 * codepage is already installed (1252 by default, so its own multibyte-lead
 * bytes in `g_mbstr` do not classify as lead bytes there -- see the corpus
 * comment). NOTHING here changes codepage 932 permanently: every runner
 * restores it before returning, per the "no process-wide side effects" rule.
 *
 * PROCESS-STATE DISCIPLINE: no runner here calls anything that could
 * terminate the process or block. `_mbstok`/`_mbstok_s` touch the calling
 * thread's ptd-owned token slot (per-thread state private to this test
 * thread, not shared) -- safe. `_setmbcp` is process-wide, which is exactly
 * why `run_mbstring_cp932` is the ONE place it is called and always restores
 * the prior value, even on an early return path (no early returns are used
 * there for this reason).
 */

/* ---- corpus --------------------------------------------------------------
 * `g_mbstr` (from ucrtdiff.c) covers ASCII, empty, and a couple of DBCS-shaped
 * byte sequences, but those DBCS sequences are only genuinely multibyte under
 * a DBCS codepage -- at the default 1252 they are just high bytes. A second,
 * Shift-JIS-specific corpus is added here for the codepage-932 pass, including
 * the mandated "lone lead byte at end of string" case.
 */
static const unsigned char *const g_sjis[] = {
    (const unsigned char *)"",
    (const unsigned char *)"abc",
    (const unsigned char *)"\x82\xa0\x82\xa2",         /* full DBCS pair x2 (hiragana) */
    (const unsigned char *)"a\x82\xa0z",                 /* SBCS + DBCS + SBCS */
    (const unsigned char *)"\x81\x40",                   /* one lead+trail pair, 0x81 class lead */
    (const unsigned char *)"\x82\xa0\x81",               /* DBCS pair then a LONE trailing lead byte */
};
#define G_SJIS_N (sizeof(g_sjis) / sizeof(g_sjis[0]))

/* Unconditional (NOT inside any `#if UCRTDIFF_REF_HAS_*` guard): several
 * independently-guarded runners below share these, and a runner's guard
 * must not depend on some OTHER file's guard being on too -- that bug
 * existed here once (`MBS_BUF` was defined inside the `_mbsncpy` guard) and
 * was caught by syntax-checking a MIXED guard combination, not just
 * all-1/all-0. */
#define MBS_BUF 16
#define MBS_TRUNCATE ((size_t)-1)

/* ================================ mbclen / mbccpy ======================= */
#if UCRTDIFF_REF_HAS__mbclen
extern size_t __cdecl ref__mbclen(const unsigned char *);
extern size_t __cdecl ref__mbclen_l(const unsigned char *, _locale_t);
extern size_t(__cdecl *const ud__mbclen)(const unsigned char *) __asm__(UD_IMP(_mbclen));
extern size_t(__cdecl *const ud__mbclen_l)(const unsigned char *, _locale_t) __asm__(UD_IMP(_mbclen_l));
extern void(__cdecl *const ud__mbccpy)(unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbccpy));
extern void __cdecl ref__mbccpy(unsigned char *, const unsigned char *);

static void run_mbstring_mbclen(void)
{
    size_t i;
    for (i = 0; i < G_MBSTR_N; i++) {
        const char *lbl = label_idx(i);
        grade2("_mbclen", lbl, (long long)ud__mbclen(g_mbstr[i]), (long long)ref__mbclen(g_mbstr[i]));
        grade2("_mbclen_l", lbl, (long long)ud__mbclen_l(g_mbstr[i], 0), (long long)ref__mbclen_l(g_mbstr[i], 0));
        {
            unsigned char b1[4] = { 0xEE, 0xEE, 0xEE, 0xEE }, b2[4] = { 0xEE, 0xEE, 0xEE, 0xEE };
            if (g_mbstr[i][0] != 0) {
                ud__mbccpy(b1, g_mbstr[i]);
                ref__mbccpy(b2, g_mbstr[i]);
                grade2("_mbccpy.buf", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));
            }
        }
    }
}
#else
static void run_mbstring_mbclen(void) {}
#endif

/* ================================ mbschr / mbsrchr ======================= */
#if UCRTDIFF_REF_HAS__mbschr
extern unsigned char *__cdecl ref__mbschr(const unsigned char *, unsigned int);
extern unsigned char *__cdecl ref__mbschr_l(const unsigned char *, unsigned int, _locale_t);
extern unsigned char *__cdecl ref__mbsrchr(const unsigned char *, unsigned int);
extern unsigned char *(__cdecl *const ud__mbschr)(const unsigned char *, unsigned int) __asm__(UD_IMP(_mbschr));
extern unsigned char *(__cdecl *const ud__mbschr_l)(const unsigned char *, unsigned int, _locale_t) __asm__(UD_IMP(_mbschr_l));
extern unsigned char *(__cdecl *const ud__mbsrchr)(const unsigned char *, unsigned int) __asm__(UD_IMP(_mbsrchr));

static void run_mbstring_mbschr(void)
{
    size_t i;
    const unsigned int needles[] = { 'a', 0, 0x8140 };
    size_t j;
    for (i = 0; i < G_MBSTR_N; i++) {
        for (j = 0; j < sizeof(needles) / sizeof(needles[0]); j++) {
            const char *lbl = label_idx(i * 10 + j);
            grade2("_mbschr", lbl, off((const char *)g_mbstr[i], (const char *)ud__mbschr(g_mbstr[i], needles[j])),
                   off((const char *)g_mbstr[i], (const char *)ref__mbschr(g_mbstr[i], needles[j])));
            grade2("_mbschr_l", lbl, off((const char *)g_mbstr[i], (const char *)ud__mbschr_l(g_mbstr[i], needles[j], 0)),
                   off((const char *)g_mbstr[i], (const char *)ref__mbschr_l(g_mbstr[i], needles[j], 0)));
            grade2("_mbsrchr", lbl, off((const char *)g_mbstr[i], (const char *)ud__mbsrchr(g_mbstr[i], needles[j])),
                   off((const char *)g_mbstr[i], (const char *)ref__mbsrchr(g_mbstr[i], needles[j])));
        }
    }
    grade2("_mbschr.null", "-", *_errno(), 0); /* placeholder that costs nothing if unused */
}
#else
static void run_mbstring_mbschr(void) {}
#endif

/* ================================ mbscmp / mbsicmp / mbscoll / mbsicoll == */
#if UCRTDIFF_REF_HAS__mbscmp
extern int __cdecl ref__mbscmp(const unsigned char *, const unsigned char *);
extern int __cdecl ref__mbsicmp(const unsigned char *, const unsigned char *);
extern int __cdecl ref__mbscoll(const unsigned char *, const unsigned char *);
extern int __cdecl ref__mbsicoll(const unsigned char *, const unsigned char *);
extern int(__cdecl *const ud__mbscmp)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbscmp));
extern int(__cdecl *const ud__mbsicmp)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsicmp));
extern int(__cdecl *const ud__mbscoll)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbscoll));
extern int(__cdecl *const ud__mbsicoll)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsicoll));

static void run_mbstring_cmp_family(const unsigned char *const *corpus, size_t n, const char *tag)
{
    size_t i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            char lbl[24];
            int k = 0;
            lbl[k++] = 'i'; lbl[k++] = '='; lbl[k++] = (char)('0' + (int)(i % 10));
            lbl[k++] = ','; lbl[k++] = 'j'; lbl[k++] = '='; lbl[k++] = (char)('0' + (int)(j % 10)); lbl[k] = 0;
            grade2(tag ? "_mbscmp.sjis" : "_mbscmp", lbl, sgn(ud__mbscmp(corpus[i], corpus[j])), sgn(ref__mbscmp(corpus[i], corpus[j])));
            grade2(tag ? "_mbsicmp.sjis" : "_mbsicmp", lbl, sgn(ud__mbsicmp(corpus[i], corpus[j])), sgn(ref__mbsicmp(corpus[i], corpus[j])));
            grade2(tag ? "_mbscoll.sjis" : "_mbscoll", lbl, sgn(ud__mbscoll(corpus[i], corpus[j])), sgn(ref__mbscoll(corpus[i], corpus[j])));
            grade2(tag ? "_mbsicoll.sjis" : "_mbsicoll", lbl, sgn(ud__mbsicoll(corpus[i], corpus[j])), sgn(ref__mbsicoll(corpus[i], corpus[j])));
        }
    }
}
#else
static void run_mbstring_cmp_family(const unsigned char *const *corpus, size_t n, const char *tag) { (void)corpus; (void)n; (void)tag; }
#endif

/* ================================ mbsnbcmp / mbsnbicmp / mbsnbcoll / mbsnbicoll
 * / mbsncmp / mbsnicmp / mbsncoll / mbsnicoll -- byte- and char-counted ======= */
#if UCRTDIFF_REF_HAS__mbsnbcmp
extern int __cdecl ref__mbsnbcmp(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsnbicmp(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsnbcoll(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsnbicoll(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsncmp(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsnicmp(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsncoll(const unsigned char *, const unsigned char *, size_t);
extern int __cdecl ref__mbsnicoll(const unsigned char *, const unsigned char *, size_t);
extern int(__cdecl *const ud__mbsnbcmp)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcmp));
extern int(__cdecl *const ud__mbsnbicmp)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbicmp));
extern int(__cdecl *const ud__mbsnbcoll)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcoll));
extern int(__cdecl *const ud__mbsnbicoll)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbicoll));
extern int(__cdecl *const ud__mbsncmp)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncmp));
extern int(__cdecl *const ud__mbsnicmp)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnicmp));
extern int(__cdecl *const ud__mbsncoll)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncoll));
extern int(__cdecl *const ud__mbsnicoll)(const unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnicoll));

static void run_mbstring_ncmp_family(const unsigned char *const *corpus, size_t n)
{
    static const size_t counts[] = { 0, 1, 2, 3, 100 };
    size_t i, j, c;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[32];
                int k = 0;
                lbl[k++] = 'i'; lbl[k++] = (char)('0' + (int)(i % 10));
                lbl[k++] = 'j'; lbl[k++] = (char)('0' + (int)(j % 10));
                lbl[k++] = 'n'; lbl[k++] = (char)('0' + (int)(counts[c] % 10)); lbl[k] = 0;
                grade2("_mbsnbcmp", lbl, sgn(ud__mbsnbcmp(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnbcmp(corpus[i], corpus[j], counts[c])));
                grade2("_mbsnbicmp", lbl, sgn(ud__mbsnbicmp(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnbicmp(corpus[i], corpus[j], counts[c])));
                grade2("_mbsnbcoll", lbl, sgn(ud__mbsnbcoll(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnbcoll(corpus[i], corpus[j], counts[c])));
                grade2("_mbsnbicoll", lbl, sgn(ud__mbsnbicoll(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnbicoll(corpus[i], corpus[j], counts[c])));
                grade2("_mbsncmp", lbl, sgn(ud__mbsncmp(corpus[i], corpus[j], counts[c])), sgn(ref__mbsncmp(corpus[i], corpus[j], counts[c])));
                grade2("_mbsnicmp", lbl, sgn(ud__mbsnicmp(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnicmp(corpus[i], corpus[j], counts[c])));
                grade2("_mbsncoll", lbl, sgn(ud__mbsncoll(corpus[i], corpus[j], counts[c])), sgn(ref__mbsncoll(corpus[i], corpus[j], counts[c])));
                grade2("_mbsnicoll", lbl, sgn(ud__mbsnicoll(corpus[i], corpus[j], counts[c])), sgn(ref__mbsnicoll(corpus[i], corpus[j], counts[c])));
            }
        }
    }
}
#else
static void run_mbstring_ncmp_family(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbscspn / mbspbrk / mbsspn / mbsspnp === */
#if UCRTDIFF_REF_HAS__mbscspn
extern size_t __cdecl ref__mbscspn(const unsigned char *, const unsigned char *);
extern unsigned char *__cdecl ref__mbspbrk(const unsigned char *, const unsigned char *);
extern size_t __cdecl ref__mbsspn(const unsigned char *, const unsigned char *);
extern unsigned char *__cdecl ref__mbsspnp(const unsigned char *, const unsigned char *);
extern size_t(__cdecl *const ud__mbscspn)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbscspn));
extern unsigned char *(__cdecl *const ud__mbspbrk)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbspbrk));
extern size_t(__cdecl *const ud__mbsspn)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsspn));
extern unsigned char *(__cdecl *const ud__mbsspnp)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsspnp));

static void run_mbstring_spn_family(const unsigned char *const *corpus, size_t n)
{
    static const unsigned char *const sets[] = { (const unsigned char *)"abc", (const unsigned char *)"",
                                                  (const unsigned char *)"\x82\xa0" };
    size_t i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < sizeof(sets) / sizeof(sets[0]); j++) {
            char lbl[16];
            lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10)); lbl[2] = 's'; lbl[3] = (char)('0' + (int)j); lbl[4] = 0;
            grade2("_mbscspn", lbl, (long long)ud__mbscspn(corpus[i], sets[j]), (long long)ref__mbscspn(corpus[i], sets[j]));
            grade2("_mbspbrk", lbl, off((const char *)corpus[i], (const char *)ud__mbspbrk(corpus[i], sets[j])),
                   off((const char *)corpus[i], (const char *)ref__mbspbrk(corpus[i], sets[j])));
            grade2("_mbsspn", lbl, (long long)ud__mbsspn(corpus[i], sets[j]), (long long)ref__mbsspn(corpus[i], sets[j]));
            grade2("_mbsspnp", lbl, off((const char *)corpus[i], (const char *)ud__mbsspnp(corpus[i], sets[j])),
                   off((const char *)corpus[i], (const char *)ref__mbsspnp(corpus[i], sets[j])));
        }
    }
}
#else
static void run_mbstring_spn_family(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbsstr ================================= */
#if UCRTDIFF_REF_HAS__mbsstr
extern unsigned char *__cdecl ref__mbsstr(const unsigned char *, const unsigned char *);
extern unsigned char *(__cdecl *const ud__mbsstr)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsstr));

static void run_mbstring_mbsstr(const unsigned char *const *corpus, size_t n)
{
    static const unsigned char *const needles[] = { (const unsigned char *)"a", (const unsigned char *)"",
                                                      (const unsigned char *)"\x82\xa0" };
    size_t i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < sizeof(needles) / sizeof(needles[0]); j++) {
            char lbl[16];
            lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10)); lbl[2] = 'n'; lbl[3] = (char)('0' + (int)j); lbl[4] = 0;
            grade2("_mbsstr", lbl, off((const char *)corpus[i], (const char *)ud__mbsstr(corpus[i], needles[j])),
                   off((const char *)corpus[i], (const char *)ref__mbsstr(corpus[i], needles[j])));
        }
    }
}
#else
static void run_mbstring_mbsstr(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbsdec / mbsinc / mbsninc / mbsnextc ==== */
#if UCRTDIFF_REF_HAS__mbsinc
extern unsigned char *__cdecl ref__mbsinc(const unsigned char *);
extern unsigned char *__cdecl ref__mbsninc(const unsigned char *, size_t);
extern unsigned char *__cdecl ref__mbsdec(const unsigned char *, const unsigned char *);
extern unsigned int __cdecl ref__mbsnextc(const unsigned char *);
extern unsigned char *(__cdecl *const ud__mbsinc)(const unsigned char *) __asm__(UD_IMP(_mbsinc));
extern unsigned char *(__cdecl *const ud__mbsninc)(const unsigned char *, size_t) __asm__(UD_IMP(_mbsninc));
extern unsigned char *(__cdecl *const ud__mbsdec)(const unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbsdec));
extern unsigned int(__cdecl *const ud__mbsnextc)(const unsigned char *) __asm__(UD_IMP(_mbsnextc));

static void run_mbstring_nav(const unsigned char *const *corpus, size_t n)
{
    static const size_t ns[] = { 0, 1, 2, 5 };
    size_t i, k;
    for (i = 0; i < n; i++) {
        const char *lbl = label_idx(i);
        grade2("_mbsinc", lbl, off((const char *)corpus[i], (const char *)ud__mbsinc(corpus[i])),
               off((const char *)corpus[i], (const char *)ref__mbsinc(corpus[i])));
        grade2("_mbsnextc", lbl, (long long)ud__mbsnextc(corpus[i]), (long long)ref__mbsnextc(corpus[i]));
        for (k = 0; k < sizeof(ns) / sizeof(ns[0]); k++) {
            char lbl2[16];
            lbl2[0] = 'i'; lbl2[1] = (char)('0' + (int)(i % 10)); lbl2[2] = 'n'; lbl2[3] = (char)('0' + (int)ns[k]); lbl2[4] = 0;
            grade2("_mbsninc", lbl2, off((const char *)corpus[i], (const char *)ud__mbsninc(corpus[i], ns[k])),
                   off((const char *)corpus[i], (const char *)ref__mbsninc(corpus[i], ns[k])));
        }
        /* _mbsdec walks BACKWARD from a position inside the string: use the
         * end (the NUL) and, when long enough, one byte before it too. */
        {
            size_t len = 0;
            while (corpus[i][len]) len++;
            grade2("_mbsdec.end", lbl, off((const char *)corpus[i], (const char *)ud__mbsdec(corpus[i], corpus[i] + len)),
                   off((const char *)corpus[i], (const char *)ref__mbsdec(corpus[i], corpus[i] + len)));
            if (len > 0) {
                grade2("_mbsdec.mid", lbl, off((const char *)corpus[i], (const char *)ud__mbsdec(corpus[i], corpus[i] + len - 1)),
                       off((const char *)corpus[i], (const char *)ref__mbsdec(corpus[i], corpus[i] + len - 1)));
            }
        }
    }
}
#else
static void run_mbstring_nav(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbslen / mbsnlen / mbsnbcnt / mbsnccnt = */
#if UCRTDIFF_REF_HAS__mbslen
extern size_t __cdecl ref__mbslen(const unsigned char *);
extern size_t __cdecl ref__mbsnlen(const unsigned char *, size_t);
extern size_t __cdecl ref__mbsnbcnt(const unsigned char *, size_t);
extern size_t __cdecl ref__mbsnccnt(const unsigned char *, size_t);
extern size_t(__cdecl *const ud__mbslen)(const unsigned char *) __asm__(UD_IMP(_mbslen));
extern size_t(__cdecl *const ud__mbsnlen)(const unsigned char *, size_t) __asm__(UD_IMP(_mbsnlen));
extern size_t(__cdecl *const ud__mbsnbcnt)(const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcnt));
extern size_t(__cdecl *const ud__mbsnccnt)(const unsigned char *, size_t) __asm__(UD_IMP(_mbsnccnt));

static void run_mbstring_len(const unsigned char *const *corpus, size_t n)
{
    static const size_t ns[] = { 0, 1, 2, 3, 10 };
    size_t i, k;
    for (i = 0; i < n; i++) {
        const char *lbl = label_idx(i);
        grade2("_mbslen", lbl, (long long)ud__mbslen(corpus[i]), (long long)ref__mbslen(corpus[i]));
        for (k = 0; k < sizeof(ns) / sizeof(ns[0]); k++) {
            char lbl2[16];
            lbl2[0] = 'i'; lbl2[1] = (char)('0' + (int)(i % 10)); lbl2[2] = 'n'; lbl2[3] = (char)('0' + (int)ns[k]); lbl2[4] = 0;
            grade2("_mbsnlen", lbl2, (long long)ud__mbsnlen(corpus[i], ns[k]), (long long)ref__mbsnlen(corpus[i], ns[k]));
            grade2("_mbsnbcnt", lbl2, (long long)ud__mbsnbcnt(corpus[i], ns[k]), (long long)ref__mbsnbcnt(corpus[i], ns[k]));
            grade2("_mbsnccnt", lbl2, (long long)ud__mbsnccnt(corpus[i], ns[k]), (long long)ref__mbsnccnt(corpus[i], ns[k]));
        }
    }
}
#else
static void run_mbstring_len(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbslwr / mbsupr / tombbmbc ============= */
#if UCRTDIFF_REF_HAS__mbslwr
extern unsigned char *__cdecl ref__mbslwr(unsigned char *);
extern unsigned char *__cdecl ref__mbsupr(unsigned char *);
extern unsigned int __cdecl ref__mbbtombc(unsigned int);
extern unsigned int __cdecl ref__mbctombb(unsigned int);
extern unsigned char *(__cdecl *const ud__mbslwr)(unsigned char *) __asm__(UD_IMP(_mbslwr));
extern unsigned char *(__cdecl *const ud__mbsupr)(unsigned char *) __asm__(UD_IMP(_mbsupr));
extern unsigned int(__cdecl *const ud__mbbtombc)(unsigned int) __asm__(UD_IMP(_mbbtombc));
extern unsigned int(__cdecl *const ud__mbctombb)(unsigned int) __asm__(UD_IMP(_mbctombb));

static void run_mbstring_case(const unsigned char *const *corpus, size_t n)
{
    size_t i;
    for (i = 0; i < n; i++) {
        unsigned char b1[32], b2[32];
        /* The whole 32 bytes are hashed: the tail must be the same sentinel on both
         * sides, not two different stacks' garbage. */
        memset(b1, 0x55, sizeof b1); memset(b2, 0x55, sizeof b2);
        size_t len = 0;
        const char *lbl = label_idx(i);
        while (corpus[i][len] && len < 30) { b1[len] = b2[len] = corpus[i][len]; len++; }
        b1[len] = b2[len] = 0;
        {
            unsigned char *r1 = ud__mbslwr(b1);
            unsigned char *r2 = ref__mbslwr(b2);
            grade2("_mbslwr.buf", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));
            grade2("_mbslwr.ret", lbl, (long long)(r1 != 0), (long long)(r2 != 0));
        }
        memset(b1, 0x55, sizeof b1); memset(b2, 0x55, sizeof b2);
        len = 0;
        while (corpus[i][len] && len < 30) { b1[len] = b2[len] = corpus[i][len]; len++; }
        b1[len] = b2[len] = 0;
        {
            unsigned char *r1 = ud__mbsupr(b1);
            unsigned char *r2 = ref__mbsupr(b2);
            grade2("_mbsupr.buf", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));
            grade2("_mbsupr.ret", lbl, (long long)(r1 != 0), (long long)(r2 != 0));
        }
    }
    {
        unsigned int cs[] = { 0x20, 'A', 0xdc, 0xB1, 0x8341, 0 };
        size_t j;
        for (j = 0; j < sizeof(cs) / sizeof(cs[0]); j++) {
            char lbl[8]; lbl[0] = 'c'; lbl[1] = (char)('0' + (int)j); lbl[2] = 0;
            grade2("_mbbtombc", lbl, (long long)ud__mbbtombc(cs[j]), (long long)ref__mbbtombc(cs[j]));
            grade2("_mbctombb", lbl, (long long)ud__mbctombb(cs[j]), (long long)ref__mbctombb(cs[j]));
        }
    }
}
#else
static void run_mbstring_case(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbsncpy / mbsnbcpy / mbsncat / mbsnbcat = */
#if UCRTDIFF_REF_HAS__mbsncpy
extern unsigned char *__cdecl ref__mbsncpy(unsigned char *, const unsigned char *, size_t);
extern unsigned char *__cdecl ref__mbsnbcpy(unsigned char *, const unsigned char *, size_t);
extern unsigned char *__cdecl ref__mbsncat(unsigned char *, const unsigned char *, size_t);
extern unsigned char *__cdecl ref__mbsnbcat(unsigned char *, const unsigned char *, size_t);
extern unsigned char *(__cdecl *const ud__mbsncpy)(unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncpy));
extern unsigned char *(__cdecl *const ud__mbsnbcpy)(unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcpy));
extern unsigned char *(__cdecl *const ud__mbsncat)(unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncat));
extern unsigned char *(__cdecl *const ud__mbsnbcat)(unsigned char *, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcat));

static void run_mbstring_ncpy_family(const unsigned char *const *corpus, size_t n)
{
    static const size_t ns[] = { 0, 1, 2, 3, 8 };
    size_t i, k;
    for (i = 0; i < n; i++) {
        for (k = 0; k < sizeof(ns) / sizeof(ns[0]); k++) {
            char lbl[16];
            unsigned char d1[MBS_BUF], d2[MBS_BUF];
            lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10)); lbl[2] = 'n'; lbl[3] = (char)('0' + (int)ns[k]); lbl[4] = 0;

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            ud__mbsncpy(d1, corpus[i], ns[k]); ref__mbsncpy(d2, corpus[i], ns[k]);
            grade2("_mbsncpy.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            ud__mbsnbcpy(d1, corpus[i], ns[k]); ref__mbsnbcpy(d2, corpus[i], ns[k]);
            grade2("_mbsnbcpy.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            /* cat onto a fixed "ab" prefix, sentinel-filled beyond it */
            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0;
            ud__mbsncat(d1, corpus[i], ns[k]); ref__mbsncat(d2, corpus[i], ns[k]);
            grade2("_mbsncat.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0;
            ud__mbsnbcat(d1, corpus[i], ns[k]); ref__mbsnbcat(d2, corpus[i], ns[k]);
            grade2("_mbsnbcat.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
    }
}
#else
static void run_mbstring_ncpy_family(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbsnset / mbsnbset / mbsset ============ */
#if UCRTDIFF_REF_HAS__mbsnset
extern unsigned char *__cdecl ref__mbsnset(unsigned char *, unsigned int, size_t);
extern unsigned char *__cdecl ref__mbsnbset(unsigned char *, unsigned int, size_t);
extern unsigned char *__cdecl ref__mbsset(unsigned char *, unsigned int);
extern unsigned char *(__cdecl *const ud__mbsnset)(unsigned char *, unsigned int, size_t) __asm__(UD_IMP(_mbsnset));
extern unsigned char *(__cdecl *const ud__mbsnbset)(unsigned char *, unsigned int, size_t) __asm__(UD_IMP(_mbsnbset));
extern unsigned char *(__cdecl *const ud__mbsset)(unsigned char *, unsigned int) __asm__(UD_IMP(_mbsset));

static void run_mbstring_set_family(void)
{
    static const unsigned int vals[] = { 'x', 0x8140 /* valid DBCS lead+trail */, 0x8100 /* lead + NUL low byte: invalid */ };
    static const size_t ns[] = { 0, 1, 2, 3, 8 };
    size_t v, k;
    for (v = 0; v < sizeof(vals) / sizeof(vals[0]); v++) {
        for (k = 0; k < sizeof(ns) / sizeof(ns[0]); k++) {
            char lbl[16];
            unsigned char d1[MBS_BUF], d2[MBS_BUF];
            lbl[0] = 'v'; lbl[1] = (char)('0' + (int)v); lbl[2] = 'n'; lbl[3] = (char)('0' + (int)ns[k]); lbl[4] = 0;

            /* seed both buffers with "abcdef" (6 real chars) + NUL */
            {
                const char *seed = "abcdef";
                size_t s;
                for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
                d1[6] = d2[6] = 0;
                for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
            }
            ud__mbsnset(d1, vals[v], ns[k]); ref__mbsnset(d2, vals[v], ns[k]);
            grade2("_mbsnset.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2("_mbsnset.errno", lbl, *_errno(), *ref__errno());

            {
                const char *seed = "abcdef";
                size_t s;
                for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
                d1[6] = d2[6] = 0;
                for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
            }
            ud__mbsnbset(d1, vals[v], ns[k]); ref__mbsnbset(d2, vals[v], ns[k]);
            grade2("_mbsnbset.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2("_mbsnbset.errno", lbl, *_errno(), *ref__errno());
        }
        {
            char lbl[8]; lbl[0] = 'v'; lbl[1] = (char)('0' + (int)v); lbl[2] = 0;
            unsigned char d1[MBS_BUF], d2[MBS_BUF];
            const char *seed = "abcdef";
            size_t s;
            for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
            d1[6] = d2[6] = 0;
            for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
            ud__mbsset(d1, vals[v]); ref__mbsset(d2, vals[v]);
            grade2("_mbsset.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2("_mbsset.errno", lbl, *_errno(), *ref__errno());
        }
    }
}
#else
static void run_mbstring_set_family(void) {}
#endif

/* ================================ mbstok / mbstok_s ======================= */
#if UCRTDIFF_REF_HAS__mbstok
extern unsigned char *__cdecl ref__mbstok(unsigned char *, const unsigned char *);
extern unsigned char *__cdecl ref__mbstok_s(unsigned char *, const unsigned char *, unsigned char **);
extern unsigned char *(__cdecl *const ud__mbstok)(unsigned char *, const unsigned char *) __asm__(UD_IMP(_mbstok));
extern unsigned char *(__cdecl *const ud__mbstok_s)(unsigned char *, const unsigned char *, unsigned char **) __asm__(UD_IMP(_mbstok_s));

static void run_mbstring_tok(void)
{
    unsigned char b1[32], b2[32];
    const char *seed = "one two,three";
    unsigned char *ctx1 = 0, *ctx2 = 0;
    unsigned char *t1, *t2;
    size_t s;
    for (s = 0; seed[s]; s++) { b1[s] = b2[s] = (unsigned char)seed[s]; }
    b1[s] = b2[s] = 0;

    t1 = ud__mbstok_s(b1, (const unsigned char *)" ,", &ctx1);
    t2 = ref__mbstok_s(b2, (const unsigned char *)" ,", &ctx2);
    grade2("_mbstok_s.tok1", "-", off((const char *)b1, (const char *)t1), off((const char *)b2, (const char *)t2));
    t1 = ud__mbstok_s(0, (const unsigned char *)" ,", &ctx1);
    t2 = ref__mbstok_s(0, (const unsigned char *)" ,", &ctx2);
    grade2("_mbstok_s.tok2", "-", off((const char *)b1, (const char *)t1), off((const char *)b2, (const char *)t2));
    t1 = ud__mbstok_s(0, (const unsigned char *)" ,", &ctx1);
    t2 = ref__mbstok_s(0, (const unsigned char *)" ,", &ctx2);
    grade2("_mbstok_s.tok3", "-", off((const char *)b1, (const char *)t1), off((const char *)b2, (const char *)t2));
    grade2("_mbstok_s.buf", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));

    for (s = 0; seed[s]; s++) { b1[s] = b2[s] = (unsigned char)seed[s]; }
    b1[s] = b2[s] = 0;
    t1 = ud__mbstok(b1, (const unsigned char *)" ,");
    t2 = ref__mbstok(b2, (const unsigned char *)" ,");
    grade2("_mbstok.tok1", "-", off((const char *)b1, (const char *)t1), off((const char *)b2, (const char *)t2));
    t1 = ud__mbstok(0, (const unsigned char *)" ,");
    t2 = ref__mbstok(0, (const unsigned char *)" ,");
    grade2("_mbstok.tok2", "-", off((const char *)b1, (const char *)t1), off((const char *)b2, (const char *)t2));
}
#else
static void run_mbstring_tok(void) {}
#endif

/* ================================ mbccpy_s / mbscat_s / mbscpy_s / mbsset_s */
#if UCRTDIFF_REF_HAS__mbccpy_s
extern int __cdecl ref__mbccpy_s(unsigned char *, size_t, int *, const unsigned char *);
extern int __cdecl ref__mbscat_s(unsigned char *, size_t, const unsigned char *);
extern int __cdecl ref__mbscpy_s(unsigned char *, size_t, const unsigned char *);
extern int __cdecl ref__mbsset_s(unsigned char *, size_t, unsigned int);
extern int(__cdecl *const ud__mbccpy_s)(unsigned char *, size_t, int *, const unsigned char *) __asm__(UD_IMP(_mbccpy_s));
extern int(__cdecl *const ud__mbscat_s)(unsigned char *, size_t, const unsigned char *) __asm__(UD_IMP(_mbscat_s));
extern int(__cdecl *const ud__mbscpy_s)(unsigned char *, size_t, const unsigned char *) __asm__(UD_IMP(_mbscpy_s));
extern int(__cdecl *const ud__mbsset_s)(unsigned char *, size_t, unsigned int) __asm__(UD_IMP(_mbsset_s));

static void run_mbstring_secure_simple(const unsigned char *const *corpus, size_t n)
{
    static const size_t sizes[] = { 0, 1, 2, 3, 8, MBS_BUF };
    size_t i, sz;
    for (i = 0; i < n; i++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            char lbl[16];
            unsigned char d1[MBS_BUF], d2[MBS_BUF];
            int copied1 = -9, copied2 = -9;
            int r1, r2;
            lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10)); lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10)); lbl[4] = 0;

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            r1 = ud__mbccpy_s(d1, sizes[sz], &copied1, corpus[i]);
            r2 = ref__mbccpy_s(d2, sizes[sz], &copied2, corpus[i]);
            grade2("_mbccpy_s.ret", lbl, r1, r2);
            grade2("_mbccpy_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2("_mbccpy_s.copied", lbl, copied1, copied2);

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            if (sizes[sz] > 0) { d1[0] = d2[0] = 0; }
            r1 = ud__mbscpy_s(d1, sizes[sz], corpus[i]);
            r2 = ref__mbscpy_s(d2, sizes[sz], corpus[i]);
            grade2("_mbscpy_s.ret", lbl, r1, r2);
            grade2("_mbscpy_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
            if (sizes[sz] >= 3) { d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0; }
            else if (sizes[sz] > 0) { d1[0] = d2[0] = 0; }
            r1 = ud__mbscat_s(d1, sizes[sz], corpus[i]);
            r2 = ref__mbscat_s(d2, sizes[sz], corpus[i]);
            grade2("_mbscat_s.ret", lbl, r1, r2);
            grade2("_mbscat_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
    }
    {
        static const unsigned int vals[] = { 'x', 0x8140, 0x8100 };
        size_t v;
        for (v = 0; v < sizeof(vals) / sizeof(vals[0]); v++) {
            char lbl[8]; lbl[0] = 'v'; lbl[1] = (char)('0' + (int)v); lbl[2] = 0;
            unsigned char d1[MBS_BUF], d2[MBS_BUF];
            int r1, r2;
            const char *seed = "abcdef";
            size_t s;
            for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
            d1[6] = d2[6] = 0;
            for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
            r1 = ud__mbsset_s(d1, MBS_BUF, vals[v]);
            r2 = ref__mbsset_s(d2, MBS_BUF, vals[v]);
            grade2("_mbsset_s.ret", lbl, r1, r2);
            grade2("_mbsset_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
    }
}
#else
static void run_mbstring_secure_simple(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ================================ mbsncat_s / mbsnbcat_s / mbsncpy_s /
 * mbsnbcpy_s / mbsnset_s / mbsnbset_s -- the shared-template `_s` family.
 * ONE RUNNER PER C FILE, each independently guarded (files may be deleted
 * separately) -- NOT one runner per shape as earlier families in this
 * snippet use, per the coordinator's explicit ask.
 *
 * Contract points every runner below hits, per the coordinator's ask:
 * count==0, a fixed too-small count/size that forces BUFFER_TOO_SMALL,
 * `_TRUNCATE` (cat/cpy only -- `_mbsnset_s`/`_mbsnbset_s` have no such path,
 * see their own doc in secure.rs), and -- under codepage 932, via
 * the codepage-932 pass (`run_mbstring_cp932`) -- a truncation that lands exactly inside
 * a DBCS pair and a source ending in a lone lead byte.
 */
/* ---- _mbsncat_s ---- */
#if UCRTDIFF_REF_HAS__mbsncat_s
extern int __cdecl ref__mbsncat_s(unsigned char *, size_t, const unsigned char *, size_t);
extern int(__cdecl *const ud__mbsncat_s)(unsigned char *, size_t, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncat_s));

static void run_mbstring_mbsncat_s(const unsigned char *const *corpus, size_t n)
{
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, MBS_TRUNCATE };
    size_t i, sz, c;
    for (i = 0; i < n; i++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10));
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)(counts[c] == MBS_TRUNCATE ? 'T' : (char)('0' + (int)(counts[c] % 10)));
                lbl[6] = 0;

                memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
                if (sizes[sz] >= 3) { d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0; }
                else if (sizes[sz] > 0) { d1[0] = d2[0] = 0; }
                r1 = ud__mbsncat_s(d1, sizes[sz], corpus[i], counts[c]);
                r2 = ref__mbsncat_s(d2, sizes[sz], corpus[i], counts[c]);
                grade2("_mbsncat_s.ret", lbl, r1, r2);
                grade2("_mbsncat_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsncat_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsncat_s(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ---- _mbsnbcat_s ---- */
#if UCRTDIFF_REF_HAS__mbsnbcat_s
extern int __cdecl ref__mbsnbcat_s(unsigned char *, size_t, const unsigned char *, size_t);
extern int(__cdecl *const ud__mbsnbcat_s)(unsigned char *, size_t, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcat_s));

static void run_mbstring_mbsnbcat_s(const unsigned char *const *corpus, size_t n)
{
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, MBS_TRUNCATE };
    size_t i, sz, c;
    for (i = 0; i < n; i++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10));
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)(counts[c] == MBS_TRUNCATE ? 'T' : (char)('0' + (int)(counts[c] % 10)));
                lbl[6] = 0;

                memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
                if (sizes[sz] >= 3) { d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0; }
                else if (sizes[sz] > 0) { d1[0] = d2[0] = 0; }
                r1 = ud__mbsnbcat_s(d1, sizes[sz], corpus[i], counts[c]);
                r2 = ref__mbsnbcat_s(d2, sizes[sz], corpus[i], counts[c]);
                grade2("_mbsnbcat_s.ret", lbl, r1, r2);
                grade2("_mbsnbcat_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsnbcat_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsnbcat_s(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ---- _mbsncpy_s ---- */
#if UCRTDIFF_REF_HAS__mbsncpy_s
extern int __cdecl ref__mbsncpy_s(unsigned char *, size_t, const unsigned char *, size_t);
extern int(__cdecl *const ud__mbsncpy_s)(unsigned char *, size_t, const unsigned char *, size_t) __asm__(UD_IMP(_mbsncpy_s));

static void run_mbstring_mbsncpy_s(const unsigned char *const *corpus, size_t n)
{
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, MBS_TRUNCATE };
    size_t i, sz, c;
    for (i = 0; i < n; i++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10));
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)(counts[c] == MBS_TRUNCATE ? 'T' : (char)('0' + (int)(counts[c] % 10)));
                lbl[6] = 0;

                memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
                r1 = ud__mbsncpy_s(d1, sizes[sz], corpus[i], counts[c]);
                r2 = ref__mbsncpy_s(d2, sizes[sz], corpus[i], counts[c]);
                grade2("_mbsncpy_s.ret", lbl, r1, r2);
                grade2("_mbsncpy_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsncpy_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsncpy_s(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ---- _mbsnbcpy_s ---- */
#if UCRTDIFF_REF_HAS__mbsnbcpy_s
extern int __cdecl ref__mbsnbcpy_s(unsigned char *, size_t, const unsigned char *, size_t);
extern int(__cdecl *const ud__mbsnbcpy_s)(unsigned char *, size_t, const unsigned char *, size_t) __asm__(UD_IMP(_mbsnbcpy_s));

static void run_mbstring_mbsnbcpy_s(const unsigned char *const *corpus, size_t n)
{
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, MBS_TRUNCATE };
    size_t i, sz, c;
    for (i = 0; i < n; i++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                lbl[0] = 'i'; lbl[1] = (char)('0' + (int)(i % 10));
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)(counts[c] == MBS_TRUNCATE ? 'T' : (char)('0' + (int)(counts[c] % 10)));
                lbl[6] = 0;

                memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
                r1 = ud__mbsnbcpy_s(d1, sizes[sz], corpus[i], counts[c]);
                r2 = ref__mbsnbcpy_s(d2, sizes[sz], corpus[i], counts[c]);
                grade2("_mbsnbcpy_s.ret", lbl, r1, r2);
                grade2("_mbsnbcpy_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsnbcpy_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsnbcpy_s(const unsigned char *const *corpus, size_t n) { (void)corpus; (void)n; }
#endif

/* ---- _mbsnset_s (no _TRUNCATE path -- see secure.rs's nset_s_common doc) ---- */
#if UCRTDIFF_REF_HAS__mbsnset_s
extern int __cdecl ref__mbsnset_s(unsigned char *, size_t, unsigned int, size_t);
extern int(__cdecl *const ud__mbsnset_s)(unsigned char *, size_t, unsigned int, size_t) __asm__(UD_IMP(_mbsnset_s));

static void run_mbstring_mbsnset_s(void)
{
    static const unsigned int vals[] = { 'x', 0x8140, 0x8100 };
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, 8 };
    size_t v, sz, c;
    for (v = 0; v < sizeof(vals) / sizeof(vals[0]); v++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                const char *seed = "abcdef";
                size_t s;
                lbl[0] = 'v'; lbl[1] = (char)('0' + (int)v);
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)('0' + (int)(counts[c] % 10)); lbl[6] = 0;

                for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
                d1[6] = d2[6] = 0;
                for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
                r1 = ud__mbsnset_s(d1, sizes[sz], vals[v], counts[c]);
                r2 = ref__mbsnset_s(d2, sizes[sz], vals[v], counts[c]);
                grade2("_mbsnset_s.ret", lbl, r1, r2);
                grade2("_mbsnset_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsnset_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsnset_s(void) {}
#endif

/* ---- _mbsnbset_s ---- */
#if UCRTDIFF_REF_HAS__mbsnbset_s
extern int __cdecl ref__mbsnbset_s(unsigned char *, size_t, unsigned int, size_t);
extern int(__cdecl *const ud__mbsnbset_s)(unsigned char *, size_t, unsigned int, size_t) __asm__(UD_IMP(_mbsnbset_s));

static void run_mbstring_mbsnbset_s(void)
{
    static const unsigned int vals[] = { 'x', 0x8140, 0x8100 };
    static const size_t sizes[] = { 0, 1, 2, 3, 4, 8, MBS_BUF };
    static const size_t counts[] = { 0, 1, 2, 3, 8 };
    size_t v, sz, c;
    for (v = 0; v < sizeof(vals) / sizeof(vals[0]); v++) {
        for (sz = 0; sz < sizeof(sizes) / sizeof(sizes[0]); sz++) {
            for (c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
                char lbl[24];
                unsigned char d1[MBS_BUF], d2[MBS_BUF];
                int r1, r2;
                const char *seed = "abcdef";
                size_t s;
                lbl[0] = 'v'; lbl[1] = (char)('0' + (int)v);
                lbl[2] = 's'; lbl[3] = (char)('0' + (int)(sizes[sz] % 10));
                lbl[4] = 'c'; lbl[5] = (char)('0' + (int)(counts[c] % 10)); lbl[6] = 0;

                for (s = 0; s < 6; s++) d1[s] = d2[s] = (unsigned char)seed[s];
                d1[6] = d2[6] = 0;
                for (s = 7; s < MBS_BUF; s++) { d1[s] = d2[s] = 0xEE; }
                r1 = ud__mbsnbset_s(d1, sizes[sz], vals[v], counts[c]);
                r2 = ref__mbsnbset_s(d2, sizes[sz], vals[v], counts[c]);
                grade2("_mbsnbset_s.ret", lbl, r1, r2);
                grade2("_mbsnbset_s.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_mbsnbset_s.errno", lbl, *_errno(), *ref__errno());
            }
        }
    }
}
#else
static void run_mbstring_mbsnbset_s(void) {}
#endif

/* ---- edge cases shared across the cat/cpy pair: _TRUNCATE landing exactly
 * inside a DBCS pair, and a source ending in a lone lead byte. Guarded on
 * BOTH mbsncat_s and mbsncpy_s since it exercises both. */
#if UCRTDIFF_REF_HAS__mbsncat_s && UCRTDIFF_REF_HAS__mbsncpy_s
static void run_mbstring_secure_ntemplate_edge_cases(void)
{
    static const unsigned char *const pair_src = (const unsigned char *)"\x82\xa0\x82\xa0"; /* two DBCS pairs */
    static const unsigned char *const lone_lead_src = (const unsigned char *)"a\x82";          /* SBCS + lone lead */
    size_t sz;
    /* Truncation landing exactly mid-pair: size 3 leaves room for only
     * "\x82\xa0\x82" before the NUL, splitting the second pair. */
    for (sz = 2; sz <= 5; sz++) {
        char lbl[8]; lbl[0] = 's'; lbl[1] = (char)('0' + (int)sz); lbl[2] = 0;
        unsigned char d1[MBS_BUF], d2[MBS_BUF];
        int r1, r2;

        memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
        if (sz > 0) { d1[0] = d2[0] = 0; }
        r1 = ud__mbsncpy_s(d1, sz, pair_src, MBS_TRUNCATE);
        r2 = ref__mbsncpy_s(d2, sz, pair_src, MBS_TRUNCATE);
        grade2("_mbsncpy_s.trunc_pair.ret", lbl, r1, r2);
        grade2("_mbsncpy_s.trunc_pair.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));

        memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
        if (sz >= 3) { d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0; }
        else if (sz > 0) { d1[0] = d2[0] = 0; }
        r1 = ud__mbsncat_s(d1, sz, pair_src, MBS_TRUNCATE);
        r2 = ref__mbsncat_s(d2, sz, pair_src, MBS_TRUNCATE);
        grade2("_mbsncat_s.trunc_pair.ret", lbl, r1, r2);
        grade2("_mbsncat_s.trunc_pair.buf", lbl, fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
    /* Source ending in a lone lead byte, char-counted, count big enough to
     * reach it: exercises the "invalid MBC in source" branch directly. */
    {
        unsigned char d1[MBS_BUF], d2[MBS_BUF];
        int r1, r2;
        memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
        d1[0] = d2[0] = 0;
        r1 = ud__mbsncpy_s(d1, MBS_BUF, lone_lead_src, 5);
        r2 = ref__mbsncpy_s(d2, MBS_BUF, lone_lead_src, 5);
        grade2("_mbsncpy_s.lone_lead.ret", "-", r1, r2);
        grade2("_mbsncpy_s.lone_lead.buf", "-", fnv(d1, sizeof d1), fnv(d2, sizeof d2));

        memset(d1, 0xEE, sizeof d1); memset(d2, 0xEE, sizeof d2);
        d1[0] = d2[0] = 'a'; d1[1] = d2[1] = 'b'; d1[2] = d2[2] = 0;
        r1 = ud__mbsncat_s(d1, MBS_BUF, lone_lead_src, 5);
        r2 = ref__mbsncat_s(d2, MBS_BUF, lone_lead_src, 5);
        grade2("_mbsncat_s.lone_lead.ret", "-", r1, r2);
        grade2("_mbsncat_s.lone_lead.buf", "-", fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
}
#else
static void run_mbstring_secure_ntemplate_edge_cases(void) {}
#endif

static void run_mbstring_ntemplates_default_cp(void)
{
    install_counting_iph();
    run_mbstring_mbsncat_s(g_mbstr, G_MBSTR_N);
    run_mbstring_mbsnbcat_s(g_mbstr, G_MBSTR_N);
    run_mbstring_mbsncpy_s(g_mbstr, G_MBSTR_N);
    run_mbstring_mbsnbcpy_s(g_mbstr, G_MBSTR_N);
    run_mbstring_mbsnset_s();
    run_mbstring_mbsnbset_s();
    run_mbstring_secure_ntemplate_edge_cases();
}


/* ================================ default-codepage pass ==================
 * Runs every corpus-driven family against `g_mbstr` at whatever codepage is
 * already installed (1252 by default) -- this is what actually exercises
 * `_mbscmp`/`_mbsnbcmp`/`_mbscspn`/`_mbsstr`/`_mbsinc`/`_mbslen`/`_mbslwr`/
   `_mbsncpy`/`_mbccpy_s` and friends; `run_mbstring_cp932` re-runs the SAME
 * family functions over the Shift-JIS corpus once codepage 932 is installed.
 */
static void run_mbstring_default_cp(void)
{
    install_counting_iph();
    run_mbstring_cmp_family(g_mbstr, G_MBSTR_N, 0);
    run_mbstring_ncmp_family(g_mbstr, G_MBSTR_N);
    run_mbstring_spn_family(g_mbstr, G_MBSTR_N);
    run_mbstring_mbsstr(g_mbstr, G_MBSTR_N);
    run_mbstring_nav(g_mbstr, G_MBSTR_N);
    run_mbstring_len(g_mbstr, G_MBSTR_N);
    run_mbstring_case(g_mbstr, G_MBSTR_N);
    run_mbstring_ncpy_family(g_mbstr, G_MBSTR_N);
    run_mbstring_secure_simple(g_mbstr, G_MBSTR_N);
}

/* ================================ codepage-932 pass ======================= */
#if UCRTDIFF_REF_HAS__mbschr && UCRTDIFF_REF_CALLABLE__setmbcp
/* `ref__getmbcp`/`ref__setmbcp` are declared unconditionally near the top of
 * ucrtdiff.c already (for the harness's own preamble use); only the DLL-side
 * import pointers are ours to add. Matching an identical extern redeclaration
 * elsewhere in the TU is legal C, not a clash -- these are the ones nothing
 * else declares unconditionally. */
extern int(__cdecl *const ud__getmbcp)(void) __asm__(UD_IMP(_getmbcp));
extern int(__cdecl *const ud__setmbcp)(int) __asm__(UD_IMP(_setmbcp));

static void run_mbstring_cp932(void)
{
    int orig1 = ud__getmbcp(), orig2 = ref__getmbcp();
    int r1 = ud__setmbcp(932), r2 = ref__setmbcp(932);
    grade2("_setmbcp(932)", "mbstring", r1, r2);
    if (r1 == 0 && r2 == 0) {
        install_counting_iph();
        run_mbstring_cmp_family(g_sjis, G_SJIS_N, "sjis");
        run_mbstring_ncmp_family(g_sjis, G_SJIS_N);
        run_mbstring_spn_family(g_sjis, G_SJIS_N);
        run_mbstring_mbsstr(g_sjis, G_SJIS_N);
        run_mbstring_nav(g_sjis, G_SJIS_N);
        run_mbstring_len(g_sjis, G_SJIS_N);
        run_mbstring_case(g_sjis, G_SJIS_N);
        run_mbstring_ncpy_family(g_sjis, G_SJIS_N);
        run_mbstring_secure_simple(g_sjis, G_SJIS_N);
        run_mbstring_mbsncat_s(g_sjis, G_SJIS_N);
        run_mbstring_mbsnbcat_s(g_sjis, G_SJIS_N);
        run_mbstring_mbsncpy_s(g_sjis, G_SJIS_N);
        run_mbstring_mbsnbcpy_s(g_sjis, G_SJIS_N);
        run_mbstring_mbsnset_s();
        run_mbstring_mbsnbset_s();
        run_mbstring_secure_ntemplate_edge_cases(); /* pair_src/lone_lead_src are ALREADY raw SJIS bytes; genuinely DBCS only here, under 932 */
    } else {
        /* Codepage 932 did not load in this guest: nothing above was
         * exercised in DBCS mode. Recorded via the `_setmbcp(932)` row
         * above rather than silently skipped. */
    }
    ud__setmbcp(orig1);
    ref__setmbcp(orig2);
}
#else
static void run_mbstring_cp932(void) {}
#endif

/* ==== NULL-argument contracts across the family =============================
 * The validation paths the corpus runners never reach. `_VALIDATE_RETURN` is errno
 * THEN the invalid-parameter handler; the port first shipped 17 of them without the
 * handler, which only a NULL argument shows. Graded: return, errno, handler count. */
#if UCRTDIFF_REF_HAS__mbscmp
#define MBN_DECL(ret, name, args) \
    ret __cdecl ref_##name args; \
    extern ret(__cdecl *const udn_##name) args __asm__(UD_IMP(name));
MBN_DECL(int, _mbscmp, (const unsigned char *, const unsigned char *))
MBN_DECL(int, _mbsicmp, (const unsigned char *, const unsigned char *))
MBN_DECL(int, _mbscoll, (const unsigned char *, const unsigned char *))
MBN_DECL(int, _mbsicoll, (const unsigned char *, const unsigned char *))
MBN_DECL(int, _mbsncmp, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnbcmp, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnicmp, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnbicmp, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsncoll, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnbcoll, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnicoll, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(int, _mbsnbicoll, (const unsigned char *, const unsigned char *, size_t))
MBN_DECL(unsigned char *, _mbsncpy, (unsigned char *, const unsigned char *, size_t))
MBN_DECL(unsigned char *, _mbsnbcpy, (unsigned char *, const unsigned char *, size_t))
MBN_DECL(unsigned char *, _mbsncat, (unsigned char *, const unsigned char *, size_t))
MBN_DECL(unsigned char *, _mbsnbcat, (unsigned char *, const unsigned char *, size_t))
MBN_DECL(unsigned char *, _mbsnset, (unsigned char *, unsigned int, size_t))
MBN_DECL(unsigned char *, _mbsnbset, (unsigned char *, unsigned int, size_t))
MBN_DECL(unsigned char *, _mbsset, (unsigned char *, unsigned int))
MBN_DECL(errno_t, _mbslwr_s, (unsigned char *, size_t))
MBN_DECL(errno_t, _mbsupr_s, (unsigned char *, size_t))
extern int(__cdecl *const udn__getmbcp)(void) __asm__(UD_IMP(_getmbcp));
extern int(__cdecl *const udn__setmbcp)(int) __asm__(UD_IMP(_setmbcp));

#define MBN_ROW(name, lbl, ...) do {                                           \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        long long r1 = (long long)(intptr_t)udn_##name(__VA_ARGS__);           \
        long long r2 = (long long)(intptr_t)ref_##name(__VA_ARGS__);           \
        grade2(#name ".null", lbl, r1, r2);                                    \
        grade2(#name ".null.errno", lbl, *_errno(), *ref__errno());            \
        grade2(#name ".null.iph", lbl, g_iph_dll, g_iph_ref);                  \
    } while (0)

static void run_mbstring_null_contracts(void)
{
    static const unsigned char s[] = "abc";
    unsigned char b1[8], b2[8];
#if UCRTDIFF_REF_CALLABLE__setmbcp
    /* Under a single-byte code page most of these hand off to strncmp & co. BEFORE
     * validating -- Microsoft's order, kept -- so NULL is undefined behaviour on both
     * sides there. The contracts are the multibyte path's. */
    int cp1 = udn__getmbcp(), cp2 = ref__getmbcp();
    if (udn__setmbcp(932) != 0 || ref__setmbcp(932) != 0) {
        udn__setmbcp(cp1); ref__setmbcp(cp2);
        return;
    }
#else
    return;
#endif
    MBN_ROW(_mbscmp, "s1", 0, s);      MBN_ROW(_mbscmp, "s2", s, 0);
    MBN_ROW(_mbsicmp, "s1", 0, s);     MBN_ROW(_mbscoll, "s2", s, 0);
    MBN_ROW(_mbsicoll, "s1", 0, s);
    MBN_ROW(_mbsncmp, "s1", 0, s, 2);  MBN_ROW(_mbsncmp, "n0", 0, s, 0);
    MBN_ROW(_mbsnbcmp, "s2", s, 0, 2); MBN_ROW(_mbsnicmp, "s1", 0, s, 2);
    MBN_ROW(_mbsnbicmp, "s1", 0, s, 2);
    MBN_ROW(_mbsncoll, "s1", 0, s, 2); MBN_ROW(_mbsncoll, "big", s, s, (size_t)0x80000000u);
    MBN_ROW(_mbsnbcoll, "s2", s, 0, 2); MBN_ROW(_mbsnicoll, "s1", 0, s, 2);
    MBN_ROW(_mbsnbicoll, "big", s, s, (size_t)0x80000000u);
    MBN_ROW(_mbsncpy, "dst", 0, s, 1);  MBN_ROW(_mbsncpy, "src", b1, 0, 1);
    MBN_ROW(_mbsnbcpy, "dst", 0, s, 1);
    MBN_ROW(_mbsncat, "dst", 0, s, 1);  MBN_ROW(_mbsnbcat, "src", b1, 0, 1);
    MBN_ROW(_mbsnset, "str", 0, 'x', 1); MBN_ROW(_mbsnbset, "str", 0, 'x', 1);
    MBN_ROW(_mbsset, "str", 0, 'x');
    MBN_ROW(_mbslwr_s, "null+size", 0, 1); MBN_ROW(_mbsupr_s, "buf+0", b1, 0);
    /* Not NUL-terminated within the size: _RETURN_DEST_NOT_NULL_TERMINATED. */
    memset(b1, 'A', sizeof b1); memset(b2, 'A', sizeof b2);
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    {
        errno_t r1 = udn__mbslwr_s(b1, sizeof b1), r2 = ref__mbslwr_s(b2, sizeof b2);
        grade2("_mbslwr_s.unterminated", "-", r1, r2);
        grade2("_mbslwr_s.unterminated.buf", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));
        grade2("_mbslwr_s.unterminated.errno", "-", *_errno(), *ref__errno());
        grade2("_mbslwr_s.unterminated.iph", "-", g_iph_dll, g_iph_ref);
    }
#if UCRTDIFF_REF_CALLABLE__setmbcp
    udn__setmbcp(cp1); ref__setmbcp(cp2);
#endif
}
#define MBSTRING_NULL_ENTRIES { "mbstring:null_contracts", run_mbstring_null_contracts },
#else
#define MBSTRING_NULL_ENTRIES
#endif

/* ================================ registry ================================ */
#define MBSTRING_HAND_ENTRIES \
    { "mbstring:mbclen+mbccpy", run_mbstring_mbclen }, \
    { "mbstring:mbschr+mbsrchr", run_mbstring_mbschr }, \
    { "mbstring:tok", run_mbstring_tok }, \
    { "mbstring:setfamily", run_mbstring_set_family }, \
    { "mbstring:defaultcp", run_mbstring_default_cp }, \
    { "mbstring:ntemplates_defaultcp", run_mbstring_ntemplates_default_cp }, \
    { "mbstring:cp932", run_mbstring_cp932 },


/* ==== staging/string_ops.c, inlined ==== */
/* string_ops.c -- guest grader for the `string-ops` batch (agent "string").
 *
 * #included into ucrtdiff.c just before run_legacy(), per the instructions
 * in the shared brief. Relies on ucrtdiff.c's own machinery already being in
 * scope at that point: grade2, fnv, label_idx, install_counting_iph,
 * loc_set_both/loc_reset, UD_IMP, ud_setlocale/ref_setlocale, _errno/
 * ref__errno, and the standard headers it already pulled in.
 *
 * Guard names used (one per C file it replaces; each is currently UNDEFINED
 * in generated-refs.h -- this batch's exports are not yet in
 * tools/gen_ucrt_tier2_cases.py's hand-graded list, so every guard evaluates
 * to 0 today. #if on an undefined macro is well-defined C (treated as 0), so
 * this snippet compiles and links cleanly either way; the lead adding these
 * names to that generator (and confirming `ref_<name>` in libucrt_ref.a,
 * verified already for all but the ones noted UNGRADABLE below) is what
 * turns each `{ "_foo", 0 }` row into a live grader):
 *   UCRTDIFF_REF_HAS__strdup            (strdup.cpp)
 *   UCRTDIFF_REF_HAS__wcsdup            (wcsdup.cpp)
 *   UCRTDIFF_REF_HAS___strncnt          (strncnt.cpp)
 *   UCRTDIFF_REF_HAS___wcsncnt          (wcsncnt.cpp)
 *   UCRTDIFF_REF_HAS_strnlen            (strnlen.cpp -- UNGRADABLE on arm64,
 *                                         see below; harness runs in an
 *                                         arm64 guest only)
 *   UCRTDIFF_REF_HAS_wcscat / _wcscmp / _wcscpy / _wcscspn / _wcsncat /
 *     _wcsncmp / _wcsncpy / _wcspbrk / __wcsrev / __wcsset / __wcsnset /
 *     _wcsspn                          (wcscat.cpp .. wcsspn.cpp, one macro
 *                                        stands in for the whole plain wcs*
 *                                        group; see run_string_ops_wcs below)
 *   UCRTDIFF_REF_HAS__memccpy, _strcmp, _strncmp, __strnset, __strrev,
 *     __strset                        (the arch-split family: memccpy.c/.s,
 *                                       strcmp.c/.s, strncmp.c/.s, strnset.c/.s,
 *                                       strrev.c/.s, strset.c/.s)
 *   UCRTDIFF_REF_HAS_strcat_s / _strcpy_s / _strncat_s / _strncpy_s /
 *     __strnset_s / __strset_s / _wcscat_s / _wcscpy_s / _wcsncat_s /
 *     _wcsncpy_s / __wcsnset_s / __wcsset_s / _memcpy_s / _wmemcpy_s /
 *     _wmemmove_s                     (the _s family, one file each)
 *   UCRTDIFF_REF_HAS_strtok / _strtok_s / _wcstok / _wcstok_s
 *                                     (strtok.cpp, strtok_s.cpp, wcstok.cpp,
 *                                      wcstok_s.cpp)
 *   UCRTDIFF_REF_HAS__memicmp / __stricmp / __strnicmp / __wcsicmp /
 *     __wcsnicmp                      (memicmp.cpp, stricmp.cpp, strnicmp.cpp,
 *                                      wcsicmp.cpp, wcsnicmp.cpp -- each
 *                                      guard's runner also grades that file's
 *                                      `_l` and `__ascii_*` exports)
 *
 * Extended for the case-mapping and collation follow-up batch (same file,
 * same rules): one guard per C file, one runner per C file (each covering
 * every export the file defines), graded under the "C" locale (pass 0) and
 * then a real non-C locale (`de-DE`, pass 1) set on BOTH sides via
 * `loc_set_both`/`loc_reset` (needs `UCRTDIFF_REF_CALLABLE_setlocale`; the
 * locale-swap pass is skipped, not miscompiled, when that guard is 0 --
 * every such block is wrapped `#if UCRTDIFF_REF_CALLABLE_setlocale ... #else
 * break; #endif`). New guards, all verified present as `ref_<name>` in
 * `build-arm64/libucrt_ref.a` via `llvm-nm`:
 *   UCRTDIFF_REF_HAS__strlwr / __strupr / __wcslwr / __wcsupr (strlwr.cpp,
 *     strupr.cpp, wcslwr.cpp, wcsupr.cpp -- each guard's runner also covers
 *     that file's `_l`, `_s` and `_s_l` exports)
 *   UCRTDIFF_REF_HAS_strcoll / __stricoll / __strncoll / __strnicoll /
 *     wcscoll / __wcsicoll / __wcsncoll / __wcsnicoll (strcoll.cpp,
 *     stricoll.cpp, strncoll.cpp, strnicoll.cpp, wcscoll.cpp, wcsicoll.cpp,
 *     wcsncoll.cpp, wcsnicoll.cpp -- each guard's runner also covers that
 *     file's `_l` export)
 *   UCRTDIFF_REF_HAS_strxfrm / wcsxfrm (strxfrm.cpp, wcsxfrm.cpp -- each
 *     guard's runner also covers that file's `_l` export, and explicitly
 *     grades the return-length-on-overflow contract with a too-small `count`)
 *
 * UNGRADABLE against the real Microsoft body on THIS (arm64) harness, noted
 * rather than silently skipped:
 *   - strnlen/wcsnlen: Microsoft's own strnlen.cpp defines NEITHER on arm64
 *     (`#if !defined(_M_ARM64)` around the definitions) -- confirmed via
 *     `aarch64-w64-mingw32-nm build-arm64/libucrt_ref.a | grep ref_strnlen`
 *     (no match). Their runners below still exist and still validate our own
 *     internal consistency (e.g. strnlen(s, n) <= n and agrees with strlen
 *     for n >= strlen(s)) via `grade2` against a value computed from a
 *     Win32-independent host `strlen`/`wcslen` call, but that is NOT a
 *     Microsoft-body oracle and must not be reported as one.
 *   - `__acrt_strtok_s_novalidation`/`__acrt_wcstok_s_novalidation` and the
 *     `__ascii_*` narrow-case-insensitive helpers: not found as `ref_*` in
 *     libucrt_ref.a for arm64 (verified via the same nm query) -- they are
 *     graded here only INDIRECTLY, through the public functions (`strtok`,
 *     `_stricmp` when the ascii fast path is live, etc.) that call them.
 */

#if UCRTDIFF_REF_HAS__strdup
char *__cdecl ref__strdup(const char *);
#endif
#if UCRTDIFF_REF_HAS__wcsdup
wchar_t *__cdecl ref__wcsdup(const wchar_t *);
#endif
#if UCRTDIFF_REF_HAS___strncnt
size_t __cdecl ref___strncnt(const char *, size_t);
#endif
#if UCRTDIFF_REF_HAS___wcsncnt
size_t __cdecl ref___wcsncnt(const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS_strnlen
size_t __cdecl ref_strnlen(const char *, size_t);
size_t __cdecl ref_wcsnlen(const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS_wcscat
wchar_t *__cdecl ref_wcscat(wchar_t *, const wchar_t *);
int __cdecl ref_wcscmp(const wchar_t *, const wchar_t *);
wchar_t *__cdecl ref_wcscpy(wchar_t *, const wchar_t *);
size_t __cdecl ref_wcscspn(const wchar_t *, const wchar_t *);
wchar_t *__cdecl ref_wcsncat(wchar_t *, const wchar_t *, size_t);
int __cdecl ref_wcsncmp(const wchar_t *, const wchar_t *, size_t);
wchar_t *__cdecl ref_wcsncpy(wchar_t *, const wchar_t *, size_t);
wchar_t *__cdecl ref_wcspbrk(const wchar_t *, const wchar_t *);
wchar_t *__cdecl ref__wcsrev(wchar_t *);
wchar_t *__cdecl ref__wcsset(wchar_t *, wchar_t);
wchar_t *__cdecl ref__wcsnset(wchar_t *, wchar_t, size_t);
size_t __cdecl ref_wcsspn(const wchar_t *, const wchar_t *);
#endif
#if UCRTDIFF_REF_HAS__memccpy
void *__cdecl ref__memccpy(void *, const void *, int, size_t);
#endif
#if UCRTDIFF_REF_HAS_strcmp
int __cdecl ref_strcmp(const char *, const char *);
#endif
#if UCRTDIFF_REF_HAS_strncmp
int __cdecl ref_strncmp(const char *, const char *, size_t);
#endif
#if UCRTDIFF_REF_HAS__strnset
char *__cdecl ref__strnset(char *, int, size_t);
#endif
#if UCRTDIFF_REF_HAS__strrev
char *__cdecl ref__strrev(char *);
#endif
#if UCRTDIFF_REF_HAS__strset
char *__cdecl ref__strset(char *, int);
#endif
#if UCRTDIFF_REF_HAS_strcat_s
errno_t __cdecl ref_strcat_s(char *, size_t, const char *);
#endif
#if UCRTDIFF_REF_HAS_strcpy_s
errno_t __cdecl ref_strcpy_s(char *, size_t, const char *);
#endif
#if UCRTDIFF_REF_HAS_strncat_s
errno_t __cdecl ref_strncat_s(char *, size_t, const char *, size_t);
#endif
#if UCRTDIFF_REF_HAS_strncpy_s
errno_t __cdecl ref_strncpy_s(char *, size_t, const char *, size_t);
#endif
#if UCRTDIFF_REF_HAS__strnset_s
errno_t __cdecl ref__strnset_s(char *, size_t, int, size_t);
#endif
#if UCRTDIFF_REF_HAS__strset_s
errno_t __cdecl ref__strset_s(char *, size_t, int);
#endif
#if UCRTDIFF_REF_HAS_wcscat_s
errno_t __cdecl ref_wcscat_s(wchar_t *, size_t, const wchar_t *);
#endif
#if UCRTDIFF_REF_HAS_wcscpy_s
errno_t __cdecl ref_wcscpy_s(wchar_t *, size_t, const wchar_t *);
#endif
#if UCRTDIFF_REF_HAS_wcsncat_s
errno_t __cdecl ref_wcsncat_s(wchar_t *, size_t, const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS_wcsncpy_s
errno_t __cdecl ref_wcsncpy_s(wchar_t *, size_t, const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS__wcsnset_s
errno_t __cdecl ref__wcsnset_s(wchar_t *, size_t, wchar_t, size_t);
#endif
#if UCRTDIFF_REF_HAS__wcsset_s
errno_t __cdecl ref__wcsset_s(wchar_t *, size_t, wchar_t);
#endif
#if UCRTDIFF_REF_HAS_memcpy_s
errno_t __cdecl ref_memcpy_s(void *, size_t, const void *, size_t);
#endif
#if UCRTDIFF_REF_HAS_wmemcpy_s
errno_t __cdecl ref_wmemcpy_s(wchar_t *, size_t, const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS_wmemmove_s
errno_t __cdecl ref_wmemmove_s(wchar_t *, size_t, const wchar_t *, size_t);
#endif
#if UCRTDIFF_REF_HAS_strtok
char *__cdecl ref_strtok(char *, const char *);
#endif
#if UCRTDIFF_REF_HAS_strtok_s
char *__cdecl ref_strtok_s(char *, const char *, char **);
#endif
#if UCRTDIFF_REF_HAS_wcstok
wchar_t *__cdecl ref_wcstok(wchar_t *, const wchar_t *, wchar_t **);
#endif
#if UCRTDIFF_REF_HAS_wcstok_s
wchar_t *__cdecl ref_wcstok_s(wchar_t *, const wchar_t *, wchar_t **);
#endif
#if UCRTDIFF_REF_HAS__memicmp
int __cdecl ref__memicmp(const void *, const void *, size_t);
int __cdecl ref__memicmp_l(const void *, const void *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__stricmp
int __cdecl ref__stricmp(const char *, const char *);
int __cdecl ref__stricmp_l(const char *, const char *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__strnicmp
int __cdecl ref__strnicmp(const char *, const char *, size_t);
int __cdecl ref__strnicmp_l(const char *, const char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsicmp
int __cdecl ref__wcsicmp(const wchar_t *, const wchar_t *);
int __cdecl ref__wcsicmp_l(const wchar_t *, const wchar_t *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsnicmp
int __cdecl ref__wcsnicmp(const wchar_t *, const wchar_t *, size_t);
int __cdecl ref__wcsnicmp_l(const wchar_t *, const wchar_t *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__strlwr
char *__cdecl ref__strlwr(char *);
char *__cdecl ref__strlwr_l(char *, _locale_t);
errno_t __cdecl ref__strlwr_s(char *, size_t);
errno_t __cdecl ref__strlwr_s_l(char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__strupr
char *__cdecl ref__strupr(char *);
char *__cdecl ref__strupr_l(char *, _locale_t);
errno_t __cdecl ref__strupr_s(char *, size_t);
errno_t __cdecl ref__strupr_s_l(char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcslwr
wchar_t *__cdecl ref__wcslwr(wchar_t *);
wchar_t *__cdecl ref__wcslwr_l(wchar_t *, _locale_t);
errno_t __cdecl ref__wcslwr_s(wchar_t *, size_t);
errno_t __cdecl ref__wcslwr_s_l(wchar_t *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsupr
wchar_t *__cdecl ref__wcsupr(wchar_t *);
wchar_t *__cdecl ref__wcsupr_l(wchar_t *, _locale_t);
errno_t __cdecl ref__wcsupr_s(wchar_t *, size_t);
errno_t __cdecl ref__wcsupr_s_l(wchar_t *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS_strcoll
int __cdecl ref_strcoll(const char *, const char *);
int __cdecl ref__strcoll_l(const char *, const char *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__stricoll
int __cdecl ref__stricoll(const char *, const char *);
int __cdecl ref__stricoll_l(const char *, const char *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__strncoll
int __cdecl ref__strncoll(const char *, const char *, size_t);
int __cdecl ref__strncoll_l(const char *, const char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__strnicoll
int __cdecl ref__strnicoll(const char *, const char *, size_t);
int __cdecl ref__strnicoll_l(const char *, const char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS_wcscoll
int __cdecl ref_wcscoll(const wchar_t *, const wchar_t *);
int __cdecl ref__wcscoll_l(const wchar_t *, const wchar_t *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsicoll
int __cdecl ref__wcsicoll(const wchar_t *, const wchar_t *);
int __cdecl ref__wcsicoll_l(const wchar_t *, const wchar_t *, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsncoll
int __cdecl ref__wcsncoll(const wchar_t *, const wchar_t *, size_t);
int __cdecl ref__wcsncoll_l(const wchar_t *, const wchar_t *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS__wcsnicoll
int __cdecl ref__wcsnicoll(const wchar_t *, const wchar_t *, size_t);
int __cdecl ref__wcsnicoll_l(const wchar_t *, const wchar_t *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS_strxfrm
size_t __cdecl ref_strxfrm(char *, const char *, size_t);
size_t __cdecl ref__strxfrm_l(char *, const char *, size_t, _locale_t);
#endif
#if UCRTDIFF_REF_HAS_wcsxfrm
size_t __cdecl ref_wcsxfrm(wchar_t *, const wchar_t *, size_t);
size_t __cdecl ref__wcsxfrm_l(wchar_t *, const wchar_t *, size_t, _locale_t);
#endif

/* ---- DLL-side import pointers (never called by plain name -- see UD_IMP's
 * comment further up in ucrtdiff.c: several of these have mingw inline
 * aliases that would otherwise grade the inline, not ucrtbase). */
extern char *(__cdecl *const ud__strdup)(const char *) __asm__(UD_IMP(_strdup));
extern wchar_t *(__cdecl *const ud__wcsdup)(const wchar_t *) __asm__(UD_IMP(_wcsdup));
extern size_t(__cdecl *const ud___strncnt)(const char *, size_t) __asm__(UD_IMP(__strncnt));
extern size_t(__cdecl *const ud___wcsncnt)(const wchar_t *, size_t) __asm__(UD_IMP(__wcsncnt));
extern size_t(__cdecl *const ud_strnlen)(const char *, size_t) __asm__(UD_IMP(strnlen));
extern size_t(__cdecl *const ud_wcsnlen)(const wchar_t *, size_t) __asm__(UD_IMP(wcsnlen));
extern wchar_t *(__cdecl *const ud_wcscat)(wchar_t *, const wchar_t *) __asm__(UD_IMP(wcscat));
extern int(__cdecl *const ud_wcscmp)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(wcscmp));
extern wchar_t *(__cdecl *const ud_wcscpy)(wchar_t *, const wchar_t *) __asm__(UD_IMP(wcscpy));
extern size_t(__cdecl *const ud_wcscspn)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(wcscspn));
extern wchar_t *(__cdecl *const ud_wcsncat)(wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(wcsncat));
extern int(__cdecl *const ud_wcsncmp)(const wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(wcsncmp));
extern wchar_t *(__cdecl *const ud_wcsncpy)(wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(wcsncpy));
extern wchar_t *(__cdecl *const ud_wcspbrk)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(wcspbrk));
extern wchar_t *(__cdecl *const ud__wcsrev)(wchar_t *) __asm__(UD_IMP(_wcsrev));
extern wchar_t *(__cdecl *const ud__wcsset)(wchar_t *, wchar_t) __asm__(UD_IMP(_wcsset));
extern wchar_t *(__cdecl *const ud__wcsnset)(wchar_t *, wchar_t, size_t) __asm__(UD_IMP(_wcsnset));
extern size_t(__cdecl *const ud_wcsspn)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(wcsspn));
extern void *(__cdecl *const ud__memccpy)(void *, const void *, int, size_t) __asm__(UD_IMP(_memccpy));
extern int(__cdecl *const ud_strcmp2)(const char *, const char *) __asm__(UD_IMP(strcmp));
extern int(__cdecl *const ud_strncmp2)(const char *, const char *, size_t) __asm__(UD_IMP(strncmp));
extern char *(__cdecl *const ud__strnset)(char *, int, size_t) __asm__(UD_IMP(_strnset));
extern char *(__cdecl *const ud__strrev)(char *) __asm__(UD_IMP(_strrev));
extern char *(__cdecl *const ud__strset)(char *, int) __asm__(UD_IMP(_strset));
extern errno_t(__cdecl *const ud_strcat_s)(char *, size_t, const char *) __asm__(UD_IMP(strcat_s));
extern errno_t(__cdecl *const ud_strcpy_s)(char *, size_t, const char *) __asm__(UD_IMP(strcpy_s));
extern errno_t(__cdecl *const ud_strncat_s)(char *, size_t, const char *, size_t) __asm__(UD_IMP(strncat_s));
extern errno_t(__cdecl *const ud_strncpy_s)(char *, size_t, const char *, size_t) __asm__(UD_IMP(strncpy_s));
extern errno_t(__cdecl *const ud__strnset_s)(char *, size_t, int, size_t) __asm__(UD_IMP(_strnset_s));
extern errno_t(__cdecl *const ud__strset_s)(char *, size_t, int) __asm__(UD_IMP(_strset_s));
extern errno_t(__cdecl *const ud_wcscat_s)(wchar_t *, size_t, const wchar_t *) __asm__(UD_IMP(wcscat_s));
extern errno_t(__cdecl *const ud_wcscpy_s)(wchar_t *, size_t, const wchar_t *) __asm__(UD_IMP(wcscpy_s));
extern errno_t(__cdecl *const ud_wcsncat_s)(wchar_t *, size_t, const wchar_t *, size_t) __asm__(UD_IMP(wcsncat_s));
extern errno_t(__cdecl *const ud_wcsncpy_s)(wchar_t *, size_t, const wchar_t *, size_t) __asm__(UD_IMP(wcsncpy_s));
extern errno_t(__cdecl *const ud__wcsnset_s)(wchar_t *, size_t, wchar_t, size_t) __asm__(UD_IMP(_wcsnset_s));
extern errno_t(__cdecl *const ud__wcsset_s)(wchar_t *, size_t, wchar_t) __asm__(UD_IMP(_wcsset_s));
extern errno_t(__cdecl *const ud_memcpy_s)(void *, size_t, const void *, size_t) __asm__(UD_IMP(memcpy_s));
extern errno_t(__cdecl *const ud_wmemcpy_s)(wchar_t *, size_t, const wchar_t *, size_t) __asm__(UD_IMP(wmemcpy_s));
extern errno_t(__cdecl *const ud_wmemmove_s)(wchar_t *, size_t, const wchar_t *, size_t) __asm__(UD_IMP(wmemmove_s));
extern char *(__cdecl *const ud_strtok2)(char *, const char *) __asm__(UD_IMP(strtok));
extern char *(__cdecl *const ud_strtok_s2)(char *, const char *, char **) __asm__(UD_IMP(strtok_s));
extern wchar_t *(__cdecl *const ud_wcstok2)(wchar_t *, const wchar_t *, wchar_t **) __asm__(UD_IMP(wcstok));
extern wchar_t *(__cdecl *const ud_wcstok_s2)(wchar_t *, const wchar_t *, wchar_t **) __asm__(UD_IMP(wcstok_s));
extern int(__cdecl *const ud__memicmp)(const void *, const void *, size_t) __asm__(UD_IMP(_memicmp));
extern int(__cdecl *const ud__memicmp_l)(const void *, const void *, size_t, _locale_t) __asm__(UD_IMP(_memicmp_l));
extern int(__cdecl *const ud__stricmp2)(const char *, const char *) __asm__(UD_IMP(_stricmp));
extern int(__cdecl *const ud__stricmp_l)(const char *, const char *, _locale_t) __asm__(UD_IMP(_stricmp_l));
extern int(__cdecl *const ud__strnicmp2)(const char *, const char *, size_t) __asm__(UD_IMP(_strnicmp));
extern int(__cdecl *const ud__strnicmp_l)(const char *, const char *, size_t, _locale_t) __asm__(UD_IMP(_strnicmp_l));
extern int(__cdecl *const ud__wcsicmp2)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wcsicmp));
extern int(__cdecl *const ud__wcsicmp_l)(const wchar_t *, const wchar_t *, _locale_t) __asm__(UD_IMP(_wcsicmp_l));
extern int(__cdecl *const ud__wcsnicmp2)(const wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(_wcsnicmp));
extern int(__cdecl *const ud__wcsnicmp_l)(const wchar_t *, const wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcsnicmp_l));
extern char *(__cdecl *const ud__strlwr)(char *) __asm__(UD_IMP(_strlwr));
extern char *(__cdecl *const ud__strlwr_l)(char *, _locale_t) __asm__(UD_IMP(_strlwr_l));
extern errno_t(__cdecl *const ud__strlwr_s)(char *, size_t) __asm__(UD_IMP(_strlwr_s));
extern errno_t(__cdecl *const ud__strlwr_s_l)(char *, size_t, _locale_t) __asm__(UD_IMP(_strlwr_s_l));
extern char *(__cdecl *const ud__strupr)(char *) __asm__(UD_IMP(_strupr));
extern char *(__cdecl *const ud__strupr_l)(char *, _locale_t) __asm__(UD_IMP(_strupr_l));
extern errno_t(__cdecl *const ud__strupr_s)(char *, size_t) __asm__(UD_IMP(_strupr_s));
extern errno_t(__cdecl *const ud__strupr_s_l)(char *, size_t, _locale_t) __asm__(UD_IMP(_strupr_s_l));
extern wchar_t *(__cdecl *const ud__wcslwr)(wchar_t *) __asm__(UD_IMP(_wcslwr));
extern wchar_t *(__cdecl *const ud__wcslwr_l)(wchar_t *, _locale_t) __asm__(UD_IMP(_wcslwr_l));
extern errno_t(__cdecl *const ud__wcslwr_s)(wchar_t *, size_t) __asm__(UD_IMP(_wcslwr_s));
extern errno_t(__cdecl *const ud__wcslwr_s_l)(wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcslwr_s_l));
extern wchar_t *(__cdecl *const ud__wcsupr)(wchar_t *) __asm__(UD_IMP(_wcsupr));
extern wchar_t *(__cdecl *const ud__wcsupr_l)(wchar_t *, _locale_t) __asm__(UD_IMP(_wcsupr_l));
extern errno_t(__cdecl *const ud__wcsupr_s)(wchar_t *, size_t) __asm__(UD_IMP(_wcsupr_s));
extern errno_t(__cdecl *const ud__wcsupr_s_l)(wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcsupr_s_l));
extern int(__cdecl *const ud_strcoll2)(const char *, const char *) __asm__(UD_IMP(strcoll));
extern int(__cdecl *const ud__strcoll_l)(const char *, const char *, _locale_t) __asm__(UD_IMP(_strcoll_l));
extern int(__cdecl *const ud__stricoll)(const char *, const char *) __asm__(UD_IMP(_stricoll));
extern int(__cdecl *const ud__stricoll_l)(const char *, const char *, _locale_t) __asm__(UD_IMP(_stricoll_l));
extern int(__cdecl *const ud__strncoll)(const char *, const char *, size_t) __asm__(UD_IMP(_strncoll));
extern int(__cdecl *const ud__strncoll_l)(const char *, const char *, size_t, _locale_t) __asm__(UD_IMP(_strncoll_l));
extern int(__cdecl *const ud__strnicoll)(const char *, const char *, size_t) __asm__(UD_IMP(_strnicoll));
extern int(__cdecl *const ud__strnicoll_l)(const char *, const char *, size_t, _locale_t) __asm__(UD_IMP(_strnicoll_l));
extern int(__cdecl *const ud_wcscoll2)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(wcscoll));
extern int(__cdecl *const ud__wcscoll_l)(const wchar_t *, const wchar_t *, _locale_t) __asm__(UD_IMP(_wcscoll_l));
extern int(__cdecl *const ud__wcsicoll)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wcsicoll));
extern int(__cdecl *const ud__wcsicoll_l)(const wchar_t *, const wchar_t *, _locale_t) __asm__(UD_IMP(_wcsicoll_l));
extern int(__cdecl *const ud__wcsncoll)(const wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(_wcsncoll));
extern int(__cdecl *const ud__wcsncoll_l)(const wchar_t *, const wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcsncoll_l));
extern int(__cdecl *const ud__wcsnicoll)(const wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(_wcsnicoll));
extern int(__cdecl *const ud__wcsnicoll_l)(const wchar_t *, const wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcsnicoll_l));
extern size_t(__cdecl *const ud_strxfrm2)(char *, const char *, size_t) __asm__(UD_IMP(strxfrm));
extern size_t(__cdecl *const ud__strxfrm_l)(char *, const char *, size_t, _locale_t) __asm__(UD_IMP(_strxfrm_l));
extern size_t(__cdecl *const ud_wcsxfrm2)(wchar_t *, const wchar_t *, size_t) __asm__(UD_IMP(wcsxfrm));
extern size_t(__cdecl *const ud__wcsxfrm_l)(wchar_t *, const wchar_t *, size_t, _locale_t) __asm__(UD_IMP(_wcsxfrm_l));

/* ---- corpora ------------------------------------------------------------ */
static const char *const so_narrow[] = {
    "", "a", "abc", "ABC", "AbC", "12", "\x80\xff\x81", " x ", "hello world",
};
#define SO_NARROW_N (sizeof(so_narrow) / sizeof(so_narrow[0]))
static const wchar_t *const so_wide[] = {
    L"", L"a", L"abc", L"ABC", L"AbC", L"12", L"\x00e9\xffff\x0080", L" x ", L"hello world",
};
#define SO_WIDE_N (sizeof(so_wide) / sizeof(so_wide[0]))

/* ---- strdup / wcsdup ----------------------------------------------------
 * Grade the copy by content hash (fnv), not the pointer (the two sides
 * allocate from different heaps) -- and grade NULL-in separately, since a
 * NULL is the one input both sides must answer identically without an
 * allocation at all. */
static void run_string_ops__strdup(void)
{
#if UCRTDIFF_REF_HAS__strdup
    for (unsigned i = 0; i < SO_NARROW_N; i++) {
        char *d = ud__strdup(so_narrow[i]);
        char *r = ref__strdup(so_narrow[i]);
        long long hd = d ? fnv(d, lstrlenA(d) + 1) : -1;
        long long hr = r ? fnv(r, lstrlenA(r) + 1) : -1;
        grade2("_strdup", label_idx(i), hd, hr);
        if (d) free(d);
        if (r) free(r);
    }
    {
        char *d = ud__strdup(0);
        char *r = ref__strdup(0);
        grade2("_strdup", "NULL", d ? 1 : 0, r ? 1 : 0);
    }
#endif
}

static void run_string_ops__wcsdup(void)
{
#if UCRTDIFF_REF_HAS__wcsdup
    for (unsigned i = 0; i < SO_WIDE_N; i++) {
        wchar_t *d = ud__wcsdup(so_wide[i]);
        wchar_t *r = ref__wcsdup(so_wide[i]);
        long long hd = d ? fnv(d, (wcslen(d) + 1) * sizeof(wchar_t)) : -1;
        long long hr = r ? fnv(r, (wcslen(r) + 1) * sizeof(wchar_t)) : -1;
        grade2("_wcsdup", label_idx(i), hd, hr);
        if (d) free(d);
        if (r) free(r);
    }
    {
        wchar_t *d = ud__wcsdup(0);
        wchar_t *r = ref__wcsdup(0);
        grade2("_wcsdup", "NULL", d ? 1 : 0, r ? 1 : 0);
    }
#endif
}

/* ---- __strncnt / __wcsncnt: count boundaries 0, exact len, beyond len --- */
static void run_string_ops___strncnt(void)
{
#if UCRTDIFF_REF_HAS___strncnt
    static const size_t counts[] = { 0, 1, 2, 3, 4, 5, 6, 100 };
    for (unsigned i = 0; i < SO_NARROW_N; i++)
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++)
            grade2("__strncnt", label_idx(i * 8 + c),
                   (long long)ud___strncnt(so_narrow[i], counts[c]),
                   (long long)ref___strncnt(so_narrow[i], counts[c]));
#endif
}

static void run_string_ops___wcsncnt(void)
{
#if UCRTDIFF_REF_HAS___wcsncnt
    static const size_t counts[] = { 0, 1, 2, 3, 4, 5, 6, 100 };
    for (unsigned i = 0; i < SO_WIDE_N; i++)
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++)
            grade2("__wcsncnt", label_idx(i * 8 + c),
                   (long long)ud___wcsncnt(so_wide[i], counts[c]),
                   (long long)ref___wcsncnt(so_wide[i], counts[c]));
#endif
}

/* ---- strnlen / wcsnlen: UNGRADABLE on this (arm64) harness (see file
 * header). Still exercised for internal boundary sanity: agrees with the
 * ordinary host strlen/wcslen whenever maximum_count exceeds the real
 * length, and never exceeds maximum_count otherwise. Reported under its own
 * "self" family name so it is never mistaken for a Microsoft-body MATCH. */
static void run_string_ops_strnlen(void)
{
    static const size_t counts[] = { 0, 1, 2, 3, 100 };
    for (unsigned i = 0; i < SO_NARROW_N; i++) {
        size_t real = lstrlenA(so_narrow[i]);
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
            size_t got = ud_strnlen(so_narrow[i], counts[c]);
            size_t want = real < counts[c] ? real : counts[c];
            grade2("strnlen.self", label_idx(i * 8 + c), (long long)got, (long long)want);
        }
#if UCRTDIFF_REF_HAS_strnlen
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++)
            grade2("strnlen", label_idx(i * 8 + c), (long long)ud_strnlen(so_narrow[i], counts[c]),
                   (long long)ref_strnlen(so_narrow[i], counts[c]));
#endif
    }
    for (unsigned i = 0; i < SO_WIDE_N; i++) {
        size_t real = wcslen(so_wide[i]);
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++) {
            size_t got = ud_wcsnlen(so_wide[i], counts[c]);
            size_t want = real < counts[c] ? real : counts[c];
            grade2("wcsnlen.self", label_idx(i * 8 + c), (long long)got, (long long)want);
        }
#if UCRTDIFF_REF_HAS_strnlen
        for (unsigned c = 0; c < sizeof(counts) / sizeof(counts[0]); c++)
            grade2("wcsnlen", label_idx(i * 8 + c), (long long)ud_wcsnlen(so_wide[i], counts[c]),
                   (long long)ref_wcsnlen(so_wide[i], counts[c]));
#endif
    }
}

/* ---- the plain wcs* family: one runner covers the whole group, since every
 * export takes the same "two short strings" shape and none is locale-aware. */
static void run_string_ops_wcs_plain(void)
{
    static const wchar_t *const a[] = { L"", L"abc", L"abc", L"ABCDE", L"hello", L"aaa" };
    static const wchar_t *const b[] = { L"", L"abc", L"abd", L"ABC", L"world", L"aa" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned i = 0; i < n; i++) {
#if UCRTDIFF_REF_HAS_wcscat
        grade2("wcscmp", label_idx(i), (long long)ud_wcscmp(a[i], b[i]), (long long)ref_wcscmp(a[i], b[i]));
        grade2("wcscspn", label_idx(i), (long long)ud_wcscspn(a[i], b[i]), (long long)ref_wcscspn(a[i], b[i]));
        grade2("wcspbrk.off", label_idx(i),
               (long long)(ud_wcspbrk(a[i], b[i]) ? ud_wcspbrk(a[i], b[i]) - a[i] : -1),
               (long long)(ref_wcspbrk(a[i], b[i]) ? ref_wcspbrk(a[i], b[i]) - a[i] : -1));
        grade2("wcsspn", label_idx(i), (long long)ud_wcsspn(a[i], b[i]), (long long)ref_wcsspn(a[i], b[i]));
        for (unsigned c = 0; c < 6; c++) {
            static const size_t counts[] = { 0, 1, 2, 5, 6, 100 };
            grade2("wcsncmp", label_idx(i * 6 + c), (long long)ud_wcsncmp(a[i], b[i], counts[c]),
                   (long long)ref_wcsncmp(a[i], b[i], counts[c]));
        }
        {
            wchar_t d1[64], d2[64];
            wcscpy(d1, a[i]); wcscpy(d2, a[i]);
            wchar_t *r1 = ud_wcscat(d1, b[i]);
            wchar_t *r2 = ref_wcscat(d2, b[i]);
            grade2("wcscat", label_idx(i), fnv(d1, (wcslen(d1) + 1) * 2), fnv(d2, (wcslen(d2) + 1) * 2));
            grade2("wcscat.ret", label_idx(i), (long long)(r1 - d1), (long long)(r2 - d2));
        }
        {
            wchar_t d1[64], d2[64];
            ud_wcscpy(d1, a[i]);
            ref_wcscpy(d2, a[i]);
            grade2("wcscpy", label_idx(i), fnv(d1, (wcslen(d1) + 1) * 2), fnv(d2, (wcslen(d2) + 1) * 2));
        }
        for (unsigned c = 0; c < 6; c++) {
            static const size_t counts[] = { 0, 1, 2, 5, 6, 100 };
            wchar_t d1[64], d2[64];
            wcscpy(d1, a[i]); wcscpy(d2, a[i]);
            ud_wcsncat(d1, b[i], counts[c]);
            ref_wcsncat(d2, b[i], counts[c]);
            /* whole-buffer hash: the tail past a short append is unspecified
             * by the contract but must still match byte for byte since both
             * sides read the same uninitialized-on-entry buffer identically. */
            grade2("wcsncat", label_idx(i * 6 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
        for (unsigned c = 0; c < 6; c++) {
            /* wcsncpy zero-pads up to count: never past d1. */
            static const size_t counts[] = { 0, 1, 2, 5, 6, 64 };
            wchar_t d1[64], d2[64];
            memset(d1, 0x55, sizeof d1);
            memset(d2, 0x55, sizeof d2);
            ud_wcsncpy(d1, a[i], counts[c]);
            ref_wcsncpy(d2, a[i], counts[c]);
            grade2("wcsncpy", label_idx(i * 6 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
#endif
    }
#if UCRTDIFF_REF_HAS_wcscat
    {
        /* _wcsrev / _wcsset / _wcsnset: mutate a private copy each time. */
        static const wchar_t *const s[] = { L"", L"a", L"ab", L"abc", L"abcde" };
        for (unsigned i = 0; i < sizeof(s) / sizeof(s[0]); i++) {
            wchar_t d1[16], d2[16];
            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            ud__wcsrev(d1);
            ref__wcsrev(d2);
            grade2("_wcsrev", label_idx(i), fnv(d1, (wcslen(d1) + 1) * 2), fnv(d2, (wcslen(d2) + 1) * 2));

            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            ud__wcsset(d1, L'Z');
            ref__wcsset(d2, L'Z');
            grade2("_wcsset", label_idx(i), fnv(d1, (wcslen(d1) + 1) * 2), fnv(d2, (wcslen(d2) + 1) * 2));

            for (unsigned c = 0; c < 4; c++) {
                static const size_t counts[] = { 0, 1, 3, 100 };
                wcscpy(d1, s[i]); wcscpy(d2, s[i]);
                ud__wcsnset(d1, L'Z', counts[c]);
                ref__wcsnset(d2, L'Z', counts[c]);
                grade2("_wcsnset", label_idx(i * 4 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
        }
    }
#endif
}

/* ---- the arch-split family: memccpy, strcmp, strncmp, _strnset, _strrev,
 * _strset. strncmp is graded by its EXACT return value (not sign): i386/amd64
 * clamp to {-1,0,1}, arm64's C fallback returns the raw byte difference --
 * this is the one place that divergence would show as a DIFF against the
 * genuine arm64 ref, which is the harness's own arch. */
static void run_string_ops_arch_split(void)
{
    static const char *const a[] = { "", "abc", "abc", "ABCDE", "hello", "aaa", "\x80\xff" };
    static const char *const b[] = { "", "abc", "abd", "ABC", "world", "aa", "\x81\xfe" };
    unsigned n = sizeof(a) / sizeof(a[0]);
#if UCRTDIFF_REF_HAS_strcmp
    for (unsigned i = 0; i < n; i++)
        grade2("strcmp", label_idx(i), (long long)ud_strcmp2(a[i], b[i]), (long long)ref_strcmp(a[i], b[i]));
#endif
#if UCRTDIFF_REF_HAS_strncmp
    for (unsigned i = 0; i < n; i++) {
        static const size_t counts[] = { 0, 1, 2, 5, 6, 100 };
        for (unsigned c = 0; c < 6; c++)
            grade2("strncmp", label_idx(i * 6 + c), (long long)ud_strncmp2(a[i], b[i], counts[c]),
                   (long long)ref_strncmp(a[i], b[i], counts[c]));
    }
#endif
#if UCRTDIFF_REF_HAS__memccpy
    {
        static const size_t counts[] = { 0, 1, 3, 4, 5, 32 };
        for (unsigned i = 0; i < n; i++)
            for (unsigned c = 0; c < 6; c++) {
                char d1[32], d2[32];
                memset(d1, 0x55, sizeof d1);
                memset(d2, 0x55, sizeof d2);
                /* _memccpy copies `count` bytes when `c` is absent, NUL or not: a
                 * padded source, and counts no bigger than it or the destination. */
                char src[32];
                memset(src, 0, sizeof src);
                strcpy(src, a[i]);
                void *r1 = ud__memccpy(d1, src, (int)'c', counts[c]);
                void *r2 = ref__memccpy(d2, src, (int)'c', counts[c]);
                grade2("_memccpy", label_idx(i * 6 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
                grade2("_memccpy.ret", label_idx(i * 6 + c),
                       (long long)(r1 ? (char *)r1 - d1 : -1), (long long)(r2 ? (char *)r2 - d2 : -1));
            }
        /* count == 0: NULL either way, no bytes touched. */
        {
            char d1[8] = { 1, 2, 3, 4, 5, 6, 7, 8 }, d2[8] = { 1, 2, 3, 4, 5, 6, 7, 8 };
            void *r1 = ud__memccpy(d1, "x", 'x', 0);
            void *r2 = ref__memccpy(d2, "x", 'x', 0);
            grade2("_memccpy.zero", "n=0", (long long)(r1 != 0), (long long)(r2 != 0));
            grade2("_memccpy.zero.buf", "n=0", fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
    }
#endif
#if UCRTDIFF_REF_HAS__strnset
    {
        static const char *const s[] = { "", "a", "ab", "abcde" };
        static const size_t counts[] = { 0, 1, 3, 100 };
        for (unsigned i = 0; i < sizeof(s) / sizeof(s[0]); i++)
            for (unsigned c = 0; c < 4; c++) {
                char d1[16], d2[16];
                strcpy(d1, s[i]); strcpy(d2, s[i]);
                ud__strnset(d1, 'Z', counts[c]);
                ref__strnset(d2, 'Z', counts[c]);
                grade2("_strnset", label_idx(i * 4 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
    }
#endif
#if UCRTDIFF_REF_HAS__strrev
    {
        static const char *const s[] = { "", "a", "ab", "abc", "abcde" };
        for (unsigned i = 0; i < sizeof(s) / sizeof(s[0]); i++) {
            char d1[16], d2[16];
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            ud__strrev(d1);
            ref__strrev(d2);
            grade2("_strrev", label_idx(i), fnv(d1, strlen(d1) + 1), fnv(d2, strlen(d2) + 1));
        }
    }
#endif
#if UCRTDIFF_REF_HAS__strset
    {
        static const char *const s[] = { "", "a", "ab", "abcde" };
        for (unsigned i = 0; i < sizeof(s) / sizeof(s[0]); i++) {
            char d1[16], d2[16];
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            ud__strset(d1, 'Z');
            ref__strset(d2, 'Z');
            grade2("_strset", label_idx(i), fnv(d1, strlen(d1) + 1), fnv(d2, strlen(d2) + 1));
        }
    }
#endif
}

/* ---- the _s family. Every runner covers: empty src, exact-fit dest,
 * too-small dest (buffer-too-small path), _TRUNCATE where the function takes
 * a count, and a too-small dest for the _n forms. All grade the WHOLE
 * destination buffer (prefilled with a sentinel) plus errno plus the return
 * code, since a silent wrong-fill is exactly the bug class this family
 * exists to catch. */
#define SO_SENTINEL 0x55

static void run_string_ops_strcat_s(void)
{
#if UCRTDIFF_REF_HAS_strcat_s
    static const struct { const char *dst; size_t sz; const char *src; } cs[] = {
        { "", 8, "abc" }, { "ab", 8, "cd" }, { "ab", 4, "cd" }, { "ab", 3, "cd" },
        { "ab", 3, "" }, { "", 1, "a" }, { "abcdefg", 8, "" },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        strcpy(d1, cs[i].dst); strcpy(d2, cs[i].dst);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_strcat_s(d1, cs[i].sz, cs[i].src);
        errno_t r2 = ref_strcat_s(d2, cs[i].sz, cs[i].src);
        grade2("strcat_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("strcat_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("strcat_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
    /* NULL src with a valid dest: EINVAL, dest reset to "". */
    {
        char d1[8] = "ab", d2[8] = "ab";
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_strcat_s(d1, sizeof d1, 0);
        errno_t r2 = ref_strcat_s(d2, sizeof d2, 0);
        grade2("strcat_s.nullsrc", "-", (long long)r1, (long long)r2);
        grade2("strcat_s.nullsrc.buf", "-", fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_strcpy_s(void)
{
#if UCRTDIFF_REF_HAS_strcpy_s
    static const struct { size_t sz; const char *src; } cs[] = {
        { 8, "" }, { 8, "abc" }, { 4, "abc" }, { 3, "abc" }, { 1, "" }, { 1, "a" },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_strcpy_s(d1, cs[i].sz, cs[i].src);
        errno_t r2 = ref_strcpy_s(d2, cs[i].sz, cs[i].src);
        grade2("strcpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("strcpy_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("strcpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_strncat_s(void)
{
#if UCRTDIFF_REF_HAS_strncat_s
    static const struct { const char *dst; size_t sz; const char *src; size_t n; } cs[] = {
        { "ab", 8, "cdef", 0 }, { "ab", 8, "cdef", 2 }, { "ab", 8, "cdef", 4 },
        { "ab", 8, "cdef", 100 }, { "ab", 6, "cdef", 100 }, { "ab", 5, "cdef", 100 },
        { "ab", 8, "cdef", (size_t)-1 } /* _TRUNCATE */, { "ab", 4, "cdef", (size_t)-1 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        strcpy(d1, cs[i].dst); strcpy(d2, cs[i].dst);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_strncat_s(d1, cs[i].sz, cs[i].src, cs[i].n);
        errno_t r2 = ref_strncat_s(d2, cs[i].sz, cs[i].src, cs[i].n);
        grade2("strncat_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("strncat_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("strncat_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_strncpy_s(void)
{
#if UCRTDIFF_REF_HAS_strncpy_s
    static const struct { size_t sz; const char *src; size_t n; } cs[] = {
        { 8, "abcdef", 0 }, { 8, "abcdef", 3 }, { 8, "abcdef", 6 }, { 8, "abcdef", 100 },
        { 4, "abcdef", 100 }, { 8, "abcdef", (size_t)-1 }, { 4, "abcdef", (size_t)-1 },
        { 8, "ab", 100 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_strncpy_s(d1, cs[i].sz, cs[i].src, cs[i].n);
        errno_t r2 = ref_strncpy_s(d2, cs[i].sz, cs[i].src, cs[i].n);
        grade2("strncpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("strncpy_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("strncpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops__strnset_s(void)
{
#if UCRTDIFF_REF_HAS__strnset_s
    static const struct { const char *s; size_t sz; size_t n; } cs[] = {
        { "abcde", 8, 0 }, { "abcde", 8, 2 }, { "abcde", 8, 100 }, { "abcde", 3, 100 }, { "", 4, 100 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        strcpy(d1, cs[i].s); strcpy(d2, cs[i].s);
        errno_t r1 = ud__strnset_s(d1, cs[i].sz, 'Z', cs[i].n);
        errno_t r2 = ref__strnset_s(d2, cs[i].sz, 'Z', cs[i].n);
        grade2("_strnset_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("_strnset_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops__strset_s(void)
{
#if UCRTDIFF_REF_HAS__strset_s
    static const struct { const char *s; size_t sz; } cs[] = {
        { "abcde", 8 }, { "abcde", 3 }, { "", 4 }, { "a", 1 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        strcpy(d1, cs[i].s); strcpy(d2, cs[i].s);
        errno_t r1 = ud__strset_s(d1, cs[i].sz, 'Z');
        errno_t r2 = ref__strset_s(d2, cs[i].sz, 'Z');
        grade2("_strset_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("_strset_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

/* Wide _s twins: same shapes, narrower corpus (the narrow runners already
 * exhaust the boundary space; these confirm the SAME template instantiated
 * over wchar_t behaves identically). */
static void run_string_ops_wcscat_s(void)
{
#if UCRTDIFF_REF_HAS_wcscat_s
    static const struct { const wchar_t *dst; size_t sz; const wchar_t *src; } cs[] = {
        { L"", 8, L"abc" }, { L"ab", 8, L"cd" }, { L"ab", 4, L"cd" }, { L"ab", 3, L"cd" },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        wcscpy(d1, cs[i].dst); wcscpy(d2, cs[i].dst);
        errno_t r1 = ud_wcscat_s(d1, cs[i].sz, cs[i].src);
        errno_t r2 = ref_wcscat_s(d2, cs[i].sz, cs[i].src);
        grade2("wcscat_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("wcscat_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_wcscpy_s(void)
{
#if UCRTDIFF_REF_HAS_wcscpy_s
    static const struct { size_t sz; const wchar_t *src; } cs[] = {
        { 8, L"" }, { 8, L"abc" }, { 4, L"abc" }, { 3, L"abc" }, { 1, L"" },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        errno_t r1 = ud_wcscpy_s(d1, cs[i].sz, cs[i].src);
        errno_t r2 = ref_wcscpy_s(d2, cs[i].sz, cs[i].src);
        grade2("wcscpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("wcscpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_wcsncat_s(void)
{
#if UCRTDIFF_REF_HAS_wcsncat_s
    static const struct { const wchar_t *dst; size_t sz; const wchar_t *src; size_t n; } cs[] = {
        { L"ab", 8, L"cdef", 2 }, { L"ab", 8, L"cdef", 100 }, { L"ab", 5, L"cdef", 100 },
        { L"ab", 8, L"cdef", (size_t)-1 }, { L"ab", 4, L"cdef", (size_t)-1 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        wcscpy(d1, cs[i].dst); wcscpy(d2, cs[i].dst);
        errno_t r1 = ud_wcsncat_s(d1, cs[i].sz, cs[i].src, cs[i].n);
        errno_t r2 = ref_wcsncat_s(d2, cs[i].sz, cs[i].src, cs[i].n);
        grade2("wcsncat_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("wcsncat_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_wcsncpy_s(void)
{
#if UCRTDIFF_REF_HAS_wcsncpy_s
    static const struct { size_t sz; const wchar_t *src; size_t n; } cs[] = {
        { 8, L"abcdef", 3 }, { 8, L"abcdef", 100 }, { 4, L"abcdef", 100 }, { 8, L"abcdef", (size_t)-1 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        errno_t r1 = ud_wcsncpy_s(d1, cs[i].sz, cs[i].src, cs[i].n);
        errno_t r2 = ref_wcsncpy_s(d2, cs[i].sz, cs[i].src, cs[i].n);
        grade2("wcsncpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("wcsncpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops__wcsnset_s(void)
{
#if UCRTDIFF_REF_HAS__wcsnset_s
    static const struct { const wchar_t *s; size_t sz; size_t n; } cs[] = {
        { L"abcde", 8, 2 }, { L"abcde", 8, 100 }, { L"abcde", 3, 100 },
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        wcscpy(d1, cs[i].s); wcscpy(d2, cs[i].s);
        errno_t r1 = ud__wcsnset_s(d1, cs[i].sz, L'Z', cs[i].n);
        errno_t r2 = ref__wcsnset_s(d2, cs[i].sz, L'Z', cs[i].n);
        grade2("_wcsnset_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("_wcsnset_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops__wcsset_s(void)
{
#if UCRTDIFF_REF_HAS__wcsset_s
    static const struct { const wchar_t *s; size_t sz; } cs[] = { { L"abcde", 8 }, { L"abcde", 3 } };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        wcscpy(d1, cs[i].s); wcscpy(d2, cs[i].s);
        errno_t r1 = ud__wcsset_s(d1, cs[i].sz, L'Z');
        errno_t r2 = ref__wcsset_s(d2, cs[i].sz, L'Z');
        grade2("_wcsset_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("_wcsset_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

/* ---- memcpy_s / wmemcpy_s / wmemmove_s ----------------------------------
 * count==0 (returns 0, no touch), NULL source (zero-fills, EINVAL),
 * dest-too-small (zero-fills, ERANGE), and a genuinely overlapping
 * wmemmove_s (the one case memcpy_s/wmemcpy_s must NOT be given). */
static void run_string_ops_memcpy_s(void)
{
#if UCRTDIFF_REF_HAS_memcpy_s
    static const char src[] = "abcdefgh";
    static const struct { size_t dsz; size_t n; } cs[] = {
        { 8, 0 }, { 8, 4 }, { 8, 8 }, { 4, 8 } /* dest too small */,
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        char d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_memcpy_s(d1, cs[i].dsz, src, cs[i].n);
        errno_t r2 = ref_memcpy_s(d2, cs[i].dsz, src, cs[i].n);
        grade2("memcpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("memcpy_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("memcpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
    /* NULL source, valid dest/size: EINVAL, whole dest zeroed. */
    {
        char d1[8], d2[8];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_memcpy_s(d1, sizeof d1, 0, 4);
        errno_t r2 = ref_memcpy_s(d2, sizeof d2, 0, 4);
        grade2("memcpy_s.nullsrc", "-", (long long)r1, (long long)r2);
        grade2("memcpy_s.nullsrc.buf", "-", fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_wmemcpy_s(void)
{
#if UCRTDIFF_REF_HAS_wmemcpy_s
    static const wchar_t src[] = L"abcdefgh";
    static const struct { size_t dsz; size_t n; } cs[] = {
        { 8, 0 }, { 8, 4 }, { 8, 8 }, { 4, 8 } /* dest too small */,
    };
    for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
        wchar_t d1[16], d2[16];
        memset(d1, SO_SENTINEL, sizeof d1);
        memset(d2, SO_SENTINEL, sizeof d2);
        *_errno() = 0; *ref__errno() = 0;
        errno_t r1 = ud_wmemcpy_s(d1, cs[i].dsz, src, cs[i].n);
        errno_t r2 = ref_wmemcpy_s(d2, cs[i].dsz, src, cs[i].n);
        grade2("wmemcpy_s", label_idx(i), (long long)r1, (long long)r2);
        grade2("wmemcpy_s.errno", label_idx(i), *_errno(), *ref__errno());
        grade2("wmemcpy_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
    }
#endif
}

static void run_string_ops_wmemmove_s(void)
{
#if UCRTDIFF_REF_HAS_wmemmove_s
    {
        static const struct { size_t dsz; size_t n; } cs[] = { { 8, 0 }, { 8, 4 }, { 8, 8 }, { 4, 8 } };
        static const wchar_t src[] = L"abcdefgh";
        for (unsigned i = 0; i < sizeof(cs) / sizeof(cs[0]); i++) {
            wchar_t d1[16], d2[16];
            memset(d1, SO_SENTINEL, sizeof d1);
            memset(d2, SO_SENTINEL, sizeof d2);
            *_errno() = 0; *ref__errno() = 0;
            errno_t r1 = ud_wmemmove_s(d1, cs[i].dsz, src, cs[i].n);
            errno_t r2 = ref_wmemmove_s(d2, cs[i].dsz, src, cs[i].n);
            grade2("wmemmove_s", label_idx(i), (long long)r1, (long long)r2);
            grade2("wmemmove_s.errno", label_idx(i), *_errno(), *ref__errno());
            grade2("wmemmove_s.buf", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
        }
    }
    /* Genuinely overlapping move: buf[0..8) <- buf[2..10), forward overlap. */
    {
        wchar_t buf1[10] = L"abcdefghi", buf2[10] = L"abcdefghi";
        errno_t r1 = ud_wmemmove_s(buf1, 8, buf1 + 2, 6);
        errno_t r2 = ref_wmemmove_s(buf2, 8, buf2 + 2, 6);
        grade2("wmemmove_s.overlap", "-", (long long)r1, (long long)r2);
        grade2("wmemmove_s.overlap.buf", "-", fnv(buf1, sizeof buf1), fnv(buf2, sizeof buf2));
    }
#endif
}

/* ---- strtok / strtok_s / wcstok / wcstok_s ------------------------------
 * A whole tokenization run per input, so the context-carry behaviour (not
 * just one call) is graded; each side's context is fully independent. */
static void run_string_ops_strtok(void)
{
#if UCRTDIFF_REF_HAS_strtok
    static const char *const in[] = { "a b  c", "  lead", "trail  ", "", "noseps", "a,b;c a" };
    static const char *const ctl[] = { " ", " ", " ", " ,;", " ", " ,;" };
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        char d1[32], d2[32];
        strcpy(d1, in[i]); strcpy(d2, in[i]);
        char *p1 = d1, *p2 = d2;
        for (unsigned step = 0; step < 6; step++) {
            char *t1 = ud_strtok2(step == 0 ? p1 : 0, ctl[i]);
            char *t2 = ref_strtok(step == 0 ? p2 : 0, ctl[i]);
            grade2("strtok.tok", label_idx(i * 6 + step),
                   (long long)(t1 ? t1 - d1 : -1), (long long)(t2 ? t2 - d2 : -1));
            grade2("strtok.tok.hash", label_idx(i * 6 + step), t1 ? fnv(t1, strlen(t1)) : -1,
                   t2 ? fnv(t2, strlen(t2)) : -1);
        }
    }
#endif
}

static void run_string_ops_strtok_s(void)
{
#if UCRTDIFF_REF_HAS_strtok_s
    static const char *const in[] = { "a b  c", "", "noseps" };
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        char d1[32], d2[32];
        strcpy(d1, in[i]); strcpy(d2, in[i]);
        char *ctx1 = 0, *ctx2 = 0;
        for (unsigned step = 0; step < 4; step++) {
            char *t1 = ud_strtok_s2(step == 0 ? d1 : 0, " ", &ctx1);
            char *t2 = ref_strtok_s(step == 0 ? d2 : 0, " ", &ctx2);
            grade2("strtok_s.tok", label_idx(i * 4 + step),
                   (long long)(t1 ? t1 - d1 : -1), (long long)(t2 ? t2 - d2 : -1));
        }
    }
    /* Validation: NULL context/control both sides -> NULL, no crash. */
    {
        char d1[4] = "a", d2[4] = "a";
        char *ctx = d1;
        grade2("strtok_s.nullctl", "-", (long long)(ud_strtok_s2(d1, 0, &ctx) != 0),
               (long long)(ref_strtok_s(d2, 0, &ctx) != 0));
    }
#endif
}

static void run_string_ops_wcstok(void)
{
#if UCRTDIFF_REF_HAS_wcstok
    static const wchar_t *const in[] = { L"a b  c", L"  lead", L"trail  ", L"", L"noseps" };
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        wchar_t d1[32], d2[32];
        wcscpy(d1, in[i]); wcscpy(d2, in[i]);
        wchar_t *ctx1 = 0, *ctx2 = 0;
        for (unsigned step = 0; step < 5; step++) {
            wchar_t *t1 = ud_wcstok2(step == 0 ? d1 : 0, L" ", &ctx1);
            wchar_t *t2 = ref_wcstok(step == 0 ? d2 : 0, L" ", &ctx2);
            grade2("wcstok.tok", label_idx(i * 5 + step),
                   (long long)(t1 ? t1 - d1 : -1), (long long)(t2 ? t2 - d2 : -1));
        }
    }
#endif
}

static void run_string_ops_wcstok_s(void)
{
#if UCRTDIFF_REF_HAS_wcstok_s
    static const wchar_t *const in[] = { L"a b  c", L"", L"noseps" };
    for (unsigned i = 0; i < sizeof(in) / sizeof(in[0]); i++) {
        wchar_t d1[32], d2[32];
        wcscpy(d1, in[i]); wcscpy(d2, in[i]);
        wchar_t *ctx1 = 0, *ctx2 = 0;
        for (unsigned step = 0; step < 4; step++) {
            wchar_t *t1 = ud_wcstok_s2(step == 0 ? d1 : 0, L" ", &ctx1);
            wchar_t *t2 = ref_wcstok_s(step == 0 ? d2 : 0, L" ", &ctx2);
            grade2("wcstok_s.tok", label_idx(i * 4 + step),
                   (long long)(t1 ? t1 - d1 : -1), (long long)(t2 ? t2 - d2 : -1));
        }
    }
#endif
}

/* ---- case-insensitive compare: memicmp, stricmp, strnicmp, wcsicmp,
 * wcsnicmp, each graded plain + `_l` twin, under BOTH the "C" locale (which
 * for the narrow forms is the `__acrt_locale_changed_data == 0` fast path,
 * not literally "C" by name) and a real non-C locale, restoring after. */
static void run_string_ops_memicmp(void)
{
#if UCRTDIFF_REF_HAS__memicmp
    static const char *const a[] = { "", "abc", "ABC", "AbC", "12", "\x80\xff", "hello" };
    static const char *const b[] = { "", "ABC", "abc", "abc", "12", "\x80\xfe", "HELLO" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    static const size_t counts[] = { 0, 1, 2, 3, 100 };
    for (unsigned i = 0; i < n; i++)
        for (unsigned c = 0; c < 5; c++) {
            size_t len = counts[c];
            grade2("_memicmp", label_idx(i * 5 + c), (long long)ud__memicmp(a[i], b[i], len),
                   (long long)ref__memicmp(a[i], b[i], len));
            grade2("_memicmp_l", label_idx(i * 5 + c), (long long)ud__memicmp_l(a[i], b[i], len, 0),
                   (long long)ref__memicmp_l(a[i], b[i], len, 0));
        }
    /* under a real non-C locale, restored after */
#if UCRTDIFF_REF_CALLABLE_setlocale
    loc_set_both(0, LC_ALL, "de-DE", 0);
    for (unsigned i = 0; i < n; i++)
        grade2("_memicmp.locale", label_idx(i), (long long)ud__memicmp(a[i], b[i], strlen(a[i])),
               (long long)ref__memicmp(a[i], b[i], strlen(a[i])));
    loc_reset();
#endif
#endif
}

static void run_string_ops_stricmp(void)
{
#if UCRTDIFF_REF_HAS__stricmp
    static const char *const a[] = { "", "abc", "ABC", "AbC", "abcd", "hello", "\x80\xff" };
    static const char *const b[] = { "", "ABC", "abc", "abc", "abc", "HELLO", "\x80\xfe" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned i = 0; i < n; i++) {
        grade2("_stricmp", label_idx(i), (long long)ud__stricmp2(a[i], b[i]), (long long)ref__stricmp(a[i], b[i]));
        grade2("_stricmp_l", label_idx(i), (long long)ud__stricmp_l(a[i], b[i], 0),
               (long long)ref__stricmp_l(a[i], b[i], 0));
    }
#if UCRTDIFF_REF_CALLABLE_setlocale
    loc_set_both(0, LC_ALL, "de-DE", 0);
    for (unsigned i = 0; i < n; i++)
        grade2("_stricmp.locale", label_idx(i), (long long)ud__stricmp2(a[i], b[i]), (long long)ref__stricmp(a[i], b[i]));
    loc_reset();
#endif
#endif
}

static void run_string_ops_strnicmp(void)
{
#if UCRTDIFF_REF_HAS__strnicmp
    static const char *const a[] = { "", "abc", "ABC", "AbC", "abcd", "hello" };
    static const char *const b[] = { "", "ABC", "abc", "abc", "abc", "HELLO" };
    static const size_t counts[] = { 0, 1, 2, 3, 4, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned i = 0; i < n; i++)
        for (unsigned c = 0; c < 6; c++) {
            grade2("_strnicmp", label_idx(i * 6 + c), (long long)ud__strnicmp2(a[i], b[i], counts[c]),
                   (long long)ref__strnicmp(a[i], b[i], counts[c]));
            grade2("_strnicmp_l", label_idx(i * 6 + c), (long long)ud__strnicmp_l(a[i], b[i], counts[c], 0),
                   (long long)ref__strnicmp_l(a[i], b[i], counts[c], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
    loc_set_both(0, LC_ALL, "de-DE", 0);
    for (unsigned i = 0; i < n; i++)
        grade2("_strnicmp.locale", label_idx(i), (long long)ud__strnicmp2(a[i], b[i], strlen(a[i]) + 1),
               (long long)ref__strnicmp(a[i], b[i], strlen(a[i]) + 1));
    loc_reset();
#endif
#endif
}

static void run_string_ops_wcsicmp(void)
{
#if UCRTDIFF_REF_HAS__wcsicmp
    static const wchar_t *const a[] = { L"", L"abc", L"ABC", L"AbC", L"abcd", L"hello" };
    static const wchar_t *const b[] = { L"", L"ABC", L"abc", L"abc", L"abc", L"HELLO" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned i = 0; i < n; i++) {
        grade2("_wcsicmp", label_idx(i), (long long)ud__wcsicmp2(a[i], b[i]), (long long)ref__wcsicmp(a[i], b[i]));
        grade2("_wcsicmp_l", label_idx(i), (long long)ud__wcsicmp_l(a[i], b[i], 0),
               (long long)ref__wcsicmp_l(a[i], b[i], 0));
    }
#if UCRTDIFF_REF_CALLABLE_setlocale
    loc_set_both(0, LC_ALL, "de-DE", 0);
    for (unsigned i = 0; i < n; i++)
        grade2("_wcsicmp.locale", label_idx(i), (long long)ud__wcsicmp2(a[i], b[i]), (long long)ref__wcsicmp(a[i], b[i]));
    loc_reset();
#endif
#endif
}

static void run_string_ops_wcsnicmp(void)
{
#if UCRTDIFF_REF_HAS__wcsnicmp
    static const wchar_t *const a[] = { L"", L"abc", L"ABC", L"AbC", L"abcd", L"hello" };
    static const wchar_t *const b[] = { L"", L"ABC", L"abc", L"abc", L"abc", L"HELLO" };
    static const size_t counts[] = { 0, 1, 2, 3, 4, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned i = 0; i < n; i++)
        for (unsigned c = 0; c < 6; c++) {
            grade2("_wcsnicmp", label_idx(i * 6 + c), (long long)ud__wcsnicmp2(a[i], b[i], counts[c]),
                   (long long)ref__wcsnicmp(a[i], b[i], counts[c]));
            grade2("_wcsnicmp_l", label_idx(i * 6 + c), (long long)ud__wcsnicmp_l(a[i], b[i], counts[c], 0),
                   (long long)ref__wcsnicmp_l(a[i], b[i], counts[c], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
    loc_set_both(0, LC_ALL, "de-DE", 0);
    for (unsigned i = 0; i < n; i++)
        grade2("_wcsnicmp.locale", label_idx(i), (long long)ud__wcsnicmp2(a[i], b[i], wcslen(a[i]) + 1),
               (long long)ref__wcsnicmp(a[i], b[i], wcslen(a[i]) + 1));
    loc_reset();
#endif
#endif
}

/* ---- case mapping: strlwr / strupr / wcslwr / wcsupr, each base/_l/_s/_s_l.
 * Every case mutates a private copy (both `ud_`/`ref_` get their own),
 * grading the whole destination buffer (sentinel-filled first) plus the
 * return value/pointer-offset and errno for the `_s` forms. Graded under the
 * "C" locale, then under one real non-C locale set on BOTH sides via
 * loc_set_both/loc_reset (ucrtdiff.c), which needs UCRTDIFF_REF_CALLABLE_setlocale. */
#define SO_CASE_SENTINEL 0x55

static void run_string_ops_strlwr(void)
{
#if UCRTDIFF_REF_HAS__strlwr
    static const char *const s[] = { "", "ABC", "abc", "AbCdE", "\x80\xff", "HELLO WORLD" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            char d1[32], d2[32];
            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            char *r1 = ud__strlwr(d1);
            char *r2 = ref__strlwr(d2);
            grade2(pass ? "_strlwr.locale" : "_strlwr", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2(pass ? "_strlwr.locale.ret" : "_strlwr.ret", label_idx(i), (long long)(r1 - d1), (long long)(r2 - d2));

            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            ud__strlwr_l(d1, 0);
            ref__strlwr_l(d2, 0);
            grade2(pass ? "_strlwr_l.locale" : "_strlwr_l", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            for (unsigned c = 0; c < 3; c++) {
                static const size_t sizes[] = { 32, 6, 2 };
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                strcpy(d1, s[i]); strcpy(d2, s[i]);
                *_errno() = 0; *ref__errno() = 0;
                errno_t e1 = ud__strlwr_s(d1, sizes[c]);
                errno_t e2 = ref__strlwr_s(d2, sizes[c]);
                grade2(pass ? "_strlwr_s.locale" : "_strlwr_s", label_idx(i * 3 + c), (long long)e1, (long long)e2);
                grade2(pass ? "_strlwr_s.locale.errno" : "_strlwr_s.errno", label_idx(i * 3 + c), *_errno(), *ref__errno());
                grade2(pass ? "_strlwr_s.locale.buf" : "_strlwr_s.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                strcpy(d1, s[i]); strcpy(d2, s[i]);
                errno_t f1 = ud__strlwr_s_l(d1, sizes[c], 0);
                errno_t f2 = ref__strlwr_s_l(d2, sizes[c], 0);
                grade2(pass ? "_strlwr_s_l.locale" : "_strlwr_s_l", label_idx(i * 3 + c), (long long)f1, (long long)f2);
                grade2(pass ? "_strlwr_s_l.locale.buf" : "_strlwr_s_l.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_strupr(void)
{
#if UCRTDIFF_REF_HAS__strupr
    static const char *const s[] = { "", "ABC", "abc", "AbCdE", "\x80\xff", "hello world" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            char d1[32], d2[32];
            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            char *r1 = ud__strupr(d1);
            char *r2 = ref__strupr(d2);
            grade2(pass ? "_strupr.locale" : "_strupr", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2(pass ? "_strupr.locale.ret" : "_strupr.ret", label_idx(i), (long long)(r1 - d1), (long long)(r2 - d2));

            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            strcpy(d1, s[i]); strcpy(d2, s[i]);
            ud__strupr_l(d1, 0);
            ref__strupr_l(d2, 0);
            grade2(pass ? "_strupr_l.locale" : "_strupr_l", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            for (unsigned c = 0; c < 3; c++) {
                static const size_t sizes[] = { 32, 6, 2 };
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                strcpy(d1, s[i]); strcpy(d2, s[i]);
                *_errno() = 0; *ref__errno() = 0;
                errno_t e1 = ud__strupr_s(d1, sizes[c]);
                errno_t e2 = ref__strupr_s(d2, sizes[c]);
                grade2(pass ? "_strupr_s.locale" : "_strupr_s", label_idx(i * 3 + c), (long long)e1, (long long)e2);
                grade2(pass ? "_strupr_s.locale.errno" : "_strupr_s.errno", label_idx(i * 3 + c), *_errno(), *ref__errno());
                grade2(pass ? "_strupr_s.locale.buf" : "_strupr_s.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                strcpy(d1, s[i]); strcpy(d2, s[i]);
                errno_t f1 = ud__strupr_s_l(d1, sizes[c], 0);
                errno_t f2 = ref__strupr_s_l(d2, sizes[c], 0);
                grade2(pass ? "_strupr_s_l.locale" : "_strupr_s_l", label_idx(i * 3 + c), (long long)f1, (long long)f2);
                grade2(pass ? "_strupr_s_l.locale.buf" : "_strupr_s_l.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcslwr(void)
{
#if UCRTDIFF_REF_HAS__wcslwr
    static const wchar_t *const s[] = { L"", L"ABC", L"abc", L"AbCdE", L"\x00c9\xffff", L"HELLO WORLD" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            wchar_t d1[32], d2[32];
            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            wchar_t *r1 = ud__wcslwr(d1);
            wchar_t *r2 = ref__wcslwr(d2);
            grade2(pass ? "_wcslwr.locale" : "_wcslwr", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2(pass ? "_wcslwr.locale.ret" : "_wcslwr.ret", label_idx(i), (long long)(r1 - d1), (long long)(r2 - d2));

            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            ud__wcslwr_l(d1, 0);
            ref__wcslwr_l(d2, 0);
            grade2(pass ? "_wcslwr_l.locale" : "_wcslwr_l", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            for (unsigned c = 0; c < 3; c++) {
                static const size_t sizes[] = { 32, 6, 2 };
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                wcscpy(d1, s[i]); wcscpy(d2, s[i]);
                errno_t e1 = ud__wcslwr_s(d1, sizes[c]);
                errno_t e2 = ref__wcslwr_s(d2, sizes[c]);
                grade2(pass ? "_wcslwr_s.locale" : "_wcslwr_s", label_idx(i * 3 + c), (long long)e1, (long long)e2);
                grade2(pass ? "_wcslwr_s.locale.buf" : "_wcslwr_s.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                wcscpy(d1, s[i]); wcscpy(d2, s[i]);
                errno_t f1 = ud__wcslwr_s_l(d1, sizes[c], 0);
                errno_t f2 = ref__wcslwr_s_l(d2, sizes[c], 0);
                grade2(pass ? "_wcslwr_s_l.locale" : "_wcslwr_s_l", label_idx(i * 3 + c), (long long)f1, (long long)f2);
                grade2(pass ? "_wcslwr_s_l.locale.buf" : "_wcslwr_s_l.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcsupr(void)
{
#if UCRTDIFF_REF_HAS__wcsupr
    static const wchar_t *const s[] = { L"", L"ABC", L"abc", L"AbCdE", L"\x00e9\xffff", L"hello world" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            wchar_t d1[32], d2[32];
            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            wchar_t *r1 = ud__wcsupr(d1);
            wchar_t *r2 = ref__wcsupr(d2);
            grade2(pass ? "_wcsupr.locale" : "_wcsupr", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            grade2(pass ? "_wcsupr.locale.ret" : "_wcsupr.ret", label_idx(i), (long long)(r1 - d1), (long long)(r2 - d2));

            memset(d1, SO_CASE_SENTINEL, sizeof d1);
            memset(d2, SO_CASE_SENTINEL, sizeof d2);
            wcscpy(d1, s[i]); wcscpy(d2, s[i]);
            ud__wcsupr_l(d1, 0);
            ref__wcsupr_l(d2, 0);
            grade2(pass ? "_wcsupr_l.locale" : "_wcsupr_l", label_idx(i), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

            for (unsigned c = 0; c < 3; c++) {
                static const size_t sizes[] = { 32, 6, 2 };
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                wcscpy(d1, s[i]); wcscpy(d2, s[i]);
                errno_t e1 = ud__wcsupr_s(d1, sizes[c]);
                errno_t e2 = ref__wcsupr_s(d2, sizes[c]);
                grade2(pass ? "_wcsupr_s.locale" : "_wcsupr_s", label_idx(i * 3 + c), (long long)e1, (long long)e2);
                grade2(pass ? "_wcsupr_s.locale.buf" : "_wcsupr_s.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));

                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                wcscpy(d1, s[i]); wcscpy(d2, s[i]);
                errno_t f1 = ud__wcsupr_s_l(d1, sizes[c], 0);
                errno_t f2 = ref__wcsupr_s_l(d2, sizes[c], 0);
                grade2(pass ? "_wcsupr_s_l.locale" : "_wcsupr_s_l", label_idx(i * 3 + c), (long long)f1, (long long)f2);
                grade2(pass ? "_wcsupr_s_l.locale.buf" : "_wcsupr_s_l.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1), fnv(d2, sizeof d2));
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

/* ---- collation: strcoll/stricoll/strncoll/strnicoll (narrow) and their
 * wide twins, each base + `_l`. Graded under "C" then one non-C locale.
 * `count` boundaries (0, exact, beyond) covered for the `n` forms. */
static void run_string_ops_strcoll(void)
{
#if UCRTDIFF_REF_HAS_strcoll
    static const char *const a[] = { "", "abc", "abc", "ABCDE", "hello", "\x80\xff" };
    static const char *const b[] = { "", "abc", "abd", "ABC", "world", "\x81\xfe" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            grade2(pass ? "strcoll.locale" : "strcoll", label_idx(i), (long long)ud_strcoll2(a[i], b[i]),
                   (long long)ref_strcoll(a[i], b[i]));
            grade2(pass ? "_strcoll_l.locale" : "_strcoll_l", label_idx(i), (long long)ud__strcoll_l(a[i], b[i], 0),
                   (long long)ref__strcoll_l(a[i], b[i], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_stricoll(void)
{
#if UCRTDIFF_REF_HAS__stricoll
    static const char *const a[] = { "", "abc", "ABC", "AbC", "hello", "\x80\xff" };
    static const char *const b[] = { "", "ABC", "abc", "abc", "HELLO", "\x80\xfe" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            grade2(pass ? "_stricoll.locale" : "_stricoll", label_idx(i), (long long)ud__stricoll(a[i], b[i]),
                   (long long)ref__stricoll(a[i], b[i]));
            grade2(pass ? "_stricoll_l.locale" : "_stricoll_l", label_idx(i), (long long)ud__stricoll_l(a[i], b[i], 0),
                   (long long)ref__stricoll_l(a[i], b[i], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_strncoll(void)
{
#if UCRTDIFF_REF_HAS__strncoll
    static const char *const a[] = { "", "abc", "abc", "ABCDE", "hello" };
    static const char *const b[] = { "", "abc", "abd", "ABC", "world" };
    static const size_t counts[] = { 0, 1, 3, 5, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++)
            for (unsigned c = 0; c < 5; c++) {
                grade2(pass ? "_strncoll.locale" : "_strncoll", label_idx(i * 5 + c),
                       (long long)ud__strncoll(a[i], b[i], counts[c]), (long long)ref__strncoll(a[i], b[i], counts[c]));
                grade2(pass ? "_strncoll_l.locale" : "_strncoll_l", label_idx(i * 5 + c),
                       (long long)ud__strncoll_l(a[i], b[i], counts[c], 0),
                       (long long)ref__strncoll_l(a[i], b[i], counts[c], 0));
            }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_strnicoll(void)
{
#if UCRTDIFF_REF_HAS__strnicoll
    static const char *const a[] = { "", "abc", "ABC", "AbC", "hello" };
    static const char *const b[] = { "", "ABC", "abc", "abc", "HELLO" };
    static const size_t counts[] = { 0, 1, 3, 5, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++)
            for (unsigned c = 0; c < 5; c++) {
                grade2(pass ? "_strnicoll.locale" : "_strnicoll", label_idx(i * 5 + c),
                       (long long)ud__strnicoll(a[i], b[i], counts[c]), (long long)ref__strnicoll(a[i], b[i], counts[c]));
                grade2(pass ? "_strnicoll_l.locale" : "_strnicoll_l", label_idx(i * 5 + c),
                       (long long)ud__strnicoll_l(a[i], b[i], counts[c], 0),
                       (long long)ref__strnicoll_l(a[i], b[i], counts[c], 0));
            }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcscoll(void)
{
#if UCRTDIFF_REF_HAS_wcscoll
    static const wchar_t *const a[] = { L"", L"abc", L"abc", L"ABCDE", L"hello" };
    static const wchar_t *const b[] = { L"", L"abc", L"abd", L"ABC", L"world" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            grade2(pass ? "wcscoll.locale" : "wcscoll", label_idx(i), (long long)ud_wcscoll2(a[i], b[i]),
                   (long long)ref_wcscoll(a[i], b[i]));
            grade2(pass ? "_wcscoll_l.locale" : "_wcscoll_l", label_idx(i), (long long)ud__wcscoll_l(a[i], b[i], 0),
                   (long long)ref__wcscoll_l(a[i], b[i], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcsicoll(void)
{
#if UCRTDIFF_REF_HAS__wcsicoll
    static const wchar_t *const a[] = { L"", L"abc", L"ABC", L"AbC", L"hello" };
    static const wchar_t *const b[] = { L"", L"ABC", L"abc", L"abc", L"HELLO" };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            grade2(pass ? "_wcsicoll.locale" : "_wcsicoll", label_idx(i), (long long)ud__wcsicoll(a[i], b[i]),
                   (long long)ref__wcsicoll(a[i], b[i]));
            grade2(pass ? "_wcsicoll_l.locale" : "_wcsicoll_l", label_idx(i), (long long)ud__wcsicoll_l(a[i], b[i], 0),
                   (long long)ref__wcsicoll_l(a[i], b[i], 0));
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcsncoll(void)
{
#if UCRTDIFF_REF_HAS__wcsncoll
    static const wchar_t *const a[] = { L"", L"abc", L"abc", L"ABCDE", L"hello" };
    static const wchar_t *const b[] = { L"", L"abc", L"abd", L"ABC", L"world" };
    static const size_t counts[] = { 0, 1, 3, 5, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++)
            for (unsigned c = 0; c < 5; c++) {
                grade2(pass ? "_wcsncoll.locale" : "_wcsncoll", label_idx(i * 5 + c),
                       (long long)ud__wcsncoll(a[i], b[i], counts[c]), (long long)ref__wcsncoll(a[i], b[i], counts[c]));
                grade2(pass ? "_wcsncoll_l.locale" : "_wcsncoll_l", label_idx(i * 5 + c),
                       (long long)ud__wcsncoll_l(a[i], b[i], counts[c], 0),
                       (long long)ref__wcsncoll_l(a[i], b[i], counts[c], 0));
            }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcsnicoll(void)
{
#if UCRTDIFF_REF_HAS__wcsnicoll
    static const wchar_t *const a[] = { L"", L"abc", L"ABC", L"AbC", L"hello" };
    static const wchar_t *const b[] = { L"", L"ABC", L"abc", L"abc", L"HELLO" };
    static const size_t counts[] = { 0, 1, 3, 5, 100 };
    unsigned n = sizeof(a) / sizeof(a[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++)
            for (unsigned c = 0; c < 5; c++) {
                grade2(pass ? "_wcsnicoll.locale" : "_wcsnicoll", label_idx(i * 5 + c),
                       (long long)ud__wcsnicoll(a[i], b[i], counts[c]), (long long)ref__wcsnicoll(a[i], b[i], counts[c]));
                grade2(pass ? "_wcsnicoll_l.locale" : "_wcsnicoll_l", label_idx(i * 5 + c),
                       (long long)ud__wcsnicoll_l(a[i], b[i], counts[c], 0),
                       (long long)ref__wcsnicoll_l(a[i], b[i], counts[c], 0));
            }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

/* ---- strxfrm / wcsxfrm: the return-length-on-overflow contract (a
 * too-small destination returns the length needed MINUS the NUL, matching
 * `strlen`'s contract) is graded explicitly via a deliberately tiny `count`
 * alongside `count == 0` (dest may be NULL) and an exact-fit destination. */
static void run_string_ops_strxfrm(void)
{
#if UCRTDIFF_REF_HAS_strxfrm
    static const char *const s[] = { "", "abc", "hello world", "\x80\xff" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            static const size_t counts[] = { 0, 1, 64 };
            for (unsigned c = 0; c < 3; c++) {
                char d1[64], d2[64];
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                char *p1 = counts[c] == 0 ? 0 : d1;
                char *p2 = counts[c] == 0 ? 0 : d2;
                *_errno() = 0; *ref__errno() = 0;
                size_t r1 = ud_strxfrm2(p1, s[i], counts[c]);
                size_t r2 = ref_strxfrm(p2, s[i], counts[c]);
                grade2(pass ? "strxfrm.locale" : "strxfrm", label_idx(i * 3 + c), (long long)r1, (long long)r2);
                grade2(pass ? "strxfrm.locale.errno" : "strxfrm.errno", label_idx(i * 3 + c), *_errno(), *ref__errno());
                if (counts[c] != 0)
                    grade2(pass ? "strxfrm.locale.buf" : "strxfrm.buf", label_idx(i * 3 + c), fnv(d1, sizeof d1),
                           fnv(d2, sizeof d2));

                size_t r3 = ud__strxfrm_l(p1, s[i], counts[c], 0);
                size_t r4 = ref__strxfrm_l(p2, s[i], counts[c], 0);
                grade2(pass ? "_strxfrm_l.locale" : "_strxfrm_l", label_idx(i * 3 + c), (long long)r3, (long long)r4);
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

static void run_string_ops_wcsxfrm(void)
{
#if UCRTDIFF_REF_HAS_wcsxfrm
    static const wchar_t *const s[] = { L"", L"abc", L"hello world", L"\x00e9\xffff" };
    unsigned n = sizeof(s) / sizeof(s[0]);
    for (unsigned pass = 0; pass < 2; pass++) {
        if (pass == 1) {
#if UCRTDIFF_REF_CALLABLE_setlocale
            loc_set_both(0, LC_ALL, "de-DE", 0);
#else
            break;
#endif
        }
        for (unsigned i = 0; i < n; i++) {
            static const size_t counts[] = { 0, 1, 64 };
            for (unsigned c = 0; c < 3; c++) {
                wchar_t d1[64], d2[64];
                memset(d1, SO_CASE_SENTINEL, sizeof d1);
                memset(d2, SO_CASE_SENTINEL, sizeof d2);
                wchar_t *p1 = counts[c] == 0 ? 0 : d1;
                wchar_t *p2 = counts[c] == 0 ? 0 : d2;
                *_errno() = 0; *ref__errno() = 0;
                size_t r1 = ud_wcsxfrm2(p1, s[i], counts[c]);
                size_t r2 = ref_wcsxfrm(p2, s[i], counts[c]);
                grade2(pass ? "wcsxfrm.locale" : "wcsxfrm", label_idx(i * 3 + c), (long long)r1, (long long)r2);
                grade2(pass ? "wcsxfrm.locale.errno" : "wcsxfrm.errno", label_idx(i * 3 + c), *_errno(), *ref__errno());

                size_t r3 = ud__wcsxfrm_l(p1, s[i], counts[c], 0);
                size_t r4 = ref__wcsxfrm_l(p2, s[i], counts[c], 0);
                grade2(pass ? "_wcsxfrm_l.locale" : "_wcsxfrm_l", label_idx(i * 3 + c), (long long)r3, (long long)r4);
            }
        }
#if UCRTDIFF_REF_CALLABLE_setlocale
        if (pass == 1) loc_reset();
#endif
    }
#endif
}

/* ---- registry: paste STRING_OPS_HAND_ENTRIES into run_legacy()'s `legacy[]`
 * table (same shape as the existing ST_HAND/FP_HAND rows just above it). */
#if UCRTDIFF_REF_HAS__strdup
#define SO_FN__strdup run_string_ops__strdup
#else
#define SO_FN__strdup 0
#endif
#if UCRTDIFF_REF_HAS__wcsdup
#define SO_FN__wcsdup run_string_ops__wcsdup
#else
#define SO_FN__wcsdup 0
#endif
#if UCRTDIFF_REF_HAS___strncnt
#define SO_FN___strncnt run_string_ops___strncnt
#else
#define SO_FN___strncnt 0
#endif
#if UCRTDIFF_REF_HAS___wcsncnt
#define SO_FN___wcsncnt run_string_ops___wcsncnt
#else
#define SO_FN___wcsncnt 0
#endif
/* strnlen's runner always runs (it grades the self-consistency check even
 * when UCRTDIFF_REF_HAS_strnlen is 0), so it is registered unconditionally. */
#define SO_FN_strnlen run_string_ops_strnlen
#define SO_FN_wcs_plain run_string_ops_wcs_plain
#define SO_FN_arch_split run_string_ops_arch_split
#define SO_HAND(n) { "string_ops:" #n, SO_FN_##n },

#define STRING_OPS_HAND_ENTRIES \
    SO_HAND(_strdup) SO_HAND(_wcsdup) SO_HAND(__strncnt) SO_HAND(__wcsncnt) \
    SO_HAND(strnlen) SO_HAND(wcs_plain) SO_HAND(arch_split) \
    { "string_ops:strcat_s", run_string_ops_strcat_s }, \
    { "string_ops:strcpy_s", run_string_ops_strcpy_s }, \
    { "string_ops:strncat_s", run_string_ops_strncat_s }, \
    { "string_ops:strncpy_s", run_string_ops_strncpy_s }, \
    { "string_ops:_strnset_s", run_string_ops__strnset_s }, \
    { "string_ops:_strset_s", run_string_ops__strset_s }, \
    { "string_ops:wcscat_s", run_string_ops_wcscat_s }, \
    { "string_ops:wcscpy_s", run_string_ops_wcscpy_s }, \
    { "string_ops:wcsncat_s", run_string_ops_wcsncat_s }, \
    { "string_ops:wcsncpy_s", run_string_ops_wcsncpy_s }, \
    { "string_ops:_wcsnset_s", run_string_ops__wcsnset_s }, \
    { "string_ops:_wcsset_s", run_string_ops__wcsset_s }, \
    { "string_ops:memcpy_s", run_string_ops_memcpy_s }, \
    { "string_ops:wmemcpy_s", run_string_ops_wmemcpy_s }, \
    { "string_ops:wmemmove_s", run_string_ops_wmemmove_s }, \
    { "string_ops:strtok", run_string_ops_strtok }, \
    { "string_ops:strtok_s", run_string_ops_strtok_s }, \
    { "string_ops:wcstok", run_string_ops_wcstok }, \
    { "string_ops:wcstok_s", run_string_ops_wcstok_s }, \
    { "string_ops:memicmp", run_string_ops_memicmp }, \
    { "string_ops:stricmp", run_string_ops_stricmp }, \
    { "string_ops:strnicmp", run_string_ops_strnicmp }, \
    { "string_ops:wcsicmp", run_string_ops_wcsicmp }, \
    { "string_ops:wcsnicmp", run_string_ops_wcsnicmp }, \
    { "string_ops:strlwr", run_string_ops_strlwr }, \
    { "string_ops:strupr", run_string_ops_strupr }, \
    { "string_ops:wcslwr", run_string_ops_wcslwr }, \
    { "string_ops:wcsupr", run_string_ops_wcsupr }, \
    { "string_ops:strcoll", run_string_ops_strcoll }, \
    { "string_ops:stricoll", run_string_ops_stricoll }, \
    { "string_ops:strncoll", run_string_ops_strncoll }, \
    { "string_ops:strnicoll", run_string_ops_strnicoll }, \
    { "string_ops:wcscoll", run_string_ops_wcscoll }, \
    { "string_ops:wcsicoll", run_string_ops_wcsicoll }, \
    { "string_ops:wcsncoll", run_string_ops_wcsncoll }, \
    { "string_ops:wcsnicoll", run_string_ops_wcsnicoll }, \
    { "string_ops:strxfrm", run_string_ops_strxfrm }, \
    { "string_ops:wcsxfrm", run_string_ops_wcsxfrm },


/* ==== staging/time.c, inlined ==== */
/* Its own names: CT_STRLEN lives in the ctime block, compiled out once ctime.cpp went. */
#define TM_STRLEN(p) strlen(p)
#define TM_WCSLEN(p) wcslen(p)
/*
 * time -- guest grader for the "time" batch: time/tzset.cpp, localtime.cpp,
 * gmtime.cpp, mktime.cpp, asctime.cpp, utime.cpp, ftime.cpp, strdate.cpp,
 * strtime.cpp, time.cpp, clock.cpp, difftime.cpp (plus the internal-only
 * loctotime.cpp/days.cpp, which have no DLL export and so no runner here --
 * they are graded only indirectly, through gmtime/mktime/loctotime's callers
 * `_findfirst*`/`_stat*` elsewhere, and by the C-file-boundary rule in the
 * batch report).
 *
 * Included verbatim into ucrtdiff.c just before `run_legacy`, exactly like
 * the FP_HAND/ST_HAND/LOC_HAND/CONVTIME_HAND blocks above it: every helper
 * this file uses (grade2, install_counting_iph, label_idx, fnv,
 * g_iph_dll/g_iph_ref, _errno, ref__errno, UD_IMP) is already in scope by
 * the inclusion point, and `<time.h>` (for `struct tm`/`__time32_t`/
 * `__time64_t`/`clock_t` and the plain DLL-side declarations) and
 * `<sys/stat.h>` are already `#include`d earlier in ucrtdiff.c (the
 * strftime/wcsftime block and the stat block, respectively) -- only
 * `<sys/timeb.h>` and `<sys/utime.h>` are new here.
 *
 * Twelve separate guards, one per C file that may be deleted independently
 * (loctotime.cpp/days.cpp have none -- see above):
 *   UCRTDIFF_REF_HAS__tzset        -- time/tzset.cpp
 *   UCRTDIFF_REF_HAS__localtime32  -- time/localtime.cpp
 *   UCRTDIFF_REF_HAS__gmtime32     -- time/gmtime.cpp
 *   UCRTDIFF_REF_HAS__mktime32     -- time/mktime.cpp
 *   UCRTDIFF_REF_HAS_asctime       -- time/asctime.cpp
 *   UCRTDIFF_REF_HAS__utime32      -- time/utime.cpp
 *   UCRTDIFF_REF_HAS__ftime32      -- time/ftime.cpp
 *   UCRTDIFF_REF_HAS__strdate      -- time/strdate.cpp
 *   UCRTDIFF_REF_HAS__strtime      -- time/strtime.cpp
 *   UCRTDIFF_REF_HAS__time32       -- time/time.cpp
 *   UCRTDIFF_REF_HAS_clock         -- time/clock.cpp
 *   UCRTDIFF_REF_HAS__difftime32   -- time/difftime.cpp
 *
 * NONE of these twelve macros exist in tools/gen_ucrt_tier2_cases.py's
 * HAND_GRADED tuple yet -- the integrator must add the twelve representative
 * names above and regenerate generated-refs.h for all three arches before
 * these guards read as 1. Until then they read as 0 (an undefined macro in
 * `#if` is 0), which is exactly the "guards=0" case this file was
 * syntax-checked under.
 *
 * All twelve representative `ref_<name>` symbols (and every other exported
 * name this file calls) were confirmed present in
 * build-{i386,amd64,arm64}/libucrt_ref.a with llvm-nm (i386 carries the
 * extra leading underscore llvm-nm shows separately from the bare name; all
 * three archives were checked) -- EXCEPT `_isindst`, `__tzset`,
 * `__getgmtimebuf`, `__loctotime32_t`/`__loctotime64_t`, which are not in
 * ucrtbase.spec (no export) and so have no `ref_`/`ud_` symbol at all and
 * are not graded here; they are exercised only indirectly, through the
 * exported functions that call them.
 *
 * None of this batch's DLL-side spellings are wrapped in a mingw
 * `__CRT_INLINE` (checked against generic-w64-mingw32/include/{time,sys/
 * timeb,sys/utime}.h: only the non-underscore convenience wrappers --
 * `time`, `localtime`, `gmtime`, `mktime`, `difftime`, `tzset`, `ftime`,
 * `_utime`/`_futime`/`_wutime`, all `__MINGW_ASM_CALL`/`__CRT_INLINE`
 * aliases onto the underscore-suffixed exports this file actually grades --
 * are inlined), so every DLL-side call below is a PLAIN name, not a `ud_`
 * import pointer, matching `convtime.c`'s precedent for the same reason.
 */
#include <sys/timeb.h>
#include <sys/utime.h>

/* ==== tzset (time/tzset.cpp) =============================================
 * `_isindst`/`__tzset` are internal (no export); `_tzset` is graded here by
 * its OBSERVABLE side effect on the shared `_timezone`/`_daylight`/
 * `_tzname` state (already-exported getters from time/timeset.cpp, graded
 * for their OWN correctness by convtime.c's `UCRTDIFF_REF_HAS__get_daylight`
 * block -- this block re-uses them only as an oracle for tzset's writes,
 * and does not re-litigate their own behaviour).
 *
 * Both sides' `_tzset` share ONE process-wide environment block, so TZ is
 * set explicitly with `_wputenv_s` before EACH side's `_tzset()` call (never
 * once for both), and restored to "unset" afterwards so later rows (and any
 * other batch's runner) do not inherit a TZ this file set. */
#if UCRTDIFF_REF_HAS__tzset
void __cdecl ref__tzset(void);
errno_t __cdecl ref__wputenv_s(const wchar_t *, const wchar_t *);
errno_t __cdecl ref__get_daylight(int *);
errno_t __cdecl ref__get_dstbias(long *);
errno_t __cdecl ref__get_timezone(long *);
errno_t __cdecl ref__get_tzname(size_t *, char *, size_t, int);

/* PST8PDT: classic US Pacific, exercises the day-in-month DST path;
 * EST5EDT similarly; a fixed-offset-only zone (no DST suffix) exercises
 * `daylight = *tz_env ? 1 : 0` == 0; unsetting TZ (NULL value) exercises the
 * `tzset_from_system_nolock` path (whatever the guest's real system zone
 * is -- graded structurally, not against a fixed expected value, since the
 * test VM's zone is not controlled by this file). */
static void tzset_row(const wchar_t *tz, const char *lbl)
{
    int day1 = -999, day2 = -999;
    long dst1 = -999, dst2 = -999, tz1 = -999, tz2 = -999;
    char nm1[64], nm2[64];
    size_t ln1 = 0, ln2 = 0;

    _wputenv_s(L"TZ", tz ? tz : L"");
    _tzset();
    _get_daylight(&day1);
    _get_dstbias(&dst1);
    _get_timezone(&tz1);
    _get_tzname(&ln1, nm1, sizeof nm1, 0);

    ref__wputenv_s(L"TZ", tz ? tz : L"");
    ref__tzset();
    ref__get_daylight(&day2);
    ref__get_dstbias(&dst2);
    ref__get_timezone(&tz2);
    ref__get_tzname(&ln2, nm2, sizeof nm2, 0);

    grade2("_tzset.daylight", lbl, day1, day2);
    grade2("_tzset.dstbias", lbl, dst1, dst2);
    grade2("_tzset.timezone", lbl, tz1, tz2);
    grade2("_tzset.tzname0", lbl, fnv(nm1, ln1), fnv(nm2, ln2));

    /* Standard-time index-1 name is meaningful only when DST is on for this
     * zone; both sides agree on `daylight` above, so gating on the DLL's own
     * value is not a second oracle. */
    if (day1) {
        size_t ln1b = 0, ln2b = 0;
        char nm1b[64], nm2b[64];
        _get_tzname(&ln1b, nm1b, sizeof nm1b, 1);
        ref__get_tzname(&ln2b, nm2b, sizeof nm2b, 1);
        grade2("_tzset.tzname1", lbl, fnv(nm1b, ln1b), fnv(nm2b, ln2b));
    }
}

static void run_time__tzset(void)
{
    install_counting_iph();
    tzset_row(L"PST8PDT", "PST8PDT");
    tzset_row(L"EST5EDT", "EST5EDT");
    tzset_row(L"UTC0", "UTC0");
    tzset_row(NULL, "unset");
    /* Restore a clean slate for any runner after this one in the sweep. */
    _wputenv_s(L"TZ", L"");
    ref__wputenv_s(L"TZ", L"");
    _tzset();
    ref__tzset();
}
#endif /* UCRTDIFF_REF_HAS__tzset */

/* ==== localtime / gmtime (time/localtime.cpp, time/gmtime.cpp) ===========
 * Graded together: `localtime.cpp` is meaningless to grade without a fixed
 * TZ (its whole job is the TZ/DST adjustment), so both blocks share one TZ
 * setup, and `_gmtime32_s`/`_gmtime64_s` (TZ-independent) are graded as
 * plain baseline rows inside the same corpus loop for locality. */
/* gmtime.cpp's `ref_` prototypes are declared here, UNCONDITIONALLY under
 * their own guard (not nested under `_localtime32`'s), because the
 * localtime block below calls them too when both guards are 1 -- a repeated
 * `extern`-style prototype declaration is legal C as long as it is
 * identical, so this one declaration serves both the localtime block's
 * inner `#if UCRTDIFF_REF_HAS__gmtime32` calls and the standalone
 * `run_time__gmtime32` entry point further down. */
#if UCRTDIFF_REF_HAS__gmtime32
errno_t __cdecl ref__gmtime32_s(struct tm *, const __time32_t *);
errno_t __cdecl ref__gmtime64_s(struct tm *, const __time64_t *);
struct tm *__cdecl ref__gmtime32(const __time32_t *);
struct tm *__cdecl ref__gmtime64(const __time64_t *);
#endif

/* `LOCGM_S_ROW`/`LOCGM_PTR_ROW` are used by BOTH the localtime block below
 * (when `_gmtime32` is also 1) and the standalone gmtime-only block further
 * down (when `_localtime32` is 0) -- defined once, unconditionally, so
 * neither guard combination leaves them undefined. */
#define LOCGM_S_ROW(fn, T, val, lbl) do {                                      \
        struct tm t1, t2;                                                      \
        memset(&t1, 0xAB, sizeof t1); memset(&t2, 0xAB, sizeof t2);            \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        T tv = (val);                                                          \
        errno_t c1 = fn(&t1, &tv);                                             \
        errno_t c2 = ref_##fn(&t2, &tv);                                       \
        grade2(#fn, lbl, c1, c2);                                              \
        grade2(#fn ".val", lbl, fnv(&t1, sizeof t1), fnv(&t2, sizeof t2));     \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

#define LOCGM_PTR_ROW(fn, T, val, lbl) do {                                    \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        T tv = (val);                                                         \
        struct tm *p1 = fn(&tv);                                              \
        struct tm *p2 = ref_##fn(&tv);                                        \
        grade2(#fn ".isnull", lbl, p1 == 0, p2 == 0);                         \
        if (p1 && p2)                                                         \
            grade2(#fn ".val", lbl, fnv(p1, sizeof *p1), fnv(p2, sizeof *p2)); \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

#if UCRTDIFF_REF_HAS__localtime32
errno_t __cdecl ref__localtime32_s(struct tm *, const __time32_t *);
errno_t __cdecl ref__localtime64_s(struct tm *, const __time64_t *);
struct tm *__cdecl ref__localtime32(const __time32_t *);
struct tm *__cdecl ref__localtime64(const __time64_t *);
void __cdecl ref__tzset(void);
errno_t __cdecl ref__wputenv_s(const wchar_t *, const wchar_t *);

/* 0 (epoch), one day in, mid-range, __crt_time_time_t_traits<__time32_t>::
 * max_time_t (0x7fffd27f), and a value inside +/-3 days of that bound (to
 * hit `common_localtime_s`'s "near an edge" hand-adjustment path) and of 0
 * (ditto, near the epoch); -1 is the one EINVAL, no-handler case. Interval
 * chosen to land near a 2024 US DST transition (2024-03-10 09:xx UTC /
 * 2024-11-03 08:xx UTC) so `_isindst` actually flips both ways. */
static const __time32_t g_lt32[] = {
    0, 1, 86400, 1000000000L, 0x7fffd27fL, 0x7fffd27fL - 100, -1,
    1710060000L /* 2024-03-10 10:00 UTC, just after US spring-forward */,
    1730620800L /* 2024-11-03 08:00 UTC, just after US fall-back */,
};
#define LT32_N (sizeof(g_lt32) / sizeof(g_lt32[0]))
static const __time64_t g_lt64[] = {
    0, 1, 86400, 1000000000LL, 2147483648LL, 0x793582affLL, -1,
    1710060000LL, 1730620800LL,
};
#define LT64_N (sizeof(g_lt64) / sizeof(g_lt64[0]))

static void run_time__localtime_and_gmtime(const wchar_t *tz, const char *tzlbl)
{
    unsigned i;
    _wputenv_s(L"TZ", tz); _tzset();
    ref__wputenv_s(L"TZ", tz); ref__tzset();

    install_counting_iph();
    for (i = 0; i < LT32_N; i++) {
        char lbl[32];
        int n = 0;
        while (tzlbl[n] && n < 16) { lbl[n] = tzlbl[n]; n++; }
        lbl[n++] = '.'; lbl[n] = 0;
        strcat(lbl, label_idx(i));
#if UCRTDIFF_REF_HAS__gmtime32
        LOCGM_S_ROW(_gmtime32_s, __time32_t, g_lt32[i], lbl);
        LOCGM_PTR_ROW(_gmtime32, __time32_t, g_lt32[i], lbl);
        LOCGM_S_ROW(_gmtime64_s, __time64_t, g_lt64[i], lbl);
        LOCGM_PTR_ROW(_gmtime64, __time64_t, g_lt64[i], lbl);
#endif
        LOCGM_S_ROW(_localtime32_s, __time32_t, g_lt32[i], lbl);
        LOCGM_PTR_ROW(_localtime32, __time32_t, g_lt32[i], lbl);
        LOCGM_S_ROW(_localtime64_s, __time64_t, g_lt64[i], lbl);
        LOCGM_PTR_ROW(_localtime64, __time64_t, g_lt64[i], lbl);
    }

    /* NULL ptm / NULL time_t: handler-calling EINVAL on both. */
    {
        __time32_t tv = 0;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = _localtime32_s((struct tm *)0, &tv);
        errno_t c2 = ref__localtime32_s((struct tm *)0, &tv);
        grade2("_localtime32_s.nullptm", tzlbl, c1, c2);
        grade2("_localtime32_s.nullptm.errno", tzlbl, *_errno(), *ref__errno());

        struct tm t1, t2;
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        c1 = _localtime32_s(&t1, (const __time32_t *)0);
        c2 = ref__localtime32_s(&t2, (const __time32_t *)0);
        grade2("_localtime32_s.nulltime", tzlbl, c1, c2);
        grade2("_localtime32_s.nulltime.errno", tzlbl, *_errno(), *ref__errno());
    }

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}

static void run_time__localtime32(void)
{
    run_time__localtime_and_gmtime(L"PST8PDT", "PST8PDT");
    run_time__localtime_and_gmtime(L"EST5EDT", "EST5EDT");
    run_time__localtime_and_gmtime(L"UTC0", "UTC0");
}
#endif /* UCRTDIFF_REF_HAS__localtime32 */

/* gmtime.cpp's own guard covers only `__getgmtimebuf` (internal, ungraded);
 * its four exports are graded inside `run_time__localtime32` above (they
 * share the corpus, and TZ-independence means they do not need their own
 * TZ loop) -- but they must still be reachable when localtime.cpp is
 * guarded 0 and gmtime.cpp is guarded 1 (independent deletion order), so a
 * second, TZ-free entry point is provided for that case. */
#if UCRTDIFF_REF_HAS__gmtime32 && !UCRTDIFF_REF_HAS__localtime32
/* ref_ prototypes already declared above, under the unconditional
 * `#if UCRTDIFF_REF_HAS__gmtime32` block. */
static const __time32_t g_gm32_only[] = { 0, 1, 86400, 1000000000L, 0x7fffd27fL, -1 };
#define GM32_ONLY_N (sizeof(g_gm32_only) / sizeof(g_gm32_only[0]))
static const __time64_t g_gm64_only[] = { 0, 1, 86400, 1000000000LL, 2147483648LL, 0x793582affLL, -1 };
#define GM64_ONLY_N (sizeof(g_gm64_only) / sizeof(g_gm64_only[0]))

static void run_time__gmtime32(void)
{
    unsigned i;
    install_counting_iph();
    for (i = 0; i < GM32_ONLY_N; i++) {
        LOCGM_S_ROW(_gmtime32_s, __time32_t, g_gm32_only[i], label_idx(i));
        LOCGM_PTR_ROW(_gmtime32, __time32_t, g_gm32_only[i], label_idx(i));
    }
    for (i = 0; i < GM64_ONLY_N; i++) {
        LOCGM_S_ROW(_gmtime64_s, __time64_t, g_gm64_only[i], label_idx(i));
        LOCGM_PTR_ROW(_gmtime64, __time64_t, g_gm64_only[i], label_idx(i));
    }
}
#endif

/* ==== mktime (time/mktime.cpp) ============================================
 * `tb` is graded BOTH ways: the returned time_t AND the (normalized) `*tb`
 * afterwards -- mktime's job is exactly to fill in/normalize the struct, so
 * grading only the return value would miss the half of the contract
 * `mktime.rs`'s own doc calls out (tm_wday/tm_yday/tm_isdst population,
 * out-of-range tm_mon/tm_mday folding). */
#if UCRTDIFF_REF_HAS__mktime32
__time32_t __cdecl ref__mktime32(struct tm *);
__time64_t __cdecl ref__mktime64(struct tm *);
__time32_t __cdecl ref__mkgmtime32(struct tm *);
__time64_t __cdecl ref__mkgmtime64(struct tm *);
void __cdecl ref__tzset(void);
errno_t __cdecl ref__wputenv_s(const wchar_t *, const wchar_t *);

#define MK_ROW(fn, lbl, ...) do {                                              \
        struct tm t1 = { __VA_ARGS__ }, t2 = { __VA_ARGS__ };                  \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        long long r1 = (long long)fn(&t1);                                    \
        long long r2 = (long long)ref_##fn(&t2);                              \
        grade2(#fn, lbl, r1, r2);                                             \
        grade2(#fn ".tb", lbl, fnv(&t1, sizeof t1), fnv(&t2, sizeof t2));     \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
    } while (0)

static void run_time__mktime32(void)
{
    _wputenv_s(L"TZ", L"PST8PDT"); _tzset();
    ref__wputenv_s(L"TZ", L"PST8PDT"); ref__tzset();
    install_counting_iph();

    /* sec,min,hour,mday,mon,year,wday,yday,isdst -- wday/yday deliberately
     * wrong (99) to prove they get overwritten; isdst=-1 ("let mktime
     * decide") for most, one row each with isdst forced 0 and 1. */
    /* 2024-06-15 12:00:00, normal case */
    MK_ROW(_mktime32, "normal", 0, 0, 12, 15, 5, 124, 99, 99, -1);
    /* Out-of-range month (13 => next Jan) and day (35 => folds forward). */
    MK_ROW(_mktime32, "mon13", 0, 0, 12, 15, 13, 124, 99, 99, -1);
    MK_ROW(_mktime32, "mday35", 0, 0, 12, 35, 0, 124, 99, 99, -1);
    /* Negative month folds the year back. */
    MK_ROW(_mktime32, "monneg1", 0, 0, 12, 15, -1, 124, 99, 99, -1);
    /* Right at a US DST transition (2024-03-10 02:30 local doesn't exist;
     * 2024-11-03 01:30 local is ambiguous) -- isdst forced both ways. */
    MK_ROW(_mktime32, "dst.isdst1", 0, 30, 1, 3, 10, 124, 99, 99, 1);
    MK_ROW(_mktime32, "dst.isdst0", 0, 30, 1, 3, 10, 124, 99, 99, 0);
    /* tm_year far out of range -> EINVAL, -1, *tb untouched. */
    MK_ROW(_mktime32, "yearbad", 0, 0, 0, 1, 0, 100000, 99, 99, -1);

    MK_ROW(_mktime64, "normal", 0, 0, 12, 15, 5, 124, 99, 99, -1);
    MK_ROW(_mktime64, "mon13", 0, 0, 12, 15, 13, 124, 99, 99, -1);

    MK_ROW(_mkgmtime32, "normal", 0, 0, 12, 15, 5, 124, 99, 99, -1);
    MK_ROW(_mkgmtime32, "mon13", 0, 0, 12, 15, 13, 124, 99, 99, -1);
    MK_ROW(_mkgmtime64, "normal", 0, 0, 12, 15, 5, 124, 99, 99, -1);

    /* NULL tb. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        long long r1 = (long long)_mktime32((struct tm *)0);
        long long r2 = (long long)ref__mktime32((struct tm *)0);
        grade2("_mktime32.nulltb", "-", r1, r2);
        grade2("_mktime32.nulltb.errno", "-", *_errno(), *ref__errno());
        grade2("_mktime32.nulltb.iph", "-", g_iph_dll, g_iph_ref);
    }

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}
#endif /* UCRTDIFF_REF_HAS__mktime32 */

/* ==== asctime (time/asctime.cpp) ========================================== */
#if UCRTDIFF_REF_HAS_asctime
errno_t __cdecl ref_asctime_s(char *, size_t, const struct tm *);
errno_t __cdecl ref__wasctime_s(wchar_t *, size_t, const struct tm *);
char *__cdecl ref_asctime(const struct tm *);
wchar_t *__cdecl ref__wasctime(const struct tm *);

/* Tue May 1 02:34:55 1984 -- the exact worked example in the C's own
 * comment; a leap-second (tm_sec=60); a single-digit mday (space-padded,
 * not zero-padded); an invalid field (tm_mon=12) to hit the handler. */
static const struct tm g_at_ok1 = { 55, 34, 2, 1, 4, 84, 2, 121, 0 };
static const struct tm g_at_leapsec = { 60, 59, 23, 31, 11, 124, 2, 365, 0 };
static const struct tm g_at_narrow_day = { 0, 0, 0, 5, 0, 100, 3, 4, 0 };
static const struct tm g_at_badmon = { 0, 0, 0, 1, 12, 100, 0, 0, 0 };

#define AS_S_ROW(fn, CH, tmv, lbl) do {                                        \
        CH b1[40], b2[40];                                                    \
        int _k; for (_k = 0; _k < 40; _k++) { b1[_k] = (CH)'X'; b2[_k] = (CH)'X'; } \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        errno_t c1 = fn(b1, 40, &(tmv));                                      \
        errno_t c2 = ref_##fn(b2, 40, &(tmv));                                \
        grade2(#fn, lbl, c1, c2);                                             \
        grade2(#fn ".val", lbl, fnv(b1, 40 * sizeof(CH)), fnv(b2, 40 * sizeof(CH))); \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                   \
        grade2(#fn ".iph", lbl, g_iph_dll, g_iph_ref);                         \
    } while (0)

static void run_time_asctime_s(void)
{
    install_counting_iph();
    AS_S_ROW(asctime_s, char, g_at_ok1, "ok1");
    AS_S_ROW(asctime_s, char, g_at_leapsec, "leapsec");
    AS_S_ROW(asctime_s, char, g_at_narrow_day, "narrowday");
    AS_S_ROW(asctime_s, char, g_at_badmon, "badmon");
    /* buffer too small (>0, <26). */
    {
        char b1[40], b2[40];
        int k; for (k = 0; k < 40; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = asctime_s(b1, 10, &g_at_ok1);
        errno_t c2 = ref_asctime_s(b2, 10, &g_at_ok1);
        grade2("asctime_s.small", "-", c1, c2);
        grade2("asctime_s.small.val", "-", fnv(b1, 40), fnv(b2, 40));
        grade2("asctime_s.small.errno", "-", *_errno(), *ref__errno());
    }
    /* NULL buffer / NULL tm. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        errno_t c1 = asctime_s((char *)0, 26, &g_at_ok1);
        errno_t c2 = ref_asctime_s((char *)0, 26, &g_at_ok1);
        grade2("asctime_s.nullbuf", "-", c1, c2);
        grade2("asctime_s.nullbuf.errno", "-", *_errno(), *ref__errno());

        char b1[40], b2[40];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        c1 = asctime_s(b1, 26, (const struct tm *)0);
        c2 = ref_asctime_s(b2, 26, (const struct tm *)0);
        grade2("asctime_s.nulltm", "-", c1, c2);
        grade2("asctime_s.nulltm.errno", "-", *_errno(), *ref__errno());
    }
}

static void run_time__wasctime_s(void)
{
    install_counting_iph();
    AS_S_ROW(_wasctime_s, wchar_t, g_at_ok1, "ok1");
    AS_S_ROW(_wasctime_s, wchar_t, g_at_leapsec, "leapsec");
    AS_S_ROW(_wasctime_s, wchar_t, g_at_narrow_day, "narrowday");
}

#define AS_PTR_ROW(fn, CH, WLEN, tmv, lbl) do {                                \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        CH *p1 = fn(&(tmv));                                                  \
        CH *p2 = ref_##fn(&(tmv));                                            \
        grade2(#fn ".isnull", lbl, p1 == 0, p2 == 0);                         \
        if (p1 && p2)                                                        \
            grade2(#fn ".val", lbl, fnv(p1, (WLEN(p1) + 1) * sizeof(CH)),     \
                   fnv(p2, (WLEN(p2) + 1) * sizeof(CH)));                     \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                  \
    } while (0)

static void run_time_asctime(void)
{
    install_counting_iph();
    AS_PTR_ROW(asctime, char, TM_STRLEN, g_at_ok1, "ok1");
    AS_PTR_ROW(asctime, char, TM_STRLEN, g_at_leapsec, "leapsec");
}

static void run_time__wasctime(void)
{
    install_counting_iph();
    AS_PTR_ROW(_wasctime, wchar_t, TM_WCSLEN, g_at_ok1, "ok1");
}
#endif /* UCRTDIFF_REF_HAS_asctime */

/* ==== utime / futime (time/utime.cpp) =====================================
 * Real filesystem side effects, on ONE shared temp file per row, set via
 * each side in turn and read back via `_stat64` (already-exported, from
 * filesystem/stat.cpp -- graded for its own correctness elsewhere; used here
 * only as the oracle). The file is created and deleted by THIS runner, never
 * left behind. */
#if UCRTDIFF_REF_HAS__utime32
void __cdecl ref__tzset(void);
errno_t __cdecl ref__wputenv_s(const wchar_t *, const wchar_t *);
int __cdecl ref__utime32(const char *, struct __utimbuf32 *);
int __cdecl ref__wutime32(const wchar_t *, struct __utimbuf32 *);
int __cdecl ref__utime64(const char *, struct __utimbuf64 *);
int __cdecl ref__wutime64(const wchar_t *, struct __utimbuf64 *);
int __cdecl ref__futime32(int, struct __utimbuf32 *);
int __cdecl ref__futime64(int, struct __utimbuf64 *);

/* The reference instance has its own lowio table: a descriptor the DLL's _open
 * returned means nothing to ref__futime*. Open the reference side's own, from a
 * Win32 handle, through Microsoft's _open_osfhandle -- and the DLL side the same way,
 * so both rows open identically. */
intptr_t __cdecl ref__open_osfhandle(intptr_t, int);
int __cdecl ref__close(int);
static int time_open_rw(const char *p, int ref)
{
    HANDLE h = CreateFileA(p, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, 0,
                           OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (h == INVALID_HANDLE_VALUE) return -1;
    return (int)(ref ? ref__open_osfhandle((intptr_t)h, _O_RDWR | _O_BINARY)
                     : _open_osfhandle((intptr_t)h, _O_RDWR | _O_BINARY));
}

static const char *time_temp_path(void)
{
    static char buf[MAX_PATH];
    GetTempPathA(sizeof buf, buf);
    strcat(buf, "ucrtdiff_time_utime.tmp");
    return buf;
}

/* Sets mtime/atime via `set_fn(path, &times)`, reads them back via
 * `_stat64`, and grades the READ-BACK st_mtime/st_atime -- not the
 * function's own return value alone, since a silently-wrong SetFileTime
 * call would still return 0. */
#define UT_ROW(fn, lbl, ut_at, ut_mt) do {                                     \
        const char *path = time_temp_path();                                   \
        FILE *f = fopen(path, "w"); if (f) fclose(f);                          \
        struct __utimbuf32 t1; t1.actime = (ut_at); t1.modtime = (ut_mt);      \
        *_errno() = 0; g_iph_dll = 0;                                          \
        int r1 = fn(path, &t1);                                                \
        int e1 = *_errno();                                                    \
        struct _stat64 st1; memset(&st1, 0, sizeof st1);                       \
        _stat64(path, &st1);                                                   \
        remove(path);                                                          \
        f = fopen(path, "w"); if (f) fclose(f);                                \
        struct __utimbuf32 t2; t2.actime = (ut_at); t2.modtime = (ut_mt);      \
        *ref__errno() = 0; g_iph_ref = 0;                                      \
        int r2 = ref_##fn(path, &t2);                                          \
        int e2 = *ref__errno();                                                \
        struct _stat64 st2; memset(&st2, 0, sizeof st2);                       \
        _stat64(path, &st2);                                                   \
        remove(path);                                                          \
        grade2(#fn, lbl, r1, r2);                                              \
        grade2(#fn ".errno", lbl, e1, e2);                                     \
        grade2(#fn ".mtime", lbl, (long long)st1.st_mtime, (long long)st2.st_mtime); \
        grade2(#fn ".atime", lbl, (long long)st1.st_atime, (long long)st2.st_atime); \
    } while (0)

static void run_time__utime32(void)
{
    install_counting_iph();
    _wputenv_s(L"TZ", L"PST8PDT"); _tzset();
    ref__wputenv_s(L"TZ", L"PST8PDT"); ref__tzset();

    UT_ROW(_utime32, "normal", 1700000000L, 1700003600L);
    /* NULL times -> both fields set to "now" (one _time32 call, reused for
     * both) -- graded structurally (mtime == atime), not by exact value. */
    {
        const char *path = time_temp_path();
        FILE *f = fopen(path, "w"); if (f) fclose(f);
        *_errno() = 0;
        int r1 = _utime32(path, (struct __utimbuf32 *)0);
        struct _stat64 st1; memset(&st1, 0, sizeof st1);
        _stat64(path, &st1);
        remove(path);
        f = fopen(path, "w"); if (f) fclose(f);
        int r2 = ref__utime32(path, (struct __utimbuf32 *)0);
        struct _stat64 st2; memset(&st2, 0, sizeof st2);
        _stat64(path, &st2);
        remove(path);
        /* Compared across the two sides, not asserted: FAT keeps the access time
         * to the DAY, so "mtime == atime" is false on this volume for both. */
        grade2("_utime32.nulltimes.samefields", "-", (long long)(st1.st_mtime == st1.st_atime),
               (long long)(st2.st_mtime == st2.st_atime));
        grade2("_utime32.nulltimes.ret", "-", r1, r2);
    }
    /* NULL filename -> EINVAL, handler called. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        int r1 = _utime32((const char *)0, (struct __utimbuf32 *)0);
        int r2 = ref__utime32((const char *)0, (struct __utimbuf32 *)0);
        grade2("_utime32.nullname", "-", r1, r2);
        grade2("_utime32.nullname.errno", "-", *_errno(), *ref__errno());
        grade2("_utime32.nullname.iph", "-", g_iph_dll, g_iph_ref);
    }

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}

static void run_time__futime32(void)
{
    const char *path = time_temp_path();
    install_counting_iph();
    _wputenv_s(L"TZ", L"PST8PDT"); _tzset();
    ref__wputenv_s(L"TZ", L"PST8PDT"); ref__tzset();

    {
        FILE *f = fopen(path, "w"); if (f) fclose(f);
        int fh1 = time_open_rw(path, 0);
        struct __utimbuf32 t1; t1.actime = 1700000000L; t1.modtime = 1700003600L;
        *_errno() = 0;
        int r1 = _futime32(fh1, &t1);
        int e1 = *_errno();
        struct _stat64 st1; memset(&st1, 0, sizeof st1);
        _stat64(path, &st1);
        _close(fh1);
        remove(path);

        f = fopen(path, "w"); if (f) fclose(f);
        int fh2 = time_open_rw(path, 1);
        struct __utimbuf32 t2; t2.actime = 1700000000L; t2.modtime = 1700003600L;
        *ref__errno() = 0;
        int r2 = ref__futime32(fh2, &t2);
        int e2 = *ref__errno();
        struct _stat64 st2; memset(&st2, 0, sizeof st2);
        _stat64(path, &st2);
        ref__close(fh2);
        remove(path);

        grade2("_futime32", "-", r1, r2);
        grade2("_futime32.errno", "-", e1, e2);
        grade2("_futime32.mtime", "-", (long long)st1.st_mtime, (long long)st2.st_mtime);
        grade2("_futime32.atime", "-", (long long)st1.st_atime, (long long)st2.st_atime);
    }
    /* Bad descriptor -> EBADF, handler called. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        int r1 = _futime32(-2, (struct __utimbuf32 *)0);
        int r2 = ref__futime32(-2, (struct __utimbuf32 *)0);
        grade2("_futime32.badfh", "-", r1, r2);
        grade2("_futime32.badfh.errno", "-", *_errno(), *ref__errno());
        grade2("_futime32.badfh.iph", "-", g_iph_dll, g_iph_ref);
    }

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}
#endif /* UCRTDIFF_REF_HAS__utime32 */

/* ==== ftime (time/ftime.cpp) ===============================================
 * `tp->time`/`tp->millitm` are wall-clock and NOT graded by value (the two
 * calls are microseconds apart at best); `tp->timezone`/`tp->dstflag` ARE
 * deterministic given a fixed TZ and are graded exactly. */
#if UCRTDIFF_REF_HAS__ftime32
errno_t __cdecl ref__ftime32_s(struct __timeb32 *);
void __cdecl ref__ftime32(struct __timeb32 *);
errno_t __cdecl ref__ftime64_s(struct __timeb64 *);
void __cdecl ref__ftime64(struct __timeb64 *);
void __cdecl ref__tzset(void);
errno_t __cdecl ref__wputenv_s(const wchar_t *, const wchar_t *);

static void run_time__ftime32_s(void)
{
    struct __timeb32 t1, t2;
    install_counting_iph();
    _wputenv_s(L"TZ", L"PST8PDT"); _tzset();
    ref__wputenv_s(L"TZ", L"PST8PDT"); ref__tzset();

    memset(&t1, 0xAB, sizeof t1); memset(&t2, 0xAB, sizeof t2);
    *_errno() = 0; *ref__errno() = 0;
    errno_t c1 = _ftime32_s(&t1);
    errno_t c2 = ref__ftime32_s(&t2);
    grade2("_ftime32_s", "-", c1, c2);
    grade2("_ftime32_s.timezone", "-", t1.timezone, t2.timezone);
    grade2("_ftime32_s.dstflag", "-", t1.dstflag, t2.dstflag);
    grade2("_ftime32_s.errno", "-", *_errno(), *ref__errno());

    /* NULL tp -> EINVAL, handler called. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _ftime32_s((struct __timeb32 *)0);
    c2 = ref__ftime32_s((struct __timeb32 *)0);
    grade2("_ftime32_s.null", "-", c1, c2);
    grade2("_ftime32_s.null.errno", "-", *_errno(), *ref__errno());
    grade2("_ftime32_s.null.iph", "-", g_iph_dll, g_iph_ref);

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}

static void run_time__ftime32(void)
{
    struct __timeb32 t1, t2;
    install_counting_iph();
    memset(&t1, 0, sizeof t1); memset(&t2, 0, sizeof t2);
    _ftime32(&t1);
    ref__ftime32(&t2);
    /* `_ftime32` has no return value; sanity only -- both report a
     * plausible (non-zero, matching order of magnitude) epoch second. */
    grade2("_ftime32.nonzero", "-", t1.time != 0, t2.time != 0);
}

static void run_time__ftime64_s(void)
{
    struct __timeb64 t1, t2;
    install_counting_iph();
    _wputenv_s(L"TZ", L"EST5EDT"); _tzset();
    ref__wputenv_s(L"TZ", L"EST5EDT"); ref__tzset();

    memset(&t1, 0xAB, sizeof t1); memset(&t2, 0xAB, sizeof t2);
    *_errno() = 0; *ref__errno() = 0;
    errno_t c1 = _ftime64_s(&t1);
    errno_t c2 = ref__ftime64_s(&t2);
    grade2("_ftime64_s", "-", c1, c2);
    grade2("_ftime64_s.timezone", "-", t1.timezone, t2.timezone);
    grade2("_ftime64_s.dstflag", "-", t1.dstflag, t2.dstflag);

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}
#endif /* UCRTDIFF_REF_HAS__ftime32 */

/* ==== strdate / strtime (time/strdate.cpp, time/strtime.cpp) ==============
 * Not graded by exact VALUE (the local date/time can differ between the two
 * calls, and races a midnight/minute boundary): graded by FORMAT invariant
 * (fixed-position digits and separators) plus the small-buffer/NULL
 * contract, which is what strdate.rs/strtime.rs actually changed.
 *
 * `is_digit_ch` is shared by both blocks below (and so defined
 * unconditionally, outside either guard) -- strdate.cpp and strtime.cpp are
 * independently deletable, so a guard-1-only definition would leave
 * `date_shape`/`time_shape` referencing it whenever only ONE of the two
 * guards is set. */
static int is_digit_ch(int c) { return c >= '0' && c <= '9'; }

#if UCRTDIFF_REF_HAS__strdate
errno_t __cdecl ref__strdate_s(char *, size_t);
errno_t __cdecl ref__wstrdate_s(wchar_t *, size_t);
char *__cdecl ref__strdate(char *);
wchar_t *__cdecl ref__wstrdate(wchar_t *);

/* "MM/DD/YY\0": digit,digit,'/',digit,digit,'/',digit,digit,'\0'. */
static long long date_shape(const char *b)
{
    return (long long)(is_digit_ch(b[0]) && is_digit_ch(b[1]) && b[2] == '/' &&
                        is_digit_ch(b[3]) && is_digit_ch(b[4]) && b[5] == '/' &&
                        is_digit_ch(b[6]) && is_digit_ch(b[7]) && b[8] == 0);
}

static void run_time__strdate_s(void)
{
    char b1[16], b2[16];
    install_counting_iph();
    memset(b1, 'X', sizeof b1); memset(b2, 'X', sizeof b2);
    *_errno() = 0; *ref__errno() = 0;
    errno_t c1 = _strdate_s(b1, 9);
    errno_t c2 = ref__strdate_s(b2, 9);
    grade2("_strdate_s", "-", c1, c2);
    grade2("_strdate_s.shape", "-", date_shape(b1), date_shape(b2));
    grade2("_strdate_s.errno", "-", *_errno(), *ref__errno());

    /* Too small (>0, <9): NUL-reset buffer[0], ERANGE. */
    memset(b1, 'X', sizeof b1); memset(b2, 'X', sizeof b2);
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _strdate_s(b1, 4);
    c2 = ref__strdate_s(b2, 4);
    grade2("_strdate_s.small", "-", c1, c2);
    grade2("_strdate_s.small.buf0", "-", b1[0], b2[0]);
    grade2("_strdate_s.small.errno", "-", *_errno(), *ref__errno());
    grade2("_strdate_s.small.iph", "-", g_iph_dll, g_iph_ref);

    /* NULL buffer -> EINVAL, handler. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _strdate_s((char *)0, 9);
    c2 = ref__strdate_s((char *)0, 9);
    grade2("_strdate_s.null", "-", c1, c2);
    grade2("_strdate_s.null.errno", "-", *_errno(), *ref__errno());
    grade2("_strdate_s.null.iph", "-", g_iph_dll, g_iph_ref);
}

static void run_time__strdate(void)
{
    char b1[16], b2[16];
    install_counting_iph();
    char *p1 = _strdate(b1);
    char *p2 = ref__strdate(b2);
    grade2("_strdate.isnull", "-", p1 == 0, p2 == 0);
    grade2("_strdate.shape", "-", date_shape(b1), date_shape(b2));
}
#endif /* UCRTDIFF_REF_HAS__strdate */

#if UCRTDIFF_REF_HAS__strtime
errno_t __cdecl ref__strtime_s(char *, size_t);
char *__cdecl ref__strtime(char *);

static long long time_shape(const char *b)
{
    return (long long)(is_digit_ch(b[0]) && is_digit_ch(b[1]) && b[2] == ':' &&
                        is_digit_ch(b[3]) && is_digit_ch(b[4]) && b[5] == ':' &&
                        is_digit_ch(b[6]) && is_digit_ch(b[7]) && b[8] == 0);
}

static void run_time__strtime_s(void)
{
    char b1[16], b2[16];
    install_counting_iph();
    memset(b1, 'X', sizeof b1); memset(b2, 'X', sizeof b2);
    *_errno() = 0; *ref__errno() = 0;
    errno_t c1 = _strtime_s(b1, 9);
    errno_t c2 = ref__strtime_s(b2, 9);
    grade2("_strtime_s", "-", c1, c2);
    grade2("_strtime_s.shape", "-", time_shape(b1), time_shape(b2));
    grade2("_strtime_s.errno", "-", *_errno(), *ref__errno());

    memset(b1, 'X', sizeof b1); memset(b2, 'X', sizeof b2);
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _strtime_s(b1, 4);
    c2 = ref__strtime_s(b2, 4);
    grade2("_strtime_s.small", "-", c1, c2);
    grade2("_strtime_s.small.buf0", "-", b1[0], b2[0]);
    grade2("_strtime_s.small.errno", "-", *_errno(), *ref__errno());
}

static void run_time__strtime(void)
{
    char b1[16], b2[16];
    install_counting_iph();
    char *p1 = _strtime(b1);
    char *p2 = ref__strtime(b2);
    grade2("_strtime.isnull", "-", p1 == 0, p2 == 0);
    grade2("_strtime.shape", "-", time_shape(b1), time_shape(b2));
}
#endif /* UCRTDIFF_REF_HAS__strtime */

/* ==== time / timespec_get (time/time.cpp) ==================================
 * Per the batch brief, NOT graded by value (wall-clock): argument/errno
 * contract, and sanity (non-negative, roughly "now", monotonic across the
 * two back-to-back calls within a few seconds' slop). */
#if UCRTDIFF_REF_HAS__time32
__time32_t __cdecl ref__time32(__time32_t *);
__time64_t __cdecl ref__time64(__time64_t *);
int __cdecl ref__timespec32_get(struct _timespec32 *, int);
int __cdecl ref__timespec64_get(struct _timespec64 *, int);

static void run_time__time32(void)
{
    __time32_t r1v, r2v;
    __time32_t r1 = _time32(&r1v);
    __time32_t r2 = ref__time32(&r2v);
    install_counting_iph();
    grade2("_time32.nonneg", "-", r1 >= 0, 1);
    grade2("_time32.nonneg", "-", r2 >= 0, 1);
    grade2("_time32.outparam_matches_return", "-", r1 == r1v, 1);
    grade2("_time32.outparam_matches_return", "-", r2 == r2v, 1);
    /* "roughly now": within a day of each other -- catches a gross unit
     * error (e.g. milliseconds instead of seconds) without pinning wall
     * clock. */
    {
        long long diff = (long long)r1 - (long long)r2;
        if (diff < 0) diff = -diff;
        grade2("_time32.agrees_with_ref_roughly", "-", diff < 86400, 1);
    }
    /* NULL result: still returns the time, no crash. */
    __time32_t r3 = _time32((__time32_t *)0);
    grade2("_time32.null_ok", "-", r3 >= 0, 1);
}

static void run_time__time64(void)
{
    __time64_t r1v, r2v;
    __time64_t r1 = _time64(&r1v);
    __time64_t r2 = ref__time64(&r2v);
    grade2("_time64.nonneg", "-", r1 >= 0, 1);
    grade2("_time64.nonneg", "-", r2 >= 0, 1);
    grade2("_time64.outparam_matches_return", "-", (long long)(r1 == r1v), 1);
    grade2("_time64.outparam_matches_return", "-", (long long)(r2 == r2v), 1);
}

static void run_time__timespec32_get(void)
{
    struct _timespec32 ts1, ts2;
    memset(&ts1, 0, sizeof ts1); memset(&ts2, 0, sizeof ts2);
    int b1 = _timespec32_get(&ts1, 1 /* TIME_UTC */);
    int b2 = ref__timespec32_get(&ts2, 1);
    grade2("_timespec32_get.base", "-", b1, b2);
    grade2("_timespec32_get.nsec_range", "-", ts1.tv_nsec >= 0 && ts1.tv_nsec < 1000000000L, 1);
    grade2("_timespec32_get.nsec_range", "-", ts2.tv_nsec >= 0 && ts2.tv_nsec < 1000000000L, 1);
    /* Unsupported base: returns 0, `*ts` untouched. */
    memset(&ts1, 0xAB, sizeof ts1); memset(&ts2, 0xAB, sizeof ts2);
    b1 = _timespec32_get(&ts1, 999);
    b2 = ref__timespec32_get(&ts2, 999);
    grade2("_timespec32_get.badbase", "-", b1, b2);
    grade2("_timespec32_get.badbase.untouched", "-", fnv(&ts1, sizeof ts1), fnv(&ts2, sizeof ts2));
    /* NULL ts -> EINVAL, handler, returns 0. */
    install_counting_iph();
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    b1 = _timespec32_get((struct _timespec32 *)0, 1);
    b2 = ref__timespec32_get((struct _timespec32 *)0, 1);
    grade2("_timespec32_get.null", "-", b1, b2);
    grade2("_timespec32_get.null.errno", "-", *_errno(), *ref__errno());
    grade2("_timespec32_get.null.iph", "-", g_iph_dll, g_iph_ref);
}
#endif /* UCRTDIFF_REF_HAS__time32 */

/* ==== clock (time/clock.cpp) ================================================
 * Per the batch brief, NOT graded by value: sanity (non-negative,
 * monotonic non-decreasing across two back-to-back calls) only. */
#if UCRTDIFF_REF_HAS_clock
clock_t __cdecl ref_clock(void);

static void run_time_clock(void)
{
    clock_t a1 = clock(), a2 = clock();
    clock_t b1 = ref_clock(), b2 = ref_clock();
    grade2("clock.nonneg", "-", a1 >= 0, 1);
    grade2("clock.nonneg", "-", b1 >= 0, 1);
    grade2("clock.monotonic", "-", a2 >= a1, 1);
    grade2("clock.monotonic", "-", b2 >= b1, 1);
}
#endif /* UCRTDIFF_REF_HAS_clock */

/* ==== difftime (time/difftime.cpp) =========================================
 * Fully deterministic given two time_t inputs -- graded exactly. */
#if UCRTDIFF_REF_HAS__difftime32
double __cdecl ref__difftime32(__time32_t, __time32_t);
double __cdecl ref__difftime64(__time64_t, __time64_t);

static unsigned long long dt_bits(double d) { unsigned long long u; memcpy(&u, &d, 8); return u; }

#define DT_ROW(fn, lbl, b, a) do {                                             \
        *_errno() = 0; *ref__errno() = 0;                                     \
        double v1 = fn((b), (a));                                             \
        double v2 = ref_##fn((b), (a));                                       \
        grade2(#fn, lbl, (long long)dt_bits(v1), (long long)dt_bits(v2));     \
        grade2(#fn ".errno", lbl, *_errno(), *ref__errno());                  \
    } while (0)

static void run_time__difftime32(void)
{
    install_counting_iph();
    DT_ROW(_difftime32, "normal", 1000, 100);
    DT_ROW(_difftime32, "zero", 500, 500);
    DT_ROW(_difftime32, "b<a", 100, 1000);
    /* Either negative -> EINVAL (no handler), returns 0.0. */
    DT_ROW(_difftime32, "aneg", 1000, -1);
    DT_ROW(_difftime32, "bneg", -1, 1000);
    DT_ROW(_difftime32, "bothneg", -5, -1);

    DT_ROW(_difftime64, "normal", 100000000000LL, 1);
    DT_ROW(_difftime64, "aneg", 1000, -1);
}
#endif /* UCRTDIFF_REF_HAS__difftime32 */

/* 2024-06-15 (Saturday), 13:45:30, yday=165 (0-based), not DST. Shared by
 * both the wcsftime and strftime blocks below (independently deletable, so
 * defined unconditionally, outside either guard -- same reasoning as
 * `is_digit_ch` above). */
static const struct tm g_wf_base = { 30, 45, 13, 15, 5, 124, 6, 165, 0 };

/* ==== wcsftime (time/wcsftime.cpp) =========================================
 * `_Wcsftime_l`/`_Getdays_l`/`_Getmonths_l` (the `_l` twins with the extra
 * `lc_time_arg` parameter) are NOT in ucrtbase.spec -- confirmed absent from
 * `libucrt_ref.a` on all three arches with `llvm-nm` (unlike every other
 * name graded in this file) -- so only the exported wrappers
 * (`wcsftime`/`_wcsftime_l`/`_Wcsftime`, `_W_Getdays`/`_W_Getmonths`/
 * `_W_Gettnames`) are graded here; `_Wcsftime_l` itself is exercised only
 * indirectly, through them. */
#if UCRTDIFF_REF_HAS_wcsftime
size_t __cdecl ref_wcsftime(wchar_t *, size_t, const wchar_t *, const struct tm *);
size_t __cdecl ref__wcsftime_l(wchar_t *, size_t, const wchar_t *, const struct tm *, _locale_t);
size_t __cdecl ref__Wcsftime(wchar_t *, size_t, const wchar_t *, const struct tm *, void *);
wchar_t *__cdecl ref__W_Getdays(void);
wchar_t *__cdecl ref__W_Getmonths(void);
void *__cdecl ref__W_Gettnames(void);

/* `_W_Getdays`/`_W_Getmonths`/`_W_Gettnames` are undocumented internal
 * helpers -- absent from every public mingw header (unlike every other name
 * in this file), so the DLL side cannot be reached by a plain call at all;
 * import pointers, same mechanism `UD_IMP` uses for a mingw-inlined name. */
extern wchar_t *(__cdecl *const ud__W_Getdays)(void) __asm__(UD_IMP(_W_Getdays));
extern wchar_t *(__cdecl *const ud__W_Getmonths)(void) __asm__(UD_IMP(_W_Getmonths));
extern void *(__cdecl *const ud__W_Gettnames)(void) __asm__(UD_IMP(_W_Gettnames));

#define WFBUF 64

/* Prefilled with 'X' so an untouched tail (a too-small buffer, or a failed
 * call leaving the C's own `*string = L'\0'` as the only write) shows
 * plainly against genuinely-formatted output. */
#define WF_ROW(lbl, fmt, tmv, size) do {                                       \
        wchar_t b1[WFBUF], b2[WFBUF];                                          \
        for (int _k = 0; _k < WFBUF; _k++) { b1[_k] = L'X'; b2[_k] = L'X'; }   \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        size_t r1 = wcsftime(b1, (size), (fmt), &(tmv));                      \
        size_t r2 = ref_wcsftime(b2, (size), (fmt), &(tmv));                  \
        grade2("wcsftime", (lbl), (long long)r1, (long long)r2);              \
        grade2("wcsftime.val", (lbl), fnv(b1, sizeof b1), fnv(b2, sizeof b2)); \
        grade2("wcsftime.errno", (lbl), *_errno(), *ref__errno());            \
        grade2("wcsftime.iph", (lbl), g_iph_dll, g_iph_ref);                  \
    } while (0)

/* `g_wf_base` is defined unconditionally, above. */

static void run_time__Wcsftime_l(void)
{
    install_counting_iph();

    /* Explicitly force the "C" locale first, so this runner's result does
     * not depend on whatever an earlier/later runner in the sweep left
     * active -- guarded the same as the en-US sub-test below, since
     * `ud_setlocale`/`ref_setlocale` (the import-pointer/ref_ pair) are
     * declared only inside that guard, further up this translation unit. */
#if UCRTDIFF_REF_CALLABLE_setlocale
    ud_setlocale(LC_ALL, "C");
    ref_setlocale(LC_ALL, "C");
#endif

    /* Every specifier, with and without '#'. `%q` (not a specifier) is
     * appended as the one deliberately-invalid case. */
    static const wchar_t specs[] = L"aAbBcCdDeFgGhHIjmMnprRStTuUVwWxXyYzZ%q";
    for (size_t i = 0; i < sizeof(specs) / sizeof(specs[0]) - 1; i++) {
        wchar_t fmt[4];
        fmt[0] = L'%'; fmt[1] = specs[i]; fmt[2] = 0;
        WF_ROW(label_idx(i), fmt, g_wf_base, WFBUF);

        wchar_t fmt_alt[4];
        fmt_alt[0] = L'%'; fmt_alt[1] = L'#'; fmt_alt[2] = specs[i]; fmt_alt[3] = 0;
        WF_ROW(label_idx(i), fmt_alt, g_wf_base, WFBUF);
    }

    /* All 7 weekdays (%a %A %u %w) and all 12 months (%b %B). */
    for (int wd = 0; wd < 7; wd++) {
        struct tm t = g_wf_base;
        t.tm_wday = wd;
        WF_ROW(label_idx((unsigned)wd), L"%a %A %u %w", t, WFBUF);
    }
    for (int mo = 0; mo < 12; mo++) {
        struct tm t = g_wf_base;
        t.tm_mon = mo;
        WF_ROW(label_idx((unsigned)mo), L"%b %B", t, WFBUF);
    }

    /* tm fields out of range, one per specifier that validates it. */
    {
        struct tm t = g_wf_base; t.tm_wday = 7;
        WF_ROW("badwday", L"%a", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_mon = 12;
        WF_ROW("badmon", L"%b", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_mday = 0;
        WF_ROW("badmday.lo", L"%d", t, WFBUF);
        t.tm_mday = 32;
        WF_ROW("badmday.hi", L"%d", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_hour = -1;
        WF_ROW("badhour.lo", L"%H", t, WFBUF);
        t.tm_hour = 24;
        WF_ROW("badhour.hi", L"%H", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_min = -1;
        WF_ROW("badmin.lo", L"%M", t, WFBUF);
        t.tm_min = 60;
        WF_ROW("badmin.hi", L"%M", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_sec = -1;
        WF_ROW("badsec.lo", L"%S", t, WFBUF);
        t.tm_sec = 61;
        WF_ROW("badsec.hi", L"%S", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_year = -1901;
        WF_ROW("badyear.lo", L"%Y", t, WFBUF);
        t.tm_year = 8100;
        WF_ROW("badyear.hi", L"%Y", t, WFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_yday = -1;
        WF_ROW("badyday.lo", L"%j", t, WFBUF);
        t.tm_yday = 366;
        WF_ROW("badyday.hi", L"%j", t, WFBUF);
    }

    /* Too-small buffer: "%A" (full weekday name, "Saturday", 8 chars + NUL)
     * into a 3-wchar_t buffer -- expect 0, ERANGE, no handler call, and
     * `*string == L'\0'`. */
    {
        wchar_t b1[WFBUF], b2[WFBUF];
        for (int k = 0; k < WFBUF; k++) { b1[k] = L'X'; b2[k] = L'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = wcsftime(b1, 3, L"%A", &g_wf_base);
        size_t r2 = ref_wcsftime(b2, 3, L"%A", &g_wf_base);
        grade2("wcsftime.small", "-", (long long)r1, (long long)r2);
        grade2("wcsftime.small.buf0", "-", b1[0], b2[0]);
        grade2("wcsftime.small.errno", "-", *_errno(), *ref__errno());
        grade2("wcsftime.small.iph", "-", g_iph_dll, g_iph_ref);
    }

    /* Invalid specifier standalone (redundant with the sweep's "%q" row,
     * kept explicit for clarity): handler-calling EINVAL, buffer reset. */
    WF_ROW("invalidspec", L"%q", g_wf_base, WFBUF);

    /* NULL string / NULL format / zero max_size. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = wcsftime((wchar_t *)0, WFBUF, L"%Y", &g_wf_base);
        size_t r2 = ref_wcsftime((wchar_t *)0, WFBUF, L"%Y", &g_wf_base);
        grade2("wcsftime.nullstring", "-", (long long)r1, (long long)r2);
        grade2("wcsftime.nullstring.errno", "-", *_errno(), *ref__errno());
        grade2("wcsftime.nullstring.iph", "-", g_iph_dll, g_iph_ref);
    }
    {
        wchar_t b1[WFBUF], b2[WFBUF];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = wcsftime(b1, WFBUF, (const wchar_t *)0, &g_wf_base);
        size_t r2 = ref_wcsftime(b2, WFBUF, (const wchar_t *)0, &g_wf_base);
        grade2("wcsftime.nullformat", "-", (long long)r1, (long long)r2);
        grade2("wcsftime.nullformat.errno", "-", *_errno(), *ref__errno());
        grade2("wcsftime.nullformat.iph", "-", g_iph_dll, g_iph_ref);
    }
    {
        wchar_t b1[WFBUF], b2[WFBUF];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = wcsftime(b1, 0, L"%Y", &g_wf_base);
        size_t r2 = ref_wcsftime(b2, 0, L"%Y", &g_wf_base);
        grade2("wcsftime.zerosize", "-", (long long)r1, (long long)r2);
        grade2("wcsftime.zerosize.errno", "-", *_errno(), *ref__errno());
        grade2("wcsftime.zerosize.iph", "-", g_iph_dll, g_iph_ref);
    }
    /* NULL timeptr with a format that has NO '%' directive: succeeds on
     * both sides (validated lazily, only once a directive is reached --
     * see wcsftime.rs's module doc). */
    {
        wchar_t b1[WFBUF], b2[WFBUF];
        *_errno() = 0; *ref__errno() = 0;
        size_t r1 = wcsftime(b1, WFBUF, L"no percent signs", (const struct tm *)0);
        size_t r2 = ref_wcsftime(b2, WFBUF, L"no percent signs", (const struct tm *)0);
        grade2("wcsftime.nulltm_no_directive", "-", (long long)r1, (long long)r2);
        grade2("wcsftime.nulltm_no_directive.val", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));
    }

    /* _W_Getdays / _W_Getmonths / _W_Gettnames: grade the returned blob's
     * content (nul-terminated for the first two; `_W_Gettnames`'s result is
     * an opaque `__crt_lc_time_data*`, graded field-by-field via the SAME
     * pointers `wcsftime` itself reads, since the two sides' allocations
     * never share an address). */
    {
        wchar_t *p1 = ud__W_Getdays();
        wchar_t *p2 = ref__W_Getdays();
        grade2("_W_Getdays.isnull", "-", p1 == 0, p2 == 0);
        if (p1 && p2)
            grade2("_W_Getdays.val", "-", fnv(p1, (wcslen(p1) + 1) * sizeof(wchar_t)), fnv(p2, (wcslen(p2) + 1) * sizeof(wchar_t)));
    }
    {
        wchar_t *p1 = ud__W_Getmonths();
        wchar_t *p2 = ref__W_Getmonths();
        grade2("_W_Getmonths.isnull", "-", p1 == 0, p2 == 0);
        if (p1 && p2)
            grade2("_W_Getmonths.val", "-", fnv(p1, (wcslen(p1) + 1) * sizeof(wchar_t)), fnv(p2, (wcslen(p2) + 1) * sizeof(wchar_t)));
    }
    {
        void *p1 = ud__W_Gettnames();
        void *p2 = ref__W_Gettnames();
        grade2("_W_Gettnames.isnull", "-", p1 == 0, p2 == 0);
        /* Field-by-field via the SAME `__crt_lc_time_data` layout the DLL's
         * own `_Wcsftime_l` reads (`<time.h>`/`corecrt_internal_time.h`
         * declare no such struct publicly, so this reads it as the
         * documented ABI shape: 7+7+12+12+2 narrow pointers, `ww_*` format
         * pointers, `ww_caltype`, `refcount`, then the wide mirror). Grading
         * every wide day/month name's CONTENT (not the blob's raw bytes,
         * which embed pointers that never match across two allocations). */
        if (p1 && p2) {
            struct gettnames_shape {
                char *wday_abbr[7], *wday[7], *month_abbr[12], *month[12], *ampm[2];
                char *ww_sdatefmt, *ww_ldatefmt, *ww_timefmt;
                int ww_caltype; long refcount;
                wchar_t *w_wday_abbr[7], *w_wday[7], *w_month_abbr[12], *w_month[12], *w_ampm[2];
                wchar_t *w_ww_sdatefmt, *w_ww_ldatefmt, *w_ww_timefmt, *w_ww_locale_name;
            };
            struct gettnames_shape *g1 = (struct gettnames_shape *)p1;
            struct gettnames_shape *g2 = (struct gettnames_shape *)p2;
            for (int i = 0; i < 7; i++)
                grade2("_W_Gettnames.wday", label_idx((unsigned)i), fnv(g1->w_wday[i], (wcslen(g1->w_wday[i]) + 1) * sizeof(wchar_t)), fnv(g2->w_wday[i], (wcslen(g2->w_wday[i]) + 1) * sizeof(wchar_t)));
            for (int i = 0; i < 12; i++)
                grade2("_W_Gettnames.month", label_idx((unsigned)i), fnv(g1->w_month[i], (wcslen(g1->w_month[i]) + 1) * sizeof(wchar_t)), fnv(g2->w_month[i], (wcslen(g2->w_month[i]) + 1) * sizeof(wchar_t)));
            grade2("_W_Gettnames.caltype", "-", g1->ww_caltype, g2->ww_caltype);
        }
    }

    /* One non-C locale, on BOTH sides, restoring "C" afterward -- only
     * attempted when `setlocale`'s own block is graded (guards the
     * `ud_setlocale`/`ref_setlocale` declarations, which live inside THAT
     * guard, further up this same translation unit). "en-US" is a real
     * Windows locale name every target Windows/ReactOS install recognizes;
     * if `setlocale` fails to switch (returns NULL) on either side, the
     * whole sub-test is skipped rather than grading a locale that never
     * actually changed. */
#if UCRTDIFF_REF_CALLABLE_setlocale
    {
        char *s1 = ud_setlocale(LC_ALL, "en-US");
        char *s2 = ref_setlocale(LC_ALL, "en-US");
        if (s1 != 0 && s2 != 0) {
            WF_ROW("enUS.c", L"%c", g_wf_base, WFBUF);
            WF_ROW("enUS.x", L"%x", g_wf_base, WFBUF);
            WF_ROW("enUS.X", L"%X", g_wf_base, WFBUF);
            WF_ROW("enUS.A", L"%A", g_wf_base, WFBUF);
            WF_ROW("enUS.B", L"%B", g_wf_base, WFBUF);
        }
        ud_setlocale(LC_ALL, "C");
        ref_setlocale(LC_ALL, "C");
    }
#endif
}
#endif /* UCRTDIFF_REF_HAS_wcsftime */

/* ==== strftime (time/strftime.cpp) ==========================================
 * The narrow wrapper: same corpus as `wcsftime` above, converted through
 * the "C" locale's codepage both ways -- graded by BUFFER CONTENT (`fnv`),
 * not by re-deriving the expected string, since the narrow<->wide<->narrow
 * round trip is exactly what `strftime.rs` added (`wcsftime.rs`'s engine
 * itself is graded independently, above). `_Strftime_l`/`_Getdays_l`/
 * `_Getmonths_l` are, like their wide twins, not in ucrtbase.spec -- only
 * the exported wrappers are graded. */
#if UCRTDIFF_REF_HAS_strftime
size_t __cdecl ref_strftime(char *, size_t, const char *, const struct tm *);
size_t __cdecl ref__strftime_l(char *, size_t, const char *, const struct tm *, _locale_t);
size_t __cdecl ref__Strftime(char *, size_t, const char *, const struct tm *, void *);
char *__cdecl ref__Getdays(void);
char *__cdecl ref__Getmonths(void);
void *__cdecl ref__Gettnames(void);

/* Same reasoning as `ud__W_Getdays` above: undocumented, no public header. */
extern char *(__cdecl *const ud__Getdays)(void) __asm__(UD_IMP(_Getdays));
extern char *(__cdecl *const ud__Getmonths)(void) __asm__(UD_IMP(_Getmonths));
extern void *(__cdecl *const ud__Gettnames)(void) __asm__(UD_IMP(_Gettnames));

#define SFBUF 64

#define SF_ROW(lbl, fmt, tmv, size) do {                                       \
        char b1[SFBUF], b2[SFBUF];                                            \
        for (int _k = 0; _k < SFBUF; _k++) { b1[_k] = 'X'; b2[_k] = 'X'; }     \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        size_t r1 = strftime(b1, (size), (fmt), &(tmv));                      \
        size_t r2 = ref_strftime(b2, (size), (fmt), &(tmv));                  \
        grade2("strftime", (lbl), (long long)r1, (long long)r2);              \
        grade2("strftime.val", (lbl), fnv(b1, sizeof b1), fnv(b2, sizeof b2)); \
        grade2("strftime.errno", (lbl), *_errno(), *ref__errno());            \
        grade2("strftime.iph", (lbl), g_iph_dll, g_iph_ref);                  \
    } while (0)

static void run_time__Strftime_l(void)
{
    install_counting_iph();
#if UCRTDIFF_REF_CALLABLE_setlocale
    ud_setlocale(LC_ALL, "C");
    ref_setlocale(LC_ALL, "C");
#endif

    static const char specs[] = "aAbBcCdDeFgGhHIjmMnprRStTuUVwWxXyYzZ%q";
    for (size_t i = 0; i < sizeof(specs) / sizeof(specs[0]) - 1; i++) {
        char fmt[4];
        fmt[0] = '%'; fmt[1] = specs[i]; fmt[2] = 0;
        SF_ROW(label_idx(i), fmt, g_wf_base, SFBUF);

        char fmt_alt[4];
        fmt_alt[0] = '%'; fmt_alt[1] = '#'; fmt_alt[2] = specs[i]; fmt_alt[3] = 0;
        SF_ROW(label_idx(i), fmt_alt, g_wf_base, SFBUF);
    }

    for (int wd = 0; wd < 7; wd++) {
        struct tm t = g_wf_base;
        t.tm_wday = wd;
        SF_ROW(label_idx((unsigned)wd), "%a %A %u %w", t, SFBUF);
    }
    for (int mo = 0; mo < 12; mo++) {
        struct tm t = g_wf_base;
        t.tm_mon = mo;
        SF_ROW(label_idx((unsigned)mo), "%b %B", t, SFBUF);
    }

    {
        struct tm t = g_wf_base; t.tm_mday = 0;
        SF_ROW("badmday", "%d", t, SFBUF);
    }
    {
        struct tm t = g_wf_base; t.tm_hour = 24;
        SF_ROW("badhour", "%H", t, SFBUF);
    }

    /* Too-small buffer, narrow side. */
    {
        char b1[SFBUF], b2[SFBUF];
        for (int k = 0; k < SFBUF; k++) { b1[k] = 'X'; b2[k] = 'X'; }
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = strftime(b1, 3, "%A", &g_wf_base);
        size_t r2 = ref_strftime(b2, 3, "%A", &g_wf_base);
        grade2("strftime.small", "-", (long long)r1, (long long)r2);
        grade2("strftime.small.buf0", "-", b1[0], b2[0]);
        grade2("strftime.small.errno", "-", *_errno(), *ref__errno());
    }

    SF_ROW("invalidspec", "%q", g_wf_base, SFBUF);

    /* NULL string / NULL format / NULL timeptr / zero max_size. */
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = strftime((char *)0, SFBUF, "%Y", &g_wf_base);
        size_t r2 = ref_strftime((char *)0, SFBUF, "%Y", &g_wf_base);
        grade2("strftime.nullstring", "-", (long long)r1, (long long)r2);
        grade2("strftime.nullstring.errno", "-", *_errno(), *ref__errno());
    }
    {
        char b1[SFBUF], b2[SFBUF];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = strftime(b1, SFBUF, (const char *)0, &g_wf_base);
        size_t r2 = ref_strftime(b2, SFBUF, (const char *)0, &g_wf_base);
        grade2("strftime.nullformat", "-", (long long)r1, (long long)r2);
        grade2("strftime.nullformat.errno", "-", *_errno(), *ref__errno());
    }
    {
        char b1[SFBUF], b2[SFBUF];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = strftime(b1, SFBUF, "%Y", (const struct tm *)0);
        size_t r2 = ref_strftime(b2, SFBUF, "%Y", (const struct tm *)0);
        grade2("strftime.nulltm", "-", (long long)r1, (long long)r2);
        grade2("strftime.nulltm.errno", "-", *_errno(), *ref__errno());
    }
    {
        char b1[SFBUF], b2[SFBUF];
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        size_t r1 = strftime(b1, 0, "%Y", &g_wf_base);
        size_t r2 = ref_strftime(b2, 0, "%Y", &g_wf_base);
        grade2("strftime.zerosize", "-", (long long)r1, (long long)r2);
        grade2("strftime.zerosize.errno", "-", *_errno(), *ref__errno());
    }

    /* _Getdays / _Getmonths / _Gettnames -- narrow twins of the `_W_*`
     * checks above. */
    {
        char *p1 = ud__Getdays();
        char *p2 = ref__Getdays();
        grade2("_Getdays.isnull", "-", p1 == 0, p2 == 0);
        if (p1 && p2)
            grade2("_Getdays.val", "-", fnv(p1, strlen(p1) + 1), fnv(p2, strlen(p2) + 1));
    }
    {
        char *p1 = ud__Getmonths();
        char *p2 = ref__Getmonths();
        grade2("_Getmonths.isnull", "-", p1 == 0, p2 == 0);
        if (p1 && p2)
            grade2("_Getmonths.val", "-", fnv(p1, strlen(p1) + 1), fnv(p2, strlen(p2) + 1));
    }
    {
        void *p1 = ud__Gettnames();
        void *p2 = ref__Gettnames();
        grade2("_Gettnames.isnull", "-", p1 == 0, p2 == 0);
    }
}
#endif /* UCRTDIFF_REF_HAS_strftime */

/* ==== the 64-bit and wide siblings ==========================================
 * The batch declared these on the reference side but graded only their 32-bit or
 * narrow twins; pecheck caught it (a ref_ declaration with no DLL import). Same
 * contracts, same idiom as the rows above, plus a post-2038 time for the 64-bit
 * utime forms, which is the only thing the width changes. */
#if UCRTDIFF_REF_HAS__utime32
static const wchar_t *time_temp_wpath(void)
{
    static wchar_t buf[MAX_PATH];
    GetTempPathW(MAX_PATH, buf);
    wcscat(buf, L"ucrtdiff_time_utime.tmp");
    return buf;
}
static void ut_touch_a(const char *p) { FILE *f = fopen(p, "w"); if (f) fclose(f); }
static void ut_touch_w(const wchar_t *p) { FILE *f = _wfopen(p, L"w"); if (f) fclose(f); }

#define UTX_ROW(fn, lbl, UT, path, touch, statf, rm, at, mt) do {              \
        UT t1, t2; t1.actime = (at); t1.modtime = (mt); t2 = t1;               \
        struct _stat64 s1, s2; memset(&s1, 0, sizeof s1); memset(&s2, 0, sizeof s2); \
        touch(path); *_errno() = 0;                                            \
        int r1 = fn((path), &t1); int e1 = *_errno();                          \
        statf((path), &s1); rm(path);                                          \
        touch(path); *ref__errno() = 0;                                        \
        int r2 = ref_##fn((path), &t2); int e2 = *ref__errno();                \
        statf((path), &s2); rm(path);                                          \
        grade2(#fn, lbl, r1, r2);                                              \
        grade2(#fn ".errno", lbl, e1, e2);                                     \
        grade2(#fn ".mtime", lbl, (long long)s1.st_mtime, (long long)s2.st_mtime); \
        grade2(#fn ".atime", lbl, (long long)s1.st_atime, (long long)s2.st_atime); \
    } while (0)

#define UTX_NULLNAME(fn, PT, UT) do {                                          \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        int r1 = fn((PT)0, (UT *)0), r2 = ref_##fn((PT)0, (UT *)0);            \
        grade2(#fn ".nullname", "-", r1, r2);                                  \
        grade2(#fn ".nullname.errno", "-", *_errno(), *ref__errno());          \
        grade2(#fn ".nullname.iph", "-", g_iph_dll, g_iph_ref);                \
    } while (0)

static void run_time_utime_siblings(void)
{
    install_counting_iph();
    _wputenv_s(L"TZ", L"PST8PDT"); _tzset();
    ref__wputenv_s(L"TZ", L"PST8PDT"); ref__tzset();

    const char *pa = time_temp_path();
    const wchar_t *pw = time_temp_wpath();
    UTX_ROW(_utime64, "normal", struct __utimbuf64, pa, ut_touch_a, _stat64, remove,
            1700000000LL, 1700003600LL);
    UTX_ROW(_utime64, "y2100", struct __utimbuf64, pa, ut_touch_a, _stat64, remove,
            4102444800LL, 4102448400LL);
    UTX_NULLNAME(_utime64, const char *, struct __utimbuf64);
    UTX_ROW(_wutime32, "normal", struct __utimbuf32, pw, ut_touch_w, _wstat64, _wremove,
            1700000000L, 1700003600L);
    UTX_NULLNAME(_wutime32, const wchar_t *, struct __utimbuf32);
    UTX_ROW(_wutime64, "normal", struct __utimbuf64, pw, ut_touch_w, _wstat64, _wremove,
            1700000000LL, 1700003600LL);
    UTX_ROW(_wutime64, "y2100", struct __utimbuf64, pw, ut_touch_w, _wstat64, _wremove,
            4102444800LL, 4102448400LL);
    UTX_NULLNAME(_wutime64, const wchar_t *, struct __utimbuf64);

    /* _futime64: the same shape as run_time__futime32, past 2038 as well. */
    for (int k = 0; k < 2; k++) {
        __time64_t at = k ? 4102444800LL : 1700000000LL, mt = at + 3600;
        struct _stat64 s1, s2; memset(&s1, 0, sizeof s1); memset(&s2, 0, sizeof s2);
        struct __utimbuf64 t1, t2; t1.actime = at; t1.modtime = mt; t2 = t1;
        ut_touch_a(pa);
        int fh1 = time_open_rw(pa, 0);
        *_errno() = 0;
        int r1 = _futime64(fh1, &t1), e1 = *_errno();
        _stat64(pa, &s1); _close(fh1); remove(pa);
        ut_touch_a(pa);
        int fh2 = time_open_rw(pa, 1);
        *ref__errno() = 0;
        int r2 = ref__futime64(fh2, &t2), e2 = *ref__errno();
        _stat64(pa, &s2); ref__close(fh2); remove(pa);
        const char *lbl = k ? "y2100" : "normal";
        grade2("_futime64", lbl, r1, r2);
        grade2("_futime64.errno", lbl, e1, e2);
        grade2("_futime64.mtime", lbl, (long long)s1.st_mtime, (long long)s2.st_mtime);
        grade2("_futime64.atime", lbl, (long long)s1.st_atime, (long long)s2.st_atime);
    }
    {
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
        int r1 = _futime64(-2, (struct __utimbuf64 *)0);
        int r2 = ref__futime64(-2, (struct __utimbuf64 *)0);
        grade2("_futime64.badfh", "-", r1, r2);
        grade2("_futime64.badfh.errno", "-", *_errno(), *ref__errno());
        grade2("_futime64.badfh.iph", "-", g_iph_dll, g_iph_ref);
    }

    _wputenv_s(L"TZ", L""); _tzset();
    ref__wputenv_s(L"TZ", L""); ref__tzset();
}
#endif /* UCRTDIFF_REF_HAS__utime32 */

#if UCRTDIFF_REF_HAS__ftime32
static void run_time__ftime64(void)
{
    struct __timeb64 t1, t2;
    install_counting_iph();
    memset(&t1, 0, sizeof t1); memset(&t2, 0, sizeof t2);
    _ftime64(&t1);
    ref__ftime64(&t2);
    grade2("_ftime64.nonzero", "-", t1.time != 0, t2.time != 0);
    grade2("_ftime64.millitm_range", "-", t1.millitm < 1000, t2.millitm < 1000);
}
#endif

#if UCRTDIFF_REF_HAS__strdate
static long long wdate_shape(const wchar_t *b)
{
    return (long long)(is_digit_ch(b[0]) && is_digit_ch(b[1]) && b[2] == L'/' &&
                        is_digit_ch(b[3]) && is_digit_ch(b[4]) && b[5] == L'/' &&
                        is_digit_ch(b[6]) && is_digit_ch(b[7]) && b[8] == 0);
}

static void run_time__wstrdate(void)
{
    wchar_t b1[16], b2[16];
    install_counting_iph();
    for (int k = 0; k < 16; k++) b1[k] = b2[k] = L'X';
    *_errno() = 0; *ref__errno() = 0;
    errno_t c1 = _wstrdate_s(b1, 9), c2 = ref__wstrdate_s(b2, 9);
    grade2("_wstrdate_s", "-", c1, c2);
    grade2("_wstrdate_s.shape", "-", wdate_shape(b1), wdate_shape(b2));
    grade2("_wstrdate_s.errno", "-", *_errno(), *ref__errno());

    for (int k = 0; k < 16; k++) b1[k] = b2[k] = L'X';
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _wstrdate_s(b1, 4); c2 = ref__wstrdate_s(b2, 4);
    grade2("_wstrdate_s.small", "-", c1, c2);
    grade2("_wstrdate_s.small.buf0", "-", b1[0], b2[0]);
    grade2("_wstrdate_s.small.errno", "-", *_errno(), *ref__errno());
    grade2("_wstrdate_s.small.iph", "-", g_iph_dll, g_iph_ref);

    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    c1 = _wstrdate_s((wchar_t *)0, 9); c2 = ref__wstrdate_s((wchar_t *)0, 9);
    grade2("_wstrdate_s.null", "-", c1, c2);
    grade2("_wstrdate_s.null.errno", "-", *_errno(), *ref__errno());
    grade2("_wstrdate_s.null.iph", "-", g_iph_dll, g_iph_ref);

    for (int k = 0; k < 16; k++) b1[k] = b2[k] = L'X';
    wchar_t *p1 = _wstrdate(b1), *p2 = ref__wstrdate(b2);
    grade2("_wstrdate.isnull", "-", p1 == 0, p2 == 0);
    grade2("_wstrdate.shape", "-", wdate_shape(b1), wdate_shape(b2));
}
#endif

#if UCRTDIFF_REF_HAS__time32
static void run_time__timespec64_get(void)
{
    struct _timespec64 ts1, ts2;
    memset(&ts1, 0, sizeof ts1); memset(&ts2, 0, sizeof ts2);
    int b1 = _timespec64_get(&ts1, 1 /* TIME_UTC */);
    int b2 = ref__timespec64_get(&ts2, 1);
    grade2("_timespec64_get.base", "-", b1, b2);
    grade2("_timespec64_get.nsec_range", "-", ts1.tv_nsec >= 0 && ts1.tv_nsec < 1000000000L, 1);
    grade2("_timespec64_get.nsec_range", "-", ts2.tv_nsec >= 0 && ts2.tv_nsec < 1000000000L, 1);
    memset(&ts1, 0xAB, sizeof ts1); memset(&ts2, 0xAB, sizeof ts2);
    b1 = _timespec64_get(&ts1, 999);
    b2 = ref__timespec64_get(&ts2, 999);
    grade2("_timespec64_get.badbase", "-", b1, b2);
    grade2("_timespec64_get.badbase.untouched", "-", fnv(&ts1, sizeof ts1), fnv(&ts2, sizeof ts2));
    install_counting_iph();
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    b1 = _timespec64_get((struct _timespec64 *)0, 1);
    b2 = ref__timespec64_get((struct _timespec64 *)0, 1);
    grade2("_timespec64_get.null", "-", b1, b2);
    grade2("_timespec64_get.null.errno", "-", *_errno(), *ref__errno());
    grade2("_timespec64_get.null.iph", "-", g_iph_dll, g_iph_ref);
}
#endif

/* The strftime wrappers that take a locale or an lc_time_arg. NULL for either means
 * "the current locale" in the C; grade that, plus a bad format and a short buffer.
 * Import pointers: _Strftime and _Wcsftime are in no public header. */
#if UCRTDIFF_REF_HAS_strftime
extern size_t(__cdecl *const ud__strftime_l)(char *, size_t, const char *, const struct tm *,
                                             _locale_t) __asm__(UD_IMP(_strftime_l));
extern size_t(__cdecl *const ud__Strftime)(char *, size_t, const char *, const struct tm *,
                                           void *) __asm__(UD_IMP(_Strftime));
#define SFX_ROW(name, call1, call2, lbl) do {                                  \
        char b1[64], b2[64]; memset(b1, 'X', sizeof b1); memset(b2, 'X', sizeof b2); \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        size_t r1 = call1, r2 = call2;                                         \
        grade2(name, lbl, (long long)r1, (long long)r2);                       \
        grade2(name ".val", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));      \
        grade2(name ".errno", lbl, *_errno(), *ref__errno());                  \
        grade2(name ".iph", lbl, g_iph_dll, g_iph_ref);                        \
    } while (0)
static void run_time_strftime_siblings(void)
{
    static const char *const fmts[] = { "%c|%x|%X", "%#c|%#x", "%A %B %d %Y %H:%M:%S %p %j %U %W",
                                        "%q", "%Y-%m-%d" };
    static const size_t sizes[] = { 64, 64, 64, 64, 5 };
    install_counting_iph();
    for (int i = 0; i < 5; i++) {
        char lbl[8]; lbl[0] = (char)('0' + i); lbl[1] = 0;
        SFX_ROW("_strftime_l", ud__strftime_l(b1, sizes[i], fmts[i], &g_wf_base, 0),
                ref__strftime_l(b2, sizes[i], fmts[i], &g_wf_base, 0), lbl);
        SFX_ROW("_Strftime", ud__Strftime(b1, sizes[i], fmts[i], &g_wf_base, 0),
                ref__Strftime(b2, sizes[i], fmts[i], &g_wf_base, 0), lbl);
    }
}
#endif

#if UCRTDIFF_REF_HAS_wcsftime
extern size_t(__cdecl *const ud__wcsftime_l)(wchar_t *, size_t, const wchar_t *, const struct tm *,
                                             _locale_t) __asm__(UD_IMP(_wcsftime_l));
extern size_t(__cdecl *const ud__Wcsftime)(wchar_t *, size_t, const wchar_t *, const struct tm *,
                                           void *) __asm__(UD_IMP(_Wcsftime));
#define WFX_ROW(name, call1, call2, lbl) do {                                  \
        wchar_t b1[64], b2[64];                                                \
        for (int _k = 0; _k < 64; _k++) b1[_k] = b2[_k] = L'X';                \
        *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;           \
        size_t r1 = call1, r2 = call2;                                         \
        grade2(name, lbl, (long long)r1, (long long)r2);                       \
        grade2(name ".val", lbl, fnv(b1, sizeof b1), fnv(b2, sizeof b2));      \
        grade2(name ".errno", lbl, *_errno(), *ref__errno());                  \
        grade2(name ".iph", lbl, g_iph_dll, g_iph_ref);                        \
    } while (0)
static void run_time_wcsftime_siblings(void)
{
    static const wchar_t *const fmts[] = { L"%c|%x|%X", L"%#c|%#x",
                                           L"%A %B %d %Y %H:%M:%S %p %j %U %W", L"%q", L"%Y-%m-%d" };
    static const size_t sizes[] = { 64, 64, 64, 64, 5 };
    install_counting_iph();
    for (int i = 0; i < 5; i++) {
        char lbl[8]; lbl[0] = (char)('0' + i); lbl[1] = 0;
        WFX_ROW("_wcsftime_l", ud__wcsftime_l(b1, sizes[i], fmts[i], &g_wf_base, 0),
                ref__wcsftime_l(b2, sizes[i], fmts[i], &g_wf_base, 0), lbl);
        WFX_ROW("_Wcsftime", ud__Wcsftime(b1, sizes[i], fmts[i], &g_wf_base, 0),
                ref__Wcsftime(b2, sizes[i], fmts[i], &g_wf_base, 0), lbl);
    }
}
#endif

#if UCRTDIFF_REF_HAS__utime32
#define TIME_HAND_SIB_UTIME { "_utime64+_wutime+_futime64", run_time_utime_siblings },
#else
#define TIME_HAND_SIB_UTIME
#endif
#if UCRTDIFF_REF_HAS__ftime32
#define TIME_HAND_SIB_FTIME { "_ftime64", run_time__ftime64 },
#else
#define TIME_HAND_SIB_FTIME
#endif
#if UCRTDIFF_REF_HAS__strdate
#define TIME_HAND_SIB_STRDATE { "_wstrdate", run_time__wstrdate },
#else
#define TIME_HAND_SIB_STRDATE
#endif
#if UCRTDIFF_REF_HAS__time32
#define TIME_HAND_SIB_TIME { "_timespec64_get", run_time__timespec64_get },
#else
#define TIME_HAND_SIB_TIME
#endif
#if UCRTDIFF_REF_HAS_strftime
#define TIME_HAND_SIB_STRFTIME { "_strftime_l+_Strftime", run_time_strftime_siblings },
#else
#define TIME_HAND_SIB_STRFTIME
#endif
#if UCRTDIFF_REF_HAS_wcsftime
#define TIME_HAND_SIB_WCSFTIME { "_wcsftime_l+_Wcsftime", run_time_wcsftime_siblings },
#else
#define TIME_HAND_SIB_WCSFTIME
#endif
#define TIME_HAND_SIBLING_ENTRIES TIME_HAND_SIB_UTIME TIME_HAND_SIB_FTIME \
    TIME_HAND_SIB_STRDATE TIME_HAND_SIB_TIME TIME_HAND_SIB_STRFTIME TIME_HAND_SIB_WCSFTIME

/*
 * Paste into run_legacy()'s `legacy[]` table, right after the CONVTIME_HAND
 * rows (or anywhere; order does not matter, only presence does). One
 * #define block per guard, exactly like FP_HAND/ST_HAND/CONVTIME_HAND above
 * it, with `{ "_name", 0 }` placeholders when the guard reads 0.
 */
#if UCRTDIFF_REF_HAS__tzset
#define TIME_HAND_TZSET_ENTRIES { "_tzset", run_time__tzset },
#else
#define TIME_HAND_TZSET_ENTRIES { "_tzset", 0 },
#endif

#if UCRTDIFF_REF_HAS__localtime32
#define TIME_HAND_LOCALTIME_ENTRIES { "_localtime32", run_time__localtime32 },
#else
#define TIME_HAND_LOCALTIME_ENTRIES { "_localtime32", 0 },
#endif

#if UCRTDIFF_REF_HAS__gmtime32 && !UCRTDIFF_REF_HAS__localtime32
#define TIME_HAND_GMTIME_ENTRIES { "_gmtime32", run_time__gmtime32 },
#else
#define TIME_HAND_GMTIME_ENTRIES { "_gmtime32", 0 },
#endif

#if UCRTDIFF_REF_HAS__mktime32
#define TIME_HAND_MKTIME_ENTRIES { "_mktime32", run_time__mktime32 },
#else
#define TIME_HAND_MKTIME_ENTRIES { "_mktime32", 0 },
#endif

#if UCRTDIFF_REF_HAS_asctime
#define TIME_HAND_ASCTIME_ENTRIES                                              \
    { "asctime_s", run_time_asctime_s }, { "_wasctime_s", run_time__wasctime_s }, \
    { "asctime", run_time_asctime }, { "_wasctime", run_time__wasctime },
#else
#define TIME_HAND_ASCTIME_ENTRIES                                              \
    { "asctime_s", 0 }, { "_wasctime_s", 0 }, { "asctime", 0 }, { "_wasctime", 0 },
#endif

#if UCRTDIFF_REF_HAS__utime32
#define TIME_HAND_UTIME_ENTRIES { "_utime32", run_time__utime32 }, { "_futime32", run_time__futime32 },
#else
#define TIME_HAND_UTIME_ENTRIES { "_utime32", 0 }, { "_futime32", 0 },
#endif

#if UCRTDIFF_REF_HAS__ftime32
#define TIME_HAND_FTIME_ENTRIES                                                \
    { "_ftime32_s", run_time__ftime32_s }, { "_ftime32", run_time__ftime32 },  \
    { "_ftime64_s", run_time__ftime64_s },
#else
#define TIME_HAND_FTIME_ENTRIES                                                \
    { "_ftime32_s", 0 }, { "_ftime32", 0 }, { "_ftime64_s", 0 },
#endif

#if UCRTDIFF_REF_HAS__strdate
#define TIME_HAND_STRDATE_ENTRIES { "_strdate_s", run_time__strdate_s }, { "_strdate", run_time__strdate },
#else
#define TIME_HAND_STRDATE_ENTRIES { "_strdate_s", 0 }, { "_strdate", 0 },
#endif

#if UCRTDIFF_REF_HAS__strtime
#define TIME_HAND_STRTIME_ENTRIES { "_strtime_s", run_time__strtime_s }, { "_strtime", run_time__strtime },
#else
#define TIME_HAND_STRTIME_ENTRIES { "_strtime_s", 0 }, { "_strtime", 0 },
#endif

#if UCRTDIFF_REF_HAS__time32
#define TIME_HAND_TIME_ENTRIES                                                 \
    { "_time32", run_time__time32 }, { "_time64", run_time__time64 },          \
    { "_timespec32_get", run_time__timespec32_get },
#else
#define TIME_HAND_TIME_ENTRIES                                                 \
    { "_time32", 0 }, { "_time64", 0 }, { "_timespec32_get", 0 },
#endif

#if UCRTDIFF_REF_HAS_clock
#define TIME_HAND_CLOCK_ENTRIES { "clock", run_time_clock },
#else
#define TIME_HAND_CLOCK_ENTRIES { "clock", 0 },
#endif

#if UCRTDIFF_REF_HAS__difftime32
#define TIME_HAND_DIFFTIME_ENTRIES { "_difftime32", run_time__difftime32 },
#else
#define TIME_HAND_DIFFTIME_ENTRIES { "_difftime32", 0 },
#endif

#if UCRTDIFF_REF_HAS_wcsftime
#define TIME_HAND_WCSFTIME_ENTRIES { "wcsftime", run_time__Wcsftime_l },
#else
#define TIME_HAND_WCSFTIME_ENTRIES { "wcsftime", 0 },
#endif

#if UCRTDIFF_REF_HAS_strftime
#define TIME_HAND_STRFTIME_ENTRIES { "strftime", run_time__Strftime_l },
#else
#define TIME_HAND_STRFTIME_ENTRIES { "strftime", 0 },
#endif

#define TIME_HAND_ENTRIES                                                      \
    TIME_HAND_TZSET_ENTRIES TIME_HAND_LOCALTIME_ENTRIES TIME_HAND_GMTIME_ENTRIES \
    TIME_HAND_MKTIME_ENTRIES TIME_HAND_ASCTIME_ENTRIES TIME_HAND_UTIME_ENTRIES  \
    TIME_HAND_FTIME_ENTRIES TIME_HAND_STRDATE_ENTRIES TIME_HAND_STRTIME_ENTRIES \
    TIME_HAND_TIME_ENTRIES TIME_HAND_CLOCK_ENTRIES TIME_HAND_DIFFTIME_ENTRIES   \
    TIME_HAND_WCSFTIME_ENTRIES TIME_HAND_STRFTIME_ENTRIES


/* ==== the stdio blocks' own temp paths ====================================
 * Unconditional, and not the printf block's pf_path: that block compiles out once
 * its guard reads 0, and these must not go with it. */
static char g_sd_dir[MAX_PATH];
static int sd_dir(void)
{
    if (g_sd_dir[0])
        return 1;
    DWORD n = GetTempPathA(MAX_PATH - 32, g_sd_dir);
    if (!n || n >= MAX_PATH - 32)
        lstrcpyA(g_sd_dir, "C:\\");
    return 1;
}
static const char *sd_path(const char *leaf, int side)
{
    static char buf[2][MAX_PATH + 48];
    sd_dir();
    lstrcpyA(buf[side], g_sd_dir);
    lstrcatA(buf[side], side ? "ucrtdiff_sd_ref_" : "ucrtdiff_sd_dll_");
    lstrcatA(buf[side], leaf);
    return buf[side];
}

#if UCRTDIFF_REF_HAS_fgetc || UCRTDIFF_REF_HAS_fgetwc || UCRTDIFF_REF_HAS_fputc
/* Each side's OWN standard stream. The harness has no console, so fd 0/1 are
 * _NO_CONSOLE_FILENO (-2) and cannot be _dup'd; freopen is what re-points a
 * standard stream, and NUL is what it goes back to. */
void *__cdecl ref___acrt_iob_func(unsigned);
FILE *__cdecl ref_freopen(const char *, const char *, FILE *);
int __cdecl ref_fflush(FILE *);
static FILE *ref_std_stream(unsigned i) { return (FILE *)ref___acrt_iob_func(i); }
#endif

/* ==== staging/stdio_bufio.c, inlined ==== */
/*
 * staging/stdio_bufio.c -- guest grader for the `bufio` piece of the stdio
 * batch (sdk/lib/rust/ucrt-rust/src/stdio/bufio/): the buffer lifecycle
 * (_getbuf/_freebuf/_sftbuf), the refill/flush engines (_filbuf/_flsbuf), and
 * everything built directly on a single character or line: fgetc/fgetwc,
 * fputc/fputwc, getw/putw, ungetc/ungetwc, gets/fgets, puts/fputs/putws/
 * fputws, and the temporary-file family (tmpfile/tmpnam/_tempnam/_rmtmp/
 * __acrt_GetTempPath2A).
 *
 * Six of the twenty-two C files this piece replaces (_getbuf.cpp,
 * _freebuf.cpp, _sftbuf.cpp, _filbuf.cpp, _flsbuf.cpp, gettemppath.cpp) have
 * NO public export of their own -- they are internal helpers with no `ref_`
 * counterpart reachable from the reference archive by name, so they are
 * graded ONLY indirectly, through the public functions that call them
 * (fgetc/fgetwc exercise _filbuf + _getbuf; fputc/fputwc exercise _flsbuf +
 * _getbuf + _sftbuf; tmpfile exercises gettemppath). Each guard below is one
 * export of the ONE C file it grades directly; the file list comment on each
 * block names what it covers indirectly too.
 *
 * Modelled on the printf block's stream helpers (`pf_dir`/`pf_path`, ~line
 * 1611 of ucrtdiff.c) -- reused here, not reinvented, since this file is
 * `#include`d into the same translation unit ahead of `run_legacy()`. Each
 * side opens its OWN temp file (`sd_path(leaf, 0)` for the DLL, `sd_path(leaf,
 * 1)` for the reference) via its OWN `fopen`/`ref_fopen`, per the batch
 * brief: a `FILE*` from one side's CRT means nothing to the other's `_iob`/
 * lowio table.
 *
 * NOTHING here touches process-wide state permanently: no TZ/locale/codepage
 * change survives a case, and every temp file this snippet creates is closed
 * and deleted (`DeleteFileA`) before the case returns.
 */

/* ==== _getbuf/_freebuf/_sftbuf/_filbuf/_flsbuf/fgetc.cpp ==== */
#if UCRTDIFF_REF_HAS_fgetc

int __cdecl ref_fgetc(FILE *);
int __cdecl ref__fgetc_nolock(FILE *);
int __cdecl ref_getc(FILE *);
int __cdecl ref__getc_nolock(FILE *);
int __cdecl ref__fgetchar(void);
int __cdecl ref_getchar(void);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
int __cdecl ref_feof(FILE *);
int __cdecl ref_ferror(FILE *);
int __cdecl ref_fseek(FILE *, long, int);

extern int(__cdecl *const ud_fgetc)(FILE *) __asm__(UD_IMP(fgetc));
extern int(__cdecl *const ud__fgetc_nolock)(FILE *) __asm__(UD_IMP(_fgetc_nolock));
extern int(__cdecl *const ud_getc)(FILE *) __asm__(UD_IMP(getc));
extern int(__cdecl *const ud__getc_nolock)(FILE *) __asm__(UD_IMP(_getc_nolock));

/* Writes `data` (narrow bytes) into a fresh two-sided temp file opened in
 * `mode`, then drains it through `ud_fgetc`/`ref_fgetc` one call at a time,
 * grading the return each step, `feof`/`ferror` at the end, and the number
 * of characters read. Exercises `_filbuf`'s refill across a buffer boundary
 * when `data` is longer than `_INTERNAL_BUFSIZ` (4096).
 */
static void run_bufio_fgetc_stream(const char *leaf, const char *mode, const char *data, size_t n)
{
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fgetc.setup.open", leaf, f1 != 0, f2 != 0);
    if (f1) { fwrite(data, 1, n, f1); fclose(f1); }
    if (f2) { ref_fwrite(data, 1, n, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), mode);
    f2 = ref_fopen(sd_path(leaf, 1), mode);
    grade2("fgetc.open", leaf, f1 != 0, f2 != 0);
    if (!f1 || !f2) { if (f1) fclose(f1); if (f2) ref_fclose(f2); return; }

    install_counting_iph();
    long long count1 = 0, count2 = 0, sum1 = 0, sum2 = 0;
    int c1, c2;
    do {
        *_errno() = 0; *ref__errno() = 0;
        c1 = ud_fgetc(f1);
        c2 = ref_fgetc(f2);
        grade2("fgetc.char", leaf, c1, c2);
        if (c1 != EOF) { count1++; sum1 += (unsigned char)c1; }
        if (c2 != EOF) { count2++; sum2 += (unsigned char)c2; }
    } while (c1 != EOF && c2 != EOF);
    grade2("fgetc.count", leaf, count1, count2);
    grade2("fgetc.sum", leaf, sum1, sum2);
    grade2("fgetc.feof", leaf, feof(f1) != 0, ref_feof(f2) != 0);
    grade2("fgetc.ferror", leaf, ferror(f1) != 0, ref_ferror(f2) != 0);

    /* `_getc_nolock` and the plain forms must agree at the current position
     * (both are EOF here since the loop above drained the stream). */
    grade2("_fgetc_nolock.eof", leaf, ud__fgetc_nolock(f1), ref__fgetc_nolock(f2));
    grade2("getc.eof", leaf, ud_getc(f1), ref_getc(f2));

    fclose(f1);
    ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

static void run_bufio_fgetc(void)
{
    run_bufio_fgetc_stream("fgetc_small.txt", "rb", "hello\n", 6);
    /* Text-mode CRLF collapse and a trailing Ctrl-Z: the on-disk bytes are
     * "a\r\nb\x1A", read back through text-mode translation. */
    run_bufio_fgetc_stream("fgetc_text.txt", "rt", "a\r\nb\x1A", 5);
    {
        /* Cross the 4096-byte `_INTERNAL_BUFSIZ` refill boundary. */
        static char big[9000];
        for (size_t i = 0; i < sizeof big; i++) big[i] = (char)('A' + (i % 26));
        run_bufio_fgetc_stream("fgetc_big.bin", "rb", big, sizeof big);
    }
    run_bufio_fgetc_stream("fgetc_empty.bin", "rb", "", 0);

    /* NULL stream: `_VALIDATE_RETURN` path, no file needed. */
    *_errno() = 0; *ref__errno() = 0;
    grade2("fgetc.null", "NULL", ud_fgetc(0), ref_fgetc(0));
    grade2("fgetc.null.errno", "NULL", *_errno(), *ref__errno());
}

#define BUFIO_HAND_FGETC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FGETC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fgetc */

/* ==== fgetwc.cpp ==== */
#if UCRTDIFF_REF_HAS_fgetwc

wint_t __cdecl ref_fgetwc(FILE *);
wint_t __cdecl ref_getwc(FILE *);
wint_t __cdecl ref__fgetwchar(void);
wint_t __cdecl ref_getwchar(void);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

extern wint_t(__cdecl *const ud_fgetwc)(FILE *) __asm__(UD_IMP(fgetwc));
extern wint_t(__cdecl *const ud_getwc)(FILE *) __asm__(UD_IMP(getwc));

/* A UTF-16LE stream (`ccs=UNICODE`) and a plain binary stream: both take
 * fgetwc's "binary (Unicode) mode" path, which is the one that survives even
 * without the `files` piece's `fopen` `ccs=` parsing landed yet -- opened
 * `"rb"` and pre-seeded with raw UTF-16LE bytes, so no mode parsing is
 * exercised, only the two-byte read and the split-code-unit refill in
 * `_filbuf`'s wide instantiation. */
static void run_bufio_fgetwc(void)
{
    static const wchar_t msg[] = L"hié!"; /* includes a non-ASCII code unit */
    const char *leaf = "fgetwc.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fgetwc.setup.open", leaf, f1 != 0, f2 != 0);
    if (f1) { fwrite(msg, sizeof(wchar_t), sizeof(msg) / sizeof(wchar_t) - 1, f1); fclose(f1); }
    if (f2) { ref_fwrite(msg, sizeof(wchar_t), sizeof(msg) / sizeof(wchar_t) - 1, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("fgetwc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        wint_t c1, c2;
        long long count1 = 0, count2 = 0;
        do {
            c1 = ud_fgetwc(f1);
            c2 = ref_fgetwc(f2);
            grade2("fgetwc.char", leaf, (long long)c1, (long long)c2);
            if (c1 != (wint_t)WEOF) count1++;
            if (c2 != (wint_t)WEOF) count2++;
        } while (c1 != (wint_t)WEOF && c2 != (wint_t)WEOF);
        grade2("fgetwc.count", leaf, count1, count2);
        grade2("getwc.eof", leaf, (long long)ud_getwc(f1), (long long)ref_getwc(f2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_FGETWC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FGETWC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fgetwc */

/* ==== fputc.cpp (and _flsbuf/_getbuf/_sftbuf indirectly) ==== */
#if UCRTDIFF_REF_HAS_fputc
int __cdecl ref_ferror(FILE *);

int __cdecl ref_fputc(int, FILE *);
int __cdecl ref_putc(int, FILE *);
int __cdecl ref__fputchar(int);
int __cdecl ref_putchar(int);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fflush(FILE *);
size_t __cdecl ref_fread(void *, size_t, size_t, FILE *);
long __cdecl ref_ftell(FILE *);

extern int(__cdecl *const ud_fputc)(int, FILE *) __asm__(UD_IMP(fputc));
extern int(__cdecl *const ud_putc)(int, FILE *) __asm__(UD_IMP(putc));

/* Writes `n` characters (from a repeating printable pattern) one at a time
 * through `ud_fputc`/`ref_fputc`, then reads the raw bytes back with Win32
 * `ReadFile` (independent of either CRT) and FNV-hashes them. `n` > 4096
 * exercises `_flsbuf`'s buffer-full flush; `n` == 0 exercises the
 * first-write `_getbuf` allocation path with nothing to flush. */
static void run_bufio_fputc_n(const char *leaf, size_t n)
{
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fputc.open", leaf, f1 != 0, f2 != 0);
    if (!f1 || !f2) { if (f1) fclose(f1); if (f2) ref_fclose(f2); return; }

    install_counting_iph();
    long long r1 = 0, r2 = 0;
    for (size_t i = 0; i < n; i++) {
        int ch = (int)('!' + (i % 90));
        int c1 = ud_fputc(ch, f1);
        int c2 = ref_fputc(ch, f2);
        if (c1 != ch) r1++;
        if (c2 != ch) r2++;
    }
    grade2("fputc.bad_returns", leaf, r1, r2);
    grade2("fputc.tell", leaf, ftell(f1), ref_ftell(f2));
    fclose(f1);
    ref_fclose(f2);

    HANDLE h1 = CreateFileA(sd_path(leaf, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    HANDLE h2 = CreateFileA(sd_path(leaf, 1), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    static unsigned char b1[16384], b2[16384];
    DWORD n1 = 0, n2 = 0;
    if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
    if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
    grade2("fputc.file.len", leaf, (long long)n1, (long long)n2);
    grade2("fputc.file", leaf, fnv(b1, n1), fnv(b2, n2));
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

/* An append-mode write to an EMPTY file: `write_buffer_nolock`'s FAPPEND
 * re-seek path with nothing yet on disk (`chars_to_write == 0`). */
static void run_bufio_fputc_append(void)
{
    const char *leaf = "fputc_append.bin";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
    FILE *f1 = fopen(sd_path(leaf, 0), "ab");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "ab");
    grade2("fputc.append.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        int c1 = ud_fputc('Z', f1);
        int c2 = ref_fputc('Z', f2);
        grade2("fputc.append.char", leaf, c1, c2);
        grade2("fputc.append.ferror", leaf, ferror(f1) != 0, ref_ferror(f2) != 0);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

static void run_bufio_fputc(void)
{
    run_bufio_fputc_n("fputc_small.bin", 5);
    run_bufio_fputc_n("fputc_zero.bin", 0);
    run_bufio_fputc_n("fputc_big.bin", 9000); /* crosses _INTERNAL_BUFSIZ */
    run_bufio_fputc_append();

    /* A read-only stream: `_UCRT_VALIDATE_STREAM_ANSI_RETURN`/EBADF path. */
    const char *leaf = "fputc_ro.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb"); if (f1) fclose(f1);
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb"); if (f2) ref_fclose(f2);
    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    if (f1 && f2) {
        *_errno() = 0; *ref__errno() = 0;
        int c1 = ud_fputc('x', f1);
        int c2 = ref_fputc('x', f2);
        grade2("fputc.readonly.ret", leaf, c1, c2);
        grade2("fputc.readonly.errno", leaf, *_errno(), *ref__errno());
        grade2("fputc.readonly.ferror", leaf, ferror(f1) != 0, ref_ferror(f2) != 0);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_FPUTC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FPUTC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputc */

/* ==== fputwc.cpp ==== */
#if UCRTDIFF_REF_HAS_fputwc
int __cdecl ref_fflush(FILE *);

wint_t __cdecl ref_fputwc(wchar_t, FILE *);
wint_t __cdecl ref_putwc(wchar_t, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

extern wint_t(__cdecl *const ud_fputwc)(wchar_t, FILE *) __asm__(UD_IMP(fputwc));

static void run_bufio_fputwc(void)
{
    const char *leaf = "fputwc.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fputwc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        static const wchar_t msg[] = L"hié!";
        long long r1 = 0, r2 = 0;
        for (size_t i = 0; i < sizeof(msg) / sizeof(wchar_t) - 1; i++) {
            wint_t c1 = ud_fputwc(msg[i], f1);
            wint_t c2 = ref_fputwc(msg[i], f2);
            if (c1 != msg[i]) r1++;
            if (c2 != msg[i]) r2++;
        }
        grade2("fputwc.bad_returns", leaf, r1, r2);
        fflush(f1); ref_fflush(f2);

        HANDLE h1 = CreateFileA(sd_path(leaf, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        HANDLE h2 = CreateFileA(sd_path(leaf, 1), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        static unsigned char b1[64], b2[64];
        DWORD n1 = 0, n2 = 0;
        if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
        if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
        grade2("fputwc.file.len", leaf, (long long)n1, (long long)n2);
        grade2("fputwc.file", leaf, fnv(b1, n1), fnv(b2, n2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_FPUTWC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FPUTWC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputwc */

/* ==== getw.cpp / putw.cpp ==== */
#if UCRTDIFF_REF_HAS__getw && UCRTDIFF_REF_HAS__putw

int __cdecl ref__getw(FILE *);
int __cdecl ref__putw(int, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fseek(FILE *, long, int);

extern int(__cdecl *const ud__getw)(FILE *) __asm__(UD_IMP(_getw));
extern int(__cdecl *const ud__putw)(int, FILE *) __asm__(UD_IMP(_putw));

static void run_bufio_getw_putw(void)
{
    const char *leaf = "getwputw.bin";
    static const int vals[] = { 0, 1, -1, 0x7fffffff, (int)0x80000000, 42 };
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("_putw.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        for (size_t i = 0; i < sizeof(vals) / sizeof(vals[0]); i++) {
            int r1 = ud__putw(vals[i], f1);
            int r2 = ref__putw(vals[i], f2);
            grade2("_putw.ret", label_idx(i), r1, r2);
        }
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("_getw.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        for (size_t i = 0; i < sizeof(vals) / sizeof(vals[0]); i++) {
            int r1 = ud__getw(f1);
            int r2 = ref__getw(f2);
            grade2("_getw.val", label_idx(i), r1, r2);
        }
        /* One past the last written int: EOF for both. */
        grade2("_getw.eof", "past-end", ud__getw(f1), ref__getw(f2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_GETW_PUTW(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_GETW_PUTW(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS__getw && UCRTDIFF_REF_HAS__putw */

/* ==== ungetc.cpp ==== */
#if UCRTDIFF_REF_HAS_ungetc

int __cdecl ref_ungetc(int, FILE *);
int __cdecl ref_fgetc(FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);

extern int(__cdecl *const ud_ungetc)(int, FILE *) __asm__(UD_IMP(ungetc));
extern int(__cdecl *const ud_fgetc)(FILE *) __asm__(UD_IMP(fgetc));

static void run_bufio_ungetc(void)
{
    const char *leaf = "ungetc.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (f1) { fwrite("AB", 1, 2, f1); fclose(f1); }
    if (f2) { ref_fwrite("AB", 1, 2, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("ungetc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        /* Read one char, unget it (at the buffer start -- `_ptr == _base`,
         * the one-character-of-room case), read it again, unget EOF
         * (forbidden), unget twice in a row (second must fail). */
        int a1 = ud_fgetc(f1), a2 = ref_fgetc(f2);
        grade2("ungetc.first_char", leaf, a1, a2);
        int u1 = ud_ungetc(a1, f1), u2 = ref_ungetc(a2, f2);
        grade2("ungetc.unget_at_start", leaf, u1, u2);
        int b1 = ud_fgetc(f1), b2 = ref_fgetc(f2);
        grade2("ungetc.reread", leaf, b1, b2);

        int e1 = ud_ungetc(EOF, f1), e2 = ref_ungetc(EOF, f2);
        grade2("ungetc.eof_forbidden", leaf, e1, e2);

        int c1 = ud_ungetc('X', f1), c2 = ref_ungetc('X', f2);
        grade2("ungetc.first", leaf, c1, c2);
        int d1 = ud_ungetc('Y', f1), d2 = ref_ungetc('Y', f2);
        grade2("ungetc.second_fails", leaf, d1, d2);

        grade2("ungetc.feof_cleared", leaf, feof(f1) != 0, ref_ferror(f2) != 0);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    /* NULL stream. */
    *_errno() = 0; *ref__errno() = 0;
    grade2("ungetc.null", "NULL", ud_ungetc('a', 0), ref_ungetc('a', 0));
    grade2("ungetc.null.errno", "NULL", *_errno(), *ref__errno());
}

#define BUFIO_HAND_UNGETC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_UNGETC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_ungetc */

/* ==== ungetwc.cpp ==== */
#if UCRTDIFF_REF_HAS_ungetwc

wint_t __cdecl ref_ungetwc(wint_t, FILE *);
wint_t __cdecl ref_fgetwc(FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

extern wint_t(__cdecl *const ud_ungetwc)(wint_t, FILE *) __asm__(UD_IMP(ungetwc));
extern wint_t(__cdecl *const ud_fgetwc)(FILE *) __asm__(UD_IMP(fgetwc));

static void run_bufio_ungetwc(void)
{
    const char *leaf = "ungetwc.bin";
    static const wchar_t msg[] = L"AB";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (f1) { fwrite(msg, sizeof(wchar_t), 2, f1); fclose(f1); }
    if (f2) { ref_fwrite(msg, sizeof(wchar_t), 2, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("ungetwc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        wint_t a1 = ud_fgetwc(f1), a2 = ref_fgetwc(f2);
        grade2("ungetwc.first_char", leaf, (long long)a1, (long long)a2);
        wint_t u1 = ud_ungetwc(a1, f1), u2 = ref_ungetwc(a2, f2);
        grade2("ungetwc.unget_at_start", leaf, (long long)u1, (long long)u2);
        wint_t b1 = ud_fgetwc(f1), b2 = ref_fgetwc(f2);
        grade2("ungetwc.reread", leaf, (long long)b1, (long long)b2);

        wint_t e1 = ud_ungetwc((wint_t)WEOF, f1), e2 = ref_ungetwc((wint_t)WEOF, f2);
        grade2("ungetwc.weof_forbidden", leaf, (long long)e1, (long long)e2);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_UNGETWC(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_UNGETWC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_ungetwc */

/* ==== gets.cpp: gets_s only (plain `gets` needs redirected stdin, which this
 * harness -- no console -- cannot supply; `gets_s`'s too-small path needs no
 * input at all beyond what we can synthesize is out of scope the same way).
 * NOT graded further: documented, not silently skipped. ==== */
#if UCRTDIFF_REF_HAS_gets_s

errno_t __cdecl ref_gets_s(char *, size_t);

extern errno_t(__cdecl *const ud_gets_s)(char *, size_t) __asm__(UD_IMP(gets_s));

static void run_bufio_gets_s(void)
{
    /* NULL buffer and a zero-size buffer are the only cases gradeable without
     * real console input (stdin is not a console in this harness). */
    char buf[8] = { 0 };
    *_errno() = 0; *ref__errno() = 0;
    /* size 0: `_VALIDATE_RETURN(result_size_in_characters > 0, ...)`. */
    errno_t r1 = ud_gets_s(buf, 0);
    errno_t r2 = ref_gets_s(buf, 0);
    grade2("gets_s.zero_size.ret", "0", r1, r2);
    grade2("gets_s.zero_size.errno", "0", *_errno(), *ref__errno());
}

#define BUFIO_HAND_GETS_S(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_GETS_S(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_gets_s */

/* ==== fgets.cpp ==== */
#if UCRTDIFF_REF_HAS_fgets

char *__cdecl ref_fgets(char *, int, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

extern char *(__cdecl *const ud_fgets)(char *, int, FILE *) __asm__(UD_IMP(fgets));

static void run_bufio_fgets_case(const char *leaf, const char *data, size_t n, int count)
{
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (f1) { fwrite(data, 1, n, f1); fclose(f1); }
    if (f2) { ref_fwrite(data, 1, n, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("fgets.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        char b1[64], b2[64];
        memset(b1, 0x5A, sizeof b1);
        memset(b2, 0x5A, sizeof b2);
        char *r1 = ud_fgets(b1, count, f1);
        char *r2 = ref_fgets(b2, count, f2);
        grade2("fgets.null_ret", leaf, r1 != 0, r2 != 0);
        grade2("fgets.buf", leaf, fnv(b1, sizeof b1), fnv(b2, sizeof b2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

static void run_bufio_fgets(void)
{
    run_bufio_fgets_case("fgets_line.txt", "hello\nworld\n", 12, 20); /* whole line + '\n' */
    run_bufio_fgets_case("fgets_trunc.txt", "hello\nworld\n", 12, 4); /* count < line length */
    run_bufio_fgets_case("fgets_empty.txt", "", 0, 10);              /* immediate EOF */
    run_bufio_fgets_case("fgets_noeol.txt", "noeol", 5, 10);         /* EOF terminates like '\n' */

    /* count == 0: must return NULL without touching the buffer. */
    FILE *f1 = fopen(sd_path("fgets_z.txt", 0), "wb"); if (f1) { fputs("x", f1); fclose(f1); }
    f1 = fopen(sd_path("fgets_z.txt", 0), "rb");
    if (f1) {
        char b[4] = { 1, 2, 3, 4 };
        char *r1 = ud_fgets(b, 0, f1);
        grade2("fgets.count_zero", "0", r1 != 0, 0);
        fclose(f1);
    }
    DeleteFileA(sd_path("fgets_z.txt", 0));
}

#define BUFIO_HAND_FGETS(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FGETS(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fgets */

/* ==== fputs.cpp / puts.cpp / putws.cpp / fputws.cpp ==== */
#if UCRTDIFF_REF_HAS_fputs

int __cdecl ref_fputs(const char *, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fflush(FILE *);

extern int(__cdecl *const ud_fputs)(const char *, FILE *) __asm__(UD_IMP(fputs));

static void run_bufio_fputs(void)
{
    const char *leaf = "fputs.txt";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fputs.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        int r1 = ud_fputs("hello, world", f1);
        int r2 = ref_fputs("hello, world", f2);
        grade2("fputs.ret", leaf, r1 >= 0, r2 >= 0); /* nonnegative-on-success is the contract */
        fflush(f1); ref_fflush(f2);

        HANDLE h1 = CreateFileA(sd_path(leaf, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        HANDLE h2 = CreateFileA(sd_path(leaf, 1), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        static unsigned char b1[64], b2[64];
        DWORD n1 = 0, n2 = 0;
        if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
        if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
        grade2("fputs.file.len", leaf, (long long)n1, (long long)n2);
        grade2("fputs.file", leaf, fnv(b1, n1), fnv(b2, n2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    /* NULL string, NULL stream. */
    *_errno() = 0; *ref__errno() = 0;
    grade2("fputs.null_string", "NULL", ud_fputs(0, stdout), ref_fputs(0, stdout));
    *_errno() = 0; *ref__errno() = 0;
    grade2("fputs.null_stream", "NULL", ud_fputs("x", 0), ref_fputs("x", 0));
}

#define BUFIO_HAND_FPUTS(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FPUTS(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputs */

#if UCRTDIFF_REF_HAS_fputws

int __cdecl ref_fputws(const wchar_t *, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fflush(FILE *);

extern int(__cdecl *const ud_fputws)(const wchar_t *, FILE *) __asm__(UD_IMP(fputws));

static void run_bufio_fputws(void)
{
    const char *leaf = "fputws.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("fputws.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        /* fputws returns EOF (not WEOF) on failure -- graded as EOF's actual
         * int value so a divergence there would show. */
        int r1 = ud_fputws(L"wideé", f1);
        int r2 = ref_fputws(L"wideé", f2);
        grade2("fputws.ret", leaf, r1, r2);
        fflush(f1); ref_fflush(f2);

        HANDLE h1 = CreateFileA(sd_path(leaf, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        HANDLE h2 = CreateFileA(sd_path(leaf, 1), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
        static unsigned char b1[64], b2[64];
        DWORD n1 = 0, n2 = 0;
        if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
        if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
        grade2("fputws.file.len", leaf, (long long)n1, (long long)n2);
        grade2("fputws.file", leaf, fnv(b1, n1), fnv(b2, n2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_HAND_FPUTWS(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_FPUTWS(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputws */

/* ==== tmpfile.cpp / rmtmp.cpp / gettemppath.cpp ====
 *
 * Graded by SHAPE, not by exact name (the batch brief's instruction for this
 * family): the prefix character, that the directory component is one both
 * sides independently resolved to an existing, accessible directory, and
 * that ten successive `tmpnam` calls never collide. Every file this creates
 * is deleted before the case returns; `tmpfile()`'s own file is `_O_TEMPORARY`
 * (deleted on close by lowio itself) so only the DLL/ref name buffers need
 * cleanup here (there is nothing to unlink).
 */
#if UCRTDIFF_REF_HAS_tmpfile

FILE *__cdecl ref_tmpfile(void);
char *__cdecl ref_tmpnam(char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fputc(int, FILE *);
int __cdecl ref_fseek(FILE *, long, int);
long __cdecl ref_ftell(FILE *);

extern FILE *(__cdecl *const ud_tmpfile)(void) __asm__(UD_IMP(tmpfile));
extern char *(__cdecl *const ud_tmpnam)(char *) __asm__(UD_IMP(tmpnam));

/* `tmpfile()` opens "w+b": read-modify-write must work, and the file must
 * not still exist under its own name after close (`_O_TEMPORARY`). */
static void run_bufio_tmpfile(void)
{
    FILE *f1 = ud_tmpfile();
    FILE *f2 = ref_tmpfile();
    grade2("tmpfile.open", "ok", f1 != 0, f2 != 0);
    if (f1 && f2) {
        /* Writing itself is not under test here (that is fputc's own guard
         * above); the plain, unprefixed `fputc` is used only as a UTILITY to
         * populate the DLL-side stream, exactly like `fwrite`/`fclose`
         * elsewhere in this file. */
        int w1 = fputc('Q', f1);
        int w2 = ref_fputc('Q', f2);
        grade2("tmpfile.write", "ok", w1, w2);
        long p1 = ftell(f1), p2 = ref_ftell(f2);
        grade2("tmpfile.tell", "ok", p1, p2);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
}

/* `tmpnam`'s SHAPE: a one-character prefix ('s' narrow, per `buffer_id::tmpnam`),
 * a directory component that exists, and no collision across ten calls. */
static void run_bufio_tmpnam_shape(void)
{
    char b1[L_tmpnam + 1], b2[L_tmpnam + 1];
    char seen1[10][L_tmpnam + 1], seen2[10][L_tmpnam + 1];
    int dup1 = 0, dup2 = 0;
    for (int i = 0; i < 10; i++) {
        char *r1 = ud_tmpnam(b1);
        char *r2 = ref_tmpnam(b2);
        if (i == 0) {
            grade2("tmpnam.nonnull", "ok", r1 != 0, r2 != 0);
        }
        if (r1) {
            lstrcpyA(seen1[i], b1);
            for (int j = 0; j < i; j++) if (lstrcmpA(seen1[j], b1) == 0) dup1 = 1;
        } else {
            seen1[i][0] = 0;
        }
        if (r2) {
            lstrcpyA(seen2[i], b2);
            for (int j = 0; j < i; j++) if (lstrcmpA(seen2[j], b2) == 0) dup2 = 1;
        } else {
            seen2[i][0] = 0;
        }
    }
    grade2("tmpnam.no_collisions", "10x", dup1, dup2);

    /* Prefix character right after the directory component: 's' for both
     * sides' narrow `tmpnam`. Find it as the first char after the last
     * backslash. */
    char *last1 = strrchr(b1, '\\');
    char *last2 = strrchr(b2, '\\');
    char p1 = last1 ? last1[1] : b1[0];
    char p2 = last2 ? last2[1] : b2[0];
    grade2("tmpnam.prefix_char", "ok", p1, p2);
    grade2("tmpnam.prefix_char_is_s", "ok", p1 == 's', p2 == 's');

    /* Both directories must exist and be accessible -- graded as booleans,
     * never as the literal path (machine-specific %TEMP%). */
    char dir1[MAX_PATH], dir2[MAX_PATH];
    lstrcpyA(dir1, b1); if (last1) dir1[last1 - b1 + 1] = 0;
    lstrcpyA(dir2, b2); if (last2) dir2[last2 - b2 + 1] = 0;
    DWORD attr1 = GetFileAttributesA(dir1);
    DWORD attr2 = GetFileAttributesA(dir2);
    grade2("tmpnam.dir_exists", "ok", attr1 != INVALID_FILE_ATTRIBUTES, attr2 != INVALID_FILE_ATTRIBUTES);
}

#define BUFIO_HAND_TMPFILE(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_TMPFILE(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_tmpfile */

/* ==== tempnam.cpp ==== */
#if UCRTDIFF_REF_HAS__tempnam

char *__cdecl ref__tempnam(const char *, const char *);

extern char *(__cdecl *const ud__tempnam)(const char *, const char *) __asm__(UD_IMP(_tempnam));

/* Also graded by SHAPE: prefix string present verbatim, directory exists,
 * ten calls never collide. `free()`s every name (the contract: caller owns
 * it). Not graded: the exact `_tempoff` numeric suffix (an internal counter,
 * not part of the documented contract) and the `TMP`-env-var-vs-fallback
 * directory CHOICE (both sides run in the same guest environment, so they
 * should agree, but asserting the literal directory string would break the
 * moment %TEMP% differs between where the DLL and the reference archive
 * were captured -- SHAPE is the contract here, per the batch brief). */
static void run_bufio_tempnam(void)
{
    char *names1[5], *names2[5];
    int dup1 = 0, dup2 = 0;
    for (int i = 0; i < 5; i++) {
        names1[i] = ud__tempnam(0, "ucd");
        names2[i] = ref__tempnam(0, "ucd");
        for (int j = 0; j < i; j++) {
            if (names1[i] && names1[j] && lstrcmpA(names1[i], names1[j]) == 0) dup1 = 1;
            if (names2[i] && names2[j] && lstrcmpA(names2[i], names2[j]) == 0) dup2 = 1;
        }
    }
    grade2("_tempnam.nonnull", "ok", names1[0] != 0, names2[0] != 0);
    grade2("_tempnam.no_collisions", "5x", dup1, dup2);

    if (names1[0] && names2[0]) {
        /* The prefix "ucd" must appear verbatim, right after the directory's
         * last separator. */
        const char *base1 = strrchr(names1[0], '\\');
        const char *base2 = strrchr(names2[0], '\\');
        base1 = base1 ? base1 + 1 : names1[0];
        base2 = base2 ? base2 + 1 : names2[0];
        grade2("_tempnam.prefix_present", "ok", strncmp(base1, "ucd", 3) == 0, strncmp(base2, "ucd", 3) == 0);
    }

    for (int i = 0; i < 5; i++) {
        if (names1[i]) free(names1[i]);
        if (names2[i]) free(names2[i]);
    }

    /* NULL prefix: still produces a name in the fallback directory. */
    char *n1 = ud__tempnam(0, 0);
    char *n2 = ref__tempnam(0, 0);
    grade2("_tempnam.null_prefix.nonnull", "ok", n1 != 0, n2 != 0);
    if (n1) free(n1);
    if (n2) free(n2);
}

#define BUFIO_HAND_TEMPNAM(name, fn) { name, run_##fn },
#else
#define BUFIO_HAND_TEMPNAM(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS__tempnam */

/* ==========================================================================
 * Registry -- paste every BUFIO_HAND_* entry into run_legacy()'s `legacy[]`
 * table, the same way ST_HAND/FP_HAND/LOC_HAND already are.
 * ========================================================================== */
#define BUFIO_HAND_ENTRIES                                                    \
    BUFIO_HAND_FGETC("fgetc+_filbuf+_getbuf", bufio_fgetc)                    \
    BUFIO_HAND_FGETWC("fgetwc", bufio_fgetwc)                                 \
    BUFIO_HAND_FPUTC("fputc+_flsbuf+_getbuf+_sftbuf", bufio_fputc)            \
    BUFIO_HAND_FPUTWC("fputwc", bufio_fputwc)                                 \
    BUFIO_HAND_GETW_PUTW("_getw+_putw", bufio_getw_putw)                      \
    BUFIO_HAND_UNGETC("ungetc", bufio_ungetc)                                 \
    BUFIO_HAND_UNGETWC("ungetwc", bufio_ungetwc)                              \
    BUFIO_HAND_GETS_S("gets_s", bufio_gets_s)                                 \
    BUFIO_HAND_FGETS("fgets", bufio_fgets)                                    \
    BUFIO_HAND_FPUTS("fputs+puts", bufio_fputs)                               \
    BUFIO_HAND_FPUTWS("fputws+putws", bufio_fputws)                          \
    BUFIO_HAND_TMPFILE("tmpfile+tmpnam+_rmtmp+gettemppath", bufio_tmpfile)     \
    BUFIO_HAND_TEMPNAM("_tempnam", bufio_tempnam)

/* `run_bufio_tmpfile`/`run_bufio_tmpnam_shape` are both registered under one
 * guard (`UCRTDIFF_REF_HAS_tmpfile`); call the shape check from the tmpfile
 * runner so one table entry covers both without a second guard block. */
#if UCRTDIFF_REF_HAS_tmpfile
static void run_bufio_tmpfile_and_tmpnam(void)
{
    run_bufio_tmpfile();
    run_bufio_tmpnam_shape();
}
#undef BUFIO_HAND_TMPFILE
#define BUFIO_HAND_TMPFILE(name, fn) { name, run_bufio_tmpfile_and_tmpnam },
#endif


/* ==== staging/stdio_files.c, inlined ==== */
/*
 * staging/stdio_files.c — guest grader for the `files` piece of the stdio
 * batch: openfile.cpp, fopen.cpp, freopen.cpp, fdopen.cpp, fclose.cpp,
 * closeall.cpp, fflush.cpp, fread.cpp, fwrite.cpp, fseek.cpp, ftell.cpp,
 * fgetpos.cpp, fsetpos.cpp, rewind.cpp, setbuf.cpp, setvbuf.cpp,
 * clearerr.cpp, feoferr.cpp, fileno.cpp, setmaxf.cpp, ncommode.cpp,
 * printf_count_output.cpp.
 *
 * Modelled on the findfile block's temp-file fixture (~line 1140) and the
 * printf block's `pf_dir`/`pf_path` (~line 1615): EACH SIDE opens and
 * operates on its OWN file, through its OWN `FILE*`/lowio descriptor -- a
 * `FILE*` the DLL's `fopen` hands back means nothing to `ref_fread` and vice
 * versa (the reference side has a private `_iob`/`__piob`/lowio table; see
 * the batch brief's "Grading stdio in the guest" section). File CONTENTS are
 * always read back independently with Win32 `ReadFile`, and every file is
 * deleted before the runner returns.
 *
 * ONE GUARD PER C FILE, since files are deleted independently. `_openfile`/
 * `_wopenfile` (openfile.cpp) have no `ref_` symbol of their own in
 * generated-refs.h's HAND_GRADED set (verified: `llvm-nm --defined-only
 * build-arm64/libucrt_ref.a | grep ' ref__openfile$'` is empty) -- they are
 * graded INDIRECTLY here, through `fopen`/`freopen`/`_fdopen`'s own rows
 * (`common_openfile` is the entire body of `_openfile`/`_wopenfile`, so a
 * `fopen` mismatch would surface there first anyway), and openfile.cpp's
 * deletion rides on the `UCRTDIFF_REF_HAS_fopen` guard.
 *
 * NOTHING here touches process-wide state permanently: `_setmaxstdio` is
 * always restored to its prior value before returning, `_set_printf_count_output`
 * likewise, and every temp file this snippet creates is deleted, on both
 * sides, even on a mid-sequence failure.
 */

#if UCRTDIFF_REF_HAS_fopen || UCRTDIFF_REF_HAS_freopen || UCRTDIFF_REF_HAS__fdopen ||           \
    UCRTDIFF_REF_HAS_fclose || UCRTDIFF_REF_HAS__fcloseall || UCRTDIFF_REF_HAS_fflush ||        \
    UCRTDIFF_REF_HAS_fread || UCRTDIFF_REF_HAS_fwrite || UCRTDIFF_REF_HAS_fseek ||              \
    UCRTDIFF_REF_HAS_ftell || UCRTDIFF_REF_HAS_fgetpos || UCRTDIFF_REF_HAS_fsetpos ||           \
    UCRTDIFF_REF_HAS_rewind || UCRTDIFF_REF_HAS_setbuf || UCRTDIFF_REF_HAS_setvbuf ||           \
    UCRTDIFF_REF_HAS_clearerr || UCRTDIFF_REF_HAS_feof || UCRTDIFF_REF_HAS__fileno ||           \
    UCRTDIFF_REF_HAS__setmaxstdio || UCRTDIFF_REF_HAS___p__commode ||                           \
    UCRTDIFF_REF_HAS__set_printf_count_output

/* ---- a temp directory both sides can create files in ---------------------- */

static char g_sf_dir[MAX_PATH];

static int sf_dir(void)
{
    if (g_sf_dir[0])
        return 1;
    char probe[MAX_PATH + 32];
    DWORD n = GetTempPathA(MAX_PATH - 32, g_sf_dir);
    if (n && n < MAX_PATH - 32) {
        lstrcpyA(probe, g_sf_dir);
        lstrcatA(probe, "ucrtdiff_sf.probe");
        HANDLE h = CreateFileA(probe, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
        if (h != INVALID_HANDLE_VALUE) {
            CloseHandle(h);
            DeleteFileA(probe);
            return 1;
        }
    }
    lstrcpyA(g_sf_dir, "C:\\");
    return 1;
}

/* `side` 0 = DLL, 1 = ref: each gets its own path, so the two CRTs never
 * fight over the same file. */
static const char *sf_path(const char *leaf, int side)
{
    static char buf[2][MAX_PATH + 48];
    sf_dir();
    lstrcpyA(buf[side], g_sf_dir);
    lstrcatA(buf[side], side ? "ucrtdiff_sf_ref_" : "ucrtdiff_sf_dll_");
    lstrcatA(buf[side], leaf);
    return buf[side];
}

static void sf_delete(const char *leaf)
{
    DeleteFileA(sf_path(leaf, 0));
    DeleteFileA(sf_path(leaf, 1));
}

/* Reads a file's whole contents back with plain Win32 ReadFile -- independent
 * of BOTH CRTs. Returns the byte count, or -1 on any failure. */
static long sf_read_back(const char *path, unsigned char *buf, unsigned long cap)
{
    HANDLE h = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
        return -1;
    DWORD n = 0;
    BOOL ok = ReadFile(h, buf, cap, &n, NULL);
    CloseHandle(h);
    return ok ? (long)n : -1;
}

/* ---- ref_ prototypes -------------------------------------------------------
 *
 * `FILE`/`fpos_t`/`size_t`/`errno_t` come from the real <stdio.h>, already
 * included above run_legacy(); only the `ref_` names are hand-declared, per
 * the harness's own convention (see the file header comment).
 */

FILE *__cdecl ref_fopen(const char *, const char *);
FILE *__cdecl ref__wfopen(const wchar_t *, const wchar_t *);
FILE *__cdecl ref__fsopen(const char *, const char *, int);
FILE *__cdecl ref__wfsopen(const wchar_t *, const wchar_t *, int);
errno_t __cdecl ref_fopen_s(FILE **, const char *, const char *);
errno_t __cdecl ref__wfopen_s(FILE **, const wchar_t *, const wchar_t *);
FILE *__cdecl ref_freopen(const char *, const char *, FILE *);
errno_t __cdecl ref_freopen_s(FILE **, const char *, const char *, FILE *);
FILE *__cdecl ref__wfreopen(const wchar_t *, const wchar_t *, FILE *);
errno_t __cdecl ref__wfreopen_s(FILE **, const wchar_t *, const wchar_t *, FILE *);
FILE *__cdecl ref__fdopen(int, const char *);
FILE *__cdecl ref__wfdopen(int, const wchar_t *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref__fclose_nolock(FILE *);
int __cdecl ref__fcloseall(void);
int __cdecl ref_fflush(FILE *);
int __cdecl ref__fflush_nolock(FILE *);
int __cdecl ref__flushall(void);
size_t __cdecl ref_fread(void *, size_t, size_t, FILE *);
size_t __cdecl ref_fread_s(void *, size_t, size_t, size_t, FILE *);
size_t __cdecl ref__fread_nolock(void *, size_t, size_t, FILE *);
size_t __cdecl ref__fread_nolock_s(void *, size_t, size_t, size_t, FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
size_t __cdecl ref__fwrite_nolock(const void *, size_t, size_t, FILE *);
int __cdecl ref_fseek(FILE *, long, int);
int __cdecl ref__fseeki64(FILE *, __int64, int);
int __cdecl ref__fseek_nolock(FILE *, long, int);
int __cdecl ref__fseeki64_nolock(FILE *, __int64, int);
long __cdecl ref_ftell(FILE *);
__int64 __cdecl ref__ftelli64(FILE *);
long __cdecl ref__ftell_nolock(FILE *);
__int64 __cdecl ref__ftelli64_nolock(FILE *);
int __cdecl ref_fgetpos(FILE *, fpos_t *);
int __cdecl ref_fsetpos(FILE *, const fpos_t *);
void __cdecl ref_rewind(FILE *);
void __cdecl ref_setbuf(FILE *, char *);
int __cdecl ref_setvbuf(FILE *, char *, int, size_t);
errno_t __cdecl ref_clearerr_s(FILE *);
void __cdecl ref_clearerr(FILE *);
int __cdecl ref_feof(FILE *);
int __cdecl ref_ferror(FILE *);
int __cdecl ref__fileno(FILE *);
int __cdecl ref__setmaxstdio(int);
int __cdecl ref__getmaxstdio(void);
int *__cdecl ref___p__commode(void);
int __cdecl ref__set_printf_count_output(int);
int __cdecl ref__get_printf_count_output(void);
int *__cdecl ref__errno(void);
unsigned long *__cdecl ref___doserrno(void);

/* ---- import pointers, the DLL side -----------------------------------------
 *
 * mingw's headers inline or alias several of these (e.g. `fopen` can resolve
 * to a `_CRT_STDIO_INLINE` wrapper in some configurations); go through the
 * import slot exactly like every other block in this harness.
 */

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern FILE *(__cdecl *const ud__wfopen)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen));
extern FILE *(__cdecl *const ud__fsopen)(const char *, const char *, int) __asm__(UD_IMP(_fsopen));
extern FILE *(__cdecl *const ud__wfsopen)(const wchar_t *, const wchar_t *, int) __asm__(UD_IMP(_wfsopen));
extern errno_t(__cdecl *const ud_fopen_s)(FILE **, const char *, const char *) __asm__(UD_IMP(fopen_s));
extern errno_t(__cdecl *const ud__wfopen_s)(FILE **, const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen_s));
extern FILE *(__cdecl *const ud_freopen)(const char *, const char *, FILE *) __asm__(UD_IMP(freopen));
extern errno_t(__cdecl *const ud_freopen_s)(FILE **, const char *, const char *, FILE *) __asm__(UD_IMP(freopen_s));
extern FILE *(__cdecl *const ud__wfreopen)(const wchar_t *, const wchar_t *, FILE *) __asm__(UD_IMP(_wfreopen));
extern errno_t(__cdecl *const ud__wfreopen_s)(FILE **, const wchar_t *, const wchar_t *, FILE *) __asm__(UD_IMP(_wfreopen_s));
extern FILE *(__cdecl *const ud__fdopen)(int, const char *) __asm__(UD_IMP(_fdopen));
extern FILE *(__cdecl *const ud__wfdopen)(int, const wchar_t *) __asm__(UD_IMP(_wfdopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern int(__cdecl *const ud__fclose_nolock)(FILE *) __asm__(UD_IMP(_fclose_nolock));
extern int(__cdecl *const ud__fcloseall)(void) __asm__(UD_IMP(_fcloseall));
extern int(__cdecl *const ud_fflush)(FILE *) __asm__(UD_IMP(fflush));
extern int(__cdecl *const ud__fflush_nolock)(FILE *) __asm__(UD_IMP(_fflush_nolock));
extern int(__cdecl *const ud__flushall)(void) __asm__(UD_IMP(_flushall));
extern size_t(__cdecl *const ud_fread)(void *, size_t, size_t, FILE *) __asm__(UD_IMP(fread));
extern size_t(__cdecl *const ud_fread_s)(void *, size_t, size_t, size_t, FILE *) __asm__(UD_IMP(fread_s));
extern size_t(__cdecl *const ud__fread_nolock)(void *, size_t, size_t, FILE *) __asm__(UD_IMP(_fread_nolock));
extern size_t(__cdecl *const ud__fread_nolock_s)(void *, size_t, size_t, size_t, FILE *) __asm__(UD_IMP(_fread_nolock_s));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));
extern size_t(__cdecl *const ud__fwrite_nolock)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(_fwrite_nolock));
extern int(__cdecl *const ud_fseek)(FILE *, long, int) __asm__(UD_IMP(fseek));
extern int(__cdecl *const ud__fseeki64)(FILE *, __int64, int) __asm__(UD_IMP(_fseeki64));
extern int(__cdecl *const ud__fseek_nolock)(FILE *, long, int) __asm__(UD_IMP(_fseek_nolock));
extern int(__cdecl *const ud__fseeki64_nolock)(FILE *, __int64, int) __asm__(UD_IMP(_fseeki64_nolock));
extern long(__cdecl *const ud_ftell)(FILE *) __asm__(UD_IMP(ftell));
extern __int64(__cdecl *const ud__ftelli64)(FILE *) __asm__(UD_IMP(_ftelli64));
extern long(__cdecl *const ud__ftell_nolock)(FILE *) __asm__(UD_IMP(_ftell_nolock));
extern __int64(__cdecl *const ud__ftelli64_nolock)(FILE *) __asm__(UD_IMP(_ftelli64_nolock));
extern int(__cdecl *const ud_fgetpos)(FILE *, fpos_t *) __asm__(UD_IMP(fgetpos));
extern int(__cdecl *const ud_fsetpos)(FILE *, const fpos_t *) __asm__(UD_IMP(fsetpos));
extern void(__cdecl *const ud_rewind)(FILE *) __asm__(UD_IMP(rewind));
extern void(__cdecl *const ud_setbuf)(FILE *, char *) __asm__(UD_IMP(setbuf));
extern int(__cdecl *const ud_setvbuf)(FILE *, char *, int, size_t) __asm__(UD_IMP(setvbuf));
extern errno_t(__cdecl *const ud_clearerr_s)(FILE *) __asm__(UD_IMP(clearerr_s));
extern void(__cdecl *const ud_clearerr)(FILE *) __asm__(UD_IMP(clearerr));
extern int(__cdecl *const ud_feof)(FILE *) __asm__(UD_IMP(feof));
extern int(__cdecl *const ud_ferror)(FILE *) __asm__(UD_IMP(ferror));
extern int(__cdecl *const ud__fileno)(FILE *) __asm__(UD_IMP(_fileno));
extern int(__cdecl *const ud__setmaxstdio)(int) __asm__(UD_IMP(_setmaxstdio));
extern int(__cdecl *const ud__getmaxstdio)(void) __asm__(UD_IMP(_getmaxstdio));
extern int *(__cdecl *const ud___p__commode)(void) __asm__(UD_IMP(__p__commode));
extern int(__cdecl *const ud__set_printf_count_output)(int) __asm__(UD_IMP(_set_printf_count_output));
extern int(__cdecl *const ud__get_printf_count_output)(void) __asm__(UD_IMP(_get_printf_count_output));

#define SF_ERRNO_SENTINEL 777
#define SF_RESET() do { *_errno() = SF_ERRNO_SENTINEL; *ref__errno() = SF_ERRNO_SENTINEL; g_iph_dll = g_iph_ref = 0; } while (0)
#define SF_GRADE(fam, lbl, r1, r2) do {                                            \
        grade2(fam ".ret", lbl, (long long)(r1), (long long)(r2));                \
        grade2(fam ".errno", lbl, *_errno(), *ref__errno());                      \
        grade2(fam ".iph", lbl, g_iph_dll, g_iph_ref);                            \
    } while (0)

/* ==== openfile.cpp / fopen.cpp: the mode-string table + basic open/close === */

#if UCRTDIFF_REF_HAS_fopen

/* Each row opens a FRESH per-side file with `mode`, grades NULL-ness/errno/
 * the resulting stream's ANSI-ness via fputc-style validation (feof/ferror
 * both start false), then closes and deletes on success. The file need not
 * pre-exist for "w"/"a" modes; for "r" modes we pre-create it with 5 bytes. */
static void sf_mode_row(const char *mode, int precreate)
{
    const char *p1 = sf_path("mode.tmp", 0);
    const char *p2 = sf_path("mode.tmp", 1);
    DeleteFileA(p1);
    DeleteFileA(p2);
    if (precreate) {
        HANDLE h1 = CreateFileA(p1, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
        HANDLE h2 = CreateFileA(p2, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
        DWORD w;
        if (h1 != INVALID_HANDLE_VALUE) { WriteFile(h1, "12345", 5, &w, NULL); CloseHandle(h1); }
        if (h2 != INVALID_HANDLE_VALUE) { WriteFile(h2, "12345", 5, &w, NULL); CloseHandle(h2); }
    }

    SF_RESET();
    FILE *f1 = ud_fopen(p1, mode);
    FILE *f2 = ref_fopen(p2, mode);
    SF_GRADE("fopen.mode", mode, f1 != NULL, f2 != NULL);
    if (f1 && f2) {
        grade2("fopen.mode.feof", mode, ud_feof(f1), ref_feof(f2));
        grade2("fopen.mode.ferror", mode, ud_ferror(f1), ref_ferror(f2));
    }
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(p1);
    DeleteFileA(p2);
}

static void run_sf_fopen_mode(void)
{
    install_counting_iph();
    /* valid combinations */
    sf_mode_row("r", 1);
    sf_mode_row("rb", 1);
    sf_mode_row("rt", 1);
    sf_mode_row("r+", 1);
    sf_mode_row("w", 0);
    sf_mode_row("wb", 0);
    sf_mode_row("w+", 0);
    sf_mode_row("a", 0);
    sf_mode_row("a+", 0);
    sf_mode_row("wc", 0);
    sf_mode_row("wn", 0);
    sf_mode_row("wS", 0);
    sf_mode_row("wR", 0);
    sf_mode_row("wT", 0);
    sf_mode_row("wD", 0);
    sf_mode_row("wN", 0);
    sf_mode_row("wx", 0);          /* 'x' after 'w' (implies _O_TRUNC): valid */
    sf_mode_row("w, ccs=UTF-8", 0);
    sf_mode_row("w, ccs=UTF-16LE", 0);
    sf_mode_row("w, ccs=UNICODE", 0);
    sf_mode_row("w,ccs=utf-8", 0); /* case-insensitive encoding name */
    /* invalid combinations: NULL + EINVAL + handler expected */
    sf_mode_row("q", 0);           /* not r/w/a */
    sf_mode_row("", 0);            /* empty: falls through to invalid first char */
    sf_mode_row("r+r", 0);         /* duplicate 'r' after first char -> unknown flag */
    sf_mode_row("w++", 0);         /* duplicate '+' */
    sf_mode_row("wbt", 0);         /* both b and t */
    sf_mode_row("wcn", 0);         /* both c and n (commit mode) */
    sf_mode_row("wSR", 0);         /* both S and R (scan mode) */
    sf_mode_row("rx", 0);          /* 'x' without preceding 'w' (_O_TRUNC unset) */
    sf_mode_row("w, ccs=BOGUS", 0);
    sf_mode_row("w, xyz=UTF-8", 0); /* not "ccs" */

    /* fopen's own extra validation: NULL args, empty file name. */
    SF_RESET();
    FILE *f1 = ud_fopen(NULL, "r");
    FILE *f2 = ref_fopen(NULL, "r");
    SF_GRADE("fopen.nullname", "-", f1 != NULL, f2 != NULL);

    SF_RESET();
    f1 = ud_fopen(sf_path("mode.tmp", 0), NULL);
    f2 = ref_fopen(sf_path("mode.tmp", 1), NULL);
    SF_GRADE("fopen.nullmode", "-", f1 != NULL, f2 != NULL);

    SF_RESET();
    f1 = ud_fopen(sf_path("mode.tmp", 0), "");
    f2 = ref_fopen(sf_path("mode.tmp", 1), "");
    SF_GRADE("fopen.emptymode", "-", f1 != NULL, f2 != NULL);

    /* _VALIDATE_RETURN_NOEXC: an empty file name sets errno but does NOT
     * invoke the handler -- iph count must stay 0 on both sides. */
    SF_RESET();
    f1 = ud_fopen("", "r");
    f2 = ref_fopen("", "r");
    SF_GRADE("fopen.emptyname", "-", f1 != NULL, f2 != NULL);

    /* fopen_s: result-pointer contract. */
    SF_RESET();
    FILE *r1 = (FILE *)0x1, *r2 = (FILE *)0x1;
    errno_t e1 = ud_fopen_s(&r1, sf_path("mode.tmp", 0), "w");
    errno_t e2 = ref_fopen_s(&r2, sf_path("mode.tmp", 1), "w");
    grade2("fopen_s.ret", "-", e1, e2);
    grade2("fopen_s.nonnull", "-", r1 != NULL, r2 != NULL);
    if (r1) ud_fclose(r1);
    if (r2) ref_fclose(r2);

    SF_RESET();
    e1 = ud_fopen_s(NULL, sf_path("mode.tmp", 0), "w");
    e2 = ref_fopen_s(NULL, sf_path("mode.tmp", 1), "w");
    SF_GRADE("fopen_s.nullresult", "-", e1, e2);

    sf_delete("mode.tmp");
}

/* fopen/fread/fwrite roundtrip: write known bytes with the DLL's fwrite,
 * close, reopen, read back with fread, and separately verify with a raw
 * Win32 ReadFile that the ON-DISK bytes match between the two CRTs (text vs.
 * binary mode CRLF translation is the whole point of this row). */
static void sf_roundtrip(const char *mode_w, const char *mode_r, const char *data, size_t n)
{
    const char *leaf = "rt.tmp";
    sf_delete(leaf);

    SF_RESET();
    FILE *w1 = ud_fopen(sf_path(leaf, 0), mode_w);
    FILE *w2 = ref_fopen(sf_path(leaf, 1), mode_w);
    if (!w1 || !w2) { grade2("fopen.roundtrip.open", mode_w, w1 != NULL, w2 != NULL); goto done; }

    {
        size_t wr1 = ud_fwrite(data, 1, n, w1);
        size_t wr2 = ref_fwrite(data, 1, n, w2);
        grade2("fwrite.roundtrip.count", mode_w, (long long)wr1, (long long)wr2);
        ud_fclose(w1);
        ref_fclose(w2);
    }

    {
        unsigned char b1[256], b2[256];
        memset(b1, 0xCD, sizeof b1);
        memset(b2, 0xCD, sizeof b2);
        long n1 = sf_read_back(sf_path(leaf, 0), b1, sizeof b1);
        long n2 = sf_read_back(sf_path(leaf, 1), b2, sizeof b2);
        grade2("fwrite.roundtrip.bytes", mode_w, n1, n2);
        if (n1 >= 0 && n2 >= 0 && n1 == n2)
            grade2("fwrite.roundtrip.hash", mode_w, fnv(b1, (size_t)n1), fnv(b2, (size_t)n2));
    }

    {
        FILE *r1 = ud_fopen(sf_path(leaf, 0), mode_r);
        FILE *r2 = ref_fopen(sf_path(leaf, 1), mode_r);
        if (r1 && r2) {
            unsigned char b1[256], b2[256];
            memset(b1, 0xCD, sizeof b1);
            memset(b2, 0xCD, sizeof b2);
            size_t rd1 = ud_fread(b1, 1, sizeof b1, r1);
            size_t rd2 = ref_fread(b2, 1, sizeof b2, r2);
            grade2("fread.roundtrip.count", mode_r, (long long)rd1, (long long)rd2);
            grade2("fread.roundtrip.hash", mode_r, fnv(b1, rd1), fnv(b2, rd2));
            grade2("fread.roundtrip.feof", mode_r, ud_feof(r1), ref_feof(r2));
        }
        if (r1) ud_fclose(r1);
        if (r2) ref_fclose(r2);
    }

done:
    sf_delete(leaf);
}

static void run_sf_roundtrip(void)
{
    install_counting_iph();
    sf_roundtrip("wb", "rb", "line1\nline2\nline3", 18);       /* binary: no translation */
    sf_roundtrip("w", "r", "line1\nline2\nline3", 18);          /* text: \n -> \r\n on write */
    sf_roundtrip("w", "rb", "a\nb\nc\n", 6);                    /* write text, read binary: see the CRLF */
}

#endif /* UCRTDIFF_REF_HAS_fopen */

/* ==== freopen.cpp ========================================================= */

#if UCRTDIFF_REF_HAS_freopen

static void run_sf_freopen(void)
{
    install_counting_iph();
    const char *leaf_a = "freopen_a.tmp", *leaf_b = "freopen_b.tmp";
    sf_delete(leaf_a);
    sf_delete(leaf_b);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf_a, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf_a, 1), "w");

    SF_RESET();
    FILE *g1 = ud_freopen(sf_path(leaf_b, 0), "w", f1);
    FILE *g2 = ref_freopen(sf_path(leaf_b, 1), "w", f2);
    SF_GRADE("freopen.reopen", "-", g1 == f1, g2 == f2); /* freopen reuses the same FILE* */
    if (g1) { ud_fwrite("hi", 1, 2, g1); ud_fclose(g1); }
    if (g2) { ref_fwrite("hi", 1, 2, g2); ref_fclose(g2); }

    unsigned char b1[8], b2[8];
    long n1 = sf_read_back(sf_path(leaf_b, 0), b1, sizeof b1);
    long n2 = sf_read_back(sf_path(leaf_b, 1), b2, sizeof b2);
    grade2("freopen.reopen.bytes", "-", n1, n2);

    /* NULL file_name: EBADF, no handler (`_VALIDATE_RETURN_ERRCODE_NOEXC`). */
    SF_RESET();
    FILE *result1 = (FILE *)0x1, *result2 = (FILE *)0x1;
    errno_t e1 = ud_freopen_s(&result1, NULL, "w", NULL);
    errno_t e2 = ref_freopen_s(&result2, NULL, "w", NULL);
    grade2("freopen_s.nullname", "-", e1, e2);
    grade2("freopen_s.nullname.iph", "-", g_iph_dll, g_iph_ref);

    /* NULL result pointer: EINVAL, handler invoked. */
    SF_RESET();
    e1 = ud_freopen_s(NULL, sf_path(leaf_a, 0), "w", NULL);
    e2 = ref_freopen_s(NULL, sf_path(leaf_a, 1), "w", NULL);
    grade2("freopen_s.nullresult", "-", e1, e2);
    grade2("freopen_s.nullresult.iph", "-", g_iph_dll, g_iph_ref);

    sf_delete(leaf_a);
    sf_delete(leaf_b);
}

#endif /* UCRTDIFF_REF_HAS_freopen */

/* ==== fdopen.cpp =========================================================== */

#if UCRTDIFF_REF_HAS__fdopen

static void run_sf_fdopen(void)
{
    install_counting_iph();
    const char *leaf = "fdopen.tmp";
    sf_delete(leaf);

    /* `_fdopen` needs a lowio handle opened by the SAME CRT instance (it
     * reads `_osfile`/`_nhandle` from that instance's own table), so each
     * side gets its own handle via ITS OWN `fopen` + `_fileno` -- exercising
     * the valid path without a separate lowio `_open` declaration. */
    SF_RESET();
    FILE *base1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *base2 = ref_fopen(sf_path(leaf, 1), "w");
    int fd1 = base1 ? ud__fileno(base1) : -1;
    int fd2 = base2 ? ref__fileno(base2) : -1;
    FILE *f1 = fd1 >= 0 ? ud__fdopen(fd1, "w") : NULL;
    FILE *f2 = fd2 >= 0 ? ref__fdopen(fd2, "w") : NULL;
    SF_GRADE("_fdopen.valid", "-", f1 != NULL, f2 != NULL);
    /* `f1`/`base1` are now the SAME underlying stream slot (fdopen doesn't
     * allocate a new lowio handle); close once. */
    if (f1) { ud_fwrite("ab", 1, 2, f1); ud_fclose(f1); } else if (base1) ud_fclose(base1);
    if (f2) { ref_fwrite("ab", 1, 2, f2); ref_fclose(f2); } else if (base2) ref_fclose(base2);

    /* Invalid handle: -1 hits `_CHECK_FH_RETURN`-adjacent EBADF path on both
     * sides identically (no real lowio table lookup needed for -1/-2). */
    SF_RESET();
    FILE *bad1 = ud__fdopen(-1, "r");
    FILE *bad2 = ref__fdopen(-1, "r");
    SF_GRADE("_fdopen.badfh", "-1", bad1 != NULL, bad2 != NULL);

    SF_RESET();
    bad1 = ud__fdopen(99999, "r");
    bad2 = ref__fdopen(99999, "r");
    SF_GRADE("_fdopen.oorfh", "99999", bad1 != NULL, bad2 != NULL);

    SF_RESET();
    bad1 = ud__fdopen(0, NULL);
    bad2 = ref__fdopen(0, NULL);
    SF_GRADE("_fdopen.nullmode", "-", bad1 != NULL, bad2 != NULL);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS__fdopen */

/* ==== fclose.cpp / closeall.cpp ============================================ */

#if UCRTDIFF_REF_HAS_fclose

static void run_sf_fclose(void)
{
    install_counting_iph();
    const char *leaf = "fclose.tmp";
    sf_delete(leaf);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "w");
    int c1 = f1 ? ud_fclose(f1) : -2;
    int c2 = f2 ? ref_fclose(f2) : -2;
    SF_GRADE("fclose.ok", "-", c1, c2);

    /* NULL stream: EINVAL, handler invoked, returns EOF (-1). */
    SF_RESET();
    c1 = ud_fclose(NULL);
    c2 = ref_fclose(NULL);
    SF_GRADE("fclose.null", "-", c1, c2);

    /* NOTE: closing an already-closed/freed FILE* is UB in the C and is NOT
     * exercised here (per the batch brief: "fclose of a freed stream is UB --
     * do not"). */

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fclose */

#if UCRTDIFF_REF_HAS__fcloseall

static void run_sf_fcloseall(void)
{
    install_counting_iph();
    const char *leafs[3] = { "ca0.tmp", "ca1.tmp", "ca2.tmp" };
    for (int i = 0; i < 3; i++)
        sf_delete(leafs[i]);

    SF_RESET();
    FILE *f1[3], *f2[3];
    for (int i = 0; i < 3; i++) {
        f1[i] = ud_fopen(sf_path(leafs[i], 0), "w");
        f2[i] = ref_fopen(sf_path(leafs[i], 1), "w");
    }

    int n1 = ud__fcloseall();
    int n2 = ref__fcloseall();
    grade2("_fcloseall.count", "-", n1, n2);

    for (int i = 0; i < 3; i++)
        sf_delete(leafs[i]);
}

#endif /* UCRTDIFF_REF_HAS__fcloseall */

/* ==== fflush.cpp ============================================================ */

#if UCRTDIFF_REF_HAS_fflush

static void run_sf_fflush(void)
{
    install_counting_iph();
    const char *leaf = "fflush.tmp";
    sf_delete(leaf);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "w");
    if (f1) ud_fwrite("xy", 1, 2, f1);
    if (f2) ref_fwrite("xy", 1, 2, f2);

    int r1 = f1 ? ud_fflush(f1) : -1;
    int r2 = f2 ? ref_fflush(f2) : -1;
    SF_GRADE("fflush.write", "-", r1, r2);

    /* Bytes should now be on disk even without closing. */
    unsigned char b1[8], b2[8];
    long n1 = sf_read_back(sf_path(leaf, 0), b1, sizeof b1);
    long n2 = sf_read_back(sf_path(leaf, 1), b2, sizeof b2);
    grade2("fflush.write.bytes", "-", n1, n2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* NULL stream: fflush(NULL) flushes-all and returns 0/EOF; no handler. */
    SF_RESET();
    r1 = ud_fflush(NULL);
    r2 = ref_fflush(NULL);
    grade2("fflush.null.iph", "-", g_iph_dll, g_iph_ref);
    (void)r1; (void)r2; /* the return is a global "did everything flush" summary, not comparable 1:1 across two independent stream tables */

    /* A read-mode stream's flush is defined as a no-op (0), never touching
     * the file position. */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "r");
    f2 = ref_fopen(sf_path(leaf, 1), "r");
    if (f1) { ud_fread(b1, 1, 1, f1); r1 = ud_fflush(f1); }
    if (f2) { ref_fread(b2, 1, 1, f2); r2 = ref_fflush(f2); }
    SF_GRADE("fflush.readmode", "-", r1, r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fflush */

/* ==== fread.cpp ============================================================= */

#if UCRTDIFF_REF_HAS_fread

static void sf_write_fixture(const char *leaf, const void *data, size_t n)
{
    sf_delete(leaf);
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "wb");
    if (f1) { ud_fwrite(data, 1, n, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite(data, 1, n, f2); ref_fclose(f2); }
}

static void run_sf_fread(void)
{
    install_counting_iph();
    const unsigned char fixture[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    const char *leaf = "fread.tmp";
    sf_write_fixture(leaf, fixture, sizeof fixture);

    /* Whole-element reads: 10 bytes as 5 elements of 2. */
    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");
    unsigned char b1[16], b2[16];
    memset(b1, 0xCD, sizeof b1); memset(b2, 0xCD, sizeof b2);
    size_t r1 = f1 ? ud_fread(b1, 2, 5, f1) : (size_t)-1;
    size_t r2 = f2 ? ref_fread(b2, 2, 5, f2) : (size_t)-1;
    grade2("fread.exact.count", "-", (long long)r1, (long long)r2);
    grade2("fread.exact.hash", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* Partial element: 10 bytes requested as 3 elements of size 4 -> only 2
     * whole elements (8 bytes) fit; the trailing 2 bytes are consumed from
     * the stream but NOT counted (fread returns whole-element count only). */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "rb");
    f2 = ref_fopen(sf_path(leaf, 1), "rb");
    memset(b1, 0xCD, sizeof b1); memset(b2, 0xCD, sizeof b2);
    r1 = f1 ? ud_fread(b1, 4, 3, f1) : (size_t)-1;
    r2 = f2 ? ref_fread(b2, 4, 3, f2) : (size_t)-1;
    grade2("fread.partial.count", "-", (long long)r1, (long long)r2);
    grade2("fread.partial.hash", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));
    /* Compared across the sides: short of the requested count, both reach EOF. */
    grade2("fread.partial.feof", "-", f1 ? ud_feof(f1) : -1, f2 ? ref_feof(f2) : -1);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* Past EOF entirely: 0 elements, feof set. */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "rb");
    f2 = ref_fopen(sf_path(leaf, 1), "rb");
    if (f1) ud_fseek(f1, 0, 2 /* SEEK_END */);
    if (f2) ref_fseek(f2, 0, 2);
    memset(b1, 0xCD, sizeof b1); memset(b2, 0xCD, sizeof b2);
    r1 = f1 ? ud_fread(b1, 1, 4, f1) : (size_t)-1;
    r2 = f2 ? ref_fread(b2, 1, 4, f2) : (size_t)-1;
    grade2("fread.ateof.count", "-", (long long)r1, (long long)r2);
    grade2("fread.ateof.feof", "-", f1 ? ud_feof(f1) : -1, f2 ? ref_feof(f2) : -1);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* fread_s buffer-too-small: ERANGE, zero-filled buffer, return 0. */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "rb");
    f2 = ref_fopen(sf_path(leaf, 1), "rb");
    memset(b1, 0xCD, 8); memset(b2, 0xCD, 8);
    r1 = f1 ? ud_fread_s(b1, 4, 1, 8, f1) : (size_t)-1;
    r2 = f2 ? ref_fread_s(b2, 4, 1, 8, f2) : (size_t)-1;
    SF_GRADE("fread_s.toosmall", "-", r1, r2);
    grade2("fread_s.toosmall.hash", "-", fnv(b1, 8), fnv(b2, 8)); /* both zero-filled */
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* NULL stream, NULL buffer: EINVAL, handler invoked. */
    SF_RESET();
    r1 = ud_fread(b1, 1, 1, NULL);
    r2 = ref_fread(b2, 1, 1, NULL);
    SF_GRADE("fread.nullstream", "-", r1, r2);

    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "rb");
    f2 = ref_fopen(sf_path(leaf, 1), "rb");
    r1 = f1 ? ud_fread(NULL, 1, 1, f1) : (size_t)-1;
    r2 = f2 ? ref_fread(NULL, 1, 1, f2) : (size_t)-1;
    SF_GRADE("fread.nullbuf", "-", r1, r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fread */

/* ==== fwrite.cpp ============================================================ */

#if UCRTDIFF_REF_HAS_fwrite

static void run_sf_fwrite(void)
{
    install_counting_iph();
    const char *leaf = "fwrite.tmp";

    /* Large write that bypasses the buffer entirely (bigger than the
     * internal 4096-byte buffer). */
    static unsigned char big[9000];
    for (size_t i = 0; i < sizeof big; i++)
        big[i] = (unsigned char)(i * 7 + 1);

    sf_delete(leaf);
    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "wb");
    size_t r1 = f1 ? ud_fwrite(big, 1, sizeof big, f1) : (size_t)-1;
    size_t r2 = f2 ? ref_fwrite(big, 1, sizeof big, f2) : (size_t)-1;
    grade2("fwrite.big.count", "-", (long long)r1, (long long)r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    unsigned char rb1[9000], rb2[9000];
    long n1 = sf_read_back(sf_path(leaf, 0), rb1, sizeof rb1);
    long n2 = sf_read_back(sf_path(leaf, 1), rb2, sizeof rb2);
    grade2("fwrite.big.bytes", "-", n1, n2);
    if (n1 == n2 && n1 > 0)
        grade2("fwrite.big.hash", "-", fnv(rb1, (size_t)n1), fnv(rb2, (size_t)n2));

    /* fwrite then fread WITHOUT an intervening seek on a "w+" update stream:
     * the C requires a reposition between a write and a read; this is
     * documented UB territory, but the CONTRACT here is what `_IOERROR`/the
     * return value do, which is deterministic in this implementation
     * (`_cnt`'s buffered-read branch sees `_IOREAD` unset after a write, so
     * it takes the refill path, which is well-defined). */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "w+b");
    f2 = ref_fopen(sf_path(leaf, 1), "w+b");
    if (f1) ud_fwrite("hello", 1, 5, f1);
    if (f2) ref_fwrite("hello", 1, 5, f2);
    unsigned char b1[8], b2[8];
    memset(b1, 0xCD, sizeof b1); memset(b2, 0xCD, sizeof b2);
    r1 = f1 ? ud_fread(b1, 1, 5, f1) : (size_t)-1;
    r2 = f2 ? ref_fread(b2, 1, 5, f2) : (size_t)-1;
    grade2("fwrite.then.fread.noseek.count", "-", (long long)r1, (long long)r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* NULL stream / NULL buffer: EINVAL, handler invoked, 0 returned. */
    SF_RESET();
    r1 = ud_fwrite("x", 1, 1, NULL);
    r2 = ref_fwrite("x", 1, 1, NULL);
    SF_GRADE("fwrite.nullstream", "-", r1, r2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fwrite */

/* ==== fseek.cpp / ftell.cpp / fgetpos.cpp / fsetpos.cpp / rewind.cpp ======== */

#if UCRTDIFF_REF_HAS_fseek || UCRTDIFF_REF_HAS_ftell || UCRTDIFF_REF_HAS_fgetpos || UCRTDIFF_REF_HAS_fsetpos || UCRTDIFF_REF_HAS_rewind

static void sf_seek_fixture(const char *leaf, const char *mode, const char *text, size_t n)
{
    sf_delete(leaf);
    FILE *f1 = ud_fopen(sf_path(leaf, 0), mode);
    FILE *f2 = ref_fopen(sf_path(leaf, 1), mode);
    if (f1) { ud_fwrite(text, 1, n, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite(text, 1, n, f2); ref_fclose(f2); }
}

#endif

#if UCRTDIFF_REF_HAS_fseek

static void run_sf_fseek(void)
{
    install_counting_iph();
    const char *leaf = "fseek.tmp";
    sf_seek_fixture(leaf, "wb", "0123456789", 10);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");

    /* SEEK_SET into the middle, then SEEK_CUR forward, then SEEK_END back. */
    int r1 = f1 ? ud_fseek(f1, 3, 0) : -1;
    int r2 = f2 ? ref_fseek(f2, 3, 0) : -1;
    SF_GRADE("fseek.set", "-", r1, r2);

    unsigned char b1[2], b2[2];
    if (f1) ud_fread(b1, 1, 1, f1);
    if (f2) ref_fread(b2, 1, 1, f2);
    grade2("fseek.set.byte", "-", f1 ? b1[0] : 0, f2 ? b2[0] : 0);

    r1 = f1 ? ud_fseek(f1, 2, 1) : -1; /* SEEK_CUR */
    r2 = f2 ? ref_fseek(f2, 2, 1) : -1;
    SF_GRADE("fseek.cur", "-", r1, r2);
    if (f1) ud_fread(b1, 1, 1, f1);
    if (f2) ref_fread(b2, 1, 1, f2);
    grade2("fseek.cur.byte", "-", f1 ? b1[0] : 0, f2 ? b2[0] : 0);

    r1 = f1 ? ud_fseek(f1, -1, 2) : -1; /* SEEK_END */
    r2 = f2 ? ref_fseek(f2, -1, 2) : -1;
    SF_GRADE("fseek.end", "-", r1, r2);
    if (f1) ud_fread(b1, 1, 1, f1);
    if (f2) ref_fread(b2, 1, 1, f2);
    grade2("fseek.end.byte", "-", f1 ? b1[0] : 0, f2 ? b2[0] : 0);

    /* Seek beyond EOF: succeeds (files can have holes); a subsequent read
     * returns 0/EOF, not an error. */
    r1 = f1 ? ud_fseek(f1, 100, 0) : -1;
    r2 = f2 ? ref_fseek(f2, 100, 0) : -1;
    SF_GRADE("fseek.beyondeof", "-", r1, r2);
    size_t rd1 = f1 ? ud_fread(b1, 1, 1, f1) : (size_t)-1;
    size_t rd2 = f2 ? ref_fread(b2, 1, 1, f2) : (size_t)-1;
    grade2("fseek.beyondeof.read", "-", (long long)rd1, (long long)rd2);

    /* Invalid whence: EINVAL, handler invoked. */
    SF_RESET();
    r1 = f1 ? ud_fseek(f1, 0, 99) : -1;
    r2 = f2 ? ref_fseek(f2, 0, 99) : -1;
    SF_GRADE("fseek.badwhence", "-", r1, r2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* NULL stream. */
    SF_RESET();
    r1 = ud_fseek(NULL, 0, 0);
    r2 = ref_fseek(NULL, 0, 0);
    SF_GRADE("fseek.nullstream", "-", r1, r2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fseek */

#if UCRTDIFF_REF_HAS_ftell

static void run_sf_ftell(void)
{
    install_counting_iph();

    /* Binary mode: ftell tracks raw byte position exactly. */
    const char *leaf = "ftell_bin.tmp";
    sf_seek_fixture(leaf, "wb", "0123456789", 10);
    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");
    unsigned char b[4];
    if (f1) ud_fread(b, 1, 4, f1);
    if (f2) ref_fread(b, 1, 4, f2);
    long t1 = f1 ? ud_ftell(f1) : -1;
    long t2 = f2 ? ref_ftell(f2) : -1;
    grade2("ftell.bin.pos", "-", t1, t2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf);

    /* Text mode with embedded newlines: ftell must account for the \n -> \r\n
     * expansion that happened on write, i.e. the position is the FILE offset
     * (post-translation), not the logical character count. */
    const char *leaf2 = "ftell_txt.tmp";
    sf_seek_fixture(leaf2, "w", "ab\ncd\nef", 8); /* logical 8 chars, 2 newlines */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf2, 0), "r");
    f2 = ref_fopen(sf_path(leaf2, 1), "r");
    /* Read past both embedded newlines (5 logical chars: 'a','b','\n','c','d'). */
    if (f1) ud_fread(b, 1, 4, f1);
    if (f2) ref_fread(b, 1, 4, f2);
    t1 = f1 ? ud_ftell(f1) : -1;
    t2 = f2 ? ref_ftell(f2) : -1;
    grade2("ftell.text.pos.mid", "-", t1, t2);

    /* Read to EOF, then ftell (exercises the "seek to end, scan buffer for
     * newlines" reconstruction path). */
    unsigned char rest[16];
    if (f1) ud_fread(rest, 1, sizeof rest, f1);
    if (f2) ref_fread(rest, 1, sizeof rest, f2);
    t1 = f1 ? ud_ftell(f1) : -1;
    t2 = f2 ? ref_ftell(f2) : -1;
    grade2("ftell.text.pos.eof", "-", t1, t2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf2);

    /* NULL stream. */
    SF_RESET();
    t1 = ud_ftell(NULL);
    t2 = ref_ftell(NULL);
    SF_GRADE("ftell.nullstream", "-", t1, t2);
}

#endif /* UCRTDIFF_REF_HAS_ftell */

#if UCRTDIFF_REF_HAS_fgetpos

static void run_sf_fgetpos(void)
{
    install_counting_iph();
    const char *leaf = "fgetpos.tmp";
    sf_seek_fixture(leaf, "wb", "0123456789", 10);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");
    unsigned char b[3];
    if (f1) ud_fread(b, 1, 3, f1);
    if (f2) ref_fread(b, 1, 3, f2);

    fpos_t p1 = -999, p2 = -999;
    int r1 = f1 ? ud_fgetpos(f1, &p1) : -1;
    int r2 = f2 ? ref_fgetpos(f2, &p2) : -1;
    SF_GRADE("fgetpos.ok", "-", r1, r2);
    grade2("fgetpos.value", "-", (long long)p1, (long long)p2);

    SF_RESET();
    r1 = ud_fgetpos(NULL, &p1);
    r2 = ref_fgetpos(NULL, &p2);
    SF_GRADE("fgetpos.nullstream", "-", r1, r2);

    SF_RESET();
    r1 = f1 ? ud_fgetpos(f1, NULL) : -1;
    r2 = f2 ? ref_fgetpos(f2, NULL) : -1;
    SF_GRADE("fgetpos.nullpos", "-", r1, r2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fgetpos */

#if UCRTDIFF_REF_HAS_fsetpos

static void run_sf_fsetpos(void)
{
    install_counting_iph();
    const char *leaf = "fsetpos.tmp";
    sf_seek_fixture(leaf, "wb", "0123456789", 10);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");

    fpos_t pos = 5;
    int r1 = f1 ? ud_fsetpos(f1, &pos) : -1;
    int r2 = f2 ? ref_fsetpos(f2, &pos) : -1;
    SF_GRADE("fsetpos.ok", "-", r1, r2);

    unsigned char b1, b2;
    if (f1) ud_fread(&b1, 1, 1, f1);
    if (f2) ref_fread(&b2, 1, 1, f2);
    grade2("fsetpos.byte", "-", f1 ? b1 : 0, f2 ? b2 : 0);

    SF_RESET();
    r1 = ud_fsetpos(NULL, &pos);
    r2 = ref_fsetpos(NULL, &pos);
    SF_GRADE("fsetpos.nullstream", "-", r1, r2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_fsetpos */

#if UCRTDIFF_REF_HAS_rewind

static void run_sf_rewind(void)
{
    install_counting_iph();
    const char *leaf = "rewind.tmp";
    sf_seek_fixture(leaf, "wb", "0123456789", 10);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");
    unsigned char b[4];
    if (f1) ud_fread(b, 1, 4, f1);
    if (f2) ref_fread(b, 1, 4, f2);

    /* Force an error flag, then verify rewind clears it (per the C, rewind
     * clears _IOERROR/_IOEOF unconditionally). */
    if (f1) { ud_fseek(f1, 0, 2); ud_fread(b, 1, 1, f1); } /* sets EOF */
    if (f2) { ref_fseek(f2, 0, 2); ref_fread(b, 1, 1, f2); }

    if (f1) ud_rewind(f1);
    if (f2) ref_rewind(f2);
    grade2("rewind.feof", "-", f1 ? ud_feof(f1) : -1, f2 ? ref_feof(f2) : -1);
    grade2("rewind.ferror", "-", f1 ? ud_ferror(f1) : -1, f2 ? ref_ferror(f2) : -1);

    long t1 = f1 ? ud_ftell(f1) : -1;
    long t2 = f2 ? ref_ftell(f2) : -1;
    grade2("rewind.pos", "-", t1, t2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf);

    /* NULL stream: rewind is void, but must not invoke the handler with a
     * mismatched count -- graded via the iph counter alone. */
    SF_RESET();
    ud_rewind(NULL);
    ref_rewind(NULL);
    grade2("rewind.nullstream.iph", "-", g_iph_dll, g_iph_ref);
}

#endif /* UCRTDIFF_REF_HAS_rewind */

/* ==== setbuf.cpp / setvbuf.cpp ============================================== */

#if UCRTDIFF_REF_HAS_setvbuf

static void run_sf_setvbuf(void)
{
    install_counting_iph();
    const char *leaf = "setvbuf.tmp";
    sf_delete(leaf);

    static char ubuf1[128], ubuf2[128];

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "w");
    int r1 = f1 ? ud_setvbuf(f1, ubuf1, 0 /* _IOFBF */, sizeof ubuf1) : -1;
    int r2 = f2 ? ref_setvbuf(f2, ubuf2, 0, sizeof ubuf2) : -1;
    SF_GRADE("setvbuf.userbuf", "-", r1, r2);
    if (f1) { ud_fwrite("hi", 1, 2, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite("hi", 1, 2, f2); ref_fclose(f2); }

    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "w");
    f2 = ref_fopen(sf_path(leaf, 1), "w");
    r1 = f1 ? ud_setvbuf(f1, NULL, 4 /* _IONBF */, 0) : -1;
    r2 = f2 ? ref_setvbuf(f2, NULL, 4, 0) : -1;
    SF_GRADE("setvbuf.nbf", "-", r1, r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* Size below the 2-byte floor: EINVAL. */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "w");
    f2 = ref_fopen(sf_path(leaf, 1), "w");
    r1 = f1 ? ud_setvbuf(f1, ubuf1, 0, 1) : -1;
    r2 = f2 ? ref_setvbuf(f2, ubuf2, 0, 1) : -1;
    SF_GRADE("setvbuf.toosmall", "-", r1, r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* Bad `type`: EINVAL. */
    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "w");
    f2 = ref_fopen(sf_path(leaf, 1), "w");
    r1 = f1 ? ud_setvbuf(f1, ubuf1, 42, 16) : -1;
    r2 = f2 ? ref_setvbuf(f2, ubuf2, 42, 16) : -1;
    SF_GRADE("setvbuf.badtype", "-", r1, r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* NULL stream. */
    SF_RESET();
    r1 = ud_setvbuf(NULL, ubuf1, 0, 16);
    r2 = ref_setvbuf(NULL, ubuf2, 0, 16);
    SF_GRADE("setvbuf.nullstream", "-", r1, r2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_setvbuf */

#if UCRTDIFF_REF_HAS_setbuf

static void run_sf_setbuf(void)
{
    install_counting_iph();
    const char *leaf = "setbuf.tmp";
    sf_delete(leaf);
    static char ubuf1[512], ubuf2[512];

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "w");
    if (f1) ud_setbuf(f1, ubuf1);
    if (f2) ref_setbuf(f2, ubuf2);
    if (f1) { ud_fwrite("z", 1, 1, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite("z", 1, 1, f2); ref_fclose(f2); }

    unsigned char b1[4], b2[4];
    long n1 = sf_read_back(sf_path(leaf, 0), b1, sizeof b1);
    long n2 = sf_read_back(sf_path(leaf, 1), b2, sizeof b2);
    grade2("setbuf.bytes", "-", n1, n2);

    SF_RESET();
    f1 = ud_fopen(sf_path(leaf, 0), "w");
    f2 = ref_fopen(sf_path(leaf, 1), "w");
    if (f1) ud_setbuf(f1, NULL); /* -> _IONBF */
    if (f2) ref_setbuf(f2, NULL);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS_setbuf */

/* ==== clearerr.cpp / feoferr.cpp ============================================ */

#if UCRTDIFF_REF_HAS_clearerr || UCRTDIFF_REF_HAS_feof

static void run_sf_clearerr(void)
{
    install_counting_iph();
    const char *leaf = "clearerr.tmp";
    sf_seek_fixture(leaf, "wb", "abc", 3);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "rb");
    unsigned char b[8];
    if (f1) { ud_fseek(f1, 0, 2); ud_fread(b, 1, 1, f1); } /* drives EOF */
    if (f2) { ref_fseek(f2, 0, 2); ref_fread(b, 1, 1, f2); }

    grade2("feof.set", "-", f1 ? ud_feof(f1) : -1, f2 ? ref_feof(f2) : -1);
    grade2("ferror.clear", "-", f1 ? ud_ferror(f1) : -1, f2 ? ref_ferror(f2) : -1);

    if (f1) ud_clearerr(f1);
    if (f2) ref_clearerr(f2);
    grade2("clearerr.feof.after", "-", f1 ? ud_feof(f1) : -1, f2 ? ref_feof(f2) : -1);

    errno_t e1 = f1 ? ud_clearerr_s(f1) : -1;
    errno_t e2 = f2 ? ref_clearerr_s(f2) : -1;
    grade2("clearerr_s.ret", "-", e1, e2);

    SF_RESET();
    e1 = ud_clearerr_s(NULL);
    e2 = ref_clearerr_s(NULL);
    SF_GRADE("clearerr_s.nullstream", "-", e1, e2);

    /* feof/ferror on NULL: EINVAL, handler invoked, returns 0. */
    SF_RESET();
    int fe1 = ud_feof(NULL), fe2 = ref_feof(NULL);
    SF_GRADE("feof.nullstream", "-", fe1, fe2);

    SF_RESET();
    fe1 = ud_ferror(NULL); fe2 = ref_ferror(NULL);
    SF_GRADE("ferror.nullstream", "-", fe1, fe2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    sf_delete(leaf);
}

#endif

/* ==== fileno.cpp ============================================================ */

#if UCRTDIFF_REF_HAS__fileno

static void run_sf_fileno(void)
{
    install_counting_iph();
    const char *leaf = "fileno.tmp";
    sf_delete(leaf);

    SF_RESET();
    FILE *f1 = ud_fopen(sf_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sf_path(leaf, 1), "w");
    int fd1 = f1 ? ud__fileno(f1) : -1;
    int fd2 = f2 ? ref__fileno(f2) : -1;
    /* Not comparable as raw numbers (independent lowio tables); only the
     * "is a real, non-negative descriptor" contract is. */
    grade2("_fileno.valid", "-", fd1 >= 0, fd2 >= 0);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    SF_RESET();
    fd1 = ud__fileno(NULL);
    fd2 = ref__fileno(NULL);
    SF_GRADE("_fileno.nullstream", "-", fd1, fd2);

    sf_delete(leaf);
}

#endif /* UCRTDIFF_REF_HAS__fileno */

/* ==== setmaxf.cpp ============================================================ */

#if UCRTDIFF_REF_HAS__setmaxstdio

static void run_sf_setmaxstdio(void)
{
    install_counting_iph();

    int old1 = ud__getmaxstdio();
    int old2 = ref__getmaxstdio();
    grade2("_getmaxstdio.default", "-", old1, old2); /* both start at 512 (_NSTREAM_) */

    SF_RESET();
    int r1 = ud__setmaxstdio(old1 + 64);
    int r2 = ref__setmaxstdio(old2 + 64);
    SF_GRADE("_setmaxstdio.grow", "-", r1, r2);
    grade2("_getmaxstdio.after_grow", "-", ud__getmaxstdio(), ref__getmaxstdio());

    /* Shrink back to a value still >= _IOB_ENTRIES and with nothing in use
     * above it, so the shrink is expected to succeed on both sides. */
    SF_RESET();
    r1 = ud__setmaxstdio(old1);
    r2 = ref__setmaxstdio(old2);
    SF_GRADE("_setmaxstdio.shrink", "-", r1, r2);

    /* Out-of-range requests: EINVAL, handler invoked, -1 returned; restores
     * nothing since nothing changed. */
    SF_RESET();
    r1 = ud__setmaxstdio(1);   /* below _IOB_ENTRIES (3) */
    r2 = ref__setmaxstdio(1);
    SF_GRADE("_setmaxstdio.toolow", "-", r1, r2);

    SF_RESET();
    r1 = ud__setmaxstdio(999999);
    r2 = ref__setmaxstdio(999999);
    SF_GRADE("_setmaxstdio.toohigh", "-", r1, r2);

    /* Restore exactly what we found, so this runner leaves no process-wide
     * trace. */
    ud__setmaxstdio(old1);
    ref__setmaxstdio(old2);
}

#endif /* UCRTDIFF_REF_HAS__setmaxstdio */

/* ==== ncommode.cpp =========================================================== */

#if UCRTDIFF_REF_HAS___p__commode

static void run_sf_commode(void)
{
    int *p1 = ud___p__commode();
    int *p2 = ref___p__commode();
    grade2("__p__commode.default", "-", p1 ? *p1 : -1, p2 ? *p2 : -1); /* both 0: nocommit default */
}

#endif /* UCRTDIFF_REF_HAS___p__commode */

/* ==== printf_count_output.cpp ================================================ */

#if UCRTDIFF_REF_HAS__set_printf_count_output

static void run_sf_printf_count_output(void)
{
    int old1 = ud__get_printf_count_output();
    int old2 = ref__get_printf_count_output();
    grade2("_get_printf_count_output.default", "-", old1, old2); /* both 0: %n disabled by default */

    int prev1 = ud__set_printf_count_output(1);
    int prev2 = ref__set_printf_count_output(1);
    grade2("_set_printf_count_output.enable.prevstate", "-", prev1, prev2);
    grade2("_get_printf_count_output.enabled", "-", ud__get_printf_count_output(), ref__get_printf_count_output());

    prev1 = ud__set_printf_count_output(0);
    prev2 = ref__set_printf_count_output(0);
    grade2("_set_printf_count_output.disable.prevstate", "-", prev1, prev2);
    grade2("_get_printf_count_output.disabled", "-", ud__get_printf_count_output(), ref__get_printf_count_output());

    /* Restore to the state found at entry, so this runner leaves no
     * process-wide trace. */
    ud__set_printf_count_output(old1);
    ref__set_printf_count_output(old2);
}

#endif /* UCRTDIFF_REF_HAS__set_printf_count_output */

#endif /* any UCRTDIFF_REF_HAS_<files-piece export> */

/* ---- registry: paste into run_legacy()'s `legacy[]` table ------------------
 *
 * One entry per runner above; each is individually guarded, `{ "name", 0 }`
 * when its guard is 0 so the table always has the same shape regardless of
 * which files have already been deleted.
 */
#if UCRTDIFF_REF_HAS_fopen
#define SF_HAND_fopen_mode run_sf_fopen_mode
#define SF_HAND_fopen_roundtrip run_sf_roundtrip
#else
#define SF_HAND_fopen_mode 0
#define SF_HAND_fopen_roundtrip 0
#endif
#if UCRTDIFF_REF_HAS_freopen
#define SF_HAND_freopen run_sf_freopen
#else
#define SF_HAND_freopen 0
#endif
#if UCRTDIFF_REF_HAS__fdopen
#define SF_HAND_fdopen run_sf_fdopen
#else
#define SF_HAND_fdopen 0
#endif
#if UCRTDIFF_REF_HAS_fclose
#define SF_HAND_fclose run_sf_fclose
#else
#define SF_HAND_fclose 0
#endif
#if UCRTDIFF_REF_HAS__fcloseall
#define SF_HAND_fcloseall run_sf_fcloseall
#else
#define SF_HAND_fcloseall 0
#endif
#if UCRTDIFF_REF_HAS_fflush
#define SF_HAND_fflush run_sf_fflush
#else
#define SF_HAND_fflush 0
#endif
#if UCRTDIFF_REF_HAS_fread
#define SF_HAND_fread run_sf_fread
#else
#define SF_HAND_fread 0
#endif
#if UCRTDIFF_REF_HAS_fwrite
#define SF_HAND_fwrite run_sf_fwrite
#else
#define SF_HAND_fwrite 0
#endif
#if UCRTDIFF_REF_HAS_fseek
#define SF_HAND_fseek run_sf_fseek
#else
#define SF_HAND_fseek 0
#endif
#if UCRTDIFF_REF_HAS_ftell
#define SF_HAND_ftell run_sf_ftell
#else
#define SF_HAND_ftell 0
#endif
#if UCRTDIFF_REF_HAS_fgetpos
#define SF_HAND_fgetpos run_sf_fgetpos
#else
#define SF_HAND_fgetpos 0
#endif
#if UCRTDIFF_REF_HAS_fsetpos
#define SF_HAND_fsetpos run_sf_fsetpos
#else
#define SF_HAND_fsetpos 0
#endif
#if UCRTDIFF_REF_HAS_rewind
#define SF_HAND_rewind run_sf_rewind
#else
#define SF_HAND_rewind 0
#endif
#if UCRTDIFF_REF_HAS_setvbuf
#define SF_HAND_setvbuf run_sf_setvbuf
#else
#define SF_HAND_setvbuf 0
#endif
#if UCRTDIFF_REF_HAS_setbuf
#define SF_HAND_setbuf run_sf_setbuf
#else
#define SF_HAND_setbuf 0
#endif
#if UCRTDIFF_REF_HAS_clearerr || UCRTDIFF_REF_HAS_feof
#define SF_HAND_clearerr run_sf_clearerr
#else
#define SF_HAND_clearerr 0
#endif
#if UCRTDIFF_REF_HAS__fileno
#define SF_HAND_fileno run_sf_fileno
#else
#define SF_HAND_fileno 0
#endif
#if UCRTDIFF_REF_HAS__setmaxstdio
#define SF_HAND_setmaxstdio run_sf_setmaxstdio
#else
#define SF_HAND_setmaxstdio 0
#endif
#if UCRTDIFF_REF_HAS___p__commode
#define SF_HAND_commode run_sf_commode
#else
#define SF_HAND_commode 0
#endif
#if UCRTDIFF_REF_HAS__set_printf_count_output
#define SF_HAND_printf_count_output run_sf_printf_count_output
#else
#define SF_HAND_printf_count_output 0
#endif

#define STDIO_FILES_HAND_ENTRIES                                     \
    { "legacy:stdio_files-fopen_mode", SF_HAND_fopen_mode },         \
    { "legacy:stdio_files-fopen_roundtrip", SF_HAND_fopen_roundtrip }, \
    { "legacy:stdio_files-freopen", SF_HAND_freopen },               \
    { "legacy:stdio_files-fdopen", SF_HAND_fdopen },                 \
    { "legacy:stdio_files-fclose", SF_HAND_fclose },                 \
    { "legacy:stdio_files-fcloseall", SF_HAND_fcloseall },           \
    { "legacy:stdio_files-fflush", SF_HAND_fflush },                 \
    { "legacy:stdio_files-fread", SF_HAND_fread },                   \
    { "legacy:stdio_files-fwrite", SF_HAND_fwrite },                 \
    { "legacy:stdio_files-fseek", SF_HAND_fseek },                   \
    { "legacy:stdio_files-ftell", SF_HAND_ftell },                   \
    { "legacy:stdio_files-fgetpos", SF_HAND_fgetpos },               \
    { "legacy:stdio_files-fsetpos", SF_HAND_fsetpos },               \
    { "legacy:stdio_files-rewind", SF_HAND_rewind },                 \
    { "legacy:stdio_files-setvbuf", SF_HAND_setvbuf },               \
    { "legacy:stdio_files-setbuf", SF_HAND_setbuf },                 \
    { "legacy:stdio_files-clearerr", SF_HAND_clearerr },             \
    { "legacy:stdio_files-fileno", SF_HAND_fileno },                 \
    { "legacy:stdio_files-setmaxstdio", SF_HAND_setmaxstdio },       \
    { "legacy:stdio_files-commode", SF_HAND_commode },               \
    { "legacy:stdio_files-printf_count_output", SF_HAND_printf_count_output },


/* ==== staging/stdio_input.c, inlined ==== */
/*
 * staging/stdio_input.c — guest grader for the `input` piece of the stdio
 * batch: `__stdio_common_vfscanf`, `__stdio_common_vfwscanf`,
 * `__stdio_common_vsscanf`, `__stdio_common_vswscanf` (stdio/input.cpp and the
 * engine in corecrt_internal_stdio_input.h).
 *
 * Modelled on the printf block (PF_ROW/PF_TRAMP_*, ~line 1349 of ucrtdiff.c)
 * and the strtod block (FP_ROW, ~line 1798): a varargs trampoline per side (a
 * scanf entry point takes a `va_list` parameter, so only a real C-variadic
 * wrapper can hand it one), sentinel-filled receivers on both sides, graded
 * with `grade2` (no separate Rust-direct-link side exists yet -- this piece
 * is ungraded until it lands, same as printf/strtod/stat were before they did).
 *
 * ONE GUARD for the whole snippet: `UCRTDIFF_REF_HAS___stdio_common_vsscanf`.
 * All four exports live in the SAME C file (stdio/input.cpp), so they are
 * deleted together and one guard covers them; `generated-refs.h` does not
 * define it yet (`grep scanf` there is empty) -- the lead's ref-set generator
 * needs to add `__stdio_common_vsscanf`/`vswscanf`/`vfscanf`/`vfwscanf` to
 * whatever list drives it before this can be `#include`d live. Verified with
 * `llvm-nm --defined-only build-arm64/libucrt_ref.a`: all four `ref_` bodies
 * already exist in the archive, so once the guard is generated this is a
 * straight `#if 1`.
 *
 * NOTHING here touches process-wide state permanently: TZ/locale/codepage are
 * never changed, and every temp file is removed before returning.
 */

#if UCRTDIFF_REF_HAS___stdio_common_vsscanf

int __cdecl ref___stdio_common_vfscanf(unsigned long long, FILE *, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vfwscanf(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list);
int __cdecl ref___stdio_common_vsscanf(unsigned long long, const char *, size_t, const char *, _locale_t, va_list);
int __cdecl ref___stdio_common_vswscanf(unsigned long long, const wchar_t *, size_t, const wchar_t *, _locale_t, va_list);

/* Through the import pointers, exactly as the printf/strtod blocks do: mingw's
 * headers alias several of these names, so a plain call could resolve to an
 * inline rather than the real export. */
extern int(__cdecl *const ud___stdio_common_vfscanf)(unsigned long long, FILE *, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfscanf));
extern int(__cdecl *const ud___stdio_common_vfwscanf)(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfwscanf));
extern int(__cdecl *const ud___stdio_common_vsscanf)(unsigned long long, const char *, size_t, const char *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vsscanf));
extern int(__cdecl *const ud___stdio_common_vswscanf)(unsigned long long, const wchar_t *, size_t, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vswscanf));

/* A varargs trampoline per side, per shape (string source vs. stream source) --
 * the only way to hand both a genuine `va_list`. */
#define SC_TRAMP_S(name, ch)                                                                 \
    static int sfd_##name(unsigned long long o, const ch *b, size_t n, const ch *f, ...)     \
    { va_list ap; va_start(ap, f); int r = ud_##name(o, b, n, f, NULL, ap); va_end(ap); return r; } \
    static int sfr_##name(unsigned long long o, const ch *b, size_t n, const ch *f, ...)     \
    { va_list ap; va_start(ap, f); int r = ref_##name(o, b, n, f, NULL, ap); va_end(ap); return r; }
#define SC_TRAMP_F(name, ch)                                                                 \
    static int sfd_##name(unsigned long long o, FILE *s, const ch *f, ...)                   \
    { va_list ap; va_start(ap, f); int r = ud_##name(o, s, f, NULL, ap); va_end(ap); return r; } \
    static int sfr_##name(unsigned long long o, FILE *s, const ch *f, ...)                   \
    { va_list ap; va_start(ap, f); int r = ref_##name(o, s, f, NULL, ap); va_end(ap); return r; }

SC_TRAMP_S(__stdio_common_vsscanf, char)
SC_TRAMP_S(__stdio_common_vswscanf, wchar_t)
SC_TRAMP_F(__stdio_common_vfscanf, char)
SC_TRAMP_F(__stdio_common_vfwscanf, wchar_t)

#define SC_ERRNO_SENTINEL 888
#define SC_RESET() do { *_errno() = SC_ERRNO_SENTINEL; *ref__errno() = SC_ERRNO_SENTINEL; g_iph_dll = g_iph_ref = 0; } while (0)
#define SC_GRADE_SCALARS(fam, lbl, r1, r2) do {                                              \
        grade2(fam ".ret", lbl, (long long)(r1), (long long)(r2));                           \
        grade2(fam ".errno", lbl, *_errno(), *ref__errno());                                 \
        grade2(fam ".iph", lbl, g_iph_dll, g_iph_ref);                                        \
    } while (0)

/* ---- integer conversions: %d %i %o %u %x %p and the length modifiers ------
 *
 * One receiver TYPE per row (matching the format's length modifier), each
 * side's receiver pre-filled with a sentinel so an unwritten receiver shows
 * up as a MISMATCH rather than an accidental match. `secure` selects the `_s`
 * option word (`_CRT_INTERNAL_SCANF_SECURECRT`), which for a bare numeric
 * conversion changes nothing observable (no buffer-size arg to read) but is
 * still exercised so a future divergence in `secure_buffers()` handling shows.
 */
#define SC_INT_ROW(T, fmt, input, secure) do {                                               \
        T d1 = (T)0x5A; T d2 = (T)0x5A;                                                        \
        SC_RESET();                                                                          \
        int r1 = sfd___stdio_common_vsscanf((secure) ? 1ULL : 0ULL, input, (size_t)-1, fmt, &d1); \
        int r2 = sfr___stdio_common_vsscanf((secure) ? 1ULL : 0ULL, input, (size_t)-1, fmt, &d2); \
        SC_GRADE_SCALARS("scanf.int." fmt, input, r1, r2);                                   \
        grade2("scanf.int.val:" fmt, input, (long long)d1, (long long)d2);                   \
    } while (0)

static void run_sc_int(void)
{
    install_counting_iph();
    SC_INT_ROW(int, "%d", "42", 0);
    SC_INT_ROW(int, "%d", "-42", 0);
    SC_INT_ROW(int, "%d", "  +7", 0);
    SC_INT_ROW(int, "%d", "-x", 0);          /* sign consumed, no digit: the one-way-latch quirk. */
    SC_INT_ROW(int, "%d", "x", 0);           /* no sign, no digit: clean non-match. */
    SC_INT_ROW(int, "%d", "", 0);
    SC_INT_ROW(int, "%d", "2147483648", 0);  /* overflows a 32-bit signed accumulator's usual range. */
    SC_INT_ROW(int, "%d", "99999999999999999999", 0); /* overflows the 64-bit accumulator itself. */
    SC_INT_ROW(int, "%3d", "12345", 0);      /* width cap. */
    SC_INT_ROW(int, "%i", "0x1F", 0);
    SC_INT_ROW(int, "%i", "017", 0);
    SC_INT_ROW(int, "%i", "0x", 0);          /* "0x" with no hex digit: base-16 detection, no digits. */
    SC_INT_ROW(unsigned, "%o", "17", 0);
    SC_INT_ROW(unsigned, "%o", "18", 0);     /* '8' is not an octal digit: stops after "1". */
    SC_INT_ROW(unsigned, "%u", "-1", 0);     /* unsigned overflow-by-negation: no ERANGE, wraps. */
    SC_INT_ROW(unsigned, "%x", "beefCAFE", 0);
    SC_INT_ROW(unsigned, "%X", "1G", 0);
    SC_INT_ROW(signed char, "%hhd", "200", 0);
    SC_INT_ROW(short, "%hd", "70000", 0);
    SC_INT_ROW(long, "%ld", "-123456", 0);
    SC_INT_ROW(long long, "%lld", "9223372036854775807", 0);
    SC_INT_ROW(long long, "%I64d", "-5", 0);
    SC_INT_ROW(int, "%I32d", "5", 0);
    SC_INT_ROW(void *, "%p", "1234ABCD", 0);
    SC_INT_ROW(int, "%d", "42", 1);
    SC_INT_ROW(int, "%d", "-x", 1);
}

/* ---- floating point: %f %e %g %a, float vs. double ------------------------
 *
 * Bit-exact, not value-exact (`FP_BITS` lives in the strtod block, which is
 * gated by its OWN guard -- `UCRTDIFF_REF_HAS_strtod` is 0 now that strtod.rs
 * has landed and its C is gone, so that macro is compiled out entirely; this
 * piece needs its own copy, not a cross-guard dependency). */
static long long sc_fp_bits32(float f) { unsigned u; memcpy(&u, &f, 4); return (long long)u; }
static long long sc_fp_bits64(double d) { unsigned long long u; memcpy(&u, &d, 8); return (long long)u; }
#define SC_FP_BITS(v) _Generic((v), float: sc_fp_bits32(v), default: sc_fp_bits64((double)(v)))

#define SC_FLOAT_ROW(T, fmt, input) do {                                                     \
        T d1 = (T)-1.0; T d2 = (T)-1.0;                                                        \
        SC_RESET();                                                                          \
        int r1 = sfd___stdio_common_vsscanf(0, input, (size_t)-1, fmt, &d1);                 \
        int r2 = sfr___stdio_common_vsscanf(0, input, (size_t)-1, fmt, &d2);                 \
        SC_GRADE_SCALARS("scanf.float." fmt, input, r1, r2);                                 \
        grade2("scanf.float.bits:" fmt, input, SC_FP_BITS(d1), SC_FP_BITS(d2));                    \
    } while (0)

static void run_sc_float(void)
{
    install_counting_iph();
    SC_FLOAT_ROW(double, "%lf", "3.14159");
    SC_FLOAT_ROW(double, "%lf", "-2.5e10");
    SC_FLOAT_ROW(double, "%lf", "  1e400");     /* overflow -> +/-infinity. */
    SC_FLOAT_ROW(double, "%lf", "1e-400");      /* underflow -> 0. */
    SC_FLOAT_ROW(double, "%lf", "inf");
    SC_FLOAT_ROW(double, "%lf", "-infinity");
    SC_FLOAT_ROW(double, "%lf", "infX");        /* partial "inf" match: still accepted as infinity. */
    SC_FLOAT_ROW(double, "%lf", "nan");
    SC_FLOAT_ROW(double, "%lf", "nan(123)");
    SC_FLOAT_ROW(double, "%lf", "nan(");        /* malformed nan: one-way-latch failure. */
    SC_FLOAT_ROW(double, "%lf", "0x1.8p3");     /* hex float. */
    SC_FLOAT_ROW(double, "%lf", "0x");          /* "0x" with nothing after: subject "0". */
    SC_FLOAT_ROW(double, "%lf", "abc");
    SC_FLOAT_ROW(double, "%lf", "1e+");         /* malformed exponent with a sign present. */
    SC_FLOAT_ROW(double, "%lf", "1e");          /* malformed exponent, no sign. */
    SC_FLOAT_ROW(float, "%f", "3.14159");
    SC_FLOAT_ROW(float, "%f", "1e40");          /* overflows a float but not a double. */
}

/* ---- %c, %s, %[ ------------------------------------------------------------
 *
 * Every buffer is sentinel-filled with 'X' (and, for a wide buffer, 0x5858)
 * before the call, and hashed WHOLE (`fnv`) so trailing/untouched bytes show
 * a diff. `secure` appends the buffer-size argument the `_s` option reads.
 */
#define SC_BUF 16
static char g_sc_d1[SC_BUF], g_sc_d2[SC_BUF];
static wchar_t g_sc_wd1[SC_BUF], g_sc_wd2[SC_BUF];

static void sc_fill(void)
{
    memset(g_sc_d1, 'X', SC_BUF);
    memset(g_sc_d2, 'X', SC_BUF);
    for (int i = 0; i < SC_BUF; i++) { g_sc_wd1[i] = 0x5858; g_sc_wd2[i] = 0x5858; }
}

#define SC_STR_ROW(fmt, input, secure) do {                                                  \
        sc_fill();                                                                           \
        SC_RESET();                                                                          \
        int r1, r2;                                                                          \
        if (secure) {                                                                        \
            r1 = sfd___stdio_common_vsscanf(1, input, (size_t)-1, fmt, g_sc_d1, (unsigned)SC_BUF); \
            r2 = sfr___stdio_common_vsscanf(1, input, (size_t)-1, fmt, g_sc_d2, (unsigned)SC_BUF); \
        } else {                                                                              \
            r1 = sfd___stdio_common_vsscanf(0, input, (size_t)-1, fmt, g_sc_d1);              \
            r2 = sfr___stdio_common_vsscanf(0, input, (size_t)-1, fmt, g_sc_d2);               \
        }                                                                                     \
        SC_GRADE_SCALARS("scanf.str." fmt, input, r1, r2);                                    \
        grade2("scanf.str.buf:" fmt, input, fnv(g_sc_d1, SC_BUF), fnv(g_sc_d2, SC_BUF));      \
    } while (0)

/* The `_s` short-buffer case: a buffer too small for the match must ENOMEM and
 * leave the buffer reset, on both sides identically. */
#define SC_STR_SHORT_ROW(fmt, input, cap) do {                                               \
        sc_fill();                                                                           \
        SC_RESET();                                                                          \
        int r1 = sfd___stdio_common_vsscanf(1, input, (size_t)-1, fmt, g_sc_d1, (unsigned)(cap)); \
        int r2 = sfr___stdio_common_vsscanf(1, input, (size_t)-1, fmt, g_sc_d2, (unsigned)(cap)); \
        SC_GRADE_SCALARS("scanf.str.short." fmt, input, r1, r2);                             \
        grade2("scanf.str.short.buf:" fmt, input, fnv(g_sc_d1, SC_BUF), fnv(g_sc_d2, SC_BUF)); \
    } while (0)

static void run_sc_string(void)
{
    install_counting_iph();
    SC_STR_ROW("%s", "hello world", 0);
    SC_STR_ROW("%s", "  leading-space", 0);
    SC_STR_ROW("%5s", "abcdefgh", 0);
    SC_STR_ROW("%c", "Q", 0);
    SC_STR_ROW("%3c", "abcdef", 0);
    SC_STR_ROW("%[abc]", "aabbccx", 0);
    SC_STR_ROW("%[^xyz]", "hello, xyz", 0);
    SC_STR_ROW("%[a-z]", "abcXYZ", 0);
    SC_STR_ROW("%[a-zA-Z]", "abcXYZ123", 0);
    SC_STR_ROW("%s", "", 0);              /* EOF before any character: EOF return. */
    SC_STR_ROW("%s", "hello", 1);         /* secure, plenty of room. */
    SC_STR_SHORT_ROW("%s", "toolongforthebuffer", 4);
    SC_STR_SHORT_ROW("%[a-z]", "abcdefghij", 3);
}

/* Wide-buffer / cross-width cases: `%S`/`%C` and the legacy-wide-specifier
 * option, over a NARROW stream -- and a WIDE stream reading a narrow `%hs`. */
#define SC_WCHAR_ROW(fmt, ch, input, opts) do {                                              \
        sc_fill();                                                                           \
        SC_RESET();                                                                          \
        int r1 = sfd___stdio_common_vsscanf(opts, input, (size_t)-1, fmt, g_sc_wd1);         \
        int r2 = sfr___stdio_common_vsscanf(opts, input, (size_t)-1, fmt, g_sc_wd2);         \
        SC_GRADE_SCALARS("scanf.wstr." fmt, ch, r1, r2);                                     \
        grade2("scanf.wstr.buf:" fmt, ch, fnv(g_sc_wd1, sizeof g_sc_wd1), fnv(g_sc_wd2, sizeof g_sc_wd2)); \
    } while (0)

static void run_sc_cross_width(void)
{
    install_counting_iph();
    SC_WCHAR_ROW("%S", "narrow", "hi!", 0ULL);            /* narrow stream, wide destination. */
    SC_WCHAR_ROW("%ls", "narrow-ls", "wide-req", 0ULL);
    SC_WCHAR_ROW("%ws", "legacy", "abc", 2ULL);           /* `_CRT_INTERNAL_SCANF_LEGACY_WIDE_SPECIFIERS`. */
}

/* ---- %n and assignment suppression ----------------------------------------- */
static void run_sc_n_and_suppress(void)
{
    int d1n = -999, d2n = -999;
    int d1v = -999, d2v = -999;
    install_counting_iph();
    SC_RESET();
    int r1 = sfd___stdio_common_vsscanf(0, "  123abc", (size_t)-1, "%*d%n", &d1n);
    int r2 = sfr___stdio_common_vsscanf(0, "  123abc", (size_t)-1, "%*d%n", &d2n);
    SC_GRADE_SCALARS("scanf.n.suppress", "case1", r1, r2);
    grade2("scanf.n.suppress.val", "case1", d1n, d2n);

    SC_RESET();
    r1 = sfd___stdio_common_vsscanf(0, "77", (size_t)-1, "%d%n", &d1v, &d1n);
    r2 = sfr___stdio_common_vsscanf(0, "77", (size_t)-1, "%d%n", &d2v, &d2n);
    SC_GRADE_SCALARS("scanf.n.value", "case2", r1, r2);
    grade2("scanf.n.value.val", "case2", d1v, d2v);
    grade2("scanf.n.value.count", "case2", d1n, d2n);
}

/* ---- wide source: vswscanf -------------------------------------------------- */
static void run_sc_wide_source(void)
{
    int d1 = 0x5A, d2 = 0x5A;
    install_counting_iph();
    SC_RESET();
    int r1 = sfd___stdio_common_vswscanf(0, L"  -42", (size_t)-1, L"%d", &d1);
    int r2 = sfr___stdio_common_vswscanf(0, L"  -42", (size_t)-1, L"%d", &d2);
    SC_GRADE_SCALARS("scanf.wide.int", "vswscanf", r1, r2);
    grade2("scanf.wide.int.val", "vswscanf", d1, d2);

    wchar_t wb1[SC_BUF], wb2[SC_BUF];
    for (int i = 0; i < SC_BUF; i++) { wb1[i] = 0x5858; wb2[i] = 0x5858; }
    SC_RESET();
    r1 = sfd___stdio_common_vswscanf(0, L"hello", (size_t)-1, L"%s", wb1);
    r2 = sfr___stdio_common_vswscanf(0, L"hello", (size_t)-1, L"%s", wb2);
    SC_GRADE_SCALARS("scanf.wide.str", "vswscanf", r1, r2);
    grade2("scanf.wide.str.buf", "vswscanf", fnv(wb1, sizeof wb1), fnv(wb2, sizeof wb2));
}

/* ---- %d/%s over a real FILE*, per side's OWN temp file --------------------- *
 *
 * Each side gets its own temp file, written with its OWN CRT (`ud_fopen`-style
 * via the public, unrenamed `fopen`/`fprintf` for the DLL side is not
 * available -- both sides share the process's actual `fopen`, so this simply
 * writes the fixture text with plain narrow I/O via the OS, then opens it with
 * EACH side's own `fopen`/`ref_fopen`, matching the "each side its own
 * descriptors" rule). Deleted at the end regardless of outcome.
 */
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

static void sc_write_fixture(const char *path, const char *text)
{
    HANDLE h = CreateFileA(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) return;
    DWORD written;
    WriteFile(h, text, (DWORD)strlen(text), &written, NULL);
    CloseHandle(h);
}

static void run_sc_file(void)
{
    char dir[MAX_PATH];
    if (GetTempPathA(sizeof dir, dir) == 0) return;
    char path1[MAX_PATH], path2[MAX_PATH];
    strcpy(path1, dir); strcat(path1, "ucrtdiff_scanf_dll.txt");
    strcpy(path2, dir); strcat(path2, "ucrtdiff_scanf_ref.txt");
    sc_write_fixture(path1, "42 hello 3.5\n");
    sc_write_fixture(path2, "42 hello 3.5\n");

    FILE *f1 = fopen(path1, "r");
    FILE *f2 = ref_fopen(path2, "r");
    if (f1 && f2) {
        int i1 = -1, i2 = -1;
        char s1[SC_BUF], s2[SC_BUF];
        memset(s1, 'X', sizeof s1); memset(s2, 'X', sizeof s2);
        double f1v = -1.0, f2v = -1.0;
        install_counting_iph();
        SC_RESET();
        int r1 = sfd___stdio_common_vfscanf(0, f1, "%d %5s %lf", &i1, s1, &f1v);
        int r2 = sfr___stdio_common_vfscanf(0, f2, "%d %5s %lf", &i2, s2, &f2v);
        SC_GRADE_SCALARS("scanf.file", "vfscanf", r1, r2);
        grade2("scanf.file.int", "vfscanf", i1, i2);
        grade2("scanf.file.str", "vfscanf", fnv(s1, sizeof s1), fnv(s2, sizeof s2));
        grade2("scanf.file.float", "vfscanf", SC_FP_BITS(f1v), SC_FP_BITS(f2v));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(path1);
    DeleteFileA(path2);
}

#endif /* UCRTDIFF_REF_HAS___stdio_common_vsscanf */

/*
 * Registry entries for `run_legacy()`'s table (paste alongside the other
 * `*_HAND` blocks; the export named in the guard is `__stdio_common_vsscanf`,
 * so that is what the registry conditional keys on, matching the FP_HAND /
 * ST_HAND style):
 *
 *     #if UCRTDIFF_REF_HAS___stdio_common_vsscanf
 *     #define SC_HAND(name) { "legacy:" name, run_sc_##name },
 *     #else
 *     #define SC_HAND(name) { "legacy:" name, 0 },
 *     #endif
 *     SC_HAND(int) SC_HAND(float) SC_HAND(string) SC_HAND(cross_width)
 *     SC_HAND(n_and_suppress) SC_HAND(wide_source) SC_HAND(file)
 */

#if UCRTDIFF_REF_HAS___stdio_common_vsscanf
#define STDIO_INPUT_HAND_ENTRIES \
    { "scanf:int", run_sc_int }, { "scanf:float", run_sc_float }, \
    { "scanf:string", run_sc_string }, { "scanf:cross_width", run_sc_cross_width }, \
    { "scanf:n_and_suppress", run_sc_n_and_suppress }, { "scanf:wide_source", run_sc_wide_source }, \
    { "scanf:file", run_sc_file },
#else
#define STDIO_INPUT_HAND_ENTRIES
#endif


/* ==== staging/stdio_bufio_more.c, inlined ==== */
/*
 * staging/stdio_bufio_more.c -- follow-up runners for the `bufio` piece of
 * the stdio batch. pecheck found nine bufio exports with a `ud_<name>`
 * import pointer declared in staging/stdio_bufio.c but never actually
 * CALLED on the DLL side, so they graded nothing: `_fgetchar`, `_fgetwchar`,
 * `_fputchar`, `getchar`, `getwchar`, `putc`, `putchar`, `putwc`,
 * `_getc_nolock`. This file is a SEPARATE #include (stdio_bufio.c is already
 * inlined into ucrtdiff.c and must not be touched again), so every ref_/ud_
 * prototype it needs is re-declared locally inside each guard, even though
 * stdio_bufio.c also declares some of the same names.
 *
 * Guards -- one per export's owning C file, reusing the guard names already
 * live in stdio_bufio.c/ucrtdiff.c's `generated-refs.h`:
 *   UCRTDIFF_REF_HAS_fgetc   -> _getc_nolock, _fgetchar, getchar   (fgetc.cpp)
 *   UCRTDIFF_REF_HAS_fgetwc  -> _fgetwchar, getwchar               (fgetwc.cpp)
 *   UCRTDIFF_REF_HAS_fputc   -> putc, _fputchar, putchar           (fputc.cpp)
 *   UCRTDIFF_REF_HAS_fputwc  -> putwc                              (fputwc.cpp)
 *
 * `_getc_nolock` and `putc` take an explicit `FILE*` and so need no console:
 * they are graded against a plain two-sided temp file, same shape as the
 * fgetc/fputc blocks in stdio_bufio.c. `_fgetchar`/`getchar`/`_fputchar`/
 * `putchar`/`getwchar` take NONE, reading/writing `stdin`/`stdout` directly,
 * so this harness's lack of a console is worked around by redirecting each
 * side's OWN fd 0/1 to its OWN temp file: `CreateFileA` gets a real Win32
 * HANDLE (a Win32 API, shared meaning on both sides), `_open_osfhandle`/
 * `ref__open_osfhandle` wraps it into a fd IN THAT SIDE's OWN lowio table
 * (the DLL and the statically-linked reference archive have their own,
 * independent fd tables even though both start out backed by the same
 * inherited OS handles), and `_dup`/`_dup2` (DLL) or `ref__dup`/`ref__dup2`
 * (reference) install/restore fd 0 or 1 in that table. `putwc` is graded
 * against a plain temp file too, not literally `stdout`: `putwc(c, stream)`
 * takes an explicit stream, so redirecting a console buys no extra coverage
 * over what `_fputchar`/`putchar` already exercise for the SAME underlying
 * `_flsbuf`, and it would need a `ref_stdout` accessor that does not exist
 * (there is no `ref___acrt_iob_func`/`ref_stdout` symbol in the reference
 * archive) -- reported here rather than invented.
 *
 * Every temp file and every saved fd is restored/closed/deleted before each
 * runner returns, on both sides, even on an early failure path.
 */

/* ==== fgetc.cpp: _getc_nolock, _fgetchar, getchar ==== */
#if UCRTDIFF_REF_HAS_fgetc

int __cdecl ref__getc_nolock(FILE *);
int __cdecl ref__fgetchar(void);
int __cdecl ref_getchar(void);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
int __cdecl ref__fileno(FILE *);

extern int(__cdecl *const ud__getc_nolock)(FILE *) __asm__(UD_IMP(_getc_nolock));
extern int(__cdecl *const ud__fgetchar)(void) __asm__(UD_IMP(_fgetchar));
extern int(__cdecl *const ud_getchar)(void) __asm__(UD_IMP(getchar));

static void run_bufio_more_getc_nolock(void)
{
    const char *leaf = "more_getc_nolock.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (f1) { fwrite("Q", 1, 1, f1); fclose(f1); }
    if (f2) { ref_fwrite("Q", 1, 1, f2); ref_fclose(f2); }

    f1 = fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    grade2("_getc_nolock.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        int c1 = ud__getc_nolock(f1);
        int c2 = ref__getc_nolock(f2);
        grade2("_getc_nolock.char", leaf, c1, c2);
        grade2("_getc_nolock.eof", leaf, ud__getc_nolock(f1), ref__getc_nolock(f2));
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

/* Redirects fd 0 to `path` for the DLL side; returns the saved fd (>= 0) to
 * pass to `restore_stdin_dll`, or -1 if redirection could not be set up (in
 * which case the caller must skip the case rather than run it unredirected). */
static int redirect_stdin_dll(const char *path)
{
    return freopen(path, "rb", stdin) ? 0 : -1;
}
static void restore_stdin_dll(int saved)
{
    if (saved >= 0) freopen("NUL", "rb", stdin);
}
static int redirect_stdin_ref(const char *path)
{
    return ref_freopen(path, "rb", ref_std_stream(0)) ? 0 : -1;
}
static void restore_stdin_ref(int saved)
{
    if (saved >= 0) ref_freopen("NUL", "rb", ref_std_stream(0));
}

/* `_fgetchar`/`getchar` read one byte from `stdin`. Each side's own fd 0 is
 * pointed at its own one-byte file, the call is made, and both fds are
 * restored -- regardless of whether either redirect actually succeeded, so a
 * side that cannot redirect is simply SKIPPED (and said so), never run
 * against the real console (there is none here). */
static void run_bufio_more_stdin_narrow(void)
{
    const char *leaf1 = "more_stdin_dll.bin";
    const char *leaf2 = "more_stdin_ref.bin";
    HANDLE w1 = CreateFileA(sd_path(leaf1, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w1 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w1, "A", 1, &wn, 0); CloseHandle(w1); }
    HANDLE w2 = CreateFileA(sd_path(leaf2, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w2 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w2, "B", 1, &wn, 0); CloseHandle(w2); }

    int saved1 = redirect_stdin_dll(sd_path(leaf1, 0));
    int saved2 = redirect_stdin_ref(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP _fgetchar/getchar: stdin redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
    } else {
        install_counting_iph();
        int c1 = ud__fgetchar();
        int c2 = ref__fgetchar();
        grade2("_fgetchar.char", "A/B", c1 == 'A', c2 == 'B'); /* each side reads its OWN byte */
    }
    restore_stdin_dll(saved1);
    restore_stdin_ref(saved2);
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));

    /* `getchar` == `_fgetchar()`: a second byte, through the public name. */
    w1 = CreateFileA(sd_path(leaf1, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w1 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w1, "C", 1, &wn, 0); CloseHandle(w1); }
    w2 = CreateFileA(sd_path(leaf2, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w2 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w2, "D", 1, &wn, 0); CloseHandle(w2); }
    saved1 = redirect_stdin_dll(sd_path(leaf1, 0));
    saved2 = redirect_stdin_ref(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP getchar: stdin redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
    } else {
        int c1 = ud_getchar();
        int c2 = ref_getchar();
        grade2("getchar.char", "C/D", c1 == 'C', c2 == 'D');
    }
    restore_stdin_dll(saved1);
    restore_stdin_ref(saved2);
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));
}

#define BUFIO_MORE_HAND_FGETC(name, fn) { name, run_##fn },
#else
#define BUFIO_MORE_HAND_FGETC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fgetc */

/* ==== fgetwc.cpp: _fgetwchar, getwchar ==== */
#if UCRTDIFF_REF_HAS_fgetwc

wint_t __cdecl ref__fgetwchar(void);
wint_t __cdecl ref_getwchar(void);
int __cdecl ref__fileno(FILE *);

extern wint_t(__cdecl *const ud__fgetwchar)(void) __asm__(UD_IMP(_fgetwchar));
extern wint_t(__cdecl *const ud_getwchar)(void) __asm__(UD_IMP(getwchar));

/* Same redirect helpers as the narrow block, duplicated (this file has no
 * shared header of its own, and stdio_bufio.c must not be edited to export
 * them): `_fgetwchar`'s binary-mode path reads two raw bytes per code unit
 * from `stdin`, same as `_filbuf`'s wide refill. */
static int redirect_stdin_dll_w(const char *path)
{
    return freopen(path, "rb", stdin) ? 0 : -1;
}
static void restore_stdin_dll_w(int saved)
{
    if (saved >= 0) freopen("NUL", "rb", stdin);
}
static int redirect_stdin_ref_w(const char *path)
{
    return ref_freopen(path, "rb", ref_std_stream(0)) ? 0 : -1;
}
static void restore_stdin_ref_w(int saved)
{
    if (saved >= 0) ref_freopen("NUL", "rb", ref_std_stream(0));
}

static void run_bufio_more_stdin_wide(void)
{
    const char *leaf1 = "more_wstdin_dll.bin";
    const char *leaf2 = "more_wstdin_ref.bin";
    static const wchar_t ca = L'é', cb = L'ê', cc = L'ë', cd = L'ì';

    HANDLE w1 = CreateFileA(sd_path(leaf1, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w1 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w1, &ca, sizeof ca, &wn, 0); CloseHandle(w1); }
    HANDLE w2 = CreateFileA(sd_path(leaf2, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w2 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w2, &cb, sizeof cb, &wn, 0); CloseHandle(w2); }

    int saved1 = redirect_stdin_dll_w(sd_path(leaf1, 0));
    int saved2 = redirect_stdin_ref_w(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP _fgetwchar/getwchar: stdin redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
    } else {
        install_counting_iph();
        wint_t r1 = ud__fgetwchar();
        wint_t r2 = ref__fgetwchar();
        grade2("_fgetwchar.char", "e/ea", r1 == ca, r2 == cb);
    }
    restore_stdin_dll_w(saved1);
    restore_stdin_ref_w(saved2);
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));

    w1 = CreateFileA(sd_path(leaf1, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w1 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w1, &cc, sizeof cc, &wn, 0); CloseHandle(w1); }
    w2 = CreateFileA(sd_path(leaf2, 0), GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (w2 != INVALID_HANDLE_VALUE) { DWORD wn; WriteFile(w2, &cd, sizeof cd, &wn, 0); CloseHandle(w2); }
    saved1 = redirect_stdin_dll_w(sd_path(leaf1, 0));
    saved2 = redirect_stdin_ref_w(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP getwchar: stdin redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
    } else {
        wint_t r1 = ud_getwchar();
        wint_t r2 = ref_getwchar();
        grade2("getwchar.char", "eb/ec", r1 == cc, r2 == cd);
    }
    restore_stdin_dll_w(saved1);
    restore_stdin_ref_w(saved2);
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));
}

#define BUFIO_MORE_HAND_FGETWC(name, fn) { name, run_##fn },
#else
#define BUFIO_MORE_HAND_FGETWC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fgetwc */

/* ==== fputc.cpp: putc, _fputchar, putchar ==== */
#if UCRTDIFF_REF_HAS_fputc

int __cdecl ref_putc(int, FILE *);
int __cdecl ref__fputchar(int);
int __cdecl ref_putchar(int);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref_fflush(FILE *);
int __cdecl ref__fileno(FILE *);

extern int(__cdecl *const ud_putc)(int, FILE *) __asm__(UD_IMP(putc));
extern int(__cdecl *const ud__fputchar)(int) __asm__(UD_IMP(_fputchar));
extern int(__cdecl *const ud_putchar)(int) __asm__(UD_IMP(putchar));

static void run_bufio_more_putc(void)
{
    const char *leaf = "more_putc.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("putc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        int c1 = ud_putc('R', f1);
        int c2 = ref_putc('R', f2);
        grade2("putc.char", leaf, c1, c2);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

static int redirect_stdout_dll(const char *path)
{
    return freopen(path, "wb", stdout) ? 0 : -1;
}
static void restore_stdout_dll(int saved)
{
    if (saved < 0) return;
    fflush(stdout);
    freopen("NUL", "wb", stdout);
}
static int redirect_stdout_ref(const char *path)
{
    return ref_freopen(path, "wb", ref_std_stream(1)) ? 0 : -1;
}
static void restore_stdout_ref(int saved)
{
    if (saved < 0) return;
    ref_fflush(ref_std_stream(1));
    ref_freopen("NUL", "wb", ref_std_stream(1));
}

static void run_bufio_more_stdout_narrow(void)
{
    const char *leaf1 = "more_stdout_dll.bin";
    const char *leaf2 = "more_stdout_ref.bin";

    int saved1 = redirect_stdout_dll(sd_path(leaf1, 0));
    int saved2 = redirect_stdout_ref(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP _fputchar/putchar: stdout redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
        restore_stdout_dll(saved1);
        restore_stdout_ref(saved2);
        return;
    }
    install_counting_iph();
    int c1 = ud__fputchar('X');
    int c2 = ref__fputchar('X');
    grade2("_fputchar.ret", "X", c1, c2);
    restore_stdout_dll(saved1);
    restore_stdout_ref(saved2);

    HANDLE h1 = CreateFileA(sd_path(leaf1, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    HANDLE h2 = CreateFileA(sd_path(leaf2, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    unsigned char b1[4] = { 0 }, b2[4] = { 0 };
    DWORD n1 = 0, n2 = 0;
    if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
    if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
    grade2("_fputchar.file", "X", n1 == 1 && b1[0] == 'X', n2 == 1 && b2[0] == 'X');
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));

    /* `putchar` == `_fputchar()`: through the public name. */
    saved1 = redirect_stdout_dll(sd_path(leaf1, 0));
    saved2 = redirect_stdout_ref(sd_path(leaf2, 0));
    if (saved1 < 0 || saved2 < 0) {
        DbgPrint("[UCRTDIFF]   SKIP putchar: stdout redirect unavailable (dll=%d ref=%d)\n", saved1 >= 0, saved2 >= 0);
        restore_stdout_dll(saved1);
        restore_stdout_ref(saved2);
        return;
    }
    c1 = ud_putchar('Y');
    c2 = ref_putchar('Y');
    grade2("putchar.ret", "Y", c1, c2);
    restore_stdout_dll(saved1);
    restore_stdout_ref(saved2);

    h1 = CreateFileA(sd_path(leaf1, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    h2 = CreateFileA(sd_path(leaf2, 0), GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);
    n1 = n2 = 0;
    memset(b1, 0, sizeof b1);
    memset(b2, 0, sizeof b2);
    if (h1 != INVALID_HANDLE_VALUE) { ReadFile(h1, b1, sizeof b1, &n1, 0); CloseHandle(h1); }
    if (h2 != INVALID_HANDLE_VALUE) { ReadFile(h2, b2, sizeof b2, &n2, 0); CloseHandle(h2); }
    grade2("putchar.file", "Y", n1 == 1 && b1[0] == 'Y', n2 == 1 && b2[0] == 'Y');
    DeleteFileA(sd_path(leaf1, 0));
    DeleteFileA(sd_path(leaf2, 0));
}

#define BUFIO_MORE_HAND_FPUTC(name, fn) { name, run_##fn },
#else
#define BUFIO_MORE_HAND_FPUTC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputc */

/* ==== fputwc.cpp: putwc ==== */
#if UCRTDIFF_REF_HAS_fputwc

wint_t __cdecl ref_putwc(wchar_t, FILE *);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);

extern wint_t(__cdecl *const ud_putwc)(wchar_t, FILE *) __asm__(UD_IMP(putwc));

/* Graded against a plain two-sided temp file, not literally `stdout` -- see
 * the file-header comment for why (no `ref_stdout` accessor exists, and
 * `putwc(c, stream)` takes an explicit stream, so a console buys nothing
 * `_fputchar`/`putchar` above don't already cover for the same `_flsbuf`). */
static void run_bufio_more_putwc(void)
{
    const char *leaf = "more_putwc.bin";
    FILE *f1 = fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    grade2("putwc.open", leaf, f1 != 0, f2 != 0);
    if (f1 && f2) {
        install_counting_iph();
        wint_t c1 = ud_putwc(L'é', f1);
        wint_t c2 = ref_putwc(L'é', f2);
        grade2("putwc.char", leaf, (long long)c1, (long long)c2);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#define BUFIO_MORE_HAND_FPUTWC(name, fn) { name, run_##fn },
#else
#define BUFIO_MORE_HAND_FPUTWC(name, fn) { name, 0 },
#endif /* UCRTDIFF_REF_HAS_fputwc */

/* ==========================================================================
 * Registry -- paste every BUFIO_MORE_HAND_* entry into run_legacy()'s
 * `legacy[]` table, the same way BUFIO_HAND_* already is.
 * ========================================================================== */
#define BUFIO_MORE_HAND_ENTRIES                                               \
    BUFIO_MORE_HAND_FGETC("_getc_nolock", bufio_more_getc_nolock)             \
    BUFIO_MORE_HAND_FGETC("_fgetchar+getchar", bufio_more_stdin_narrow)       \
    BUFIO_MORE_HAND_FGETWC("_fgetwchar+getwchar", bufio_more_stdin_wide)      \
    BUFIO_MORE_HAND_FPUTC("putc", bufio_more_putc)                            \
    BUFIO_MORE_HAND_FPUTC("_fputchar+putchar", bufio_more_stdout_narrow)      \
    BUFIO_MORE_HAND_FPUTWC("putwc", bufio_more_putwc)


/* ==== staging/stdio_files_more.c, inlined ==== */
/*
 * staging/stdio_files_more.c — follow-up guest grader for the `files` piece
 * of the stdio batch, closing the pecheck gap on the first pass
 * (staging/stdio_files.c, already inlined into ucrtdiff.c): a set of
 * exports that were DECLARED (`ref_X`/`ud_X`) there but never actually
 * CALLED on the DLL side, so they would have landed ungraded --
 * `_fclose_nolock _fflush_nolock _flushall _fread_nolock _fread_nolock_s
 * _fseek_nolock _fseeki64 _fseeki64_nolock _fsopen _ftell_nolock _ftelli64
 * _ftelli64_nolock _fwrite_nolock _wfdopen _wfopen _wfopen_s _wfreopen
 * _wfreopen_s _wfsopen` -- plus `_doserrno`, graded here via `fopen` of a
 * missing path (the one place in this piece's own files where the C sets
 * it: the failing `_sopen_s`/`CreateFile` underneath `common_openfile` maps
 * `GetLastError()` through `__acrt_errno_map_os_error`, which sets both
 * `errno` and `_doserrno`).
 *
 * Independent of staging/stdio_files.c on purpose: uses the harness's own
 * unconditional `sd_path`/`sd_dir` (not that file's `sf_path`, which lives
 * inside its own OR-of-all-its-guards block and is therefore not something
 * this file should depend on), and redeclares every `ref_`/`ud_` prototype
 * it touches even where stdio_files.c already declared the same one --
 * this file must stand on its own regardless of which of its sibling's
 * guards happen to be on.
 *
 * Each runner is guarded by the `UCRTDIFF_REF_HAS_<export>` of the ONE C
 * file it is grading (reusing the same guard names stdio_files.c's own
 * blocks use), except the 64-bit-position row, which genuinely exercises
 * BOTH fseek.cpp and ftell.cpp and is guarded by the AND of both.
 *
 * `_fclose_nolock`/`_fflush_nolock`/`_fread_nolock`/`_fread_nolock_s`/
 * `_fwrite_nolock`/`_fseek_nolock`/`_fseeki64_nolock`/`_ftell_nolock`/
 * `_ftelli64_nolock` are called DIRECTLY, with no explicit `_lock_file`
 * bracketing: this harness is single-threaded, so the lock is not
 * observable, and the "locked" and "nolock" forms of a given operation are
 * graded against the SAME sequences their locked twins already used in
 * stdio_files.c (per the integration note: the lock itself is not the
 * contract under test here, the buffered I/O logic beneath it is).
 *
 * NOTHING here touches process-wide state permanently: every temp file
 * created (narrow or wide) is deleted, on both sides, before its runner
 * returns.
 */

/* A local read-back helper, independent of stdio_files.c's `sf_read_back` --
 * plain Win32 ReadFile, not either CRT. */
static long sfm_read_back(const char *path, unsigned char *buf, unsigned long cap)
{
    HANDLE h = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
        return -1;
    DWORD n = 0;
    BOOL ok = ReadFile(h, buf, cap, &n, NULL);
    CloseHandle(h);
    return ok ? (long)n : -1;
}

/* Builds `<sd_dir-and-side-prefix><wleaf>` in `out` -- the wide-path
 * equivalent of `sd_path`, since `sd_path` itself only hands back a narrow
 * `char*`. `sd_path("", side)` gives just the directory + per-side prefix
 * with no leaf, which is ASCII by construction (a Win32 temp path), so the
 * `MultiByteToWideChar` conversion of THAT part is safe; `wleaf` (which may
 * contain a non-ASCII character) is appended in wide form directly. */
static void sfm_wide_path(wchar_t *out, int outcap, const wchar_t *wleaf, int side)
{
    wchar_t wprefix[MAX_PATH + 48];
    MultiByteToWideChar(CP_ACP, 0, sd_path("", side), -1, wprefix, MAX_PATH + 48);
    lstrcpyW(out, wprefix);
    lstrcatW(out, wleaf);
    (void)outcap;
}

/* ==== fopen.cpp: _fsopen, _wfopen, _wfopen_s, _wfsopen, and _doserrno ===== */

#if UCRTDIFF_REF_HAS_fopen

FILE *__cdecl ref__fsopen(const char *, const char *, int);
FILE *__cdecl ref__wfopen(const wchar_t *, const wchar_t *);
errno_t __cdecl ref__wfopen_s(FILE **, const wchar_t *, const wchar_t *);
FILE *__cdecl ref__wfsopen(const wchar_t *, const wchar_t *, int);
FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
int *__cdecl ref__errno(void);
unsigned long *__cdecl ref___doserrno(void);

extern FILE *(__cdecl *const ud__fsopen)(const char *, const char *, int) __asm__(UD_IMP(_fsopen));
extern FILE *(__cdecl *const ud__wfopen)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen));
extern errno_t(__cdecl *const ud__wfopen_s)(FILE **, const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen_s));
extern FILE *(__cdecl *const ud__wfsopen)(const wchar_t *, const wchar_t *, int) __asm__(UD_IMP(_wfsopen));
extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));

#define SFM_SH_DENYNO 0x40 /* _SH_DENYNO (share.h) */

static void run_sfm_fsopen(void)
{
    install_counting_iph();
    const char *leaf = "sfm_fsopen.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    *_errno() = 0; *ref__errno() = 0;
    FILE *f1 = ud__fsopen(sd_path(leaf, 0), "w", SFM_SH_DENYNO);
    FILE *f2 = ref__fsopen(sd_path(leaf, 1), "w", SFM_SH_DENYNO);
    grade2("_fsopen.ret", "-", f1 != 0, f2 != 0);
    grade2("_fsopen.errno", "-", *_errno(), *ref__errno());
    if (f1) { ud_fwrite("hi", 1, 2, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite("hi", 1, 2, f2); ref_fclose(f2); }

    unsigned char b1[4], b2[4];
    long n1 = sfm_read_back(sd_path(leaf, 0), b1, sizeof b1);
    long n2 = sfm_read_back(sd_path(leaf, 1), b2, sizeof b2);
    grade2("_fsopen.bytes", "-", n1, n2);

    /* NULL path: EINVAL, handler invoked. */
    *_errno() = 0; *ref__errno() = 0; g_iph_dll = g_iph_ref = 0;
    f1 = ud__fsopen(NULL, "w", SFM_SH_DENYNO);
    f2 = ref__fsopen(NULL, "w", SFM_SH_DENYNO);
    grade2("_fsopen.nullpath.ret", "-", f1 != 0, f2 != 0);
    grade2("_fsopen.nullpath.errno", "-", *_errno(), *ref__errno());
    grade2("_fsopen.nullpath.iph", "-", g_iph_dll, g_iph_ref);

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

static void run_sfm_wfopen(void)
{
    install_counting_iph();

    /* ASCII wide path. */
    wchar_t w1[MAX_PATH + 64], w2[MAX_PATH + 64];
    sfm_wide_path(w1, MAX_PATH + 64, L"sfm_wfopen_ascii.tmp", 0);
    sfm_wide_path(w2, MAX_PATH + 64, L"sfm_wfopen_ascii.tmp", 1);
    DeleteFileW(w1);
    DeleteFileW(w2);

    *_errno() = 0; *ref__errno() = 0;
    FILE *f1 = ud__wfopen(w1, L"w");
    FILE *f2 = ref__wfopen(w2, L"w");
    grade2("_wfopen.ascii.ret", "-", f1 != 0, f2 != 0);
    grade2("_wfopen.ascii.errno", "-", *_errno(), *ref__errno());
    if (f1) { ud_fwrite("wide", 1, 4, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite("wide", 1, 4, f2); ref_fclose(f2); }

    unsigned char b1[8], b2[8];
    {
        char narrow1[MAX_PATH + 64], narrow2[MAX_PATH + 64];
        WideCharToMultiByte(CP_ACP, 0, w1, -1, narrow1, sizeof narrow1, NULL, NULL);
        WideCharToMultiByte(CP_ACP, 0, w2, -1, narrow2, sizeof narrow2, NULL, NULL);
        long n1 = sfm_read_back(narrow1, b1, sizeof b1);
        long n2 = sfm_read_back(narrow2, b2, sizeof b2);
        grade2("_wfopen.ascii.bytes", "-", n1, n2);
    }
    DeleteFileW(w1);
    DeleteFileW(w2);

    /* Non-ASCII wide path (an e-acute), exercising the wide code path with
     * a character that has no narrow-mode equivalent in the mode string
     * itself but DOES appear in the file NAME -- `_wfopen` must pass the
     * wide name straight to `_wsopen_s`, never round-tripping it through a
     * narrow codepage. */
    sfm_wide_path(w1, MAX_PATH + 64, L"sfm_wfopen_é.tmp", 0);
    sfm_wide_path(w2, MAX_PATH + 64, L"sfm_wfopen_é.tmp", 1);
    DeleteFileW(w1);
    DeleteFileW(w2);

    *_errno() = 0; *ref__errno() = 0;
    f1 = ud__wfopen(w1, L"w");
    f2 = ref__wfopen(w2, L"w");
    grade2("_wfopen.nonascii.ret", "-", f1 != 0, f2 != 0);
    grade2("_wfopen.nonascii.errno", "-", *_errno(), *ref__errno());
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    /* Confirm the file really was created under its non-ASCII name (not
     * silently mangled/truncated) via a wide FindFirstFile-free existence
     * check: GetFileAttributesW. */
    DWORD a1 = GetFileAttributesW(w1);
    DWORD a2 = GetFileAttributesW(w2);
    grade2("_wfopen.nonascii.exists", "-", a1 != INVALID_FILE_ATTRIBUTES, a2 != INVALID_FILE_ATTRIBUTES);
    DeleteFileW(w1);
    DeleteFileW(w2);

    /* _wfopen_s. */
    sfm_wide_path(w1, MAX_PATH + 64, L"sfm_wfopen_s.tmp", 0);
    sfm_wide_path(w2, MAX_PATH + 64, L"sfm_wfopen_s.tmp", 1);
    DeleteFileW(w1);
    DeleteFileW(w2);
    FILE *r1 = (FILE *)0x1, *r2 = (FILE *)0x1;
    errno_t e1 = ud__wfopen_s(&r1, w1, L"w");
    errno_t e2 = ref__wfopen_s(&r2, w2, L"w");
    grade2("_wfopen_s.ret", "-", e1, e2);
    grade2("_wfopen_s.nonnull", "-", r1 != 0, r2 != 0);
    if (r1) ud_fclose(r1);
    if (r2) ref_fclose(r2);
    DeleteFileW(w1);
    DeleteFileW(w2);

    /* _wfsopen with an explicit share flag. */
    sfm_wide_path(w1, MAX_PATH + 64, L"sfm_wfsopen.tmp", 0);
    sfm_wide_path(w2, MAX_PATH + 64, L"sfm_wfsopen.tmp", 1);
    DeleteFileW(w1);
    DeleteFileW(w2);
    *_errno() = 0; *ref__errno() = 0;
    f1 = ud__wfsopen(w1, L"w", SFM_SH_DENYNO);
    f2 = ref__wfsopen(w2, L"w", SFM_SH_DENYNO);
    grade2("_wfsopen.ret", "-", f1 != 0, f2 != 0);
    grade2("_wfsopen.errno", "-", *_errno(), *ref__errno());
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileW(w1);
    DeleteFileW(w2);
}

static void run_sfm_doserrno(void)
{
    const char *leaf = "sfm_doserrno_missing.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    *_errno() = 0; *__doserrno() = 0;
    *ref__errno() = 0; *ref___doserrno() = 0;

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "r"); /* does not exist: must fail */
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "r");
    grade2("fopen.missing.ret", "-", f1 != 0, f2 != 0);
    grade2("fopen.missing.errno", "-", *_errno(), *ref__errno());
    grade2("fopen.missing.doserrno", "-", (long long)*__doserrno(), (long long)*ref___doserrno());
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
}

#endif /* UCRTDIFF_REF_HAS_fopen */

/* ==== freopen.cpp: _wfreopen, _wfreopen_s ================================= */

#if UCRTDIFF_REF_HAS_freopen

FILE *__cdecl ref__wfreopen(const wchar_t *, const wchar_t *, FILE *);
errno_t __cdecl ref__wfreopen_s(FILE **, const wchar_t *, const wchar_t *, FILE *);
FILE *__cdecl ref__wfopen(const wchar_t *, const wchar_t *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);

extern FILE *(__cdecl *const ud__wfreopen)(const wchar_t *, const wchar_t *, FILE *) __asm__(UD_IMP(_wfreopen));
extern errno_t(__cdecl *const ud__wfreopen_s)(FILE **, const wchar_t *, const wchar_t *, FILE *) __asm__(UD_IMP(_wfreopen_s));
extern FILE *(__cdecl *const ud__wfopen)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));

static void run_sfm_wfreopen(void)
{
    install_counting_iph();
    wchar_t wa1[MAX_PATH + 64], wa2[MAX_PATH + 64], wb1[MAX_PATH + 64], wb2[MAX_PATH + 64];
    sfm_wide_path(wa1, MAX_PATH + 64, L"sfm_wfreopen_a.tmp", 0);
    sfm_wide_path(wa2, MAX_PATH + 64, L"sfm_wfreopen_a.tmp", 1);
    sfm_wide_path(wb1, MAX_PATH + 64, L"sfm_wfreopen_b.tmp", 0);
    sfm_wide_path(wb2, MAX_PATH + 64, L"sfm_wfreopen_b.tmp", 1);
    DeleteFileW(wa1); DeleteFileW(wa2); DeleteFileW(wb1); DeleteFileW(wb2);

    FILE *f1 = ud__wfopen(wa1, L"w");
    FILE *f2 = ref__wfopen(wa2, L"w");

    FILE *g1 = ud__wfreopen(wb1, L"w", f1);
    FILE *g2 = ref__wfreopen(wb2, L"w", f2);
    grade2("_wfreopen.reuse", "-", g1 == f1, g2 == f2);
    if (g1) { ud_fwrite("q", 1, 1, g1); ud_fclose(g1); }
    if (g2) { ref_fwrite("q", 1, 1, g2); ref_fclose(g2); }

    /* _wfreopen_s. */
    FILE *r1 = (FILE *)0x1, *r2 = (FILE *)0x1;
    errno_t e1 = ud__wfreopen_s(&r1, wb1, L"r", NULL);
    errno_t e2 = ref__wfreopen_s(&r2, wb2, L"r", NULL);
    grade2("_wfreopen_s.nullstream.ret", "-", e1, e2); /* freopen_s validates stream.valid() -> EINVAL */

    DeleteFileW(wa1); DeleteFileW(wa2); DeleteFileW(wb1); DeleteFileW(wb2);
}

#endif /* UCRTDIFF_REF_HAS_freopen */

/* ==== fdopen.cpp: _wfdopen ================================================= */

#if UCRTDIFF_REF_HAS__fdopen

FILE *__cdecl ref__wfdopen(int, const wchar_t *);
FILE *__cdecl ref__wfopen(const wchar_t *, const wchar_t *);
int __cdecl ref__fileno(FILE *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);

extern FILE *(__cdecl *const ud__wfdopen)(int, const wchar_t *) __asm__(UD_IMP(_wfdopen));
extern FILE *(__cdecl *const ud__wfopen)(const wchar_t *, const wchar_t *) __asm__(UD_IMP(_wfopen));
extern int(__cdecl *const ud__fileno)(FILE *) __asm__(UD_IMP(_fileno));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));

static void run_sfm_wfdopen(void)
{
    install_counting_iph();
    wchar_t w1[MAX_PATH + 64], w2[MAX_PATH + 64];
    sfm_wide_path(w1, MAX_PATH + 64, L"sfm_wfdopen.tmp", 0);
    sfm_wide_path(w2, MAX_PATH + 64, L"sfm_wfdopen.tmp", 1);
    DeleteFileW(w1);
    DeleteFileW(w2);

    /* Same pattern as the narrow `_fdopen` row already graded: each side
     * dups a handle it owns via its OWN `_wfopen` + `_fileno`. */
    FILE *base1 = ud__wfopen(w1, L"w");
    FILE *base2 = ref__wfopen(w2, L"w");
    int fd1 = base1 ? ud__fileno(base1) : -1;
    int fd2 = base2 ? ref__fileno(base2) : -1;

    FILE *f1 = fd1 >= 0 ? ud__wfdopen(fd1, L"w") : NULL;
    FILE *f2 = fd2 >= 0 ? ref__wfdopen(fd2, L"w") : NULL;
    grade2("_wfdopen.valid", "-", f1 != 0, f2 != 0);
    if (f1) { ud_fwrite("wd", 1, 2, f1); ud_fclose(f1); } else if (base1) ud_fclose(base1);
    if (f2) { ref_fwrite("wd", 1, 2, f2); ref_fclose(f2); } else if (base2) ref_fclose(base2);

    /* Invalid handle. */
    FILE *bad1 = ud__wfdopen(-1, L"r");
    FILE *bad2 = ref__wfdopen(-1, L"r");
    grade2("_wfdopen.badfh", "-1", bad1 != 0, bad2 != 0);

    DeleteFileW(w1);
    DeleteFileW(w2);
}

#endif /* UCRTDIFF_REF_HAS__fdopen */

/* ==== fclose.cpp: _fclose_nolock ============================================ */

#if UCRTDIFF_REF_HAS_fclose

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref__fclose_nolock(FILE *);
int *__cdecl ref__errno(void);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud__fclose_nolock)(FILE *) __asm__(UD_IMP(_fclose_nolock));

static void run_sfm_fclose_nolock(void)
{
    install_counting_iph();
    const char *leaf = "sfm_fclose_nolock.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "w");
    int c1 = f1 ? ud__fclose_nolock(f1) : -2;
    int c2 = f2 ? ref__fclose_nolock(f2) : -2;
    grade2("_fclose_nolock.ok", "-", c1, c2);

    *_errno() = 0; *ref__errno() = 0;
    c1 = ud__fclose_nolock(NULL);
    c2 = ref__fclose_nolock(NULL);
    grade2("_fclose_nolock.null", "-", c1, c2);
    grade2("_fclose_nolock.null.errno", "-", *_errno(), *ref__errno());

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fclose */

/* ==== fflush.cpp: _fflush_nolock, _flushall ================================ */

#if UCRTDIFF_REF_HAS_fflush

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
int __cdecl ref__fflush_nolock(FILE *);
int __cdecl ref__flushall(void);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));
extern int(__cdecl *const ud__fflush_nolock)(FILE *) __asm__(UD_IMP(_fflush_nolock));
extern int(__cdecl *const ud__flushall)(void) __asm__(UD_IMP(_flushall));

static void run_sfm_fflush_nolock(void)
{
    install_counting_iph();
    const char *leaf = "sfm_fflush_nolock.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "w");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "w");
    if (f1) ud_fwrite("xy", 1, 2, f1);
    if (f2) ref_fwrite("xy", 1, 2, f2);

    int r1 = f1 ? ud__fflush_nolock(f1) : -1;
    int r2 = f2 ? ref__fflush_nolock(f2) : -1;
    grade2("_fflush_nolock.ret", "-", r1, r2);

    unsigned char b1[4], b2[4];
    long n1 = sfm_read_back(sd_path(leaf, 0), b1, sizeof b1);
    long n2 = sfm_read_back(sd_path(leaf, 1), b2, sizeof b2);
    grade2("_fflush_nolock.bytes", "-", n1, n2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* _flushall: return is the count of streams it (would have) flushed --
     * not comparable 1:1 across two independent stream tables that may
     * already hold a different number of open standard streams, so this
     * grades only that BOTH sides return successfully (>= 0), never a
     * negative/error sentinel. */
    r1 = ud__flushall();
    r2 = ref__flushall();
    grade2("_flushall.nonneg", "-", r1 >= 0, r2 >= 0);

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fflush */

/* ==== fread.cpp: _fread_nolock, _fread_nolock_s ============================= */

#if UCRTDIFF_REF_HAS_fread

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
size_t __cdecl ref__fread_nolock(void *, size_t, size_t, FILE *);
size_t __cdecl ref__fread_nolock_s(void *, size_t, size_t, size_t, FILE *);
int *__cdecl ref__errno(void);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));
extern size_t(__cdecl *const ud__fread_nolock)(void *, size_t, size_t, FILE *) __asm__(UD_IMP(_fread_nolock));
extern size_t(__cdecl *const ud__fread_nolock_s)(void *, size_t, size_t, size_t, FILE *) __asm__(UD_IMP(_fread_nolock_s));

static void run_sfm_fread_nolock(void)
{
    install_counting_iph();
    const unsigned char fixture[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    const char *leaf = "sfm_fread_nolock.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
    FILE *w1 = ud_fopen(sd_path(leaf, 0), "wb");
    FILE *w2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (w1) { ud_fwrite(fixture, 1, sizeof fixture, w1); ud_fclose(w1); }
    if (w2) { ref_fwrite(fixture, 1, sizeof fixture, w2); ref_fclose(w2); }

    /* Exact element count, same shape as the locked `fread` row already
     * graded (5 elements of 2 bytes). */
    FILE *f1 = ud_fopen(sd_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "rb");
    unsigned char b1[16], b2[16];
    memset(b1, 0xCD, sizeof b1); memset(b2, 0xCD, sizeof b2);
    size_t r1 = f1 ? ud__fread_nolock(b1, 2, 5, f1) : (size_t)-1;
    size_t r2 = f2 ? ref__fread_nolock(b2, 2, 5, f2) : (size_t)-1;
    grade2("_fread_nolock.count", "-", (long long)r1, (long long)r2);
    grade2("_fread_nolock.hash", "-", fnv(b1, sizeof b1), fnv(b2, sizeof b2));
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    /* _fread_nolock_s buffer-too-small: ERANGE, zero-filled, 0 returned. */
    f1 = ud_fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    *_errno() = 0; *ref__errno() = 0;
    memset(b1, 0xCD, 8); memset(b2, 0xCD, 8);
    r1 = f1 ? ud__fread_nolock_s(b1, 4, 1, 8, f1) : (size_t)-1;
    r2 = f2 ? ref__fread_nolock_s(b2, 4, 1, 8, f2) : (size_t)-1;
    grade2("_fread_nolock_s.toosmall.ret", "-", (long long)r1, (long long)r2);
    grade2("_fread_nolock_s.toosmall.errno", "-", *_errno(), *ref__errno());
    grade2("_fread_nolock_s.toosmall.hash", "-", fnv(b1, 8), fnv(b2, 8));
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fread */

/* ==== fwrite.cpp: _fwrite_nolock ============================================ */

#if UCRTDIFF_REF_HAS_fwrite

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref__fwrite_nolock(const void *, size_t, size_t, FILE *);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud__fwrite_nolock)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(_fwrite_nolock));

static void run_sfm_fwrite_nolock(void)
{
    install_counting_iph();
    const char *leaf = "sfm_fwrite_nolock.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    size_t r1 = f1 ? ud__fwrite_nolock("hello!", 1, 6, f1) : (size_t)-1;
    size_t r2 = f2 ? ref__fwrite_nolock("hello!", 1, 6, f2) : (size_t)-1;
    grade2("_fwrite_nolock.count", "-", (long long)r1, (long long)r2);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    unsigned char b1[8], b2[8];
    long n1 = sfm_read_back(sd_path(leaf, 0), b1, sizeof b1);
    long n2 = sfm_read_back(sd_path(leaf, 1), b2, sizeof b2);
    grade2("_fwrite_nolock.bytes", "-", n1, n2);
    if (n1 == n2 && n1 > 0)
        grade2("_fwrite_nolock.hash", "-", fnv(b1, (size_t)n1), fnv(b2, (size_t)n2));

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fwrite */

/* ==== fseek.cpp: _fseek_nolock, _fseeki64, _fseeki64_nolock ================ */

#if UCRTDIFF_REF_HAS_fseek

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
size_t __cdecl ref_fread(void *, size_t, size_t, FILE *);
int __cdecl ref__fseek_nolock(FILE *, long, int);
int __cdecl ref__fseeki64(FILE *, __int64, int);
int __cdecl ref__fseeki64_nolock(FILE *, __int64, int);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));
extern size_t(__cdecl *const ud_fread)(void *, size_t, size_t, FILE *) __asm__(UD_IMP(fread));
extern int(__cdecl *const ud__fseek_nolock)(FILE *, long, int) __asm__(UD_IMP(_fseek_nolock));
extern int(__cdecl *const ud__fseeki64)(FILE *, __int64, int) __asm__(UD_IMP(_fseeki64));
extern int(__cdecl *const ud__fseeki64_nolock)(FILE *, __int64, int) __asm__(UD_IMP(_fseeki64_nolock));

static void sfm_seek_fixture(const char *leaf)
{
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
    FILE *f1 = ud_fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (f1) { ud_fwrite("0123456789", 1, 10, f1); ud_fclose(f1); }
    if (f2) { ref_fwrite("0123456789", 1, 10, f2); ref_fclose(f2); }
}

static void run_sfm_fseek_nolock_variants(void)
{
    install_counting_iph();
    const char *leaf = "sfm_fseek_variants.tmp";
    sfm_seek_fixture(leaf);

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "rb");
    unsigned char b1, b2;

    int r1 = f1 ? ud__fseek_nolock(f1, 4, 0) : -1; /* SEEK_SET */
    int r2 = f2 ? ref__fseek_nolock(f2, 4, 0) : -1;
    grade2("_fseek_nolock.set.ret", "-", r1, r2);
    if (f1) ud_fread(&b1, 1, 1, f1);
    if (f2) ref_fread(&b2, 1, 1, f2);
    grade2("_fseek_nolock.set.byte", "-", f1 ? b1 : 0, f2 ? b2 : 0);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    f1 = ud_fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    r1 = f1 ? ud__fseeki64(f1, -2, 2) : -1; /* SEEK_END */
    r2 = f2 ? ref__fseeki64(f2, -2, 2) : -1;
    grade2("_fseeki64.end.ret", "-", r1, r2);
    if (f1) ud_fread(&b1, 1, 1, f1);
    if (f2) ref_fread(&b2, 1, 1, f2);
    grade2("_fseeki64.end.byte", "-", f1 ? b1 : 0, f2 ? b2 : 0);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    f1 = ud_fopen(sd_path(leaf, 0), "rb");
    f2 = ref_fopen(sd_path(leaf, 1), "rb");
    if (f1) ud_fread(&b1, 1, 1, f1); /* position 1 */
    if (f2) ref_fread(&b2, 1, 1, f2);
    r1 = f1 ? ud__fseeki64_nolock(f1, 3, 1) : -1; /* SEEK_CUR -> position 4 */
    r2 = f2 ? ref__fseeki64_nolock(f2, 3, 1) : -1;
    grade2("_fseeki64_nolock.cur.ret", "-", r1, r2);
    if (f1) ud_fread(&b1, 1, 1, f1);
    if (f2) ref_fread(&b2, 1, 1, f2);
    grade2("_fseeki64_nolock.cur.byte", "-", f1 ? b1 : 0, f2 ? b2 : 0);
    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);

    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fseek */

/* ==== ftell.cpp: _ftell_nolock, _ftelli64, _ftelli64_nolock ================= */

#if UCRTDIFF_REF_HAS_ftell

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
size_t __cdecl ref_fwrite(const void *, size_t, size_t, FILE *);
size_t __cdecl ref_fread(void *, size_t, size_t, FILE *);
long __cdecl ref__ftell_nolock(FILE *);
__int64 __cdecl ref__ftelli64(FILE *);
__int64 __cdecl ref__ftelli64_nolock(FILE *);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern size_t(__cdecl *const ud_fwrite)(const void *, size_t, size_t, FILE *) __asm__(UD_IMP(fwrite));
extern size_t(__cdecl *const ud_fread)(void *, size_t, size_t, FILE *) __asm__(UD_IMP(fread));
extern long(__cdecl *const ud__ftell_nolock)(FILE *) __asm__(UD_IMP(_ftell_nolock));
extern __int64(__cdecl *const ud__ftelli64)(FILE *) __asm__(UD_IMP(_ftelli64));
extern __int64(__cdecl *const ud__ftelli64_nolock)(FILE *) __asm__(UD_IMP(_ftelli64_nolock));

static void run_sfm_ftell_nolock_variants(void)
{
    install_counting_iph();
    const char *leaf = "sfm_ftell_variants.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
    FILE *w1 = ud_fopen(sd_path(leaf, 0), "wb");
    FILE *w2 = ref_fopen(sd_path(leaf, 1), "wb");
    if (w1) { ud_fwrite("0123456789", 1, 10, w1); ud_fclose(w1); }
    if (w2) { ref_fwrite("0123456789", 1, 10, w2); ref_fclose(w2); }

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "rb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "rb");
    unsigned char b[4];
    if (f1) ud_fread(b, 1, 4, f1);
    if (f2) ref_fread(b, 1, 4, f2);

    long t1 = f1 ? ud__ftell_nolock(f1) : -1;
    long t2 = f2 ? ref__ftell_nolock(f2) : -1;
    grade2("_ftell_nolock.pos", "-", t1, t2);

    __int64 ti1 = f1 ? ud__ftelli64(f1) : -1;
    __int64 ti2 = f2 ? ref__ftelli64(f2) : -1;
    grade2("_ftelli64.pos", "-", (long long)ti1, (long long)ti2);

    __int64 tin1 = f1 ? ud__ftelli64_nolock(f1) : -1;
    __int64 tin2 = f2 ? ref__ftelli64_nolock(f2) : -1;
    grade2("_ftelli64_nolock.pos", "-", (long long)tin1, (long long)tin2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_ftell */

/* ==== fseek.cpp + ftell.cpp together: a 64-bit position ==================== */

#if UCRTDIFF_REF_HAS_fseek && UCRTDIFF_REF_HAS_ftell

FILE *__cdecl ref_fopen(const char *, const char *);
int __cdecl ref_fclose(FILE *);
int __cdecl ref__fseeki64(FILE *, __int64, int);
__int64 __cdecl ref__ftelli64(FILE *);

extern FILE *(__cdecl *const ud_fopen)(const char *, const char *) __asm__(UD_IMP(fopen));
extern int(__cdecl *const ud_fclose)(FILE *) __asm__(UD_IMP(fclose));
extern int(__cdecl *const ud__fseeki64)(FILE *, __int64, int) __asm__(UD_IMP(_fseeki64));
extern __int64(__cdecl *const ud__ftelli64)(FILE *) __asm__(UD_IMP(_ftelli64));

static void run_sfm_seek64_big(void)
{
    install_counting_iph();
    const char *leaf = "sfm_seek64_big.tmp";
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));

    /* A position past 2^31 (bigger than `long` can hold) on a fresh, empty
     * file: a sparse SEEK_SET past EOF is cheap (no write, no real disk
     * allocation), then `_ftelli64` reads the position straight back --
     * this is the binary-mode, `!has_big_buffer()` fast path
     * (`lowio_position - stream->_cnt`), which is exact at any width. */
    const __int64 big = ((__int64)1 << 33) + 12345; /* ~8 GiB + change */

    FILE *f1 = ud_fopen(sd_path(leaf, 0), "wb");
    FILE *f2 = ref_fopen(sd_path(leaf, 1), "wb");

    int r1 = f1 ? ud__fseeki64(f1, big, 0) : -1; /* SEEK_SET */
    int r2 = f2 ? ref__fseeki64(f2, big, 0) : -1;
    grade2("seek64.set.ret", "-", r1, r2);

    __int64 t1 = f1 ? ud__ftelli64(f1) : -1;
    __int64 t2 = f2 ? ref__ftelli64(f2) : -1;
    grade2("seek64.tell", "-", (long long)t1, (long long)t2);

    if (f1) ud_fclose(f1);
    if (f2) ref_fclose(f2);
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

#endif /* UCRTDIFF_REF_HAS_fseek && UCRTDIFF_REF_HAS_ftell */

/* ---- registry: paste STDIO_FILES_MORE_HAND_ENTRIES into run_legacy()'s
 * `legacy[]` table, right after STDIO_FILES_HAND_ENTRIES. -------------------- */

#if UCRTDIFF_REF_HAS_fopen
#define SFM_HAND_fsopen run_sfm_fsopen
#define SFM_HAND_wfopen run_sfm_wfopen
#define SFM_HAND_doserrno run_sfm_doserrno
#else
#define SFM_HAND_fsopen 0
#define SFM_HAND_wfopen 0
#define SFM_HAND_doserrno 0
#endif
#if UCRTDIFF_REF_HAS_freopen
#define SFM_HAND_wfreopen run_sfm_wfreopen
#else
#define SFM_HAND_wfreopen 0
#endif
#if UCRTDIFF_REF_HAS__fdopen
#define SFM_HAND_wfdopen run_sfm_wfdopen
#else
#define SFM_HAND_wfdopen 0
#endif
#if UCRTDIFF_REF_HAS_fclose
#define SFM_HAND_fclose_nolock run_sfm_fclose_nolock
#else
#define SFM_HAND_fclose_nolock 0
#endif
#if UCRTDIFF_REF_HAS_fflush
#define SFM_HAND_fflush_nolock run_sfm_fflush_nolock
#else
#define SFM_HAND_fflush_nolock 0
#endif
#if UCRTDIFF_REF_HAS_fread
#define SFM_HAND_fread_nolock run_sfm_fread_nolock
#else
#define SFM_HAND_fread_nolock 0
#endif
#if UCRTDIFF_REF_HAS_fwrite
#define SFM_HAND_fwrite_nolock run_sfm_fwrite_nolock
#else
#define SFM_HAND_fwrite_nolock 0
#endif
#if UCRTDIFF_REF_HAS_fseek
#define SFM_HAND_fseek_variants run_sfm_fseek_nolock_variants
#else
#define SFM_HAND_fseek_variants 0
#endif
#if UCRTDIFF_REF_HAS_ftell
#define SFM_HAND_ftell_variants run_sfm_ftell_nolock_variants
#else
#define SFM_HAND_ftell_variants 0
#endif
#if UCRTDIFF_REF_HAS_fseek && UCRTDIFF_REF_HAS_ftell
#define SFM_HAND_seek64_big run_sfm_seek64_big
#else
#define SFM_HAND_seek64_big 0
#endif

#define STDIO_FILES_MORE_HAND_ENTRIES                                         \
    { "legacy:stdio_files_more-fsopen", SFM_HAND_fsopen },                    \
    { "legacy:stdio_files_more-wfopen", SFM_HAND_wfopen },                    \
    { "legacy:stdio_files_more-doserrno", SFM_HAND_doserrno },                \
    { "legacy:stdio_files_more-wfreopen", SFM_HAND_wfreopen },                \
    { "legacy:stdio_files_more-wfdopen", SFM_HAND_wfdopen },                  \
    { "legacy:stdio_files_more-fclose_nolock", SFM_HAND_fclose_nolock },      \
    { "legacy:stdio_files_more-fflush_nolock", SFM_HAND_fflush_nolock },      \
    { "legacy:stdio_files_more-fread_nolock", SFM_HAND_fread_nolock },        \
    { "legacy:stdio_files_more-fwrite_nolock", SFM_HAND_fwrite_nolock },      \
    { "legacy:stdio_files_more-fseek_variants", SFM_HAND_fseek_variants },    \
    { "legacy:stdio_files_more-ftell_variants", SFM_HAND_ftell_variants },    \
    { "legacy:stdio_files_more-seek64_big", SFM_HAND_seek64_big },


/* ==== staging/stdio_input_more.c, inlined ==== */
/*
 * staging/stdio_input_more.c — follow-up to staging/stdio_input.c (already
 * inlined into ucrtdiff.c; do NOT edit that file, this is a separate one).
 *
 * pecheck found `__stdio_common_vfwscanf` declared (the `ref_`/`ud_` pair, in
 * the inlined block) but never actually CALLED on the DLL side -- the
 * existing runners only ever drove `vfscanf` (narrow) over a real FILE*.
 * This file adds that missing call: `__stdio_common_vfwscanf` over a WIDE
 * stream, one temp file PER SIDE (never sharing a `FILE*` across sides, same
 * rule as every other stdio block), opened `ccs=UTF-16LE`.
 *
 * WHY `ccs=UTF-16LE` rather than a raw-binary UTF-16LE file: the printf block
 * already opens streams that way (`PF_STREAM_RUN(..., "w+, ccs=UTF-16LE")`,
 * ~line 1702 of ucrtdiff.c) and that block is landed and green, so the mode
 * string's parser is proven live on both the shipping DLL and the genuine
 * reference. A binary-mode stream would not work here even as a fallback:
 * `_getwc_nolock` decodes through the stream's text-mode/Unicode flags, not
 * by reading two raw bytes -- without `ccs=`/`_O_U16TEXT` it would treat each
 * byte of a UTF-16LE pair as a SEPARATE narrow multibyte character and
 * corrupt every read. So `ccs=UTF-16LE` was reachable and is what is used;
 * the binary-file fallback the brief allowed for is not needed.
 *
 * Both sides' fixture files get IDENTICAL bytes (written once, directly via
 * `WriteFile`, no CRT translation involved in the WRITE at all) and are then
 * each opened for reading by that side's OWN `_wfopen`, so a byte-level
 * difference in what each side's stream-mode parser or `_getwc_nolock`
 * decoding produces is exactly what gets graded -- not a difference in what
 * was written.
 *
 * Same guard as staging/stdio_input.c (one C file, `stdio/input.cpp`, so one
 * guard for the whole batch): `UCRTDIFF_REF_HAS___stdio_common_vsscanf`.
 * Every `ref_`/`ud_` prototype this file touches is redeclared here (a
 * harmless duplicate extern declaration in C, unlike a duplicate function
 * DEFINITION) so this file has no ordering dependency on stdio_input.c's
 * block -- it does NOT reuse that file's `static` trampolines (redefining
 * them under the same names would be a hard duplicate-definition error), and
 * uses its own, distinctly-named ones instead.
 *
 * Uses `sd_path(leaf, side)` (unconditional in ucrtdiff.c, ~line 10027) for
 * every temp path, per the rules. Nothing here changes process-wide state,
 * and every fixture file is deleted before returning regardless of outcome.
 */

#if UCRTDIFF_REF_HAS___stdio_common_vsscanf

int __cdecl ref___stdio_common_vfwscanf(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list);
extern int(__cdecl *const ud___stdio_common_vfwscanf)(unsigned long long, FILE *, const wchar_t *, _locale_t, va_list) __asm__(UD_IMP(__stdio_common_vfwscanf));

FILE *__cdecl ref__wfopen(const wchar_t *, const wchar_t *);
int __cdecl ref_fclose(FILE *);

/* A varargs trampoline over `FILE*`, distinct names from stdio_input.c's
 * `sfd_`/`sfr_` (which this file must not redefine). */
static int sc2d_vfwscanf(unsigned long long o, FILE *s, const wchar_t *f, ...)
{
    va_list ap; va_start(ap, f);
    int r = ud___stdio_common_vfwscanf(o, s, f, NULL, ap);
    va_end(ap);
    return r;
}
static int sc2r_vfwscanf(unsigned long long o, FILE *s, const wchar_t *f, ...)
{
    va_list ap; va_start(ap, f);
    int r = ref___stdio_common_vfwscanf(o, s, f, NULL, ap);
    va_end(ap);
    return r;
}

/* `sd_path` returns a narrow, ASCII-only path (temp dir + a fixed ASCII
 * leaf) -- widening it byte-for-byte is exact, no `MultiByteToWideChar`
 * needed. */
static void sc2_wpath(const char *leaf, int side, wchar_t *out)
{
    const char *p = sd_path(leaf, side);
    while (*p) { *out++ = (wchar_t)(unsigned char)*p++; }
    *out = 0;
}

static long long sc2_fp_bits64(double d) { unsigned long long u; memcpy(&u, &d, 8); return (long long)u; }

/* Writes `text` (a NUL-terminated array of `wchar_t`, no BOM -- `ccs=` already
 * names the encoding, so none is needed) as raw UTF-16LE bytes into BOTH
 * sides' own copies of `leaf`, then opens each side's copy with that side's
 * own `_wfopen`/`ref__wfopen` in `ccs=UTF-16LE` read mode and runs `fmt`
 * against it, grading the return, errno, the handler count, and every
 * receiver `grade` is handed.
 */
static void sc2_write_wide_fixture(const char *leaf, const wchar_t *text)
{
    size_t chars = 0;
    while (text[chars]) chars++;
    DWORD bytes = (DWORD)(chars * sizeof(wchar_t));
    for (int side = 0; side < 2; side++) {
        HANDLE h = CreateFileA(sd_path(leaf, side), GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                                FILE_ATTRIBUTE_NORMAL, NULL);
        if (h == INVALID_HANDLE_VALUE) continue;
        DWORD written = 0;
        WriteFile(h, text, bytes, &written, NULL);
        CloseHandle(h);
    }
}

static void sc2_cleanup(const char *leaf)
{
    DeleteFileA(sd_path(leaf, 0));
    DeleteFileA(sd_path(leaf, 1));
}

/* ---- %d, %ls, %lf, %[a-z] over a wide FILE* stream, one case each --------- */

static void run_sc2_vfwscanf_int(void)
{
    const wchar_t text[] = L"  -7\n";
    sc2_write_wide_fixture("sc2_int.txt", text);
    wchar_t p1[MAX_PATH], p2[MAX_PATH];
    sc2_wpath("sc2_int.txt", 0, p1);
    sc2_wpath("sc2_int.txt", 1, p2);
    FILE *f1 = _wfopen(p1, L"r, ccs=UTF-16LE");
    FILE *f2 = ref__wfopen(p2, L"r, ccs=UTF-16LE");
    grade2("scanf.wfile.int.open", "case", f1 != 0, f2 != 0);
    if (f1 && f2) {
        int d1 = 0x5A, d2 = 0x5A;
        install_counting_iph();
        *_errno() = 888; *ref__errno() = 888; g_iph_dll = g_iph_ref = 0;
        int r1 = sc2d_vfwscanf(0, f1, L"%d", &d1);
        int r2 = sc2r_vfwscanf(0, f2, L"%d", &d2);
        grade2("scanf.wfile.int.ret", "case", r1, r2);
        grade2("scanf.wfile.int.val", "case", d1, d2);
        grade2("scanf.wfile.int.errno", "case", *_errno(), *ref__errno());
        grade2("scanf.wfile.int.iph", "case", g_iph_dll, g_iph_ref);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    sc2_cleanup("sc2_int.txt");
}

static void run_sc2_vfwscanf_string_and_float(void)
{
    const wchar_t text[] = L"123 hello 4.5\n";
    sc2_write_wide_fixture("sc2_strf.txt", text);
    wchar_t p1[MAX_PATH], p2[MAX_PATH];
    sc2_wpath("sc2_strf.txt", 0, p1);
    sc2_wpath("sc2_strf.txt", 1, p2);
    FILE *f1 = _wfopen(p1, L"r, ccs=UTF-16LE");
    FILE *f2 = ref__wfopen(p2, L"r, ccs=UTF-16LE");
    grade2("scanf.wfile.strf.open", "case", f1 != 0, f2 != 0);
    if (f1 && f2) {
        int i1 = 0x5A, i2 = 0x5A;
        wchar_t s1[16], s2[16];
        for (int k = 0; k < 16; k++) { s1[k] = 0x5858; s2[k] = 0x5858; }
        double d1 = -1.0, d2 = -1.0;
        install_counting_iph();
        *_errno() = 888; *ref__errno() = 888; g_iph_dll = g_iph_ref = 0;
        int r1 = sc2d_vfwscanf(0, f1, L"%d %ls %lf", &i1, s1, &d1);
        int r2 = sc2r_vfwscanf(0, f2, L"%d %ls %lf", &i2, s2, &d2);
        grade2("scanf.wfile.strf.ret", "case", r1, r2);
        grade2("scanf.wfile.strf.int", "case", i1, i2);
        grade2("scanf.wfile.strf.str", "case", fnv(s1, sizeof s1), fnv(s2, sizeof s2));
        grade2("scanf.wfile.strf.float", "case", sc2_fp_bits64(d1), sc2_fp_bits64(d2));
        grade2("scanf.wfile.strf.errno", "case", *_errno(), *ref__errno());
        grade2("scanf.wfile.strf.iph", "case", g_iph_dll, g_iph_ref);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    sc2_cleanup("sc2_strf.txt");
}

static void run_sc2_vfwscanf_scanset(void)
{
    const wchar_t text[] = L"abcXYZ\n";
    sc2_write_wide_fixture("sc2_set.txt", text);
    wchar_t p1[MAX_PATH], p2[MAX_PATH];
    sc2_wpath("sc2_set.txt", 0, p1);
    sc2_wpath("sc2_set.txt", 1, p2);
    FILE *f1 = _wfopen(p1, L"r, ccs=UTF-16LE");
    FILE *f2 = ref__wfopen(p2, L"r, ccs=UTF-16LE");
    grade2("scanf.wfile.set.open", "case", f1 != 0, f2 != 0);
    if (f1 && f2) {
        wchar_t s1[16], s2[16];
        for (int k = 0; k < 16; k++) { s1[k] = 0x5858; s2[k] = 0x5858; }
        install_counting_iph();
        *_errno() = 888; *ref__errno() = 888; g_iph_dll = g_iph_ref = 0;
        int r1 = sc2d_vfwscanf(0, f1, L"%[a-z]", s1);
        int r2 = sc2r_vfwscanf(0, f2, L"%[a-z]", s2);
        grade2("scanf.wfile.set.ret", "case", r1, r2);
        grade2("scanf.wfile.set.buf", "case", fnv(s1, sizeof s1), fnv(s2, sizeof s2));
        grade2("scanf.wfile.set.errno", "case", *_errno(), *ref__errno());
        grade2("scanf.wfile.set.iph", "case", g_iph_dll, g_iph_ref);
    }
    if (f1) fclose(f1);
    if (f2) ref_fclose(f2);
    sc2_cleanup("sc2_set.txt");
}

static void run_sc2_vfwscanf(void)
{
    run_sc2_vfwscanf_int();
    run_sc2_vfwscanf_string_and_float();
    run_sc2_vfwscanf_scanset();
}

#endif /* UCRTDIFF_REF_HAS___stdio_common_vsscanf */

/*
 * Registry entry for `run_legacy()`'s table (paste alongside the `SC_HAND`
 * block staging/stdio_input.c's comment describes):
 *
 *     #if UCRTDIFF_REF_HAS___stdio_common_vsscanf
 *     #define STDIO_INPUT_MORE_HAND_ENTRIES { "legacy:vfwscanf", run_sc2_vfwscanf },
 *     #else
 *     #define STDIO_INPUT_MORE_HAND_ENTRIES
 *     #endif
 */


#if UCRTDIFF_REF_HAS___stdio_common_vsscanf
#define STDIO_INPUT_MORE_HAND_ENTRIES { "scanf:vfwscanf", run_sc2_vfwscanf },
#else
#define STDIO_INPUT_MORE_HAND_ENTRIES
#endif

static void run_legacy(void)
{
    static const struct {
        const char *name;
        void (*fn)(void);
    } legacy[] = {
        { "legacy:strtox", run_strtox },
        { "legacy:strtox-family", run_strtox_family },
        { "legacy:wcstox-family", run_wcstox_family },
        { "legacy:atox-family", run_atox_family },
        { "legacy:xtoa-family", run_xtoa_family },
        { "legacy:xtoa_s-family", run_xtoa_s_family },
        { "legacy:findfile-family", run_findfile_family },
        { "legacy:ctype", run_ctype },
        { "legacy:string", run_string },
#define HAND(name) { #name, HAND_FN_##name },
#if UCRTDIFF_REF_HAS_strcat
#define HAND_FN_strcat run_strcat
#else
#define HAND_FN_strcat 0
#endif
#if UCRTDIFF_REF_HAS_strcpy
#define HAND_FN_strcpy run_strcpy
#else
#define HAND_FN_strcpy 0
#endif
#if UCRTDIFF_REF_HAS_strncat
#define HAND_FN_strncat run_strncat
#else
#define HAND_FN_strncat 0
#endif
#if UCRTDIFF_REF_HAS_strncpy
#define HAND_FN_strncpy run_strncpy
#else
#define HAND_FN_strncpy 0
#endif
#if UCRTDIFF_REF_HAS_strpbrk
#define HAND_FN_strpbrk run_strpbrk
#else
#define HAND_FN_strpbrk 0
#endif
        HAND(strcat) HAND(strcpy) HAND(strncat) HAND(strncpy) HAND(strpbrk)
#if UCRTDIFF_REF_HAS___stdio_common_vsprintf
/* Keyed by the real export name, the unit the table and the takeover list speak in. */
#define PF_HAND(name) { "__stdio_common_" #name, run_pf_##name },
#else
#define PF_HAND(name) { "__stdio_common_" #name, 0 },
#endif
        PF_HAND(vsprintf) PF_HAND(vswprintf) PF_HAND(vsprintf_s) PF_HAND(vswprintf_s)
        PF_HAND(vsnprintf_s) PF_HAND(vsnwprintf_s) PF_HAND(vsprintf_p) PF_HAND(vswprintf_p)
        PF_HAND(vfprintf) PF_HAND(vfwprintf) PF_HAND(vfprintf_s) PF_HAND(vfwprintf_s)
        PF_HAND(vfprintf_p) PF_HAND(vfwprintf_p)
#if UCRTDIFF_REF_HAS_strtod
#define FP_HAND(name) { #name, run_fp_##name },
#else
#define FP_HAND(name) { #name, 0 },
#endif
        FP_HAND(strtof) FP_HAND(_strtof_l) FP_HAND(strtod) FP_HAND(_strtod_l)
        FP_HAND(strtold) FP_HAND(_strtold_l) FP_HAND(wcstof) FP_HAND(_wcstof_l)
        FP_HAND(wcstod) FP_HAND(_wcstod_l) FP_HAND(wcstold) FP_HAND(_wcstold_l)
#if UCRTDIFF_REF_HAS__stat64
#define ST_HAND(name) { #name, run_st_##name },
#else
#define ST_HAND(name) { #name, 0 },
#endif
        ST_HAND(_stat32) ST_HAND(_stat32i64) ST_HAND(_stat64i32) ST_HAND(_stat64)
        ST_HAND(_wstat32) ST_HAND(_wstat32i64) ST_HAND(_wstat64i32) ST_HAND(_wstat64)
        ST_HAND(_fstat32) ST_HAND(_fstat32i64) ST_HAND(_fstat64i32) ST_HAND(_fstat64)
#if UCRTDIFF_REF_HAS_setlocale
#define LOC_HAND(name) { #name, run_loc_##name },
#define LOC_HAND_AS(row, name) { row, run_loc_##name },
#else
#define LOC_HAND(name) { #name, 0 },
#define LOC_HAND_AS(row, name) { row, 0 },
#endif
        LOC_HAND(setlocale) LOC_HAND(_wsetlocale) LOC_HAND(_create_locale)
        LOC_HAND(_wcreate_locale) LOC_HAND(_get_current_locale) LOC_HAND(_configthreadlocale)
        LOC_HAND(localeconv) LOC_HAND(___lc_codepage_func) LOC_HAND(___lc_collate_cp_func)
        LOC_HAND(___lc_locale_name_func) LOC_HAND(___mb_cur_max_func) LOC_HAND(__pctype_func)
        LOC_HAND(__pwctype_func) LOC_HAND(__p__mbctype) LOC_HAND(__p__mbcasemap) LOC_HAND(_wctype)
        LOC_HAND_AS("_setmbcp+tables", _setmbcp)
        LOC_HAND(__initialize_lconv_for_unsigned_char)
        CONVTIME_HAND_ENTRIES
        HEAPIO_ALIGN_HAND_ENTRIES
        HEAPIO_IPH_HAND_ENTRIES
        HEAPIO_OPEN_HAND_ENTRIES
        SPV_HAND_ENTRIES
        MBSTRING_HAND_ENTRIES
        MBSTRING_NULL_ENTRIES
        STRING_OPS_HAND_ENTRIES
        TIME_HAND_ENTRIES
        TIME_HAND_SIBLING_ENTRIES
        BUFIO_HAND_ENTRIES
        STDIO_FILES_HAND_ENTRIES
        STDIO_INPUT_HAND_ENTRIES
        BUFIO_MORE_HAND_ENTRIES
        STDIO_FILES_MORE_HAND_ENTRIES
        STDIO_INPUT_MORE_HAND_ENTRIES
    };
    size_t i;
    for (i = 0; i < sizeof(legacy) / sizeof(legacy[0]); i++) {
        if (!legacy[i].fn) {
            DbgPrint("[UCRTDIFF] %s: no reference body (taken over) -- not graded\n",
                     legacy[i].name);
            continue;
        }
        if (export_begin(legacy[i].name)) {
            /* Before, not after: a runner that takes the whole GUEST down (a kernel
             * bugcheck, which no guard can catch) is otherwise named by nothing. */
            DbgPrint("[UCRTDIFF] -> %s\n", legacy[i].name);
            /* Every row starts with the counting handler on BOTH sides. A runner
             * that forgot it inherited whatever the shard had installed so far --
             * usually nothing, and then its first invalid-parameter case is
             * _invoke_watson's fastfail on both sides, which the arm64 guest shows
             * as a fault loop and the row as a HANG. */
            install_counting_iph();
            export_end(run_guarded(legacy[i].fn), 1);
        }
    }
}

static int ucrtdiff_main(void);

void ucrtdiff_entry(void);
void ucrtdiff_entry(void)
{
    int rc;
    /* The main thread owns the accounting until a job takes it, and the guard
     * self-test below already needs that to be true. */
    g_owner_tid = GetCurrentThreadId();
    DbgPrint("[UCRTDIFF] entry\n");
    /* BEFORE the C initializers, which is the order real startup has: ucrtbase's
     * DllMain runs __acrt_initialize long before any image's .CRT$XI* table. Linking
     * it pulls in reference objects that register XI initializers of their own, and
     * those allocate -- run first, ref__calloc_base called HeapAlloc on a NULL
     * __acrt_heap and the process died in RtlAllocateHeap. */
    if (!init_ref_crt()) {
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        ExitProcess(1);
    }
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
    g_deadline_tick = GetTickCount() + UCRTDIFF_BUDGET_MS;
    DbgPrint("[UCRTDIFF] shard=%d/%d budget=%dms\n",
             (int)UCRTDIFF_SHARD_K, (int)UCRTDIFF_SHARD_N, (int)UCRTDIFF_BUDGET_MS);
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
    /* STATE ISOLATION, and it is load-bearing for how the DIFF table is read.
     *
     * The three sides each carry their OWN locale and mbcinfo: rust_ is initialised by
     * msvcrt_init_locale() above, the DLL carries the process CRT's state set up by real
     * startup, and ref_ is a private copy of Microsoft's C linked into this exe that
     * NOTHING initialises. Measured before this call: dll=0 rust=1252 ref=0, and the
     * ref_ side's ctype tables were never populated at all.
     *
     * That produced a table of fake findings. Eight mbc case/classification exports
     * showed the DLL and rust_ sides failing identically against ref_, which read as
     * "the shipping DLL already answers these with our Rust and both are wrong". Both
     * halves were false: none of those exports are in ucrtbase's cohab archive, so the
     * DLL answers them with Microsoft's C -- the SAME code as ref_ -- and the only thing
     * that could differ was state. The uninitialised REFERENCE was the odd one out, and
     * the product was right.
     *
     * So put all three into one explicit, identical state and prove it on serial. A
     * difftest whose sides disagree about the locale is not grading our code.
     *
     * MEASURED, now that ref_ gets real startup (init_ref_crt): all three calls return
     * 0 and the line reads `dll=0 rust=20127 ref=0`. The DLL and ref_ -- the same
     * Microsoft C -- agree on every case (dll_fail=0), so no row is ungradeable any
     * more. The rust_ side stores 20127 because Wine's _setmbcp maps _MB_CP_SBCS to
     * ASCII (20127) where Microsoft stores 0: a real CONTRACT difference, and the
     * root of the rust_ side's high-range mbc answers.
     *
     * CORRECTION. An earlier version of this comment reported `rust=1252` after these
     * calls and concluded that our _setmbcp "resolves 0 to the ANSI codepage". Neither
     * holds: create_mbcinfo maps 0 to 20127, and with return codes on serial the
     * rust_ call is seen to succeed and store exactly that. WHY that earlier run read
     * 1252 is not established -- 1252 is msvcrt_init_locale's value, so the call's
     * effect never reached the readback, and a stale harness binary would do it, but
     * that was not shown. What is established is the lesson: a readback without its
     * call's return code is not a measurement of the call. */
    /* A line BEFORE each call as well as after: the process has died here with
     * STATUS_POSSIBLE_DEADLOCK (0xC0000194), and a call that never returns prints
     * nothing of its own. The last "->" line on serial names it. */
    DbgPrint("[UCRTDIFF] -> dll setlocale\n");
    setlocale(LC_ALL, "C");
    /* The reference's locale exists only while the locale takeover's Microsoft
     * objects do: once taken over, a plain sweep has no ref_ side to align. */
#if UCRTDIFF_REF_CALLABLE_setlocale
    DbgPrint("[UCRTDIFF] -> ref setlocale\n");
    ref_setlocale(LC_ALL, "C");
#endif
    DbgPrint("[UCRTDIFF] -> rust setlocale\n");
    rust_setlocale(LC_ALL, "C");
    DbgPrint("[UCRTDIFF] -> setmbcp\n");
    /* One serial line PER CALL, each printed after its call returns: a failing
     * _setmbcp leaves the PREVIOUS codepage installed, so a readback alone cannot
     * tell "stored 1252" from "refused and kept the 1252 msvcrt_init_locale put
     * there". And one of these three has been seen to die in a critical section
     * (the process exited STATUS_POSSIBLE_DEADLOCK, 0xC0000194, right here) -- with
     * a line per call, the last line on serial names which one. */
    DbgPrint("[UCRTDIFF] setmbcp(0) dll rc=%d\n", _setmbcp(0));
#if UCRTDIFF_REF_CALLABLE_setlocale
    DbgPrint("[UCRTDIFF] setmbcp(0) ref rc=%d\n", ref__setmbcp(0));
#endif
    DbgPrint("[UCRTDIFF] setmbcp(0) rust rc=%d\n", rust__setmbcp(0));

    /* The old diagnostic, kept as the PROOF the alignment took. If these three ever
     * disagree again, every mbc row below is comparing codepages, not implementations.
     * The three sides each carry their OWN locale and mbcinfo: rust_ is initialised by
     * msvcrt_init_locale() above, the DLL carries the process CRT's state, and ref_ is
     * a private copy of Microsoft's C that nothing here initialises. If their multibyte
     * codepages differ then every mbc classification and case-mapping row is graded
     * against a different codepage on each side, and the resulting "divergences" say
     * nothing about our code. The design doc listed this as NOT VERIFIED; print it, so
     * the table can never again be read without knowing which case holds. */
#if UCRTDIFF_REF_CALLABLE_setlocale
    DbgPrint("[UCRTDIFF] mbcp: dll=%d rust=%d ref=%d\n",
             _getmbcp(), rust__getmbcp(), ref__getmbcp());
#else
    DbgPrint("[UCRTDIFF] mbcp: dll=%d rust=%d ref=(taken over)\n", _getmbcp(), rust__getmbcp());
#endif

    run_generated();
    run_legacy();
    /* A ZERO-case run is a vacuous pass and must read as a failure, the same
     * rule tests/qemu/cli/difftest.py's graded() enforces for the host tier. */
    if (g_cases == 0) {
        DbgPrint("[UCRTDIFF] FATAL: no cases ran\n");
        DbgPrint("[UCRTDIFF] done cases=0 dll_fail=1 rust_fail=1\n");
        return 1;
    }
    /* PARTIAL is the important field: it is the difference between "this shard
     * graded everything it owns" and "the deadline cut it short", which previously
     * were the same observation -- a serial log that stopped. */
    DbgPrint("[UCRTDIFF] done cases=%d dll_pass=%d dll_fail=%d rust_pass=%d rust_fail=%d "
             "shard=%d/%d exports=%d skipped_shard=%d partial=%d "
             "zombie_faults=%d dropped_writes=%d\n",
             g_cases, g_dll_pass, g_dll_fail, g_rust_pass, g_rust_fail,
             (int)UCRTDIFF_SHARD_K, (int)UCRTDIFF_SHARD_N, g_ran_exports,
             g_skip_shard, g_skip_budget, g_zombie_faults, g_dropped_writes);
    return (g_dll_fail || g_rust_fail) != 0;
}
