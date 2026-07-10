/*
 * User-mode SEH boot self-test — the userland sibling of ntoskrnl/seh_poc.c
 * (crash-value lane; see docs/engineering/dummy-pseh-verdict.md).
 *
 * Compiled with -fms-extensions so __try/__except are the REAL clang SEH
 * keywords (this file deliberately includes no headers so no pseh macro
 * layer can interfere). Spawned fire-and-forget by userinit before the
 * shell starts; prints [SEH-USER] lines on the kernel debug serial.
 *
 * Exercises the full user-mode exception chain both ways:
 *   1. hardware fault: NULL write in a noinline callee (clang's mingw SEH
 *      is CALLS-ONLY — a direct deref in the __try body gets its handler
 *      DCE'd, so the fault must arrive through a call frame) →
 *      MmAccessFault → KiDispatchException → KiUserExceptionDispatcher →
 *      ntdll RtlDispatchException → .pdata walk → __C_specific_handler →
 *      filter → RtlUnwind → handler.
 *   2. software raise: kernel32 RaiseException → NtRaiseException →
 *      same dispatch chain.
 *
 * A FAIL (or this process dying silently) means user-mode exception
 * delivery regressed — check KiDispatchExceptionToUser, the dispatch.S
 * swallow branch, and that this module still ships real unwind tables.
 */

unsigned long __stdcall GetVersion(void);         /* anchor kernel32 import */
unsigned long DbgPrint(const char *Format, ...);  /* ntdll */
void __stdcall RaiseException(unsigned long Code, unsigned long Flags,
                              unsigned long NumArgs, const unsigned long long *Args);

#define SEH_USER_RAISE_STATUS 0xE0DEAD02L
#define STATUS_ACCESS_VIOLATION 0xC0000005L

/* noinline + noipa-ish: keep the fault in a real callee frame */
__attribute__((noinline)) static void FaultingWrite(volatile int *p)
{
    *p = 42;
}

static int TestHardwareFault(void)
{
    volatile int caught = 0;
    volatile long code = 0;

    __try {
        FaultingWrite((volatile int *)0);
        caught = -1; /* unreachable if the fault dispatches */
    } __except (_exception_code() == STATUS_ACCESS_VIOLATION ? 1 : 0) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-USER] hw-fault: caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == STATUS_ACCESS_VIOLATION) ? "PASS" : "FAIL");
    return caught == 1 && code == STATUS_ACCESS_VIOLATION;
}

static int TestSoftwareRaise(void)
{
    volatile int caught = 0;
    volatile long code = 0;

    __try {
        RaiseException(SEH_USER_RAISE_STATUS, 0, 0, 0);
        caught = -1;
    } __except (_exception_code() == SEH_USER_RAISE_STATUS ? 1 : 0) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-USER] sw-raise: caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == SEH_USER_RAISE_STATUS) ? "PASS" : "FAIL");
    return caught == 1 && code == SEH_USER_RAISE_STATUS;
}

static int TestFinally(void)
{
    volatile int ran_finally = 0;
    volatile int caught = 0;

    __try {
        __try {
            RaiseException(SEH_USER_RAISE_STATUS, 0, 0, 0);
        } __finally {
            ran_finally = 1;
        }
    } __except (1) {
        caught = 1;
    }

    DbgPrint("[SEH-USER] finally: ran=%d caught=%d (%s)\n",
             ran_finally, caught,
             (ran_finally == 1 && caught == 1) ? "PASS" : "FAIL");
    return ran_finally == 1 && caught == 1;
}

int main(void)
{
    int pass = 1;

    DbgPrint("[SEH-USER] user-mode SEH self-test starting (GetVersion=0x%lx)\n",
             GetVersion());

    pass &= TestSoftwareRaise();   /* least demanding first */
    pass &= TestFinally();
    pass &= TestHardwareFault();   /* the real hardware→userland roundtrip */

    DbgPrint("[SEH-USER] RESULT: %s\n", pass ? "PASS" : "FAIL");
    return pass ? 0 : 1;
}
