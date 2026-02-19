/*
 * kmt_test.h — Host-side test framework replacing ReactOS kmt_test.h
 *
 * Same assertion macros (ok, trace, skip, ok_eq_*) and START_TEST as
 * the ReactOS kernel-mode test framework, targeting macOS/Linux host.
 */

#ifndef _KMTEST_TEST_H_
#define _KMTEST_TEST_H_

#include "kmt_platform.h"
#include <string.h>

#define GetNTVersion() ((SharedUserData->NtMajorVersion << 8) | SharedUserData->NtMinorVersion)

/* ====================================================================
 * Result tracking
 * ==================================================================== */

typedef struct {
    volatile LONG Successes;
    volatile LONG Failures;
    volatile LONG Skipped;
    const char *CurrentTest;
} KMT_HOST_RESULTS;

extern KMT_HOST_RESULTS _kmt_results;

/* ====================================================================
 * Test function signature
 * ==================================================================== */

typedef VOID KMT_TESTFUNC(VOID);

#define START_TEST(name) VOID Test_##name(VOID)

/* ====================================================================
 * Assertion macros
 * ==================================================================== */

/* Use simple ++ instead of InterlockedIncrement — single-threaded host.
 * Also avoids link errors when ExInterlocked.c does #undef InterlockedIncrement. */
#define ok(test, ...) \
    do { \
        if (test) { \
            ++_kmt_results.Successes; \
        } else { \
            ++_kmt_results.Failures; \
            fprintf(stderr, "  FAIL %s:%d: ", __FILE__, __LINE__); \
            fprintf(stderr, __VA_ARGS__); \
        } \
    } while(0)

#define ok_(test, file, line, ...) \
    do { \
        if (test) { \
            ++_kmt_results.Successes; \
        } else { \
            ++_kmt_results.Failures; \
            fprintf(stderr, "  FAIL %s:%d: ", file, line); \
            fprintf(stderr, __VA_ARGS__); \
        } \
    } while(0)

#define trace(...) \
    do { \
        fprintf(stderr, "  TRACE %s:%d: ", __FILE__, __LINE__); \
        fprintf(stderr, __VA_ARGS__); \
    } while(0)

static inline int _kmt_skip(const char *file, int line, int test, const char *fmt, ...) {
    if (!(test)) {
        ++_kmt_results.Skipped;
        fprintf(stderr, "  SKIP %s:%d: ", file, line);
        va_list ap;
        va_start(ap, fmt);
        vfprintf(stderr, fmt, ap);
        va_end(ap);
        return 1;
    }
    return 0;
}
#define skip(test, ...) _kmt_skip(__FILE__, __LINE__, (int)(test), __VA_ARGS__)

/* ok_eq_* — typed equality assertions */
#define ok_eq_print(value, expected, spec) \
    ok((value) == (expected), \
       #value " = " spec ", expected " spec "\n", value, expected)

#define ok_eq_hex(value, expected)   ok_eq_print((unsigned long)(value), (unsigned long)(expected), "0x%08lx")
#define ok_eq_uint(value, expected)  ok_eq_print((unsigned int)(value), (unsigned int)(expected), "%u")
#define ok_eq_ulong(value, expected) ok_eq_print((unsigned long)(value), (unsigned long)(expected), "%lu")
#define ok_eq_long(value, expected)  ok_eq_print((long)(value), (long)(expected), "%ld")
#define ok_eq_int(value, expected)   ok_eq_print((int)(value), (int)(expected), "%d")
#define ok_eq_char(value, expected)  ok_eq_print((int)(value), (int)(expected), "%d")
#define ok_eq_size(value, expected)  ok_eq_print((unsigned long)(value), (unsigned long)(expected), "%lu")
#define ok_eq_wchar(value, expected) ok_eq_print((int)(value), (int)(expected), "%d")
#define ok_eq_pointer(value, expected) ok((value) == (expected), #value " = %p, expected %p\n", (void*)(value), (void*)(expected))
#define ok_eq_str(value, expected) ok(!strcmp(value, expected), #value " = \"%s\", expected \"%s\"\n", value, expected)

/* ok_eq_wstr uses 16-bit wcscmp (our _nt_wcscmp) since -fshort-wchar makes system wcscmp wrong.
 * Use %ls with caution — on macOS with -fshort-wchar, %ls may not print correctly.
 * The assertion comparison is correct regardless. */
#define ok_eq_wstr(value, expected) ok(!wcscmp(value, expected), #value " doesn't match expected wide string\n")

#define ok_eq_bool(value, expected) ok((value) == (expected), #value " = %s, expected %s\n", \
                                                (value) ? "TRUE" : "FALSE",                  \
                                                (expected) ? "TRUE" : "FALSE")

#define ok_bool_true(value, desc) ok((value) != 0, desc " is FALSE, expected TRUE\n")
#define ok_bool_false(value, desc) ok((value) == 0, desc " is TRUE, expected FALSE\n")

#define ok_irql(expected) ok_eq_uint(KeGetCurrentIrql(), (expected))
#define ok_eq_tag(value, expected) ok_eq_hex((value), (expected))

/* ====================================================================
 * SEH wrappers for tests
 * ==================================================================== */

#define KmtStartSeh() \
    { \
        NTSTATUS ExceptionStatus = STATUS_SUCCESS; \
        _SEH2_TRY \
        {

#define KmtEndSeh(ExpectedStatus) \
        } \
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) \
        { \
            ExceptionStatus = _SEH2_GetExceptionCode(); \
        } \
        _SEH2_END; \
        ok_eq_hex(ExceptionStatus, ExpectedStatus); \
    }

/* ====================================================================
 * KmtOk / KmtSkip — file:line-based assertion functions
 * Used by tests that pass custom file/line strings (e.g., RtlRangeList).
 * ==================================================================== */

#define KmtInvalidPointer ((PVOID)0x5555555555555555ULL)

static inline BOOLEAN KmtOk(INT Condition, PCSTR FileAndLine, PCSTR Format, ...) {
    if (Condition) {
        InterlockedIncrement(&_kmt_results.Successes);
        return TRUE;
    } else {
        InterlockedIncrement(&_kmt_results.Failures);
        fprintf(stderr, "  FAIL %s: ", FileAndLine);
        va_list ap;
        va_start(ap, Format);
        vfprintf(stderr, Format, ap);
        va_end(ap);
        return FALSE;
    }
}

static inline BOOLEAN KmtSkip(INT Condition, PCSTR FileAndLine, PCSTR Format, ...) {
    if (Condition) return FALSE; /* not skipped */
    InterlockedIncrement(&_kmt_results.Skipped);
    fprintf(stderr, "  SKIP %s: ", FileAndLine);
    va_list ap;
    va_start(ap, Format);
    vfprintf(stderr, Format, ap);
    va_end(ap);
    return TRUE; /* skipped */
}

/* ====================================================================
 * Guarded memory allocation via mmap + guard pages
 * ==================================================================== */

static inline PVOID KmtAllocateGuarded(SIZE_T Size) {
    size_t page_size = (size_t)getpagesize();
    size_t total_pages = (Size + page_size - 1) / page_size + 1;
    size_t total_size = total_pages * page_size;

    void *base = mmap(NULL, total_size, PROT_READ | PROT_WRITE,
                       MAP_PRIVATE | MAP_ANON, -1, 0);
    if (base == MAP_FAILED) return NULL;

    /* Last page is a guard (no access) */
    mprotect((char *)base + (total_pages - 1) * page_size, page_size, PROT_NONE);

    /* Return pointer so Buffer+Size ends at guard page boundary */
    return (char *)base + (total_pages - 1) * page_size - Size;
}

static inline void KmtFreeGuarded(PVOID Pointer) {
    size_t page_size = (size_t)getpagesize();
    uintptr_t ptr = (uintptr_t)Pointer;
    uintptr_t page_base = ptr & ~(page_size - 1);
    munmap((void *)page_base, page_size * 2);
}

/* ====================================================================
 * KmtGetSystemOrEmbeddedRoutineAddress — on host, just use the
 * linked-in function directly (sets pFuncName = FuncName)
 * ==================================================================== */

#define KmtGetSystemOrEmbeddedRoutineAddress(RoutineName) \
    p##RoutineName = RoutineName

#endif /* _KMTEST_TEST_H_ */
