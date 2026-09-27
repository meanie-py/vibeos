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
#if UCRTDIFF_REF_HAS_setlocale
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
#if UCRTDIFF_REF_HAS_setlocale
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
#if UCRTDIFF_REF_HAS_setlocale
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
