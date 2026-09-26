/*
 * kmsafecopy.c — fault-tolerant kernel memory copy (conformance-ratchet quest
 * M3, the "memcpy+0x40" crasher cluster fix).
 *
 * The three amd64 quarantines ObSymbolicLink / ZwWaitForMultipleObjects /
 * IoReadWrite all crash at the identical PC (memcpy+0x40): a kernel API captures
 * a caller-supplied buffer with a copy that in C sat inside _SEH2_TRY. Under
 * USE_DUMMY_PSEH (the default C compile mode on ALL arches) that handler is dead
 * code, so a hostile pointer (NULL / 0x42424242 — both below MmUserProbeAddress,
 * so a range-only probe passes them) faults the capture memcpy -> BUGCHECK 0x1E
 * instead of returning STATUS_ACCESS_VIOLATION. The Rust translations dropped
 * the SEH entirely (RtlpSafeCopyMemory is a plain RtlCopyMemory now), so no
 * fault-tolerant copy exists anywhere in the kernel. This is that primitive.
 *
 * Native clang __try/__except (NOT the DUMMY_PSEH macros): compiled with
 * -fms-extensions via the c_shims wiring in ntoskrnl-cargo/build.rs +
 * ntkrnlmp-cargo/build.rs, exactly like seh_poc.c. Headerless so pseh2.h's
 * USE_DUMMY_PSEH macro layer can't demote __try back to a dead stub. Codegen
 * emits a .pdata/.xdata scope table + __C_specific_handler references (linked
 * via sdk/lib/crt/except/amd64/ehandler.c, already in the amd64 c_shims list).
 *
 * The "calls-only" native-SEH contract (dummy-pseh-verdict.md §C): clang without
 * -fasync-exceptions does NOT guarantee coverage of a direct deref written
 * inside the __try body, but a fault one frame DOWN (inside a callee) IS caught,
 * because the return address into the __try body sits within the scope table's
 * covered range. So the actual copy is performed by an out-of-line, noinline
 * helper: the faulting load/store happens inside KmCopyRaw (a callee of the
 * __try body) -> KiPageFault -> KiDispatchException -> RtlDispatchException
 * walks up from KmCopyRaw's frame to KmProtectedCopyMemory's __try -> handler
 * -> return 0. This is the same dispatch chain seh_poc.c validates for a
 * software raise; here it also correctly pages-in a valid-but-nonresident page
 * first (the fault goes through MmAccessFault before reaching the dispatcher),
 * which a bare MmIsAddressValid pre-check could not.
 *
 * ALL THREE ARCHES are protected today, and that is a MEASURED result, not a
 * guess: KmSafeCopySelfTest() below runs on every boot and prints
 * `[KMSAFECOPY] ... (PASS)`. amd64 and arm64 pass through this file; i386 passes
 * through its own primitive (see the i386 paragraph below).
 *
 * arm64 got here the long way, and the history matters because it is the reason the
 * self-test exists at all. What follows is the record of WHY the oracle is
 * load-bearing — it is no longer the current state. The live answer is
 * `ntoskrnl-cargo/build.rs`, which pushes this file into the arm64 c_shims, plus the
 * boot oracle; if those two ever disagree with this prose, believe them.
 *
 * arm64: TRIED AND REJECTED (2026-07-12), then RECOVERED once task #10 / §9.69 fixed
 * the xdata save_next / RtlVirtualUnwind decode it actually depended on.
 *
 * It looked ready on paper the first time — the arm64
 * kernel does define the personality routine (`llvm-nm build-arm64/ntoskrnl.exe`
 * -> `T __C_specific_handler`, from rtl-except's translated arm64 handler) and
 * dummy-pseh-verdict.md §C says native __try/__except "compiles today" on arm64.
 * It compiles, and it catches a SOFTWARE raise. But it does NOT catch a HARDWARE
 * page fault in kernel mode: with the copy enabled, a probe at a genuinely-unmapped
 * VA (0x0000700000000000) did not reach the __except at all — the boot simply WEDGED
 * (HANG_TIMEOUT, no bugcheck banner, no desktop). arm64 kept the plain copy until
 * that unwind decode was fixed.
 *
 * That is the whole reason the self-test exists. Shipping the cfg without it would
 * have made safe_copy_from_user *look* protected on arm64 while actually converting a
 * hostile-pointer BUGCHECK into a silent HANG — strictly worse than the honest crash,
 * and far harder to attribute. An unproven protection is worse than a documented gap.
 *
 * Root cause of the HANG (field manual §9.69): MmAccessFault correctly declines the
 * fault (0xC0000005) and KiDispatchException IS entered with a valid kernel stack, but
 * arm64 kernel-mode RtlDispatchException never returns — it recurses/faults (first inner
 * fault is an INSTRUCTION abort, EC=0x21: it CALLs a bogus handler routine) and blows the
 * kernel stack, ending in an infinite re-entry of KiArm64VectorTable. So the bug is in
 * the arm64 virtual-unwind / __C_specific_handler path for kernel frames, NOT in Mm and
 * NOT in the (never-reached) DUMMY_PSEH silent-skip fixup. That was task #10 (field
 * manual §9.69), and it is FIXED — which is what let arm64 switch this on.
 *
 * Beware the false negative that nearly sank this: address 0x1 FAULTS on amd64 but is
 * MAPPED on arm64 (the copy silently succeeded and returned garbage). A bad-pointer
 * test that never actually faults proves nothing — assert BOTH that a valid copy
 * still succeeds AND that an invalid one is caught, or a no-op "protection" passes.
 *
 * i386: PROTECTED since 2026-07-24, but NOT by this file — it took the OTHER close
 * this comment used to merely list. Its frame-based SEH lowering is MSVC-env-only
 * (dummy-pseh-verdict.md §C) AND the kernel-mode catch still does not work there
 * (task #18), so i386 exports the same KmProtectedCopyMemory symbol/ABI from
 * ntoskrnl-rust ke/i386/safecopy.rs as a Linux-extable-style fault-fixup primitive:
 * one `rep movsb` whose address is the sole key of an exception table, consulted by
 * KiTrap0EHandler after MmAccessFault declines. No exception dispatch, no
 * registration chain, no unwinder. Field manual §9.123. i386 is therefore NOT built
 * into this file's c_shims list, and supplies its own KmSafeCopySelfTest twin so the
 * per-boot [KMSAFECOPY] oracle below covers all three arches.
 *
 * The rejected non-fix stays rejected on EVERY arch: an MmIsAddressValid pre-check
 * cannot distinguish "invalid" from "valid but paged out", so on the I/O completion
 * hot path it would spuriously fail legitimate I/O with STATUS_ACCESS_VIOLATION under
 * memory pressure. Silently degrading to a wrong answer is worse than a loud unfixed
 * gap (AGENTS.md: fail loud, never silently degrade).
 *
 * Returns 1 on success, 0 if the copy faulted (the Rust caller maps 0 ->
 * Err(STATUS_ACCESS_VIOLATION)).
 */

typedef unsigned long long KMSIZE;

void *memcpy(void *Dst, const void *Src, KMSIZE Len);

/*
 * Out-of-line so the faulting access is a CALLEE of the __try body (calls-only
 * coverage, above). noinline is load-bearing: it guarantees the CALL exists, so a
 * fault below it unwinds up into KmProtectedCopyMemory's __try scope.
 *
 * The body delegates to the real memcpy rather than open-coding a byte loop: this
 * primitive sits on the I/O hot path (every buffered-I/O completion copy-back), and
 * a hand-rolled `while (Len--) *Dst++ = *Src++;` compiled to an actual byte-at-a-time
 * loop here (verified by disassembly — clang's loop-idiom pass did NOT lower it to
 * memcpy), which would have been a real throughput regression. Faulting inside
 * memcpy is still a callee fault one frame further down, so it is still caught.
 */
__attribute__((noinline))
static void KmCopyRaw(void *Dst, const void *Src, KMSIZE Len)
{
    memcpy(Dst, Src, Len);
}

int KmProtectedCopyMemory(void *Dst, const void *Src, KMSIZE Len)
{
    __try
    {
        KmCopyRaw(Dst, Src, Len);
        return 1;
    }
    __except (1 /* EXCEPTION_EXECUTE_HANDLER */)
    {
        return 0;
    }
}

/*
 * KmProtectedInterlockedAdd — the atomic twin of KmProtectedCopyMemory.
 *
 * A read-modify-write on USER memory (win32k's IMEUI.nCntInIMEProc) cannot go
 * through the copy: capture -> mutate -> publish is three accesses, not one, and
 * silently trading the atomicity for fault-tolerance is the wrong bargain. So the
 * RMW itself is the protected access. Same calls-only rule as KmCopyRaw: the
 * faulting `lock xadd` sits in a noinline callee of the __try body.
 *
 * `int` rather than `long`: this file is headerless, and the contract is a
 * Windows LONG (32-bit) on every arch.
 *
 * Returns 1 and stores the NEW value in *Result on success, 0 if the add faulted
 * (nothing is stored).
 */
__attribute__((noinline))
static int KmXaddRaw(volatile int *Addend, int Value)
{
    return __atomic_fetch_add(Addend, Value, __ATOMIC_SEQ_CST);
}

int KmProtectedInterlockedAdd(volatile int *Addend, int Value, int *Result)
{
    __try
    {
        *Result = KmXaddRaw(Addend, Value) + Value;
        return 1;
    }
    __except (1 /* EXCEPTION_EXECUTE_HANDLER */)
    {
        return 0;
    }
}

/*
 * KmSafeCopySelfTest — a per-boot tripwire proving the HARDWARE-fault path works.
 *
 * Why this exists: seh_poc.c only ever proved that a *software* raise
 * (ExRaiseStatus) dispatches. That is NOT what safe_copy_* depends on. The thing
 * we actually rely on is a real page fault, taken inside a callee, unwinding into
 * an __except one frame up: KiPageFault -> MmAccessFault (declines) ->
 * KiDispatchException -> RtlDispatchException -> __C_specific_handler -> filter.
 * Nothing was checking that end-to-end, on any arch, on any boot.
 *
 * Two assertions, both necessary:
 *   good — a VALID copy must still succeed (a primitive that "never faults"
 *          because it silently copies nothing would otherwise pass a bad-only test)
 *   bad  — an INVALID source must be CAUGHT, not bugcheck
 *
 * Address 0x1 is deliberately chosen: unmapped, and below MmUserProbeAddress, so a
 * range-only probe would happily wave it through — exactly the shape of the hostile
 * pointer that produced the memcpy+0x40 cluster.
 *
 * Prints `[KMSAFECOPY] ... (PASS)`. A FAIL means the copy is silently unprotected;
 * a BUGCHECK here means this arch's kernel SEH cannot catch a hardware fault, and
 * the fault-tolerant copy MUST NOT be enabled on it. Failing loudly at boot is the
 * entire point — a silently-unprotected copy is how the original cluster hid.
 */
extern unsigned long DbgPrint(const char *Format, ...);

/* The hostile source address for the "bad copy" assertion. amd64 uses 0x1 (a
 * canonical low VA that page-faults). On arm64 the low page is MAPPED at early
 * boot, so 0x1 never faults there (a false FAIL that masked the real result);
 * use 0x0000700000000000 instead — genuinely unmapped on arm64, still below
 * MmUserProbeAddress so it's the same range-passes hostile-pointer shape. */
#if defined(__aarch64__) || defined(_M_ARM64)
#define KMSAFECOPY_BAD_SRC ((const void *)0x0000700000000000ULL)
#else
#define KMSAFECOPY_BAD_SRC ((const void *)0x1)
#endif

/*
 * The same two assertions for the atomic, plus the one a copy does not need: the
 * add must have landed IN MEMORY, not merely returned the right sum.
 */
static int KmSafeAtomicSelfTest(void)
{
    volatile int counter = 41;
    int result = 0, junk = 0;
    int good, bad, landed, ok;

    good   = KmProtectedInterlockedAdd(&counter, 1, &result);
    landed = (counter == 42 && result == 42);
    bad    = KmProtectedInterlockedAdd((volatile int *)KMSAFECOPY_BAD_SRC, 1, &junk);

    ok = (good == 1 && landed && bad == 0);

    DbgPrint("[KMSAFECOPY] fault-tolerant interlocked add: valid=%d landed=%d invalid_caught=%d (%s)\n",
             good, landed, (bad == 0), ok ? "PASS" : "FAIL");

    return ok;
}

int KmSafeCopySelfTest(void)
{
    volatile char dst[8];
    const char src[8] = { 'v', 'i', 'b', 'e', 'o', 's', '!', 0 };
    int good, bad, ok;

    good = KmProtectedCopyMemory((void *)dst, src, sizeof(src));
    bad  = KmProtectedCopyMemory((void *)dst, KMSAFECOPY_BAD_SRC, sizeof(src));

    ok = (good == 1 && bad == 0);

    DbgPrint("[KMSAFECOPY] fault-tolerant copy: valid=%d invalid_caught=%d (%s)\n",
             good, (bad == 0), ok ? "PASS" : "FAIL");

    return ok && KmSafeAtomicSelfTest();
}
