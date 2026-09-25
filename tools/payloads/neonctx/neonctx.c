/* neonctx — do the arm64 V registers survive an exception and NtContinue?
 *
 * Loads a distinct pattern into v0-v31, takes an access violation, and lets a vectored handler
 * resume it with EXCEPTION_CONTINUE_EXECUTION (ntdll then NtContinues the CONTEXT). Checks:
 *   seen      the handler's CONTEXT carries the pattern in V[0..31] (KeTrapFrameToContext with
 *             CONTEXT_FULL; before 2026-09-25 the kernel asked for CONTROL|INTEGER only)
 *   resumed   after the resume, v0-v30 hold the pattern again, although the handler scribbled
 *             over the real registers (KiExceptionExit reloading the trap's NEON area)
 *   edited    v31 holds the value the handler WROTE into CONTEXT.V[31] (KeContextToTrapFrame)
 *   fpcr      FPCR round-trips too (rounding mode set before the fault)
 * Verdict: "[NEONCTX] done VERDICT=PASS|FAIL". */
#include <windows.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define ROUNDS 20
static unsigned long long pattern[64], out[64];
static volatile LONG seen_ok, handled;
static const unsigned long long EDIT_LO = 0x1122334455667788ull, EDIT_HI = 0x99AABBCCDDEEFF00ull;
static volatile unsigned long long scribble_sink;

static LONG CALLBACK handler(PEXCEPTION_POINTERS ep)
{
    PCONTEXT c = ep->ContextRecord;
    int i, ok = 1;
    unsigned char junk[4096];
    if (ep->ExceptionRecord->ExceptionCode != EXCEPTION_ACCESS_VIOLATION)
        return EXCEPTION_CONTINUE_SEARCH;
    for (i = 0; i < 32; i++)
        if (c->V[i].Low != pattern[2 * i] || c->V[i].High != pattern[2 * i + 1])
            ok = 0;
    seen_ok = ok && (c->ContextFlags & CONTEXT_FLOATING_POINT) == CONTEXT_FLOATING_POINT;
    /* Scribble over the live vector registers: whatever resumes must come from the CONTEXT. */
    __asm__ volatile("movi v0.16b, #0xEE\n movi v1.16b, #0xEE\n movi v8.16b, #0xEE\n"
                     "movi v15.16b, #0xEE\n movi v30.16b, #0xEE\n movi v31.16b, #0xEE"
                     ::: "v0", "v1", "v8", "v15", "v30", "v31");
    memset(junk, 0x5A, sizeof junk);
    scribble_sink = junk[123];
    c->V[31].Low = EDIT_LO;
    c->V[31].High = EDIT_HI;
    c->Pc += 4; /* past the faulting ldr */
    handled++;
    return EXCEPTION_CONTINUE_EXECUTION;
}

static int one_round(int r, char *why, size_t cb)
{
    int i, bad = 0;
    unsigned long long fpcr_before = 0x00C00000ull, fpcr_after; /* RMode = round toward zero */
    for (i = 0; i < 64; i++)
        pattern[i] = 0x0101010101010101ull * (unsigned)(i + 1) ^ ((unsigned long long)r << 56);
    memset(out, 0, sizeof out);
    seen_ok = 0;
    __asm__ volatile(
        "msr fpcr, %3\n"
        "ldp q0,  q1,  [%1, #0]\n   ldp q2,  q3,  [%1, #32]\n  ldp q4,  q5,  [%1, #64]\n"
        "ldp q6,  q7,  [%1, #96]\n  ldp q8,  q9,  [%1, #128]\n ldp q10, q11, [%1, #160]\n"
        "ldp q12, q13, [%1, #192]\n ldp q14, q15, [%1, #224]\n ldp q16, q17, [%1, #256]\n"
        "ldp q18, q19, [%1, #288]\n ldp q20, q21, [%1, #320]\n ldp q22, q23, [%1, #352]\n"
        "ldp q24, q25, [%1, #384]\n ldp q26, q27, [%1, #416]\n ldp q28, q29, [%1, #448]\n"
        "ldp q30, q31, [%1, #480]\n"
        "mov x9, #0\n"
        "ldr x10, [x9]\n"            /* the fault; the handler resumes at the next insn */
        "stp q0,  q1,  [%2, #0]\n   stp q2,  q3,  [%2, #32]\n  stp q4,  q5,  [%2, #64]\n"
        "stp q6,  q7,  [%2, #96]\n  stp q8,  q9,  [%2, #128]\n stp q10, q11, [%2, #160]\n"
        "stp q12, q13, [%2, #192]\n stp q14, q15, [%2, #224]\n stp q16, q17, [%2, #256]\n"
        "stp q18, q19, [%2, #288]\n stp q20, q21, [%2, #320]\n stp q22, q23, [%2, #352]\n"
        "stp q24, q25, [%2, #384]\n stp q26, q27, [%2, #416]\n stp q28, q29, [%2, #448]\n"
        "stp q30, q31, [%2, #480]\n"
        "mrs %0, fpcr\n"
        "msr fpcr, xzr\n"
        : "=&r"(fpcr_after)
        : "r"(pattern), "r"(out), "r"(fpcr_before)
        : "x9", "x10", "memory",
          "v0","v1","v2","v3","v4","v5","v6","v7","v8","v9","v10","v11","v12","v13","v14","v15",
          "v16","v17","v18","v19","v20","v21","v22","v23","v24","v25","v26","v27","v28","v29",
          "v30","v31");
    __asm__ volatile("" ::: "memory");
    for (i = 0; i < 62; i++)
        if (out[i] != pattern[i]) bad++;
    snprintf(why, cb, "seen=%s resumed=%s(%d/62 words wrong) edited=%s fpcr=%s(%llx)",
             seen_ok ? "OK" : "BAD", bad ? "BAD" : "OK", bad,
             (out[62] == EDIT_LO && out[63] == EDIT_HI) ? "OK" : "BAD",
             fpcr_after == fpcr_before ? "OK" : "BAD", fpcr_after);
    return seen_ok && !bad && out[62] == EDIT_LO && out[63] == EDIT_HI && fpcr_after == fpcr_before;
}

int main(void)
{
    char why[160];
    int r, pass = 0;
    AddVectoredExceptionHandler(1, handler);
    DbgPrint("[NEONCTX] start\n");
    for (r = 0; r < ROUNDS; r++) {
        int ok = one_round(r, why, sizeof why);
        pass += ok;
        if (!ok || r == 0)
            DbgPrint("[NEONCTX] round %d: %s\n", r, why);
    }
    DbgPrint("[NEONCTX] rounds=%d pass=%d handled=%ld\n", ROUNDS, pass, (long)handled);
    DbgPrint("[NEONCTX] done VERDICT=%s\n", (pass == ROUNDS && handled == ROUNDS) ? "PASS" : "FAIL");
    return 0;
}
