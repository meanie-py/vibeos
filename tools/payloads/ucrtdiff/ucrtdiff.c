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
    };
    size_t i;
    for (i = 0; i < sizeof(legacy) / sizeof(legacy[0]); i++) {
        if (!legacy[i].fn) {
            DbgPrint("[UCRTDIFF] %s: no reference body (taken over) -- not graded\n",
                     legacy[i].name);
            continue;
        }
        if (export_begin(legacy[i].name))
            export_end(run_guarded(legacy[i].fn), 1);
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
