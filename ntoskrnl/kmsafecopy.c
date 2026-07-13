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
 * amd64 only for now: the amd64 table-based SEH personality is the validated one
 * (seh_poc.c PASS + __C_specific_handler in the link). i386's frame-based model
 * and arm64 are not yet exercised here, so on those arches the Rust safe_copy_*
 * primitive still does a PLAIN copy_nonoverlapping and a bad user pointer still
 * bugchecks (see probe.rs) — an honest, visible gap, tracked as owed work.
 *
 * That gap is deliberately NOT papered over with an MmIsAddressValid pre-check:
 * MmIsAddressValid cannot distinguish "invalid" from "valid but paged out", so on
 * the I/O completion hot path it would spuriously fail legitimate I/O with
 * STATUS_ACCESS_VIOLATION under memory pressure. Silently degrading to a wrong
 * answer is worse than a loud unfixed gap (AGENTS.md: fail loud, never silently
 * degrade). The real close is native SEH on i386/arm64, or a Linux-extable-style
 * fault-fixup copy primitive.
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
