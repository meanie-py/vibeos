/*
 * test_KeEvent_host.c — Host wrapper for KeEvent functional test
 *
 * Includes KeEvent.c directly but only calls TestEventFunctional.
 * Concurrent and scheduling tests require real threading — skipped.
 */

#include "include/kmt_test.h"

/* Constants referenced by concurrent test paths (never executed) */
#define GENERIC_ALL     0x10000000
#define SYNCHRONIZE     0x00100000
#define WaitAll         0
#define WaitAny         1
#define IO_NO_INCREMENT 0
#define STATUS_WAIT_0   ((NTSTATUS)0x00000000)
#define MILLISECOND     (10000LL)

/* Object type stub for ObReferenceObjectByHandle */
typedef PVOID POBJECT_TYPE;
static POBJECT_TYPE _PsThreadType_val;
#define PsThreadType (&_PsThreadType_val)

/* Stubs for thread/object functions used by concurrent test paths */
static NTSTATUS _stub_PsCreateSystemThread(PHANDLE a, ULONG b, PVOID c,
    HANDLE d, PVOID e, void (*f)(PVOID), PVOID g) {
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;
    return STATUS_UNSUCCESSFUL;
}
#define PsCreateSystemThread(a,b,c,d,e,f,g) _stub_PsCreateSystemThread(a,b,c,d,e,f,g)

static NTSTATUS _stub_ObRefByHandle(HANDLE a, ULONG b, POBJECT_TYPE c,
    LONG d, PVOID *e, PVOID f) {
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    return STATUS_UNSUCCESSFUL;
}
#define ObReferenceObjectByHandle(a,b,c,d,e,f) _stub_ObRefByHandle(a,b,c,d,e,f)

static void _stub_ObDeref(PVOID o) { (void)o; }
#define ObDereferenceObject(o) _stub_ObDeref(o)

static NTSTATUS _stub_WaitMulti(ULONG a, PVOID *b, ULONG c, KWAIT_REASON d,
    LONG e, BOOLEAN f, PLARGE_INTEGER g, PVOID h) {
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;
    return STATUS_UNSUCCESSFUL;
}
#define KeWaitForMultipleObjects(a,b,c,d,e,f,g,h) _stub_WaitMulti(a,b,c,d,e,f,g,h)

static NTSTATUS _stub_Delay(LONG a, BOOLEAN b, PLARGE_INTEGER c) {
    (void)a;(void)b;(void)c;
    return STATUS_SUCCESS;
}
#define KeDelayExecutionThread(a,b,c) _stub_Delay(a,b,c)

static KPRIORITY _stub_QueryPrio(PKTHREAD t) { (void)t; return 8; }
#define KeQueryPriorityThread(t) _stub_QueryPrio(t)

static PKTHREAD _stub_StartThread(void (*f)(PVOID), PVOID c) {
    (void)f;(void)c; return NULL;
}
#define KmtStartThread(f, c) _stub_StartThread((void(*)(PVOID))(f), c)

static void _stub_FinishThread(PKTHREAD t, PVOID r) { (void)t;(void)r; }
#define KmtFinishThread(t, r) _stub_FinishThread(t, r)

/* Override START_TEST so the original entry point compiles but is unused */
#undef START_TEST
#define START_TEST(name) \
    static void _Unused_##name(void) __attribute__((unused)); \
    static void _Unused_##name(void)

/* Include the actual test source — picks up static TestEventFunctional */
#include "../../modules/rostests/kmtests/ntos_ke/KeEvent.c"

/* Our test entry point — functional parts only */
#undef START_TEST
void Test_KeEvent(void) {
    KEVENT Event;
    KIRQL Irql;
    KIRQL Irqls[] = { PASSIVE_LEVEL, APC_LEVEL, DISPATCH_LEVEL };

    for (unsigned i = 0; i < RTL_NUMBER_OF(Irqls); ++i) {
        KeRaiseIrql(Irqls[i], &Irql);
        TestEventFunctional(&Event, NotificationEvent, Irqls[i]);
        TestEventFunctional(&Event, SynchronizationEvent, Irqls[i]);
        KeLowerIrql(Irql);
    }
}
