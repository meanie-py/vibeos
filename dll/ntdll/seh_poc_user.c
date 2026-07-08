/*
 * User-mode SEH / fault-delivery boot self-test (arm64-focused) — the FEX
 * bring-up "day-one risk" proof: can vibeos deliver a synthesized fault into a
 * controlled user context and have a handler run? See
 * docs/engineering/fex-bringup-plan.md rung 0e and FEX-ARM-SPIKE-BRIEF.md.
 *
 * This is the user-mode sibling of ntoskrnl/seh_poc.c. Compiled with
 * -fms-extensions so __try/__except are the REAL clang SEH keywords (not the
 * pseh2.h DUMMY macros); includes no headers so no macro layer interferes.
 *
 * Two independent probes, both printing `[SEH-POC-U] ...` on serial:
 *
 *   1. TABLE-BASED DISPATCH (software raise via a call): RtlRaiseException with
 *      a custom status, caught by __try/__except. Exercises
 *      KiUserExceptionDispatcher -> RtlDispatchException -> .pdata/.xdata walk
 *      (the new arm64 unwind-code decoder) -> __C_specific_handler -> filter ->
 *      RtlUnwindEx -> handler. This is the "does SEH work through Rust ntdll
 *      frames" test. (Note: a *hardware* deref inside __try would NOT be caught
 *      on mingw — async-exception handler liveness isn't emitted — so we raise
 *      via a call, matching the FEX raise path.)
 *
 *   2. VECTORED HANDLER on a HARDWARE fault (the FEX analog): register a VEH,
 *      deliberately dereference a null pointer to raise a real access
 *      violation, and have the VEH advance PC past the faulting instruction and
 *      resume (EXCEPTION_CONTINUE_EXECUTION). Exercises the kernel->user fault
 *      delivery path AND the newly-wired first-chance
 *      RtlCallVectoredExceptionHandlers in arm64 RtlDispatchException. This is
 *      the mechanism FEX uses to catch guest #PF/#GP.
 *
 * A FAIL line (or a silent process death / second-chance) means the user-mode
 * exception machinery isn't ready for FEX; that is itself the deliverable
 * (a precise wedge report) per the brief.
 */

/* ---- minimal NT decls (no headers on purpose) ---- */
typedef unsigned long ULONG;
typedef unsigned long long ULONG64;
typedef long LONG;
typedef void *PVOID;

typedef struct _EXCEPTION_RECORD {
    ULONG ExceptionCode;
    ULONG ExceptionFlags;
    struct _EXCEPTION_RECORD *ExceptionRecord;
    PVOID ExceptionAddress;
    ULONG NumberParameters;
    ULONG64 ExceptionInformation[15];
} EXCEPTION_RECORD, *PEXCEPTION_RECORD;

/* arm64 CONTEXT prefix — we only need Pc, which sits after ContextFlags(4) +
 * Cpsr(4) + X[0..30] (31*8) + Sp(8). Offsets: Pc at 0x108. We use a byte-offset
 * accessor rather than the full struct to avoid pulling headers. */
typedef struct _CONTEXT_ARM64_MIN {
    ULONG ContextFlags;
    ULONG Cpsr;
    ULONG64 X[31];   /* X0..X30 (X29=Fp, X30=Lr) */
    ULONG64 Sp;
    ULONG64 Pc;
    /* ... V[]/Fpcr/... follow; not needed here */
} CONTEXT_ARM64_MIN;

typedef struct _EXCEPTION_POINTERS {
    PEXCEPTION_RECORD ExceptionRecord;
    CONTEXT_ARM64_MIN *ContextRecord;
} EXCEPTION_POINTERS;

#define EXCEPTION_CONTINUE_EXECUTION (-1)
#define EXCEPTION_CONTINUE_SEARCH    (0)
#define STATUS_ACCESS_VIOLATION      0xC0000005UL
#define SEH_POC_U_STATUS             0xE0DEAD02UL

/* imports from ntdll itself */
void RtlRaiseException(PEXCEPTION_RECORD ExceptionRecord);
PVOID RtlAddVectoredExceptionHandler(ULONG First, PVOID Handler);
ULONG RtlRemoveVectoredExceptionHandler(PVOID Handle);
ULONG DbgPrint(const char *Format, ...);

/* ---- probe 1: table-based dispatch via software raise ---- */
static void SehPocUserRaiseTest(void)
{
    volatile int caught = 0;
    volatile ULONG code = 0;

    __try {
        EXCEPTION_RECORD rec;
        rec.ExceptionCode = SEH_POC_U_STATUS;
        rec.ExceptionFlags = 0;              /* continuable */
        rec.ExceptionRecord = 0;
        rec.ExceptionAddress = 0;
        rec.NumberParameters = 0;
        RtlRaiseException(&rec);
        caught = -1;                          /* unreachable if dispatched */
    } __except (_exception_code() == SEH_POC_U_STATUS
                    ? 1 /* EXCEPTION_EXECUTE_HANDLER */
                    : 0 /* EXCEPTION_CONTINUE_SEARCH */) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-POC-U] table dispatch (RtlRaiseException): caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == SEH_POC_U_STATUS) ? "PASS" : "FAIL");
}

/* ---- probe 2: VEH on a hardware access violation (FEX analog) ---- */
static volatile int g_veh_fired = 0;

static LONG SehPocUserVeh(EXCEPTION_POINTERS *Info)
{
    if (Info && Info->ExceptionRecord
        && Info->ExceptionRecord->ExceptionCode == STATUS_ACCESS_VIOLATION) {
        g_veh_fired = 1;
        /* Advance past the faulting 4-byte arm64 instruction and resume, so the
         * store to *null is skipped instead of re-faulting. This is exactly the
         * "fix up guest state and continue" move a JIT does. */
        if (Info->ContextRecord)
            Info->ContextRecord->Pc += 4;
        return EXCEPTION_CONTINUE_EXECUTION;
    }
    return EXCEPTION_CONTINUE_SEARCH;
}

static void SehPocUserVehTest(void)
{
    PVOID handle;
    volatile int reached_after = 0;

    g_veh_fired = 0;
    handle = RtlAddVectoredExceptionHandler(1 /* first */, (PVOID)SehPocUserVeh);
    if (!handle) {
        DbgPrint("[SEH-POC-U] VEH register FAILED (RtlAddVectoredExceptionHandler==NULL)\n");
        return;
    }

    /* Deliberate null-store hardware fault. `volatile` keeps the store; the VEH
     * skips it. If delivery/VEH-dispatch is broken this second-chances and the
     * process dies (observably: no PASS line, boot may wedge). */
    {
        volatile int *p = (volatile int *)0;
        *p = 0x1234;
        reached_after = 1;
    }

    RtlRemoveVectoredExceptionHandler(handle);

    DbgPrint("[SEH-POC-U] VEH on hardware AV: fired=%d resumed=%d (%s)\n",
             g_veh_fired, reached_after,
             (g_veh_fired == 1 && reached_after == 1) ? "PASS" : "FAIL");
}

/*
 * Probe 2 (VEH-on-hardware-AV) is DISABLED pending the kernel fix. It proved
 * that arm64 kernel->user hardware-fault delivery works AND the VEH fires and
 * resumes — but the resume corrupts callee-saved x19..x28, because the arm64
 * fault path (ke/arm64/trap.rs) calls KiDispatchException with a NULL
 * KEXCEPTION_FRAME, so KeTrapFrameToContext can't capture x19..x28 and
 * NtContinue resumes them as 0. Re-enable once trap.rs builds a real exception
 * frame. Full analysis: docs/engineering/fex-bringup-plan.md (rung 0e).
 */
#define SEH_POC_U_ENABLE_VEH_PROBE 0

/* Called once, early, from user-mode process init (see wiring in ntdll-rust
 * LdrpInit, guarded to smss.exe). Probe 1 (table-based SEH dispatch) is kept
 * armed as a permanent arm64 user-mode SEH regression tripwire — like the
 * kernel's ntoskrnl/seh_poc.c. A regression makes smss print FAIL / die, which
 * the arm64 smoke catches. */
void SehPocUserRun(void)
{
    DbgPrint("[SEH-POC-U] begin user-mode fault-delivery self-test\n");
    SehPocUserRaiseTest();
#if SEH_POC_U_ENABLE_VEH_PROBE
    SehPocUserVehTest();
#else
    (void)SehPocUserVehTest;
    DbgPrint("[SEH-POC-U] VEH-on-hardware-AV probe DISABLED (kernel KEXCEPTION_FRAME wedge; see fex-bringup-plan.md)\n");
#endif
    DbgPrint("[SEH-POC-U] end\n");
}
