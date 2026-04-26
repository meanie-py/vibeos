/*
 * test_runner.c — Host-side test runner for ReactOS kernel algorithm tests
 *
 * Compiles and runs GREEN-category kernel tests natively on macOS/Linux
 * without booting QEMU. Uses the same assertion macros as the in-target
 * kmtest framework.
 *
 * Usage:
 *   make              # build all tests
 *   ./run_tests       # run all tests
 *   ./run_tests Name  # run a specific test
 */

#include "include/kmt_test.h"

/* Global result tracking */
KMT_HOST_RESULTS _kmt_results = { 0, 0, 0, NULL };

/* Forward declarations of test functions */
extern VOID Test_RtlSplayTree(VOID);
extern VOID Test_RtlAvlTree(VOID);
extern VOID Test_FsRtlDissect(VOID);
extern VOID Test_FsRtlLegal(VOID);
extern VOID Test_FsRtlExpression(VOID);
extern VOID Test_RtlMemory(VOID);
extern VOID Test_ExDoubleList(VOID);
extern VOID Test_ExSingleList(VOID);
extern VOID Test_RtlUnicodeString(VOID);
extern VOID Test_RtlRangeList(VOID);
extern VOID Test_FsRtlMcb(VOID);
extern VOID Test_FsRtlTunnel(VOID);
extern VOID Test_FsRtlRemoveDotsFromPath(VOID);
extern VOID Test_RtlException(VOID);
extern VOID Test_ExUuid(VOID);
extern VOID Test_ExInterlocked(VOID);
extern VOID Test_KeDeviceQueue(VOID);
extern VOID Test_KeIrql(VOID);
extern VOID Test_GuardedMemory(VOID);
extern VOID Test_KeApc(VOID);
extern VOID Test_RtlIsValidOemCharacter(VOID);
extern VOID Test_ExSequencedList(VOID);
extern VOID Test_KeTimer(VOID);
extern VOID Test_KeMutex(VOID);
extern VOID Test_KeEvent(VOID);

/* FsRtl global initializers */
extern VOID NTAPI FsRtlInitializeLargeMcbs(VOID);
extern VOID NTAPI FsRtlInitializeTunnels(VOID);

/* Test registry */
typedef struct {
    const char *name;
    KMT_TESTFUNC *func;
} TEST_ENTRY;

static const TEST_ENTRY TestList[] = {
    { "RtlSplayTree",    Test_RtlSplayTree },
    { "RtlAvlTree",      Test_RtlAvlTree },
    { "FsRtlDissect",    Test_FsRtlDissect },
    { "FsRtlLegal",      Test_FsRtlLegal },
    { "FsRtlExpression", Test_FsRtlExpression },
    { "RtlMemory",       Test_RtlMemory },
    { "ExDoubleList",    Test_ExDoubleList },
    { "ExSingleList",    Test_ExSingleList },
    { "RtlUnicodeString", Test_RtlUnicodeString },
    { "RtlRangeList",    Test_RtlRangeList },
    { "FsRtlMcb",        Test_FsRtlMcb },
    { "FsRtlTunnel",     Test_FsRtlTunnel },
    { "FsRtlRemoveDotsFromPath", Test_FsRtlRemoveDotsFromPath },
    { "RtlException",   Test_RtlException },
    { "ExUuid",          Test_ExUuid },
    { "ExInterlocked",   Test_ExInterlocked },
    { "KeDeviceQueue",   Test_KeDeviceQueue },
    { "KeIrql",          Test_KeIrql },
    { "GuardedMemory",   Test_GuardedMemory },
    { "KeApc",           Test_KeApc },
    { "RtlIsValidOemCharacter", Test_RtlIsValidOemCharacter },
    { "ExSequencedList", Test_ExSequencedList },
    { "KeTimer",         Test_KeTimer },
    { "KeMutex",         Test_KeMutex },
    { "KeEvent",         Test_KeEvent },
};

#define NUM_TESTS (sizeof(TestList) / sizeof(TestList[0]))

static void run_test(const TEST_ENTRY *test) {
    LONG prev_success = _kmt_results.Successes;
    LONG prev_fail = _kmt_results.Failures;
    LONG prev_skip = _kmt_results.Skipped;

    _kmt_results.CurrentTest = test->name;
    fprintf(stderr, "[TEST] %s ...\n", test->name);

    test->func();

    LONG s = _kmt_results.Successes - prev_success;
    LONG f = _kmt_results.Failures - prev_fail;
    LONG k = _kmt_results.Skipped - prev_skip;

    if (f == 0) {
        fprintf(stderr, "[PASS] %s (%ld ok", test->name, (long)s);
    } else {
        fprintf(stderr, "[FAIL] %s (%ld ok, %ld failed", test->name, (long)s, (long)f);
    }
    if (k > 0) fprintf(stderr, ", %ld skipped", (long)k);
    fprintf(stderr, ")\n\n");
}

int main(int argc, char **argv) {
    const char *filter = argc > 1 ? argv[1] : NULL;
    int ran = 0;

    fprintf(stderr, "ReactOS Host-Side Test Runner\n");
    fprintf(stderr, "=============================\n\n");

    /* Initialize host thread state (MutantListHead etc.) */
    _host_init_thread();

    /* Initialize FsRtl subsystems */
    FsRtlInitializeLargeMcbs();
    FsRtlInitializeTunnels();

    for (size_t i = 0; i < NUM_TESTS; i++) {
        if (filter && strcmp(filter, TestList[i].name) != 0) continue;
        run_test(&TestList[i]);
        ran++;
    }

    if (ran == 0 && filter) {
        fprintf(stderr, "No test matching '%s'. Available tests:\n", filter);
        for (size_t i = 0; i < NUM_TESTS; i++) {
            fprintf(stderr, "  %s\n", TestList[i].name);
        }
        return 1;
    }

    fprintf(stderr, "=============================\n");
    fprintf(stderr, "Total: %ld passed, %ld failed, %ld skipped\n",
            (long)_kmt_results.Successes,
            (long)_kmt_results.Failures,
            (long)_kmt_results.Skipped);

    return _kmt_results.Failures > 0 ? 1 : 0;
}
